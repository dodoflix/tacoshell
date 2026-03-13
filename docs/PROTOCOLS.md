# Tacoshell — Protocol Adapter Specifications

## 1. Design Principles

- Every protocol adapter implements a common set of Rust traits
- Adapters are instantiated per-connection and are `Send + Sync`
- All async operations use Tokio as the async runtime
- Errors are typed (`thiserror`-derived enums), never boxed `dyn Error` at API boundaries
- Adapters emit progress/output via channels (mpsc), not callbacks, for cleaner async ergonomics

---

## 2. Common Traits

```rust
/// Implemented by all protocol adapters
#[async_trait]
pub trait ConnectionAdapter: Send + Sync {
    async fn connect(profile: &ConnectionProfile) -> Result<Self, ConnectionError>
    where Self: Sized;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    fn is_alive(&self) -> bool;
    async fn reconnect(&mut self) -> Result<(), ConnectionError>;
    fn protocol(&self) -> Protocol;
}

/// Implemented by SSH (and SSH tunnels for SFTP)
#[async_trait]
pub trait TerminalAdapter: ConnectionAdapter {
    async fn send_input(&self, data: &[u8]) -> Result<(), ConnectionError>;
    fn output_stream(&self) -> mpsc::Receiver<Vec<u8>>;
    async fn resize(&self, cols: u16, rows: u16) -> Result<(), ConnectionError>;
    async fn exec(&self, command: &str) -> Result<ExecResult, ConnectionError>;
}

/// Implemented by SFTP and FTP
#[async_trait]
pub trait FileTransferAdapter: ConnectionAdapter {
    async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>, ConnectionError>;
    async fn upload(&self, local: &Path, remote: &str) -> Result<TransferHandle, ConnectionError>;
    async fn download(&self, remote: &str, local: &Path) -> Result<TransferHandle, ConnectionError>;
    async fn delete(&self, path: &str) -> Result<(), ConnectionError>;
    async fn mkdir(&self, path: &str) -> Result<(), ConnectionError>;
    async fn rename(&self, from: &str, to: &str) -> Result<(), ConnectionError>;
    async fn stat(&self, path: &str) -> Result<FileEntry, ConnectionError>;
}

/// Implemented by the Kubernetes adapter
#[async_trait]
pub trait KubernetesAdapter: ConnectionAdapter {
    async fn list_namespaces(&self) -> Result<Vec<String>, ConnectionError>;
    async fn list_pods(&self, namespace: &str) -> Result<Vec<PodInfo>, ConnectionError>;
    async fn exec_pod(&self, pod: &str, container: &str, command: &[&str])
        -> Result<ExecStream, ConnectionError>;
    async fn pod_logs(&self, pod: &str, container: &str, follow: bool)
        -> Result<LogStream, ConnectionError>;
    async fn apply_manifest(&self, yaml: &str) -> Result<(), ConnectionError>;
    async fn delete_resource(&self, kind: &str, name: &str, namespace: &str)
        -> Result<(), ConnectionError>;
}
```

---

## 3. SSH Adapter

**Library**: `russh` (pure Rust SSH2 implementation)

**Authentication methods supported**:
- Password
- Public key (RSA, Ed25519, ECDSA)
- SSH agent forwarding (desktop/mobile only)
- Keyboard-interactive

**Key features**:
- Multiplexed channels over a single TCP connection (multiple shell/exec sessions)
- Terminal keepalive via SSH keepalive messages (configurable interval)
- Port forwarding (local, remote, dynamic SOCKS5)
- SFTP subsystem negotiation (used by `SftpAdapter`)

**Connection profile fields**:
```rust
pub struct SshProfile {
    pub host: String,
    pub port: u16,                    // default: 22
    pub username: String,
    pub auth: SshAuth,
    pub keepalive_interval: Option<Duration>,
    pub connect_timeout: Duration,
    pub host_key_verification: HostKeyPolicy,
    pub jump_hosts: Vec<SshProfile>,  // ProxyJump support
}

pub enum SshAuth {
    Password(SecretString),
    PublicKey { key_id: VaultItemId },
    Agent,
    KeyboardInteractive,
}

pub enum HostKeyPolicy {
    StrictFirstConnect,   // Trust on first use, reject changes
    AcceptAll,            // Not recommended; user must opt in
    KnownHosts(PathBuf),  // Use a custom known_hosts file
}
```

**Web platform**: SSH frames are forwarded over WebSocket to `ws-proxy`. The SSH handshake and all encryption happen in WASM, end-to-end.

---

## 4. SFTP Adapter

**Library**: `russh-sftp` (companion crate to russh)

**Built on top of**: `SshAdapter` — negotiates the SFTP subsystem on an SSH channel

**Supported operations**:
- Directory listing with file metadata (size, permissions, timestamps, symlink targets)
- File upload and download with progress tracking
- In-place file editing (download → edit → upload)
- Recursive directory operations (upload/download entire directories)
- Permission management (`chmod`)
- Symbolic link creation

**File entry type**:
```rust
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub permissions: u32,
    pub modified_at: DateTime<Utc>,
    pub owner: String,
    pub group: String,
}
```

**Transfer handle** (for progress tracking):
```rust
pub struct TransferHandle {
    pub id: TransferId,
    pub progress: watch::Receiver<TransferProgress>,
    pub cancel: CancellationToken,
}
```

---

## 5. FTP Adapter

**Library**: `suppaftp` (async Rust FTP/FTPS client)

**Variants supported**:
- Plain FTP (port 21) — not recommended; clearly marked as insecure in UI
- FTPS (explicit TLS, port 21 with STARTTLS)
- FTPS (implicit TLS, port 990)

**Connection profile fields**:
```rust
pub struct FtpProfile {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: SecretString,
    pub mode: FtpMode,
    pub passive_mode: bool,   // default: true
    pub connect_timeout: Duration,
}

pub enum FtpMode {
    Plain,
    ExplicitTls,
    ImplicitTls,
}
```

**Limitations vs SFTP** (displayed in UI):
- No permission management
- No symlink support
- File modification times may be read-only depending on server

---

## 6. Kubernetes Adapter

**Library**: `kube-rs` (de facto Rust k8s client)

**Authentication methods**:
- kubeconfig file (parsed from vault — stores only the relevant cluster/context/auth triples)
- Bearer token (service account tokens)
- Client certificate

**Supported resources**:
- Namespaces (list)
- Pods (list, describe, exec, logs, delete)
- Deployments (list, describe, scale, restart)
- Services (list, describe)
- ConfigMaps and Secrets (list names only; Secret values shown only on explicit request)
- Nodes (list, describe)
- Events (list per namespace)
- Custom Resource Definitions (list)

**kubeconfig vault item**:
```rust
pub struct KubeConfigItem {
    pub display_name: String,
    pub cluster_name: String,
    pub server: String,
    pub ca_cert: Option<String>,       // base64 PEM
    pub auth: KubeAuth,
    pub namespace: Option<String>,     // default namespace
}

pub enum KubeAuth {
    Token(SecretString),
    ClientCert { cert: String, key: SecretString },
    ExecCredential { command: String, args: Vec<String> }, // e.g., aws eks get-token
}
```

---

## 7. Transfer Progress Model

File transfers (SFTP and FTP) use a shared progress model:

```rust
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,      // 0 if unknown
    pub speed_bps: f64,        // bytes per second (rolling average)
    pub eta: Option<Duration>,
    pub status: TransferStatus,
}

pub enum TransferStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}
```

Multiple transfers can be queued and run concurrently (configurable max concurrency, default: 3).

---

## 8. Error Types

```rust
#[derive(Debug, thiserror::Error)]
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

    #[error("Operation not supported by {protocol}")]
    NotSupported { protocol: Protocol },

    #[error("Transfer cancelled")]
    Cancelled,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

---

## 9. Web Platform Adaptation

For the web platform, raw TCP is unavailable. The adaptation layer:

| Protocol | Web Approach |
|----------|-------------|
| SSH | WASM SshAdapter sends raw SSH frames over WebSocket to `ws-proxy` |
| SFTP | Same WebSocket path as SSH (SFTP runs inside an SSH channel) |
| FTP | WASM FtpAdapter sends FTP commands over WebSocket to `ws-proxy` |
| Kubernetes | Direct HTTPS from browser (k8s API is HTTP-based, no proxy needed) |

The `wasm-bridge.ts` module injects the WebSocket transport layer into the Rust core before initialization:

```typescript
import init, { set_transport_config } from '../core-wasm/pkg'

await init()
set_transport_config({
  proxy_url: 'wss://proxy.tacoshell.dev',  // or self-hosted
  proxy_token: await getProxyAuthToken(),
})
```
