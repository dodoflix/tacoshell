# tacoshell Development Patterns

> Auto-generated skill from repository analysis

## Overview

tacoshell is a Tauri-based desktop application built with TypeScript, featuring a modular monorepo architecture. The codebase follows a clear separation between Rust backend functionality (core logic and Tauri commands) and TypeScript frontend (UI components, services, and stores). The project emphasizes comprehensive testing, conventional commits, and phase-based development cycles.

## Coding Conventions

**File Naming:** camelCase for all TypeScript files
```
useAuthStore.ts
LoginComponent.tsx
fileService.ts
```

**Import Style:** Relative imports preferred
```typescript
import { Component } from '../components'
import { useAuth } from './hooks/useAuth'
```

**Export Style:** Mixed - both named and default exports
```typescript
// Named exports
export const useAuthStore = () => { ... }

// Default exports
export default LoginComponent
```

**Commit Format:** Conventional commits with specific prefixes
```
feat: add user authentication workflow
fix: resolve file upload validation issue
docs: update API documentation
chore: update dependencies
```

## Workflows

### Tauri Command Implementation
**Trigger:** When adding new backend capabilities accessible from the UI
**Command:** `/add-tauri-command`

1. Create new command function in `packages/tauri-app/src-tauri/src/commands/*.rs`
```rust
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    // Implementation
    Ok(result)
}
```

2. Register the command in `packages/tauri-app/src-tauri/src/lib.rs`
```rust
.invoke_handler(tauri::generate_handler![
    commands::existing_command,
    commands::new_command  // Add here
])
```

3. Create TypeScript service wrapper in `packages/tauri-app/src/services/*.ts`
```typescript
import { invoke } from '@tauri-apps/api/tauri'

export const newCommandService = async (param: string): Promise<string> => {
  return await invoke('new_command', { param })
}
```

4. Add corresponding tests in `packages/tauri-app/src/services/__tests__/*.test.ts`

### UI Component Development
**Trigger:** When creating new reusable UI components
**Command:** `/new-component`

1. Create component file `packages/ui/src/components/ComponentName/ComponentName.tsx`
```typescript
import React from 'react'

interface ComponentNameProps {
  // Define props
}

export const ComponentName: React.FC<ComponentNameProps> = (props) => {
  return (
    <div>
      {/* Component implementation */}
    </div>
  )
}
```

2. Add comprehensive tests in `packages/ui/src/components/ComponentName/__tests__/ComponentName.test.tsx`
```typescript
import { render, screen } from '@testing-library/react'
import { ComponentName } from '../ComponentName'

describe('ComponentName', () => {
  it('should render correctly', () => {
    render(<ComponentName />)
    // Test assertions
  })
})
```

3. Create barrel export `packages/ui/src/components/ComponentName/index.ts`
```typescript
export { ComponentName } from './ComponentName'
export type { ComponentNameProps } from './ComponentName'
```

4. Update `packages/ui/src/components/index.ts` to export the new component

### Auth Feature Development
**Trigger:** When adding authentication functionality
**Command:** `/add-auth-feature`

1. Create hook in `packages/ui/src/features/auth/use*.ts`
```typescript
import { useState, useEffect } from 'react'

export const useAuthFeature = () => {
  const [state, setState] = useState(initialState)
  
  // Hook implementation
  
  return { state, actions }
}
```

2. Add corresponding tests `packages/ui/src/features/auth/__tests__/use*.test.ts`
3. Create related component `packages/ui/src/features/auth/*Component.tsx`
4. Add component tests `packages/ui/src/features/auth/__tests__/*Component.test.tsx`
5. Update store `packages/ui/src/stores/useAuthStore.ts` with new functionality and tests

### Core Module Implementation
**Trigger:** When adding new core Rust functionality modules
**Command:** `/add-core-module`

1. Create new module file `packages/core/src/module_name/mod.rs`
```rust
pub mod submodule;

pub use submodule::*;

// Module-level functionality
```

2. Implement specific functionality in `packages/core/src/module_name/*.rs` files
3. Add comprehensive unit tests within the module files
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Test implementation
    }
}
```

4. Update `packages/core/src/lib.rs` to expose the module
```rust
pub mod module_name;
pub use module_name::*;
```

### Cargo Dependency Management
**Trigger:** When adding new Rust crate dependencies
**Command:** `/add-rust-dep`

1. Add dependency to relevant package `Cargo.toml`
```toml
[dependencies]
new-crate = "1.0.0"
```

2. Update workspace configuration if needed
3. Run `cargo update` to update `Cargo.lock`
4. Test integration and update any affected modules

### Project Phase Completion
**Trigger:** When finishing a major project phase
**Command:** `/complete-phase`

1. Implement all functionality for the phase across multiple packages
2. Add comprehensive test coverage (aim for high test counts as seen in commits)
3. Update `TODO.md` to mark phase as complete
```markdown
## Phase X.X - Feature Name ✅
- [x] Feature implementation
- [x] Tests: XX unit, XX integration
```

4. Update documentation and configuration files as needed

## Testing Patterns

**Framework:** vitest for all TypeScript tests
**File Pattern:** `*.test.tsx` for components, `*.test.ts` for utilities/hooks

**Test Structure:**
```typescript
import { describe, it, expect } from 'vitest'

describe('ComponentName', () => {
  it('should handle specific behavior', () => {
    // Arrange
    // Act  
    // Assert
    expect(result).toBe(expected)
  })
})
```

**Rust Tests:** Inline with modules using `#[cfg(test)]`

## Commands

| Command | Purpose |
|---------|---------|
| `/add-tauri-command` | Add new backend command with frontend integration |
| `/new-component` | Create new UI component with tests and exports |
| `/add-auth-feature` | Add authentication-related functionality |
| `/add-core-module` | Create new core Rust module with tests |
| `/add-rust-dep` | Add and configure Rust dependencies |
| `/complete-phase` | Finalize development phase with documentation |