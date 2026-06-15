//! Goal Parser - Parse natural language into structured goals

use acp_core::{GoalSpec, CompletionConditionSpec, EvaluatorSpec};
use crate::{ParseResult, ComplexityLevel, DependencyDetector, ConditionInference};
use thiserror::Error;
use uuid::Uuid;
use regex::Regex;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Empty requirement")]
    EmptyRequirement,

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Dependency cycle detected")]
    DependencyCycle,
}

/// Goal parser for converting requirements to GoalSpec
pub struct GoalParser {
    /// Dependency detector
    dependency_detector: DependencyDetector,
    /// Condition inference engine
    condition_inference: ConditionInference,
    /// Default token budget
    default_token_budget: u64,
    /// Default max iterations
    default_max_iterations: u32,
}

impl GoalParser {
    pub fn new() -> Self {
        Self {
            dependency_detector: DependencyDetector::new(),
            condition_inference: ConditionInference::new(),
            default_token_budget: 50_000,
            default_max_iterations: 5,
        }
    }

    pub fn with_defaults(mut self, token_budget: u64, max_iterations: u32) -> Self {
        self.default_token_budget = token_budget;
        self.default_max_iterations = max_iterations;
        self
    }

    /// Parse a requirement string into goals
    pub fn parse(&self, requirement: &str) -> Result<ParseResult, ParseError> {
        if requirement.trim().is_empty() {
            return Err(ParseError::EmptyRequirement);
        }

        // Split requirement into tasks
        let tasks = self.split_into_tasks(requirement);

        if tasks.is_empty() {
            return Err(ParseError::InvalidFormat("No tasks detected".into()));
        }

        // Convert tasks to GoalSpec
        let mut goals: Vec<GoalSpec> = tasks
            .iter()
            .enumerate()
            .map(|(i, task)| self.task_to_goal(task, i))
            .collect();

        // Detect and add dependencies
        self.dependency_detector.detect_dependencies(&mut goals);

        // Validate no cycles
        if self.has_dependency_cycle(&goals) {
            return Err(ParseError::DependencyCycle);
        }

        // Infer completion conditions
        for goal in &mut goals {
            if matches!(goal.completion_condition, CompletionConditionSpec::Custom { .. }) {
                goal.completion_condition = self.condition_inference.infer(&goal.description);
            }
        }

        let complexity = ComplexityLevel::from_goal_count(goals.len());

        Ok(ParseResult {
            goals,
            source: requirement.to_string(),
            complexity,
        })
    }

    /// Split requirement into individual tasks
    fn split_into_tasks(&self, requirement: &str) -> Vec<String> {
        // Patterns: numbered lists, bullet points, or sentence splits
        let numbered = Regex::new(r"(?m)^\d+[\.\)]\s*(.+)$").unwrap();
        let bulleted = Regex::new(r"(?m)^[-*]\s*(.+)$").unwrap();

        // Try numbered list first
        let numbered_tasks: Vec<String> = numbered
            .captures_iter(requirement)
            .map(|c| c[1].trim().to_string())
            .collect();

        if !numbered_tasks.is_empty() {
            return numbered_tasks;
        }

        // Try bulleted list
        let bulleted_tasks: Vec<String> = bulleted
            .captures_iter(requirement)
            .map(|c| c[1].trim().to_string())
            .collect();

        if !bulleted_tasks.is_empty() {
            return bulleted_tasks;
        }

        // Fallback: treat whole requirement as single task if meaningful
        let trimmed = requirement.trim();
        if trimmed.len() >= 5 {
            return vec![trimmed.to_string()];
        }

        // Split by newlines, accepting shorter lines
        requirement
            .split('\n')
            .filter(|s| s.trim().len() >= 5)
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// Convert a task string to GoalSpec
    fn task_to_goal(&self, task: &str, _index: usize) -> GoalSpec {
        let id = format!("goal-{}", Uuid::new_v4().to_string().split('-').next().unwrap());

        GoalSpec {
            id,
            description: task.to_string(),
            completion_condition: CompletionConditionSpec::Custom {
                evaluator: "auto".to_string(),
            },
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: self.default_token_budget,
            max_iterations: self.default_max_iterations,
        }
    }

    /// Check for dependency cycles using DFS
    fn has_dependency_cycle(&self, goals: &[GoalSpec]) -> bool {
        let goal_ids: std::collections::HashSet<&str> = goals.iter().map(|g| g.id.as_str()).collect();

        for goal in goals {
            for dep in &goal.depends_on {
                if !goal_ids.contains(dep.as_str()) {
                    // Dependency not in set - skip (external dependency)
                    continue;
                }
                // Check if dep depends on goal (cycle)
                if let Some(dep_goal) = goals.iter().find(|g| g.id == *dep) {
                    if dep_goal.depends_on.contains(&goal.id) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Default for GoalParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_new() {
        let parser = GoalParser::new();
        assert_eq!(parser.default_token_budget, 50_000);
        assert_eq!(parser.default_max_iterations, 5);
    }

    #[test]
    fn parser_empty_requirement() {
        let parser = GoalParser::new();
        let result = parser.parse("");
        assert!(matches!(result, Err(ParseError::EmptyRequirement)));
    }

    #[test]
    fn parser_whitespace_only() {
        let parser = GoalParser::new();
        let result = parser.parse("   \n   ");
        assert!(matches!(result, Err(ParseError::EmptyRequirement)));
    }

    #[test]
    fn parser_numbered_list() {
        let parser = GoalParser::new();
        let result = parser.parse("1. Fix bug\n2. Add tests\n3. Commit changes").unwrap();

        assert_eq!(result.goals.len(), 3);
        assert_eq!(result.complexity, ComplexityLevel::Moderate);
        assert!(result.goals[0].description.contains("Fix bug"));
    }

    #[test]
    fn parser_bulleted_list() {
        let parser = GoalParser::new();
        let result = parser.parse("- Task one\n- Task two").unwrap();

        assert_eq!(result.goals.len(), 2);
        assert!(result.goals[0].description.contains("Task one"));
    }

    #[test]
    fn parser_single_task() {
        let parser = GoalParser::new();
        let result = parser.parse("Fix the compilation error in main.rs").unwrap();

        assert_eq!(result.goals.len(), 1);
        assert_eq!(result.complexity, ComplexityLevel::Simple);
    }

    #[test]
    fn parser_with_defaults() {
        let parser = GoalParser::new().with_defaults(100_000, 10);
        let result = parser.parse("Test task").unwrap();

        assert_eq!(result.goals[0].token_budget, 100_000);
        assert_eq!(result.goals[0].max_iterations, 10);
    }

    #[test]
    fn parser_generates_unique_ids() {
        let parser = GoalParser::new();
        let result = parser.parse("1. Task one\n2. Task two").unwrap();

        assert_ne!(result.goals[0].id, result.goals[1].id);
    }

    #[test]
    fn parser_default_fields() {
        let parser = GoalParser::new();
        let result = parser.parse("Test task").unwrap();

        let goal = &result.goals[0];
        assert!(goal.executor.is_none());
        assert!(goal.depends_on.is_empty());
        assert_eq!(goal.evaluator, EvaluatorSpec::Auto);
    }
}