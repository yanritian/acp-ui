# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-06-22 — Loop Engine Edition

### Added

- **Loop Engine** — Unified orchestrator for all loop-based systems
  - `loop_engine.rs` — Rust backend with 4-layer architecture
  - `loop-api.ts` — Frontend API (4 Tauri commands)
  - `loop.ts` — Vue reactive store with health monitoring
  - `LoopDashboard.vue` — Real-time visualization panel

- **Architecture Documentation**
  - `docs/loop-engine-architecture.md` — 4-layer Loop design
  - `docs/RELEASE-v0.1.14.md` — Release notes

### Fixed — 17 Bug Fixes Verified

#### P0 — Critical (4 items)
- `goal_graph.rs:261-287` — `remove_goal()` logic bug (data leak)
- `goal_evaluator.rs:69-116` — Command injection vulnerability
- `client.ts:76-86` — WebSocket Promise leak on close
- `swarm_orchestrator.rs:886-912` — Fake data replaced

#### P1 — High (4 items)
- `remote-control/server.ts` — 8 fake handlers → real swarm-api
- `queen_lease.rs:280-297` — Dual-lock deadlock fixed
- `channel-registry.ts:168-180` — Hardcoded channel fixed
- `capability-evolver.ts` — Score calculation fixed

#### P2 — Medium (4 items)
- `kv-cache.ts:51-63` — 64-bit BigInt hash precision
- `model-router.ts:178` — CJK token estimation (1.5 tokens/char)
- `client.ts` — Backpressure dead code cleaned
- `capability-evolver.ts` — Unused import removed

#### P3 — Low (4 items)
- `goal_graph.rs` — Empty loop dead code removed
- `topology.rs` — Pipeline topology implemented
- `swarm_cancel_task` — Command naming unified
- `topology.rs` — `add_goal` errors now logged

### Changed

- 8 reserved modules marked `#![allow(dead_code)]` with docs
- `indexmap` upgraded to v2 (schemars compatibility)
- `orchestrator.ts`, `task-parser.ts` marked `@deprecated`

### Verified

- TypeScript: ✅ typecheck passes, 0 errors
- Unit tests: ✅ 294/294 passed
- Rust: ✅ cargo check passes, 0 warnings
- Build: ✅ npm run build succeeds

[0.2.1]: https://github.com/anthropics/acp-swarm/compare/v0.2.0...v0.2.1

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