//! Goal-Driven Architecture — Core Data Structures (RFC-001)
//!
//! **C-1 Unification**: Goal types now use acp-core's unified definitions.
//! This module re-exports from acp-core and provides local types only where
//! the acp-core versions differ structurally (CompletionCondition, Evaluator).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::swarm_adapters::{now_ms, WorkerId};

// ===========================================================================
// Re-exports from acp-core (C-1 unified types)
// ===========================================================================

pub use acp_core::{
    CompletionConditionSpec, ConditionResult, EvaluationResult, EvaluatorSpec, GoalGraphSummary,
    GoalRuntime as Goal, GoalStatus, IterationRecord,
};

// ===========================================================================
// CompletionCondition (local — different fields from acp_core's CompletionConditionSpec)
//
// Used by goal_evaluator.rs for condition evaluation.
// The acp_core version has different field names (e.g., `args` vs `expected_exit_code`),
// so this local type is kept for compatibility with the evaluation pipeline.
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompletionCondition {
    CommandSuccess {
        command: String,
        expected_exit_code: i32,
    },
    OutputContains {
        text: String,
        case_sensitive: bool,
    },
    OutputMatches {
        pattern: String,
    },
    All {
        conditions: Vec<CompletionCondition>,
    },
    Any {
        conditions: Vec<CompletionCondition>,
    },
    FileCheck {
        path: String,
        must_exist: bool,
        content_contains: Option<String>,
    },
    HttpHealthCheck {
        url: String,
        expected_status: u16,
    },
    QueenJudgment {
        criteria: String,
    },
}

impl CompletionCondition {
    /// Convert to unified CompletionConditionSpec
    pub fn to_spec(&self) -> CompletionConditionSpec {
        match self {
            CompletionCondition::CommandSuccess {
                command,
                expected_exit_code,
            } => CompletionConditionSpec::CommandSuccess {
                command: command.clone(),
                args: vec![expected_exit_code.to_string()],
                cwd: None,
            },
            CompletionCondition::OutputContains {
                text,
                case_sensitive,
            } => CompletionConditionSpec::OutputContains {
                command: "".into(),
                pattern: text.clone(),
                case_sensitive: *case_sensitive,
            },
            CompletionCondition::OutputMatches { pattern } => {
                CompletionConditionSpec::OutputMatches {
                    command: "".into(),
                    regex: pattern.clone(),
                }
            }
            CompletionCondition::All { conditions } => CompletionConditionSpec::All {
                conditions: conditions.iter().map(|c| c.to_spec()).collect(),
            },
            CompletionCondition::Any { conditions } => CompletionConditionSpec::Any {
                conditions: conditions.iter().map(|c| c.to_spec()).collect(),
            },
            CompletionCondition::FileCheck {
                path,
                content_contains,
                ..
            } => CompletionConditionSpec::FileCheck {
                path: path.clone(),
                content_contains: content_contains.clone(),
                max_size_bytes: None,
            },
            CompletionCondition::HttpHealthCheck {
                url,
                expected_status,
            } => CompletionConditionSpec::HttpHealthCheck {
                url: url.clone(),
                method: "GET".into(),
                expected_status: Some(*expected_status),
            },
            CompletionCondition::QueenJudgment { criteria } => {
                CompletionConditionSpec::QueenJudgment {
                    criteria: criteria.clone(),
                }
            }
        }
    }

    /// Convert from unified CompletionConditionSpec
    pub fn from_spec(spec: &CompletionConditionSpec) -> Self {
        match spec {
            CompletionConditionSpec::CommandSuccess { command, .. } => {
                CompletionCondition::CommandSuccess {
                    command: command.clone(),
                    expected_exit_code: 0,
                }
            }
            CompletionConditionSpec::OutputContains {
                pattern,
                case_sensitive,
                ..
            } => CompletionCondition::OutputContains {
                text: pattern.clone(),
                case_sensitive: *case_sensitive,
            },
            CompletionConditionSpec::OutputMatches { regex, .. } => {
                CompletionCondition::OutputMatches {
                    pattern: regex.clone(),
                }
            }
            CompletionConditionSpec::All { conditions } => CompletionCondition::All {
                conditions: conditions.iter().map(|c| Self::from_spec(c)).collect(),
            },
            CompletionConditionSpec::Any { conditions } => CompletionCondition::Any {
                conditions: conditions.iter().map(|c| Self::from_spec(c)).collect(),
            },
            CompletionConditionSpec::FileCheck {
                path,
                content_contains,
                ..
            } => CompletionCondition::FileCheck {
                path: path.clone(),
                must_exist: true,
                content_contains: content_contains.clone(),
            },
            CompletionConditionSpec::HttpHealthCheck {
                url,
                expected_status,
                ..
            } => CompletionCondition::HttpHealthCheck {
                url: url.clone(),
                expected_status: expected_status.unwrap_or(200),
            },
            CompletionConditionSpec::QueenJudgment { criteria } => {
                CompletionCondition::QueenJudgment {
                    criteria: criteria.clone(),
                }
            }
            CompletionConditionSpec::Custom { evaluator } => CompletionCondition::QueenJudgment {
                criteria: evaluator.clone(),
            },
        }
    }
}

// ===========================================================================
// Evaluator (local — different serde format from acp_core's EvaluatorSpec)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Evaluator {
    Auto,
    Queen {
        queen_worker_id: WorkerId,
    },
    Adversarial {
        primary: WorkerId,
        adversary: WorkerId,
    },
    Hybrid {
        auto_conditions: Vec<CompletionCondition>,
        queen_criteria: String,
    },
}

impl Evaluator {
    pub fn to_spec(&self) -> EvaluatorSpec {
        match self {
            Evaluator::Auto => EvaluatorSpec::Auto,
            Evaluator::Queen { queen_worker_id } => EvaluatorSpec::Queen {
                queen_worker_id: queen_worker_id.clone(),
            },
            Evaluator::Adversarial { primary, adversary } => EvaluatorSpec::Adversarial {
                primary: primary.clone(),
                adversary: adversary.clone(),
            },
            Evaluator::Hybrid {
                auto_conditions,
                queen_criteria,
            } => EvaluatorSpec::Hybrid {
                auto_conditions: auto_conditions.iter().map(|c| c.to_spec()).collect(),
                queen_criteria: queen_criteria.clone(),
            },
        }
    }

    pub fn from_spec(spec: &EvaluatorSpec) -> Self {
        match spec {
            EvaluatorSpec::Auto => Evaluator::Auto,
            EvaluatorSpec::Queen { queen_worker_id } => Evaluator::Queen {
                queen_worker_id: queen_worker_id.clone(),
            },
            EvaluatorSpec::Adversarial { primary, adversary } => Evaluator::Adversarial {
                primary: primary.clone(),
                adversary: adversary.clone(),
            },
            EvaluatorSpec::Hybrid {
                auto_conditions,
                queen_criteria,
            } => Evaluator::Hybrid {
                auto_conditions: auto_conditions
                    .iter()
                    .map(|c| CompletionCondition::from_spec(c))
                    .collect(),
                queen_criteria: queen_criteria.clone(),
            },
        }
    }
}

// ===========================================================================
// GoalGraph - Goal Dependency Graph Manager
// ===========================================================================

use std::sync::Mutex;

/// Graph of all goals with dependency tracking and state management.
pub struct GoalGraph {
    goals: Mutex<HashMap<String, Goal>>,
}

impl GoalGraph {
    pub fn new() -> Self {
        Self {
            goals: Mutex::new(HashMap::new()),
        }
    }

    /// Submit a new goal to the graph
    pub fn submit(&self, goal: Goal) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        if goals.contains_key(&goal.id) {
            return Err(format!("Goal '{}' already exists", goal.id));
        }

        for dep_id in &goal.depends_on {
            if !goals.contains_key(dep_id) {
                return Err(format!("Dependency '{}' not found", dep_id));
            }
        }

        goals.insert(goal.id.clone(), goal);
        Ok(())
    }

    /// Get a goal by ID
    pub fn get(&self, id: &str) -> Option<Goal> {
        let goals = self.goals.lock().ok()?;
        goals.get(id).cloned()
    }

    /// Update goal status
    pub fn update_status(&self, id: &str, status: GoalStatus) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;
        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;
        goal.status = status;
        Ok(())
    }

    /// Add an iteration record to a goal
    pub fn add_iteration(&self, id: &str, record: IterationRecord) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;
        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;
        goal.iteration_log.push(record);
        Ok(())
    }

    /// Cancel a goal
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        self.update_status(id, GoalStatus::Cancelled)
    }

    /// Get all goals
    pub fn list(&self) -> Vec<Goal> {
        let goals = self.goals.lock().ok();
        match goals {
            Some(g) => g.values().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> GoalGraphSummary {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return GoalGraphSummary::default(),
        };

        let mut summary = GoalGraphSummary::default();
        for goal in goals.values() {
            match &goal.status {
                GoalStatus::Pending => summary.pending += 1,
                GoalStatus::Active => summary.active += 1,
                GoalStatus::Evaluating => summary.evaluating += 1,
                GoalStatus::Converged => summary.converged += 1,
                GoalStatus::Iterating { .. } => summary.iterating += 1,
                GoalStatus::Failed { .. } => summary.failed += 1,
                GoalStatus::BudgetExhausted => summary.budget_exhausted += 1,
                GoalStatus::MaxIterReached => summary.failed += 1,
                GoalStatus::Cancelled => summary.cancelled += 1,
            }
        }
        summary.total = goals.len() as u32;
        summary
    }

    /// Get goals ready to execute (all dependencies converged)
    pub fn get_ready_goals(&self) -> Vec<Goal> {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        goals
            .values()
            .filter(|goal| {
                if goal.status != GoalStatus::Pending {
                    return false;
                }
                goal.depends_on.iter().all(|dep_id| {
                    goals
                        .get(dep_id)
                        .map(|dep| dep.status == GoalStatus::Converged)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .collect()
    }

    /// Assign a worker to a goal
    pub fn assign_worker(&self, id: &str, worker_id: WorkerId) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;
        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;
        goal.executor = Some(worker_id);
        goal.status = GoalStatus::Active;
        goal.started_at = Some(now_ms());
        Ok(())
    }

    /// Execute one reconcile iteration for a goal
    pub fn execute_once(
        &self,
        id: &str,
        reconcile: &crate::reconcile::ReconcileLoop,
    ) -> Result<GoalStatus, String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;
        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;

        if goal.status.is_terminal() {
            return Ok(goal.status.clone());
        }

        reconcile.reconcile_once(goal);
        Ok(goal.status.clone())
    }

    /// Get IDs of goals ready to execute
    pub fn get_ready_ids(&self) -> Vec<String> {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        let converged_ids: Vec<&str> = goals
            .values()
            .filter(|g| g.status.is_success())
            .map(|g| g.id.as_str())
            .collect();

        goals
            .values()
            .filter(|g| g.status == GoalStatus::Pending)
            .filter(|g| {
                g.depends_on
                    .iter()
                    .all(|dep| converged_ids.contains(&dep.as_str()))
            })
            .map(|g| g.id.clone())
            .collect()
    }
}

impl Default for GoalGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tauri Commands
// ===========================================================================

use crate::AppState;
use tauri::State;

/// Submit a new goal
#[tauri::command]
pub fn goal_submit(state: State<'_, AppState>, goal: Goal) -> Result<(), String> {
    state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .submit(goal)
}

/// Get goal status
#[tauri::command]
pub fn goal_get_status(state: State<'_, AppState>, id: String) -> Result<Goal, String> {
    state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
        .ok_or_else(|| format!("Goal '{}' not found", id))
}

/// Cancel a goal
#[tauri::command]
pub fn goal_cancel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .cancel(&id)
}

/// Get graph summary
#[tauri::command]
pub fn goal_get_graph_summary(state: State<'_, AppState>) -> Result<GoalGraphSummary, String> {
    Ok(state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .summary())
}

/// List all goals
#[tauri::command]
pub fn goal_list(state: State<'_, AppState>) -> Result<Vec<Goal>, String> {
    Ok(state.goal_graph.lock().map_err(|e| e.to_string())?.list())
}

/// Get goals ready to execute
#[tauri::command]
pub fn goal_get_ready(state: State<'_, AppState>) -> Result<Vec<Goal>, String> {
    Ok(state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .get_ready_goals())
}

/// Assign a worker to a goal
#[tauri::command]
pub fn goal_assign_worker(
    state: State<'_, AppState>,
    id: String,
    worker_id: String,
) -> Result<(), String> {
    state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .assign_worker(&id, worker_id)
}

/// Add iteration record to a goal
#[tauri::command]
pub fn goal_add_iteration(
    state: State<'_, AppState>,
    id: String,
    record: IterationRecord,
) -> Result<(), String> {
    state
        .goal_graph
        .lock()
        .map_err(|e| e.to_string())?
        .add_iteration(&id, record)
}

/// Execute a single iteration for a goal.
#[tauri::command]
pub fn goal_execute_once(state: State<'_, AppState>, id: String) -> Result<GoalStatus, String> {
    let reconcile = state.reconcile_loop.lock().map_err(|e| e.to_string())?;
    let graph = state.goal_graph.lock().map_err(|e| e.to_string())?;
    graph.execute_once(&id, &reconcile)
}

/// Execute all ready goals in the graph.
#[tauri::command]
pub fn goal_execute_ready(state: State<'_, AppState>) -> Result<Vec<(String, GoalStatus)>, String> {
    let reconcile = state.reconcile_loop.lock().map_err(|e| e.to_string())?;
    let graph = state.goal_graph.lock().map_err(|e| e.to_string())?;

    let ready_ids = graph.get_ready_ids();
    let mut results = Vec::new();

    for id in ready_ids {
        let status = graph.execute_once(&id, &reconcile)?;
        results.push((id, status));
    }

    Ok(results)
}
