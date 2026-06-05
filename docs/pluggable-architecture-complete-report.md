# ACP-UI Pluggable Architecture - Complete Change Report

**Branch**: `refactor/project-cleanup` (based on `my-agent-teams-platform` @ commit 3a848f4)
**Date**: 2026-06-05
**Total Commits**: 3 (cac9673, a75bb2e, 872b243)

## Overview

This document describes all changes made to transform ACP-UI from a monolithic Tauri app into a fully pluggable agent platform. The architecture supports Skill/MCP/Hook/CLI/Adapter plugins, Agent Swarm orchestration (Codex + Claude Code as top-level agents), Ultra Workflow Engine, OpenClacky-style token optimization, and agent-to-agent communication.

---

## Commit 1: Project Cleanup (cac9673)

### Problem
- Root directory cluttered with 80+ screenshots, scripts, temp folders
- TypeScript strict mode disabled (`strict: false`, `noImplicitAny: false`)
- `lib.rs` at 2157 lines (monolithic command handler)
- `App.vue` at 1724 lines (no routing, no composables)
- No code splitting or lazy loading

### Changes

**Root Directory Cleanup**:
- `.gitignore`: Updated to exclude root-level scripts (`/*.py`, `/*.bat`), `_cleanup/`, `workspace/`, `flutter/`, `hermes/`
- Screenshots moved to `docs/screenshots/`
- Scripts moved to `docs/`

**TypeScript Strict Mode** (`tsconfig.json`):
- `"strict": false` -> `"strict": true`
- `"noImplicitAny": false` -> `"noImplicitAny": true`
- Fixed 10 resulting type errors across 5 files

**lib.rs Split** (2157 -> 291 lines):
- 14 new command modules in `src-tauri/src/commands/`:
  - `mod.rs`, `config.rs`, `agent_lifecycle.rs`, `task_history.rs`, `memory.rs`
  - `self_healing.rs`, `self_evolution.rs`, `gateway.rs`, `websocket_cmds.rs`
  - `log_stream_cmds.rs`, `permission.rs`, `agent_config.rs`, `executive.rs`, `teams.rs`

**App.vue Refactoring** (1724 -> 455 lines):
- `src/router.ts`: Vue Router with 22 lazy-loaded routes using `createWebHashHistory()` (hash mode for Tauri WebView compatibility)
- 4 composables extracted: `useResponsiveLayout.ts`, `useReconnect.ts`, `usePreferences.ts`, `useBotCommand.ts`
- 4 sub-components extracted: `AppSidebar.vue`, `ConnectionBanner.vue`, `WelcomeScreen.vue`, `StatusView.vue`, `MonitorView.vue`
- `src/lib/mock-task-dag.ts`: Mock DAG data extracted

---

## Commit 2: Pluggable Architecture Core (a75bb2e)

### Plugin Registry (`src-tauri/src/plugin_registry.rs`, ~520 lines)

Unified plugin system treating all capabilities as first-class plugins:

```rust
pub enum PluginKind { Skill, Mcp, Hook, Cli, Adapter }

pub struct PluginMeta {
    id: String,
    name: String,
    kind: PluginKind,
    version: String,
    enabled: bool,
    health_score: f64,
    config: HashMap<String, serde_json::Value>,
    // ...
}
```

**9 Tauri commands**: `plugin_list`, `plugin_register`, `plugin_unregister`, `plugin_get`, `plugin_set_enabled`, `plugin_update_config`, `plugin_search`, `plugin_get_stats`, `plugin_get_history`

**Default plugins seeded**: 16 core skills + 2 MCP servers

### Swarm Orchestrator (`src-tauri/src/swarm_orchestrator.rs`, ~680 lines)

Top-level agent coordination (Codex + Claude Code as peers, not sub-agents):

```rust
pub enum SwarmTopology { Hierarchical, Mesh, Pipeline, Star, Adaptive }
pub enum AgentRole { Orchestrator, Executor, Reviewer, Researcher, Specialist }
pub enum ConsensusStrategy { FirstWins, Majority, BestScore, Merge, Adversarial }
```

**7 Tauri commands**: `swarm_register_agent`, `swarm_list_agents`, `swarm_create_task`, `swarm_submit_result`, `swarm_get_task`, `swarm_get_health`, `swarm_cancel_task`

### Ultra Workflow Engine (`src-tauri/src/workflow_engine.rs`, ~750 lines)

Multi-stage orchestrated workflows with DAG validation:

```rust
pub enum StageStrategy { Sequential, Parallel, Conditional, Loop, MapReduce }
pub enum FailurePolicy { StopOnFailure, ContinueOnError, RetryN, Fallback }
```

**10 Tauri commands**: `workflow_create`, `workflow_validate`, `workflow_list`, `workflow_get`, `workflow_get_progress`, `workflow_update_status`, `workflow_submit_result`, `workflow_generate_from_task`, `workflow_cancel`, `workflow_save`

### Frontend Services & Stores
- `src/lib/plugin-system/`: TypeScript service layer (`plugin-service.ts`, `swarm-service.ts`, `workflow-service.ts`)
- `src/stores/plugin-registry.ts`: Pinia store with filtered plugins, health counts
- `src/stores/swarm.ts`: Pinia store with agents, tasks, health

---

## Commit 3: Complete Architecture (872b243)

### MCP JSON-RPC Client (`src-tauri/src/mcp_client.rs`, ~450 lines)

Full MCP protocol communication layer:

```rust
pub struct McpClient {
    connections: HashMap<String, McpConnection>,
    tool_cache: HashMap<String, Vec<McpTool>>,
}

struct McpConnection {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    server_info: ServerInfo,
}
```

**8 Tauri commands**: `mcp_list_tools`, `mcp_list_server_tools`, `mcp_call_tool`, `mcp_connected_servers`, `mcp_is_connected`, `mcp_connect`, `mcp_disconnect`, `mcp_get_stats`

### Agent Communication Bus (`src-tauri/src/agent_bus.rs`, ~500 lines)

Agent-to-agent messaging with mailbox + pub/sub:

```rust
pub struct AgentBus {
    mailboxes: HashMap<String, VecDeque<AgentMessage>>,
    topics: HashMap<String, Vec<String>>,
    message_history: Vec<AgentMessage>,
    // ...
}
```

**13 Tauri commands**: `agent_bus_register`, `agent_bus_unregister`, `agent_bus_send`, `agent_bus_receive`, `agent_bus_peek`, `agent_bus_inbox_count`, `agent_bus_subscribe`, `agent_bus_publish`, `agent_bus_stats`, `agent_bus_history`, `agent_bus_list_agents`, `agent_bus_list_topics`, `agent_bus_is_registered`

### Hook Executor Integration (`src-tauri/src/hooks_executor.rs`)

Added 8 Tauri commands to expose the existing HooksExecutor:

- `hook_register`, `hook_register_for_agent`, `hook_unregister_agent`
- `hook_execute_pre_tool`, `hook_execute_post_tool`, `hook_execute_post_tool_failure`
- `hook_list`, `hook_get_agent_hooks`

These enable Pre/Post tool use hooks in the agent execution lifecycle.

### Self-Healing Executor (`src-tauri/src/self_healing.rs`)

Closed-loop EWMA detection -> auto-repair -> resolution:

```rust
pub struct HealingExecutor {
    actions: HashMap<String, HealingActionRecord>,
    anomaly_detector: Arc<Mutex<AnomalyDetector>>,
}
```

**4 Tauri commands**: `healing_execute`, `healing_list_actions`, `healing_resolve_action`, `healing_get_stats`

Action types: `RestartAgent`, `SwitchModel`, `CompressContext`, `ClearCache`, `Reconnect`, `ScaleUp`, `ScaleDown`, `NotifyUser`

### Feishu Webhook Receiver (`src-tauri/src/bot_adapters/feishu.rs`)

Axum-based HTTP server for receiving Feishu events:
- `POST /webhook/challenge`: URL verification (echoes back `challenge` field)
- `POST /webhook/event`: Event callback handling (v2.0 schema, `im.message.receive_v1`)
- Auto-reply via `send_message()` + `handle_command()` pipeline
- Configurable port (default 9876)

**Dependencies added**: `axum = "0.7"`, `tracing = "0.1"`

### Token Optimizer (`src/lib/agent-runtime/token-optimizer.ts`, ~660 lines)

OpenClacky-style token optimization (TypeScript):
- `freezeSystemPrompt()`: Dual-cache markers for frozen system prompts
- `buildSessionContext()`: Session context with cached/active sections
- `compressContext()`: Insert-then-Compress pattern
- `IdleCompressionTimer`: Auto-compress after idle period
- `ArchivedChunk`: Compressed history with AI-lookback references

### Frontend Components

**PluginManagerView.vue** (~350 lines): Card grid with search, kind filter, health indicators, enable/disable toggle, details panel

**SwarmDashboard.vue** (~292 lines): Agent list with role badges, status indicators, token usage bars, task list, create task form

**WorkflowEditor.vue** (~391 lines): Workflow list, stage timeline visualization, progress bar, create form with "Generate from task"

**TokenOptimizerPanel.vue** (~300 lines): Context window gauge, system prompt freeze status, dual-cache hit/miss stats, compression metrics, idle timer, archived chunks list, manual "Compress Now" button

### Feature Registry & Router Updates

4 new features registered:
- `plugins` -> `/plugins` -> `PluginManagerView.vue`
- `swarm-dashboard` -> `/swarm-dashboard` -> `SwarmDashboard.vue`
- `workflow-editor` -> `/workflow-editor` -> `WorkflowEditor.vue`
- `token-optimizer` -> `/token-optimizer` -> `TokenOptimizerPanel.vue`

---

## Architecture Summary

### Tauri Command Count (Total: ~120+)

| Module | Commands |
|--------|----------|
| Config | 4 |
| Agent Lifecycle | 12 |
| Task History | 7 |
| Memory | 5 |
| Self-Healing (DB) | 5 |
| Self-Evolution | 5 |
| Gateway | 6 |
| WebSocket | 5 |
| LogStream | 8 |
| Permission | 4 |
| Agent Config | 6 |
| Executive Agent | 11 |
| Agent Teams | 8 |
| Skills | 6 |
| Plugin Registry | 9 |
| Swarm Orchestrator | 7 |
| Workflow Engine | 10 |
| Hooks Executor | 8 |
| Healing Executor | 4 |
| MCP Client | 8 |
| Agent Bus | 13 |

### Key Design Decisions

1. **Hash-mode routing** (`createWebHashHistory`) for Tauri WebView compatibility
2. **All plugins as first-class citizens**: Skills, MCP servers, Hooks, CLI tools, and Bot Adapters share the same PluginRegistry
3. **Swarm topology**: Codex and Claude Code are top-level agents (not sub-agents), coordinated by the SwarmOrchestrator
4. **MCP over stdio**: JSON-RPC 2.0 via stdin/stdout for maximum compatibility with existing MCP servers
5. **Agent Bus over WebSocket**: Internal mailbox pattern for agent-to-agent messaging, pub/sub for topic-based broadcasting
6. **EWMA anomaly detection**: Exponentially weighted moving average for dynamic baseline, with closed-loop auto-repair

### Compilation Status

- `cargo check`: PASS (0 errors, 31 warnings - all unused items)
- `vue-tsc --noEmit`: PASS (0 errors)

### Files Changed Summary

- **3 commits** total
- **~30 files** modified or created
- **~8,500 lines** of new code added
- **Rust backend**: ~5,000 lines (mcp_client, agent_bus, plugin_registry, swarm_orchestrator, workflow_engine, hooks_executor, self_healing enhancements, feishu webhook)
- **TypeScript frontend**: ~3,500 lines (4 Vue components, token optimizer, services, stores)
