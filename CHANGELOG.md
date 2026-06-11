# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-11

### Added

- **Goal-Driven Architecture** (RFC-001)
  - Goal data structure with 8 CompletionCondition types
  - Reconcile Loop for iterative execution
  - GoalGraph for dependency DAG management
  
- **Swarm Engine** crate
  - Star/Chain topology execution
  - Queen Lease election mechanism
  - Skill routing system
  
- **Tool Sandbox** crate
  - Denied patterns for .env files
  - Permission checker
  - Resource limits
  
- **Hook Runtime** crate
  - 12 new hook types (GoalSubmitted, WorkerRegistered, etc.)
  - Global and agent-specific hook registry
  
- **ACP Transport** crate
  - Stdio/WebSocket/HTTP adapters
  - Transport abstraction layer

- **Frontend Components**
  - GoalGraphView.vue - Goal state visualization
  - WorkerHealthPanel.vue - Worker health dashboard
  - GoalStore Pinia store integration

- **Documentation**
  - Getting Started guide
  - Worker integration guide
  - Protocol specifications (Worker/Goal/Event)

- **Examples**
  - hello-swarm.goal.yaml demo

### Changed

- Converted to Cargo workspace with 6 crates
- Removed dead code (hermes_traits.rs, finance/, adapter files)
- Unified Phase system (Phase 0-6)

### Fixed

- InstantWrapper time semantics (absolute timestamps)
- swarm-api.ts host abstraction layer
- remote-control/server.ts scaffold markers

## [0.1.14] - 2026-05-05

### Added

- Initial ACP-UI release
- Multi-agent support
- WebSocket remote agent connection
- Session management
- Permission controls

[0.2.0]: https://github.com/anthropics/acp-swarm/compare/v0.1.14...v0.2.0
[0.1.14]: https://github.com/anthropics/acp-swarm/releases/tag/v0.1.14