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

use crate::profile::types::{ConnectionProfile, Protocol};
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

pub mod ssh;

// ---------------------------------------------------------------------------
// Error Types
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Connection refused: {host}:{port}")]
    Refused { host: String, port: u16 },

    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    #[error("Host key verification failed for {host}")]
    HostKeyMismatch { host: String },

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

pub type Result<T> = std::result::Result<T, ConnectionError>;

// ---------------------------------------------------------------------------
// Common Traits
// ---------------------------------------------------------------------------

/// Implemented by all protocol adapters
#[async_trait]
pub trait ConnectionAdapter: Send + Sync {
    async fn connect(profile: &ConnectionProfile) -> Result<Self>
    where
        Self: Sized;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_alive(&self) -> bool;
    async fn reconnect(&mut self) -> Result<()>;
    fn protocol(&self) -> Protocol;
}

/// Implemented by SSH (and SSH tunnels for SFTP)
#[async_trait]
pub trait TerminalAdapter: ConnectionAdapter {
    async fn send_input(&self, data: &[u8]) -> Result<()>;
    fn output_stream(&self) -> mpsc::Receiver<Vec<u8>>;
    async fn resize(&self, cols: u16, rows: u16) -> Result<()>;
    async fn exec(&self, command: &str) -> Result<ExecResult>;
}

/// Result of executing a command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Implemented by SFTP and FTP
#[async_trait]
pub trait FileTransferAdapter: ConnectionAdapter {
    async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>>;
    async fn upload(&self, local: &Path, remote: &str) -> Result<TransferHandle>;
    async fn download(&self, remote: &str, local: &Path) -> Result<TransferHandle>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn mkdir(&self, path: &str) -> Result<()>;
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
    async fn stat(&self, path: &str) -> Result<FileEntry>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub permissions: u32,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub owner: String,
    pub group: String,
}

pub type TransferId = String;

pub struct TransferHandle {
    pub id: TransferId,
    pub progress: tokio::sync::watch::Receiver<TransferProgress>,
    pub cancel: tokio_util::sync::CancellationToken,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64, // 0 if unknown
    pub speed_bps: f64,   // bytes per second (rolling average)
    pub eta: Option<Duration>,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// Implemented by the Kubernetes adapter
#[async_trait]
pub trait KubernetesAdapter: ConnectionAdapter {
    async fn list_namespaces(&self) -> Result<Vec<String>>;
    async fn list_pods(&self, namespace: &str) -> Result<Vec<PodInfo>>;
    async fn exec_pod(
        &self,
        pod: &str,
        container: &str,
        command: &[&str],
    ) -> Result<ExecStream>;
    async fn pod_logs(&self, pod: &str, container: &str, follow: bool) -> Result<LogStream>;
    async fn apply_manifest(&self, yaml: &str) -> Result<()>;
    async fn delete_resource(&self, kind: &str, name: &str, namespace: &str) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub containers: Vec<String>,
}

pub struct ExecStream {
    pub stdout: mpsc::Receiver<Vec<u8>>,
    pub stderr: mpsc::Receiver<Vec<u8>>,
}

pub struct LogStream {
    pub logs: mpsc::Receiver<String>,
}
