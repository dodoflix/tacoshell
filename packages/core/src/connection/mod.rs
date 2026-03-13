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
// Error type
// ---------------------------------------------------------------------------

/// All errors that can arise during connection lifecycle and protocol operations.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("connection refused: {host}:{port}")]
    Refused { host: String, port: u16 },

    #[error("authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("host key verification failed for {host}")]
    HostKeyMismatch { host: String },

    #[error("connection timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("operation not supported")]
    NotSupported,

    #[error("transfer cancelled")]
    Cancelled,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<russh::Error> for ConnectionError {
    fn from(e: russh::Error) -> Self {
        match e {
            russh::Error::IO(io_err) => ConnectionError::Io(io_err),
            russh::Error::ConnectionTimeout | russh::Error::InactivityTimeout => {
                // russh doesn't expose the configured timeout value in the error;
                // preserve the variant information in the message.
                ConnectionError::Protocol(e.to_string())
            }
            russh::Error::NotAuthenticated => ConnectionError::AuthFailed {
                reason: "not authenticated".to_string(),
            },
            russh::Error::KeepaliveTimeout => ConnectionError::Protocol(
                "server did not respond to keepalive".to_string(),
            ),
            other => ConnectionError::Protocol(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Result of a non-interactive command execution
// ---------------------------------------------------------------------------

/// Result returned by `TerminalAdapter::exec`.
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

// ---------------------------------------------------------------------------
// ConnectionAdapter trait
// ---------------------------------------------------------------------------

/// Implemented by every protocol adapter (SSH, SFTP, FTP, Kubernetes).
///
/// Each adapter is instantiated per-connection via `connect`. The adapter is
/// responsible for managing the full connection lifecycle.
#[async_trait]
pub trait ConnectionAdapter: Send + Sync + Sized {
    /// Connect using the supplied profile. Returns a live adapter on success.
    async fn connect(profile: &ConnectionProfile) -> Result<Self, ConnectionError>;

    /// Gracefully close the connection.
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;

    /// Returns `true` if the underlying transport is still active.
    fn is_alive(&self) -> bool;

    /// Re-establish a dropped connection using the same profile and credentials
    /// that were used at construction time.
    async fn reconnect(&mut self) -> Result<(), ConnectionError>;

    /// The protocol this adapter handles.
    fn protocol(&self) -> Protocol;
}

// ---------------------------------------------------------------------------
// TerminalAdapter trait
// ---------------------------------------------------------------------------

/// Implemented by adapters that provide interactive terminal sessions (SSH).
#[async_trait]
pub trait TerminalAdapter: ConnectionAdapter {
    /// Send raw bytes to the remote shell (user keystrokes, paste, etc.).
    async fn send_input(&self, data: &[u8]) -> Result<(), ConnectionError>;

    /// Take the output byte-stream receiver. May only be called once per connection;
    /// subsequent calls return a closed receiver.
    fn output_stream(&self) -> mpsc::Receiver<Vec<u8>>;

    /// Notify the remote pty of a terminal resize.
    async fn resize(&self, cols: u16, rows: u16) -> Result<(), ConnectionError>;

    /// Execute a single command non-interactively and return its output.
    async fn exec(&self, command: &str) -> Result<ExecResult, ConnectionError>;
}
