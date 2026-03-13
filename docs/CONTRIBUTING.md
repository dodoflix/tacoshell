# Contributing to Tacoshell

Thank you for your interest in contributing. This document explains how to set up the development environment, the contribution workflow, and the quality standards we enforce.

---

## 1. Development Setup

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.78+ | `rustup.rs` |
| Node.js | 20+ | `nodejs.org` or `nvm` |
| pnpm | 9+ | `npm i -g pnpm` |
| Tauri CLI | 2.x | `cargo install tauri-cli` |
| wasm-pack | latest | `cargo install wasm-pack` |
| Docker | any | Required for integration tests |

### Clone and Install

```bash
git clone https://github.com/your-org/tacoshell.git
cd tacoshell

# Install JS dependencies
pnpm install

# Build Rust workspace (verifies toolchain)
cargo build --workspace
```

### Set Up Pre-commit Hooks

```bash
pnpm prepare    # installs husky hooks
```

This installs hooks that run linting and tests before each commit. Do not bypass these with `--no-verify`.

---

## 2. Running the App

### Desktop (Tauri)

```bash
pnpm --filter tauri-app tauri dev
```

### Web

```bash
pnpm --filter web-app dev
```

### WebSocket Proxy (required for web SSH/SFTP/FTP)

```bash
cargo run --package ws-proxy
# or with Docker:
docker run --rm -p 9000:9000 tacoshell/ws-proxy
```

---

## 3. Running Tests

### All Tests

```bash
# Rust
cargo test --workspace

# TypeScript
pnpm -r run test

# WASM
wasm-pack test packages/core-wasm --headless --chrome
```

### Watch Mode (Development)

```bash
# Rust
cargo watch -x "test --workspace"

# TypeScript
pnpm --filter ui test -- --watch
```

### E2E Tests

```bash
# Desktop
pnpm --filter tauri-app e2e

# Web (requires web-app running)
pnpm --filter web-app e2e
```

### Integration Tests (requires Docker)

```bash
cargo test --workspace --features integration
```

---

## 4. Code Standards

### TDD is Mandatory

Every contribution that changes production behavior must include tests written **before** the implementation. See [`TDD.md`](TDD.md) for the workflow.

### Rust

- Follow `rustfmt` formatting (`cargo fmt`)
- Pass `cargo clippy --all-targets -- -D warnings` with no warnings
- Use `thiserror` for error types at module boundaries
- Use `anyhow` only inside `main.rs` or top-level binary entry points
- Sensitive data must use `SecretString` (secrecy crate) or `Zeroizing<T>` (zeroize crate)
- No `unwrap()` or `expect()` outside of test code

### TypeScript

- Strict TypeScript — `"strict": true` in tsconfig, no `any` casts
- ESLint + Prettier enforce formatting
- No `console.log` in production code (use the structured logger)
- Components must be accessible: use Radix UI primitives, include ARIA labels
- No direct Tauri `invoke` calls in component files — go through a typed service layer

### Commit Messages

Follow [Conventional Commits](https://conventionalcommits.org/):

```
feat(ssh): add public key authentication support
fix(vault): handle SHA mismatch during concurrent sync
test(crypto): add round-trip tests for AES-256-GCM
docs(architecture): update tab tree model section
chore(ci): pin wasm-pack to 0.13
```

---

## 5. Pull Request Process

1. Fork the repo and create a branch: `git checkout -b feat/your-feature`
2. Write failing tests first (TDD RED phase)
3. Implement the feature (TDD GREEN phase)
4. Refactor if needed (TDD REFACTOR phase)
5. Ensure all tests pass: `cargo test --workspace && pnpm -r run test`
6. Ensure coverage has not dropped: check the Codecov report in CI
7. Open a PR against `main`

**PR title**: follow Conventional Commits format (`feat:`, `fix:`, `docs:`, etc.)

**PR description must include**:
- What problem this solves / what feature this adds
- How to test it manually (steps)
- Any migration steps required (vault schema changes, breaking API changes)
- Screenshots for UI changes

### Review Criteria

- All CI checks green (lint, unit tests, integration tests, E2E, coverage)
- At least one approval from a maintainer
- No `unwrap()` / `any` type escapes in production code
- Tests cover the new behavior (not just lines)
- No security regressions (maintainers check `cargo audit` output)

---

## 6. Project Layout Quick Reference

```
packages/core/          Rust: connection engine, crypto, storage
packages/core-wasm/     Rust: WASM bindings
packages/ui/            TypeScript: shared React components + Zustand stores
packages/tauri-app/     Desktop + Mobile shell
packages/web-app/       Web shell
packages/ws-proxy/      Rust: WebSocket-to-TCP proxy
docs/                   Architecture and design documents
.github/workflows/      CI/CD pipelines
```

---

## 7. Getting Help

- Open a Discussion on GitHub for design questions before implementing
- Open an Issue for bugs with a minimal reproduction
- Tag `@maintainers` in your PR if you need a faster review
- Security issues: email `security@tacoshell.dev` (do not open public issues)
