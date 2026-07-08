# Contributor Guide

Welcome to the Hermes Game Operator contributor guide! This document will help you get started with contributing to the project.

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Git
- A code editor (VS Code recommended)

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/acp-ui.git
cd acp-ui

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Project Structure

```
acp-ui/
├── src/                          # Frontend source code
│   ├── features/
│   │   └── game-operator/       # Game Operator UI
│   ├── api/                     # API clients
│   └── types/                   # TypeScript types
├── src-tauri/                   # Backend source code
│   └── src/
│       ├── operator/           # Operator modules
│       └── domains/games/godot/ # Godot domain pack
├── docs/                        # Documentation
├── skills/                      # Skill documentation
└── test-godot-project/         # Test project
```

## Contribution Workflow

### 1. Find an Issue

- Check [GitHub Issues](https://github.com/yourusername/acp-ui/issues)
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to claim it

### 2. Create a Branch

```bash
# Create a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Follow the coding standards
- Write tests for new features
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run tests
npm run test

# Type check
npm run typecheck

# Build
npm run build
```

### 5. Commit Your Changes

Use conventional commit format:

```bash
git add .
git commit -m "feat: add new feature"
# or
git commit -m "fix: resolve bug in module"
```

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Coding Standards

### TypeScript/Vue

#### File Naming

- Use PascalCase for components: `GameOperatorView.vue`
- Use camelCase for utilities: `operatorApi.ts`
- Use kebab-case for styles: `game-operator.css`

#### Component Structure

```vue
<script setup lang="ts">
// Imports
import { ref, computed } from 'vue'

// Props
interface Props {
  taskId: string
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  complete: [result: string]
}>()

// State
const loading = ref(false)

// Computed
const isValid = computed(() => /* ... */)

// Methods
async function handleSubmit() {
  // Implementation
}
</script>

<template>
  <!-- Template -->
</template>

<style scoped>
/* Styles */
</style>
```

#### Type Definitions

```typescript
// Good: Explicit types
interface Task {
  id: string
  status: TaskStatus
  goal: string
}

// Bad: Implicit any
const task = {} // No type annotation
```

#### Error Handling

```typescript
// Good: Proper error handling
try {
  const result = await riskyOperation()
  return result
} catch (error) {
  console.error('Operation failed:', error)
  throw new Error('User-friendly message')
}

// Bad: Silent failures
try {
  await riskyOperation()
} catch (error) {
  // Silent failure
}
```

### Rust

#### File Organization

```rust
// Module structure
pub mod types;
pub mod state_machine;
pub mod commands;

// Re-exports
pub use types::*;
pub use state_machine::TaskStateMachine;
```

#### Error Handling

```rust
// Good: Custom error types
#[derive(Debug)]
pub enum OperatorError {
    InvalidState(String),
    TaskNotFound(String),
}

impl std::fmt::Display for OperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            OperatorError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
        }
    }
}

// Bad: Using unwrap()
let value = map.get("key").unwrap(); // Can panic
```

#### Documentation

```rust
/// Calculate the next state based on current state and action
///
/// # Arguments
///
/// * `current` - The current task state
/// * `action` - The action to perform
///
/// # Returns
///
/// The next state if the transition is valid
///
/// # Errors
///
/// Returns an error if the transition is invalid
pub fn next_state(current: State, action: Action) -> Result<State, Error> {
    // Implementation
}
```

## Testing Guidelines

### Unit Tests

```typescript
// Example unit test
describe('TaskStateMachine', () => {
  it('should transition from Idle to Planning', () => {
    const machine = new TaskStateMachine()
    machine.startTask()
    expect(machine.currentStatus).toBe('planning')
  })
})
```

### Integration Tests

```typescript
// Example integration test
describe('Task Execution', () => {
  it('should execute complete workflow', async () => {
    const task = await OperatorApi.startTask({
      domain: 'game.godot',
      goal: 'Test task'
    })
    
    expect(task.status).toBe('planning')
    
    await OperatorApi.approve({
      task_id: task.task_id,
      decision: 'approve'
    })
    
    const updated = await OperatorApi.getTask(task.task_id)
    expect(updated.status).toBe('running')
  })
})
```

### E2E Tests

```typescript
// Example E2E test
test('complete task workflow', async ({ page }) => {
  await page.goto('/games')
  await page.fill('[data-testid="project-path"]', '/path/to/project')
  await page.fill('[data-testid="task-goal"]', 'Add double jump')
  await page.click('[data-testid="start-task"]')
  
  await expect(page.locator('[data-testid="task-status"]'))
    .toContainText('PLANNING')
})
```

## Documentation Guidelines

### API Documentation

```typescript
/**
 * Start a new operator task
 *
 * @param request - The task start request
 * @returns The created task
 *
 * @example
 * ```typescript
 * const task = await OperatorApi.startTask({
 *   domain: 'game.godot',
 *   goal: 'Add feature'
 * })
 * ```
 */
async function startTask(request: StartTaskRequest): Promise<OperatorTask> {
  // Implementation
}
```

### README Sections

- Clear description
- Installation instructions
- Usage examples
- API reference
- Contributing guide
- License

### Inline Comments

```typescript
// Good: Explain why, not what
// Use exponential backoff to avoid overwhelming the API
await delay(Math.pow(2, retryCount) * 1000)

// Bad: State the obvious
// Increment counter
counter++
```

## Review Process

### For Contributors

1. **Self-Review**
   - Check code quality
   - Run all tests
   - Update documentation
   - Ensure CI passes

2. **Request Review**
   - Tag appropriate reviewers
   - Provide context
   - Link related issues

3. **Address Feedback**
   - Respond to comments
   - Make requested changes
   - Re-request review

### For Reviewers

1. **Code Quality**
   - Follows coding standards
   - Proper error handling
   - No security issues
   - Performance considerations

2. **Testing**
   - Tests included
   - Tests pass
   - Edge cases covered

3. **Documentation**
   - Updated as needed
   - Clear and accurate
   - Examples provided

4. **Approve or Request Changes**
   - Provide constructive feedback
   - Explain reasoning
   - Suggest improvements

## Common Issues

### Build Fails

```bash
# Clean and rebuild
rm -rf node_modules
rm -rf dist
npm install
npm run build
```

### Tests Fail

```bash
# Run specific test
npm run test -- --grep "test name"

# Update snapshots
npm run test -- --updateSnapshot
```

### Type Errors

```bash
# Check types
npm run typecheck

# Fix type errors
# Read error messages carefully
```

## Getting Help

- **Documentation**: Check docs/ directory
- **Issues**: Search GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: Join our server
- **Email**: support@example.com

## Recognition

Contributors are recognized in:
- README.md
- CHANGELOG.md
- Release notes
- Contributors page

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Assume good intentions
- Help others learn and grow

---

**Thank you for contributing to Hermes Game Operator!** 🎉

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
