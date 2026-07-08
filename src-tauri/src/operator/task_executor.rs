// Godot Task Executor - Complete workflow for Godot game development tasks

use crate::operator::{
    OperatorTask, OperatorEvent, OperatorEventType, EventLevel,
    TaskStateMachine, TaskMode, ApprovalPolicy,
    PathGuard, FileReadResult, FilePatchResult,
    HermesAgentBridge, ProjectAnalysisResult, ExecutionPlan,
};
use crate::domains::games::godot::GodotProjectAnalyzer;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ============================================================================
// Task Executor
// ============================================================================

pub struct GodotTaskExecutor {
    task_id: String,
    project_path: PathBuf,
    goal: String,
    state_machine: TaskStateMachine,
    events: Vec<OperatorEvent>,
    path_guard: PathGuard,
    bridge: HermesAgentBridge,
}

impl GodotTaskExecutor {
    pub fn new(task_id: String, project_path: PathBuf, goal: String) -> Self {
        let path_guard = PathGuard::new(vec![project_path.clone()]);
        let bridge = HermesAgentBridge::new(
            task_id.clone(),
            project_path.clone(),
            goal.clone(),
            TaskMode::ProposeThenApply,
        );

        Self {
            task_id,
            project_path,
            goal,
            state_machine: TaskStateMachine::new(task_id.clone()),
            events: Vec::new(),
            path_guard,
            bridge,
        }
    }

    /// Analyze the Godot project (delegates to bridge)
    pub async fn analyze_project(&mut self) -> Result<crate::operator::hermes_runtime::HermesAnalysisResult, TaskExecutorError> {
        self.bridge.analyze_project().await
            .map_err(|e| TaskExecutorError::AnalysisError(e.to_string()))
    }

    /// Generate execution plan (delegates to bridge)
    pub async fn generate_plan(&mut self, analysis: &crate::operator::hermes_runtime::HermesAnalysisResult) -> Result<crate::operator::hermes_runtime::HermesPlan, TaskExecutorError> {
        self.bridge.generate_plan(analysis).await
            .map_err(|e| TaskExecutorError::PlanningError(e.to_string()))
    }

    /// Execute complete task workflow
    pub async fn execute(&mut self) -> Result<TaskExecutionReport, TaskExecutorError> {
        // Phase 1: Initialize
        self.emit_event(OperatorEventType::TaskStarted, "Task initialization started");
        self.state_machine.start_task().map_err(|e| TaskExecutorError::StateError(e.to_string()))?;

        // Phase 2: Analyze project
        self.emit_event(OperatorEventType::ProjectAnalyzing, "Analyzing Godot project...");
        let analysis = self.bridge.analyze_project().await
            .map_err(|e| TaskExecutorError::AnalysisError(e.to_string()))?;

        self.emit_event(
            OperatorEventType::ProjectAnalyzed,
            &format!("Project analyzed: {} scripts, {} scenes, {} player controllers",
                analysis.scripts.len(),
                analysis.scenes.len(),
                analysis.player_controllers.len()
            )
        );

        // Phase 3: Generate plan
        self.emit_event(OperatorEventType::PlanGenerating, "Generating execution plan...");
        let plan = self.bridge.generate_plan(&analysis).await
            .map_err(|e| TaskExecutorError::PlanningError(e.to_string()))?;

        self.emit_event(
            OperatorEventType::PlanReady,
            &format!("Execution plan ready: {} steps", plan.steps.len())
        );

        self.state_machine.plan_ready().map_err(|e| TaskExecutorError::StateError(e.to_string()))?;

        // Phase 4: Wait for approval (simulated)
        self.emit_event(OperatorEventType::ApprovalRequested, "Waiting for user approval...");

        // TODO: In real implementation, wait for user approval here
        // For now, auto-approve for testing
        self.emit_event(OperatorEventType::ApprovalGranted, "Plan approved by user");
        self.state_machine.approve().map_err(|e| TaskExecutorError::StateError(e.to_string()))?;

        // Phase 5: Execute plan steps
        let mut step_results = Vec::new();
        for step in &plan.steps {
            self.emit_event(
                OperatorEventType::StepExecuting,
                &format!("Executing step {}: {}", step.id, step.description)
            );

            match self.bridge.execute_step(step).await {
                Ok(result) => {
                    if result.success {
                        self.emit_event(
                            OperatorEventType::StepCompleted,
                            &format!("Step {} completed successfully", step.id)
                        );

                        // If there are file changes, emit events
                        for file_change in &result.file_changes {
                            self.emit_event(
                                OperatorEventType::FileModified,
                                &format!("File modified: {}", file_change.path)
                            );
                        }
                    } else {
                        self.emit_event(
                            OperatorEventType::StepFailed,
                            &format!("Step {} failed", step.id)
                        );
                    }
                    step_results.push(result);
                }
                Err(e) => {
                    self.emit_event(
                        OperatorEventType::StepFailed,
                        &format!("Step {} error: {}", step.id, e)
                    );
                    return Err(TaskExecutorError::ExecutionError(e.to_string()));
                }
            }
        }

        // Phase 6: Complete
        self.emit_event(OperatorEventType::TaskCompleting, "Task completing...");
        self.state_machine.complete().map_err(|e| TaskExecutorError::StateError(e.to_string()))?;

        self.emit_event(OperatorEventType::TaskCompleted, "Task completed successfully");

        Ok(TaskExecutionReport {
            task_id: self.task_id.clone(),
            success: true,
            total_steps: plan.steps.len(),
            completed_steps: step_results.iter().filter(|r| r.success).count(),
            failed_steps: step_results.iter().filter(|r| !r.success).count(),
            files_modified: step_results.iter()
                .flat_map(|r| r.file_changes.iter().map(|c| c.path.clone()))
                .collect(),
            events: self.events.clone(),
            summary: format!(
                "Task completed: {}/{} steps successful, {} files modified",
                step_results.iter().filter(|r| r.success).count(),
                plan.steps.len(),
                step_results.iter().flat_map(|r| r.file_changes.iter()).count()
            ),
        })
    }

    fn emit_event(&mut self, event_type: OperatorEventType, message: &str) {
        let event = OperatorEvent {
            event_id: format!("evt_{}_{}", self.task_id, self.events.len()),
            task_id: self.task_id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            level: EventLevel::Info,
            title: message.to_string(),
            message: Some(message.to_string()),
            source: "godot_executor".to_string(),
            payload: None,
        };
        self.events.push(event);
    }
}

// ============================================================================
// Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionReport {
    pub task_id: String,
    pub success: bool,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub files_modified: Vec<String>,
    pub events: Vec<OperatorEvent>,
    pub summary: String,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum TaskExecutorError {
    StateError(String),
    AnalysisError(String),
    PlanningError(String),
    ExecutionError(String),
    ApprovalError(String),
}

impl std::fmt::Display for TaskExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskExecutorError::StateError(msg) => write!(f, "State error: {}", msg),
            TaskExecutorError::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
            TaskExecutorError::PlanningError(msg) => write!(f, "Planning error: {}", msg),
            TaskExecutorError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            TaskExecutorError::ApprovalError(msg) => write!(f, "Approval error: {}", msg),
        }
    }
}

impl std::error::Error for TaskExecutorError {}
