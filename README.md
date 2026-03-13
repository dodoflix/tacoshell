# Tacoshell

> Open-source, cross-platform connection manager for SSH, SFTP, FTP, and Kubernetes.

Tacoshell runs on **Web**, **Desktop** (macOS, Windows, Linux), and **Mobile** (iOS, Android) from a single TypeScript + Rust codebase. User credentials are encrypted client-side and stored in the user's own private GitHub repository — no third-party servers involved.

---

## Features

- **Protocol support**: SSH, SFTP, FTP, Kubernetes (kubeconfig, pod exec, logs)
- **BYOD storage**: Vault stored in your own private GitHub repository, AES-256-GCM encrypted
- **GitHub OAuth**: Single sign-on via GitHub — no separate account required
- **Advanced tab system**: Split, merge, and rearrange panes like a modern IDE or tmux
- **Cross-platform**: One codebase for Web, Desktop (Tauri v2), and Mobile (Tauri v2 Mobile)
- **Strict TDD**: Every feature is test-driven from day one

---

## Architecture Overview

```
UI Layer (React + TypeScript)
    |
Core Engine (Rust — compiles to native binary and WebAssembly)
    |
GitHub Storage (private repo, client-side encrypted)
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full design.

---

## Monorepo Structure

```
tacoshell/
  packages/
    core/           # Rust crate: connection engine, crypto, storage
    core-wasm/      # WASM wrapper for the web platform
    ui/             # Shared React component library + Zustand stores
    tauri-app/      # Desktop + Mobile shell (Tauri v2)
    web-app/        # Web shell (Vite SPA)
    ws-proxy/       # Optional WebSocket-to-TCP relay for web SSH/FTP
  docs/             # Architecture, security, contributing guides
  .github/
    workflows/      # CI/CD pipelines
```

---

## Quick Start

### Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.78+ (via rustup) |
| Node.js | 20+ |
| pnpm | 9+ |
| Tauri CLI | 2.x |

### Install

```bash
# Clone the repo
git clone https://github.com/your-org/tacoshell.git
cd tacoshell

# Install JS dependencies
pnpm install

# Build the Rust workspace
cargo build --workspace
```

### Run Desktop App

```bash
pnpm --filter tauri-app tauri dev
```

### Run Web App

```bash
pnpm --filter web-app dev
```

### Run Tests

```bash
# Rust tests
cargo test --workspace

# TypeScript tests
pnpm -r run test

# E2E tests (Playwright)
pnpm --filter tauri-app e2e
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Full system architecture |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Encryption model and threat model |
| [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) | How to contribute |
| [`docs/TDD.md`](docs/TDD.md) | TDD workflow and testing strategy |
| [`docs/PROTOCOLS.md`](docs/PROTOCOLS.md) | Protocol adapter specifications |
| [`docs/STORAGE.md`](docs/STORAGE.md) | GitHub storage engine design |
| [`docs/STATE.md`](docs/STATE.md) | State management and tab tree model |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Phased delivery plan and milestones |

---

## License

MIT — see [`LICENSE`](LICENSE).
