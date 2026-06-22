//! Loop Engine — Unified orchestrator for all loop-based systems
//!
//! Integrates four layers:
//! 1. ReconcileLoop — Goal iterative execution
//! 2. Self-Healing Loop — Anomaly detection + auto-repair
//! 3. Evolution Loop — Capability evolution
//! 4. Learning Loop — Knowledge persistence
//!
//! The Loop Engine coordinates these layers, enabling:
//! - Goal failure → Self-Healing trigger
//! - Goal success → Evolution trigger
//! - Pattern extraction → Learning storage

use crate::goal::{Goal, GoalStatus};
use crate::reconcile::ReconcileLoop;
use crate::self_healing::{AnomalyDetector, AnomalyRecord, HealingExecutor, HealingActionRecord};
use crate::swarm_adapters::{SwarmAgentAdapter, now_ms};
use acp_core::{GoalOutcome, CompletionConditionSpec};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Loop Engine state
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LoopState {
    /// Active goals being processed
    pub active_goals: HashMap<String, Goal>,
    /// Detected anomalies (active + history)
    pub anomalies: Vec<AnomalyRecord>,
    /// Healing actions (history)
    pub healing_actions: Vec<HealingActionRecord>,
    /// Evolution events (history)
    pub evolution_events: Vec<EvolutionEvent>,
    /// Loop statistics
    pub stats: LoopStats,
}

/// Evolution event record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvolutionEvent {
    pub id: String,
    pub event_type: EvolutionType,
    pub goal_id: String,
    pub pattern: Option<String>,
    pub skill_id: Option<String>,
    pub fitness_score: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EvolutionType {
    GoalConverged,
    PatternExtracted,
    SkillGenerated,
    SkillEvolved,
}

/// Loop statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LoopStats {
    pub goals_total: u64,
    pub goals_converged: u64,
    pub goals_failed: u64,
    pub anomalies_detected: u64,
    pub healings_triggered: u64,
    pub healings_successful: u64,
    pub evolutions_applied: u64,
    pub patterns_learned: u64,
}

/// The unified Loop Engine orchestrator
pub struct LoopEngine {
    /// Reconcile loop for Goal execution
    reconcile: ReconcileLoop,
    /// Anomaly detector (EWMA baselines)
    anomaly_detector: Arc<Mutex<AnomalyDetector>>,
    /// Healing executor
    healer: Arc<Mutex<HealingExecutor>>,
    /// Loop state
    state: Arc<Mutex<LoopState>>,
    /// Workers registry
    workers: Arc<Mutex<HashMap<String, Arc<dyn SwarmAgentAdapter + Send + Sync>>>>,
}

impl LoopEngine {
    /// Create a new Loop Engine
    pub fn new(
        workers: Arc<Mutex<HashMap<String, Arc<dyn SwarmAgentAdapter + Send + Sync>>>>,
    ) -> Self {
        let anomaly_detector = Arc::new(Mutex::new(AnomalyDetector::new()));
        let healer = Arc::new(Mutex::new(HealingExecutor::new(anomaly_detector.clone())));
        let reconcile = ReconcileLoop::new(workers.clone());
        let state = Arc::new(Mutex::new(LoopState::default()));

        Self {
            reconcile,
            anomaly_detector,
            healer,
            state,
            workers,
        }
    }

    /// Execute a Goal through the full loop pipeline
    ///
    /// 1. ReconcileLoop executes the goal
    /// 2. If failed → Self-Healing triggers
    /// 3. If converged → Evolution triggers
    /// 4. Patterns learned for future
    pub async fn execute_goal(&self, goal: &mut Goal) -> GoalOutcome {
        // Record goal start
        self.record_goal_start(goal);

        // Execute via ReconcileLoop
        let status = self.reconcile.reconcile_until_done_async(goal).await;

        // Determine outcome
        let outcome = match &status {
            GoalStatus::Converged => {
                // Trigger Evolution
                self.trigger_evolution(goal);
                GoalOutcome::Converged {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                }
            }
            GoalStatus::Failed { reason } => {
                // Trigger Self-Healing
                self.trigger_healing(goal, reason);
                GoalOutcome::Failed {
                    reason: reason.clone(),
                    iterations: goal.current_iteration,
                }
            }
            GoalStatus::BudgetExhausted => {
                GoalOutcome::BudgetExhausted {
                    tokens_used: goal.tokens_used,
                }
            }
            GoalStatus::MaxIterReached => {
                // Also trigger Self-Healing for max iter
                self.trigger_healing(goal, "Max iterations reached");
                GoalOutcome::MaxIterReached {
                    iterations: goal.current_iteration,
                }
            }
            _ => GoalOutcome::Failed {
                reason: "Unknown status".into(),
                iterations: goal.current_iteration,
            },
        };

        // Record goal completion
        self.record_goal_completion(goal, &outcome);

        outcome
    }

    /// Trigger Self-Healing when a Goal fails
    fn trigger_healing(&self, goal: &Goal, _reason: &str) {
        // Create anomaly from Goal failure
        let anomaly = AnomalyRecord {
            id: uuid::Uuid::new_v4().to_string(),
            anomaly_type: crate::self_healing::AnomalyType::HighErrorRate,
            severity: crate::self_healing::AnomalySeverity::Medium,
            target: goal.executor.clone().unwrap_or_else(|| "unknown".into()),
            target_type: "goal".into(),
            health_score: 50.0,
            baseline_value: 90.0,
            current_value: 0.0,
            deviation: 100.0,
            detected_at: chrono::Utc::now(),
            resolved_at: None,
        };

        // Store anomaly
        if let Ok(mut state) = self.state.lock() {
            state.anomalies.push(anomaly.clone());
            state.stats.anomalies_detected += 1;
        }

        // Execute healing
        if let Ok(mut healer) = self.healer.lock() {
            if let Ok(action) = healer.execute_healing(&anomaly.id) {
                // Record healing action
                if let Ok(mut state) = self.state.lock() {
                    let is_success = action.status == "success";
                    state.healing_actions.push(action);
                    state.stats.healings_triggered += 1;
                    if is_success {
                        state.stats.healings_successful += 1;
                    }
                }
            }
        }
    }

    /// Trigger Evolution when a Goal converges
    fn trigger_evolution(&self, goal: &Goal) {
        // Extract pattern from successful execution
        let pattern = self.extract_pattern(goal);

        // Create evolution event
        let event = EvolutionEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: EvolutionType::GoalConverged,
            goal_id: goal.id.clone(),
            pattern: Some(pattern),
            skill_id: None,
            fitness_score: self.calculate_fitness(goal),
            timestamp: now_ms(),
        };

        // Store evolution
        if let Ok(mut state) = self.state.lock() {
            state.evolution_events.push(event);
            state.stats.evolutions_applied += 1;
            state.stats.patterns_learned += 1;
        }
    }

    /// Extract pattern from Goal execution log
    fn extract_pattern(&self, goal: &Goal) -> String {
        // Simplified pattern extraction
        // Real implementation would analyze iteration_log for patterns
        format!(
            "goal_type:{}|iterations:{}|executor:{}",
            goal.description.split_whitespace().take(3).collect::<Vec<_>>().join("_"),
            goal.current_iteration,
            goal.executor.as_ref().unwrap_or(&"unknown".into())
        )
    }

    /// Calculate fitness score from Goal execution
    fn calculate_fitness(&self, goal: &Goal) -> f64 {
        // Base score from convergence
        let base = 1.0;

        // Efficiency factor (fewer iterations = higher)
        let efficiency = 1.0 - (goal.current_iteration as f64 / goal.max_iterations as f64) * 0.3;

        // Budget factor (fewer tokens = higher)
        let budget = if let Some(token_budget) = goal.token_budget {
            if token_budget > 0 {
                1.0 - (goal.tokens_used as f64 / token_budget as f64) * 0.2
            } else {
                1.0
            }
        } else {
            1.0
        };

        base * efficiency * budget
    }

    /// Record goal start
    fn record_goal_start(&self, goal: &Goal) {
        if let Ok(mut state) = self.state.lock() {
            state.active_goals.insert(goal.id.clone(), goal.clone());
            state.stats.goals_total += 1;
        }
    }

    /// Record goal completion
    fn record_goal_completion(&self, goal: &Goal, outcome: &GoalOutcome) {
        if let Ok(mut state) = self.state.lock() {
            state.active_goals.remove(&goal.id);

            match outcome {
                GoalOutcome::Converged { .. } => {
                    state.stats.goals_converged += 1;
                }
                GoalOutcome::Failed { .. } => {
                    state.stats.goals_failed += 1;
                }
                _ => {}
            }
        }
    }

    /// Get loop state (for visualization)
    pub fn get_state(&self) -> Result<LoopState, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.clone())
    }

    /// Get loop statistics
    pub fn get_stats(&self) -> Result<LoopStats, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.stats.clone())
    }

    /// Update health metrics (for periodic monitoring)
    pub fn update_health_metrics(&self, metrics: HashMap<String, f64>) {
        if let Ok(mut detector) = self.anomaly_detector.lock() {
            for (name, value) in metrics {
                detector.update_baseline(&name, value);

                // Check for anomalies
                if let Some(anomaly) = detector.check(&name, value) {
                    if let Ok(mut state) = self.state.lock() {
                        state.anomalies.push(anomaly);
                        state.stats.anomalies_detected += 1;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

use tauri::State;
use crate::AppState;

/// Execute a goal through the Loop Engine
///
/// Note: This command spawns a blocking task to handle the MutexGuard
/// across async boundary, as MutexGuard is not Send.
#[tauri::command]
pub async fn loop_execute_goal(
    state: State<'_, AppState>,
    goal_id: String,
    description: String,
    completion_condition: String,
    executor: Option<String>,
) -> Result<serde_json::Value, String> {
    // Clone Arc references for async execution
    let loop_engine_arc = state.loop_engine.clone();

    // Spawn blocking task to execute goal
    let result = tokio::task::spawn_blocking(move || {
        let loop_engine = loop_engine_arc.lock().map_err(|e| e.to_string())?;

        // Create goal with custom completion condition
        let mut goal = crate::goal::Goal::new(
            goal_id,
            description,
            CompletionConditionSpec::Custom { evaluator: completion_condition },
        );
        goal.executor = executor;

        // Execute synchronously (blocking task)
        let status = loop_engine.reconcile.reconcile_until_done(&mut goal);

        // Return simplified JSON
        Ok::<serde_json::Value, String>(serde_json::json!({
            "goal_id": goal.id,
            "status": status,
            "iterations": goal.iteration_log.len(),
            "executor": goal.executor,
        }))
    }).await.map_err(|e| e.to_string())?;

    result
}

/// Get Loop Engine state
#[tauri::command]
pub fn loop_get_state(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let loop_engine = state.loop_engine.lock().map_err(|e| e.to_string())?;
    loop_engine.get_state().and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string()))
}

/// Get Loop Engine statistics
#[tauri::command]
pub fn loop_get_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let loop_engine = state.loop_engine.lock().map_err(|e| e.to_string())?;
    loop_engine.get_stats().and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string()))
}

/// Update health metrics (for monitoring)
#[tauri::command]
pub fn loop_update_metrics(
    state: State<'_, AppState>,
    metrics: HashMap<String, f64>,
) -> Result<(), String> {
    let loop_engine = state.loop_engine.lock().map_err(|e| e.to_string())?;
    loop_engine.update_health_metrics(metrics);
    Ok(())
}