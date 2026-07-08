# Changelog

All notable changes to Hermes Game Operator will be documented in this file.

## [0.1.0-alpha] - 2026-07-08

### Added

#### Core Features
- Task lifecycle management with 10 states
- Event stream with append-only storage
- Approval queue for safe execution
- Pause/Resume/Stop controls
- Task redirection support

#### Godot Integration
- Project detection (`project.godot`)
- Project analysis (scripts, scenes, assets)
- Player controller identification
- Scene parsing

#### Security
- PathGuard for path boundary validation
- CommandGuard for command allowlist
- Injection prevention for dangerous patterns
- API key validation

#### Hermes CLI Integration
- Subprocess bridge for Hermes CLI
- JSON event streaming
- Connection status checking
- Plan generation and step execution

#### API
- 20+ Tauri commands
- TypeScript type definitions
- Rust type definitions (matching)
- Frontend API layer

#### UI Components
- GameOperatorView (main view)
- OperatorControlBar (controls)
- ProgressTimeline (event stream)
- PlanPanel (execution plan)
- ApprovalDrawer (approvals)

### Fixed
- PathGuard now handles non-existent paths
- CommandGuard prevents injection attacks
- State machine has proper transition guards
- Mutex unwrap() replaced with error handling
- API key validation enforced

### Tests
- 301 unit tests
- 7 E2E tests
- Integration tests for Game Operator

### Documentation
- Configuration guide
- API reference
- Quick start guide
- Release checklist
- Execution breakdown
- Hermes integration plan

## [0.0.1] - 2026-06-01

### Added
- Initial project structure
- Basic Tauri + Vue setup
