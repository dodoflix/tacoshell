# tacoshell Development Patterns

> Auto-generated skill from repository analysis

## Overview

The tacoshell project is a TypeScript-based monorepo that combines a Tauri desktop application with a UI component library and Rust core functionality. It follows conventional commit patterns and emphasizes comprehensive testing with vitest. The codebase is structured as a multi-package workspace with clear separation between UI components, Tauri desktop integration, and core Rust modules.

## Coding Conventions

### File Naming
- Use **camelCase** for all TypeScript files: `userProfile.tsx`, `apiService.ts`
- Component files follow the pattern: `ComponentName.tsx`
- Test files use: `*.test.tsx` or `*.test.ts`

### Import/Export Style
- Use **relative imports** for local modules: `import { utils } from '../lib/utils'`
- **Mixed export style**: combination of named and default exports
- Barrel exports in `index.ts` files for clean package interfaces

### Commit Messages
- Follow **conventional commits** format
- Common prefixes: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`
- Keep messages around 63 characters average
- Examples:
  ```
  feat: add user authentication component
  fix: resolve memory leak in file watcher
  refactor: extract shared validation utilities
  ```

## Workflows

### Refactoring and Code Quality
**Trigger:** When code organization needs improvement, shared utilities should be extracted, or component props need simplification  
**Command:** `/refactor`

1. **Identify refactoring opportunities** in existing components
2. **Extract shared modules** to `packages/ui/src/lib/*.ts`
3. **Simplify component interfaces** by reducing prop complexity
4. **Update imports and dependencies** across affected files
5. **Add documentation comments** for improved maintainability

```typescript
// Before refactoring
export interface ComplexProps {
  userConfig: UserConfig;
  themeSettings: ThemeSettings;
  apiEndpoints: ApiEndpoints;
}

// After refactoring
export interface SimpleProps {
  config: AppConfig; // Combined configuration
}
```

### Feature Implementation with Tests
**Trigger:** When adding new features or capabilities to the application  
**Command:** `/feature`

1. **Create feature implementation files** in appropriate package directory
2. **Write comprehensive unit tests** using vitest framework
3. **Add integration tests** if the feature spans multiple components
4. **Update TypeScript definitions** and interfaces
5. **Update TODO.md** and relevant documentation

```typescript
// Feature implementation example
export function newFeature(input: FeatureInput): FeatureOutput {
  // Implementation
}

// Corresponding test
describe('newFeature', () => {
  it('should handle valid input correctly', () => {
    const result = newFeature(mockInput);
    expect(result).toEqual(expectedOutput);
  });
});
```

### UI Component Development
**Trigger:** When adding new components to the design system  
**Command:** `/ui-component`

1. **Create component directory** in `packages/ui/src/components/`
2. **Implement component** with proper TypeScript props interface
3. **Add comprehensive unit tests** in `__tests__` subdirectory
4. **Create barrel export** in component's `index.ts`
5. **Update main components index** for package-level exports

```typescript
// Component structure
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

### Rust Crate Development
**Trigger:** When adding new Rust functionality or modules  
**Command:** `/rust-module`

1. **Create Rust module files** in `packages/core/src/`
2. **Implement core functionality** with proper error handling
3. **Add comprehensive unit tests** with `#[cfg(test)]` modules
4. **Update Cargo.toml dependencies** if needed
5. **Update module exports** in `lib.rs` or `main.rs`

```rust
// Module implementation
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessError> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let result = process_data("test input");
        assert!(result.is_ok());
    }
}
```

### Tauri Desktop Integration
**Trigger:** When adding desktop functionality or Tauri commands  
**Command:** `/tauri-command`

1. **Create/update Tauri command** in `packages/tauri-app/src-tauri/src/`
2. **Update capabilities and permissions** in JSON configuration files
3. **Create TypeScript service wrapper** for frontend integration
4. **Add tests** for both Rust and TypeScript implementations
5. **Update Tauri configuration** files as needed

```rust
// Tauri command
#[tauri::command]
async fn process_file(path: String) -> Result<ProcessResult, String> {
    // Implementation
}

// TypeScript wrapper
export async function processFile(path: string): Promise<ProcessResult> {
  return await invoke('process_file', { path });
}
```

### Dependency and Configuration Updates
**Trigger:** When adding new dependencies or updating tooling configuration  
**Command:** `/update-deps`

1. **Update package.json or Cargo.toml** files with new dependencies
2. **Regenerate lock files** (Cargo.lock, package-lock.json)
3. **Adjust configuration files** (tsconfig.json, eslint.config.js)
4. **Update related imports/exports** throughout the codebase
5. **Test compatibility** across all packages

### Workspace-wide Fixes
**Trigger:** When fixes affect multiple packages or the entire workspace  
**Command:** `/workspace-fix`

1. **Identify all affected packages** in the monorepo
2. **Update imports and paths** consistently across packages
3. **Fix configuration issues** in build and tooling files
4. **Update build scripts** and package.json scripts
5. **Test changes** across all packages to ensure compatibility

## Testing Patterns

### Vitest Configuration
- Use **vitest** as the primary testing framework
- Test files follow the pattern: `*.test.tsx` for components, `*.test.ts` for utilities
- Place tests in `__tests__` directories or alongside source files

### Test Structure
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

  it('should handle user interactions', () => {
    // Interaction tests
  });
});
```

## Commands

| Command | Purpose |
|---------|---------|
| `/refactor` | Improve code organization and extract shared utilities |
| `/feature` | Implement new features with comprehensive test coverage |
| `/ui-component` | Create new UI components with tests and TypeScript definitions |
| `/rust-module` | Develop Rust modules with comprehensive test coverage |
| `/tauri-command` | Implement Tauri commands and desktop-specific functionality |
| `/update-deps` | Update project dependencies and configuration files |
| `/workspace-fix` | Apply fixes that span multiple packages in the monorepo |