# tacoshell Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill teaches development patterns for tacoshell, a TypeScript project that follows conventional commit standards and modern testing practices. The codebase emphasizes clean architecture with camelCase naming conventions and comprehensive testing using Vitest.

## Coding Conventions

### File Naming
Use camelCase for all file names:
```
userService.ts
componentHelpers.ts
authManager.tsx
```

### Import Style
Use relative imports for project files:
```typescript
import { userService } from './userService'
import { ComponentHelpers } from '../helpers/componentHelpers'
import { AuthManager } from '../../auth/authManager'
```

### Export Style
Mix of named and default exports based on context:
```typescript
// Named exports for utilities
export const formatDate = (date: Date) => { ... }
export const validateInput = (input: string) => { ... }

// Default exports for main components/classes
export default class UserManager { ... }
```

### Commit Messages
Follow conventional commit format:
- `feat: add user authentication system`
- `fix: resolve memory leak in data processing`
- `docs: update API documentation`
- `chore: upgrade dependencies to latest versions`

Keep messages around 62 characters for consistency.

## Workflows

### Feature Development
**Trigger:** Adding new functionality
**Command:** `/add-feature`

1. Create feature branch following naming convention
2. Write failing tests first using Vitest
3. Implement feature with proper TypeScript typing
4. Ensure all tests pass
5. Commit with `feat:` prefix
6. Create pull request

### Bug Fix
**Trigger:** Fixing existing issues
**Command:** `/fix-bug`

1. Reproduce the bug with a test case
2. Identify root cause in codebase
3. Implement minimal fix
4. Verify fix resolves issue without side effects
5. Commit with `fix:` prefix
6. Update documentation if needed

### Refactoring
**Trigger:** Improving code quality without changing functionality
**Command:** `/refactor`

1. Ensure comprehensive test coverage exists
2. Make incremental changes
3. Run tests after each change
4. Commit with `refactor:` prefix
5. Verify no functionality changes

## Testing Patterns

### Test File Structure
Create test files with `.test.tsx` extension:
```typescript
// userService.test.tsx
import { describe, it, expect } from 'vitest'
import { userService } from './userService'

describe('userService', () => {
  it('should handle user authentication', () => {
    // Test implementation
    expect(userService.authenticate('user', 'pass')).toBe(true)
  })

  it('should validate user input', () => {
    // Test implementation
    expect(userService.validateInput('')).toBe(false)
  })
})
```

### Testing Best Practices
- Write descriptive test names
- Use `describe` blocks to group related tests
- Test both happy path and edge cases
- Mock external dependencies appropriately

## Commands
| Command | Purpose |
|---------|---------|
| `/add-feature` | Guide through feature development workflow |
| `/fix-bug` | Step through bug resolution process |
| `/refactor` | Help with safe code refactoring |
| `/test` | Generate test structure for new functionality |
| `/commit` | Format conventional commit message |