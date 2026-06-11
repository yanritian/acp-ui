//! Goal System Integration Tests
//!
//! Tests for Goal-Driven Architecture (RFC-001)
//! - GoalGraph: add_goal, remove_goal, ready_goals, topological_order
//! - Goal lifecycle: submit → active → converged/failed

use acp_ui_lib::{Goal, GoalGraph, GoalStatus, CompletionCondition, Evaluator};

/// Helper to create a simple goal
fn create_goal(id: &str, description: &str) -> Goal {
    Goal::new(
        id.to_string(),
        description.to_string(),
        CompletionCondition::OutputContains {
            text: "done".to_string(),
            case_sensitive: false,
        },
    )
}

#[test]
fn test_goal_graph_submit() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Test goal 1");

    assert!(graph.submit(goal).is_ok());
    assert!(graph.get("g1").is_some());
}

#[test]
fn test_goal_graph_duplicate_id() {
    let graph = GoalGraph::new();
    let goal1 = create_goal("g1", "First goal");
    let goal2 = create_goal("g1", "Duplicate goal");

    assert!(graph.submit(goal1).is_ok());
    assert!(graph.submit(goal2).is_err());
}

#[test]
fn test_goal_graph_dependency_missing() {
    let graph = GoalGraph::new();
    let mut goal = create_goal("g2", "Goal with missing dependency");
    goal.dependencies.push("missing_dep".to_string());

    let result = graph.submit(goal);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Dependency 'missing_dep' not found"));
}

#[test]
fn test_goal_graph_ready_goals() {
    let graph = GoalGraph::new();

    // Submit parent goal first
    let parent = create_goal("parent", "Parent goal");
    graph.submit(parent).unwrap();

    // Submit child with dependency
    let mut child = create_goal("child", "Child goal");
    child.dependencies.push("parent".to_string());
    graph.submit(child).unwrap();

    // Only parent should be ready (no dependencies)
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "parent");

    // Mark parent as converged
    graph.update_status("parent", GoalStatus::Converged).unwrap();

    // Now child should be ready
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "child");
}

#[test]
fn test_goal_graph_cancel() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Test goal");
    graph.submit(goal).unwrap();

    // Cancel should work
    assert!(graph.cancel("g1").is_ok());

    let cancelled = graph.get("g1").unwrap();
    assert_eq!(cancelled.status, GoalStatus::Cancelled);
}

#[test]
fn test_goal_graph_summary() {
    let graph = GoalGraph::new();

    // Add multiple goals with different statuses
    let g1 = create_goal("g1", "Pending");
    graph.submit(g1).unwrap();

    let mut g2 = create_goal("g2", "Active");
    g2.status = GoalStatus::Active;
    graph.submit(g2).unwrap();

    let mut g3 = create_goal("g3", "Converged");
    g3.status = GoalStatus::Converged;
    graph.submit(g3).unwrap();

    let summary = graph.summary();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.pending, 1);
    assert_eq!(summary.active, 1);
    assert_eq!(summary.converged, 1);
}

#[test]
fn test_goal_assign_worker() {
    let graph = GoalGraph::new();
    let goal = create_goal("g1", "Test goal");
    graph.submit(goal).unwrap();

    // Assign worker
    assert!(graph.assign_worker("g1", "worker_123".to_string()).is_ok());

    let assigned = graph.get("g1").unwrap();
    assert_eq!(assigned.assigned_worker, Some("worker_123".to_string()));
    assert_eq!(assigned.status, GoalStatus::Active);
    assert!(assigned.started_at.is_some());
}

#[test]
fn test_goal_max_iterations() {
    let goal = create_goal("g1", "Test goal");

    // Default max iterations is 5
    assert_eq!(goal.max_iterations, 5);

    // Custom max iterations
    let goal_with_custom = goal.with_max_iterations(10);
    assert_eq!(goal_with_custom.max_iterations, 10);
}

#[test]
fn test_goal_budget_tracking() {
    let goal = create_goal("g1", "Test goal");

    // No budget by default
    assert!(!goal.is_budget_exhausted());

    // With budget
    let goal_with_budget = goal.with_token_budget(1000);
    assert!(!goal_with_budget.is_budget_exhausted());

    // Simulate token usage
    let mut used_goal = goal_with_budget;
    used_goal.token_used = 1000;
    assert!(used_goal.is_budget_exhausted());
}