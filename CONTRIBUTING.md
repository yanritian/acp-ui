# Contributing to ACP UI

Thank you for your interest in contributing to ACP UI! This document provides guidelines and information for contributors.

## 🎯 Project Overview

ACP UI is a modern, cross-platform client for the Agent Client Protocol (ACP). It includes the Hermes Game Operator, an AI-powered game development assistant for Godot Engine.

## 🚀 Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Platform-specific build tools (see [Tauri Prerequisites](https://tauri.app/start/prerequisites/))

### Setup

```bash
# Clone the repository
git clone https://github.com/formulahendry/acp-ui.git
cd acp-ui

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## 📋 Development Workflow

### 1. Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally
3. Add the upstream repository as a remote

```bash
git remote add upstream https://github.com/formulahendry/acp-ui.git
```

### 2. Create a Branch

Create a branch for your changes:

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions or changes

### 3. Make Changes

- Write clean, readable code
- Follow existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run TypeScript type checking
npm run typecheck

# Run tests
npm run test

# Build the project
npm run build
```

### 5. Commit Your Changes

Use conventional commit messages:

```
feat: add new feature
fix: resolve bug
docs: update documentation
test: add tests
refactor: refactor code
chore: update dependencies
```

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## 🎮 Hermes Game Operator

The Hermes Game Operator is a key component of ACP UI. When contributing to this area:

### Architecture

- **Frontend**: Vue 3 + TypeScript
  - `src/features/game-operator/` - UI components
  - `src/api/operatorApi.ts` - API client
  - `src/types/operator.ts` - Type definitions

- **Backend**: Rust + Tauri
  - `src-tauri/src/operator/` - Operator modules
  - `src-tauri/src/domains/games/godot/` - Godot domain pack

### Key Modules

1. **State Machine** (`state_machine.rs`)
   - 10-state task lifecycle
   - State transitions and validation

2. **Commands** (`commands.rs`)
   - 17 Tauri commands
   - Task management and control

3. **Security** (`security.rs`)
   - PathGuard for path validation
   - CommandGuard for command whitelisting

4. **File Tools** (`file_tools.rs`)
   - Safe file operations
   - Backup and diff generation

5. **Agent Bridge** (`agent_bridge.rs`)
   - Hermes Agent integration
   - Task execution

### Testing

- Unit tests in each module
- E2E tests in `e2e_tests.rs`
- Test project in `test-godot-project/`

## 📝 Code Style

### TypeScript/Vue

- Use TypeScript for all new code
- Follow Vue 3 Composition API patterns
- Use proper type definitions
- No `any` types unless absolutely necessary

### Rust

- Follow Rust naming conventions
- Use proper error handling
- Document public APIs
- Write unit tests

### Documentation

- Update README.md for user-facing changes
- Update API docs for API changes
- Add inline comments for complex logic
- Keep documentation in sync with code

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Step-by-step instructions
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**: OS, version, etc.
6. **Logs**: Any relevant error messages

## 💡 Feature Requests

For feature requests:

1. Check existing issues first
2. Describe the problem you're trying to solve
3. Explain your proposed solution
4. Consider alternative approaches
5. Be open to discussion

## 📚 Documentation

When adding new features:

1. Update API documentation
2. Add usage examples
3. Update relevant guides
4. Add inline code comments

## 🔒 Security

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email security@example.com
3. Include detailed description
4. Wait for response before disclosing

## 📊 Pull Request Process

1. **Review**: Your PR will be reviewed by maintainers
2. **Changes**: Address any requested changes
3. **Tests**: Ensure all tests pass
4. **Documentation**: Update documentation as needed
5. **Merge**: Once approved, your PR will be merged

## 🎯 Review Guidelines

When reviewing PRs:

- Check code quality and style
- Verify tests are included
- Ensure documentation is updated
- Test the changes locally
- Provide constructive feedback

## 📖 Resources

- [API Documentation](docs/api.md)
- [Project Summary](docs/PROJECT-SUMMARY.md)
- [Test Plan](docs/codex/test-plan.md)
- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)

## 🤝 Community

- Be respectful and inclusive
- Help others learn and grow
- Share knowledge and experience
- Celebrate contributions

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to ACP UI! 🎉