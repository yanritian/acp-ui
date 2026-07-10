//! FT-5: ReconcileLoop + GoalGraph Integration Test
//!
//! Validates ReconcileLoop driving GoalGraph DAG execution

use std::sync::Arc;
use swarm_engine::{
    CompletionCondition, EchoExecutor, Goal, GoalGraph, GoalOutcome, GoalStatus, ReconcileLoop,
};
use tokio::sync::Mutex;

fn make_goal(id: &str, depends_on: Vec<String>) -> Goal {
    let mut goal = Goal::new(
        id,
        "Test goal",
        CompletionCondition::command_success("echo"),
    );
    goal.depends_on = depends_on;
    goal
}

#[tokio::test]
async fn test_reconcile_single_goal_converges() {
    let reconciler = ReconcileLoop::with_echo(true);
    let mut goal = make_goal("goal-a", vec![]);

    let outcome = reconciler.reconcile_goal(&mut goal).await;

    assert!(matches!(outcome, GoalOutcome::Converged { .. }));
    assert_eq!(goal.status, GoalStatus::Converged);
}

#[tokio::test]
async fn test_reconcile_graph_dag_order() {
    let reconciler = ReconcileLoop::with_echo(true);
    let mut graph = GoalGraph::new();

    // A → B → C chain
    graph.add_goal(make_goal("goal-a", vec![]));
    graph.add_goal(make_goal("goal-b", vec!["goal-a".to_string()]));
    graph.add_goal(make_goal("goal-c", vec!["goal-b".to_string()]));

    let results = reconciler.reconcile_graph(&mut graph).await;

    // All 3 goals should be executed
    assert_eq!(results.len(), 3);

    // All should converge
    for (_, outcome) in &results {
        assert!(matches!(outcome, GoalOutcome::Converged { .. }));
    }

    // Graph should show all converged
    assert!(graph.all_converged());
}

#[tokio::test]
async fn test_reconcile_budget_exhausted() {
    let reconciler = ReconcileLoop::with_echo(true);
    let mut goal = make_goal("limited-goal", vec![]);
    goal.token_budget = Some(0); // Zero budget forces immediate exhaustion

    let outcome = reconciler.reconcile_goal(&mut goal).await;

    assert!(matches!(outcome, GoalOutcome::BudgetExhausted { .. }));
}
