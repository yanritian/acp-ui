//! Swarm Types - Core data structures for swarm orchestration
//!
//! This module defines the core types used across the swarm orchestration layer,
//! including task shards, replica assignment, and fault tolerance structures.

#![allow(dead_code)] // Reserved types for future shard-based execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::swarm_adapters::{now_ms, HealthStatus, OutputFormat, WorkerId};

// ---------------------------------------------------------------------------
// Task Shard (ES Sharding Pattern)
// ---------------------------------------------------------------------------

/// Task shard status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardStatus {
    /// Shard is pending assignment
    Pending,
    /// Shard is assigned to primary worker
    AssignedPrimary,
    /// Shard is assigned to replica worker (backup)
    AssignedReplica,
    /// Shard is executing on primary
    RunningPrimary,
    /// Shard is executing on replica (after primary failure)
    RunningReplica,
    /// Shard completed successfully
    Completed,
    /// Shard failed
    Failed,
    /// Shard was cancelled
    Cancelled,
}

/// Task payload content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPayload {
    /// The actual task description/prompt
    pub prompt: String,
    /// Working directory for execution
    pub working_dir: Option<String>,
    /// Additional files or context
    pub context_files: HashMap<String, String>,
    /// Expected output format
    pub expected_output: OutputFormat,
}

impl TaskPayload {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            working_dir: None,
            context_files: HashMap::new(),
            expected_output: OutputFormat::Text,
        }
    }

    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_context_file(mut self, name: String, content: String) -> Self {
        self.context_files.insert(name, content);
        self
    }
}

/// Task Shard - Inspired by Elasticsearch sharding pattern
///
/// A shard represents a portion of a larger task, allowing:
/// - Parallel execution across multiple workers
/// - Replica assignment for fault tolerance
/// - Primary/Replica switching on failure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskShard {
    /// Unique shard identifier
    pub id: String,
    /// Parent task this shard belongs to
    pub parent_task: String,
    /// Shard index (0, 1, 2, ...)
    pub shard_index: u32,
    /// Total number of shards for this task
    pub total_shards: u32,
    /// The payload to execute
    pub payload: TaskPayload,
    /// Primary worker assigned to this shard
    pub primary_worker: WorkerId,
    /// Replica worker (backup) - assigned after primary succeeds or fails
    pub replica_worker: Option<WorkerId>,
    /// Current status
    pub status: ShardStatus,
    /// Execution timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Current retry count
    pub retry_count: u32,
    /// Result from primary execution
    pub primary_result: Option<String>,
    /// Result from replica execution (fallback)
    pub replica_result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// When this shard was created
    pub created_at: u64,
    /// When execution started
    pub started_at: Option<u64>,
    /// When execution completed
    pub completed_at: Option<u64>,
}

impl TaskShard {
    /// Create a new task shard
    pub fn new(
        id: String,
        parent_task: String,
        shard_index: u32,
        total_shards: u32,
        payload: TaskPayload,
        primary_worker: WorkerId,
    ) -> Self {
        Self {
            id,
            parent_task,
            shard_index,
            total_shards,
            payload,
            primary_worker,
            replica_worker: None,
            status: ShardStatus::Pending,
            timeout_ms: 60000,
            max_retries: 3,
            retry_count: 0,
            primary_result: None,
            replica_result: None,
            error: None,
            created_at: now_ms(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Assign a replica worker for fault tolerance
    pub fn assign_replica(&mut self, replica_worker: WorkerId) {
        self.replica_worker = Some(replica_worker);
        self.status = ShardStatus::AssignedPrimary;
    }

    /// Mark shard as running on primary
    pub fn start_primary(&mut self) {
        self.status = ShardStatus::RunningPrimary;
        self.started_at = Some(now_ms());
    }

    /// Mark shard as running on replica (after primary failure)
    pub fn start_replica(&mut self) {
        self.status = ShardStatus::RunningReplica;
        self.retry_count += 1;
    }

    /// Complete shard with result from primary
    pub fn complete_primary(&mut self, result: String) {
        self.status = ShardStatus::Completed;
        self.primary_result = Some(result);
        self.completed_at = Some(now_ms());
    }

    /// Complete shard with result from replica
    pub fn complete_replica(&mut self, result: String) {
        self.status = ShardStatus::Completed;
        self.replica_result = Some(result);
        self.completed_at = Some(now_ms());
    }

    /// Fail shard with error
    pub fn fail(&mut self, error: String) {
        self.status = ShardStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(now_ms());
    }

    /// Check if shard can retry
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Check if shard is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, ShardStatus::Completed)
    }

    /// Check if shard failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ShardStatus::Failed)
    }

    /// Get the final result (primary or replica)
    pub fn get_result(&self) -> Option<&String> {
        self.primary_result
            .as_ref()
            .or(self.replica_result.as_ref())
    }

    /// Get effective worker (primary if healthy, replica if primary failed)
    pub fn get_effective_worker(&self) -> &WorkerId {
        if self.status == ShardStatus::RunningReplica {
            self.replica_worker.as_ref().unwrap_or(&self.primary_worker)
        } else {
            &self.primary_worker
        }
    }
}

// ---------------------------------------------------------------------------
// Shard Group (Collection of shards for a task)
// ---------------------------------------------------------------------------

/// Status of a shard group (parent task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardGroupStatus {
    /// Task is being partitioned
    Partitioning,
    /// Shards are being assigned to workers
    Assigning,
    /// Shards are executing
    Executing,
    /// Waiting for all shards to complete
    Aggregating,
    /// Task completed successfully
    Completed,
    /// Task failed (one or more shards failed)
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Collection of shards for a parent task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardGroup {
    /// Parent task ID
    pub task_id: String,
    /// Original task prompt
    pub original_prompt: String,
    /// All shards for this task
    pub shards: HashMap<String, TaskShard>,
    /// Current status of the group
    pub status: ShardGroupStatus,
    /// Number of shards
    pub shard_count: u32,
    /// Number of completed shards
    pub completed_count: u32,
    /// Number of failed shards
    pub failed_count: u32,
    /// Aggregated final result
    pub final_result: Option<String>,
    /// Aggregated error messages
    pub errors: Vec<String>,
    /// When this group was created
    pub created_at: u64,
    /// When execution completed
    pub completed_at: Option<u64>,
}

impl ShardGroup {
    /// Create a new shard group
    pub fn new(task_id: String, original_prompt: String) -> Self {
        Self {
            task_id,
            original_prompt,
            shards: HashMap::new(),
            status: ShardGroupStatus::Partitioning,
            shard_count: 0,
            completed_count: 0,
            failed_count: 0,
            final_result: None,
            errors: Vec::new(),
            created_at: now_ms(),
            completed_at: None,
        }
    }

    /// Add a shard to the group
    pub fn add_shard(&mut self, shard: TaskShard) {
        self.shards.insert(shard.id.clone(), shard);
        self.shard_count = self.shards.len() as u32;
    }

    /// Update shard status and update counts
    pub fn update_shard(&mut self, shard_id: &str, shard: TaskShard) {
        self.shards.insert(shard_id.to_string(), shard);
        self.update_counts();
    }

    /// Update completion counts
    fn update_counts(&mut self) {
        self.completed_count = self.shards.values().filter(|s| s.is_complete()).count() as u32;
        self.failed_count = self.shards.values().filter(|s| s.is_failed()).count() as u32;
    }

    /// Check if all shards are complete or failed
    pub fn is_finished(&self) -> bool {
        (self.completed_count + self.failed_count) == self.shard_count
    }

    /// Aggregate results from all shards
    pub fn aggregate_results(&mut self) {
        let results: Vec<String> = self
            .shards
            .values()
            .filter_map(|s| s.get_result().cloned())
            .collect();

        if results.len() == self.shard_count as usize {
            // All shards completed - merge results
            let merged = results.join("\n\n--- Shard Boundary ---\n\n");
            self.final_result = Some(merged);
            self.status = ShardGroupStatus::Completed;
        } else if self.failed_count > 0 {
            // Some shards failed
            self.errors = self
                .shards
                .values()
                .filter_map(|s| s.error.clone())
                .collect();
            self.status = ShardGroupStatus::Failed;
        }

        self.completed_at = Some(now_ms());
    }

    /// Get shard by ID
    pub fn get_shard(&self, shard_id: &str) -> Option<&TaskShard> {
        self.shards.get(shard_id)
    }

    /// Get all pending shards
    pub fn get_pending_shards(&self) -> Vec<&TaskShard> {
        self.shards
            .values()
            .filter(|s| s.status == ShardStatus::Pending)
            .collect()
    }

    /// Get all running shards
    pub fn get_running_shards(&self) -> Vec<&TaskShard> {
        self.shards
            .values()
            .filter(|s| {
                matches!(
                    s.status,
                    ShardStatus::RunningPrimary | ShardStatus::RunningReplica
                )
            })
            .collect()
    }
}
