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
//!
//! ## Async vs Sync (M-4 fix)
//! - `reconcile_once()` — synchronous, uses std::thread::sleep (blocking)
//! - `reconcile_once_async()` — async, uses tokio::time::sleep (non-blocking)
//! - Prefer async version in Tauri command handlers to avoid blocking the executor.

use crate::goal::*;
use crate::goal_evaluator::ConditionEvaluator;
use crate::swarm_adapters::{SwarmAgentAdapter, TaskDescription};

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

        let _handle = {
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

        // For QueenJudgment, send output to Queen worker for evaluation
        let eval_result = match &goal.completion_condition {
            CompletionCondition::QueenJudgment { criteria } => {
                self.eval_queen_judgment(&output, criteria, &worker_id)
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

    /// Evaluate QueenJudgment by calling a Queen worker.
    ///
    /// The Queen worker receives the execution output and criteria,
    /// then returns a judgment on whether the goal is converged.
    fn eval_queen_judgment(
        &self,
        output: &str,
        criteria: &str,
        primary_worker_id: &str,
    ) -> EvaluationResult {
        // Find a Queen worker (Claude Code type)
        let workers = match self.workers.lock() {
            Ok(w) => w,
            Err(e) => {
                return EvaluationResult {
                    passed: false,
                    explanation: format!("Worker lock poisoned: {}", e),
                    details: vec![ConditionResult {
                        description: "Queen evaluation".into(),
                        passed: false,
                        evidence: format!("Lock error: {}", e),
                    }],
                };
            }
        };

        // Find Queen worker: prefer Claude Code worker, fallback to any available
        let queen_worker = workers
            .values()
            .find(|w| {
                let caps = w.capabilities();
                caps.worker_type == "claude_code" && caps.worker_id != primary_worker_id
            })
            .or_else(|| {
                // Fallback: any worker except the primary executor
                workers
                    .values()
                    .find(|w| w.capabilities().worker_id != primary_worker_id)
            });

        let queen_adapter = match queen_worker {
            Some(a) => a.clone(),
            None => {
                // No Queen worker available — use self-evaluation fallback
                return EvaluationResult {
                    passed: false,
                    explanation: "No Queen worker available for judgment".into(),
                    details: vec![ConditionResult {
                        description: "Queen evaluation".into(),
                        passed: false,
                        evidence: "No suitable Queen worker found".into(),
                    }],
                };
            }
        };

        // Build Queen evaluation prompt
        let queen_prompt = format!(
            "You are the Queen evaluator. Judge whether the following execution result meets the criteria.\n\n\
            **Criteria**: {}\n\n\
            **Execution Output**:\n{}\n\n\
            **Task**: Respond with exactly one of:\n\
            - 'CONVERGED: <brief explanation>' if the criteria is fully satisfied\n\
            - 'NOT_CONVERGED: <brief explanation>' if the criteria is not yet satisfied\n\n\
            Be strict but fair. If the output clearly demonstrates the criteria is met, say CONVERGED.",
            criteria,
            if output.len() > 4000 {
                format!("{}... (truncated, {} total chars)", output.chars().take(4000).collect::<String>(), output.len())
            } else {
                output.to_string()
            }
        );

        // Send evaluation task to Queen
        let queen_task_id = format!("queen-eval-{}-{}", primary_worker_id, crate::swarm_adapters::now_ms());
        let queen_task = TaskDescription::new(queen_task_id.clone(), queen_prompt)
            .with_timeout(60_000); // 60 seconds for Queen evaluation

        match queen_adapter.send_task(&queen_task) {
            Ok(_handle) => {
                // Poll for Queen response with timeout
                let queen_timeout_ms = 90_000; // 90 seconds
                let queen_poll_interval_ms = 500;
                let queen_start = crate::swarm_adapters::now_ms();
                let mut queen_output = String::new();

                loop {
                    let elapsed = crate::swarm_adapters::now_ms() - queen_start;
                    if elapsed > queen_timeout_ms {
                        let _ = queen_adapter.cancel_task(&queen_task_id);
                        return EvaluationResult {
                            passed: false,
                            explanation: format!("Queen evaluation timed out after {}ms", queen_timeout_ms),
                            details: vec![ConditionResult {
                                description: "Queen evaluation".into(),
                                passed: false,
                                evidence: "Timeout waiting for Queen response".into(),
                            }],
                        };
                    }

                    // Get Queen output
                    if let Ok(task_output) = queen_adapter.get_task_output(&queen_task_id) {
                        if !task_output.is_empty() {
                            queen_output = task_output;
                            break;
                        }
                    }

                    // Check if Queen is still working
                    if let Ok(status) = queen_adapter.get_status() {
                        if status.current_task.is_none() {
                            // Queen finished but we didn't get output — try one more time
                            if let Ok(task_output) = queen_adapter.get_task_output(&queen_task_id) {
                                queen_output = task_output;
                            }
                            break;
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(queen_poll_interval_ms));
                }

                // Parse Queen response
                let queen_output_lower = queen_output.to_lowercase();
                let passed = queen_output_lower.contains("converged")
                    && !queen_output_lower.contains("not_converged");

                EvaluationResult {
                    passed,
                    explanation: if passed {
                        format!("Queen judged as CONVERGED: {}", extract_explanation(&queen_output))
                    } else {
                        format!("Queen judged as NOT_CONVERGED: {}", extract_explanation(&queen_output))
                    },
                    details: vec![ConditionResult {
                        description: format!("Queen judgment: {}", criteria),
                        passed,
                        evidence: format!("Queen response: {} chars", queen_output.len()),
                    }],
                }
            }
            Err(e) => EvaluationResult {
                passed: false,
                explanation: format!("Failed to dispatch Queen evaluation: {}", e),
                details: vec![ConditionResult {
                    description: "Queen evaluation".into(),
                    passed: false,
                    evidence: format!("Dispatch error: {}", e),
                }],
            },
        }
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

/// Extract explanation from Queen response (CONVERGED: xxx or NOT_CONVERGED: xxx).
fn extract_explanation(queen_output: &str) -> String {
    // Look for pattern: CONVERGED: <explanation> or NOT_CONVERGED: <explanation>
    let lower = queen_output.to_lowercase();
    let marker = if lower.contains("converged:") {
        "converged:"
    } else if lower.contains("not_converged:") {
        "not_converged:"
    } else {
        // No clear marker — return truncated output
        return queen_output.chars().take(200).collect::<String>();
    };

    // Find the marker position and extract explanation
    let marker_pos = lower.find(marker).unwrap_or(0);
    let start_pos = marker_pos + marker.len();

    queen_output
        .chars()
        .skip(start_pos)
        .take(300)
        .collect::<String>()
        .trim()
        .to_string()
}

// =========================================================================
// Async versions (M-4 fix — non-blocking execution)
// =========================================================================

impl ReconcileLoop {
    /// Execute a single iteration of the reconcile loop asynchronously.
    ///
    /// Uses tokio::time::sleep instead of std::thread::sleep to avoid
    /// blocking the executor. This is the preferred version for Tauri commands.
    pub async fn reconcile_once_async(&self, goal: &mut Goal) -> GoalStatus {
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

        let _handle = {
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

        // 5. Wait for task to complete (async polling with timeout)
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

        // Async poll loop using tokio::time::sleep
        let timeout_ms = 300_000; // 5 minutes
        let poll_interval_ms = 1_000;
        let start = crate::swarm_adapters::now_ms();
        let mut output = String::new();

        loop {
            let elapsed = crate::swarm_adapters::now_ms() - start;
            if elapsed > timeout_ms {
                let _ = adapter.cancel_task(&task_id);
                goal.status = GoalStatus::Failed {
                    reason: format!("Task timed out after {}ms", timeout_ms),
                };
                return goal.status.clone();
            }

            // Check task output
            if let Ok(task_output) = adapter.get_task_output(&task_id) {
                if !task_output.is_empty() {
                    output = task_output;
                }
            }

            // Check worker status
            if let Ok(status) = adapter.get_status() {
                if status.current_task.is_none() {
                    break;
                }
            }

            // Non-blocking sleep
            tokio::time::sleep(tokio::time::Duration::from_millis(poll_interval_ms)).await;
        }

        // 6. Evaluate the result
        goal.status = GoalStatus::Evaluating;

        let eval_result = match &goal.completion_condition {
            CompletionCondition::QueenJudgment { criteria } => {
                self.eval_queen_judgment(&output, criteria, &worker_id)
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
            tokens_used: 0,
            timestamp: crate::swarm_adapters::now_ms(),
            duration_ms: crate::swarm_adapters::now_ms() - start,
        };

        goal.iterations.push(iteration);

        // 8. Update status
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

    /// Run the full reconcile loop asynchronously until terminal state.
    pub async fn reconcile_until_done_async(&self, goal: &mut Goal) -> GoalStatus {
        loop {
            let status = self.reconcile_once_async(goal).await;
            match &status {
                GoalStatus::Converged => return status,
                GoalStatus::Failed { .. } => return status,
                GoalStatus::BudgetExhausted => return status,
                GoalStatus::Cancelled => return status,
                GoalStatus::Iterating { .. } => continue,
                _ => return status,
            }
        }
    }
}
