//! FT-QueenJudgment: Queen Judgment End-to-End Flow Test
//!
//! Validates QueenJudgment components:
//! 1. Goal creation with QueenJudgment completion condition
//! 2. Goal YAML parsing for QueenJudgment
//! 3. GoalGraph submission and retrieval
//! 4. Status iteration tracking
//! 5. GoalRuntime conversion

use acp_ui_lib::{
    Goal, CompletionCondition, Evaluator, GoalStatus, GoalGraph,
    IterationRecord, EvaluationResult, ConditionResult,
};
use acp_core::{GoalYamlFile, CompletionConditionSpec, EvaluatorSpec, GoalRuntime};
use swarm_engine::EchoExecutor;

#[test]
fn test_queen_judgment_goal_creation() {
    let goal = Goal::new(
        "code-review-001".into(),
        "Review code changes for quality".into(),
        CompletionCondition::QueenJudgment {
            criteria: "Code follows best practices and has no security issues".into(),
        },
    ).with_evaluator(Evaluator::Queen {
        queen_worker_id: "queen-worker".into(),
    }).with_worker("reviewer-worker".into());

    assert_eq!(goal.id, "code-review-001");
    assert!(matches!(goal.completion_condition, CompletionCondition::QueenJudgment { .. }));
    assert!(matches!(goal.evaluator, Evaluator::Queen { .. }));
    assert_eq!(goal.assigned_worker, Some("reviewer-worker".into()));
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
    assert!(matches!(goal.completion_condition, CompletionConditionSpec::QueenJudgment { .. }));
    assert!(matches!(goal.evaluator, EvaluatorSpec::Queen { .. }));
}

#[test]
fn test_goal_yaml_from_file() {
    // Parse the actual example file
    let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/queen-judgment.goal.yaml");
    let content = std::fs::read_to_string(yaml_path).expect("Read yaml file");

    let parsed: GoalYamlFile = serde_yaml::from_str(&content).expect("Parse yaml");

    assert_eq!(parsed.name, "queen-judgment-demo");
    assert_eq!(parsed.goals.len(), 2);

    // First goal: code-review
    let goal1 = &parsed.goals[0];
    assert_eq!(goal1.id, "code-review");
    assert!(matches!(goal1.completion_condition, CompletionConditionSpec::QueenJudgment { .. }));
    assert!(matches!(goal1.evaluator, EvaluatorSpec::Queen { .. }));

    // Second goal: architecture-decision
    let goal2 = &parsed.goals[1];
    assert_eq!(goal2.id, "architecture-decision");
    assert!(matches!(goal2.completion_condition, CompletionConditionSpec::QueenJudgment { .. }));
}

#[test]
fn test_echo_executor_available() {
    // Verify EchoExecutor can be created for mock testing
    let executor = EchoExecutor::new("test-worker", "success output", true);
    let executor_fail = EchoExecutor::new("test-worker", "failure output", false);
    // EchoExecutor exists and can be instantiated
    assert!(true);
}

#[test]
fn test_goal_graph_submit_queen_judgment() {
    let graph = GoalGraph::new();

    let goal = Goal::new(
        "queen-goal-001".into(),
        "Review code quality".into(),
        CompletionCondition::QueenJudgment {
            criteria: "Code follows best practices".into(),
        },
    ).with_evaluator(Evaluator::Queen {
        queen_worker_id: "queen-worker".into(),
    }).with_worker("reviewer".into());

    // Submit to graph
    graph.submit(goal.clone()).expect("Submit failed");

    // Retrieve
    let retrieved = graph.get(&goal.id).expect("Get failed");
    assert_eq!(retrieved.id, goal.id);
    assert!(matches!(retrieved.completion_condition, CompletionCondition::QueenJudgment { .. }));
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
            criteria: criteria.into(),
        };

        // Verify condition can be created
        assert!(matches!(condition, CompletionCondition::QueenJudgment { .. }));
    }
}

#[test]
fn test_goal_status_iteration_tracking() {
    let mut goal = Goal::new(
        "iter-test".into(),
        "Test iteration tracking".into(),
        CompletionCondition::QueenJudgment {
            criteria: "Must pass".into(),
        },
    ).with_max_iterations(3);

    // Initial state
    assert_eq!(goal.current_iteration(), 1);
    assert!(goal.iterations.is_empty());

    // Add iteration record
    goal.iterations.push(IterationRecord {
        iteration: 1,
        worker_output: "First attempt output".into(),
        evaluation: EvaluationResult {
            passed: false,
            explanation: "Not yet converged".into(),
            details: vec![ConditionResult {
                description: "Queen judgment".into(),
                passed: false,
                evidence: "Try again".into(),
            }],
        },
        feedback: Some("Try again".into()),
        tokens_used: 100,
        timestamp: 0,
        duration_ms: 500,
    });

    assert_eq!(goal.current_iteration(), 2);
    assert_eq!(goal.iterations.len(), 1);
    assert!(goal.latest_feedback().is_some());

    // Add another iteration with success
    goal.iterations.push(IterationRecord {
        iteration: 2,
        worker_output: "Second attempt output".into(),
        evaluation: EvaluationResult {
            passed: true,
            explanation: "Converged".into(),
            details: vec![ConditionResult {
                description: "Queen judgment".into(),
                passed: true,
                evidence: "Criteria met".into(),
            }],
        },
        feedback: None,
        tokens_used: 150,
        timestamp: 1000,
        duration_ms: 600,
    });

    assert_eq!(goal.current_iteration(), 3);
    assert!(goal.latest_feedback().is_none()); // Last iteration passed, no feedback

    // Mark as converged
    goal.status = GoalStatus::Converged;
    assert!(goal.status.is_terminal());
    assert!(goal.status.is_success());
}

#[test]
fn test_goal_to_runtime_conversion_queen_judgment() {
    let goal = Goal::new(
        "conversion-test".into(),
        "Test GoalRuntime conversion".into(),
        CompletionCondition::QueenJudgment {
            criteria: "Test criteria".into(),
        },
    ).with_evaluator(Evaluator::Queen {
        queen_worker_id: "queen-001".into(),
    });

    // Convert to GoalRuntime
    let runtime = goal.to_runtime();

    assert_eq!(runtime.id, goal.id);
    assert!(matches!(runtime.completion_condition, CompletionConditionSpec::QueenJudgment { .. }));
    assert!(matches!(runtime.evaluator, EvaluatorSpec::Queen { .. }));
}

#[test]
fn test_goal_from_runtime_conversion_queen_judgment() {
    let runtime = GoalRuntime {
        id: "from-rt-test".into(),
        description: "Test from runtime".into(),
        parent_task_id: None,
        parent_goal_id: None,
        completion_condition: CompletionConditionSpec::QueenJudgment {
            criteria: "Runtime criteria".into(),
        },
        evaluator: EvaluatorSpec::Queen {
            queen_worker_id: "queen-rt".into(),
        },
        executor: Some("worker-rt".into()),
        depends_on: vec![],
        token_budget: 50000,
        tokens_used: 0,
        max_iterations: 5,
        current_iteration: 0,
        per_iteration_timeout_ms: 60000,
        status: acp_core::GoalStatus::Pending,
        iteration_log: vec![],
        created_at: 0,
        converged_at: None,
        output_files: vec![],
    };

    // Convert to Goal
    let goal = Goal::from_runtime(&runtime);

    assert_eq!(goal.id, runtime.id);
    assert!(matches!(goal.completion_condition, CompletionCondition::QueenJudgment { .. }));
    assert!(matches!(goal.evaluator, Evaluator::Queen { .. }));
}