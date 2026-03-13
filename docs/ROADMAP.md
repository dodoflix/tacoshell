# Tacoshell — Roadmap

## Delivery Philosophy

Each phase produces a working, testable increment — not a partial skeleton. At the end of every phase, the app can be demoed end-to-end. No feature ships without tests. No phase ships without the previous phase's E2E suite passing in CI.

---

## Phase 1 — Foundation (Weeks 1–4)

**Goal**: Establish the monorepo skeleton, cryptographic foundation, GitHub storage engine, design system, and CI pipeline. At the end of this phase, a developer can log in via GitHub OAuth, create an empty vault, and browse a styled empty state in the desktop app.

### Milestones

- [ ] **1.1 Monorepo scaffold**
  - [ ] Cargo workspace (`packages/core`, `packages/core-wasm`, `packages/ws-proxy`)
  - [ ] pnpm workspace (`packages/ui`, `packages/tauri-app`, `packages/web-app`)
  - [ ] Shared TypeScript `tsconfig.base.json`
  - [ ] ESLint + Prettier config
  - [ ] `rustfmt.toml` and `clippy.toml`

- [ ] **1.2 CI pipeline** (`.github/workflows/ci.yml`)
  - [ ] Lint job: `cargo clippy`, `eslint`, `cargo fmt --check`
  - [ ] Rust unit test job (matrix: ubuntu, macos, windows)
  - [ ] TypeScript test job (`vitest`)
  - [ ] WASM test job (`wasm-pack test --headless --chrome`)
  - [ ] Codecov integration with coverage gates

- [ ] **1.3 Crypto engine** (`packages/core/src/crypto/`)
  - [ ] `kdf.rs` — Argon2id key derivation with full test suite
  - [ ] `cipher.rs` — AES-256-GCM encrypt/decrypt with round-trip tests
  - [ ] `vault.rs` — Vault serialization/deserialization with property-based tests
  - [ ] Memory zeroization on drop (`zeroize` crate)
  - [ ] 90%+ unit test coverage

- [ ] **1.4 GitHub storage engine** (`packages/core/src/storage/`)
  - [ ] `github.rs` — GitHub API client (octocrab): create repo, read/write file
  - [ ] `sync.rs` — Sync protocol: fetch, push, SHA-based optimistic locking
  - [ ] Conflict resolution: per-item last-write-wins merge
  - [ ] `cache.rs` — Local cache (file-based on desktop, mocked in tests)
  - [ ] Integration tests using `wiremock-rs` (GitHub API mock)

- [ ] **1.5 Profile manager** (`packages/core/src/profile/`)
  - [ ] `types.rs` — `ConnectionProfile`, `SshKey`, `Password`, `KubeConfigItem` types
  - [ ] `manager.rs` — CRUD operations on vault items
  - [ ] Full unit test suite

- [ ] **1.6 Design system** (`packages/ui/src/components/`)
  - [ ] Button (primary, secondary, ghost, icon, loading state)
  - [ ] Input (text, password with reveal, search)
  - [ ] Modal (accessible, focus trap, keyboard dismiss)
  - [ ] Panel + SplitPane layout container
  - [ ] StatusBadge (connected, error, loading, disconnected)
  - [ ] Sidebar shell
  - [ ] Every component has Vitest + RTL tests

- [ ] **1.7 Auth feature** (`packages/ui/src/features/auth/`)
  - [ ] GitHub OAuth flow (PKCE, desktop + web)
  - [ ] `useAuthStore` with full unit tests
  - [ ] Vault passphrase modal (first-time setup + unlock)
  - [ ] `SessionGuard` HOC that blocks access until authenticated
  - [ ] Tauri OAuth plugin integration (desktop)

- [ ] **1.8 Vault store** (`packages/ui/src/stores/useVaultStore.ts`)
  - [ ] Load, sync status, offline mode
  - [ ] Unit tests with mocked Tauri IPC / WASM bridge

- [ ] **1.9 Desktop shell** (`packages/tauri-app/`)
  - [ ] Tauri v2 project setup with `packages/ui` imported
  - [ ] Tauri commands: `create_vault`, `load_vault`, `save_vault`, `get_user_profile`
  - [ ] App runs and GitHub OAuth completes on desktop

---

## Phase 2 — First Connection (Weeks 5–8)

**Goal**: A user can connect to an SSH server in a terminal tab on the desktop app. The tab system supports splitting and merging.

### Milestones

- [ ] **2.1 SSH adapter** (`packages/core/src/connection/ssh.rs`)
  - [ ] Password authentication
  - [ ] Public key authentication (RSA, Ed25519)
  - [ ] SSH agent forwarding
  - [ ] Keepalive support
  - [ ] Host key verification (trust on first use)
  - [ ] Integration tests: `testcontainers-rs` with `openssh-server` Docker image

- [ ] **2.2 Terminal view** (`packages/ui/src/features/terminal/`)
  - [ ] `TerminalView` wrapping `xterm.js` with fit addon and weblinks addon
  - [ ] `TerminalToolbar` (connection status, resize, close)
  - [ ] Tauri IPC events for streaming terminal output
  - [ ] Input forwarding via Tauri command
  - [ ] Unit tests for toolbar; Playwright E2E for full terminal interaction

- [ ] **2.3 Tab system** (`packages/ui/src/features/tabs/`)
  - [ ] `useTabStore` — full binary tree model with all operations
  - [ ] `TabWorkspace` — renders tree recursively as nested SplitPane + TabBar
  - [ ] `TabBar` — tabs, close button, drag to reorder within pane
  - [ ] Drag tab to split boundary → creates new pane
  - [ ] Keyboard shortcuts: split (Ctrl+\, Ctrl+-), close tab (Ctrl+W), navigate panes
  - [ ] Full Vitest unit tests for all store operations
  - [ ] Playwright E2E: split, drag tab, merge

- [ ] **2.4 Sidebar** (`packages/ui/src/features/sidebar/`)
  - [ ] Profile list from vault
  - [ ] Double-click to connect
  - [ ] Quick connect form
  - [ ] `useConnectionStore` — state machine for each session
  - [ ] Connection status badge

- [ ] **2.5 Settings — Connection editor** (`packages/ui/src/features/settings/`)
  - [ ] `ConnectionForm` — create/edit SSH connection profiles
  - [ ] Credential management (add password, add SSH key from file or paste)
  - [ ] Vault sync status panel

- [ ] **2.6 E2E test: full SSH flow**
  - [ ] Login via GitHub OAuth (mocked in CI)
  - [ ] Create SSH profile → connect → type command → see output
  - [ ] Split terminal → two simultaneous connections
  - [ ] Close tab → pane merges correctly

---

## Phase 3 — Protocol Expansion (Weeks 9–12)

**Goal**: SFTP, FTP, and Kubernetes support. Web app working with WASM and ws-proxy.

### Milestones

- [ ] **3.1 SFTP adapter** (`packages/core/src/connection/sftp.rs`)
  - [ ] Directory listing with metadata
  - [ ] Upload / download with progress tracking
  - [ ] Delete, mkdir, rename
  - [ ] Integration tests: testcontainers with `atmoz/sftp`

- [ ] **3.2 SFTP browser UI** (`packages/ui/src/features/sftp-browser/`)
  - [ ] `FileBrowser` — tree and list views, column sorting
  - [ ] `TransferQueue` — shows active transfers with progress bars, cancel button
  - [ ] Drag and drop from OS → upload
  - [ ] Context menu: download, rename, delete, permissions
  - [ ] E2E test: browse, upload, download

- [ ] **3.3 FTP adapter** (`packages/core/src/connection/ftp.rs`)
  - [ ] Plain FTP, FTPS explicit, FTPS implicit
  - [ ] Reuses `FileTransferAdapter` trait → SFTP browser UI works with FTP
  - [ ] Integration tests: testcontainers with `fauria/vsftpd`

- [ ] **3.4 Kubernetes adapter** (`packages/core/src/connection/k8s.rs`)
  - [ ] kubeconfig vault item parsing
  - [ ] List namespaces, pods, deployments, services, nodes
  - [ ] Pod exec (terminal)
  - [ ] Pod logs (streaming, follow mode)
  - [ ] Integration tests: `k3d` or `kind` cluster in CI
  - [ ] Unit tests for kubeconfig parsing

- [ ] **3.5 Kubernetes dashboard UI** (`packages/ui/src/features/k8s-dashboard/`)
  - [ ] `NamespaceSelector`
  - [ ] `PodList` with status badges and resource columns
  - [ ] `PodExecTerminal` — reuses `TerminalView`
  - [ ] `LogViewer` — scrollable, searchable, follow mode toggle
  - [ ] E2E test: connect to k3d cluster, list pods, exec into pod

- [ ] **3.6 WASM core build** (`packages/core-wasm/`)
  - [ ] wasm-bindgen exports for all core operations
  - [ ] `wasm-pack build` integrated into CI
  - [ ] WASM test suite matching native test coverage
  - [ ] WASM binary < 2MB gzipped (enforced by CI size check)

- [ ] **3.7 Web app** (`packages/web-app/`)
  - [ ] `wasm-bridge.ts` — WASM loading + WebSocket transport injection
  - [ ] Vite config, CSP headers
  - [ ] GitHub OAuth PKCE flow for web
  - [ ] Deploy to Cloudflare Pages on every merge to main (preview) and tags (production)

- [ ] **3.8 WebSocket proxy** (`packages/ws-proxy/`)
  - [ ] Stateless TCP-over-WebSocket relay
  - [ ] Docker image published to GHCR
  - [ ] Connection logging (no payload logging)
  - [ ] Authentication via short-lived tokens (from the web app)

- [ ] **3.9 E2E: web platform**
  - [ ] SSH session via ws-proxy
  - [ ] SFTP browse and download via ws-proxy
  - [ ] Kubernetes connect directly (HTTPS)

---

## Phase 4 — Mobile & Polish (Weeks 13–16)

**Goal**: iOS and Android apps. Biometric unlock. Multi-device conflict resolution UI. Performance and accessibility audit. Open-source public release.

### Milestones

- [ ] **4.1 Tauri Mobile builds**
  - [ ] iOS target (`tauri ios build`)
  - [ ] Android target (`tauri android build`)
  - [ ] Terminal rendering benchmark on iPhone and Android device (target: 60fps)
  - [ ] Touch-friendly tab management (long-press to split, pinch to resize)
  - [ ] App icons and splash screens

- [ ] **4.2 Biometric unlock**
  - [ ] iOS: Face ID / Touch ID via Tauri plugin or `LocalAuthentication`
  - [ ] Android: BiometricPrompt via Tauri plugin
  - [ ] Master Key wrapped with Secure Enclave / StrongBox key
  - [ ] Graceful fallback to passphrase
  - [ ] Opt-in UI with clear security disclosure

- [ ] **4.3 Conflict resolution UI**
  - [ ] Side-by-side diff view for true vault conflicts
  - [ ] Per-profile keep-local / keep-remote / manual-merge actions
  - [ ] Conflict indicator in sidebar

- [ ] **4.4 Mobile CI pipeline**
  - [ ] iOS simulator build in CI (macOS runner)
  - [ ] Android emulator build in CI
  - [ ] Appium E2E: connect SSH, split pane (swipe gesture), biometric mock

- [ ] **4.5 Performance**
  - [ ] Terminal frame rate profiling (target 60fps on mid-range devices)
  - [ ] WASM load time < 2 seconds on 4G connection
  - [ ] Vault decrypt time < 100ms for 1,000 items
  - [ ] App startup time < 1 second (desktop), < 2 seconds (mobile)

- [ ] **4.6 Accessibility audit**
  - [ ] All interactive elements keyboard-navigable
  - [ ] Screen reader testing (VoiceOver/TalkBack for mobile, NVDA/JAWS for desktop)
  - [ ] Color contrast WCAG AA compliance
  - [ ] Focus indicators visible in all themes

- [ ] **4.7 Open-source release preparation**
  - [ ] `LICENSE` (MIT)
  - [ ] `CHANGELOG.md` (auto-generated from Conventional Commits via `git-cliff`)
  - [ ] GitHub Release with signed binaries (macOS notarized, Windows signed)
  - [ ] WASM package published to npm (`@tacoshell/core-wasm`)
  - [ ] Documentation site (VitePress or Docusaurus)
  - [ ] GitHub Discussions enabled
  - [ ] Issue templates (bug report, feature request, security)
  - [ ] Code of Conduct

---

## Backlog (Post-v1.0)

- [ ] SSH tunneling / port forwarding UI
- [ ] Multi-hop SSH (ProxyJump) UI
- [ ] SFTP in-browser file editor
- [ ] Kubernetes resource editor (apply manifests from UI)
- [ ] Connection groups and folders in sidebar
- [ ] Shared vaults (team collaboration, separate GitHub org repo)
- [ ] Plugin system for additional protocols (RDP, VNC, database clients)
- [ ] Custom themes and color schemes
- [ ] Command palette (Ctrl+K)
- [ ] Session recording and replay (terminal audit log)
- [ ] Import from: PuTTY, iTerm2, Royal TSX, Termius
- [ ] Homebrew formula and WinGet package
- [ ] Flatpak for Linux
- [ ] Self-hosted relay server (alternative to GitHub storage)
