//! FT-1: Goal Complete Lifecycle Test
//!
//! Validates the full flow: Submit → Assign → Execute → Iterate → Converge

use acp_ui_lib::{
    CompletionConditionSpec, ConditionResult, EvaluationResult, Goal, GoalGraph, GoalStatus,
    IterationRecord,
};

/// Helper to create a goal with OutputContains condition
fn create_goal(id: &str, description: &str) -> Goal {
    Goal::new(
        id.to_string(),
        description.to_string(),
        CompletionConditionSpec::OutputContains {
            command: "".to_string(),
            pattern: "success".to_string(),
            case_sensitive: false,
        },
    )
}

#[test]
fn test_goal_submit_initial_status() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Fix TypeScript errors");

    // Submit should succeed
    assert!(graph.submit(goal).is_ok());

    // Initial status should be Pending
    let stored = graph.get("g1").unwrap();
    assert_eq!(stored.status, GoalStatus::Pending);
    assert!(stored.iteration_log.is_empty());
    assert!(stored.started_at.is_none());
    assert!(stored.converged_at.is_none());
}

#[test]
fn test_goal_assign_worker_transitions_to_active() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Run tests");
    graph.submit(goal).unwrap();

    // Assign worker
    graph
        .assign_worker("g1", "codex-worker-001".to_string())
        .unwrap();

    // Should transition to Active
    let assigned = graph.get("g1").unwrap();
    assert_eq!(assigned.status, GoalStatus::Active);
    assert_eq!(assigned.executor, Some("codex-worker-001".to_string()));
    assert!(assigned.started_at.is_some());
}

#[test]
fn test_goal_iteration_flow_to_converged() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Build project");
    graph.submit(goal).unwrap();
    graph.assign_worker("g1", "worker-001".to_string()).unwrap();

    // First iteration - failed
    let iteration1 = IterationRecord {
        iteration: 1,
        worker_output: "Build failed: 3 errors".to_string(),
        evaluation: EvaluationResult {
            passed: false,
            explanation: "Compilation errors remain".to_string(),
            details: vec![ConditionResult {
                description: "Output contains 'success'".to_string(),
                passed: false,
                evidence: "Output: 'Build failed: 3 errors'".to_string(),
            }],
        },
        feedback: Some("Fix the 3 TypeScript errors in src/utils.ts".to_string()),
        tokens_used: 5000,
        timestamp: 1000,
        duration_ms: 30000,
    };
    graph.add_iteration("g1", iteration1).unwrap();

    // Second iteration - failed again
    let iteration2 = IterationRecord {
        iteration: 2,
        worker_output: "Build failed: 1 error".to_string(),
        evaluation: EvaluationResult {
            passed: false,
            explanation: "One error left".to_string(),
            details: vec![ConditionResult {
                description: "Output contains 'success'".to_string(),
                passed: false,
                evidence: "Output: 'Build failed: 1 error'".to_string(),
            }],
        },
        feedback: Some("Fix the remaining error in src/api.ts".to_string()),
        tokens_used: 3000,
        timestamp: 2000,
        duration_ms: 25000,
    };
    graph.add_iteration("g1", iteration2).unwrap();

    // Third iteration - success!
    let iteration3 = IterationRecord {
        iteration: 3,
        worker_output: "Build success! No errors".to_string(),
        evaluation: EvaluationResult {
            passed: true,
            explanation: "Build completed successfully".to_string(),
            details: vec![ConditionResult {
                description: "Output contains 'success'".to_string(),
                passed: true,
                evidence: "Found 'success' in output".to_string(),
            }],
        },
        feedback: None,
        tokens_used: 2000,
        timestamp: 3000,
        duration_ms: 15000,
    };
    graph.add_iteration("g1", iteration3).unwrap();

    // Mark as converged
    graph.update_status("g1", GoalStatus::Converged).unwrap();

    // Verify final state
    let final_goal = graph.get("g1").unwrap();
    assert_eq!(final_goal.status, GoalStatus::Converged);
    assert_eq!(final_goal.iteration_log.len(), 3);

    // Note: token_used is not automatically accumulated by GoalGraph.add_iteration
    // The actual reconciliation loop would handle this
}

#[test]
fn test_goal_summary_tracks_all_statuses() {
    let graph = GoalGraph::new();

    // Create goals with different statuses
    let g1 = create_goal("g1", "Pending goal");
    graph.submit(g1).unwrap();

    let mut g2 = create_goal("g2", "Active goal");
    g2.status = GoalStatus::Active;
    graph.submit(g2).unwrap();

    let mut g3 = create_goal("g3", "Converged goal");
    g3.status = GoalStatus::Converged;
    graph.submit(g3).unwrap();

    let mut g4 = create_goal("g4", "Failed goal");
    g4.status = GoalStatus::Failed {
        reason: "Max iterations".to_string(),
    };
    graph.submit(g4).unwrap();

    // Get summary
    let summary = graph.summary();

    assert_eq!(summary.total, 4);
    assert_eq!(summary.pending, 1);
    assert_eq!(summary.active, 1);
    assert_eq!(summary.converged, 1);
    assert_eq!(summary.failed, 1);
}
