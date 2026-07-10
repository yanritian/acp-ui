// Cost Tracker - Real-time cost tracking and budget management
//
// Phase 4 Week 4: Cost Tracker + Token 追踪
// Unified cost aggregation, budget limits, cost alerts

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost Tracker for managing API costs across adapters
pub struct CostTracker {
    /// Current usage state
    usage: CostUsage,
    /// Budget limits
    budget: CostBudget,
    /// Cost history per adapter
    history: HashMap<String, Vec<CostRecord>>,
    /// Alert thresholds (percentage of budget)
    alert_thresholds: Vec<u8>,
    /// Whether budget exceeded
    budget_exceeded: bool,
}

/// Current cost usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostUsage {
    /// Total cost today
    pub daily_total: f32,
    /// Total cost this month
    pub monthly_total: f32,
    /// Total cost all time
    pub total_all_time: f32,
    /// Per-adapter costs
    pub adapter_totals: HashMap<String, f32>,
    /// Per-scene costs
    pub scene_totals: HashMap<String, f32>,
    /// Token usage summary
    pub token_totals: TokenTotals,
}

/// Token totals across all executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl Default for TokenTotals {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        }
    }
}

/// Budget limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBudget {
    /// Daily budget limit (USD)
    pub daily_limit: f32,
    /// Monthly budget limit (USD)
    pub monthly_limit: f32,
    /// Currency
    pub currency: String,
    /// Enable hard stop (reject tasks when budget exceeded)
    pub hard_stop: bool,
    /// Enable alerts
    pub enable_alerts: bool,
}

impl Default for CostBudget {
    fn default() -> Self {
        Self {
            daily_limit: 10.0,    // $10/day default
            monthly_limit: 100.0, // $100/month default
            currency: "USD".to_string(),
            hard_stop: false,
            enable_alerts: true,
        }
    }
}

/// Cost record for a single execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// Task ID
    pub task_id: String,
    /// Adapter ID
    pub adapter_id: String,
    /// Cost amount
    pub amount: f32,
    /// Currency
    pub currency: String,
    /// Token usage
    pub tokens: TokenUsageRecord,
    /// Scene type
    pub scene: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Token usage for a single execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageRecord {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

/// Cost forecast prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostForecast {
    /// Predicted daily cost
    pub predicted_daily: f32,
    /// Predicted monthly cost
    pub predicted_monthly: f32,
    /// Days remaining in month
    pub days_remaining: u32,
    /// Budget remaining percentage
    pub budget_remaining_percent: f32,
    /// Whether will exceed budget
    pub will_exceed: bool,
}

/// Cost alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAlert {
    /// Alert level
    pub level: AlertLevel,
    /// Message
    pub message: String,
    /// Current usage
    pub current_usage: f32,
    /// Threshold
    pub threshold: f32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Warning,  // 80% of budget
    Critical, // 95% of budget
    Exceeded, // 100% of budget
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            usage: CostUsage {
                daily_total: 0.0,
                monthly_total: 0.0,
                total_all_time: 0.0,
                adapter_totals: HashMap::new(),
                scene_totals: HashMap::new(),
                token_totals: TokenTotals::default(),
            },
            budget: CostBudget::default(),
            history: HashMap::new(),
            alert_thresholds: vec![80, 95, 100],
            budget_exceeded: false,
        }
    }

    pub fn with_budget(budget: CostBudget) -> Self {
        Self {
            usage: CostUsage {
                daily_total: 0.0,
                monthly_total: 0.0,
                total_all_time: 0.0,
                adapter_totals: HashMap::new(),
                scene_totals: HashMap::new(),
                token_totals: TokenTotals::default(),
            },
            budget,
            history: HashMap::new(),
            alert_thresholds: vec![80, 95, 100],
            budget_exceeded: false,
        }
    }

    /// Record a cost
    pub fn record_cost(&mut self, record: CostRecord) -> Option<CostAlert> {
        // Update totals
        self.usage.daily_total += record.amount;
        self.usage.monthly_total += record.amount;
        self.usage.total_all_time += record.amount;

        // Update adapter totals
        self.usage
            .adapter_totals
            .entry(record.adapter_id.clone())
            .and_modify(|v| *v += record.amount)
            .or_insert(record.amount);

        // Update scene totals
        if let Some(scene) = &record.scene {
            self.usage
                .scene_totals
                .entry(scene.clone())
                .and_modify(|v| *v += record.amount)
                .or_insert(record.amount);
        }

        // Update token totals
        self.usage.token_totals.input_tokens += record.tokens.input_tokens as u64;
        self.usage.token_totals.output_tokens += record.tokens.output_tokens as u64;
        self.usage.token_totals.total_tokens += record.tokens.total_tokens as u64;

        // Store in history
        self.history
            .entry(record.adapter_id.clone())
            .or_insert_with(Vec::new)
            .push(record);

        // Check for alerts
        self.check_alerts()
    }

    /// Check if budget allows execution
    pub fn is_budget_allowed(&self) -> bool {
        if self.budget.hard_stop && self.budget_exceeded {
            return false;
        }

        // Check daily limit
        if self.usage.daily_total >= self.budget.daily_limit {
            return !self.budget.hard_stop;
        }

        // Check monthly limit
        if self.usage.monthly_total >= self.budget.monthly_limit {
            return !self.budget.hard_stop;
        }

        true
    }

    /// Check and generate alerts
    fn check_alerts(&mut self) -> Option<CostAlert> {
        if !self.budget.enable_alerts {
            return None;
        }

        let daily_percent = (self.usage.daily_total / self.budget.daily_limit) * 100.0;
        let monthly_percent = (self.usage.monthly_total / self.budget.monthly_limit) * 100.0;

        // Check if exceeded
        if daily_percent >= 100.0 || monthly_percent >= 100.0 {
            self.budget_exceeded = true;
            return Some(CostAlert {
                level: AlertLevel::Exceeded,
                message: "Budget exceeded! Operations may be restricted.".to_string(),
                current_usage: self.usage.monthly_total,
                threshold: self.budget.monthly_limit,
                timestamp: Utc::now(),
            });
        }

        // Check critical (95%)
        if monthly_percent >= 95.0 {
            return Some(CostAlert {
                level: AlertLevel::Critical,
                message: "Budget nearly exceeded (95%)!".to_string(),
                current_usage: self.usage.monthly_total,
                threshold: self.budget.monthly_limit,
                timestamp: Utc::now(),
            });
        }

        // Check warning (80%)
        if monthly_percent >= 80.0 {
            return Some(CostAlert {
                level: AlertLevel::Warning,
                message: "Budget warning (80%)!".to_string(),
                current_usage: self.usage.monthly_total,
                threshold: self.budget.monthly_limit,
                timestamp: Utc::now(),
            });
        }

        None
    }

    /// Get cost forecast
    pub fn get_forecast(&self) -> CostForecast {
        // Calculate days remaining in month
        let now = Utc::now();
        let days_in_month = 30; // Simplified
        let day_of_month = now.day();
        let days_remaining = days_in_month - day_of_month;

        // Average daily cost so far
        let avg_daily = if day_of_month > 0 {
            self.usage.monthly_total / day_of_month as f32
        } else {
            0.0
        };

        // Predict monthly total
        let predicted_monthly = self.usage.monthly_total + (avg_daily * days_remaining as f32);

        // Budget remaining percentage
        let budget_remaining = ((self.budget.monthly_limit - self.usage.monthly_total)
            / self.budget.monthly_limit)
            * 100.0;

        CostForecast {
            predicted_daily: avg_daily,
            predicted_monthly,
            days_remaining,
            budget_remaining_percent: budget_remaining.max(0.0),
            will_exceed: predicted_monthly > self.budget.monthly_limit,
        }
    }

    /// Get usage summary
    pub fn get_usage(&self) -> &CostUsage {
        &self.usage
    }

    /// Get budget config
    pub fn get_budget(&self) -> &CostBudget {
        &self.budget
    }

    /// Set budget limits
    pub fn set_budget(&mut self, budget: CostBudget) {
        self.budget_exceeded = self.usage.monthly_total >= budget.monthly_limit;
        self.budget = budget;
    }

    /// Get cost history for adapter
    pub fn get_adapter_history(&self, adapter_id: &str, limit: usize) -> Vec<&CostRecord> {
        self.history
            .get(adapter_id)
            .map(|records| records.iter().rev().take(limit).collect())
            .unwrap_or_default()
    }

    /// Get total records count
    pub fn get_total_records(&self) -> usize {
        self.history.values().map(|v| v.len()).sum()
    }

    /// Reset daily usage (called at midnight)
    pub fn reset_daily(&mut self) {
        self.usage.daily_total = 0.0;
        self.budget_exceeded = self.usage.monthly_total >= self.budget.monthly_limit;
    }

    /// Reset monthly usage (called at month start)
    pub fn reset_monthly(&mut self) {
        self.usage.monthly_total = 0.0;
        self.usage.daily_total = 0.0;
        self.budget_exceeded = false;
    }

    /// Format cost as string
    pub fn format_cost(amount: f32, currency: &str) -> String {
        if amount < 0.01 {
            format!("{} {:.4}", currency, amount)
        } else if amount < 1.0 {
            format!("{} {:.2}", currency, amount)
        } else {
            format!("{} {:.2}", currency, amount)
        }
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker_new() {
        let tracker = CostTracker::new();
        assert_eq!(tracker.usage.daily_total, 0.0);
        assert_eq!(tracker.budget.daily_limit, 10.0);
        assert!(!tracker.budget_exceeded);
    }

    #[test]
    fn test_cost_budget_default() {
        let budget = CostBudget::default();
        assert_eq!(budget.daily_limit, 10.0);
        assert_eq!(budget.monthly_limit, 100.0);
        assert_eq!(budget.currency, "USD");
    }

    #[test]
    fn test_record_cost() {
        let mut tracker = CostTracker::new();
        let record = CostRecord {
            task_id: "test-1".to_string(),
            adapter_id: "kimi".to_string(),
            amount: 0.5,
            currency: "USD".to_string(),
            tokens: TokenUsageRecord {
                input_tokens: 1000,
                output_tokens: 500,
                total_tokens: 1500,
            },
            scene: Some("marketing".to_string()),
            timestamp: Utc::now(),
        };

        tracker.record_cost(record);

        assert_eq!(tracker.usage.daily_total, 0.5);
        assert_eq!(tracker.usage.adapter_totals.get("kimi"), Some(&0.5));
        assert_eq!(tracker.usage.token_totals.input_tokens, 1000);
    }

    #[test]
    fn test_is_budget_allowed() {
        let mut tracker = CostTracker::new();
        assert!(tracker.is_budget_allowed());

        // Exceed daily limit
        tracker.usage.daily_total = 15.0;
        tracker.budget_exceeded = true;
        tracker.budget.hard_stop = true;

        assert!(!tracker.is_budget_allowed());
    }

    #[test]
    fn test_budget_alert_warning() {
        let mut tracker = CostTracker::with_budget(CostBudget {
            monthly_limit: 100.0,
            enable_alerts: true,
            ..Default::default()
        });

        tracker.usage.monthly_total = 80.0; // 80% of budget

        let record = CostRecord {
            task_id: "test".to_string(),
            adapter_id: "test".to_string(),
            amount: 1.0,
            currency: "USD".to_string(),
            tokens: TokenUsageRecord::default(),
            scene: None,
            timestamp: Utc::now(),
        };

        let alert = tracker.record_cost(record);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().level, AlertLevel::Warning);
    }

    #[test]
    fn test_budget_alert_exceeded() {
        let mut tracker = CostTracker::with_budget(CostBudget {
            monthly_limit: 10.0,
            enable_alerts: true,
            ..Default::default()
        });

        tracker.usage.monthly_total = 9.0;

        let record = CostRecord {
            task_id: "test".to_string(),
            adapter_id: "test".to_string(),
            amount: 2.0, // Will exceed 10.0 limit
            currency: "USD".to_string(),
            tokens: TokenUsageRecord::default(),
            scene: None,
            timestamp: Utc::now(),
        };

        let alert = tracker.record_cost(record);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().level, AlertLevel::Exceeded);
        assert!(tracker.budget_exceeded);
    }

    #[test]
    fn test_get_forecast() {
        let tracker = CostTracker::new();
        let forecast = tracker.get_forecast();

        assert!(forecast.days_remaining > 0);
        assert!(!forecast.will_exceed);
    }

    #[test]
    fn test_format_cost() {
        assert_eq!(CostTracker::format_cost(0.005, "USD"), "USD 0.0050");
        assert_eq!(CostTracker::format_cost(0.5, "USD"), "USD 0.50");
        assert_eq!(CostTracker::format_cost(10.0, "USD"), "USD 10.00");
    }

    #[test]
    fn test_reset_daily() {
        let mut tracker = CostTracker::new();
        tracker.usage.daily_total = 5.0;

        tracker.reset_daily();
        assert_eq!(tracker.usage.daily_total, 0.0);
    }

    #[test]
    fn test_set_budget() {
        let mut tracker = CostTracker::new();

        let new_budget = CostBudget {
            daily_limit: 20.0,
            monthly_limit: 200.0,
            currency: "USD".to_string(),
            hard_stop: true,
            enable_alerts: true,
        };

        tracker.set_budget(new_budget);
        assert_eq!(tracker.budget.daily_limit, 20.0);
        assert_eq!(tracker.budget.monthly_limit, 200.0);
    }
}
