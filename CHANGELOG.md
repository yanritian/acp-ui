# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha] - 2026-07-11

### ✨ Major Updates

#### Highest Priority Defects Fixed (4/4)
- **DSK-001**: Fixed 1024x720 approval button accessibility
- **I18N-001**: Completed Game Operator internationalization
- **BACKEND-001**: Standardized backend event/error format
- **PLAT-001**: Implemented VSCode/IDEA clients

#### Internationalization (13 Languages)
Added complete support for 13 languages:
- zh-CN (简体中文), zh-TW (繁体中文), en-US (English)
- pt-BR (Português - Brasil), de-DE (Deutsch), es-ES (Español)
- ru-RU (Русский), ja-JP (日本語), ko-KR (한국어)
- vi-VN (Tiếng Việt), th-TH (ไทย), ms-MY (Bahasa Melayu), fr-FR (Français)

#### Testing Achievement
- **Total Tests**: 1,278 (up from 308)
- **Test Files**: 83
- **Pass Rate**: 100%
- **Coverage**: 85%+ statements, 80%+ branches

#### Documentation
- Added bilingual README (English/Chinese)
- Created comprehensive completion reports
- Added project summary and architecture docs
- Updated all documentation for v0.1.0-alpha

### 📊 Phase Completion
- **Phase A**: Core Infrastructure - ✅ 100%
- **Phase B**: Game Operator Implementation - ✅ 100%
- **Phase C**: Approval System - ✅ 100%
- **Phase D**: File Tools - ✅ 100%
- **Phase E**: Security Guards - ✅ 100%

### 🔗 Release
- **GitHub**: https://github.com/yanritian/acp-ui
- **Release**: https://github.com/yanritian/acp-ui/releases/tag/v0.1.0-alpha

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
