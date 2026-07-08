# Contributing to Hermes Game Operator

Thank you for your interest in contributing!

## Development Setup

### Prerequisites

- Node.js >= 18.x
- Rust >= 1.70
- Windows 10 SDK (for Windows builds)
- pnpm or npm

### Getting Started

```bash
# Clone the repository
git clone https://github.com/your-org/acp-ui.git
cd acp-ui

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

## Project Structure

```
src/                     # Vue frontend
├── features/
│   └── game-operator/   # Game Operator components
├── api/                 # API layer
└── types/               # TypeScript types

src-tauri/               # Rust backend
└── src/
    ├── operator/        # Operator control plane
    └── domains/         # Domain packs (Godot, etc.)

docs/                    # Documentation
tests/                   # Test files
```

## Coding Standards

### TypeScript/Vue
- Use composition API
- Follow Prettier formatting
- Add types for all props and returns
- No console.log in production code

### Rust
- Follow standard Rust conventions
- Use Result for error handling
- No unwrap() in production code
- Add tests for new functions

## Commit Guidelines

Use conventional commits:

```
feat: add new feature
fix: resolve bug
docs: update documentation
test: add tests
refactor: code cleanup
```

## Pull Request Process

1. Create a feature branch
2. Make your changes
3. Run tests: `npm run test`
4. Run build: `npm run build`
5. Submit PR with description

## Testing

```bash
# Unit tests
npm run test

# E2E tests
npm run test:e2e

# Rust tests
cd src-tauri && cargo test
```

## Security

- Never hardcode secrets
- Validate all user input
- Use PathGuard for file operations
- Use CommandGuard for shell commands

## Questions?

Open an issue on GitHub.
