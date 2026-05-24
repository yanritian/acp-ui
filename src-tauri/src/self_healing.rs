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