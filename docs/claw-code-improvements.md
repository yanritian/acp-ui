# Claw Code Improvements Implementation Plan

## Overview
Implementing four key improvements from Claw Code architecture analysis.

## 1. Hooks System Enhancement

### Changes to `hooks_executor.rs`:
- Add `PostToolUseFailure` to `HookType` enum
- Add `HookAbortSignal` struct for aborting tool execution
- Add `updated_input` field to `HookResult` for input modification

### Implementation:
```rust
pub enum HookType {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,  // NEW: triggered when tool fails
    Stop,
}

pub struct HookAbortSignal {
    aborted: bool,
    reason: Option<String>,
}

pub struct HookResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub should_block: bool,
    pub message: Option<String>,
    pub updated_input: Option<String>,  // NEW: modified input for tool
    pub abort_signal: Option<HookAbortSignal>,  // NEW: abort signal
}
```

## 2. Permission Hierarchy Enhancement

### Changes to `permission_checker.rs`:
- Add `PermissionMode` enum with 5 levels:
  - ReadOnly: Can only read files
  - WorkspaceWrite: Can write within workspace
  - DangerFullAccess: Full access (dangerous)
  - Prompt: Ask user for each operation
  - Allow: Auto-allow (current behavior)

### Implementation:
```rust
pub enum PermissionMode {
    ReadOnly,        // Read operations only
    WorkspaceWrite,  // Read + Write within cwd
    DangerFullAccess, // Full access, skip checks
    Prompt,          // Ask user for each operation
    Allow,           // Auto-allow with rules
}

pub struct PermissionConfig {
    pub mode: PermissionMode,
    pub cwd: Option<String>,
    pub allow: Vec<PermissionRule>,
    pub deny: Vec<PermissionRule>,
    pub deny_by_default: bool,
    pub ask_override: bool,  // Allow user to override Prompt mode
}
```

## 3. Session Management

### New module: `session_manager.rs`
- Session persistence to database
- Session compaction (compress old messages)
- Branch lock collision detection

### Features:
```rust
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub created_at: DateTime,
    pub messages: Vec<Message>,
    pub branch_lock: Option<String>,  // Lock for branch isolation
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    db: DatabaseConnection,
}

impl SessionManager {
    pub fn save_session(&self, session: &Session);
    pub fn load_session(&self, id: &str) -> Option<Session>;
    pub fn compact_session(&self, id: &str);  // Compress messages
    pub fn detect_collision(&self, branch: &str) -> bool;  // Branch lock check
}
```

## 4. Event Router Separation

### New module: `event_router.rs`
- Move event routing logic outside agent context window
- Like Claw Code's clawhip layer

### Features:
```rust
pub struct EventRouter {
    subscribers: HashMap<String, Vec<EventHandler>>,
    event_queue: VecDeque<Event>,
}

pub enum Event {
    ToolUse { agent_id, tool_name, args },
    ToolResult { agent_id, result },
    HookTriggered { agent_id, hook_type },
    SessionUpdate { session_id, action },
}

impl EventRouter {
    pub fn subscribe(&mut self, event_type: &str, handler: EventHandler);
    pub fn route(&mut self, event: Event);  // Route to handlers
    pub fn process_queue(&mut self);  // Batch process events
}
```

## Implementation Order

1. Hooks System Enhancement (highest priority)
2. Permission Hierarchy Enhancement
3. Session Management
4. Event Router Separation

## Testing

- Unit tests for each new module
- Integration tests with existing agent system
- Cargo check after each change