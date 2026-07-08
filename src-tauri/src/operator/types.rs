// Hermes Game Operator Protocol Types (Rust Backend)
// These types mirror the TypeScript definitions in src/types/operator.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Task State Machine
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorTaskStatus {
    Idle,
    Planning,
    WaitingApproval,
    Running,
    Paused,
    Redirecting,
    Cancelling,
    Cancelled,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorTask {
    pub task_id: String,
    pub domain: String,
    pub project_path: String,
    pub goal: String,
    pub status: OperatorTaskStatus,
    pub mode: TaskMode,
    pub approval_policy: ApprovalPolicy,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskMode {
    ProposeThenApply,
    ApplyDirectly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalPolicy {
    SafeDefault,
    Permissive,
    Strict,
}

// ============================================================================
// Event Stream
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorEventType {
    TaskCreated,
    TaskStarted,
    ProjectAnalyzing,
    ProjectAnalyzed,
    ProjectAnalysisFailed,
    PlanStarted,
    PlanGenerating,
    PlanReady,
    PlanFailed,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    ToolCallStarted,
    ToolCallSucceeded,
    ToolCallFailed,
    StepStarted,
    StepExecuting,
    StepCompleted,
    StepFailed,
    FileRead,
    FileModified,
    FilePatchProposed,
    FilePatchApplied,
    MemoryRead,
    MemoryWritten,
    HookStarted,
    HookSucceeded,
    HookFailed,
    TaskPaused,
    TaskResumed,
    TaskRedirected,
    TaskCancelling,
    TaskCompleting,
    TaskCancelled,
    TaskFailed,
    TaskCompleted,
    SummaryReady,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Info,
    Warning,
    Error,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorEvent {
    pub event_id: String,
    pub task_id: String,
    pub timestamp: String,
    pub event_type: OperatorEventType,
    pub level: EventLevel,
    pub title: String,
    pub message: Option<String>,
    pub source: String,
    pub payload: Option<serde_json::Value>,
}

// ============================================================================
// Approval Model
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalLevel {
    Silent,
    Notify,
    Approve,
    Forbidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approve,
    Reject,
    RequestChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub task_id: String,
    pub level: ApprovalLevel,
    pub action: String,
    pub title: String,
    pub reason: String,
    pub risk: Option<String>,
    pub preview: Option<ApprovalPreview>,
    pub options: Vec<ApprovalDecision>,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub decision: Option<ApprovalDecision>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPreview {
    pub files: Option<Vec<String>>,
    pub diff_id: Option<String>,
    pub command: Option<String>,
}

// ============================================================================
// Tool Call
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub task_id: String,
    pub tool: String,
    pub input: serde_json::Value,
    pub approval_level: ApprovalLevel,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub result: Option<ToolResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub artifacts: Option<ToolArtifacts>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArtifacts {
    pub files: Option<Vec<String>>,
    pub diffs: Option<Vec<String>>,
    pub logs: Option<Vec<String>>,
}

// ============================================================================
// Memory
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Project,
    Task,
    Operator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub memory_id: String,
    pub scope: MemoryScope,
    pub domain: String,
    pub key: String,
    pub value: serde_json::Value,
    pub source_event_id: Option<String>,
    pub confidence: f64,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_accessed_at: Option<String>,
}

// ============================================================================
// Domain Pack
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPackManifest {
    pub domain: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub project_detector: ProjectDetector,
    pub analyzer: ProjectAnalyzer,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
    pub hooks: Vec<String>,
    pub validation_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetector {
    pub markers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalyzer {
    pub file_patterns: Vec<String>,
}

// ============================================================================
// Agent Run Config
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunConfig {
    pub domain: String,
    pub project_path: String,
    pub goal: String,
    pub mode: TaskMode,
    pub approval_policy: ApprovalPolicy,
    pub model: Option<String>,
    pub max_iterations: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub allowed_tools: Option<Vec<String>>,
    pub forbidden_tools: Option<Vec<String>>,
}

// ============================================================================
// API Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTaskRequest {
    pub domain: String,
    pub project_path: String,
    pub goal: String,
    pub mode: Option<TaskMode>,
    pub approval_policy: Option<ApprovalPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTaskResponse {
    pub task_id: String,
    pub status: OperatorTaskStatus,
    pub event_stream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub task_id: String,
    pub approval_id: String,
    pub decision: ApprovalDecision,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectRequest {
    pub task_id: String,
    pub new_goal: String,
    pub preserve_completed_work: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub status: OperatorTaskStatus,
    pub goal: String,
    pub summary: String,
    pub files_changed: Vec<String>,
    pub files_created: Vec<String>,
    pub files_deleted: Vec<String>,
    pub duration_seconds: u64,
    pub iterations: u32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
