//! Reconcile Loop — iterative Goal execution with evaluation feedback
//!
//! The reconcile loop is the core of the Goal-driven architecture:
//! 1. Budget check (token + iteration limits)
//! 2. Dispatch to assigned Worker
//! 3. Evaluate completion condition
//! 4. If passed → Converged
//! 5. If failed → Record feedback, retry with context
//!
//! This replaces the old "fire and forget" task model with an iterative
//! convergence model inspired by Kubernetes controllers.

use crate::goal::*;
use crate::goal_evaluator::ConditionEvaluator;
use crate::swarm_adapters::{SwarmAgentAdapter, TaskDescription, TaskStatus};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The reconcile loop engine.
///
/// Holds references to worker adapters and manages goal execution.
pub struct ReconcileLoop {
    /// Worker adapters for dispatching tasks
    workers: Arc<Mutex<HashMap<String, Arc<dyn SwarmAgentAdapter + Send + Sync>>>>,
    /// Condition evaluator
    evaluator: ConditionEvaluator,
}

impl ReconcileLoop {
    pub fn new(
        workers: Arc<Mutex<HashMap<String, Arc<dyn SwarmAgentAdapter + Send + Sync>>>>,
    ) -> Self {
        Self {
            workers,
            evaluator: ConditionEvaluator::new(),
        }
    }

    /// Execute a single iteration of the reconcile loop for a goal.
    ///
    /// Returns the updated GoalStatus after this iteration.
    /// The caller should check if the status is terminal (Converged, Failed, etc.)
    /// and decide whether to call again.
    pub fn reconcile_once(&self, goal: &mut Goal) -> GoalStatus {
        // 1. Pre-checks
        if goal.is_budget_exhausted() {
            goal.status = GoalStatus::BudgetExhausted;
            return GoalStatus::BudgetExhausted;
        }

        if goal.is_max_iterations_reached() {
            goal.status = GoalStatus::Failed {
                reason: format!(
                    "Max iterations ({}) exceeded without convergence",
                    goal.max_iterations
                ),
            };
            return goal.status.clone();
        }

        let worker_id = match &goal.assigned_worker {
            Some(id) => id.clone(),
            None => {
                goal.status = GoalStatus::Failed {
                    reason: "No worker assigned to goal".into(),
                };
                return goal.status.clone();
            }
        };

        // 2. Build prompt with feedback from previous iteration
        let prompt = self.build_prompt(goal);

        // 3. Mark as Active
        goal.status = GoalStatus::Active;
        if goal.started_at.is_none() {
            goal.started_at = Some(crate::swarm_adapters::now_ms());
        }

        // 4. Dispatch to worker
        let task_id = format!("goal-{}-iter-{}", goal.id, goal.current_iteration());
        let task = TaskDescription::new(task_id.clone(), prompt);

        let handle = {
            let workers = match self.workers.lock() {
                Ok(w) => w,
                Err(e) => {
                    goal.status = GoalStatus::Failed {
                        reason: format!("Worker lock poisoned: {}", e),
                    };
                    return goal.status.clone();
                }
            };

            let adapter = match workers.get(&worker_id) {
                Some(a) => a.clone(),
                None => {
                    goal.status = GoalStatus::Failed {
                        reason: format!("Worker '{}' not found", worker_id),
                    };
                    return goal.status.clone();
                }
            };

            match adapter.send_task(&task) {
                Ok(h) => h,
                Err(e) => {
                    goal.status = GoalStatus::Failed {
                        reason: format!("Dispatch failed: {}", e),
                    };
                    return goal.status.clone();
                }
            }
        };

        // 5. Wait for task to complete (poll with timeout)
        let adapter = {
            let workers = match self.workers.lock() {
                Ok(w) => w,
                Err(e) => {
                    goal.status = GoalStatus::Failed {
                        reason: format!("Worker lock poisoned: {}", e),
                    };
                    return goal.status.clone();
                }
            };
            workers.get(&worker_id).cloned()
        };

        let adapter = match adapter {
            Some(a) => a,
            None => {
                goal.status = GoalStatus::Failed {
                    reason: format!("Worker '{}' disappeared", worker_id),
                };
                return goal.status.clone();
            }
        };

        // Poll for completion (blocking with timeout)
        // NOTE: This polling loop uses std::thread::sleep which blocks the OS thread.
        // When called from a tokio async context, wrap the caller in
        // tokio::task::spawn_blocking() to avoid starving the executor.
        // TODO(Phase 3): Convert ReconcileLoop to async and use tokio::time::sleep.
        let timeout_ms = 300_000; // 5 minutes
        let poll_interval_ms = 1_000;
        let start = crate::swarm_adapters::now_ms();
        let mut output = String::new();

        loop {
            let elapsed = crate::swarm_adapters::now_ms() - start;
            if elapsed > timeout_ms {
                // Try to cancel the task
                let _ = adapter.cancel_task(&task_id);
                goal.status = GoalStatus::Failed {
                    reason: format!("Task timed out after {}ms", timeout_ms),
                };
                return goal.status.clone();
            }

            // Check task status
            if let Ok(task_output) = adapter.get_task_output(&task_id) {
                if !task_output.is_empty() {
                    output = task_output;
                }
            }

            // Get current task status from the handle in the adapter
            if let Ok(status) = adapter.get_status() {
                if status.current_task.is_none() {
                    // Task completed (worker is idle again)
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms));
        }

        // 6. Evaluate the result
        goal.status = GoalStatus::Evaluating;

        // For QueenJudgment, we defer evaluation (requires a separate Queen call)
        let eval_result = match &goal.completion_condition {
            CompletionCondition::QueenJudgment { .. } => {
                // TODO(Phase 3): Send output to Queen worker for judgment
                EvaluationResult {
                    passed: false,
                    explanation: "QueenJudgment not yet wired — treating as failed for retry".into(),
                    details: vec![],
                }
            }
            condition => self.evaluator.eval(condition, &output),
        };

        // 7. Record iteration
        let iteration = IterationRecord {
            iteration: goal.current_iteration(),
            worker_output: output.clone(),
            evaluation: eval_result.clone(),
            feedback: if eval_result.passed {
                None
            } else {
                Some(eval_result.explanation.clone())
            },
            tokens_used: 0, // TODO: Track tokens from worker output
            timestamp: crate::swarm_adapters::now_ms(),
            duration_ms: crate::swarm_adapters::now_ms() - start,
        };

        goal.iterations.push(iteration);

        // 8. Update status based on evaluation
        if eval_result.passed {
            goal.status = GoalStatus::Converged;
            goal.converged_at = Some(crate::swarm_adapters::now_ms());
        } else {
            goal.status = GoalStatus::Iterating {
                feedback: eval_result.explanation,
            };
        }

        goal.status.clone()
    }

    /// Run the full reconcile loop until the goal reaches a terminal state.
    pub fn reconcile_until_done(&self, goal: &mut Goal) -> GoalStatus {
        loop {
            let status = self.reconcile_once(goal);
            match &status {
                GoalStatus::Converged => return status,
                GoalStatus::Failed { .. } => return status,
                GoalStatus::BudgetExhausted => return status,
                GoalStatus::Cancelled => return status,
                GoalStatus::Iterating { .. } => continue,
                _ => {
                    // Active or Evaluating should not be returned from reconcile_once
                    // but handle defensively
                    return status;
                }
            }
        }
    }

    /// Build the prompt for the worker, including feedback from previous iterations.
    fn build_prompt(&self, goal: &Goal) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!("Goal: {}\n\n", goal.description));

        // Add completion condition description
        prompt.push_str(&format!(
            "Success criteria: {}\n\n",
            describe_condition(&goal.completion_condition)
        ));

        // Add feedback from previous iteration
        if let Some(feedback) = goal.latest_feedback() {
            prompt.push_str("--- Previous Attempt Feedback ---\n");
            prompt.push_str(feedback);
            prompt.push_str("\n\nPlease address the above issues and try again.\n");
        }

        // Add iteration context
        prompt.push_str(&format!(
            "\nAttempt {}/{} (tokens used: {})\n",
            goal.current_iteration(),
            goal.max_iterations,
            goal.token_used,
        ));

        prompt
    }
}

/// Human-readable description of a completion condition.
fn describe_condition(condition: &CompletionCondition) -> String {
    match condition {
        CompletionCondition::CommandSuccess { command, expected_exit_code } => {
            format!("Command '{}' exits with code {}", command, expected_exit_code)
        }
        CompletionCondition::OutputContains { text, case_sensitive } => {
            format!("Output contains '{}' (case_sensitive={})", text, case_sensitive)
        }
        CompletionCondition::OutputMatches { pattern } => {
            format!("Output matches regex '{}'", pattern)
        }
        CompletionCondition::All { conditions } => {
            let descs: Vec<String> = conditions.iter().map(describe_condition).collect();
            format!("ALL of: [{}]", descs.join("; "))
        }
        CompletionCondition::Any { conditions } => {
            let descs: Vec<String> = conditions.iter().map(describe_condition).collect();
            format!("ANY of: [{}]", descs.join("; "))
        }
        CompletionCondition::FileCheck { path, must_exist, content_contains } => {
            let exist_desc = if *must_exist {
                format!("File '{}' exists", path)
            } else {
                format!("File '{}' does NOT exist", path)
            };
            match content_contains {
                Some(text) => format!("{} AND contains '{}'", exist_desc, text),
                None => exist_desc,
            }
        }
        CompletionCondition::HttpHealthCheck { url, expected_status } => {
            format!("HTTP GET '{}' returns status {}", url, expected_status)
        }
        CompletionCondition::QueenJudgment { criteria } => {
            format!("Queen judges: {}", criteria)
        }
    }
}
