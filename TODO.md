# Tacoshell — Master TODO List

> Track progress here. Check off items as they are completed. Each item links to the relevant doc for context.

Legend: `[ ]` = not started · `[~]` = in progress · `[x]` = done

---

## Phase 1 — Foundation

### 1.1 Monorepo Scaffold

- [x] Create `Cargo.workspace.toml` with all Rust members
- [x] Create `pnpm-workspace.yaml`
- [x] Create root `package.json` with workspace scripts
- [x] Create `tsconfig.base.json` (shared TS config)
- [x] Set up `rustfmt.toml`
- [x] Set up `clippy.toml` (deny unsafe, unwrap in non-test code)
- [x] Create `.editorconfig`
- [x] Create `.gitignore` (Rust + Node + Tauri + IDE)
- [x] Install and configure `husky` pre-commit hooks
- [x] Configure `lint-staged` for Rust and TypeScript

### 1.2 CI Pipeline

- [x] `.github/workflows/ci.yml` — lint, test-rust, test-typescript, test-wasm, integration, build-desktop, build-web
- [x] `.github/workflows/release.yml` — signed binaries, npm WASM publish, web deploy, changelog
- [x] Configure Codecov with coverage thresholds (Rust: 90%, TS: 80%)
- [x] Add `cargo audit` step to CI
- [x] Add PR template (`.github/pull_request_template.md`)
- [x] Add issue templates (bug, feature, security)

### 1.3 Crypto Engine (`packages/core/src/crypto/`)

- [x] Write failing tests for `kdf.rs` (Argon2id key derivation)
- [x] Implement `kdf.rs`
- [x] Write failing tests for `cipher.rs` (AES-256-GCM encrypt/decrypt)
- [x] Implement `cipher.rs`
- [x] Write failing tests for `vault.rs` (serialization round-trips)
- [x] Implement `vault.rs`
- [x] Implement `zeroize` on all sensitive types
- [x] Verify 90%+ coverage via `cargo-tarpaulin`
- [x] Add `cargo-mutants` baseline

### 1.4 GitHub Storage Engine (`packages/core/src/storage/`)

- [x] Write failing tests for `github.rs` (octocrab integration, wiremock mocks)
- [x] Implement `github.rs` — repo create, file read, file write
- [x] Write failing tests for `sync.rs` — optimistic locking, SHA mismatch flow
- [x] Implement `sync.rs` — sync protocol with conflict resolution
- [x] Write failing tests for `cache.rs` — local cache read/write
- [x] Implement `cache.rs`
- [x] Integration test: full vault create → modify → sync → re-load cycle

### 1.5 Profile Manager (`packages/core/src/profile/`)

- [x] Define types in `types.rs`: `ConnectionProfile`, `SshKey`, `Password`, `KubeConfigItem`
- [x] Write failing tests for `manager.rs` (CRUD operations)
- [x] Implement `manager.rs`
- [x] Test: create profile → save → reload → compare

### 1.6 Design System (`packages/ui/src/components/`)

- [x] Button — write tests → implement (primary, secondary, ghost, icon, loading)
- [x] Input — write tests → implement (text, password with reveal, search)
- [x] Modal — write tests → implement (focus trap, keyboard dismiss, ARIA)
- [x] Panel — write tests → implement
- [x] SplitPane — write tests → implement (drag handle, ratio prop)
- [x] TabBar — write tests → implement (tabs, close, active indicator)
- [x] StatusBadge — write tests → implement (connected, error, loading, disconnected)
- [x] Sidebar shell — write tests → implement
- [x] TreeView — write tests → implement
- [x] Storybook (or equivalent) for component visual review

### 1.7 Auth Feature (`packages/ui/src/features/auth/`)

- [x] Write failing tests for `useAuthStore`
- [x] Implement `useAuthStore` (token, user profile, auth status)
- [x] Implement GitHub OAuth PKCE flow (web)
- [x] Implement `GitHubLoginButton`
- [x] Implement `VaultPassphraseModal` (first-time setup + unlock)
- [x] Implement `SessionGuard` HOC

### 1.8 Vault Store (`packages/ui/src/stores/useVaultStore.ts`)

- [x] Write failing tests for `useVaultStore`
- [x] Implement `useVaultStore` (load, sync, offline mode, item CRUD)

### 1.9 Desktop Shell (`packages/tauri-app/`)

- [x] Tauri v2 project setup
- [x] Link `packages/ui` as a workspace dependency
- [x] Implement Tauri commands: `create_vault`, `load_vault`, `save_vault`, `get_user_profile`
- [x] Integrate Tauri OAuth plugin (desktop) ← moved from 1.7
- [x] Wire `useVaultStore` to Tauri IPC commands ← moved from 1.8
- [x] Implement `exchange_oauth_code` Tauri command (PKCE token exchange, no client_secret)
- [x] Implement `open_url` Tauri command (cross-platform system browser opener)
- [x] Implement `startDesktopOAuth` service (full PKCE OAuth flow via tauri-plugin-oauth)
- [x] Wire desktop OAuth login to `useAuthStore` in `App.tsx`
- [ ] App runs and GitHub OAuth completes on desktop (macOS)
- [ ] App runs on Windows
- [ ] App runs on Linux

---

## Phase 2 — First Connection

### 2.1 SSH Adapter (`packages/core/src/connection/ssh.rs`)

- [x] Write failing tests for password authentication
- [x] Implement password auth
- [x] Write failing tests for public key authentication (Ed25519, RSA)
- [x] Implement public key auth
- [x] Write failing tests for SSH agent forwarding
- [x] Implement SSH agent forwarding (desktop only)
- [x] Write failing tests for host key verification (TOFU policy)
- [x] Implement TOFU host key verification
- [x] Implement keepalive (configurable interval)
- [x] Write failing integration tests (testcontainers + openssh-server)
- [ ] Integration tests pass (requires Docker — run with `--features integration`)

### 2.2 Terminal View (`packages/ui/src/features/terminal/`)

- [ ] Write failing tests for `TerminalToolbar`
- [ ] Implement `TerminalView` (xterm.js + fit addon + weblinks addon)
- [ ] Implement `TerminalToolbar`
- [ ] Wire terminal output stream from Tauri IPC events
- [ ] Wire terminal input to Tauri command
- [ ] Implement resize handler (cols/rows → Tauri command)
- [ ] E2E test: terminal renders SSH output, input echoes correctly

### 2.3 Tab System (`packages/ui/src/features/tabs/`)

- [ ] Write failing tests for `useTabStore` — all operations (add, close, split, merge, move)
- [ ] Implement `useTabStore` with binary tree model
- [ ] Write failing tests for `TabWorkspace` (renders tree structure)
- [ ] Implement `TabWorkspace` (recursive tree renderer)
- [ ] Implement `TabBar` with drag-to-reorder
- [ ] Implement drag tab to split boundary → create new pane
- [ ] Implement keyboard shortcuts (split, close, navigate panes)
- [ ] E2E test: split horizontal, split vertical, drag tab between panes, merge

### 2.4 Connection Sidebar (`packages/ui/src/features/sidebar/`)

- [ ] Write failing tests for `useConnectionStore` state machine
- [ ] Implement `useConnectionStore`
- [ ] Implement `ConnectionSidebar` (profile list, status badges)
- [ ] Implement double-click to connect
- [ ] Implement `QuickConnect` form

### 2.5 Connection Editor (`packages/ui/src/features/settings/`)

- [ ] Implement `ConnectionForm` (create/edit SSH profiles)
- [ ] Implement `CredentialManager` (add password, add SSH key)
- [ ] Implement `VaultSyncStatus` panel
- [ ] E2E test: create profile → connect → see terminal

### 2.6 E2E — Full SSH Flow

- [ ] Setup: GitHub OAuth mock server for CI
- [ ] Setup: testcontainers SSH server for E2E
- [ ] E2E test: full login flow (web mock + desktop) ← moved from 1.7
- [ ] E2E test: login → create SSH profile → connect → execute command → see output
- [ ] E2E test: split terminal → two simultaneous SSH connections
- [ ] E2E test: close tab → pane merges
- [ ] E2E test: reconnect after drop

---

## Phase 3 — Protocol Expansion

### 3.1 SFTP Adapter

- [ ] Write failing tests for directory listing
- [ ] Implement `SftpAdapter::list_dir`
- [ ] Write failing tests for upload/download with progress
- [ ] Implement upload and download
- [ ] Implement delete, mkdir, rename
- [ ] Integration tests (testcontainers atmoz/sftp)

### 3.2 SFTP Browser UI

- [ ] Write failing tests for `FileBrowser`
- [ ] Implement `FileBrowser` (tree + list views, column sort)
- [ ] Implement `TransferQueue` (progress bars, cancel)
- [ ] Implement OS drag-and-drop → upload
- [ ] Implement context menu (download, rename, delete, permissions)
- [ ] E2E test: browse, upload, download

### 3.3 FTP Adapter

- [ ] Write failing tests for plain FTP
- [ ] Implement `FtpAdapter` (plain, FTPS explicit, FTPS implicit)
- [ ] FTP reuses SFTP browser UI — verify trait compatibility
- [ ] Integration tests (testcontainers fauria/vsftpd)

### 3.4 Kubernetes Adapter

- [ ] Write failing tests for kubeconfig parsing
- [ ] Implement kubeconfig vault item handling
- [ ] Write failing tests for pod listing
- [ ] Implement `K8sAdapter::list_namespaces`, `list_pods`
- [ ] Implement `list_deployments`, `list_services`, `list_nodes`
- [ ] Implement `exec_pod`
- [ ] Implement `pod_logs` (streaming, follow mode)
- [ ] Integration tests (kind or k3d cluster in CI)

### 3.5 Kubernetes Dashboard UI

- [ ] Write failing tests for `PodList`
- [ ] Implement `NamespaceSelector`
- [ ] Implement `PodList`
- [ ] Implement `PodExecTerminal` (reuses TerminalView)
- [ ] Implement `LogViewer` (searchable, follow mode)
- [ ] E2E test: connect → list pods → exec into pod → stream logs

### 3.6 WASM Core Build

- [ ] Set up `packages/core-wasm` Cargo.toml with wasm-bindgen
- [ ] Export all core operations via wasm-bindgen
- [ ] wasm-pack test suite — 100% coverage match to native
- [ ] CI binary size check (< 5MB uncompressed)
- [ ] wasm-opt pass in release build

### 3.7 Web App

- [ ] Set up `packages/web-app` (Vite + React)
- [ ] Implement `wasm-bridge.ts` (WASM load + WebSocket transport injection)
- [ ] GitHub OAuth PKCE flow for web
- [ ] CSP headers configured
- [ ] Web app connects to GitHub OAuth
- [ ] Web app loads vault

### 3.8 WebSocket Proxy

- [ ] Implement `ws-proxy` (stateless WebSocket-to-TCP relay)
- [ ] Add authentication (short-lived tokens from web app)
- [ ] Add structured logging (no payload logging)
- [ ] Docker image: `ghcr.io/your-org/tacoshell/ws-proxy`
- [ ] Self-hosting documentation

### 3.9 E2E — Web Platform

- [ ] SSH session via ws-proxy (Playwright)
- [ ] SFTP browse and download via ws-proxy (Playwright)
- [ ] Kubernetes connect directly (Playwright)

---

## Phase 4 — Mobile & Polish

### 4.1 Tauri Mobile

- [ ] iOS build target configured (`tauri ios build`)
- [ ] Android build target configured (`tauri android build`)
- [ ] Terminal rendering benchmark: 60fps target on mid-range device
- [ ] Touch-friendly tab management (long-press to split, pinch to resize)
- [ ] App icons for iOS and Android
- [ ] Splash screens for iOS and Android

### 4.2 Biometric Unlock

- [ ] iOS: Face ID / Touch ID integration
- [ ] Android: BiometricPrompt integration
- [ ] Master Key wrapped with Secure Enclave / StrongBox key
- [ ] Graceful fallback to vault passphrase
- [ ] Opt-in UI with security disclosure

### 4.3 Conflict Resolution UI

- [ ] Implement side-by-side diff view for vault conflicts
- [ ] Per-profile actions: keep-local, keep-remote, manual-merge
- [ ] Conflict indicator in sidebar

### 4.4 Mobile CI

- [ ] iOS simulator build in CI (macOS runner)
- [ ] Android emulator build in CI
- [ ] Mobile E2E tests (Appium or Detox)

### 4.5 Performance

- [ ] Terminal frame rate profiling
- [ ] WASM load time measurement (target < 2s on 4G)
- [ ] Vault decrypt benchmark (target < 100ms for 1,000 items)
- [ ] App startup time benchmarks

### 4.6 Accessibility

- [ ] Keyboard navigation audit (all interactive elements reachable)
- [ ] Screen reader testing: VoiceOver (macOS/iOS), NVDA (Windows), TalkBack (Android)
- [ ] Color contrast: WCAG AA compliance
- [ ] Focus indicator visibility in all themes

### 4.7 Open-Source Release

- [ ] `LICENSE` file (MIT)
- [ ] `CHANGELOG.md` (generated via git-cliff from Conventional Commits)
- [ ] GitHub Releases with signed binaries (macOS notarized, Windows signed)
- [ ] WASM package published to npm (`@tacoshell/core-wasm`)
- [ ] Documentation site (VitePress)
- [ ] GitHub Discussions enabled
- [ ] Issue templates (bug, feature, security)
- [ ] Code of Conduct (`CODE_OF_CONDUCT.md`)
- [ ] `SECURITY.md` in repo root (GitHub security policy)
- [ ] Homebrew formula (macOS)
- [ ] Announce on GitHub, dev.to, Hacker News

---

## Ongoing / Cross-Cutting

- [ ] `cargo audit` passes on every PR (no known vulnerabilities)
- [ ] `npm audit` passes at high severity level
- [ ] Rust coverage ≥ 90% (enforced by Codecov)
- [ ] TypeScript coverage ≥ 80% (enforced by Codecov)
- [ ] All E2E critical paths covered
- [ ] No `unwrap()` / `expect()` in non-test Rust code (clippy lint)
- [ ] No `any` casts in TypeScript (strict mode)
- [ ] Dependency updates reviewed monthly (Dependabot)
- [ ] Mutation testing score ≥ 70% (quarterly)
