//! Workflow definition and execution
//!
//! A Workflow is a multi-stage orchestrated process with:
//! - DAG-based stage dependencies
//! - Checkpoint support for rollback
//! - Status tracking for each stage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow created but not started
    Pending,
    /// Workflow is running
    Running,
    /// Workflow paused (manual or checkpoint)
    Paused,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow cancelled by user
    Cancelled,
}

impl WorkflowStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_success(&self) -> bool {
        *self == Self::Completed
    }
}

/// Workflow stage status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    /// Stage not yet started
    Pending,
    /// Stage is running
    Running,
    /// Stage completed successfully
    Completed,
    /// Stage failed
    Failed,
    /// Stage skipped (dependency failed)
    Skipped,
}

impl StageStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Skipped)
    }
}

/// Workflow stage definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStage {
    /// Stage unique ID
    pub id: String,
    /// Stage name
    pub name: String,
    /// Stage description
    pub description: String,
    /// Dependencies (stage IDs that must complete before this stage)
    pub dependencies: Vec<String>,
    /// Stage status
    pub status: StageStatus,
    /// Stage output/result
    pub output: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count
    pub retries: u32,
    /// Max retries allowed
    pub max_retries: u32,
}

impl WorkflowStage {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            dependencies: Vec::new(),
            status: StageStatus::Pending,
            output: None,
            error: None,
            retries: 0,
            max_retries: 3,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_dependency(mut self, dep_id: impl Into<String>) -> Self {
        self.dependencies.push(dep_id.into());
        self
    }

    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    pub fn is_ready(&self, completed_stages: &[&str]) -> bool {
        self.status == StageStatus::Pending
            && self.dependencies.iter().all(|dep| completed_stages.contains(&dep.as_str()))
    }

    pub fn can_retry(&self) -> bool {
        self.status == StageStatus::Failed && self.retries < self.max_retries
    }
}

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow unique ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow description
    pub description: String,
    /// All stages
    pub stages: Vec<WorkflowStage>,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Current stage index (for sequential execution)
    pub current_stage_index: usize,
    /// Workflow metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp (UNIX ms)
    pub created_at: u64,
    /// Start timestamp
    pub started_at: Option<u64>,
    /// Completion timestamp
    pub completed_at: Option<u64>,
}

impl Workflow {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            stages: Vec::new(),
            status: WorkflowStatus::Pending,
            current_stage_index: 0,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            started_at: None,
            completed_at: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_stage(mut self, stage: WorkflowStage) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn add_stage(&mut self, stage: WorkflowStage) {
        self.stages.push(stage);
    }

    /// Get stages ready to execute (dependencies satisfied)
    pub fn ready_stages(&self) -> Vec<&WorkflowStage> {
        let completed: Vec<&str> = self
            .stages
            .iter()
            .filter(|s| s.status == StageStatus::Completed)
            .map(|s| s.id.as_str())
            .collect();

        self.stages
            .iter()
            .filter(|s| s.is_ready(&completed))
            .collect()
    }

    /// Get stage by ID
    pub fn get_stage(&self, id: &str) -> Option<&WorkflowStage> {
        self.stages.iter().find(|s| s.id == id)
    }

    /// Get mutable stage by ID
    pub fn get_stage_mut(&mut self, id: &str) -> Option<&mut WorkflowStage> {
        self.stages.iter_mut().find(|s| s.id == id)
    }

    /// Update stage status
    pub fn update_stage_status(&mut self, stage_id: &str, status: StageStatus) {
        if let Some(stage) = self.get_stage_mut(stage_id) {
            stage.status = status;
        }
        self.update_workflow_status();
    }

    /// Update workflow status based on stage statuses
    fn update_workflow_status(&mut self) {
        if self.status.is_terminal() {
            return;
        }

        let all_completed = self.stages.iter().all(|s| s.status == StageStatus::Completed);
        let any_failed = self.stages.iter().any(|s| s.status == StageStatus::Failed);
        let any_running = self.stages.iter().any(|s| s.status == StageStatus::Running);

        if all_completed {
            self.status = WorkflowStatus::Completed;
            self.completed_at = Some(now_ms());
        } else if any_failed && !any_running {
            // Check if any stage can retry
            let can_retry = self.stages.iter().any(|s| s.can_retry());
            if !can_retry {
                self.status = WorkflowStatus::Failed;
                self.completed_at = Some(now_ms());
            }
        }
    }

    /// Start workflow
    pub fn start(&mut self) {
        if self.status == WorkflowStatus::Pending {
            self.status = WorkflowStatus::Running;
            self.started_at = Some(now_ms());
        }
    }

    /// Pause workflow
    pub fn pause(&mut self) {
        if self.status == WorkflowStatus::Running {
            self.status = WorkflowStatus::Paused;
        }
    }

    /// Resume workflow
    pub fn resume(&mut self) {
        if self.status == WorkflowStatus::Paused {
            self.status = WorkflowStatus::Running;
        }
    }

    /// Cancel workflow
    pub fn cancel(&mut self) {
        if !self.status.is_terminal() {
            self.status = WorkflowStatus::Cancelled;
            self.completed_at = Some(now_ms());
            // Mark all pending stages as skipped
            for stage in &mut self.stages {
                if stage.status == StageStatus::Pending {
                    stage.status = StageStatus::Skipped;
                }
            }
        }
    }

    /// Get progress percentage
    pub fn progress(&self) -> f32 {
        if self.stages.is_empty() {
            return 100.0;
        }
        let completed = self.stages.iter().filter(|s| s.status == StageStatus::Completed).count();
        (completed as f32 / self.stages.len() as f32) * 100.0
    }
}

/// Workflow execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub status: WorkflowStatus,
    pub total_stages: usize,
    pub completed_stages: usize,
    pub failed_stages: usize,
    pub pending_stages: usize,
    pub progress: f32,
    pub duration_ms: Option<u64>,
}

impl Workflow {
    pub fn summary(&self) -> WorkflowSummary {
        WorkflowSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status,
            total_stages: self.stages.len(),
            completed_stages: self.stages.iter().filter(|s| s.status == StageStatus::Completed).count(),
            failed_stages: self.stages.iter().filter(|s| s.status == StageStatus::Failed).count(),
            pending_stages: self.stages.iter().filter(|s| s.status == StageStatus::Pending).count(),
            progress: self.progress(),
            duration_ms: self.completed_at.and_then(|end| self.started_at.map(|start| end - start)),
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_new() {
        let wf = Workflow::new("wf-001", "Test Workflow");
        assert_eq!(wf.id, "wf-001");
        assert_eq!(wf.name, "Test Workflow");
        assert_eq!(wf.status, WorkflowStatus::Pending);
        assert!(wf.stages.is_empty());
    }

    #[test]
    fn workflow_with_stages() {
        let wf = Workflow::new("wf-001", "Test")
            .with_stage(WorkflowStage::new("stage-1", "First"))
            .with_stage(WorkflowStage::new("stage-2", "Second"));

        assert_eq!(wf.stages.len(), 2);
    }

    #[test]
    fn stage_dependencies() {
        let stage = WorkflowStage::new("stage-2", "Second")
            .with_dependency("stage-1");

        assert_eq!(stage.dependencies.len(), 1);
        assert!(!stage.is_ready(&[]));
        assert!(!stage.is_ready(&["other"]));
        assert!(stage.is_ready(&["stage-1"]));
    }

    #[test]
    fn workflow_start_pause_resume() {
        let mut wf = Workflow::new("wf-001", "Test");
        assert_eq!(wf.status, WorkflowStatus::Pending);

        wf.start();
        assert_eq!(wf.status, WorkflowStatus::Running);
        assert!(wf.started_at.is_some());

        wf.pause();
        assert_eq!(wf.status, WorkflowStatus::Paused);

        wf.resume();
        assert_eq!(wf.status, WorkflowStatus::Running);
    }

    #[test]
    fn workflow_cancel() {
        let mut wf = Workflow::new("wf-001", "Test")
            .with_stage(WorkflowStage::new("stage-1", "First"));

        wf.start();
        wf.cancel();

        assert_eq!(wf.status, WorkflowStatus::Cancelled);
        assert_eq!(wf.stages[0].status, StageStatus::Skipped);
    }

    #[test]
    fn workflow_progress() {
        let mut wf = Workflow::new("wf-001", "Test")
            .with_stage(WorkflowStage::new("stage-1", "First"))
            .with_stage(WorkflowStage::new("stage-2", "Second"));

        assert_eq!(wf.progress(), 0.0);

        wf.update_stage_status("stage-1", StageStatus::Completed);
        assert_eq!(wf.progress(), 50.0);

        wf.update_stage_status("stage-2", StageStatus::Completed);
        assert_eq!(wf.progress(), 100.0);
        assert_eq!(wf.status, WorkflowStatus::Completed);
    }

    #[test]
    fn workflow_status_is_terminal() {
        assert!(WorkflowStatus::Completed.is_terminal());
        assert!(WorkflowStatus::Failed.is_terminal());
        assert!(WorkflowStatus::Cancelled.is_terminal());
        assert!(!WorkflowStatus::Pending.is_terminal());
        assert!(!WorkflowStatus::Running.is_terminal());
        assert!(!WorkflowStatus::Paused.is_terminal());
    }

    #[test]
    fn stage_can_retry() {
        let mut stage = WorkflowStage::new("stage-1", "Test").with_max_retries(2);
        stage.status = StageStatus::Failed;

        assert!(stage.can_retry());
        stage.retries = 1;
        assert!(stage.can_retry());
        stage.retries = 2;
        assert!(!stage.can_retry());
    }
}