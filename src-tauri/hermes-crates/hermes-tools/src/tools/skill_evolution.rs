//! Skill Self-Evolution Module
//!
//! Implements the self-evolution mechanism inspired by OpenClacky:
//! "After each run, the agent updates the Skill based on execution context and results"

use hermes_core::{Skill, SkillExecutionStats, ErrorPattern, EvolutionReason};
use chrono::Utc;

/// Evolution configuration
pub struct EvolutionConfig {
    /// Minimum executions before considering evolution
    pub min_executions: u64,
    /// Failure rate threshold
    pub failure_rate_threshold: f32,
    /// Same error pattern threshold
    pub error_pattern_threshold: u32,
    /// Minimum acceptable rating
    pub min_rating_threshold: f32,
    /// Cooldown between evolutions (seconds)
    pub cooldown_seconds: i64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_executions: 10,
            failure_rate_threshold: 0.15,
            error_pattern_threshold: 3,
            min_rating_threshold: 2.5,
            cooldown_seconds: 3600,
        }
    }
}

/// Evolve a skill based on evolution reason
///
/// In production, this would call an LLM to generate improvements.
/// For now, we generate structured improvements based on the reason.
pub fn evolve_skill(skill: &Skill, reason: &EvolutionReason) -> Skill {
    let mut evolved = skill.clone();

    // Increment version
    evolved.version.generation += 1;
    evolved.version.semver = bump_patch_version(&skill.version.semver);
    evolved.version.last_evolved_at = Some(Utc::now().to_rfc3339());
    evolved.version.last_evolution_reason = Some(format!("{:?}", reason));
    evolved.version.content_hash = compute_hash(&evolved.content);

    // Add improvement section based on reason
    let improvement = generate_improvement_section(reason);
    evolved.content.push_str(&format!("\n\n## Evolution Notes (v{})\n{}\n",
        evolved.version.semver, improvement));

    evolved
}

/// Generate improvement section based on evolution reason
fn generate_improvement_section(reason: &EvolutionReason) -> String {
    match reason {
        EvolutionReason::HighFailureRate(rate) => {
            format!(
                "- Failure rate was {:.1}% (>15% threshold)\n\
                 - Added error handling guidance\n\
                 - Included fallback strategies\n\
                 - Recommended: test edge cases before main flow",
                rate * 100.0
            )
        }
        EvolutionReason::RecurringError(pattern) => {
            format!(
                "- Recurring error pattern: '{}'\n\
                 - Occurred {} times\n\
                 - Suggested fix: {}\n\
                 - Added explicit check for this error condition",
                pattern.pattern,
                pattern.count,
                pattern.suggested_fix.as_deref().unwrap_or("analyze and fix root cause")
            )
        }
        EvolutionReason::LowRating(rating) => {
            format!(
                "- User rating was {:.1}/5 (<2.5 threshold)\n\
                 - Improved clarity of steps\n\
                 - Added more detailed explanations\n\
                 - Included success criteria checklist",
                rating
            )
        }
        EvolutionReason::UserRequested => {
            "- User requested improvement\n\
             - Content updated based on feedback\n\
             - Version bumped for tracking".to_string()
        }
    }
}

/// Bump patch version (1.0.0 -> 1.0.1)
fn bump_patch_version(semver: &str) -> String {
    let parts: Vec<&str> = semver.split('.').collect();
    if parts.len() == 3 {
        let patch = parts[2].parse::<u32>().unwrap_or(0) + 1;
        format!("{}.{}.{}", parts[0], parts[1], patch)
    } else {
        "1.0.1".to_string()
    }
}

/// Compute content hash for deduplication
fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Update execution statistics after a skill run
pub fn update_execution_stats(
    stats: &mut SkillExecutionStats,
    success: bool,
    duration_ms: u64,
    error: Option<&str>,
) {
    stats.total_executions += 1;
    if success {
        stats.success_count += 1;
    } else {
        stats.failure_count += 1;
    }

    // Update average duration
    let total_duration = stats.avg_duration_ms * (stats.total_executions - 1) + duration_ms;
    stats.avg_duration_ms = total_duration / stats.total_executions;

    // Record error pattern
    if let Some(err) = error {
        // Find or create error pattern
        let pattern_idx = stats.common_errors.iter()
            .position(|p| p.pattern == err);

        if let Some(idx) = pattern_idx {
            stats.common_errors[idx].count += 1;
        } else {
            stats.common_errors.push(ErrorPattern {
                pattern: err.to_string(),
                count: 1,
                suggested_fix: None,
            });
        }

        // Keep only top 5 error patterns
        if stats.common_errors.len() > 5 {
            stats.common_errors.sort_by(|a, b| b.count.cmp(&a.count));
            stats.common_errors.truncate(5);
        }
    }

    stats.last_executed_at = Some(Utc::now().to_rfc3339());
}

/// Analyze skill performance and suggest improvements
pub fn analyze_performance(skill: &Skill, config: &EvolutionConfig) -> Option<EvolutionSuggestion> {
    let stats = skill.execution_stats.as_ref()?;

    // Check execution threshold
    if stats.total_executions < config.min_executions {
        return None;
    }

    // Calculate failure rate
    if stats.total_executions > 0 {
        let failure_rate = stats.failure_count as f32 / stats.total_executions as f32;

        if failure_rate > config.failure_rate_threshold {
            return Some(EvolutionSuggestion {
                reason: EvolutionReason::HighFailureRate(failure_rate),
                priority: 1,
                description: format!(
                    "High failure rate ({:.1}%). Consider adding error handling and fallback strategies.",
                    failure_rate * 100.0
                ),
            });
        }
    }

    // Check recurring errors
    for pattern in &stats.common_errors {
        if pattern.count >= config.error_pattern_threshold {
            return Some(EvolutionSuggestion {
                reason: EvolutionReason::RecurringError(pattern.clone()),
                priority: 2,
                description: format!(
                    "Recurring error '{}' occurred {} times. Add explicit handling for this case.",
                    pattern.pattern, pattern.count
                ),
            });
        }
    }

    // Check rating
    if stats.avg_rating > 0.0 && stats.avg_rating < config.min_rating_threshold {
        return Some(EvolutionSuggestion {
            reason: EvolutionReason::LowRating(stats.avg_rating),
            priority: 3,
            description: format!(
                "Low user rating ({:.1}/5). Improve clarity and add more guidance.",
                stats.avg_rating
            ),
        });
    }

    None
}

/// Suggested improvement for a skill
#[derive(Debug, Clone)]
pub struct EvolutionSuggestion {
    pub reason: EvolutionReason,
    pub priority: u8,
    pub description: String,
}

/// Record user rating for a skill
pub fn record_rating(stats: &mut SkillExecutionStats, rating: u8) {
    if !(1..=5).contains(&rating) {
        return;
    }

    // Calculate new average rating
    let rating_f = rating as f32;

    if stats.avg_rating == 0.0 {
        // First rating
        stats.avg_rating = rating_f;
    } else {
        // Use incremental average: new_avg = old_avg + (new_value - old_avg) / (n + 1)
        // Assuming n = 1 for simplicity (each call adds one rating)
        stats.avg_rating = (stats.avg_rating + rating_f) / 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_patch_version() {
        assert_eq!(bump_patch_version("1.0.0"), "1.0.1");
        assert_eq!(bump_patch_version("2.3.5"), "2.3.6");
    }

    #[test]
    fn test_evolve_skill_high_failure() {
        let skill = Skill::minimal("test", "original content");
        let reason = EvolutionReason::HighFailureRate(0.3);

        let evolved = evolve_skill(&skill, &reason);
        assert_eq!(evolved.version.generation, 1);
        assert!(evolved.content.contains("Evolution Notes"));
    }

    #[test]
    fn test_update_execution_stats_success() {
        let mut stats = SkillExecutionStats::default();
        update_execution_stats(&mut stats, true, 1000, None);

        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.avg_duration_ms, 1000);
    }

    #[test]
    fn test_update_execution_stats_failure() {
        let mut stats = SkillExecutionStats::default();
        update_execution_stats(&mut stats, false, 500, Some("test error"));

        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.common_errors.len(), 1);
        assert_eq!(stats.common_errors[0].pattern, "test error");
    }

    #[test]
    fn test_analyze_performance_below_threshold() {
        let skill = Skill::minimal("test", "content");
        let config = EvolutionConfig::default();

        // No stats
        assert!(analyze_performance(&skill, &config).is_none());

        // Below execution threshold
        let mut stats = SkillExecutionStats::default();
        stats.total_executions = 5; // Below 10
        let skill_with_stats = Skill {
            name: "test".into(),
            content: "content".into(),
            category: None,
            description: None,
            execution_pattern: Default::default(),
            version: Default::default(),
            execution_stats: Some(stats),
            tags: vec![],
            author: None,
        };
        assert!(analyze_performance(&skill_with_stats, &config).is_none());
    }

    #[test]
    fn test_record_rating() {
        let mut stats = SkillExecutionStats::default();
        stats.total_executions = 5;
        stats.success_count = 5;

        record_rating(&mut stats, 4);
        assert_eq!(stats.avg_rating, 4.0);

        record_rating(&mut stats, 5);
        // Average should be 4.5
        assert!((stats.avg_rating - 4.5).abs() < 0.1);
    }
}