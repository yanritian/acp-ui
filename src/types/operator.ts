// Hermes Game Operator Protocol Types
// These types define the contract between frontend and backend

// ============================================================================
// Task State Machine
// ============================================================================

export type OperatorTaskStatus =
  | 'idle'
  | 'planning'
  | 'waiting_approval'
  | 'running'
  | 'paused'
  | 'redirecting'
  | 'cancelling'
  | 'cancelled'
  | 'failed'
  | 'completed'

export interface OperatorTask {
  task_id: string
  /** Optimistic concurrency control revision, incremented on every state change */
  revision: number
  domain: string  // e.g., 'game.godot'
  project_path: string
  goal: string
  status: OperatorTaskStatus
  mode: 'propose_then_apply' | 'apply_directly'
  approval_policy: 'safe_default' | 'permissive' | 'strict'
  created_at: string
  updated_at: string
  started_at?: string
  completed_at?: string
  summary?: string
  error?: string
}

// ============================================================================
// Event Stream
// ============================================================================

export type OperatorEventType =
  | 'task_created'
  | 'task_started'
  | 'project_analyzing'
  | 'project_analyzed'
  | 'project_analysis_failed'
  | 'plan_started'
  | 'plan_generating'
  | 'plan_ready'
  | 'plan_failed'
  | 'approval_requested'
  | 'approval_granted'
  | 'approval_rejected'
  | 'tool_call_started'
  | 'tool_call_succeeded'
  | 'tool_call_failed'
  | 'step_started'
  | 'step_executing'
  | 'step_completed'
  | 'step_failed'
  | 'file_read'
  | 'file_modified'
  | 'file_patch_proposed'
  | 'file_patch_applied'
  | 'validation_started'
  | 'validation_passed'
  | 'validation_failed'
  | 'validation_skipped'
  | 'memory_read'
  | 'memory_written'
  | 'hook_started'
  | 'hook_succeeded'
  | 'hook_failed'
  | 'task_paused'
  | 'task_resumed'
  | 'task_redirected'
  | 'task_cancelling'
  | 'task_completing'
  | 'task_cancelled'
  | 'task_failed'
  | 'task_completed'
  | 'summary_ready'

export type EventLevel = 'info' | 'warning' | 'error' | 'debug'

export interface OperatorEvent {
  event_id: string
  task_id: string
  timestamp: string
  type: OperatorEventType
  level: EventLevel
  title: string
  message?: string
  source: string  // e.g., 'agent_runtime', 'tool', 'hook'
  payload?: Record<string, any>
}

// ============================================================================
// Approval Model
// ============================================================================

export type ApprovalLevel = 'silent' | 'notify' | 'approve' | 'forbidden'

export type ApprovalDecision = 'approve' | 'reject' | 'request_changes'

export interface FileDiffPreview {
  path: string
  operation: 'create' | 'replace'
  diff: string
}

export interface ApprovalRequest {
  approval_id: string
  task_id: string
  level: ApprovalLevel
  action: string  // e.g., 'file.patch', 'shell.command'
  title: string
  reason: string
  risk?: string
  preview?: {
    files?: string[]
    diff_id?: string
    command?: string
    diffs?: FileDiffPreview[]
  }
  options: ApprovalDecision[]
  created_at: string
  resolved_at?: string
  decision?: ApprovalDecision
  resolved_by?: string
}

// ============================================================================
// Tool Call
// ============================================================================

export interface ToolCall {
  call_id: string
  task_id: string
  tool: string  // e.g., 'file.read', 'file.patch', 'godot.analyze_project'
  input: Record<string, any>
  approval_level: ApprovalLevel
  started_at: string
  completed_at?: string
  result?: ToolResult
  error?: string
}

export interface ToolResult {
  success: boolean
  output?: any
  artifacts?: {
    files?: string[]
    diffs?: string[]
    logs?: string[]
  }
  duration_ms: number
}

// ============================================================================
// Memory
// ============================================================================

export type MemoryScope = 'project' | 'task' | 'operator'

export interface MemoryRecord {
  memory_id: string
  scope: MemoryScope
  domain: string  // e.g., 'godot'
  key: string
  value: any
  source_event_id?: string
  confidence: number  // 0.0 - 1.0
  created_at: string
  expires_at?: string
  last_accessed_at?: string
}

// ============================================================================
// Domain Pack
// ============================================================================

export interface DomainPackManifest {
  domain: string  // e.g., 'game.godot'
  version: string
  name: string
  description: string
  project_detector: {
    markers: string[]  // e.g., ['project.godot']
  }
  analyzer: {
    file_patterns: string[]  // e.g., ['*.gd', '*.tscn']
  }
  skills: string[]  // skill names
  tools: string[]  // tool names
  hooks: string[]  // hook names
  validation_checks: string[]
}

// ============================================================================
// Agent Run Config
// ============================================================================

export interface AgentRunConfig {
  domain: string
  project_path: string
  goal: string
  mode: 'propose_then_apply' | 'apply_directly'
  approval_policy: 'safe_default' | 'permissive' | 'strict'
  model?: string
  max_iterations?: number
  timeout_seconds?: number
  allowed_tools?: string[]
  forbidden_tools?: string[]
}

// ============================================================================
// API Request/Response Types
// ============================================================================

export interface StartTaskRequest {
  domain: string
  project_path: string
  goal: string
  mode?: 'propose_then_apply' | 'apply_directly'
  approval_policy?: 'safe_default' | 'permissive' | 'strict'
}

export interface StartTaskResponse {
  task_id: string
  status: OperatorTaskStatus
  event_stream: string  // e.g., 'operator://tasks/task_123/events'
}

export interface ApproveRequest {
  task_id: string
  approval_id: string
  decision: ApprovalDecision
  comment?: string
  reason?: string
}

export interface RedirectRequest {
  task_id: string
  new_goal: string
  preserve_completed_work: boolean
}

export interface RemoteRedirectRequest {
  new_goal: string
  preserve_completed_work?: boolean
}

export interface RemoteCommandResponse {
  ok: boolean
  task_id?: string
}

export interface RemotePlatformCapability {
  id: string
  name: string
  platform_type: 'desktop_app' | 'web' | 'ide_extension' | 'ide_plugin' | 'mobile' | 'game_engine' | string
  transport: string[]
  status: string
  capabilities: string[]
}

export interface RemoteAuditRecord {
  audit_id: string
  request_id: string
  timestamp: string
  principal: 'bearer_token' | 'local_trust' | 'unauthenticated' | string
  client_id: string
  method: string
  path: string
  action: string
  outcome: 'attempted' | 'succeeded' | 'failed' | 'denied' | string
  status_code: number
  denial_code: string | null
}

export interface TaskSummary {
  task_id: string
  status: OperatorTaskStatus
  goal: string
  summary: string
  files_changed: string[]
  files_created: string[]
  files_deleted: string[]
  duration_seconds: number
  iterations: number
  errors: string[]
  warnings: string[]
}
