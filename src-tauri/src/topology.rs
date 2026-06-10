//! Swarm Topology — execution patterns for multi-Goal coordination
//!
//! Provides three topology patterns:
//! - **Star**: One Queen goal decomposes into sub-goals executed in parallel
//! - **Chain**: Goals execute sequentially, each receiving the previous output
//! - **Pipeline**: Combination — chain of star batches
//!
//! These topologies operate on `GoalGraph` and use `ReconcileLoop` for
//! individual goal execution.

use crate::goal::{Goal, GoalStatus, CompletionCondition, Evaluator};
use crate::goal_graph::GoalGraph;
use crate::reconcile::ReconcileLoop;
use crate::swarm_adapters::WorkerId;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Topology type for goal execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Topology {
    /// All sub-goals execute in parallel (no dependencies between them)
    Star,
    /// Goals execute sequentially (each depends on the previous)
    Chain,
    /// Sequential batches of parallel goals
    Pipeline,
}

/// Execute a set of goals using the Star topology.
///
/// All goals execute in parallel — no dependencies.
/// Returns when all goals reach a terminal state.
pub fn execute_star(
    goals: Vec<Goal>,
    reconcile: &ReconcileLoop,
) -> HashMap<String, GoalStatus> {
    let mut graph = GoalGraph::new();
    for goal in goals {
        let _ = graph.add_goal(goal);
    }

    run_graph(&mut graph, reconcile)
}

/// Execute a set of goals using the Chain topology.
///
/// Each goal depends on the previous one — sequential execution.
/// The output of each goal is fed as context to the next.
pub fn execute_chain(
    goals: Vec<Goal>,
    reconcile: &ReconcileLoop,
) -> HashMap<String, GoalStatus> {
    let mut graph = GoalGraph::new();
    let mut prev_id: Option<String> = None;

    for mut goal in goals {
        if let Some(ref prev) = prev_id {
            goal.dependencies.push(prev.clone());
        }
        let id = goal.id.clone();
        prev_id = Some(id.clone());
        let _ = graph.add_goal(goal);
    }

    run_graph(&mut graph, reconcile)
}

/// Execute a GoalGraph to completion.
///
/// This is the core execution engine:
/// 1. Find ready goals (dependencies all converged)
/// 2. Execute each ready goal through reconcile loop
/// 3. Check for blocked/deadlocked goals
/// 4. Repeat until all goals are terminal or blocked
fn run_graph(
    graph: &mut GoalGraph,
    reconcile: &ReconcileLoop,
) -> HashMap<String, GoalStatus> {
    let mut results: HashMap<String, GoalStatus> = HashMap::new();

    loop {
        // Check if all done
        if graph.all_goals().iter().all(|g| is_terminal(&g.status)) {
            break;
        }

        // Find blocked goals and mark them as failed
        let blocked = graph.blocked_goals();
        for blocked_id in &blocked {
            if let Some(goal) = graph.get_goal_mut(blocked_id) {
                goal.status = GoalStatus::Failed {
                    reason: "Dependency failed — goal is blocked".into(),
                };
            }
        }

        // Get ready goals
        let ready: Vec<String> = graph
            .ready_goals()
            .into_iter()
            .map(|g| g.id.clone())
            .collect();

        if ready.is_empty() {
            // No ready goals and not all done — deadlock or all finished
            break;
        }

        // Execute each ready goal
        for goal_id in &ready {
            // Take the goal out of the graph for mutation
            let mut goal = match graph.get_goal(goal_id).cloned() {
                Some(g) => g,
                None => continue,
            };

            // Run reconcile loop for this goal
            let status = reconcile.reconcile_until_done(&mut goal);

            // Put the updated goal back
            if let Some(g) = graph.get_goal_mut(goal_id) {
                *g = goal;
            }

            results.insert(goal_id.clone(), status);
        }
    }

    // Collect final statuses
    for goal in graph.all_goals() {
        results.entry(goal.id.clone()).or_insert_with(|| goal.status.clone());
    }

    results
}

/// Check if a goal status is terminal.
fn is_terminal(status: &GoalStatus) -> bool {
    matches!(
        status,
        GoalStatus::Converged
            | GoalStatus::Failed { .. }
            | GoalStatus::BudgetExhausted
            | GoalStatus::Cancelled
    )
}

/// Build a Star topology from a parent goal and sub-goal descriptions.
///
/// The parent goal is decomposed into N sub-goals that execute in parallel.
pub fn build_star_goals(
    parent_description: &str,
    sub_descriptions: Vec<(String, String, CompletionCondition)>,
    worker_assignments: HashMap<String, WorkerId>,
) -> Vec<Goal> {
    sub_descriptions
        .into_iter()
        .map(|(id, description, condition)| {
            let mut goal = Goal::new(id, description, condition);
            if let Some(worker_id) = worker_assignments.get(&goal.id) {
                goal = goal.with_worker(worker_id.clone());
            }
            goal
        })
        .collect()
}

/// Build a Chain topology from sequential step descriptions.
///
/// Each step depends on the previous step's output.
pub fn build_chain_goals(
    steps: Vec<(String, String, CompletionCondition)>,
    worker_assignments: HashMap<String, WorkerId>,
) -> Vec<Goal> {
    let mut goals: Vec<Goal> = Vec::new();

    for (i, (id, description, condition)) in steps.into_iter().enumerate() {
        let mut goal = Goal::new(id.clone(), description, condition);

        // Add dependency on previous step
        if i > 0 {
            if let Some(prev) = goals.last() {
                goal = goal.with_dependency(prev.id.clone());
            }
        }

        if let Some(worker_id) = worker_assignments.get(&id) {
            goal = goal.with_worker(worker_id.clone());
        }

        goals.push(goal);
    }

    goals
}
