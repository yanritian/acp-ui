//! Workflow Stage extensions
//!
//! Provides stage execution traits and utilities

use crate::workflow::{WorkflowStage, StageStatus};

/// Stage executor trait - defines how a stage is executed
pub trait StageExecutor {
    /// Execute a stage
    fn execute(&self, stage: &WorkflowStage) -> Result<String, String>;

    /// Check if stage can be executed
    fn can_execute(&self, stage: &WorkflowStage) -> bool {
        stage.status == StageStatus::Pending || stage.can_retry()
    }

    /// Get executor name
    fn name(&self) -> &str;
}

/// Default stage executor - returns placeholder result
pub struct DefaultStageExecutor {
    name: String,
}

impl DefaultStageExecutor {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl StageExecutor for DefaultStageExecutor {
    fn execute(&self, stage: &WorkflowStage) -> Result<String, String> {
        // Default executor just returns success
        Ok(format!("Stage '{}' executed by default executor", stage.id))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stage transition helper
pub struct StageTransition;

impl StageTransition {
    /// Mark stage as running
    pub fn start(stage: &mut WorkflowStage) {
        stage.status = StageStatus::Running;
        stage.error = None;
    }

    /// Mark stage as completed with output
    pub fn complete(stage: &mut WorkflowStage, output: String) {
        stage.status = StageStatus::Completed;
        stage.output = Some(output);
        stage.error = None;
    }

    /// Mark stage as failed with error
    pub fn fail(stage: &mut WorkflowStage, error: String) {
        stage.status = StageStatus::Failed;
        stage.error = Some(error);
        stage.retries += 1;
    }

    /// Skip stage with reason
    pub fn skip(stage: &mut WorkflowStage, reason: String) {
        stage.status = StageStatus::Skipped;
        stage.output = Some(reason);
    }

    /// Retry stage (reset to pending)
    pub fn retry(stage: &mut WorkflowStage) -> bool {
        if stage.can_retry() {
            stage.status = StageStatus::Pending;
            stage.error = None;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stage_executor() {
        let executor = DefaultStageExecutor::new("test-executor");
        assert_eq!(executor.name(), "test-executor");

        let stage = WorkflowStage::new("stage-1", "Test");
        let result = executor.execute(&stage);
        assert!(result.is_ok());
    }

    #[test]
    fn stage_transition_start() {
        let mut stage = WorkflowStage::new("stage-1", "Test");
        assert_eq!(stage.status, StageStatus::Pending);

        StageTransition::start(&mut stage);
        assert_eq!(stage.status, StageStatus::Running);
        assert!(stage.error.is_none());
    }

    #[test]
    fn stage_transition_complete() {
        let mut stage = WorkflowStage::new("stage-1", "Test");
        StageTransition::start(&mut stage);

        StageTransition::complete(&mut stage, "Output data".to_string());
        assert_eq!(stage.status, StageStatus::Completed);
        assert_eq!(stage.output, Some("Output data".to_string()));
    }

    #[test]
    fn stage_transition_fail() {
        let mut stage = WorkflowStage::new("stage-1", "Test");
        StageTransition::start(&mut stage);

        StageTransition::fail(&mut stage, "Something went wrong".to_string());
        assert_eq!(stage.status, StageStatus::Failed);
        assert_eq!(stage.error, Some("Something went wrong".to_string()));
        assert_eq!(stage.retries, 1);
    }

    #[test]
    fn stage_transition_skip() {
        let mut stage = WorkflowStage::new("stage-1", "Test");

        StageTransition::skip(&mut stage, "Dependency failed".to_string());
        assert_eq!(stage.status, StageStatus::Skipped);
        assert_eq!(stage.output, Some("Dependency failed".to_string()));
    }

    #[test]
    fn stage_transition_retry() {
        let mut stage = WorkflowStage::new("stage-1", "Test").with_max_retries(2);
        StageTransition::start(&mut stage);
        StageTransition::fail(&mut stage, "Error".to_string());

        // Can retry (retries=1, max=2)
        assert!(StageTransition::retry(&mut stage));
        assert_eq!(stage.status, StageStatus::Pending);

        // Fail again
        StageTransition::start(&mut stage);
        StageTransition::fail(&mut stage, "Error again".to_string());
        assert_eq!(stage.retries, 2);

        // Cannot retry anymore
        assert!(!StageTransition::retry(&mut stage));
    }

    #[test]
    fn stage_executor_can_execute() {
        let executor = DefaultStageExecutor::new("test");

        let pending = WorkflowStage::new("s1", "Test");
        assert!(executor.can_execute(&pending));

        let mut failed = WorkflowStage::new("s2", "Test").with_max_retries(1);
        failed.status = StageStatus::Failed;
        failed.retries = 0;
        assert!(executor.can_execute(&failed));

        let mut running = WorkflowStage::new("s3", "Test");
        running.status = StageStatus::Running;
        assert!(!executor.can_execute(&running));
    }
}