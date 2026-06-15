//! Goal Parser - Parse natural language requirements into GoalSpec
//!
//! Provides:
//! - Natural language parsing to structured goals
//! - Dependency detection and ordering
//! - Default completion conditions inference

pub mod parser;
pub mod dependency_detector;
pub mod condition_inference;

pub use parser::{GoalParser, ParseError};
pub use dependency_detector::DependencyDetector;
pub use condition_inference::ConditionInference;

use acp_core::GoalSpec;

/// Parse result containing goals and metadata
#[derive(Debug)]
pub struct ParseResult {
    /// Parsed goals
    pub goals: Vec<GoalSpec>,
    /// Original requirement text
    pub source: String,
    /// Detected complexity level
    pub complexity: ComplexityLevel,
}

/// Complexity level of the requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

impl ComplexityLevel {
    pub fn from_goal_count(count: usize) -> Self {
        if count <= 1 {
            ComplexityLevel::Simple
        } else if count <= 5 {
            ComplexityLevel::Moderate
        } else {
            ComplexityLevel::Complex
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_level_simple() {
        assert_eq!(ComplexityLevel::from_goal_count(1), ComplexityLevel::Simple);
    }

    #[test]
    fn complexity_level_moderate() {
        assert_eq!(ComplexityLevel::from_goal_count(3), ComplexityLevel::Moderate);
        assert_eq!(ComplexityLevel::from_goal_count(5), ComplexityLevel::Moderate);
    }

    #[test]
    fn complexity_level_complex() {
        assert_eq!(ComplexityLevel::from_goal_count(6), ComplexityLevel::Complex);
        assert_eq!(ComplexityLevel::from_goal_count(10), ComplexityLevel::Complex);
    }

    #[test]
    fn parse_result_new() {
        let result = ParseResult {
            goals: vec![],
            source: "test".to_string(),
            complexity: ComplexityLevel::Simple,
        };
        assert!(result.goals.is_empty());
    }
}