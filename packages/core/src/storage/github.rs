use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::storage::StorageError;

// ---------------------------------------------------------------------------
// Response / request shapes for the GitHub Contents API
// ---------------------------------------------------------------------------

/// JSON shape returned by `GET /repos/{owner}/{repo}/contents/{path}`.
#[derive(Debug, Deserialize)]
struct ContentsResponse {
    sha: String,
    /// Base64-encoded file content, possibly split across multiple lines.
    content: String,
}

/// JSON shape returned by `PUT /repos/{owner}/{repo}/contents/{path}` (200 OK).
#[derive(Debug, Deserialize)]
struct PutFileResponse {
    content: PutFileContent,
}

#[derive(Debug, Deserialize)]
struct PutFileContent {
    sha: String,
}

/// Body sent for `PUT /repos/{owner}/{repo}/contents/{path}`.
#[derive(Debug, Serialize)]
struct PutFileRequest<'a> {
    message: &'a str,
    /// Base64-encoded file content.
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<&'a str>,
}

/// Body sent for `POST /user/repos` (create repository).
#[derive(Debug, Serialize)]
struct CreateRepoRequest<'a> {
    name: &'a str,
    private: bool,
    auto_init: bool,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Raw file content and its GitHub blob SHA.
#[derive(Debug, Clone)]
pub struct FileContent {
    /// Decoded file bytes (NOT base64).
    pub content: Vec<u8>,
    /// GitHub blob SHA — used as an optimistic-concurrency token for writes.
    pub sha: String,
}

// ---------------------------------------------------------------------------
// Trait definition
// ---------------------------------------------------------------------------

/// Capability required for reading/writing vault files on GitHub.
///
/// This trait is thin: it operates on raw bytes and returns the raw GitHub
/// blob SHA. Higher-level encoding/decryption happens in `SyncEngine`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait GitHubStorage: Send + Sync {
    /// Returns `true` if the private `tacoshell-vault` repository exists.
    async fn repo_exists(&self, owner: &str) -> Result<bool, StorageError>;

    /// Creates a private repository named `tacoshell-vault` in the
    /// authenticated user's account. No-ops if the repo already exists.
    async fn create_vault_repo(&self, owner: &str) -> Result<(), StorageError>;

    /// Reads the file at `path` inside the vault repository.
    ///
    /// Returns `None` when the file does not exist (404).
    async fn read_file(&self, owner: &str, path: &str)
        -> Result<Option<FileContent>, StorageError>;

    /// Creates a new file at `path` (no prior SHA required).
    ///
    /// Returns the new blob SHA.
    async fn create_file(
        &self,
        owner: &str,
        path: &str,
        content: &[u8],
        message: &str,
    ) -> Result<String, StorageError>;

    /// Updates the file at `path` using optimistic concurrency.
    ///
    /// `sha` must match the current blob SHA. Returns `ShaMismatch` on 422.
    /// Returns the new blob SHA on success.
    async fn write_file(
        &self,
        owner: &str,
        path: &str,
        content: &[u8],
        sha: &str,
        message: &str,
    ) -> Result<String, StorageError>;
}

// ---------------------------------------------------------------------------
// Production implementation
// ---------------------------------------------------------------------------

pub struct GitHubClient {
    inner: octocrab::Octocrab,
    /// The name of the vault repository (always "tacoshell-vault").
    vault_repo: &'static str,
}

impl GitHubClient {
    const VAULT_REPO: &'static str = "tacoshell-vault";

    /// Build a client from a personal access token.
    pub fn new(token: &str) -> Result<Self, StorageError> {
        let inner = octocrab::OctocrabBuilder::new()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| StorageError::Auth(e.to_string()))?;
        Ok(GitHubClient {
            inner,
            vault_repo: Self::VAULT_REPO,
        })
    }

    /// Build a client pointing at a custom base URL (for wiremock tests).
    #[cfg(test)]
    pub fn with_base_url(token: &str, base_url: &str) -> Result<Self, StorageError> {
        let inner = octocrab::OctocrabBuilder::new()
            .personal_token(token.to_string())
            .base_uri(base_url)
            .map_err(|e: octocrab::Error| StorageError::Auth(e.to_string()))?
            .build()
            .map_err(|e: octocrab::Error| StorageError::Auth(e.to_string()))?;
        Ok(GitHubClient {
            inner,
            vault_repo: Self::VAULT_REPO,
        })
    }

    fn repo(&self) -> &'static str {
        self.vault_repo
    }

    /// Classify an octocrab error into a typed `StorageError`.
    fn classify(e: octocrab::Error) -> StorageError {
        if let octocrab::Error::GitHub { source, .. } = &e {
            return match source.status_code.as_u16() {
                404 => StorageError::RepoNotFound,
                422 => StorageError::ShaMismatch,
                403 => StorageError::InsufficientScope,
                429 => StorageError::RateLimited,
                _ => StorageError::GitHub(source.message.clone()),
            };
        }
        StorageError::GitHub(e.to_string())
    }
}

#[async_trait]
impl GitHubStorage for GitHubClient {
    #[instrument(skip(self), fields(owner = %owner))]
    async fn repo_exists(&self, owner: &str) -> Result<bool, StorageError> {
        let route = format!("/repos/{}/{}", owner, self.repo());
        let result: Result<serde_json::Value, _> = self.inner.get(route, None::<&()>).await;
        match result {
            Ok(_) => Ok(true),
            Err(ref e) => {
                if let octocrab::Error::GitHub { source, .. } = e {
                    if source.status_code.as_u16() == 404 {
                        return Ok(false);
                    }
                }
                Err(Self::classify(result.unwrap_err()))
            }
        }
    }

    #[instrument(skip(self), fields(owner = %owner))]
    async fn create_vault_repo(&self, owner: &str) -> Result<(), StorageError> {
        let body = CreateRepoRequest {
            name: self.repo(),
            private: true,
            auto_init: false,
        };
        let _: serde_json::Value = self
            .inner
            .post("/user/repos", Some(&body))
            .await
            .map_err(Self::classify)?;
        Ok(())
    }

    #[instrument(skip(self), fields(owner = %owner, path = %path))]
    async fn read_file(
        &self,
        owner: &str,
        path: &str,
    ) -> Result<Option<FileContent>, StorageError> {
        let route = format!("/repos/{}/{}/contents/{}", owner, self.repo(), path);
        let result: Result<ContentsResponse, _> = self.inner.get(route, None::<&()>).await;
        match result {
            Ok(resp) => {
                // GitHub encodes content in base64 with embedded newlines.
                let raw = resp.content.replace('\n', "");
                let bytes = BASE64.decode(raw)?;
                Ok(Some(FileContent {
                    content: bytes,
                    sha: resp.sha,
                }))
            }
            Err(ref e) => {
                if let octocrab::Error::GitHub { source, .. } = e {
                    if source.status_code.as_u16() == 404 {
                        return Ok(None);
                    }
                }
                Err(Self::classify(result.unwrap_err()))
            }
        }
    }

    #[instrument(skip(self, content), fields(owner = %owner, path = %path))]
    async fn create_file(
        &self,
        owner: &str,
        path: &str,
        content: &[u8],
        message: &str,
    ) -> Result<String, StorageError> {
        let body = PutFileRequest {
            message,
            content: BASE64.encode(content),
            sha: None,
        };
        let route = format!("/repos/{}/{}/contents/{}", owner, self.repo(), path);
        let resp: PutFileResponse = self
            .inner
            .put(route, Some(&body))
            .await
            .map_err(Self::classify)?;
        Ok(resp.content.sha)
    }

    #[instrument(skip(self, content), fields(owner = %owner, path = %path))]
    async fn write_file(
        &self,
        owner: &str,
        path: &str,
        content: &[u8],
        sha: &str,
        message: &str,
    ) -> Result<String, StorageError> {
        let body = PutFileRequest {
            message,
            content: BASE64.encode(content),
            sha: Some(sha),
        };
        let route = format!("/repos/{}/{}/contents/{}", owner, self.repo(), path);
        let result: Result<PutFileResponse, _> = self.inner.put(route, Some(&body)).await;
        match result {
            Ok(resp) => Ok(resp.content.sha),
            Err(ref e) => {
                if let octocrab::Error::GitHub { source, .. } = e {
                    if source.status_code.as_u16() == 422 {
                        return Err(StorageError::ShaMismatch);
                    }
                    if source.status_code.as_u16() == 429 {
                        return Err(StorageError::RateLimited);
                    }
                }
                Err(Self::classify(result.unwrap_err()))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    const OWNER: &str = "test-owner";
    const TOKEN: &str = "test-token";

    async fn make_client(server: &MockServer) -> GitHubClient {
        GitHubClient::with_base_url(TOKEN, &server.uri()).unwrap()
    }

    fn vault_json() -> Vec<u8> {
        br#"{"schema_version":"1","items":[]}"#.to_vec()
    }

    fn file_response(content: &[u8], sha: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "file",
            "name": "vault.json",
            "path": "vault.json",
            "sha": sha,
            "size": content.len(),
            "encoding": "base64",
            // GitHub adds a newline every 60 base64 chars; we just use plain here.
            "content": BASE64.encode(content),
        })
    }

    fn not_found_response() -> serde_json::Value {
        serde_json::json!({
            "message": "Not Found",
            "documentation_url": "https://docs.github.com/rest"
        })
    }

    fn sha_mismatch_response() -> serde_json::Value {
        serde_json::json!({
            "message": "422 Unprocessable Entity",
            "errors": [{"resource": "Commit", "code": "custom", "message": "sha does not match"}],
            "documentation_url": "https://docs.github.com/rest"
        })
    }

    fn put_ok_response(new_sha: &str) -> serde_json::Value {
        serde_json::json!({
            "content": { "sha": new_sha, "name": "vault.json", "path": "vault.json" },
            "commit": { "sha": "commit-sha" }
        })
    }

    // --- repo_exists ---

    #[tokio::test]
    async fn repo_exists_returns_true_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!("/repos/{}/tacoshell-vault", OWNER)))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 1})))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        assert!(client.repo_exists(OWNER).await.unwrap());
    }

    #[tokio::test]
    async fn repo_exists_returns_false_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!("/repos/{}/tacoshell-vault", OWNER)))
            .respond_with(ResponseTemplate::new(404).set_body_json(not_found_response()))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        assert!(!client.repo_exists(OWNER).await.unwrap());
    }

    // --- read_file ---

    #[tokio::test]
    async fn read_file_decodes_content_and_returns_sha() {
        let server = MockServer::start().await;
        let content = vault_json();
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(file_response(&content, "sha-abc")),
            )
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let result = client.read_file(OWNER, "vault.json").await.unwrap();
        let file = result.expect("expected Some(FileContent)");
        assert_eq!(file.sha, "sha-abc");
        assert_eq!(file.content, content);
    }

    #[tokio::test]
    async fn read_file_returns_none_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(404).set_body_json(not_found_response()))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let result = client.read_file(OWNER, "vault.json").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn read_file_strips_newlines_from_base64_content() {
        let server = MockServer::start().await;
        let content = b"hello world vault data";
        // Simulate GitHub's newline-embedded base64.
        let b64_with_nl = {
            let raw = BASE64.encode(content);
            // Insert newline after every 4 chars to simulate GitHub's format.
            raw.chars()
                .collect::<Vec<_>>()
                .chunks(4)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "type": "file",
                "name": "vault.json",
                "path": "vault.json",
                "sha": "sha-nl",
                "size": content.len(),
                "encoding": "base64",
                "content": b64_with_nl,
            })))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let file = client
            .read_file(OWNER, "vault.json")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(file.content, content);
    }

    // --- create_file ---

    #[tokio::test]
    async fn create_file_returns_new_sha() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(201).set_body_json(put_ok_response("new-sha-1")))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let sha = client
            .create_file(OWNER, "vault.json", b"data", "initial commit")
            .await
            .unwrap();
        assert_eq!(sha, "new-sha-1");
    }

    // --- write_file ---

    #[tokio::test]
    async fn write_file_returns_new_sha_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(put_ok_response("sha-v2")))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let sha = client
            .write_file(
                OWNER,
                "vault.json",
                b"updated",
                "old-sha",
                "tacoshell: sync",
            )
            .await
            .unwrap();
        assert_eq!(sha, "sha-v2");
    }

    #[tokio::test]
    async fn write_file_returns_sha_mismatch_on_422() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(422).set_body_json(sha_mismatch_response()))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let err = client
            .write_file(OWNER, "vault.json", b"data", "stale-sha", "msg")
            .await
            .unwrap_err();
        assert!(
            matches!(err, StorageError::ShaMismatch),
            "expected ShaMismatch, got {err}"
        );
    }

    #[tokio::test]
    async fn write_file_returns_rate_limited_on_429() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(format!(
                "/repos/{}/tacoshell-vault/contents/vault.json",
                OWNER
            )))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "message": "API rate limit exceeded"
            })))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        let err = client
            .write_file(OWNER, "vault.json", b"data", "sha", "msg")
            .await
            .unwrap_err();
        assert!(
            matches!(err, StorageError::RateLimited),
            "expected RateLimited, got {err}"
        );
    }

    // --- create_vault_repo ---

    #[tokio::test]
    async fn create_vault_repo_posts_to_user_repos() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/user/repos"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 42,
                "name": "tacoshell-vault",
                "private": true
            })))
            .mount(&server)
            .await;

        let client = make_client(&server).await;
        client.create_vault_repo(OWNER).await.unwrap();
        // The test passes if no error is returned and the mock was hit.
    }
}
