//! Swarm Agent Adapters - Stdio Process Communication Layer
//!
//! This module provides adapters for external agent CLIs (Codex, Claude Code, etc.)
//! Each adapter manages a subprocess with stdio communication, real-time output
//! streaming, and process lifecycle management.
//!
//! ## Architecture
//!
//! The adapter trait abstracts away the differences between agent CLIs:
//! - Codex: `codex --full-auto` (autonomous mode)
//! - Claude Code: `claude --print` (print mode for non-interactive)
//!
//! Each adapter:
//! 1. Spawns a subprocess via stdio
//! 2. Monitors stdout/stderr in real-time
//! 3. Tracks process health (PID, memory, CPU)
//! 4. Handles graceful shutdown and crash recovery

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current UNIX timestamp in milliseconds (safe, never panics).
///
/// Shared utility used across the swarm subsystem for consistent timestamps.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Core Types
// ---------------------------------------------------------------------------

/// Unique identifier for a worker in the swarm
pub type WorkerId = String;

/// Unique identifier for a task
pub type TaskId = String;

/// Error type for swarm operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmError {
    /// Worker process failed to start
    ProcessStartFailed(String),
    /// Task execution failed
    TaskFailed(String),
    /// Worker crashed or became unhealthy
    WorkerCrashed(String),
    /// Communication error with worker
    CommunicationError(String),
    /// Timeout waiting for response
    Timeout(String),
    /// Worker not found
    WorkerNotFound(WorkerId),
    /// Invalid task description
    InvalidTask(String),
    /// Internal error
    InternalError(String),
}

impl std::fmt::Display for SwarmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwarmError::ProcessStartFailed(msg) => write!(f, "Process start failed: {}", msg),
            SwarmError::TaskFailed(msg) => write!(f, "Task failed: {}", msg),
            SwarmError::WorkerCrashed(msg) => write!(f, "Worker crashed: {}", msg),
            SwarmError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
            SwarmError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            SwarmError::WorkerNotFound(id) => write!(f, "Worker not found: {}", id),
            SwarmError::InvalidTask(msg) => write!(f, "Invalid task: {}", msg),
            SwarmError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for SwarmError {}

// ---------------------------------------------------------------------------
// Task Description
// ---------------------------------------------------------------------------

/// Description of a task to be executed by a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDescription {
    /// Unique task identifier
    pub id: TaskId,
    /// Human-readable task prompt
    pub prompt: String,
    /// Working directory for execution
    pub working_dir: Option<String>,
    /// Additional context or files
    pub context: HashMap<String, String>,
    /// Maximum execution time in milliseconds
    pub timeout_ms: u64,
    /// Priority level (higher = more urgent)
    pub priority: u8,
    /// Expected output format
    pub expected_format: OutputFormat,
}

impl TaskDescription {
    /// Create a new task with defaults
    pub fn new(id: TaskId, prompt: String) -> Self {
        Self {
            id,
            prompt,
            working_dir: None,
            context: HashMap::new(),
            timeout_ms: 60000, // 60 seconds default
            priority: 5,
            expected_format: OutputFormat::Text,
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Output format produced by the worker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Structured JSON output
    Structured,
    /// Markdown formatted text
    Markdown,
    /// Git diff format
    Diff,
    /// Plain text
    Text,
    /// Code files
    Code,
}

// ---------------------------------------------------------------------------
// Task Handle
// ---------------------------------------------------------------------------

/// Handle for tracking an in-progress task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHandle {
    /// Task ID
    pub task_id: TaskId,
    /// Worker assigned to this task
    pub worker_id: WorkerId,
    /// Current status
    pub status: TaskStatus,
    /// When the task started
    pub started_at: InstantWrapper,
    /// Output accumulated so far
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Process ID of the worker
    pub pid: Option<u32>,
}

/// Serializable timestamp — stores absolute UNIX milliseconds.
///
/// Previously stored `Instant::elapsed()` which is a relative duration with no
/// fixed reference point, making time comparisons meaningless. Now stores the
/// absolute UNIX timestamp so values are comparable across threads, processes,
/// and restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantWrapper {
    pub timestamp_ms: u64,
}

impl InstantWrapper {
    /// Create with the current time
    pub fn now() -> Self {
        Self {
            timestamp_ms: now_ms(),
        }
    }

    /// Create from a specific UNIX millisecond timestamp
    pub fn from_ms(ms: u64) -> Self {
        Self { timestamp_ms: ms }
    }

    /// Read back the stored UNIX millisecond timestamp
    pub fn as_ms(&self) -> u64 {
        self.timestamp_ms
    }
}

/// Status of a task execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued but not started
    Pending,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    Timeout,
}

// ---------------------------------------------------------------------------
// Worker Status
// ---------------------------------------------------------------------------

/// Status of a worker process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerStatus {
    /// Worker ID
    pub worker_id: WorkerId,
    /// Worker type (codex, claude_code, etc.)
    pub worker_type: String,
    /// Current health status
    pub health: HealthStatus,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_percent: Option<f32>,
    /// Number of tasks completed
    pub tasks_completed: u32,
    /// Number of tasks failed
    pub tasks_failed: u32,
    /// Current task if busy
    pub current_task: Option<TaskId>,
    /// When the worker started
    pub started_at: Option<InstantWrapper>,
    /// Last heartbeat time
    pub last_heartbeat: Option<InstantWrapper>,
}

/// Health status of a worker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Worker is healthy and ready
    Healthy,
    /// Worker is busy executing a task
    Busy,
    /// Worker is starting up
    Starting,
    /// Worker is shutting down
    Stopping,
    /// Worker crashed or is unhealthy
    Unhealthy,
    /// Worker is offline
    Offline,
}

// ---------------------------------------------------------------------------
// Worker Capabilities
// ---------------------------------------------------------------------------

/// Capabilities of a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCapabilities {
    /// Worker ID
    pub worker_id: WorkerId,
    /// Worker type
    pub worker_type: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Maximum task complexity
    pub max_complexity: u8,
    /// Maximum concurrent tasks
    pub max_concurrent: u8,
    /// Supports streaming output
    pub supports_streaming: bool,
    /// Supports cancellation
    pub supports_cancel: bool,
    /// Default timeout for tasks
    pub default_timeout_ms: u64,
}

// ---------------------------------------------------------------------------
// Swarm Agent Adapter Trait
// ---------------------------------------------------------------------------

/// Trait for adapting external agent CLIs to the swarm orchestration layer
///
/// Each implementation handles:
/// - Process lifecycle (spawn, monitor, kill)
/// - Stdio communication (stdin for input, stdout/stderr for output)
/// - Task execution and cancellation
/// - Health monitoring and reporting
pub trait SwarmAgentAdapter: Send + Sync {
    /// Send a task to this worker for execution
    ///
    /// Returns a TaskHandle for tracking the execution.
    /// The task runs asynchronously; use get_task_status to check progress.
    fn send_task(&self, task: &TaskDescription) -> Result<TaskHandle, SwarmError>;

    /// Get the current status of this worker
    fn get_status(&self) -> Result<WorkerStatus, SwarmError>;

    /// Cancel a running task
    fn cancel_task(&self, task_id: &str) -> Result<(), SwarmError>;

    /// Get the capabilities of this worker
    fn capabilities(&self) -> WorkerCapabilities;

    /// Check if the worker is healthy and responsive
    fn health_check(&self) -> bool;

    /// Get the worker ID
    fn worker_id(&self) -> &WorkerId;

    /// Get the worker type (codex, claude_code, etc.)
    fn worker_type(&self) -> &str;

    /// Shutdown the worker gracefully
    fn shutdown(&self) -> Result<(), SwarmError>;

    /// Get accumulated output for a task
    fn get_task_output(&self, task_id: &str) -> Result<String, SwarmError>;
}

// ---------------------------------------------------------------------------
// Module Exports
// ---------------------------------------------------------------------------

mod claude_code;
mod codex;

pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
