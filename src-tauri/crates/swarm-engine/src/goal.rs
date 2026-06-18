//! Goal - Unified types from acp-core (C-1 solution)
//!
//! **COMPLETE UNIFICATION**: This module now fully re-exports types from acp-core.
//! The previous duplicate Goal/CompletionCondition/Evaluator definitions have been removed.
//!
//! **Migration Guide**:
//! - `Goal` → use `GoalRuntime` from acp-core (aliased here for convenience)
//! - `CompletionCondition` → use `CompletionConditionSpec` from acp-core (aliased here)
//! - `Evaluator` → use `EvaluatorSpec` from acp-core (aliased here)
//! - Helper functions `command_success()` and `file_exists()` are available on CompletionCondition

// Re-export ALL types from acp-core (C-1 complete solution)
pub use acp_core::{
    GoalRuntime as Goal,           // Alias for backward compatibility
    GoalSpec,                       // Submission specification
    GoalStatus,                     // Execution status enum
    CompletionConditionSpec as CompletionCondition,  // Alias for backward compatibility
    EvaluatorSpec as Evaluator,     // Alias for backward compatibility
    IterationRecord,                // Iteration feedback record
    EvaluationResult,               // Condition evaluation result
    ConditionResult,                // Single condition result
    GoalOutcome,                    // Final execution outcome
    GoalGraphSummary,               // Graph status summary
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_runtime_from_spec() {
        let spec = GoalSpec {
            id: "test-goal".to_string(),
            description: "Test description".to_string(),
            completion_condition: CompletionCondition::command_success("echo"),
            evaluator: Evaluator::Auto,
            executor: Some("worker-001".to_string()),
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let goal = Goal::from_spec(spec);

        assert_eq!(goal.id, "test-goal");
        assert_eq!(goal.description, "Test description");
        assert_eq!(goal.executor, Some("worker-001".to_string()));
        assert_eq!(goal.status, GoalStatus::Pending);
        assert_eq!(goal.tokens_used, 0);
    }

    #[test]
    fn goal_status_is_terminal() {
        // Terminal states
        assert!(GoalStatus::Converged.is_terminal());
        assert!(GoalStatus::Failed { reason: "test".into() }.is_terminal());
        assert!(GoalStatus::Cancelled.is_terminal());
        assert!(GoalStatus::BudgetExhausted.is_terminal());
        assert!(GoalStatus::MaxIterReached.is_terminal());

        // Non-terminal states
        assert!(!GoalStatus::Pending.is_terminal());
        assert!(!GoalStatus::Active.is_terminal());
        assert!(!GoalStatus::Evaluating.is_terminal());
        assert!(!GoalStatus::Iterating { feedback: "test".into() }.is_terminal());
    }

    #[test]
    fn goal_status_is_success() {
        assert!(GoalStatus::Converged.is_success());
        assert!(!GoalStatus::Failed { reason: "test".into() }.is_success());
        assert!(!GoalStatus::Pending.is_success());
        assert!(!GoalStatus::Active.is_success());
    }

    #[test]
    fn goal_budget_exhausted() {
        let spec = GoalSpec {
            id: "g1".to_string(),
            description: "Budget test".to_string(),
            completion_condition: CompletionCondition::command_success("echo"),
            evaluator: Evaluator::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: 1000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let mut goal = Goal::from_spec(spec);

        assert!(!goal.budget_exhausted());

        goal.tokens_used = 500;
        assert!(!goal.budget_exhausted());

        goal.tokens_used = 1000;
        assert!(goal.budget_exhausted());

        goal.tokens_used = 1500;
        assert!(goal.budget_exhausted());
    }

    #[test]
    fn goal_max_iter_reached() {
        let spec = GoalSpec {
            id: "g1".to_string(),
            description: "Iter test".to_string(),
            completion_condition: CompletionCondition::command_success("echo"),
            evaluator: Evaluator::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 3,
            per_iteration_timeout_ms: 60_000,
        };

        let mut goal = Goal::from_spec(spec);

        assert!(!goal.max_iter_reached());

        goal.current_iteration = 2;
        assert!(!goal.max_iter_reached());

        goal.current_iteration = 3;
        assert!(goal.max_iter_reached());
    }

    #[test]
    fn goal_is_ready_no_deps() {
        let spec = GoalSpec {
            id: "g1".to_string(),
            description: "Ready test".to_string(),
            completion_condition: CompletionCondition::command_success("echo"),
            evaluator: Evaluator::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let goal = Goal::from_spec(spec);

        assert!(goal.is_ready(&[]));
    }

    #[test]
    fn goal_is_ready_with_deps() {
        let spec = GoalSpec {
            id: "g1".to_string(),
            description: "Dep test".to_string(),
            completion_condition: CompletionCondition::command_success("echo"),
            evaluator: Evaluator::Auto,
            executor: None,
            depends_on: vec!["goal-a".to_string(), "goal-b".to_string()],
            token_budget: 50_000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let goal = Goal::from_spec(spec);

        // No deps converged -> not ready
        assert!(!goal.is_ready(&[]));

        // Partial deps converged -> not ready
        assert!(!goal.is_ready(&["goal-a"]));

        // All deps converged -> ready
        assert!(goal.is_ready(&["goal-a", "goal-b"]));
    }

    #[test]
    fn completion_condition_helpers() {
        let cc_cmd = CompletionCondition::command_success("npm test");
        assert!(matches!(cc_cmd, CompletionCondition::CommandSuccess { .. }));

        let cc_file = CompletionCondition::file_exists("README.md");
        assert!(matches!(cc_file, CompletionCondition::FileCheck { .. }));
    }
}