// Protocol adapter traits and implementations
//
// Each protocol adapter implements ConnectionAdapter plus one or more
// capability traits (TerminalAdapter, FileTransferAdapter, KubernetesAdapter).
//
// Implementations live in sub-modules:
//   ssh.rs   — SSH (implements TerminalAdapter)
//   sftp.rs  — SFTP (implements FileTransferAdapter, built on top of SSH)
//   ftp.rs   — FTP/FTPS (implements FileTransferAdapter)
//   k8s.rs   — Kubernetes (implements KubernetesAdapter)

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::profile::types::{ConnectionProfile, Protocol};

pub mod ssh;

// ---------------------------------------------------------------------------
// Credential
// ---------------------------------------------------------------------------

/// Resolved credential for a connection.
///
/// Never stored inside `ConnectionProfile`; the caller resolves the credential
/// from the vault just before calling [`ConnectionAdapter::connect`].
#[derive(Clone)]
pub enum Credential {
    /// Username/password authentication.
    Password(secrecy::SecretString),
    /// OpenSSH private key (PEM-encoded). Supply `passphrase` for encrypted keys.
    PublicKey {
        private_key_pem: secrecy::SecretString,
        passphrase: Option<secrecy::SecretString>,
    },
    /// Delegate signing to the SSH agent at `SSH_AUTH_SOCK`.
    SshAgent,
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Credential::Password(_) => write!(f, "Credential::Password([redacted])"),
            Credential::PublicKey { .. } => write!(f, "Credential::PublicKey([redacted])"),
            Credential::SshAgent => write!(f, "Credential::SshAgent"),
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed error returned by all connection-layer operations.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Connection refused: {host}:{port}")]
    Refused { host: String, port: u16 },

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Host key verification failed for {host}:{port}")]
    HostKeyMismatch { host: String, port: u16 },

    #[error("Connection timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Operation not supported by {protocol:?}")]
    NotSupported { protocol: Protocol },

    #[error("Transfer cancelled")]
    Cancelled,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// ExecResult
// ---------------------------------------------------------------------------

/// Output from a one-shot remote command execution.
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<u32>,
}

impl ExecResult {
    /// Decode `stdout` as lossy UTF-8.
    pub fn stdout_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.stdout)
    }

    /// Decode `stderr` as lossy UTF-8.
    pub fn stderr_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.stderr)
    }
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Core trait shared by all protocol adapters.
#[async_trait]
pub trait ConnectionAdapter: Send + Sync {
    /// Open a new connection to the server described by `profile`.
    async fn connect(
        profile: &ConnectionProfile,
        credential: Credential,
    ) -> Result<Self, ConnectionError>
    where
        Self: Sized;

    /// Gracefully close the connection.
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;

    /// Returns `true` while the underlying transport is alive.
    fn is_alive(&self) -> bool;

    /// Drop and re-establish the connection using the stored credential.
    async fn reconnect(&mut self) -> Result<(), ConnectionError>;

    /// The protocol this adapter implements.
    fn protocol(&self) -> Protocol;
}

/// Extended trait for adapters that provide an interactive terminal (SSH).
#[async_trait]
pub trait TerminalAdapter: ConnectionAdapter {
    /// Write raw bytes to the remote PTY / shell stdin.
    async fn send_input(&self, data: &[u8]) -> Result<(), ConnectionError>;

    /// Take the output receiver.
    ///
    /// Returns `Some` on the first call; subsequent calls return `None`
    /// (the receiver has been moved out).
    fn output_stream(&mut self) -> Option<mpsc::Receiver<Vec<u8>>>;

    /// Send a PTY window-resize notification to the server.
    async fn resize(&self, cols: u16, rows: u16) -> Result<(), ConnectionError>;

    /// Execute a one-shot command and collect its stdout / stderr / exit code.
    async fn exec(&self, command: &str) -> Result<ExecResult, ConnectionError>;
}
