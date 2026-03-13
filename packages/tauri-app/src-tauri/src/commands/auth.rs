use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

/// Fetch the authenticated GitHub user's profile.
/// The token is passed from the frontend (stored in useAuthStore).
#[tauri::command]
pub async fn get_user_profile(token: String) -> Result<UserProfile, String> {
    let resp = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "tacoshell/0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }

    resp.json::<UserProfile>().await.map_err(|e| e.to_string())
}

/// Core logic for exchanging an OAuth code for a token.
/// `token_endpoint` is the full URL of the token endpoint — parameterised so tests can
/// point it at a local mock server instead of the live GitHub API.
async fn exchange_oauth_code_impl(
    token_endpoint: &str,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<String, String> {
    let body = serde_json::json!({
        "client_id": client_id,
        "code": code,
        "redirect_uri": redirect_uri,
        "code_verifier": code_verifier,
    });

    let resp = reqwest::Client::new()
        .post(token_endpoint)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("User-Agent", "tacoshell/0.1.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub token exchange error: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        let description = json
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or(error);
        return Err(description.to_string());
    }

    json.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Missing access_token in GitHub response".to_string())
}

/// Exchange a GitHub OAuth authorization code for an access token using PKCE.
///
/// GitHub supports PKCE for OAuth Apps, allowing the code exchange without
/// a client_secret when a code_verifier is provided.
#[tauri::command]
pub async fn exchange_oauth_code(
    client_id: String,
    code: String,
    redirect_uri: String,
    code_verifier: String,
) -> Result<String, String> {
    exchange_oauth_code_impl(
        "https://github.com/login/oauth/access_token",
        &client_id,
        &code,
        &redirect_uri,
        &code_verifier,
    )
    .await
}

/// Allowed URL schemes and hosts for `open_url`.
/// Only HTTPS GitHub OAuth authorization URLs are permitted.
fn validate_open_url(url: &str) -> Result<(), String> {
    let parsed = url
        .parse::<url::Url>()
        .map_err(|e| format!("Invalid URL: {e}"))?;

    if parsed.scheme() != "https" {
        return Err(format!(
            "Disallowed URL scheme '{}': only https is permitted",
            parsed.scheme()
        ));
    }

    match parsed.host_str() {
        Some("github.com") => Ok(()),
        Some(host) => Err(format!(
            "Disallowed host '{host}': only github.com is permitted"
        )),
        None => Err("URL has no host".to_string()),
    }
}

/// Open a URL in the system's default browser.
/// Only HTTPS URLs to github.com are allowed to prevent open-redirect abuse.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    validate_open_url(&url)?;
    open::that(&url).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_profile_serializes_correctly() {
        let profile = UserProfile {
            login: "testuser".to_string(),
            name: Some("Test User".to_string()),
            avatar_url: "https://avatars.githubusercontent.com/u/1".to_string(),
        };
        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(json["login"], "testuser");
        assert_eq!(json["name"], "Test User");
        assert_eq!(
            json["avatar_url"],
            "https://avatars.githubusercontent.com/u/1"
        );
    }

    #[test]
    fn user_profile_serializes_null_name() {
        let profile = UserProfile {
            login: "testuser".to_string(),
            name: None,
            avatar_url: "https://avatars.githubusercontent.com/u/1".to_string(),
        };
        let json = serde_json::to_value(&profile).unwrap();
        assert!(json["name"].is_null());
    }

    // --- validate_open_url tests ---

    #[test]
    fn open_url_allows_https_github_com() {
        assert!(
            validate_open_url("https://github.com/login/oauth/authorize?client_id=x&state=y")
                .is_ok()
        );
    }

    #[test]
    fn open_url_rejects_non_https_scheme() {
        let err = validate_open_url("http://github.com/login/oauth/authorize").unwrap_err();
        assert!(err.contains("https"), "expected scheme error, got: {err}");
    }

    #[test]
    fn open_url_rejects_file_scheme() {
        let err = validate_open_url("file:///etc/passwd").unwrap_err();
        assert!(err.contains("https"), "expected scheme error, got: {err}");
    }

    #[test]
    fn open_url_rejects_non_github_host() {
        let err = validate_open_url("https://evil.example.com/path").unwrap_err();
        assert!(
            err.contains("github.com"),
            "expected host error, got: {err}"
        );
    }

    #[test]
    fn open_url_rejects_github_subdomain_lookalike() {
        let err = validate_open_url("https://github.com.evil.example.com/path").unwrap_err();
        assert!(
            err.contains("github.com"),
            "expected host error, got: {err}"
        );
    }

    #[test]
    fn open_url_rejects_malformed_url() {
        assert!(validate_open_url("not a url").is_err());
    }

    // --- exchange_oauth_code_impl tests (test the real function via injectable endpoint) ---

    #[tokio::test]
    async fn exchange_oauth_code_returns_access_token_on_success() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "access_token": "gho_test123" })),
            )
            .mount(&server)
            .await;

        let endpoint = format!("{}/login/oauth/access_token", server.uri());
        let result = exchange_oauth_code_impl(
            &endpoint,
            "test-client",
            "test-code",
            "http://localhost:12345",
            "verifier123",
        )
        .await;

        assert_eq!(result.unwrap(), "gho_test123");
    }

    #[tokio::test]
    async fn exchange_oauth_code_surfaces_github_error_description() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "error": "bad_verification_code",
                "error_description": "The code passed is incorrect or expired.",
            })))
            .mount(&server)
            .await;

        let endpoint = format!("{}/login/oauth/access_token", server.uri());
        let result = exchange_oauth_code_impl(
            &endpoint,
            "test-client",
            "expired-code",
            "http://localhost:12345",
            "verifier123",
        )
        .await;

        let err = result.unwrap_err();
        assert!(
            err.contains("incorrect or expired"),
            "expected error_description, got: {err}"
        );
    }

    #[tokio::test]
    async fn exchange_oauth_code_errors_on_http_failure() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let endpoint = format!("{}/login/oauth/access_token", server.uri());
        let result = exchange_oauth_code_impl(
            &endpoint,
            "test-client",
            "test-code",
            "http://localhost:12345",
            "verifier123",
        )
        .await;

        let err = result.unwrap_err();
        assert!(err.contains("500"), "expected HTTP 500 error, got: {err}");
    }

    #[tokio::test]
    async fn exchange_oauth_code_errors_when_access_token_missing() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login/oauth/access_token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "token_type": "bearer" })),
            )
            .mount(&server)
            .await;

        let endpoint = format!("{}/login/oauth/access_token", server.uri());
        let result = exchange_oauth_code_impl(
            &endpoint,
            "test-client",
            "test-code",
            "http://localhost:12345",
            "verifier123",
        )
        .await;

        let err = result.unwrap_err();
        assert!(
            err.contains("Missing access_token"),
            "expected missing token error, got: {err}"
        );
    }
}
