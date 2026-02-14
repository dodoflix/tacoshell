//! Core traits defining interfaces for Tacoshell components
//!
//! These traits enable decoupling between business logic and implementations,
//! allowing for easy testing and potential UI swaps.

use crate::{Result, Secret, Server, ServerSecret};
use std::future::Future;
use uuid::Uuid;

/// Trait for secret storage backends
pub trait SecretStore: Send + Sync {
    /// Store a new secret
    fn store(&self, secret: &Secret) -> impl Future<Output = Result<()>> + Send;

    /// Retrieve a secret by ID
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Option<Secret>>> + Send;

    /// List all secrets
    fn list(&self) -> impl Future<Output = Result<Vec<Secret>>> + Send;

    /// Delete a secret by ID
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;

    /// Update an existing secret
    fn update(&self, secret: &Secret) -> impl Future<Output = Result<()>> + Send;
}

/// Trait for server configuration storage
pub trait ServerStore: Send + Sync {
    /// Store a new server
    fn store(&self, server: &Server) -> impl Future<Output = Result<()>> + Send;

    /// Retrieve a server by ID
    fn get(&self, id: Uuid) -> impl Future<Output = Result<Option<Server>>> + Send;

    /// List all servers
    fn list(&self) -> impl Future<Output = Result<Vec<Server>>> + Send;

    /// Delete a server by ID
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<()>> + Send;

    /// Update an existing server
    fn update(&self, server: &Server) -> impl Future<Output = Result<()>> + Send;

    /// Get secrets linked to a server, ordered by priority
    fn get_secrets(&self, server_id: Uuid) -> impl Future<Output = Result<Vec<Secret>>> + Send;

    /// Link a secret to a server
    fn link_secret(&self, link: &ServerSecret) -> impl Future<Output = Result<()>> + Send;

    /// Unlink a secret from a server
    fn unlink_secret(
        &self,
        server_id: Uuid,
        secret_id: Uuid,
    ) -> impl Future<Output = Result<()>> + Send;
}

/// Authentication credentials for connecting to a server
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Password authentication
    Password(String),
    /// Private key authentication with optional passphrase
    PrivateKey { key: String, passphrase: Option<String> },
    /// SSH agent authentication
    Agent,
}

/// Trait for session providers (SSH, etc.)
pub trait SessionProvider: Send + Sync {
    /// Session handle type
    type Session: Send;

    /// Connect to a server with the given authentication
    fn connect(
        &self,
        server: &Server,
        auth: AuthMethod,
    ) -> impl Future<Output = Result<Self::Session>> + Send;

    /// Disconnect a session
    fn disconnect(&self, session: Self::Session) -> impl Future<Output = Result<()>> + Send;

    /// Check if a session is still alive
    fn is_connected(&self, session: &Self::Session) -> bool;
}

/// Trait for file transfer operations
pub trait FileTransfer: Send + Sync {
    /// List directory contents
    fn list_dir(&self, path: &str) -> impl Future<Output = Result<Vec<FileEntry>>> + Send;

    /// Download a file
    fn download(
        &self,
        remote_path: &str,
        local_path: &str,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Upload a file
    fn upload(
        &self,
        local_path: &str,
        remote_path: &str,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Create a directory
    fn mkdir(&self, path: &str) -> impl Future<Output = Result<()>> + Send;

    /// Remove a file or directory
    fn remove(&self, path: &str) -> impl Future<Output = Result<()>> + Send;
}

/// File entry information for directory listings
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub permissions: Option<u32>,
}

/// Callback for progress reporting during transfers
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

