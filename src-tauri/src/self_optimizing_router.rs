// Self-Optimizing Router - Historical data-based intelligent routing
//
// Extends Smart Router with:
// - Historical execution data analysis
// - Agent proficiency scoring per scene/platform
// - Cost-performance trade-off optimization
// - Privacy-aware routing decisions

use crate::agent_adapter::types::{ExecutionRecord, Platform, SceneType};
use crate::smart_router::{InputType, RouteDecision, RouteTarget, TaskAnalyzer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Self-Optimizing Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfOptimizingConfig {
    /// Minimum history entries before optimization kicks in
    pub min_history_size: u32,
    /// EWMA smoothing factor for proficiency updates
    pub ewma_alpha: f32,
    /// Cost weight in decision (0.0-1.0)
    pub cost_weight: f32,
    /// Performance weight in decision (0.0-1.0)
    pub performance_weight: f32,
    /// Privacy weight - higher = prefer local agents
    pub privacy_weight: f32,
}

impl Default for SelfOptimizingConfig {
    fn default() -> Self {
        Self {
            min_history_size: 10,
            ewma_alpha: 0.15,
            cost_weight: 0.3,
            performance_weight: 0.4,
            privacy_weight: 0.3,
        }
    }
}

/// Agent proficiency score per scene/platform combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencyScore {
    pub agent_id: String,
    pub scene: SceneType,
    pub platform: Platform,
    /// EWMA proficiency (0.0-1.0)
    pub proficiency: f32,
    /// Success count
    pub success_count: u32,
    /// Failure count
    pub failure_count: u32,
    /// Average latency (ms)
    pub avg_latency_ms: u64,
    /// Average cost (USD)
    pub avg_cost: f32,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl ProficiencyScore {
    pub fn new(agent_id: String, scene: SceneType, platform: Platform) -> Self {
        Self {
            agent_id,
            scene,
            platform,
            proficiency: 0.5, // Start neutral
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0,
            avg_cost: 0.0,
            updated_at: Utc::now(),
        }
    }

    /// Update proficiency with new execution record
    pub fn update(&mut self, success: bool, latency_ms: u64, cost: f32, alpha: f32) {
        let new_value = if success {
            // Success: proficiency increases based on latency
            let latency_factor = 1.0 - (latency_ms as f32 / 30000.0).min(0.5); // Max 30s penalty
            0.8 + latency_factor * 0.2
        } else {
            // Failure: proficiency decreases
            0.2
        };

        // EWMA update
        self.proficiency = alpha * new_value + (1.0 - alpha) * self.proficiency;

        // Update counters
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }

        // Update averages (simple moving average for now)
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.avg_latency_ms =
                (self.avg_latency_ms * (total - 1) as u64 + latency_ms) / total as u64;
            self.avg_cost = (self.avg_cost * (total - 1) as f32 + cost) / total as f32;
        }

        self.updated_at = Utc::now();
    }

    /// Calculate composite score for routing decision
    pub fn composite_score(&self, config: &SelfOptimizingConfig) -> f32 {
        let success_rate = if self.success_count + self.failure_count == 0 {
            0.5
        } else {
            self.success_count as f32 / (self.success_count + self.failure_count) as f32
        };

        // Cost score: lower cost = higher score
        let cost_score = 1.0 - (self.avg_cost / 1.0).min(0.9); // Max $1 penalty

        // Latency score: lower latency = higher score
        let latency_score = 1.0 - (self.avg_latency_ms as f32 / 10000.0).min(0.5); // Max 10s penalty

        // Composite score
        config.performance_weight
            * (success_rate * 0.5 + self.proficiency * 0.3 + latency_score * 0.2)
            + config.cost_weight * cost_score
            + config.privacy_weight * 0.5 // Local agents get privacy bonus
    }
}

/// Self-Optimizing Router state
pub struct SelfOptimizingRouter {
    config: SelfOptimizingConfig,
    analyzer: TaskAnalyzer,
    /// Proficiency scores per (agent_id, scene, platform)
    proficiencies: HashMap<String, ProficiencyScore>,
    /// Execution history (last N records)
    history: Vec<ExecutionRecord>,
    /// Max history size
    max_history: usize,
}

impl SelfOptimizingRouter {
    pub fn new() -> Self {
        Self {
            config: SelfOptimizingConfig::default(),
            analyzer: TaskAnalyzer::new(),
            proficiencies: HashMap::new(),
            history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn with_config(config: SelfOptimizingConfig) -> Self {
        Self {
            config,
            analyzer: TaskAnalyzer::new(),
            proficiencies: HashMap::new(),
            history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Record execution for learning
    pub fn record_execution(&mut self, record: ExecutionRecord) {
        // Update proficiency
        let key = format!(
            "{}:{}:{}",
            record.agent_id,
            scene_to_str(&record.scene_type),
            platform_to_str(&record.platform)
        );
        let proficiency = self.proficiencies.entry(key).or_insert_with(|| {
            ProficiencyScore::new(
                record.agent_id.clone(),
                record.scene_type.clone(),
                record.platform.clone(),
            )
        });

        proficiency.update(
            record.success,
            record.duration_ms,
            record.cost,
            self.config.ewma_alpha,
        );

        // Add to history
        self.history.push(record);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Route task with self-optimization
    pub fn route(
        &self,
        input: &str,
        input_type: InputType,
        scene: Option<SceneType>,
        platform: Option<Platform>,
        available_agents: &[String],
    ) -> SelfOptimizingDecision {
        // First, use smart router for base analysis
        let base_decision = self.analyzer.analyze(input, input_type.clone());

        // Get default agent before moving base_decision
        let default_agent = self.default_agent(&base_decision.route_target, available_agents);

        // Check if we have enough history for optimization
        if self.history.len() < self.config.min_history_size as usize {
            // Fall back to heuristic routing
            return SelfOptimizingDecision {
                base_decision,
                selected_agent: default_agent,
                optimization_applied: false,
                proficiency_scores: vec![],
                reason: "Insufficient history for optimization".to_string(),
            };
        }

        // Get scene and platform (use defaults if not specified)
        let scene = scene.unwrap_or(SceneType::WebDevelopment);
        let platform = platform.unwrap_or(Platform::Web);

        // Find proficiency scores for available agents
        let scores: Vec<ProficiencyScore> = available_agents
            .iter()
            .filter_map(|agent_id| {
                let key = format!(
                    "{}:{}:{}",
                    agent_id,
                    scene_to_str(&scene),
                    platform_to_str(&platform)
                );
                self.proficiencies.get(&key).cloned()
            })
            .collect();

        // If no scores, use default agent
        if scores.is_empty() {
            return SelfOptimizingDecision {
                base_decision,
                selected_agent: default_agent,
                optimization_applied: false,
                proficiency_scores: vec![],
                reason: "No proficiency data for available agents".to_string(),
            };
        }

        // Select best agent based on composite score
        let best = scores.iter().max_by(|a, b| {
            let score_a = a.composite_score(&self.config);
            let score_b = b.composite_score(&self.config);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let (selected_agent, best_score) = best
            .map(|p| (p.agent_id.clone(), p.composite_score(&self.config)))
            .unwrap_or_else(|| (default_agent, 0.0));

        SelfOptimizingDecision {
            base_decision,
            selected_agent: selected_agent.clone(),
            optimization_applied: true,
            proficiency_scores: scores,
            reason: format!(
                "Selected {} with composite score {:.2}",
                selected_agent, best_score
            ),
        }
    }

    /// Get default agent for route target
    fn default_agent(&self, target: &RouteTarget, available: &[String]) -> String {
        match target {
            RouteTarget::ClaudeCodeHaiku | RouteTarget::ClaudeCodeSonnet => available
                .iter()
                .find(|a| a.contains("claude"))
                .cloned()
                .unwrap_or_else(|| "claude-code".to_string()),
            RouteTarget::CodexHaiku | RouteTarget::CodexSonnet => available
                .iter()
                .find(|a| a.contains("codex"))
                .cloned()
                .unwrap_or_else(|| "codex".to_string()),
            RouteTarget::Team => {
                // For team, select the best leader agent
                available
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "claude-code".to_string())
            }
            RouteTarget::HumanReview => "human-review".to_string(),
        }
    }

    /// Get proficiency stats for an agent
    pub fn get_agent_stats(&self, agent_id: &str) -> Vec<ProficiencyScore> {
        self.proficiencies
            .values()
            .filter(|p| p.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get all proficiency scores
    pub fn get_all_proficiencies(&self) -> Vec<ProficiencyScore> {
        self.proficiencies.values().cloned().collect()
    }

    /// Get history summary
    pub fn get_history_summary(&self) -> HistorySummary {
        let total = self.history.len();
        let successes = self.history.iter().filter(|r| r.success).count();
        let avg_latency = if total > 0 {
            self.history.iter().map(|r| r.duration_ms).sum::<u64>() / total as u64
        } else {
            0
        };
        let total_cost = self.history.iter().map(|r| r.cost).sum();

        HistorySummary {
            total_executions: total,
            success_count: successes,
            failure_count: total - successes,
            success_rate: if total > 0 {
                successes as f32 / total as f32
            } else {
                0.0
            },
            avg_latency_ms: avg_latency,
            total_cost,
        }
    }

    /// Reset proficiency scores (for testing or retraining)
    pub fn reset_proficiencies(&mut self) {
        self.proficiencies.clear();
    }
}

impl Default for SelfOptimizingRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Self-optimizing decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfOptimizingDecision {
    /// Base routing decision from Smart Router
    pub base_decision: RouteDecision,
    /// Selected agent ID
    pub selected_agent: String,
    /// Whether optimization was applied
    pub optimization_applied: bool,
    /// Proficiency scores considered
    pub proficiency_scores: Vec<ProficiencyScore>,
    /// Reason for selection
    pub reason: String,
}

/// History summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySummary {
    pub total_executions: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub success_rate: f32,
    pub avg_latency_ms: u64,
    pub total_cost: f32,
}

// Helper functions

fn scene_to_str(scene: &SceneType) -> &'static str {
    match scene {
        SceneType::WebDevelopment => "web",
        SceneType::MiniProgramDevelopment => "mini",
        SceneType::DesktopDevelopment => "desktop",
        SceneType::GameDevelopment => "game",
        SceneType::GameArtGeneration => "game_art",
        SceneType::GameCrossPlatform => "game_cross",
        SceneType::GamePerformanceOptimization => "game_perf",
        SceneType::Marketing => "marketing",
        SceneType::Finance => "finance",
        SceneType::Design => "design",
    }
}

fn platform_to_str(platform: &Platform) -> &'static str {
    match platform {
        Platform::Web => "web",
        Platform::MiniProgram => "mini",
        Platform::Desktop => "desktop",
        Platform::Mobile => "mobile",
        Platform::Game => "game",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_optimizing_router_new() {
        let router = SelfOptimizingRouter::new();
        assert_eq!(router.history.len(), 0);
        assert_eq!(router.proficiencies.len(), 0);
    }

    #[test]
    fn test_proficiency_score_new() {
        let score = ProficiencyScore::new(
            "claude-code".to_string(),
            SceneType::WebDevelopment,
            Platform::Web,
        );
        assert_eq!(score.agent_id, "claude-code");
        assert_eq!(score.proficiency, 0.5);
        assert_eq!(score.success_count, 0);
    }

    #[test]
    fn test_proficiency_update_success() {
        let mut score = ProficiencyScore::new(
            "claude-code".to_string(),
            SceneType::WebDevelopment,
            Platform::Web,
        );

        score.update(true, 5000, 0.01, 0.15);

        assert_eq!(score.success_count, 1);
        assert!(score.proficiency > 0.5);
        assert_eq!(score.avg_latency_ms, 5000);
        assert_eq!(score.avg_cost, 0.01);
    }

    #[test]
    fn test_proficiency_update_failure() {
        let mut score = ProficiencyScore::new(
            "claude-code".to_string(),
            SceneType::WebDevelopment,
            Platform::Web,
        );

        score.update(false, 10000, 0.0, 0.15);

        assert_eq!(score.failure_count, 1);
        assert!(score.proficiency < 0.5);
    }

    #[test]
    fn test_composite_score() {
        let config = SelfOptimizingConfig::default();
        let mut score = ProficiencyScore::new(
            "claude-code".to_string(),
            SceneType::WebDevelopment,
            Platform::Web,
        );

        // After some successful executions
        score.update(true, 3000, 0.01, 0.15);
        score.update(true, 2500, 0.01, 0.15);

        let composite = score.composite_score(&config);
        assert!(composite > 0.0);
        assert!(composite < 1.0);
    }

    #[test]
    fn test_record_execution() {
        let mut router = SelfOptimizingRouter::new();

        let record = ExecutionRecord {
            task_id: "task-1".to_string(),
            scene_type: SceneType::WebDevelopment,
            platform: Platform::Web,
            agent_id: "claude-code".to_string(),
            success: true,
            duration_ms: 5000,
            cost: 0.01,
            timestamp: 0,
        };

        router.record_execution(record);

        assert_eq!(router.history.len(), 1);
        assert_eq!(router.proficiencies.len(), 1);

        let stats = router.get_agent_stats("claude-code");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].success_count, 1);
    }

    #[test]
    fn test_route_without_history() {
        let router = SelfOptimizingRouter::new();

        let decision = router.route(
            "Fix a bug",
            InputType::Text,
            None,
            None,
            &["claude-code".to_string()],
        );

        assert!(!decision.optimization_applied);
        assert!(decision.selected_agent.contains("claude"));
    }

    #[test]
    fn test_route_with_history() {
        let mut router = SelfOptimizingRouter::new();

        // Add enough history
        for i in 0..15 {
            router.record_execution(ExecutionRecord {
                task_id: format!("task-{}", i),
                scene_type: SceneType::WebDevelopment,
                platform: Platform::Web,
                agent_id: "claude-code".to_string(),
                success: true,
                duration_ms: 3000,
                cost: 0.01,
                timestamp: i as u64,
            });
        }

        let decision = router.route(
            "Fix a bug",
            InputType::Text,
            Some(SceneType::WebDevelopment),
            Some(Platform::Web),
            &["claude-code".to_string()],
        );

        assert!(decision.optimization_applied);
        assert_eq!(decision.selected_agent, "claude-code");
    }

    #[test]
    fn test_history_summary() {
        let mut router = SelfOptimizingRouter::new();

        for i in 0..10 {
            router.record_execution(ExecutionRecord {
                task_id: format!("task-{}", i),
                scene_type: SceneType::WebDevelopment,
                platform: Platform::Web,
                agent_id: "claude-code".to_string(),
                success: i % 2 == 0,
                duration_ms: 5000,
                cost: 0.01,
                timestamp: i as u64,
            });
        }

        let summary = router.get_history_summary();
        assert_eq!(summary.total_executions, 10);
        assert_eq!(summary.success_count, 5);
        assert_eq!(summary.failure_count, 5);
        assert_eq!(summary.success_rate, 0.5);
        assert_eq!(summary.avg_latency_ms, 5000);
        assert!((summary.total_cost - 0.1).abs() < 0.01); // Floating point tolerance
    }

    #[test]
    fn test_reset_proficiencies() {
        let mut router = SelfOptimizingRouter::new();

        router.record_execution(ExecutionRecord {
            task_id: "task-1".to_string(),
            scene_type: SceneType::WebDevelopment,
            platform: Platform::Web,
            agent_id: "claude-code".to_string(),
            success: true,
            duration_ms: 5000,
            cost: 0.01,
            timestamp: 0,
        });

        assert_eq!(router.proficiencies.len(), 1);

        router.reset_proficiencies();
        assert_eq!(router.proficiencies.len(), 0);
    }
}
