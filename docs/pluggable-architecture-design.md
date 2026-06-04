# ACP-UI Pluggable Architecture Design

## Core Principle

> **ACP is the control & communication protocol. Everything else is a plugin.**

All capabilities — Skills, MCP servers, Hooks, CLI extensions, Bot Adapters — are first-class
plugins that register through a unified Plugin Registry. The ACP protocol handles message routing,
session management, and permission control between the UI client and agent processes.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    UI Layer (Vue 3 + Router)                 │
│  Chat │ Swarm Dashboard │ Workflow Editor │ Plugin Manager   │
├─────────────────────────────────────────────────────────────┤
│              Swarm Orchestrator (Rust)                       │
│  Topology │ Consensus │ Agent Routing │ Result Aggregation   │
├─────────────────────────────────────────────────────────────┤
│              Ultra Workflow Engine (Rust)                    │
│  Script Generation │ Stage Execution │ Progress Tracking     │
├─────────────────────────────────────────────────────────────┤
│                 Plugin Registry (Rust)                       │
│  Skills │ MCP Servers │ Hooks │ CLI Extensions │ Adapters    │
├─────────────────────────────────────────────────────────────┤
│            ACP Protocol Layer (TypeScript + Rust)            │
│  JSON-RPC │ Sessions │ Transport (Stdio/WS/HTTP) │ Auth     │
├─────────────────────────────────────────────────────────────┤
│              Token Optimizer (TypeScript)                    │
│  Context Compression │ Dual-Cache │ Chunk Archiving          │
├─────────────────────────────────────────────────────────────┤
│            Self-Evolution Engine (Rust + TypeScript)         │
│  EWMA Detection │ Auto-Healing │ Pattern Learning │ Reflect  │
└─────────────────────────────────────────────────────────────┘
```

## Plugin System

### Plugin Trait (Rust)

Every capability implements the `Plugin` trait:

```rust
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn kind(&self) -> PluginKind;  // Skill | Mcp | Hook | Cli | Adapter
    fn capabilities(&self) -> Vec<Capability>;
    fn activate(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn deactivate(&mut self) -> Result<(), PluginError>;
    fn handle_message(&self, msg: PluginMessage) -> Result<PluginResponse, PluginError>;
}
```

### Plugin Registry

Central registry that manages plugin lifecycle:
- `register(plugin)` — Register a new plugin
- `unregister(id)` — Remove a plugin
- `discover(kind)` — List plugins by kind
- `invoke(id, message)` — Send message to a plugin
- `health_check(id)` — Check plugin health

### Plugin Kinds

| Kind | Description | Example |
|------|-------------|---------|
| Skill | Reusable agent capabilities | code-review, web-research |
| MCP | Model Context Protocol servers | filesystem, git, database |
| Hook | Agent lifecycle interceptors | pre-tool-check, post-repair |
| CLI | Command-line extensions | custom commands, integrations |
| Adapter | Platform bridges | telegram, feishu, discord |

## Swarm Orchestrator

### Top-Level Agent Pattern

Codex and Claude Code are treated as **peer agents** in the swarm, not as sub-agents:

```
User Request → Swarm Router → Topology Selector → Agent Assignment
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              Claude Code        Codex           Custom Agent
              (Orchestrator)    (Executor)       (Specialist)
```

### Topology Types

- **Hierarchical**: Leader agent delegates to workers (default)
- **Mesh**: All agents communicate peer-to-peer
- **Pipeline**: Sequential chain of agents
- **Star**: Central coordinator with peripheral agents
- **Adaptive**: Dynamic topology selection based on task

### Consensus Strategies

- **Raft**: Strong consistency with leader election
- **Majority**: Democratic voting on results
- **First-Wins**: Fastest agent's result accepted
- **Merge**: Combine results from multiple agents

## Ultra Workflow Engine

### Workflow Script

Complex tasks are orchestrated via generated workflow definitions:

```rust
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub stages: Vec<WorkflowStage>,
    pub max_concurrent_agents: u32,
    pub timeout_ms: u64,
    pub on_failure: FailurePolicy,
}

pub struct WorkflowStage {
    pub id: String,
    pub name: String,
    pub agents: Vec<AgentAssignment>,
    pub strategy: StageStrategy,  // Parallel, Sequential, MapReduce
    pub depends_on: Vec<String>,
    pub sync_points: Vec<SyncPoint>,
}
```

### Workflow Lifecycle

1. **Generate**: Analyze task complexity → generate workflow definition
2. **Validate**: Check DAG acyclicity, resource availability
3. **Execute**: Run stages respecting dependencies and concurrency limits
4. **Monitor**: Track progress, token usage, agent health
5. **Aggregate**: Merge results from parallel agents
6. **Persist**: Save workflow for replay

## Token Optimizer

### Dual-Cache Strategy (OpenClacky Pattern)

```
System Prompt (frozen, byte-identical) → Cache Hit ~100%
User Context (dynamic, injected per-session) → Cache Miss → Compress
```

### Context Compression Pipeline

1. **Insert-then-Compress**: Add summary markers, compress old messages
2. **Chunk Archiving**: Archive compressed messages as .md files with reference paths
3. **Idle Compression**: Auto-compress after 5 min idle before cache expiry
4. **Skill Invocation**: Use `invoke_skill` meta-tool to avoid loading all tool schemas

## Self-Evolution (Hook-Based)

### Agent Lifecycle Hooks

```rust
pub enum AgentLifecycleHook {
    OnAgentStart,       // Initialize agent context
    OnToolCall,         // Intercept before tool execution
    OnToolResult,       // Process after tool execution
    OnToolFailure,      // Auto-repair on failure
    OnAgentIdle,        // Trigger self-reflection
    OnAgentError,       // Auto-healing
    OnSessionEnd,       // Extract learnings
}
```

### Self-Reflection Loop

```
Execute → Observe → Reflect → Learn → Adapt
   │         │         │         │        │
   ▼         ▼         ▼         ▼        ▼
 Tool    Telemetry  Pattern   Memory   Strategy
 Call    Collection  Analysis  Update   Adjustment
```

## Integration Points

### ACP Protocol Extensions

New JSON-RPC methods for plugin management:
- `plugin/list` — List active plugins
- `plugin/register` — Register a new plugin
- `plugin/invoke` — Invoke a plugin capability
- `plugin/health` — Health check

### Swarm Extensions

- `swarm/init` — Initialize swarm topology
- `swarm/spawn` — Spawn agent in swarm
- `swarm/route` — Route task to swarm
- `swarm/status` — Swarm health and progress

### Workflow Extensions

- `workflow/create` — Create workflow from task
- `workflow/execute` — Execute workflow
- `workflow/status` — Workflow progress
- `workflow/save` — Persist for replay
