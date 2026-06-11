//! Worker Protocol - Agent registration and communication

use serde::{Deserialize, Serialize};

/// Worker capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Worker ID
    pub worker_id: String,
    /// Worker type (e.g., "claude-code", "codex")
    pub worker_type: String,
    /// Worker version
    pub version: String,
    /// Skills with proficiency scores
    pub skills: Vec<SkillDeclaration>,
    /// Maximum concurrent tasks
    pub max_concurrency: u32,
    /// Maximum context tokens
    pub max_context_tokens: u64,
}

/// Skill declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDeclaration {
    pub name: String,
    pub proficiency: f32,
    pub avg_duration_ms: u64,
}

/// Worker status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Disconnected,
    Error,
}

/// Worker heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub status: WorkerStatus,
    pub current_load: u32,
    pub active_goals: Vec<String>,
}