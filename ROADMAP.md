# Tacoshell Roadmap

## Phase 1: Core Architecture & Basic SSH ✅
**Status: Complete**

- [x] Initialize Cargo workspace with modular crates
- [x] Implement `SshSession` trait and basic connect/auth/exec
- [x] Set up SQLite schema and migrations
- [x] Scaffold Tauri app with terminal view (xterm.js)
- [x] Define IPC contract (connect, disconnect, send_input)
- [x] Secret management with encryption

## Phase 2: Tab & Split-Screen Window Manager 🚧
**Status: In Progress**

- [x] Implement tab management (add/close) with Zustand state
- [x] Basic layout with sidebar, tab bar, and main content
- [x] Settings panel with multiple sections
- [x] Secrets manager UI component
- [ ] Integrate react-mosaic for split-pane layout
- [ ] Persist layout state to local storage
- [ ] Support drag-and-drop to split/merge panes
- [ ] Tab reordering via drag-and-drop

## Phase 3: File Transfer Protocols – SFTP
**Status: Planned**

- [ ] Create `tacoshell-transfer` crate wrapping `ssh2::Sftp`
- [ ] Build dual-pane file browser component (local ↔ remote)
- [ ] Implement upload/download progress via Tauri event streams
- [ ] Add FTP support via `suppaftp` crate (optional)

## Phase 4: Kubernetes Integration
**Status: Planned**

- [ ] Add `tacoshell-k8s` crate using `kube-rs` + `k8s-openapi`
- [ ] Implement kubeconfig discovery and context switching
- [ ] Build dashboard: namespace selector, pod list, log viewer, exec shell
- [ ] Stream logs via Tauri events

## Phase 5: Polish & Community Release
**Status: Planned**

- [ ] Add settings persistence (themes, keybindings, default shell)
- [ ] Implement connection profiles import/export (JSON)
- [ ] Cross-platform builds (Windows, macOS, Linux) via GitHub Actions
- [ ] Write contribution guide, finalize ADRs
- [ ] Publish v0.1.0-beta
