# Tacoshell — Claude Code Guide

## Project Overview

Tacoshell is a cross-platform, open-source connection manager (SSH, SFTP, FTP, Kubernetes). Stack: **Rust** (core engine, compiles to native + WASM) + **React/TypeScript** (UI, shared across all platforms) + **Tauri v2** (desktop and mobile shell).

Key constraint: **strict TDD** — tests are written before production code, always.

## Repository Layout

```
packages/core/       Rust: crypto, storage, protocol adapters
packages/core-wasm/  Rust: wasm-bindgen exports
packages/ui/         TypeScript: shared React components + Zustand stores
packages/tauri-app/  Desktop + Mobile shell (Tauri v2)
packages/web-app/    Web shell (Vite SPA)
packages/ws-proxy/   Rust: WebSocket-to-TCP relay
docs/                Architecture and design docs (read these first)
TODO.md              Master task list — update as work progresses
```

## Essential Docs to Read Before Coding

1. `docs/ARCHITECTURE.md` — overall system design
2. `docs/SECURITY.md` — encryption model (never skip this for crypto changes)
3. `docs/STATE.md` — tab tree model and store design
4. `docs/TDD.md` — mandatory TDD workflow
5. `docs/PROTOCOLS.md` — protocol trait definitions
6. `docs/STORAGE.md` — GitHub storage engine and sync protocol

## Development Commands

```bash
# Run desktop app
pnpm --filter tauri-app tauri dev

# Run web app
pnpm --filter web-app dev

# Run all Rust tests
cargo test --workspace

# Run all TypeScript tests
pnpm -r run test

# Run WASM tests
wasm-pack test packages/core-wasm --headless --chrome

# Build WASM
wasm-pack build packages/core-wasm --target web

# Run integration tests (requires Docker)
cargo test --workspace --features integration

# Lint Rust
cargo clippy --workspace --all-targets -- -D warnings

# Lint TypeScript
pnpm -r run lint
```

## TDD Rules (Non-negotiable)

1. **Red first**: Write a failing test before any production code
2. **Green next**: Write the minimum code to make it pass
3. **Refactor last**: Clean up with tests still green
4. Never commit production code without a test covering the new behavior
5. Run `cargo test --workspace && pnpm -r run test` before every commit

## Code Conventions

### Rust
- No `unwrap()` or `expect()` in non-test code — use `?` operator and typed errors
- Use `thiserror` for error types at module boundaries
- Sensitive data uses `SecretString` (secrecy) or `Zeroizing<T>` (zeroize)
- All public types implement `Debug` (no sensitive data in Debug output)
- Tests live in `#[cfg(test)] mod tests {}` blocks in the same file

### TypeScript
- Strict TypeScript — no `any` casts
- No direct `invoke()` calls in components — use typed service layer
- Zustand stores use selector pattern to minimize re-renders
- All components use Radix UI primitives for accessibility
- Tests in `__tests__/` directories adjacent to source files

### Commits
Follow Conventional Commits: `feat:`, `fix:`, `test:`, `docs:`, `chore:`, `refactor:`

## Architecture Rules

- The Rust core has **zero knowledge** of the UI layer
- The UI has **zero** direct network/filesystem access — everything goes through Tauri IPC or WASM bridge
- Stores never directly import each other — use subscriptions via the orchestrator
- New protocols must implement the trait(s) in `packages/core/src/connection/mod.rs`
- Vault writes always go through the storage engine — never write to GitHub directly from the UI

## Security Rules

- **Never** log decrypted vault contents, passwords, or private keys
- **Never** store the Master Key on disk — memory only, zeroize on drop
- All data written to GitHub must be AES-256-GCM encrypted first
- New GitHub API calls must not request scopes beyond `repo` and `read:user`
- Run `cargo audit` before merging any dependency changes

## No Suppression Policy (Non-negotiable)

Never suppress, silence, or ignore any warning, error, or alert — fix it or refactor the code. This applies to all languages and tooling:

- **TypeScript/ESLint:** No `eslint-disable`, `eslint-disable-next-line`, `@ts-ignore`, or `@ts-expect-error`
- **Rust:** No `#[allow(...)]` Clippy attributes in non-test production code — fix the underlying issue
- **Tests:** No `console.error = ...` or `console.warn = ...` patches in test setup files — fix the root cause (e.g. switch test environments, fix component accessibility, update the tool)
- **Git:** Never use `--no-verify` to skip hooks — fix what the hook is reporting
- **General:** No `// TODO: remove once upstream fixes...` workarounds left unresolved — either fix it now or open a tracked issue

If a tool produces a false-positive, fix the root cause (e.g. switch to a more compatible test environment, add a proper type, restructure the code) rather than silencing the warning.

## Current Phase

See `TODO.md` for the active task list. Check off items as they are completed.
