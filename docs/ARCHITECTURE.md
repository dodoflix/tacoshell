# Tacoshell — System Architecture

## 1. Overview

Tacoshell is a cross-platform connection management application. The architecture enforces a strict separation of concerns across three layers:

1. **UI Layer** — React components and Zustand stores, shared across all platforms
2. **Core Engine** — Rust crate that handles connections, encryption, and storage
3. **Platform Shells** — Thin wrappers (Tauri for desktop/mobile, Vite for web) that host the UI and bridge to the core

This design achieves maximum code reuse (DRY) while allowing platform-specific optimizations where necessary.

---

## 2. Layered Architecture

```
╔══════════════════════════════════════════════════════════════════════╗
║                         CLIENT PLATFORMS                             ║
║  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  ║
║  │   Desktop App    │  │     Web App      │  │   Mobile App     │  ║
║  │   (Tauri v2)     │  │  (Vite + React)  │  │ (Tauri v2 Mobile)│  ║
║  │                  │  │                  │  │                  │  ║
║  │  Tauri IPC       │  │  WASM bridge     │  │  Tauri IPC       │  ║
║  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘  ║
╚═══════════╪════════════════════╪════════════════════╪══════════════╝
            │                    │                    │
            ▼                    ▼                    ▼
╔══════════════════════════════════════════════════════════════════════╗
║                    SHARED UI LAYER  (packages/ui)                    ║
║  ┌──────────────┐  ┌──────────────────┐  ┌──────────────────────┐  ║
║  │ Tab Manager  │  │ Connection Views  │  │   Auth / Settings    │  ║
║  │ Split / Merge│  │ Terminal  SFTP    │  │   GitHub OAuth       │  ║
║  │ DnD          │  │ K8s Dashboard     │  │   Vault Passphrase   │  ║
║  └──────────────┘  └──────────────────┘  └──────────────────────┘  ║
║  ┌──────────────────────────────────────────────────────────────┐   ║
║  │              Zustand Stores (5 independent stores)            │   ║
║  └──────────────────────────────────────────────────────────────┘   ║
╚══════════════════════════════════════════════════════════════════════╝
            │                    │                    │
            ▼                    ▼                    ▼
╔══════════════════════════════════════════════════════════════════════╗
║               CORE ENGINE  (packages/core — Rust)                    ║
║  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  ║
║  │  Connection  │  │    Crypto    │  │    GitHub Storage         │  ║
║  │  Manager     │  │    Engine    │  │    Engine                 │  ║
║  │  SSH / SFTP  │  │  AES-256-GCM │  │    (private repo CRUD)    │  ║
║  │  FTP / k8s   │  │  Argon2id    │  │    + local cache          │  ║
║  └──────────────┘  └──────────────┘  └──────────────────────────┘  ║
║  ┌──────────────┐  ┌──────────────┐                                 ║
║  │   Profile /  │  │  Protocol    │                                 ║
║  │   Credential │  │  Adapters    │                                 ║
║  │   Manager    │  │  (trait-based)│                                ║
║  └──────────────┘  └──────────────┘                                 ║
╚══════════════════════════════════════════════════════════════════════╝
```

---

## 3. Monorepo Layout

```
tacoshell/
├── packages/
│   ├── core/                    # Rust crate — the shared engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection/      # SessionManager, connection lifecycle
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ssh.rs       # SshAdapter (russh)
│   │   │   │   ├── sftp.rs      # SftpAdapter
│   │   │   │   ├── ftp.rs       # FtpAdapter (suppaftp)
│   │   │   │   └── k8s.rs       # K8sAdapter (kube-rs)
│   │   │   ├── crypto/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── kdf.rs       # Argon2id key derivation
│   │   │   │   ├── cipher.rs    # AES-256-GCM encrypt/decrypt
│   │   │   │   └── vault.rs     # Vault serialization
│   │   │   ├── storage/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── github.rs    # GitHub API client (octocrab)
│   │   │   │   ├── sync.rs      # Sync protocol, conflict resolution
│   │   │   │   └── cache.rs     # Local encrypted cache
│   │   │   └── profile/
│   │   │       ├── mod.rs
│   │   │       ├── types.rs     # ConnectionProfile, Credential types
│   │   │       └── manager.rs   # CRUD on profiles
│   │   └── Cargo.toml
│   │
│   ├── core-wasm/               # WASM build wrapper
│   │   ├── src/
│   │   │   └── lib.rs           # wasm-bindgen exports
│   │   └── Cargo.toml
│   │
│   ├── ui/                      # Shared React component library
│   │   ├── src/
│   │   │   ├── components/      # Design system atoms
│   │   │   │   ├── Button/
│   │   │   │   ├── Input/
│   │   │   │   ├── Modal/
│   │   │   │   ├── Panel/
│   │   │   │   ├── SplitPane/
│   │   │   │   ├── TabBar/
│   │   │   │   ├── StatusBadge/
│   │   │   │   └── TreeView/
│   │   │   ├── features/
│   │   │   │   ├── tabs/        # TabWorkspace, tab tree operations
│   │   │   │   ├── terminal/    # TerminalView (xterm.js wrapper)
│   │   │   │   ├── sftp-browser/# FileBrowser, TransferQueue
│   │   │   │   ├── k8s-dashboard/# PodList, LogViewer, PodExecTerminal
│   │   │   │   ├── auth/        # GitHubLoginButton, SessionGuard
│   │   │   │   ├── settings/    # ProfileEditor, VaultSyncStatus
│   │   │   │   └── sidebar/     # ConnectionSidebar, ProfileList
│   │   │   ├── hooks/           # useConnection, useTabTree, useVault...
│   │   │   ├── stores/
│   │   │   │   ├── useAuthStore.ts
│   │   │   │   ├── useVaultStore.ts
│   │   │   │   ├── useConnectionStore.ts
│   │   │   │   ├── useTabStore.ts
│   │   │   │   └── useSettingsStore.ts
│   │   │   └── types/           # Shared TypeScript types
│   │   └── package.json
│   │
│   ├── tauri-app/               # Desktop + Mobile shell
│   │   ├── src/
│   │   │   └── main.tsx         # Entry — imports from ui package
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   └── main.rs      # Tauri commands bridging to core
│   │   │   ├── tauri.conf.json
│   │   │   └── Cargo.toml
│   │   └── package.json
│   │
│   ├── web-app/                 # Web deployment shell
│   │   ├── src/
│   │   │   ├── main.tsx         # Entry — imports from ui package
│   │   │   └── wasm-bridge.ts   # Loads core-wasm, adapts API surface
│   │   └── package.json
│   │
│   └── ws-proxy/                # Optional WebSocket-to-TCP relay
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── docs/
│   ├── ARCHITECTURE.md          # This document
│   ├── SECURITY.md
│   ├── CONTRIBUTING.md
│   ├── TDD.md
│   ├── PROTOCOLS.md
│   ├── STORAGE.md
│   ├── STATE.md
│   └── ROADMAP.md
│
├── .github/
│   └── workflows/
│       ├── ci.yml               # Main CI pipeline
│       └── release.yml          # Release pipeline
│
├── tools/
│   └── scripts/                 # Build helpers
│
├── Cargo.workspace.toml
├── pnpm-workspace.yaml
└── README.md
```

---

## 4. Cross-Platform Communication

### Desktop and Mobile (Tauri IPC)

The Rust core runs as the native Tauri backend process. The React UI invokes core functions via Tauri's typed command system:

```typescript
// UI side
import { invoke } from '@tauri-apps/api/core';
const session = await invoke('connect_ssh', { profile });

// Rust side (src-tauri/src/main.rs)
#[tauri::command]
async fn connect_ssh(profile: ConnectionProfile) -> Result<SessionId, String> {
    core::connection::ssh::connect(profile).await.map_err(|e| e.to_string())
}
```

Streaming data (terminal output, file transfer progress) flows back via Tauri events:

```typescript
listen('terminal_output', (event) => appendToTerminal(event.payload));
```

### Web (WASM Bridge)

The Rust core is compiled to WebAssembly via `wasm-pack`. The `wasm-bridge.ts` module presents the same API surface as the Tauri IPC layer, so the UI code is identical across platforms.

For SSH/SFTP/FTP (raw TCP), the WASM adapter connects to the `ws-proxy` over WebSocket. Kubernetes uses HTTPS directly.

```
Browser → WebSocket → ws-proxy → TCP → Remote server
```

The proxy is stateless and proxies encrypted SSH/TLS frames. It never decrypts anything.

---

## 5. Protocol Adapter Design

All protocol adapters implement a common set of Rust traits:

```rust
pub trait ConnectionAdapter: Send + Sync {
    async fn connect(profile: &ConnectionProfile) -> Result<Session>;
    async fn disconnect(session: &mut Session) -> Result<()>;
    fn is_alive(session: &Session) -> bool;
    async fn reconnect(session: &mut Session) -> Result<()>;
}

pub trait TerminalAdapter: ConnectionAdapter {
    async fn send_input(session: &Session, data: &[u8]) -> Result<()>;
    fn on_output(session: &Session, callback: impl Fn(&[u8]) + Send + 'static) -> Result<()>;
    async fn resize(session: &Session, cols: u16, rows: u16) -> Result<()>;
}

pub trait FileTransferAdapter: ConnectionAdapter {
    async fn list_dir(session: &Session, path: &str) -> Result<Vec<FileEntry>>;
    async fn upload(session: &Session, local: &Path, remote: &str) -> Result<TransferId>;
    async fn download(session: &Session, remote: &str, local: &Path) -> Result<TransferId>;
    async fn delete(session: &Session, path: &str) -> Result<()>;
    async fn mkdir(session: &Session, path: &str) -> Result<()>;
    async fn rename(session: &Session, from: &str, to: &str) -> Result<()>;
}

pub trait KubernetesAdapter: ConnectionAdapter {
    async fn list_namespaces(session: &Session) -> Result<Vec<String>>;
    async fn list_pods(session: &Session, namespace: &str) -> Result<Vec<Pod>>;
    async fn exec(session: &Session, pod: &str, container: &str, command: &[&str]) -> Result<ExecStream>;
    async fn logs(session: &Session, pod: &str, container: &str, follow: bool) -> Result<LogStream>;
}
```

Implementations: `SshAdapter` (implements `TerminalAdapter`), `SftpAdapter` (implements `FileTransferAdapter`), `FtpAdapter` (implements `FileTransferAdapter`), `K8sAdapter` (implements `KubernetesAdapter`).

---

## 6. Key Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Core language | Rust | Compiles to native + WASM. Memory safety. Single implementation for all platforms. |
| Desktop/Mobile | Tauri v2 | Rust backend is the core engine itself — no bridge needed. 10x smaller binaries than Electron. |
| UI framework | React + TypeScript | Works in Tauri webview and standalone web. Largest ecosystem. |
| State management | Zustand | Minimal boilerplate, excellent TS support, stores are plain functions — easy to test. |
| SSH | russh | Pure Rust SSH2, compiles to WASM. |
| FTP | suppaftp | Async Rust FTP client. |
| Kubernetes | kube-rs | De facto Rust k8s client, handles kubeconfig natively. |
| Encryption | AES-256-GCM + Argon2id | Hardware-accelerated cipher. Memory-hard KDF resistant to GPU attacks. |
| Storage | Private GitHub repo | Versioning, conflict detection via SHA, proper structure, no Gist limitations. |
| Monorepo | pnpm + Cargo workspace | Simplest approach, no additional build tool dependency. |
