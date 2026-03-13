# tacoshell Development Patterns

> Auto-generated skill from repository analysis

## Overview

The tacoshell repository is a TypeScript-based monorepo that implements a cross-platform shell application using Rust core modules, Tauri for desktop integration, and a comprehensive UI component library. The codebase follows a modular architecture with strict separation between UI components, core Rust functionality, and platform-specific implementations.

## Coding Conventions

### File Naming
- Use **camelCase** for all TypeScript files
- Component files: `ComponentName.tsx`
- Test files: `ComponentName.test.tsx`
- Rust files: `snake_case.rs` (types.rs, manager.rs, mod.rs)

### Import/Export Style
```typescript
// Relative imports preferred
import { Component } from './ComponentName'
import { useStore } from '../stores'

// Mixed export styles - both named and default exports
export { ComponentName } from './ComponentName'
export default ComponentName
```

### Commit Conventions
- Use conventional commit format: `type: description`
- Common prefixes: `feat`, `fix`, `docs`, `chore`, `ci`
- Keep messages around 62 characters
- Examples:
  ```
  feat: add new Button component with variants
  fix: resolve auth store state persistence
  docs: update component API documentation
  ```

## Workflows

### UI Component Implementation
**Trigger:** When adding new reusable UI components to the design system
**Command:** `/new-component`

1. Create component implementation in `packages/ui/src/components/{ComponentName}/{ComponentName}.tsx`
2. Add comprehensive test file in `packages/ui/src/components/{ComponentName}/__tests__/{ComponentName}.test.tsx`
3. Create barrel export in `packages/ui/src/components/{ComponentName}/index.ts`
4. Update main components index to export the new component

**Example Structure:**
```typescript
// packages/ui/src/components/Button/Button.tsx
export interface ButtonProps {
  variant?: 'primary' | 'secondary'
  children: React.ReactNode
}

export const Button = ({ variant = 'primary', children }: ButtonProps) => {
  return <button className={`btn-${variant}`}>{children}</button>
}

// packages/ui/src/components/Button/index.ts
export { Button } from './Button'
export type { ButtonProps } from './Button'
```

### Auth Feature Implementation
**Trigger:** When adding authentication-related functionality
**Command:** `/new-auth-feature`

1. Create auth store in `packages/ui/src/stores/`
2. Implement auth hooks in `packages/ui/src/features/auth/`
3. Create auth components in `packages/ui/src/features/auth/`
4. Add comprehensive tests for stores, hooks, and components
5. Update barrel exports in index.ts files

**Example Store Pattern:**
```typescript
// packages/ui/src/stores/useAuthStore.ts
import { create } from 'zustand'

interface AuthState {
  user: User | null
  login: (credentials: Credentials) => Promise<void>
  logout: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  login: async (credentials) => {
    // implementation
  },
  logout: () => set({ user: null })
}))
```

### Rust Core Module Implementation
**Trigger:** When adding new core Rust modules
**Command:** `/new-rust-module`

1. Create module directory in `packages/core/src/{module}/`
2. Add `types.rs` for data structures and enums
3. Add `manager.rs` or implementation files
4. Include comprehensive unit tests within the same files
5. Create `mod.rs` to expose module
6. Update `TODO.md` to mark phase completion
7. Update `Cargo.toml` dependencies if needed

**Example Module Structure:**
```rust
// packages/core/src/shell/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

// packages/core/src/shell/manager.rs
impl ShellManager {
    pub fn new() -> Self {
        Self { /* fields */ }
    }
    
    pub async fn execute_command(&self, cmd: Command) -> Result<Output> {
        // implementation with comprehensive error handling
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_execution() {
        // comprehensive test coverage
    }
}
```

### Tauri Integration
**Trigger:** When connecting Rust backend to desktop frontend
**Command:** `/tauri-integration`

1. Add Tauri commands in `packages/tauri-app/src-tauri/src/commands/`
2. Create frontend service wrappers in `packages/tauri-app/src/services/`
3. Add service tests in `packages/tauri-app/src/services/__tests__/`
4. Update Tauri configuration in `src-tauri/tauri.conf.json`
5. Update `Cargo.toml` workspace to include new Tauri modules

**Example Integration:**
```rust
// Tauri command
#[tauri::command]
pub async fn execute_shell_command(command: String) -> Result<String, String> {
    // implementation
}
```

```typescript
// Frontend service
import { invoke } from '@tauri-apps/api/tauri'

export const shellService = {
  executeCommand: async (command: string): Promise<string> => {
    return await invoke('execute_shell_command', { command })
  }
}
```

### Phase Completion
**Trigger:** When completing a major development milestone
**Command:** `/complete-phase`

1. Implement all features for the phase
2. Add comprehensive test coverage
3. Update `TODO.md` to mark phase completion and note test counts
4. Update configuration files (Cargo.toml, package.json, etc.)
5. Verify all tests pass

### Dependency and Configuration Updates
**Trigger:** When updating project dependencies or configuration
**Command:** `/update-deps`

1. Update `Cargo.toml` and `Cargo.lock` for Rust dependencies
2. Update `package.json` and `pnpm-lock.yaml` for Node dependencies
3. Update configuration files (eslint.config.js, tsconfig files, etc.)
4. Update workspace configuration files
5. Test that all packages still build and tests pass

## Testing Patterns

### Test Framework
- **Frontend:** Vitest for TypeScript/React components
- **Backend:** Built-in Rust testing with `#[cfg(test)]`

### Test File Patterns
- Frontend tests: `*.test.tsx` in `__tests__/` directories
- Rust tests: Inline with `#[cfg(test)]` modules

### Test Coverage Expectations
- All new components require comprehensive test coverage
- Store logic must be fully tested
- Rust modules include unit tests for public APIs
- Integration tests for Tauri commands

**Example Test Structure:**
```typescript
// Component test
import { render, screen } from '@testing-library/react'
import { Button } from '../Button'

describe('Button', () => {
  it('renders with correct variant class', () => {
    render(<Button variant="primary">Click me</Button>)
    expect(screen.getByRole('button')).toHaveClass('btn-primary')
  })
})
```

## Commands

| Command | Purpose |
|---------|---------|
| `/new-component` | Create a new UI component with tests and exports |
| `/new-auth-feature` | Add authentication functionality with stores and hooks |
| `/new-rust-module` | Create a new Rust core module with comprehensive tests |
| `/tauri-integration` | Connect Rust backend functionality to desktop frontend |
| `/complete-phase` | Mark development phase complete with TODO updates |
| `/update-deps` | Update project dependencies and configuration files |