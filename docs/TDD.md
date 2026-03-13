# Tacoshell — TDD Workflow & Testing Strategy

## 1. Philosophy

Every feature in Tacoshell is built test-first. No production code is written without a failing test. This is non-negotiable and enforced by pre-commit hooks, CI gates, and coverage requirements.

The TDD loop is:
```
RED   → Write a failing test
GREEN → Write the minimum code to make it pass
REFACTOR → Improve the code without breaking tests
```

---

## 2. Testing Pyramid

```
                  ┌──────────────────┐
                  │   E2E Tests      │  ~10%
                  │  (Playwright)    │
                  └────────┬─────────┘
               ┌───────────┴────────────┐
               │   Integration Tests     │  ~30%
               │ (testcontainers, WASM)  │
               └───────────┬────────────┘
            ┌──────────────┴──────────────────┐
            │         Unit Tests               │  ~60%
            │  (cargo test, Vitest, RTL)        │
            └──────────────────────────────────┘
```

### Coverage Targets

| Layer | Target |
|-------|--------|
| Rust core (unit) | 90% line coverage |
| Rust core (integration) | 100% of critical paths |
| TypeScript UI | 80% line coverage |
| E2E | 100% of user-facing critical paths |

---

## 3. Testing Frameworks

### 3.1 Rust — Unit & Integration

```toml
# In Cargo.toml
[dev-dependencies]
mockall = "0.13"          # Mock trait implementations
tokio-test = "0.4"        # Async test utilities
testcontainers = "0.23"   # Docker-based integration tests
assert_matches = "1.5"    # Ergonomic assertions
pretty_assertions = "1.4" # Diff-style assertion failures
```

**Unit tests** live in the same file as the production code:
```rust
// src/crypto/cipher.rs

pub fn encrypt(key: &[u8; 32], data: &[u8]) -> Result<EncryptedEnvelope> { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_produces_different_ciphertext_each_time() {
        let key = [0u8; 32];
        let data = b"hello world";
        let a = encrypt(&key, data).unwrap();
        let b = encrypt(&key, data).unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn decrypt_round_trips_correctly() {
        let key = [0u8; 32];
        let data = b"hello world";
        let envelope = encrypt(&key, data).unwrap();
        let decrypted = decrypt(&key, &envelope).unwrap();
        assert_eq!(decrypted, data);
    }
}
```

**Integration tests** use Docker containers via testcontainers:
```rust
// tests/ssh_integration.rs
#[tokio::test]
async fn ssh_connect_and_run_command() {
    let container = SshdContainer::new().await;
    let profile = ConnectionProfile {
        host: container.host(),
        port: container.port(),
        username: "test".into(),
        auth: AuthMethod::Password("test".into()),
        ..Default::default()
    };
    let session = SshAdapter::connect(&profile).await.unwrap();
    let output = session.exec("echo hello").await.unwrap();
    assert_eq!(output.stdout.trim(), "hello");
}
```

### 3.2 TypeScript — Unit & Component

```json
// packages/ui/package.json devDependencies
{
  "vitest": "^1.x",
  "@testing-library/react": "^16.x",
  "@testing-library/user-event": "^14.x",
  "msw": "^2.x",
  "happy-dom": "^14.x"
}
```

**Component tests**:
```typescript
// src/features/tabs/__tests__/TabBar.test.tsx
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { TabBar } from '../TabBar'

describe('TabBar', () => {
  it('renders all tabs', () => {
    const tabs = [
      { id: '1', title: 'server-01', type: 'terminal' as const },
      { id: '2', title: 'server-02', type: 'terminal' as const },
    ]
    render(<TabBar tabs={tabs} activeTabId="1" onSelect={vi.fn()} onClose={vi.fn()} />)
    expect(screen.getByText('server-01')).toBeInTheDocument()
    expect(screen.getByText('server-02')).toBeInTheDocument()
  })

  it('calls onClose when close button is clicked', async () => {
    const onClose = vi.fn()
    const tabs = [{ id: '1', title: 'server-01', type: 'terminal' as const }]
    render(<TabBar tabs={tabs} activeTabId="1" onSelect={vi.fn()} onClose={onClose} />)
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledWith('1')
  })
})
```

**Store tests**:
```typescript
// src/stores/__tests__/useTabStore.test.ts
import { act, renderHook } from '@testing-library/react'
import { useTabStore } from '../useTabStore'

beforeEach(() => useTabStore.setState(useTabStore.getInitialState()))

describe('useTabStore — split', () => {
  it('converts a pane into a split node', () => {
    const { result } = renderHook(() => useTabStore())
    const paneId = result.current.rootPane.id
    act(() => result.current.splitPane(paneId, 'horizontal'))
    expect(result.current.root.type).toBe('split')
    expect(result.current.root.direction).toBe('horizontal')
  })
})
```

### 3.3 WASM Tests

```bash
# Run in headless Chrome (matches browser environment)
wasm-pack test packages/core-wasm --headless --chrome

# Run in Node.js
wasm-pack test packages/core-wasm --node
```

WASM tests validate that the WASM bindings produce identical results to the native Rust functions.

### 3.4 E2E Tests (Playwright)

```typescript
// e2e/connect-ssh.spec.ts
import { test, expect } from '@playwright/test'

test('user can connect to an SSH server', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Login with GitHub' }).click()
  // GitHub OAuth mock in CI
  await page.waitForURL('/dashboard')

  await page.getByRole('button', { name: 'New Connection' }).click()
  await page.getByLabel('Host').fill('localhost')
  await page.getByLabel('Port').fill('22')
  await page.getByLabel('Username').fill('test')
  await page.getByLabel('Password').fill('test')
  await page.getByRole('button', { name: 'Connect' }).click()

  await expect(page.getByTestId('terminal')).toBeVisible()
  await expect(page.getByText('Connected')).toBeVisible()
})

test('user can split a terminal tab horizontally', async ({ page }) => {
  // ...
})
```

---

## 4. TDD Enforcement

### 4.1 Pre-commit Hooks (husky + lint-staged)

```json
// .husky/pre-commit
#!/bin/sh
pnpm lint-staged
cargo test --workspace --quiet
```

```json
// package.json lint-staged config
{
  "lint-staged": {
    "packages/**/*.{ts,tsx}": ["eslint --fix", "vitest related --run"],
    "packages/**/*.rs": ["cargo fmt --check", "cargo clippy"]
  }
}
```

### 4.2 CI Coverage Gates

- `cargo-tarpaulin` generates Rust coverage → uploaded to Codecov
- `vitest --coverage` (v8 provider) generates TS coverage → uploaded to Codecov
- Codecov is configured to **block PRs** if coverage drops below thresholds
- New code paths with 0% coverage trigger a review requirement

### 4.3 Mutation Testing (Periodic)

Run weekly in CI:
```bash
# Rust
cargo mutants --workspace

# TypeScript
npx stryker run
```

Mutation score below 70% triggers a PR to add missing assertions.

---

## 5. Test Data and Fixtures

### Rust
- `tests/fixtures/` contains sample kubeconfig files, SSH keys (test-only, never real), and vault JSON blobs
- Docker images for integration tests: `tacoshell/test-sshd`, `tacoshell/test-vsftpd`, `rancher/k3s`

### TypeScript
- `packages/ui/src/__fixtures__/` contains typed factory functions (not `any` casts)
- MSW handlers in `packages/ui/src/mocks/` for GitHub API and Tauri IPC mocking

---

## 6. Test File Conventions

### Rust
- Unit tests: in the same file as production code, in a `#[cfg(test)] mod tests { }` block
- Integration tests: in `packages/core/tests/` directory
- File naming: same as production file (e.g., `cipher.rs` tests are in `cipher.rs`)

### TypeScript
- Unit/component tests: in `__tests__/` directories adjacent to the code
- File naming: `ComponentName.test.tsx` or `functionName.test.ts`
- E2E tests: in `e2e/` at the package root, named `feature-name.spec.ts`

---

## 7. Writing Tests First — Checklist

Before writing any production code:

- [ ] Identify the unit of behavior to test (not implementation details)
- [ ] Write a test that describes the expected behavior in plain language
- [ ] Run the test and confirm it fails (RED)
- [ ] Write the minimum production code to make it pass (GREEN)
- [ ] Run all tests to confirm nothing regressed
- [ ] Refactor if needed, keeping tests green (REFACTOR)
- [ ] Commit both test and production code together
