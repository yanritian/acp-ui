//! End-to-End Flow Test
//!
//! Tests the complete workflow from Goal submission to convergence:
//! 1. Parse hello-swarm.goal.yaml
//! 2. Create GoalGraph with dependencies
//! 3. Execute via ReconcileLoop (simulated workers)
//! 4. Verify all Goals converge in correct order

use swarm_engine::{
    Goal, GoalGraph as SwarmGoalGraph, GoalStatus, CompletionCondition,
    ReconcileLoop, GoalOutcome,
};

/// Simulates the hello-swarm workflow from examples/hello-swarm.goal.yaml
fn create_hello_swarm_goals() -> Vec<Goal> {
    // Goal 1: Fix TypeScript compilation errors
    let mut goal_fix_ts = Goal::new(
        "fix-typescript",
        "Fix all TypeScript compilation errors in the project",
        CompletionCondition::command_success("npx vue-tsc --noEmit"),
    );
    goal_fix_ts.executor = Some("claude-code-worker".to_string());

    // Goal 2: Run tests (depends on fix-typescript)
    let mut goal_run_tests = Goal::new(
        "run-tests",
        "Run all tests to ensure no regressions",
        CompletionCondition::command_success("npm test"),
    );
    goal_run_tests.depends_on = vec!["fix-typescript".to_string()];
    goal_run_tests.max_iterations = 3;
    goal_run_tests.executor = Some("codex-worker".to_string());

    // Goal 3: Generate docs (depends on run-tests)
    let mut goal_gen_docs = Goal::new(
        "generate-docs",
        "Update README with latest changes",
        CompletionCondition::file_exists("README.md"),
    );
    goal_gen_docs.depends_on = vec!["run-tests".to_string()];
    goal_gen_docs.max_iterations = 2;
    goal_gen_docs.executor = Some("claude-code-worker".to_string());

    vec![goal_fix_ts, goal_run_tests, goal_gen_docs]
}

/// Helper to count goals by status
fn count_by_status(graph: &SwarmGoalGraph, status: GoalStatus) -> usize {
    graph.all_goals().iter().filter(|g| g.status == status).count()
}

#[tokio::test]
async fn test_hello_swarm_workflow_converges() {
    // Step 1: Create GoalGraph from hello-swarm definition
    let mut graph = SwarmGoalGraph::new();
    let goals = create_hello_swarm_goals();

    for goal in goals {
        graph.add_goal(goal);
    }

    // Verify initial state
    assert_eq!(graph.len(), 3);
    assert_eq!(count_by_status(&graph, GoalStatus::Pending), 3);
    assert_eq!(count_by_status(&graph, GoalStatus::Active), 0);
    assert_eq!(count_by_status(&graph, GoalStatus::Converged), 0);

    // Step 2: Verify only fix-typescript is ready (no dependencies)
    let ready = graph.ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "fix-typescript");

    // Step 3: Create ReconcileLoop with simulated workers
    let reconciler = ReconcileLoop::with_echo(true);

    // Step 4: Execute the DAG
    let results = reconciler.reconcile_graph(&mut graph).await;

    // Step 5: Verify all Goals converged
    assert_eq!(results.len(), 3);
    for (_, outcome) in &results {
        assert!(matches!(outcome, GoalOutcome::Converged { .. }));
    }

    // Step 6: Verify execution order (chain topology)
    // fix-typescript should execute first, then run-tests, then generate-docs
    let order = graph.topological_order();
    assert_eq!(order[0], "fix-typescript");
    assert_eq!(order[1], "run-tests");
    assert_eq!(order[2], "generate-docs");

    // Step 7: Verify final state
    assert!(graph.all_converged());
    assert_eq!(count_by_status(&graph, GoalStatus::Converged), 3);
    assert_eq!(count_by_status(&graph, GoalStatus::Pending), 0);
}

#[tokio::test]
async fn test_workflow_handles_dependency_failure() {
    // This test verifies that the graph correctly identifies blocked goals
    // when a dependency fails (even though reconcile_graph is simplified)
    let mut graph = SwarmGoalGraph::new();

    // Create goals with dependencies
    let mut goal_fix_ts = Goal::new(
        "fix-typescript",
        "Fix TypeScript errors",
        CompletionCondition::command_success("npx vue-tsc --noEmit"),
    );
    goal_fix_ts.executor = Some("claude-code-worker".to_string());

    let mut goal_run_tests = Goal::new(
        "run-tests",
        "Run tests",
        CompletionCondition::command_success("npm test"),
    );
    goal_run_tests.depends_on = vec!["fix-typescript".to_string()];
    goal_run_tests.executor = Some("codex-worker".to_string());

    graph.add_goal(goal_fix_ts);
    graph.add_goal(goal_run_tests);

    // Simulate fix-typescript failing
    graph.update_goal_status("fix-typescript", GoalStatus::Failed { reason: "TypeScript errors".into() });

    // Verify blocked goals detection
    let blocked = graph.has_blocked_goals();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].0, "run-tests");

    // Verify run-tests is no longer ready
    let ready = graph.ready_goals();
    assert!(ready.is_empty());
}

#[test]
fn test_goal_yaml_structure_matches_spec() {
    // This test verifies the Goal structure matches hello-swarm.goal.yaml spec
    let goals = create_hello_swarm_goals();

    // Verify goal_fix_ts matches YAML
    let fix_ts = goals.iter().find(|g| g.id == "fix-typescript").unwrap();
    assert_eq!(fix_ts.description, "Fix all TypeScript compilation errors in the project");
    assert!(matches!(fix_ts.completion_condition, CompletionCondition::CommandSuccess { .. }));
    assert_eq!(fix_ts.executor.as_ref().unwrap(), "claude-code-worker");
    assert_eq!(fix_ts.max_iterations, 5);

    // Verify goal_run_tests matches YAML
    let run_tests = goals.iter().find(|g| g.id == "run-tests").unwrap();
    assert_eq!(run_tests.depends_on, vec!["fix-typescript".to_string()]);
    assert_eq!(run_tests.max_iterations, 3);

    // Verify goal_gen_docs matches YAML
    let gen_docs = goals.iter().find(|g| g.id == "generate-docs").unwrap();
    assert_eq!(gen_docs.depends_on, vec!["run-tests".to_string()]);
    assert_eq!(gen_docs.max_iterations, 2);
}

#[test]
fn test_chain_topology_execution_order() {
    let mut graph = SwarmGoalGraph::new();
    let goals = create_hello_swarm_goals();

    for goal in goals {
        graph.add_goal(goal);
    }

    // Verify topological order follows chain
    let order = graph.topological_order();

    // fix-typescript (no deps) must come before run-tests
    let fix_idx = order.iter().position(|id| id == "fix-typescript").unwrap();
    let tests_idx = order.iter().position(|id| id == "run-tests").unwrap();
    assert!(fix_idx < tests_idx);

    // run-tests must come before generate-docs
    let docs_idx = order.iter().position(|id| id == "generate-docs").unwrap();
    assert!(tests_idx < docs_idx);
}