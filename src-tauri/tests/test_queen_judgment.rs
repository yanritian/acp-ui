//! FT-QueenJudgment: Queen Judgment End-to-End Flow Test
//!
//! Validates QueenJudgment components:
//! 1. Goal creation with QueenJudgment completion condition
//! 2. Goal YAML parsing for QueenJudgment
//! 3. GoalGraph submission and retrieval
//! 4. Status iteration tracking
//! 5. GoalRuntime conversion

use acp_core::{GoalRuntime, GoalYamlFile};
use acp_ui_lib::{
    CompletionCondition, CompletionConditionSpec, ConditionResult, EvaluationResult, EvaluatorSpec,
    Goal, GoalGraph, GoalStatus, IterationRecord,
};
use swarm_engine::EchoExecutor;

/// Helper: convert local CompletionCondition to CompletionConditionSpec
fn make_queen_judgment(criteria: &str) -> CompletionConditionSpec {
    CompletionConditionSpec::QueenJudgment {
        criteria: criteria.to_string(),
    }
}

/// Helper: convert local Evaluator to EvaluatorSpec
fn make_queen_evaluator(queen_worker_id: &str) -> EvaluatorSpec {
    EvaluatorSpec::Queen {
        queen_worker_id: queen_worker_id.to_string(),
    }
}

#[test]
fn test_queen_judgment_goal_creation() {
    let mut goal = Goal::new(
        "code-review-001".to_string(),
        "Review code changes for quality".to_string(),
        make_queen_judgment("Code follows best practices and has no security issues"),
    );
    goal.evaluator = make_queen_evaluator("queen-worker");
    goal.executor = Some("reviewer-worker".to_string());

    assert_eq!(goal.id, "code-review-001");
    assert!(matches!(
        goal.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
    assert!(matches!(goal.evaluator, EvaluatorSpec::Queen { .. }));
    assert_eq!(goal.executor, Some("reviewer-worker".to_string()));
}

#[test]
fn test_goal_yaml_parse_queen_judgment() {
    let yaml_content = r#"
name: queen-test
description: "Queen judgment test"
goals:
  - id: test-goal
    description: "Test Queen judgment"
    completion_condition:
      type: queen_judgment
      criteria: "Output is correct"
    evaluator:
      type: queen
      queen_worker_id: queen-001
    executor: worker-001
    depends_on: []
    token_budget: 50000
    max_iterations: 5
"#;

    let parsed: GoalYamlFile = serde_yaml::from_str(yaml_content).expect("YAML parse failed");

    assert_eq!(parsed.name, "queen-test");
    assert_eq!(parsed.goals.len(), 1);

    let goal = &parsed.goals[0];
    assert_eq!(goal.id, "test-goal");
    assert!(matches!(
        goal.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
    assert!(matches!(goal.evaluator, EvaluatorSpec::Queen { .. }));
}

#[test]
fn test_goal_yaml_from_file() {
    // Parse the actual example file
    let yaml_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/queen-judgment.goal.yaml"
    );
    let content = std::fs::read_to_string(yaml_path).expect("Read yaml file");

    let parsed: GoalYamlFile = serde_yaml::from_str(&content).expect("Parse yaml");

    assert_eq!(parsed.name, "queen-judgment-demo");
    assert_eq!(parsed.goals.len(), 2);

    // First goal: code-review
    let goal1 = &parsed.goals[0];
    assert_eq!(goal1.id, "code-review");
    assert!(matches!(
        goal1.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
    assert!(matches!(goal1.evaluator, EvaluatorSpec::Queen { .. }));

    // Second goal: architecture-decision
    let goal2 = &parsed.goals[1];
    assert_eq!(goal2.id, "architecture-decision");
    assert!(matches!(
        goal2.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
}

#[test]
fn test_echo_executor_available() {
    // Verify EchoExecutor can be created for mock testing
    let _executor = EchoExecutor::new(
        "test-worker".to_string(),
        "success output".to_string(),
        true,
    );
    let _executor_fail = EchoExecutor::new(
        "test-worker".to_string(),
        "failure output".to_string(),
        false,
    );
    // EchoExecutor exists and can be instantiated
    assert!(true);
}

#[test]
fn test_goal_graph_submit_queen_judgment() {
    let graph = GoalGraph::new();

    let mut goal = Goal::new(
        "queen-goal-001".to_string(),
        "Review code quality".to_string(),
        make_queen_judgment("Code follows best practices"),
    );
    goal.evaluator = make_queen_evaluator("queen-worker");
    goal.executor = Some("reviewer".to_string());

    // Submit to graph
    graph.submit(goal.clone()).expect("Submit failed");

    // Retrieve
    let retrieved = graph.get(&goal.id).expect("Get failed");
    assert_eq!(retrieved.id, goal.id);
    assert!(matches!(
        retrieved.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
}

#[test]
fn test_queen_judgment_criteria_variants() {
    // Test various criteria formats
    let criteria_samples = [
        "Code follows best practices",
        "No security vulnerabilities found",
        "Documentation is complete",
        "Performance meets requirements (latency < 100ms)",
    ];

    for criteria in criteria_samples {
        let condition = CompletionCondition::QueenJudgment {
            criteria: criteria.to_string(),
        };

        // Verify condition can be created and converted
        assert!(matches!(
            condition,
            CompletionCondition::QueenJudgment { .. }
        ));
        let spec = condition.to_spec();
        assert!(matches!(
            spec,
            CompletionConditionSpec::QueenJudgment { .. }
        ));
    }
}

#[test]
fn test_goal_status_iteration_tracking() {
    let mut goal = Goal::new(
        "iter-test".to_string(),
        "Test iteration tracking".to_string(),
        make_queen_judgment("Must pass"),
    );
    goal.max_iterations = 3;

    // Initial state
    assert_eq!(goal.current_iteration_number(), 1);
    assert!(goal.iteration_log.is_empty());

    // Add iteration record
    goal.iteration_log.push(IterationRecord {
        iteration: 1,
        worker_output: "First attempt output".to_string(),
        evaluation: EvaluationResult {
            passed: false,
            explanation: "Not yet converged".to_string(),
            details: vec![ConditionResult {
                description: "Queen judgment".to_string(),
                passed: false,
                evidence: "Try again".to_string(),
            }],
        },
        feedback: Some("Try again".to_string()),
        tokens_used: 100,
        timestamp: 0,
        duration_ms: 500,
    });

    assert_eq!(goal.current_iteration_number(), 2);
    assert_eq!(goal.iteration_log.len(), 1);

    // Add another iteration with success
    goal.iteration_log.push(IterationRecord {
        iteration: 2,
        worker_output: "Second attempt output".to_string(),
        evaluation: EvaluationResult {
            passed: true,
            explanation: "Converged".to_string(),
            details: vec![ConditionResult {
                description: "Queen judgment".to_string(),
                passed: true,
                evidence: "Criteria met".to_string(),
            }],
        },
        feedback: None,
        tokens_used: 150,
        timestamp: 1000,
        duration_ms: 600,
    });

    assert_eq!(goal.current_iteration_number(), 3);

    // Mark as converged
    goal.status = GoalStatus::Converged;
    assert!(goal.status.is_terminal());
    assert!(goal.status.is_success());
}

#[test]
fn test_goal_to_runtime_conversion_queen_judgment() {
    let mut goal = Goal::new(
        "conversion-test".to_string(),
        "Test GoalRuntime conversion".to_string(),
        make_queen_judgment("Test criteria"),
    );
    goal.evaluator = make_queen_evaluator("queen-001");

    // Goal IS GoalRuntime (re-exported), so it already has the spec fields
    assert_eq!(goal.id, "conversion-test");
    assert!(matches!(
        goal.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
    assert!(matches!(goal.evaluator, EvaluatorSpec::Queen { .. }));
}

#[test]
fn test_goal_from_runtime_conversion_queen_judgment() {
    let runtime = GoalRuntime {
        id: "from-rt-test".to_string(),
        description: "Test from runtime".to_string(),
        parent_task_id: None,
        parent_goal_id: None,
        completion_condition: CompletionConditionSpec::QueenJudgment {
            criteria: "Runtime criteria".to_string(),
        },
        evaluator: EvaluatorSpec::Queen {
            queen_worker_id: "queen-rt".to_string(),
        },
        executor: Some("worker-rt".to_string()),
        depends_on: vec![],
        token_budget: Some(50000),
        tokens_used: 0,
        max_iterations: 5,
        current_iteration: 0,
        per_iteration_timeout_ms: 60000,
        status: acp_core::GoalStatus::Pending,
        iteration_log: vec![],
        created_at: 0,
        started_at: None,
        converged_at: None,
        output_files: vec![],
    };

    // Clone GoalRuntime directly (Goal is re-exported as GoalRuntime)
    let goal: Goal = runtime.clone();

    assert_eq!(goal.id, runtime.id);
    assert!(matches!(
        goal.completion_condition,
        CompletionConditionSpec::QueenJudgment { .. }
    ));
    assert!(matches!(goal.evaluator, EvaluatorSpec::Queen { .. }));
}
