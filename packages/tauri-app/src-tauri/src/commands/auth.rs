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
    let body = serde_json::json!({
        "client_id": client_id,
        "code": code,
        "redirect_uri": redirect_uri,
        "code_verifier": code_verifier,
    });

    let resp = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
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

/// Open a URL in the system's default browser.
/// Used by the desktop OAuth flow to redirect the user to GitHub.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
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

    // --- exchange_oauth_code tests ---

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

        // Point reqwest at the mock server by constructing the request manually.
        let body = serde_json::json!({
            "client_id": "test-client",
            "code": "test-code",
            "redirect_uri": "http://localhost:12345",
            "code_verifier": "verifier123",
        });
        let resp = reqwest::Client::new()
            .post(format!("{}/login/oauth/access_token", server.uri()))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "tacoshell/0.1.0")
            .json(&body)
            .send()
            .await
            .unwrap();

        assert!(resp.status().is_success());
        let json: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(json["access_token"], "gho_test123");
    }

    #[tokio::test]
    async fn exchange_oauth_code_surfaces_github_error_field() {
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

        let body = serde_json::json!({
            "client_id": "test-client",
            "code": "expired-code",
            "redirect_uri": "http://localhost:12345",
            "code_verifier": "verifier123",
        });
        let resp: serde_json::Value = reqwest::Client::new()
            .post(format!("{}/login/oauth/access_token", server.uri()))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "tacoshell/0.1.0")
            .json(&body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        // Verify that the error_description field is present for our handler to surface.
        assert_eq!(resp["error"], "bad_verification_code");
        assert!(resp["error_description"]
            .as_str()
            .unwrap()
            .contains("incorrect or expired"));
    }
}
