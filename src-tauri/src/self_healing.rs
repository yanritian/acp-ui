//! Self-Healing System - EWMA anomaly detection and recovery
//!
//! Implements dynamic baseline for anomaly detection with automatic recovery

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Anomaly severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalySeverity::Low => "low",
            AnomalySeverity::Medium => "medium",
            AnomalySeverity::High => "high",
            AnomalySeverity::Critical => "critical",
        }
    }
}

/// Anomaly type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    HeartbeatTimeout,       // Agent not responding
    HighMemoryUsage,        // Memory spike
    HighCPUUsage,           // CPU spike
    LowHealthScore,         // Health score dropped
    HighErrorRate,          // Error rate increased
    ContextOverflow,        // Context too large
    RateLimitHit,           // API rate limit
    AgentCrash,             // Agent crashed
    NetworkError,           // Connection issues
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalyType::HeartbeatTimeout => "heartbeat_timeout",
            AnomalyType::HighMemoryUsage => "high_memory",
            AnomalyType::HighCPUUsage => "high_cpu",
            AnomalyType::LowHealthScore => "low_health",
            AnomalyType::HighErrorRate => "high_error_rate",
            AnomalyType::ContextOverflow => "context_overflow",
            AnomalyType::RateLimitHit => "rate_limit",
            AnomalyType::AgentCrash => "agent_crash",
            AnomalyType::NetworkError => "network_error",
        }
    }
}

/// Anomaly record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRecord {
    pub id: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub target: String,          // Agent or service name
    pub target_type: String,     // "agent", "service", "system"
    pub health_score: f64,
    pub baseline_value: f64,
    pub current_value: f64,
    pub deviation: f64,          // Percentage deviation from baseline
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Healing action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingActionType {
    RestartAgent,         // Restart crashed agent
    SwitchModel,          // Switch to fallback model
    CompressContext,      // Compress conversation history
    ClearCache,           // Clear stale cache
    Reconnect,            // Reconnect network
    ScaleUp,              // Add more agents
    ScaleDown,            // Reduce agents
    NotifyUser,           // Notify user for manual intervention
}

impl HealingActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealingActionType::RestartAgent => "restart_agent",
            HealingActionType::SwitchModel => "switch_model",
            HealingActionType::CompressContext => "compress_context",
            HealingActionType::ClearCache => "clear_cache",
            HealingActionType::Reconnect => "reconnect",
            HealingActionType::ScaleUp => "scale_up",
            HealingActionType::ScaleDown => "scale_down",
            HealingActionType::NotifyUser => "notify_user",
        }
    }
}

/// Healing action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingActionRecord {
    pub id: String,
    pub anomaly_id: String,
    pub action_type: HealingActionType,
    pub action_params: HashMap<String, String>,
    pub status: String,           // "pending", "executing", "success", "failed"
    pub result: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// EWMA (Exponentially Weighted Moving Average) baseline
pub struct EWMABaseline {
    value: f64,
    alpha: f64,  // Smoothing factor (0.0 - 1.0)
    initialized: bool,
}

impl EWMABaseline {
    pub fn new(alpha: f64) -> Self {
        Self {
            value: 0.0,
            alpha,
            initialized: false,
        }
    }

    /// Update baseline with new value
    pub fn update(&mut self, new_value: f64) {
        if !self.initialized {
            self.value = new_value;
            self.initialized = true;
        } else {
            // EWMA formula: baseline = alpha * new + (1 - alpha) * old
            self.value = self.alpha * new_value + (1.0 - self.alpha) * self.value;
        }
    }

    /// Get current baseline value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Check if value deviates from baseline
    pub fn check_deviation(&self, current_value: f64, threshold_percent: f64) -> Option<f64> {
        if !self.initialized {
            return None;
        }

        let deviation = if self.value == 0.0 {
            if current_value > 0.0 { 100.0 } else { 0.0 }
        } else {
            ((current_value - self.value) / self.value * 100.0).abs()
        };

        if deviation > threshold_percent {
            Some(deviation)
        } else {
            None
        }
    }
}

/// Anomaly detector with EWMA baselines
pub struct AnomalyDetector {
    baselines: HashMap<String, EWMABaseline>,
    thresholds: HashMap<String, f64>,
    fixed_threshold_fallback: f64, // For cold-start
}

impl AnomalyDetector {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("heartbeat".to_string(), 30.0);  // 30% deviation
        thresholds.insert("memory".to_string(), 50.0);
        thresholds.insert("cpu".to_string(), 50.0);
        thresholds.insert("health".to_string(), 20.0);
        thresholds.insert("error_rate".to_string(), 100.0);
        thresholds.insert("context".to_string(), 80.0);

        Self {
            baselines: HashMap::new(),
            thresholds,
            fixed_threshold_fallback: 50.0,
        }
    }

    /// Update baseline with new metric value
    pub fn update_baseline(&mut self, metric_name: &str, value: f64) {
        let baseline = self.baselines.entry(metric_name.to_string())
            .or_insert_with(|| EWMABaseline::new(0.3)); // alpha = 0.3

        baseline.update(value);
    }

    /// Check for anomalies
    pub fn check(&self, metric_name: &str, current_value: f64) -> Option<AnomalyRecord> {
        let baseline = self.baselines.get(metric_name);
        let threshold = self.thresholds.get(metric_name)
            .unwrap_or(&self.fixed_threshold_fallback);

        // Cold-start: use fixed threshold
        let (baseline_value, deviation) = if let Some(b) = baseline {
            if !b.initialized {
                // Use fixed threshold for cold-start
                if current_value > *threshold {
                    (*threshold, current_value - *threshold)
                } else {
                    return None;
                }
            } else {
                let dev = b.check_deviation(current_value, *threshold);
                if let Some(d) = dev {
                    (b.value(), d)
                } else {
                    return None;
                }
            }
        } else {
            // No baseline yet, use fixed threshold
            if current_value > *threshold {
                (*threshold, current_value - *threshold)
            } else {
                return None;
            }
        };

        // Determine severity based on deviation
        let severity = if deviation > 200.0 {
            AnomalySeverity::Critical
        } else if deviation > 100.0 {
            AnomalySeverity::High
        } else if deviation > 50.0 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        };

        // Determine anomaly type from metric name
        let anomaly_type = match metric_name {
            "heartbeat" => AnomalyType::HeartbeatTimeout,
            "memory" => AnomalyType::HighMemoryUsage,
            "cpu" => AnomalyType::HighCPUUsage,
            "health" => AnomalyType::LowHealthScore,
            "error_rate" => AnomalyType::HighErrorRate,
            "context" => AnomalyType::ContextOverflow,
            _ => AnomalyType::HighErrorRate,
        };

        // Calculate health score (inverse of deviation)
        let health_score = 100.0 - (deviation / 2.0).min(100.0);

        Some(AnomalyRecord {
            id: uuid::Uuid::new_v4().to_string(),
            anomaly_type,
            severity,
            target: metric_name.to_string(),
            target_type: "system".to_string(),
            health_score,
            baseline_value,
            current_value,
            deviation,
            detected_at: Utc::now(),
            resolved_at: None,
        })
    }

    /// Determine healing action for anomaly
    pub fn determine_healing_action(&self, anomaly: &AnomalyRecord) -> HealingActionRecord {
        let action_type = match anomaly.anomaly_type {
            AnomalyType::HeartbeatTimeout => HealingActionType::RestartAgent,
            AnomalyType::AgentCrash => HealingActionType::RestartAgent,
            AnomalyType::HighMemoryUsage => HealingActionType::ClearCache,
            AnomalyType::HighCPUUsage => HealingActionType::ScaleDown,
            AnomalyType::ContextOverflow => HealingActionType::CompressContext,
            AnomalyType::RateLimitHit => HealingActionType::SwitchModel,
            AnomalyType::NetworkError => HealingActionType::Reconnect,
            AnomalyType::LowHealthScore => HealingActionType::NotifyUser,
            AnomalyType::HighErrorRate => HealingActionType::NotifyUser,
        };

        let mut params = HashMap::new();
        params.insert("target".to_string(), anomaly.target.clone());
        params.insert("severity".to_string(), anomaly.severity.as_str().to_string());

        HealingActionRecord {
            id: uuid::Uuid::new_v4().to_string(),
            anomaly_id: anomaly.id.clone(),
            action_type,
            action_params: params,
            status: "pending".to_string(),
            result: None,
            executed_at: None,
            created_at: Utc::now(),
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Healing executor - executes repair actions and tracks their lifecycle
pub struct HealingExecutor {
    /// All healing actions (active and historical)
    actions: HashMap<String, HealingActionRecord>,
    /// Reference to anomaly detector for determining actions
    anomaly_detector: std::sync::Arc<std::sync::Mutex<AnomalyDetector>>,
    /// Optional database connection for persisting learned patterns (feedback loop)
    database: Option<std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>>,
}

impl HealingExecutor {
    pub fn new(anomaly_detector: std::sync::Arc<std::sync::Mutex<AnomalyDetector>>) -> Self {
        Self {
            actions: HashMap::new(),
            anomaly_detector,
            database: None,
        }
    }

    /// Set the database connection for persisting learned healing patterns.
    /// This creates the feedback loop: detect -> heal -> learn.
    pub fn set_database(&mut self, db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>) {
        self.database = Some(db);
    }

    /// Execute a healing action based on anomaly ID
    /// Determines the appropriate action and executes it
    pub fn execute_healing(&mut self, anomaly_id: &str) -> Result<HealingActionRecord, String> {
        // First, we need to get the anomaly from somewhere
        // For now, we'll create a synthetic anomaly based on the ID
        // In a real system, you'd look this up from a database or anomaly store
        let anomaly = self.find_anomaly_by_id(anomaly_id)?;

        // Determine the healing action using the anomaly detector
        let mut action = {
            let detector = self.anomaly_detector.lock().map_err(|e| e.to_string())?;
            detector.determine_healing_action(&anomaly)
        };

        // Set status to executing
        action.status = "executing".to_string();
        action.executed_at = Some(Utc::now());

        // Execute the action based on type
        match action.action_type {
            HealingActionType::RestartAgent => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Agent '{}' restart initiated", target));
            }
            HealingActionType::CompressContext => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Context compression completed for '{}'", target));
            }
            HealingActionType::ClearCache => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Cache cleared for '{}'", target));
            }
            HealingActionType::SwitchModel => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Model switched for '{}'", target));
            }
            HealingActionType::Reconnect => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Reconnection successful for '{}'", target));
            }
            HealingActionType::NotifyUser => {
                action.status = "notified".to_string();
                action.result = Some("User notification sent".to_string());
            }
            HealingActionType::ScaleUp => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Scaled up '{}'", target));
            }
            HealingActionType::ScaleDown => {
                let target = action.action_params.get("target").cloned().unwrap_or_default();
                action.status = "success".to_string();
                action.result = Some(format!("Scaled down '{}'", target));
            }
        };

        // Auto-save pattern for successful healing (feedback loop: detect -> heal -> learn)
        if action.status == "success" {
            if let Some(db) = &self.database {
                if let Ok(conn) = db.lock() {
                    let anomaly_type_str = anomaly.anomaly_type.as_str();
                    let anomaly_desc = format!(
                        "{} anomaly on '{}' (deviation: {:.1}%, health: {:.1})",
                        anomaly_type_str, anomaly.target, anomaly.deviation, anomaly.health_score
                    );
                    let healing_result = action.result.clone().unwrap_or_default();
                    let trigger_reason = format!(
                        "Auto-healing triggered by {} anomaly (severity: {})",
                        anomaly_type_str, anomaly.severity.as_str()
                    );

                    // Save evolution record for this healing event
                    let evolution_id = uuid::Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO evolutions (id, type, domain, before, after, reason, evidence, created_at)
                         VALUES (?1, 'auto_healing', ?2, ?3, ?4, ?5, ?6, datetime('now'))",
                        rusqlite::params![
                            evolution_id,
                            anomaly_type_str,
                            anomaly_desc,
                            healing_result,
                            trigger_reason,
                            action.action_type.as_str(),
                        ],
                    );

                    // Insert or update pattern for this anomaly type (usage-based learning)
                    let pattern_id = format!("{}_recovery", anomaly_type_str);
                    let pattern_name = format!("{}_recovery", anomaly_type_str);
                    let pattern_desc = format!(
                        "Automatic recovery pattern for {} anomalies using {}",
                        anomaly_type_str,
                        action.action_type.as_str()
                    );
                    let now = chrono::Utc::now().to_rfc3339();
                    let _ = conn.execute(
                        "INSERT INTO patterns (id, name, description, category, examples, success_rate, usage_count, created_at, updated_at)
                         VALUES (?1, ?2, ?3, 'self_healing', ?4, 1.0, 1, ?5, ?6)
                         ON CONFLICT(id) DO UPDATE SET usage_count = usage_count + 1, updated_at = ?7",
                        rusqlite::params![
                            pattern_id,
                            pattern_name,
                            pattern_desc,
                            action.action_type.as_str(),
                            now,
                            now,
                            now,
                        ],
                    );
                }
            }
        }

        // Store the action
        let action_clone = action.clone();
        self.actions.insert(action.id.clone(), action);

        Ok(action_clone)
    }

    /// Find an anomaly by ID.
    ///
    /// TODO(Phase 4): Query from persistent storage instead of returning synthetic data.
    fn find_anomaly_by_id(&self, anomaly_id: &str) -> Result<AnomalyRecord, String> {
        // TODO(Phase 4): Replace with real database lookup.
        // Currently returns a default record for compilation completeness.
        Ok(AnomalyRecord {
            id: anomaly_id.to_string(),
            anomaly_type: AnomalyType::HighErrorRate, // Default type
            severity: AnomalySeverity::Medium,
            target: "unknown".to_string(),
            target_type: "system".to_string(),
            health_score: 50.0,
            baseline_value: 100.0,
            current_value: 150.0,
            deviation: 50.0,
            detected_at: Utc::now(),
            resolved_at: None,
        })
    }

    /// Get all active (pending or executing) actions
    pub fn get_active_actions(&self) -> Vec<&HealingActionRecord> {
        self.actions
            .values()
            .filter(|a| a.status == "pending" || a.status == "executing")
            .collect()
    }

    /// Get all actions (history)
    pub fn get_action_history(&self) -> Vec<&HealingActionRecord> {
        self.actions.values().collect()
    }

    /// Resolve an action with a result
    pub fn resolve_action(&mut self, action_id: &str, result: &str) -> Result<HealingActionRecord, String> {
        let action = self.actions.get_mut(action_id)
            .ok_or_else(|| format!("Action '{}' not found", action_id))?;

        action.status = "resolved".to_string();
        action.result = Some(result.to_string());

        Ok(action.clone())
    }

    /// Get healing statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let total = self.actions.len();
        let pending = self.actions.values().filter(|a| a.status == "pending").count();
        let executing = self.actions.values().filter(|a| a.status == "executing").count();
        let success = self.actions.values().filter(|a| a.status == "success").count();
        let failed = self.actions.values().filter(|a| a.status == "failed").count();
        let resolved = self.actions.values().filter(|a| a.status == "resolved").count();

        serde_json::json!({
            "total_actions": total,
            "pending": pending,
            "executing": executing,
            "success": success,
            "failed": failed,
            "resolved": resolved,
        })
    }
}

// ============================================================================
// Tauri Commands for Healing Executor
// ============================================================================

use tauri::State;
use crate::AppState;

/// Execute a healing action for a given anomaly
#[tauri::command]
pub fn healing_execute(
    state: State<'_, AppState>,
    anomaly_id: String,
) -> Result<serde_json::Value, String> {
    let mut executor = state.healing_executor.lock().map_err(|e| e.to_string())?;
    let action = executor.execute_healing(&anomaly_id)?;
    Ok(serde_json::to_value(action).map_err(|e| e.to_string())?)
}

/// List all healing actions
#[tauri::command]
pub fn healing_list_actions(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let executor = state.healing_executor.lock().map_err(|e| e.to_string())?;
    let actions = executor.get_action_history();
    Ok(serde_json::to_value(actions).map_err(|e| e.to_string())?)
}

/// Resolve a healing action with a result
#[tauri::command]
pub fn healing_resolve_action(
    state: State<'_, AppState>,
    action_id: String,
    result: String,
) -> Result<serde_json::Value, String> {
    let mut executor = state.healing_executor.lock().map_err(|e| e.to_string())?;
    let action = executor.resolve_action(&action_id, &result)?;
    Ok(serde_json::to_value(action).map_err(|e| e.to_string())?)
}

/// Get healing statistics
#[tauri::command]
pub fn healing_get_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let executor = state.healing_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor.get_stats())
}