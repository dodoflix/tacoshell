# tacoshell Development Patterns

> Auto-generated skill from repository analysis — updated for accuracy

## Overview

Tacoshell is a cross-platform, open-source connection manager (SSH, SFTP, FTP, Kubernetes) built with **Rust** (core engine, compiles to native + WASM) + **React/TypeScript** (UI, shared across all platforms) + **Tauri v2** (desktop and mobile shell).

**Key constraint:** **strict TDD** — tests are written before production code, always. This is non-negotiable.

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

## TDD Rules (Non-negotiable)

1. **Red first**: Write a failing test before any production code
2. **Green next**: Write the minimum code to make it pass
3. **Refactor last**: Clean up with tests still green
4. Never commit production code without a test covering the new behavior
5. Run `cargo test --workspace && pnpm -r run test` before every commit

## Coding Conventions

### File Naming
- Component files follow **PascalCase**: `ComponentName.tsx`
- Other TypeScript files use **camelCase**: `apiService.ts`, `crypto.ts`
- Test files use: `ComponentName.test.tsx` or `functionName.test.ts`
- Rust files use **snake_case**: `cipher.rs`, `ssh_adapter.rs`

### Import/Export Style
- Use **relative imports** for local modules: `import { utils } from '../lib/utils'`
- **Mixed export style**: combination of named and default exports
- Barrel exports in `index.ts` files for clean package interfaces

### Commit Messages
- Follow **conventional commits** format: `feat:`, `fix:`, `test:`, `docs:`, `chore:`, `refactor:`
- Keep messages around 50-63 characters
- Examples:
  ```
  feat: add user authentication component
  fix: resolve memory leak in file watcher
  test: add comprehensive tests for crypto module
  refactor: extract shared validation utilities
  ```

### Rust Conventions
- No `unwrap()` or `expect()` in non-test code — use `?` operator and typed errors
- Use `thiserror` for error types at module boundaries
- Sensitive data uses `SecretString` (secrecy) or `Zeroizing<T>` (zeroize)
- All public types implement `Debug` (no sensitive data in Debug output)
- Tests live in `#[cfg(test)] mod tests {}` blocks in the same file

### TypeScript Conventions
- Strict TypeScript — no `any` casts
- No direct `invoke()` calls in components — use typed service layer
- Zustand stores use selector pattern to minimize re-renders
- All components use Radix UI primitives for accessibility
- Tests in `__tests__/` directories adjacent to source files

## Architecture Rules (Critical)

- The Rust core has **zero knowledge** of the UI layer
- The UI has **zero** direct network/filesystem access — everything goes through Tauri IPC or WASM bridge
- Stores never directly import each other — use subscriptions via the orchestrator
- New protocols must implement the trait(s) in `packages/core/src/connection/mod.rs`
- Vault writes always go through the storage engine — never write to GitHub directly from the UI

## Security Rules (Critical)

- **Never** log decrypted vault contents, passwords, or private keys
- **Never** store the Master Key on disk — memory only, zeroize on drop
- All data written to GitHub must be AES-256-GCM encrypted first
- New GitHub API calls must not request scopes beyond `repo` and `read:user`
- Run `cargo audit` before merging any dependency changes

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

## Common Workflows

### Feature Implementation (TDD Required)

**Always follow Test-Driven Development:**

1. **Write failing test first** (RED) — Test the expected behavior before writing any production code
2. **Write minimal production code** (GREEN) — Make the test pass with the simplest implementation
3. **Refactor** — Clean up with tests still green
4. **Update documentation** — Update TODO.md and relevant docs
5. **Verify all tests pass** — Run `cargo test --workspace && pnpm -r run test`

```typescript
// 1. Write test first (RED)
describe('newFeature', () => {
  it('should handle valid input correctly', () => {
    const result = newFeature(mockInput);
    expect(result).toEqual(expectedOutput);
  });
});

// 2. Implement feature (GREEN)
export function newFeature(input: FeatureInput): FeatureOutput {
  // Implementation
}
```

### UI Component Development

1. **Write component tests first** in `__tests__/` subdirectory
2. **Implement component** with proper TypeScript props interface
3. **Verify tests pass** — Run `pnpm -r run test`
4. **Create barrel export** in component's `index.ts`
5. **Update main components index** for package-level exports

```typescript
// packages/ui/src/components/Button/__tests__/Button.test.tsx
import { render, screen } from '@testing-library/react';
import { Button } from '../Button';

describe('Button', () => {
  it('renders children correctly', () => {
    render(<Button variant="primary" onClick={() => {}}>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });
});

// packages/ui/src/components/Button/Button.tsx
export interface ButtonProps {
  variant: 'primary' | 'secondary';
  onClick: () => void;
  children: React.ReactNode;
}

export function Button({ variant, onClick, children }: ButtonProps) {
  // Implementation
}

// packages/ui/src/components/Button/index.ts
export { Button, type ButtonProps } from './Button';
```

### Rust Module Development

1. **Write tests first** in `#[cfg(test)] mod tests {}` block
2. **Implement functionality** with proper error handling (no `unwrap()` or `expect()`)
3. **Verify tests pass** — Run `cargo test --workspace`
4. **Update Cargo.toml dependencies** if needed
5. **Update module exports** in `lib.rs`

```rust
// Test first (RED)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let result = process_data("test input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, "processed: test input");
    }
}

// Implementation (GREEN)
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessError> {
    Ok(ProcessedData {
        value: format!("processed: {}", input),
    })
}
```

### Tauri Command Integration

1. **Write Rust tests first** for the Tauri command
2. **Implement Tauri command** in `packages/tauri-app/src-tauri/src/`
3. **Update capabilities and permissions** in JSON configuration files
4. **Write TypeScript tests** for the service wrapper
5. **Create TypeScript service wrapper** for frontend integration
6. **Verify all tests pass**

```rust
// Tauri command with test
#[tauri::command]
async fn process_file(path: String) -> Result<ProcessResult, String> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_file() {
        let result = process_file("test.txt".to_string()).await;
        assert!(result.is_ok());
    }
}
```

```typescript
// TypeScript service wrapper with test
import { describe, it, expect, vi } from 'vitest';

describe('processFile', () => {
  it('calls Tauri command with correct parameters', async () => {
    const result = await processFile('test.txt');
    expect(result).toBeDefined();
  });
});

export async function processFile(path: string): Promise<ProcessResult> {
  return await invoke('process_file', { path });
}
```

### Refactoring and Code Quality

1. **Ensure all tests pass** before refactoring
2. **Extract shared modules** to `packages/ui/src/lib/*.ts` or `packages/core/src/`
3. **Simplify component interfaces** by reducing prop complexity
4. **Update imports and dependencies** across affected files
5. **Verify tests still pass** after refactoring

## Testing Patterns

### Test Frameworks
- **Rust**: Built-in `cargo test`, with `mockall`, `tokio-test`, `testcontainers` for integration tests
- **TypeScript**: `vitest` with `@testing-library/react`, `@testing-library/user-event`, `msw` for mocking

### Test Structure

**Rust:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        // Arrange
        let input = "test";
        // Act
        let result = function_under_test(input);
        // Assert
        assert_eq!(result, expected);
    }
}
```

**TypeScript:**
```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Component } from './Component';

describe('Component', () => {
  beforeEach(() => {
    // Setup
  });

  it('should render correctly with props', () => {
    render(<Component prop="value" />);
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });

  it('should handle user interactions', async () => {
    const user = userEvent.setup();
    render(<Component />);
    await user.click(screen.getByRole('button'));
    expect(screen.getByText('Result')).toBeInTheDocument();
  });
});
```

### Coverage Targets
- Rust core: 90% line coverage (unit), 100% of critical paths (integration)
- TypeScript UI: 80% line coverage
- E2E: 100% of user-facing critical paths

## Remember

1. **TDD is mandatory** — Write tests before production code, always
2. **No security violations** — Never log secrets, never store keys on disk
3. **Architecture boundaries** — Rust core knows nothing about UI, UI accesses nothing directly
4. **Error handling** — No `unwrap()` in production Rust code, no `any` in TypeScript
5. **Conventional commits** — Use proper prefixes and keep messages concise
6. **Read the docs first** — Especially ARCHITECTURE.md, SECURITY.md, TDD.md before making changes