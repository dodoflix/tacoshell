---
name: tacoshell_project
description: Tacoshell project — architecture decisions, stack, and current phase
type: project
---

Open-source cross-platform connection manager (SSH, SFTP, FTP, Kubernetes). Greenfield project started 2026-03-13.

**Why:** Build a BYOD (bring your own database) connection manager with no third-party server involvement — all data encrypted client-side and stored in user's private GitHub repo.

**Stack:**
- Core: Rust (compiles to native + WASM)
- UI: React + TypeScript + Zustand (shared across all platforms)
- Desktop/Mobile: Tauri v2
- Web: Vite SPA + WASM bridge
- Storage: AES-256-GCM encrypted vault in private GitHub repo
- Auth: GitHub OAuth only

**Phase:** Currently starting Phase 1 (Foundation). All architecture docs, TODO list, CI pipelines, and directory structure have been created. No source code implemented yet.

**Key docs:**
- `TODO.md` — master task list (check off as work progresses)
- `docs/ARCHITECTURE.md` — full system design
- `docs/SECURITY.md` — encryption model
- `docs/TDD.md` — mandatory TDD workflow
- `CLAUDE.md` — Claude Code guide for this project

**How to apply:** Always read `TODO.md` at the start of a session to know what's next. Follow strict TDD — tests before code, always.
