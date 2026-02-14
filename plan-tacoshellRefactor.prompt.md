## Plan: Tacoshell – Unified Infrastructure Management GUI (Complete Refactor)

A comprehensive technical specification for rebuilding Tacoshell as a modular, browser-like infrastructure management tool with SSH, SFTP, and Kubernetes support. The plan prioritizes UX through a tabbed/split-screen interface, decoupled data architecture, and a Rust-native stack optimized for plugin extensibility.

---

### 1. Tech Stack & GUI Framework Recommendation

| Framework | Pros | Cons | Best For |
|-----------|------|------|----------|
| **Tauri** | Mature web ecosystem, rapid UI dev via HTML/CSS/JS, excellent for complex UIs (tabs, splits), smaller bundles than Electron | Two runtimes (Rust + WebView), IPC overhead, platform WebView quirks | Complex "Lens-like" dashboards, rapid iteration |
| **Iced** | Pure Rust, Elm-arch, good for custom widgets, no external runtime | Steeper UI complexity, fewer pre-built components, slower dev for rich UIs | Terminal-heavy apps, simpler interfaces |
| **Slint** | Declarative DSL, compiles to native, commercial backing | Less mature ecosystem, DSL learning curve | Embedded, resource-constrained |

**Recommendation: Tauri v2**  
- The "browser-like" tabbed interface with split-screens, K8s dashboards, and file browsers maps naturally to web technologies (React/Vue + xterm.js for terminals).  
- Tauri's Rust backend handles SSH/SFTP/K8s logic with full native performance where it matters.
- Frontend: **React + TypeScript** with **xterm.js** (terminal), **Monaco** (optional editor), and a layout library like **react-mosaic** (split panes).

---

### 2. Monorepo & Crate Architecture

```
tacoshell/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── tacoshell-core/     # Shared types, traits, error handling
│   ├── tacoshell-secrets/  # Secret storage, encryption, keyring integration
│   ├── tacoshell-ssh/      # SSH2 wrapper, session lifecycle
│   ├── tacoshell-transfer/ # SFTP/FTP protocol handlers
│   ├── tacoshell-k8s/      # kube-rs based cluster management
│   ├── tacoshell-db/       # SQLite + migrations (SQLx/rusqlite)
│   └── tacoshell-app/      # Tauri app: commands, state, IPC bridge
├── ui/                     # Frontend (React + TypeScript)
│   ├── src/
│   │   ├── components/     # Tabs, SplitPane, Terminal, K8sDashboard
│   │   ├── hooks/          # useTauriCommand, useSession
│   │   └── stores/         # Zustand/Redux state
│   └── package.json
├── docs/
│   └── adr/                # Architecture Decision Records
└── configs/
```

**Decoupling Strategy:**
- `tacoshell-core` defines traits (`SessionProvider`, `SecretStore`, `FileTransfer`) that crates implement.
- `tacoshell-app` is the *only* crate depending on Tauri; all business logic lives in protocol crates.
- The UI communicates via Tauri's `invoke()` commands—allowing a future CLI or alternative GUI to reuse the same crates.

---

### 3. Data Model Strategy

**Storage:** SQLite via `rusqlite` or `sqlx` (async). Encrypted at-rest using SQLCipher or Rust-native `age` encryption for the secrets table.

```rust
// crates/tacoshell-db/src/models.rs

pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub kind: SecretKind,           // Password | PrivateKey | Token
    pub encrypted_value: Vec<u8>,   // Encrypted blob
    pub created_at: DateTime<Utc>,
}

pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub tags: Vec<String>,
}

// Junction table for many-to-many
pub struct ServerSecret {
    pub server_id: Uuid,
    pub secret_id: Uuid,
    pub priority: i32,              // Allows fallback ordering
}
```

**Query pattern:** When connecting, fetch `SELECT s.* FROM secrets s JOIN server_secrets ss ON ... WHERE ss.server_id = ? ORDER BY priority`.

---

### 4. Phased Implementation Roadmap

#### Phase 1: Core Architecture & Basic SSH (Weeks 1–4)
1. Initialize Cargo workspace with `tacoshell-core`, `tacoshell-ssh`, `tacoshell-db`, `tacoshell-secrets`.
2. Implement `SshSession` trait and basic connect/auth/exec in `tacoshell-ssh`.
3. Set up SQLite schema and migrations in `tacoshell-db`.
4. Scaffold Tauri app with a single terminal view (xterm.js) calling Rust backend.
5. Define IPC contract (commands: `connect`, `disconnect`, `send_input`).

#### Phase 2: Tab & Split-Screen Window Manager (Weeks 5–7)
1. Integrate `react-mosaic` (or similar) for split-pane layout.
2. Implement tab management (add/close/reorder) with Zustand state.
3. Persist layout state to local storage.
4. Support drag-and-drop to split/merge panes.

#### Phase 3: File Transfer Protocols – SFTP (Weeks 8–10)
1. Create `tacoshell-transfer` crate wrapping `ssh2::Sftp`.
2. Build dual-pane file browser component (local ↔ remote).
3. Implement upload/download progress via Tauri event streams.
4. Add FTP support via `suppaftp` crate (optional, lower priority).

#### Phase 4: Kubernetes Integration (Weeks 11–14)
1. Add `tacoshell-k8s` crate using `kube-rs` + `k8s-openapi`.
2. Implement kubeconfig discovery and context switching.
3. Build dashboard components: namespace selector, pod list, log viewer, exec shell.
4. Stream logs via Tauri events (`watch` API).

#### Phase 5: Polish & Community Release (Weeks 15–18)
1. Add settings UI (themes, keybindings, default shell).
2. Implement connection profiles import/export (JSON).
3. Cross-platform builds (Windows, macOS, Linux) via GitHub Actions.
4. Write contribution guide, finalize ADRs, publish v0.1.0-beta.

---

### Further Considerations

1. **Secret Backend:** Use OS keyring (`keyring` crate) vs. embedded encrypted DB? *Recommend: SQLCipher for portability + optional keyring unlock.*
2. **Plugin System:** Should Phase 5 include a plugin API (e.g., WASM-based)? Adds scope but future-proofs extensibility.
3. **Terminal Multiplexing:** Native PTY handling per-session vs. a shared multiplexer (like tmux-style)? *Recommend: Per-session for isolation.*

