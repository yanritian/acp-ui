//! Task Partitioner - Breaks down complex tasks into shards
//!
//! # DEPRECATED
//!
//! This module is superseded by the Goal-driven architecture (`goal.rs`,
//! `goal_graph.rs`, `reconcile.rs`). Use `GoalGraph` for task decomposition
//! and `ReconcileLoop` for iterative execution.
//!
//! The TaskShard/TaskPartitioner system will be removed in a future release.

#![allow(dead_code)] // DEPRECATED - Use Goal-driven architecture instead

use crate::swarm_adapters::{SwarmError, WorkerCapabilities};
use crate::swarm_types::{ShardGroup, TaskPayload, TaskShard, WorkerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Task Decomposition Result
// ---------------------------------------------------------------------------

/// Decomposition of a task into subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDecomposition {
    /// Original task prompt
    pub original_prompt: String,
    /// Analysis of the task
    pub analysis: TaskAnalysisResult,
    /// Decomposed subtasks
    pub subtasks: Vec<SubTask>,
    /// Recommended worker assignments
    pub worker_assignments: HashMap<String, WorkerId>,
    /// Whether to use replicas for fault tolerance
    pub use_replicas: bool,
    /// Number of replica workers per shard
    pub replica_count: u32,
}

/// Analysis result for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAnalysisResult {
    /// Detected task type
    pub task_type: String,
    /// Estimated complexity level
    pub complexity_level: u8,
    /// Whether task is parallelizable
    pub is_parallelizable: bool,
    /// Number of recommended shards
    pub recommended_shards: u32,
    /// Estimated execution time per shard (ms)
    pub estimated_time_per_shard: u64,
    /// Required capabilities for workers
    pub required_capabilities: Vec<String>,
    /// Dependencies between subtasks (if not fully parallel)
    pub dependencies: Vec<SubTaskDependency>,
}

/// A decomposed subtask
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubTask {
    /// Subtask identifier
    pub id: String,
    /// Subtask index
    pub index: u32,
    /// Prompt for this subtask
    pub prompt: String,
    /// Expected output format
    pub expected_output: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Estimated complexity (1-5)
    pub estimated_complexity: u8,
    /// Whether this subtask is critical (must succeed for overall success)
    pub is_critical: bool,
    /// Working directory override
    pub working_dir: Option<String>,
    /// Context files needed
    pub context_files: HashMap<String, String>,
}

/// Dependency between subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubTaskDependency {
    /// Subtask that depends on another
    pub dependent_id: String,
    /// Subtask that must complete first
    pub depends_on_id: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
}

/// Type of dependency between subtasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    /// Output of one is input to another
    DataFlow,
    /// Both must complete before next stage
    SyncPoint,
    /// One must start before another can start
    OrderConstraint,
}

// ---------------------------------------------------------------------------
// Task Partitioner
// ---------------------------------------------------------------------------

/// Task partitioner that breaks down complex tasks
pub struct TaskPartitioner {
    /// Maximum shards per task
    max_shards: u32,
    /// Default replica count
    default_replica_count: u32,
}

impl TaskPartitioner {
    /// Create a new task partitioner
    pub fn new() -> Self {
        Self {
            max_shards: 10,
            default_replica_count: 1, // One replica per shard
        }
    }

    /// Set maximum shards per task
    pub fn with_max_shards(mut self, max: u32) -> Self {
        self.max_shards = max;
        self
    }

    /// Analyze a task and determine if decomposition is needed
    pub fn analyze_task(&self, prompt: &str) -> TaskAnalysisResult {
        // Heuristic-based task analysis (deprecated — use Goal system instead)

        // Determine task type based on prompt content
        let task_type = self.detect_task_type(prompt);
        let is_parallelizable = self.is_parallelizable(prompt, &task_type);
        let recommended_shards = if is_parallelizable {
            self.calculate_shard_count(prompt, &task_type)
        } else {
            1
        };

        TaskAnalysisResult {
            task_type,
            complexity_level: self.estimate_complexity(prompt),
            is_parallelizable,
            recommended_shards,
            estimated_time_per_shard: self.estimate_time(prompt),
            required_capabilities: self.detect_required_capabilities(prompt),
            dependencies: Vec::new(), // Simplified: no dependencies for now
        }
    }

    /// Decompose a task into subtasks
    pub fn decompose(&self, prompt: &str) -> TaskDecomposition {
        let analysis = self.analyze_task(prompt);

        let subtasks = if analysis.recommended_shards > 1 {
            self.create_subtasks(prompt, &analysis)
        } else {
            // Single task - no decomposition
            vec![SubTask {
                id: format!("subtask-{}-0", uuid()), // In production: use proper UUID
                index: 0,
                prompt: prompt.to_string(),
                expected_output: "text".to_string(),
                required_capabilities: analysis.required_capabilities.clone(),
                estimated_complexity: analysis.complexity_level,
                is_critical: true,
                working_dir: None,
                context_files: HashMap::new(),
            }]
        };

        let complexity_level = analysis.complexity_level;

        TaskDecomposition {
            original_prompt: prompt.to_string(),
            analysis,
            subtasks,
            worker_assignments: HashMap::new(), // Assigned later by orchestrator
            use_replicas: complexity_level > 3,
            replica_count: if complexity_level > 3 { 1 } else { 0 },
        }
    }

    /// Assign workers to subtasks based on capabilities
    pub fn assign_workers(
        &self,
        decomposition: &mut TaskDecomposition,
        available_workers: &[WorkerCapabilities],
    ) -> Result<(), SwarmError> {
        for subtask in &decomposition.subtasks {
            // Find best worker for this subtask
            let best_worker = self.find_best_worker(subtask, available_workers)?;

            decomposition
                .worker_assignments
                .insert(subtask.id.clone(), best_worker.worker_id.clone());
        }

        Ok(())
    }

    /// Find the best worker for a subtask
    fn find_best_worker(
        &self,
        subtask: &SubTask,
        workers: &[WorkerCapabilities],
    ) -> Result<WorkerCapabilities, SwarmError> {
        // Score each worker based on:
        // 1. Capability match
        // 2. Complexity handling
        // 3. Current load (if available)

        let mut best_score = 0;
        let mut best_worker: Option<&WorkerCapabilities> = None;

        for worker in workers {
            let capability_match =
                self.score_capability_match(&subtask.required_capabilities, &worker.capabilities);
            let complexity_score = if worker.max_complexity >= subtask.estimated_complexity {
                10
            } else {
                0
            };

            let total_score = capability_match + complexity_score;

            if total_score > best_score {
                best_score = total_score;
                best_worker = Some(worker);
            }
        }

        best_worker
            .cloned()
            .ok_or_else(|| SwarmError::WorkerNotFound("No suitable worker found".to_string()))
    }

    /// Score capability match (0-10)
    fn score_capability_match(&self, required: &[String], available: &[String]) -> u8 {
        let matched = required
            .iter()
            .filter(|r| available.iter().any(|a| a.eq_ignore_ascii_case(r)))
            .count();

        if required.is_empty() {
            return 5; // No specific requirements
        }

        ((matched as f32 / required.len() as f32) * 10.0) as u8
    }

    /// Create subtasks from decomposition (deprecated — use GoalGraph instead)
    fn create_subtasks(&self, prompt: &str, analysis: &TaskAnalysisResult) -> Vec<SubTask> {
        // Heuristic-based decomposition

        let shard_count = analysis.recommended_shards.min(self.max_shards);

        // Example decomposition for "write login page"
        if prompt.contains("login") || prompt.contains("登录") {
            return self.decompose_login_page(prompt);
        }

        // Generic decomposition: split by functional areas
        let mut subtasks = Vec::new();
        for i in 0..shard_count {
            subtasks.push(SubTask {
                id: format!("subtask-{}-{}", uuid(), i),
                index: i,
                prompt: format!("Part {} of: {}", i + 1, prompt),
                expected_output: "text".to_string(),
                required_capabilities: analysis.required_capabilities.clone(),
                estimated_complexity: (analysis.complexity_level / shard_count as u8).max(1),
                is_critical: i == 0, // First shard is critical
                working_dir: None,
                context_files: HashMap::new(),
            });
        }

        subtasks
    }

    /// Decompose a login page task into specific components
    fn decompose_login_page(&self, prompt: &str) -> Vec<SubTask> {
        vec![
            SubTask {
                id: format!("subtask-login-{}-0", uuid()),
                index: 0,
                prompt: format!("Create the main login page component: {}", prompt),
                expected_output: "code".to_string(),
                required_capabilities: vec!["code_generation".to_string(), "frontend".to_string()],
                estimated_complexity: 3,
                is_critical: true,
                working_dir: None,
                context_files: HashMap::new(),
            },
            SubTask {
                id: format!("subtask-login-{}-1", uuid()),
                index: 1,
                prompt: "Create form validation logic for login page".to_string(),
                expected_output: "code".to_string(),
                required_capabilities: vec![
                    "code_generation".to_string(),
                    "validation".to_string(),
                ],
                estimated_complexity: 2,
                is_critical: false,
                working_dir: None,
                context_files: HashMap::new(),
            },
            SubTask {
                id: format!("subtask-login-{}-2", uuid()),
                index: 2,
                prompt: "Create unit tests for login page component".to_string(),
                expected_output: "code".to_string(),
                required_capabilities: vec!["test_writing".to_string()],
                estimated_complexity: 2,
                is_critical: false,
                working_dir: None,
                context_files: HashMap::new(),
            },
        ]
    }

    /// Detect task type from prompt
    fn detect_task_type(&self, prompt: &str) -> String {
        if prompt.contains("test") || prompt.contains("测试") {
            "test_writing".to_string()
        } else if prompt.contains("review") || prompt.contains("审查") {
            "code_review".to_string()
        } else if prompt.contains("refactor") || prompt.contains("重构") {
            "refactoring".to_string()
        } else if prompt.contains("fix") || prompt.contains("bug") || prompt.contains("修复") {
            "bug_fix".to_string()
        } else if prompt.contains("login") || prompt.contains("page") || prompt.contains("页面") {
            "feature_implementation".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Check if task is parallelizable
    fn is_parallelizable(&self, prompt: &str, task_type: &str) -> bool {
        // Tasks involving multiple files or components are parallelizable
        match task_type {
            "feature_implementation" => prompt.contains("and") || prompt.contains("以及"),
            "test_writing" => prompt.contains("files") || prompt.contains("多个"),
            "refactoring" => prompt.contains("modules") || prompt.contains("模块"),
            _ => false,
        }
    }

    /// Calculate shard count
    fn calculate_shard_count(&self, prompt: &str, task_type: &str) -> u32 {
        // Simple heuristic based on prompt length and task type
        let base_count = match task_type {
            "feature_implementation" => 3,
            "test_writing" => 2,
            "refactoring" => 2,
            _ => 1,
        };

        // Scale by prompt complexity indicators
        let indicators = ["multiple", "多个", "components", "组件", "files", "文件"];
        let indicator_count = indicators.iter().filter(|i| prompt.contains(*i)).count();

        (base_count + indicator_count as u32).min(self.max_shards)
    }

    /// Estimate complexity (1-5)
    fn estimate_complexity(&self, prompt: &str) -> u8 {
        let length_score = if prompt.len() > 500 {
            2
        } else if prompt.len() > 200 {
            1
        } else {
            0
        };
        let keyword_score =
            prompt.matches("complex").count() as u8 + prompt.matches("复杂").count() as u8;

        (1 + length_score + keyword_score).min(5)
    }

    /// Estimate execution time (ms)
    fn estimate_time(&self, prompt: &str) -> u64 {
        // Simple heuristic: 30 seconds base + complexity multiplier
        30000 + self.estimate_complexity(prompt) as u64 * 60000
    }

    /// Detect required capabilities from prompt
    fn detect_required_capabilities(&self, prompt: &str) -> Vec<String> {
        let mut capabilities = Vec::new();

        if prompt.contains("test") || prompt.contains("测试") {
            capabilities.push("test_writing".to_string());
        }
        if prompt.contains("review") || prompt.contains("审查") {
            capabilities.push("code_review".to_string());
        }
        if prompt.contains("refactor") || prompt.contains("重构") {
            capabilities.push("refactoring".to_string());
        }
        if prompt.contains("debug") || prompt.contains("debugging") || prompt.contains("调试") {
            capabilities.push("debugging".to_string());
        }
        if prompt.contains("frontend") || prompt.contains("UI") || prompt.contains("前端") {
            capabilities.push("frontend".to_string());
        }
        if prompt.contains("backend") || prompt.contains("API") || prompt.contains("后端") {
            capabilities.push("backend".to_string());
        }

        if capabilities.is_empty() {
            capabilities.push("code_generation".to_string());
        }

        capabilities
    }

    /// Create shards from decomposition
    pub fn create_shards(
        &self,
        decomposition: &TaskDecomposition,
        parent_task_id: String,
    ) -> ShardGroup {
        let mut group = ShardGroup::new(
            parent_task_id.clone(),
            decomposition.original_prompt.clone(),
        );
        group.status = crate::swarm_types::ShardGroupStatus::Assigning;

        for subtask in &decomposition.subtasks {
            let worker_id = decomposition
                .worker_assignments
                .get(&subtask.id)
                .cloned()
                .unwrap_or_else(|| "default-worker".to_string());

            let payload = TaskPayload::new(subtask.prompt.clone());

            let shard = TaskShard::new(
                subtask.id.clone(),
                group.task_id.clone(),
                subtask.index,
                decomposition.subtasks.len() as u32,
                payload,
                worker_id,
            );

            group.add_shard(shard);
        }

        group
    }
}

impl Default for TaskPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unique ID for subtasks
fn uuid() -> String {
    Uuid::new_v4().to_string()
}
