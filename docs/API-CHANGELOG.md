# API Changelog

This document tracks all API changes for Hermes Game Operator.

## Version 1.0.0 (2026-07-08)

### Initial Release

#### Operator Commands (17 commands)

**Task Management (5)**:
- `operator_start_task` - Start a new operator task
- `operator_get_task` - Get task details by ID
- `operator_list_tasks` - List all tasks
- `operator_pause_task` - Pause a running task
- `operator_resume_task` - Resume a paused task

**Task Control (4)**:
- `operator_stop_task` - Stop a running or paused task
- `operator_redirect_task` - Redirect task to new goal
- `operator_approve` - Approve or reject an action
- `operator_get_pending_approvals` - Get pending approvals

**Events & Summary (2)**:
- `operator_list_events` - List events for a task
- `operator_get_task_summary` - Get task summary

**Godot Specific (2)**:
- `godot_detect_project` - Detect if directory is Godot project
- `godot_analyze_project` - Analyze Godot project structure

**File Tools (4)**:
- `operator_file_read` - Safely read a file
- `operator_file_patch` - Apply patch to file
- `operator_file_patch_preview` - Preview patch without applying
- `operator_file_list` - List files in directory

---

#### Request/Response Types

**StartTaskRequest**:
```typescript
{
  domain: string;              // e.g., "game.godot"
  project_path: string;        // Absolute path to project
  goal: string;                // Task goal description
  mode?: "propose_then_apply" | "apply_directly";
  approval_policy?: "safe_default" | "permissive" | "strict";
}
```

**StartTaskResponse**:
```typescript
{
  task_id: string;
  status: OperatorTaskStatus;
  event_stream: string;        // e.g., "operator://tasks/task_123/events"
}
```

**OperatorTask**:
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

**OperatorEvent**:
```typescript
{
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

**ApprovalRequest**:
```typescript
{
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

#### Enum Types

**OperatorTaskStatus**:
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

**OperatorEventType**:
```typescript
type OperatorEventType =
  | 'task_created'
  | 'task_started'
  | 'project_analyzed'
  | 'plan_ready'
  | 'approval_requested'
  | 'approval_granted'
  | 'approval_rejected'
  | 'file_read'
  | 'file_modified'
  | 'task_completed'
  | 'task_failed';
```

**ApprovalLevel**:
```typescript
type ApprovalLevel = 'silent' | 'notify' | 'approve' | 'forbidden';
```

**ApprovalDecision**:
```typescript
type ApprovalDecision = 'approve' | 'reject' | 'request_changes';
```

---

#### Error Types

**OperatorError**:
```typescript
{
  code: string;              // e.g., "1001"
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

**ErrorCategory**:
```typescript
type ErrorCategory = 
  | 'Validation'
  | 'Security'
  | 'Network'
  | 'FileSystem'
  | 'Agent'
  | 'State'
  | 'Approval'
  | 'Config'
  | 'Unknown';
```

**ErrorSeverity**:
```typescript
type ErrorSeverity = 'Info' | 'Warning' | 'Error' | 'Critical';
```

---

## Breaking Changes

### v0.x → v1.0.0

#### 1. Domain Format

**Old (v0.x)**:
```typescript
{ domain: 'godot' }
```

**New (v1.0.0)**:
```typescript
{ domain: 'game.godot' }
```

**Migration**: Update domain format to include engine type prefix.

---

#### 2. Required Fields

**Old (v0.x)**:
```typescript
{
  domain: 'godot',
  goal: 'Add feature'
}
```

**New (v1.0.0)**:
```typescript
{
  domain: 'game.godot',
  project_path: '/path/to/project',  // NEW: Required
  goal: 'Add feature',
  mode: 'propose_then_apply',         // NEW: Required
  approval_policy: 'safe_default'     // NEW: Required
}
```

**Migration**: Add `project_path`, `mode`, and `approval_policy` fields.

---

#### 3. Event Structure

**Old (v0.x)**:
```typescript
{
  type: 'file_changed',
  data: { path: 'file.gd' }
}
```

**New (v1.0.0)**:
```typescript
{
  type: 'file_modified',
  level: 'info',
  title: 'File modified',
  payload: { path: 'file.gd' }
}
```

**Migration**: 
- Rename event types
- Add `level` and `title` fields
- Move data to `payload`

---

#### 4. Approval Decision

**Old (v0.x)**:
```typescript
{
  approved: true
}
```

**New (v1.0.0)**:
```typescript
{
  decision: 'approve',  // Changed from boolean
  comment: 'Looks good'  // NEW: Optional
}
```

**Migration**: Change `approved: boolean` to `decision: string`.

---

## Deprecations

### None

No deprecated APIs in v1.0.0.

---

## Future API Changes (Planned)

### v1.1.0 (Q3 2026)

#### New Commands

- `operator_get_memory` - Get memory records for task
- `operator_delete_memory` - Delete memory record
- `operator_export_events` - Export events to file
- `operator_import_task` - Import task from file

#### New Types

**MemoryRecord**:
```typescript
{
  memory_id: string;
  scope: MemoryScope;
  domain: string;
  key: string;
  value: any;
  source_event_id?: string;
  confidence: number;
  created_at: string;
  expires_at?: string;
}
```

**MemoryScope**:
```typescript
type MemoryScope = 'project' | 'task' | 'operator';
```

---

### v1.2.0 (Q4 2026)

#### New Commands

- `operator_batch_execute` - Execute multiple tasks
- `operator_cancel_all` - Cancel all running tasks
- `operator_get_stats` - Get operator statistics
- `operator_configure` - Update operator configuration

#### New Types

**BatchExecuteRequest**:
```typescript
{
  tasks: StartTaskRequest[];
  parallel: boolean;
  stop_on_error: boolean;
}
```

**OperatorStats**:
```typescript
{
  total_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  average_duration: number;
  success_rate: number;
}
```

---

### v2.0.0 (2027)

#### Breaking Changes

- WebSocket API for real-time events
- GraphQL API support
- Plugin API for custom tools
- Multi-user support with authentication

#### Migration Guide

Will be provided with v2.0.0 release.

---

## API Stability Guarantees

### Semantic Versioning

- **Major (X.0.0)**: Breaking changes
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, fully compatible

### Stability Policy

- **Stable APIs**: No breaking changes in minor/patch releases
- **Beta APIs**: May change in minor releases (marked with `@beta`)
- **Experimental APIs**: May change in any release (marked with `@experimental`)

### Deprecation Policy

- **Deprecation Notice**: 6 months before removal
- **Migration Guide**: Provided with deprecation notice
- **Support**: Old API supported during deprecation period

---

## API Best Practices

### 1. Error Handling

```typescript
try {
  const result = await OperatorApi.startTask(request);
} catch (error) {
  if (error.code === '1001') {
    // Handle validation error
  } else if (error.code === '2001') {
    // Handle security error
  } else {
    // Handle other errors
    console.error('Unexpected error:', error);
  }
}
```

---

### 2. Event Polling

```typescript
// Poll events periodically
setInterval(async () => {
  const events = await OperatorApi.listEvents(taskId, 10);
  processEvents(events);
}, 2000);
```

---

### 3. Approval Workflow

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
    decision: decision
  });
}
```

---

### 4. Task Monitoring

```typescript
// Monitor task status
async function monitorTask(taskId: string) {
  let status = 'running';
  
  while (status === 'running' || status === 'paused') {
    const task = await OperatorApi.getTask(taskId);
    status = task.status;
    
    console.log(`Task status: ${status}`);
    
    // Wait before next check
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  
  console.log(`Task finished with status: ${status}`);
}
```

---

## Resources

- [API Reference](api.md)
- [API Examples](API-EXAMPLES.md)
- [Migration Guide](MIGRATION-GUIDE.md)
- [Upgrade Guide](UPGRADE.md)

---

**Changelog Version**: 1.0.0  
**Last Updated**: 2026-07-08
