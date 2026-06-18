//! FT-2: Goal Dependency Chain Test (DAG Topology)
//!
//! Validates A → B → C dependency chain where each Goal waits for predecessor

use acp_ui_lib::{Goal, GoalGraph, GoalStatus, CompletionCondition};

fn create_goal_with_dep(id: &str, description: &str, deps: Vec<String>) -> Goal {
    let mut goal = Goal::new(
        id.to_string(),
        description.to_string(),
        CompletionCondition::OutputContains {
            text: "done".to_string(),
            case_sensitive: false,
        }.to_spec(),
    );
    goal.depends_on = deps;
    goal
}

#[test]
fn test_chain_only_first_goal_ready() {
    let graph = GoalGraph::new();

    // A (no deps) → B (depends on A) → C (depends on B)
    let a = create_goal_with_dep("goal-a", "First task", vec![]);
    let b = create_goal_with_dep("goal-b", "Second task", vec!["goal-a".to_string()]);
    let c = create_goal_with_dep("goal-c", "Third task", vec!["goal-b".to_string()]);

    // Must submit in order (parent first)
    graph.submit(a).unwrap();
    graph.submit(b).unwrap();
    graph.submit(c).unwrap();

    // Only A should be ready
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-a");
}

#[test]
fn test_chain_progressive_ready_after_convergence() {
    let graph = GoalGraph::new();

    let a = create_goal_with_dep("goal-a", "First", vec![]);
    let b = create_goal_with_dep("goal-b", "Second", vec!["goal-a".to_string()]);
    let c = create_goal_with_dep("goal-c", "Third", vec!["goal-b".to_string()]);

    graph.submit(a).unwrap();
    graph.submit(b).unwrap();
    graph.submit(c).unwrap();

    // Stage 1: A ready, B and C blocked
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-a");

    // Mark A as converged
    graph.update_status("goal-a", GoalStatus::Converged).unwrap();

    // Stage 2: B ready, C still blocked
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-b");

    // Mark B as converged
    graph.update_status("goal-b", GoalStatus::Converged).unwrap();

    // Stage 3: C ready
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-c");

    // Mark C as converged
    graph.update_status("goal-c", GoalStatus::Converged).unwrap();

    // All complete - no ready goals
    let ready = graph.get_ready_goals();
    assert!(ready.is_empty());

    // Summary should show all converged
    let summary = graph.summary();
    assert_eq!(summary.converged, 3);
    assert_eq!(summary.total, 3);
}

#[test]
fn test_chain_blocked_on_failure() {
    let graph = GoalGraph::new();

    let a = create_goal_with_dep("goal-a", "First", vec![]);
    let b = create_goal_with_dep("goal-b", "Second", vec!["goal-a".to_string()]);
    let c = create_goal_with_dep("goal-c", "Third", vec!["goal-b".to_string()]);

    graph.submit(a).unwrap();
    graph.submit(b).unwrap();
    graph.submit(c).unwrap();

    // A fails
    graph.update_status("goal-a", GoalStatus::Failed { reason: "Budget exhausted".to_string() }).unwrap();

    // B and C should never become ready
    let ready = graph.get_ready_goals();
    assert!(ready.is_empty());

    // Even if we try to mark B as converged manually, C still depends on B
    // which depends on failed A - the chain is broken
    graph.update_status("goal-b", GoalStatus::Converged).unwrap();
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1); // C is now ready because B converged
    assert_eq!(ready[0].id, "goal-c");
}