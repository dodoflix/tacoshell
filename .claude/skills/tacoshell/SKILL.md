# tacoshell Development Patterns

> Auto-generated skill from repository analysis

## Overview

This skill covers development patterns for the `tacoshell` project, a TypeScript/Rust hybrid application using Tauri for desktop integration. The codebase follows a monorepo structure with conventional commits, comprehensive testing with Vitest, and a phased development approach. The project combines TypeScript UI components with Rust backend services in a cohesive desktop application.

## Coding Conventions

### File Naming
- Use **camelCase** for TypeScript files: `componentManager.ts`, `userService.ts`
- Follow Rust conventions for `.rs` files: `mod.rs`, `lib.rs`, `command_handler.rs`

### Import/Export Style
```typescript
// Relative imports preferred
import { Component } from './Component'
import { utils } from '../utils'

// Mixed export styles
export default MyComponent
export { MyComponent, MyComponentProps }
```

### Commit Messages
- Follow conventional commit format
- Use prefixes: `feat:`, `fix:`, `docs:`, `chore:`
- Keep messages around 61 characters
- Examples: `feat: add user authentication component`, `fix: resolve desktop integration issue`

## Workflows

### Phase Completion Workflow
**Trigger:** When completing a numbered development phase (1.1, 1.2, etc.)
**Command:** `/complete-phase`

1. Implement core functionality in appropriate packages (`packages/*/src/`)
2. Add comprehensive test coverage with specific test counts
   ```typescript
   // packages/ui/src/components/Button/__tests__/Button.test.tsx
   import { render, screen } from '@testing-library/react'
   import { Button } from '../Button'
   
   describe('Button', () => {
     it('renders correctly', () => {
       render(<Button>Click me</Button>)
       expect(screen.getByText('Click me')).toBeInTheDocument()
     })
   })
   ```
3. Update `TODO.md` marking phase as complete with ✅
4. Update configuration files (`Cargo.toml`, `package.json`)
5. Ensure all tests pass and commit with test count: `feat: complete phase 1.x (42 tests passing)`

### Rust Package Addition
**Trigger:** When adding new core functionality that needs a separate crate
**Command:** `/add-rust-crate`

1. Create new `Cargo.toml` for the crate:
   ```toml
   [package]
   name = "new-crate-name"
   version.workspace = true
   edition.workspace = true
   
   [dependencies]
   serde.workspace = true
   ```
2. Add crate to workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = ["packages/new-crate-name"]
   ```
3. Update `Cargo.lock` by running `cargo check`
4. Create `src/lib.rs` and module structure
5. Add workspace dependencies in `Cargo.workspace.toml`

### UI Component Implementation
**Trigger:** When adding new reusable UI components to the design system
**Command:** `/add-ui-component`

1. Create component directory: `packages/ui/src/components/ComponentName/`
2. Implement component with TypeScript:
   ```typescript
   // ComponentName.tsx
   import React from 'react'
   
   export interface ComponentNameProps {
     children: React.ReactNode
     variant?: 'primary' | 'secondary'
   }
   
   export const ComponentName: React.FC<ComponentNameProps> = ({
     children,
     variant = 'primary'
   }) => {
     return <div className={`component-${variant}`}>{children}</div>
   }
   ```
3. Add comprehensive test file in `__tests__/` directory
4. Create `index.ts` export file:
   ```typescript
   export { ComponentName, type ComponentNameProps } from './ComponentName'
   ```
5. Update main `components/index.ts`

### Tauri Desktop Integration
**Trigger:** When implementing desktop-specific features or fixing desktop integration
**Command:** `/update-tauri-integration`

1. Update Tauri Rust commands in `src-tauri/src/commands/`:
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String) -> Result<String, String> {
       // Implementation
       Ok(format!("Processed: {}", param))
   }
   ```
2. Update Tauri configuration files (`capabilities/default.json`, schemas)
3. Implement TypeScript service layer:
   ```typescript
   import { invoke } from '@tauri-apps/api/tauri'
   
   export class MyService {
     static async processData(data: string): Promise<string> {
       return await invoke('my_command', { param: data })
     }
   }
   ```
4. Add service integration tests
5. Update `Cargo.toml` and rebuild to update `Cargo.lock`

### Configuration and Tooling Setup
**Trigger:** When initializing project structure or updating development workflow
**Command:** `/setup-tooling`

1. Update workspace configuration (`pnpm-workspace.yaml`, `tsconfig.base.json`)
2. Configure linting and formatting:
   ```javascript
   // eslint.config.js
   export default {
     extends: ['@typescript-eslint/recommended'],
     rules: {
       '@typescript-eslint/no-unused-vars': 'error'
     }
   }
   ```
3. Setup CI/CD workflows in `.github/workflows/`
4. Configure package.json files across workspace
5. Update `.gitignore`, `.editorconfig`, `rustfmt.toml`, `clippy.toml`

## Testing Patterns

### Framework: Vitest
- Test files use pattern: `*.test.tsx` or `*.test.ts`
- Tests located in `__tests__/` directories within component folders
- Comprehensive test coverage expected for each phase completion
- Example test structure:
```typescript
import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'

describe('ComponentName', () => {
  it('should render with default props', () => {
    // Test implementation
  })
  
  it('should handle user interactions', () => {
    // Test implementation
  })
})
```

## Commands

| Command | Purpose |
|---------|---------|
| `/complete-phase` | Complete a numbered development phase with implementation, testing, and documentation |
| `/add-rust-crate` | Add new Rust crate to workspace with proper configuration |
| `/add-ui-component` | Implement new UI component with tests and exports |
| `/update-tauri-integration` | Update Tauri desktop functionality with Rust commands and TypeScript services |
| `/setup-tooling` | Configure project tooling, linting, and development workflow |