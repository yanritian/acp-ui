# Hermes Game Operator API Reference

## Overview

This document provides a complete reference for all Tauri commands exposed by the Hermes Game Operator.

## Operator Commands

### Task Management

#### `operator_start_task`

Start a new operator task.

**Parameters:**
```typescript
{
  request: {
    domain: string;              // e.g., "game.godot"
    project_path: string;        // Absolute path to project
    goal: string;                // Task goal description
    mode?: "propose_then_apply" | "apply_directly";
    approval_policy?: "safe_default" | "permissive" | "strict";
  }
}
```

**Returns:**
```typescript
{
  task_id: string;
  status: OperatorTaskStatus;
  event_stream: string;          // e.g., "operator://tasks/task_123/events"
}
```

**Example:**
```typescript
const result = await OperatorApi.startTask({
  domain: "game.godot",
  project_path: "/path/to/project",
  goal: "Add double jump to player",
  mode: "propose_then_apply",
  approval_policy: "safe_default"
});
```

---

#### `operator_get_task`

Get task details by ID.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:**
```typescript
{
  task_id: string;
  domain: string;
  project_path: string;
  goal: string;
  status: OperatorTaskStatus;
  mode: TaskMode;
  approval_policy: ApprovalPolicy;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
  summary?: string;
  error?: string;
}
```

**Example:**
```typescript
const task = await OperatorApi.getTask("task_123");
```

---

#### `operator_list_tasks`

List all tasks.

**Parameters:** None

**Returns:**
```typescript
OperatorTask[]
```

**Example:**
```typescript
const tasks = await OperatorApi.listTasks();
```

---

### Task Control

#### `operator_pause_task`

Pause a running task.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:** `void`

**Example:**
```typescript
await OperatorApi.pauseTask("task_123");
```

---

#### `operator_resume_task`

Resume a paused task.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:** `void`

**Example:**
```typescript
await OperatorApi.resumeTask("task_123");
```

---

#### `operator_stop_task`

Stop a running or paused task.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:** `void`

**Example:**
```typescript
await OperatorApi.stopTask("task_123");
```

---

#### `operator_redirect_task`

Redirect task to a new goal.

**Parameters:**
```typescript
{
  request: {
    task_id: string;
    new_goal: string;
    preserve_completed_work: boolean;
  }
}
```

**Returns:** `void`

**Example:**
```typescript
await OperatorApi.redirectTask({
  task_id: "task_123",
  new_goal: "Add triple jump instead",
  preserve_completed_work: true
});
```

---

### Approval

#### `operator_approve`

Approve or reject an action.

**Parameters:**
```typescript
{
  request: {
    task_id: string;
    approval_id: string;
    decision: "approve" | "reject" | "request_changes";
    comment?: string;
  }
}
```

**Returns:** `void`

**Example:**
```typescript
await OperatorApi.approve({
  task_id: "task_123",
  approval_id: "approval_001",
  decision: "approve",
  comment: "Looks good"
});
```

---

#### `operator_get_pending_approvals`

Get pending approvals for a task.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:**
```typescript
ApprovalRequest[]
```

**Example:**
```typescript
const approvals = await OperatorApi.getPendingApprovals("task_123");
```

---

### Events

#### `operator_list_events`

List events for a task.

**Parameters:**
```typescript
{
  taskId: string;
  limit?: number;  // Default: 100
}
```

**Returns:**
```typescript
OperatorEvent[]
```

**Example:**
```typescript
const events = await OperatorApi.listEvents("task_123", 50);
```

---

### Summary

#### `operator_get_task_summary`

Get task summary.

**Parameters:**
```typescript
{
  taskId: string;
}
```

**Returns:**
```typescript
{
  task_id: string;
  status: OperatorTaskStatus;
  goal: string;
  summary: string;
  files_changed: string[];
  files_created: string[];
  files_deleted: string[];
  duration_seconds: number;
  iterations: number;
  errors: string[];
  warnings: string[];
}
```

**Example:**
```typescript
const summary = await OperatorApi.getTaskSummary("task_123");
```

---

## File Tool Commands

### `operator_file_read`

Safely read a file with path validation.

**Parameters:**
```typescript
{
  path: string;
  allowedRoots: string[];
}
```

**Returns:**
```typescript
{
  path: string;
  content: string;
  size_bytes: number;
  line_count: number;
}
```

**Example:**
```typescript
const result = await OperatorApi.fileRead(
  "/path/to/project/scripts/Player.gd",
  ["/path/to/project"]
);
```

---

### `operator_file_patch`

Apply a patch to a file with optional backup.

**Parameters:**
```typescript
{
  path: string;
  newContent: string;
  allowedRoots: string[];
  createBackup?: boolean;  // Default: true
}
```

**Returns:**
```typescript
{
  path: string;
  success: boolean;
  lines_changed: number;
  backup_path?: string;
}
```

**Example:**
```typescript
const result = await OperatorApi.filePatch(
  "/path/to/project/scripts/Player.gd",
  newContent,
  ["/path/to/project"],
  true
);
```

---

### `operator_file_patch_preview`

Preview a patch without applying it.

**Parameters:**
```typescript
{
  path: string;
  newContent: string;
  allowedRoots: string[];
}
```

**Returns:**
```typescript
{
  path: string;
  original_content: string;
  new_content: string;
  diff: string;
}
```

**Example:**
```typescript
const preview = await OperatorApi.filePatchPreview(
  "/path/to/project/scripts/Player.gd",
  newContent,
  ["/path/to/project"]
);
```

---

### `operator_file_list`

List files and directories in a path.

**Parameters:**
```typescript
{
  path: string;
  allowedRoots: string[];
}
```

**Returns:**
```typescript
{
  path: string;
  files: string[];
  directories: string[];
}
```

**Example:**
```typescript
const listing = await OperatorApi.fileList(
  "/path/to/project/scripts",
  ["/path/to/project"]
);
```

---

## Godot Commands

### `godot_detect_project`

Detect if a directory is a Godot project.

**Parameters:**
```typescript
{
  path: string;
}
```

**Returns:** `boolean`

**Example:**
```typescript
const isGodot = await GodotOperatorApi.detectProject("/path/to/project");
```

---

### `godot_analyze_project`

Analyze a Godot project.

**Parameters:**
```typescript
{
  projectPath: string;
}
```

**Returns:**
```typescript
{
  project_path: string;
  project_file: string;
  project_name: string;
  godot_version: string;
  main_scene: string;
  scripts: string[];
  scenes: string[];
  assets: string[];
  entry_scene: string;
}
```

**Example:**
```typescript
const analysis = await GodotOperatorApi.analyzeProject("/path/to/project");
```

---

## Type Definitions

### OperatorTaskStatus

```typescript
type OperatorTaskStatus =
  | 'idle'
  | 'planning'
  | 'waiting_approval'
  | 'running'
  | 'paused'
  | 'redirecting'
  | 'cancelling'
  | 'cancelled'
  | 'failed'
  | 'completed';
```

### OperatorEvent

```typescript
interface OperatorEvent {
  event_id: string;
  task_id: string;
  timestamp: string;
  type: OperatorEventType;
  level: EventLevel;
  title: string;
  message?: string;
  source: string;
  payload?: Record<string, any>;
}
```

### ApprovalRequest

```typescript
interface ApprovalRequest {
  approval_id: string;
  task_id: string;
  level: ApprovalLevel;
  action: string;
  title: string;
  reason: string;
  risk?: string;
  preview?: {
    files?: string[];
    diff_id?: string;
    command?: string;
  };
  options: ApprovalDecision[];
  created_at: string;
  resolved_at?: string;
  decision?: ApprovalDecision;
  resolved_by?: string;
}
```

---

## Error Handling

All commands can throw errors with the following structure:

```typescript
interface OperatorError {
  code: string;           // e.g., "1001"
  category: ErrorCategory;
  severity: ErrorSeverity;
  message: string;
  details?: string;
  source?: string;
  timestamp: string;
  task_id?: string;
  recoverable: boolean;
}
```

### Error Categories

- `Validation` (1xxx)
- `Security` (2xxx)
- `Network` (3xxx)
- `FileSystem` (4xxx)
- `Agent` (5xxx)
- `State` (6xxx)
- `Approval` (7xxx)
- `Config` (8xxx)

### Error Severity

- `Info`
- `Warning`
- `Error`
- `Critical`

---

## Best Practices

### 1. Always Validate Paths

```typescript
// Good
const allowedRoots = ["/path/to/project"];
const result = await OperatorApi.fileRead(filePath, allowedRoots);

// Bad - no validation
const result = await OperatorApi.fileRead(filePath, []);
```

### 2. Handle Errors Gracefully

```typescript
try {
  const result = await OperatorApi.startTask(request);
} catch (error) {
  if (error.code === "2001") {
    console.error("Path outside boundary");
  } else {
    console.error("Unexpected error:", error);
  }
}
```

### 3. Monitor Events

```typescript
// Poll events periodically
setInterval(async () => {
  const events = await OperatorApi.listEvents(taskId, 10);
  updateUI(events);
}, 2000);
```

### 4. Use Approval Workflow

```typescript
// Check for pending approvals
const approvals = await OperatorApi.getPendingApprovals(taskId);

for (const approval of approvals) {
  // Show approval dialog
  const decision = await showApprovalDialog(approval);

  // Submit decision
  await OperatorApi.approve({
    task_id: taskId,
    approval_id: approval.approval_id,
    decision
  });
}
```

---

## See Also

- [Architecture Documentation](architecture.md)
- [Security Model](security.md)
- [Error Codes Reference](error-codes.md)
- [Testing Guide](testing.md)
