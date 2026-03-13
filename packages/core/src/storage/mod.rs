use thiserror::Error;

pub mod cache;
pub mod github;
pub mod sync;

pub use cache::Cache;
pub use github::GitHubStorage;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("SHA mismatch — concurrent edit detected")]
    ShaMismatch,

    #[error("vault repository not found — run first-time setup")]
    RepoNotFound,

    #[error("rate limited by GitHub — retry later")]
    RateLimited,

    #[error("insufficient GitHub scope — re-authenticate with `repo` scope")]
    InsufficientScope,

    #[error("vault parse error: {0}")]
    VaultParse(#[from] crate::crypto::vault::VaultError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("offline: {0}")]
    Offline(String),
}
