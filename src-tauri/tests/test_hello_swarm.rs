//! FT-HelloSwarm: hello-swarm.goal.yaml End-to-End Test
//!
//! Validates the complete flow:
//! 1. GoalYamlFile parsing from hello-swarm.goal.yaml
//! 2. GoalSpec → Goal conversion via from_runtime
//! 3. GoalGraph submission with dependencies
//! 4. Topological execution order verification

use acp_core::{GoalRuntime, GoalStatus, GoalYamlFile};
use acp_ui_lib::{Goal, GoalGraph};

fn load_hello_swarm_yaml() -> GoalYamlFile {
    let yaml_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/hello-swarm.goal.yaml"
    );
    let content = std::fs::read_to_string(yaml_path).expect("Read yaml file");
    serde_yaml::from_str(&content).expect("Parse yaml")
}

fn convert_spec_to_goal(spec: &acp_core::GoalSpec) -> Goal {
    let mut runtime = GoalRuntime {
        id: spec.id.clone(),
        description: spec.description.clone(),
        parent_task_id: None,
        parent_goal_id: None,
        completion_condition: spec.completion_condition.clone(),
        evaluator: spec.evaluator.clone(),
        executor: spec.executor.clone(),
        depends_on: spec.depends_on.clone(),
        token_budget: Some(spec.token_budget),
        tokens_used: 0,
        max_iterations: spec.max_iterations,
        current_iteration: 0,
        per_iteration_timeout_ms: spec.per_iteration_timeout_ms,
        status: GoalStatus::Pending,
        iteration_log: vec![],
        created_at: 0,
        started_at: None,
        converged_at: None,
        output_files: vec![],
    };
    runtime.executor = spec.executor.clone();
    runtime // GoalRuntime is re-exported as Goal
}

/// Parse hello-swarm.goal.yaml and verify structure
#[test]
fn test_hello_swarm_yaml_parse() {
    let parsed = load_hello_swarm_yaml();

    assert_eq!(parsed.name, "hello-swarm");
    assert_eq!(parsed.goals.len(), 3);
    assert_eq!(parsed.topology, "chain");
    assert_eq!(parsed.workers.len(), 2);
}

/// Verify Goal 1: fix-typescript structure
#[test]
fn test_hello_swarm_goal_fix_typescript() {
    let parsed = load_hello_swarm_yaml();
    let goal = parsed
        .goals
        .iter()
        .find(|g| g.id == "fix-typescript")
        .expect("Find goal");

    assert_eq!(goal.executor, Some("claude-code-worker".to_string()));
    assert!(goal.depends_on.is_empty());
    assert_eq!(goal.token_budget, 80000);
    assert_eq!(goal.max_iterations, 5);
}

/// Verify Goal 2: run-tests structure and dependency
#[test]
fn test_hello_swarm_goal_run_tests() {
    let parsed = load_hello_swarm_yaml();
    let goal = parsed
        .goals
        .iter()
        .find(|g| g.id == "run-tests")
        .expect("Find goal");

    assert_eq!(goal.executor, Some("codex-worker".to_string()));
    assert_eq!(goal.depends_on, vec!["fix-typescript"]);
    assert_eq!(goal.token_budget, 50000);
}

/// Verify Goal 3: generate-docs structure and dependency chain
#[test]
fn test_hello_swarm_goal_generate_docs() {
    let parsed = load_hello_swarm_yaml();
    let goal = parsed
        .goals
        .iter()
        .find(|g| g.id == "generate-docs")
        .expect("Find goal");

    assert_eq!(goal.executor, Some("claude-code-worker".to_string()));
    assert_eq!(goal.depends_on, vec!["run-tests"]);
    assert_eq!(goal.token_budget, 30000);
}

/// Verify GoalSpec → Goal conversion for all goals
#[test]
fn test_hello_swarm_goal_conversion() {
    let parsed = load_hello_swarm_yaml();

    for spec in &parsed.goals {
        let goal = convert_spec_to_goal(spec);
        assert_eq!(goal.id, spec.id);
        assert_eq!(goal.executor, spec.executor);
    }
}

/// Verify GoalGraph submission with dependency order
#[test]
fn test_hello_swarm_goal_graph_submission() {
    let parsed = load_hello_swarm_yaml();
    let graph = GoalGraph::new();

    // Submit all goals (order matters for dependencies)
    for spec in &parsed.goals {
        let goal = convert_spec_to_goal(spec);
        graph.submit(goal).expect("Submit goal");
    }

    // Verify graph state
    let summary = graph.summary();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.pending, 3);

    // Only fix-typescript should be ready (no dependencies)
    let ready = graph.get_ready_ids();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0], "fix-typescript");
}

/// Verify topological execution order (chain topology)
#[test]
fn test_hello_swarm_topological_order() {
    let parsed = load_hello_swarm_yaml();

    // Verify dependency chain: fix-typescript → run-tests → generate-docs
    let g1 = parsed
        .goals
        .iter()
        .find(|g| g.id == "fix-typescript")
        .expect("Goal 1");
    let g2 = parsed
        .goals
        .iter()
        .find(|g| g.id == "run-tests")
        .expect("Goal 2");
    let g3 = parsed
        .goals
        .iter()
        .find(|g| g.id == "generate-docs")
        .expect("Goal 3");

    assert!(g1.depends_on.is_empty());
    assert!(g2.depends_on.contains(&"fix-typescript".to_string()));
    assert!(g3.depends_on.contains(&"run-tests".to_string()));
    assert_eq!(parsed.topology, "chain");
}

/// Verify worker configuration from YAML
#[test]
fn test_hello_swarm_worker_config() {
    let parsed = load_hello_swarm_yaml();

    let cc_worker = parsed
        .workers
        .iter()
        .find(|w| w.id == "claude-code-worker")
        .expect("Worker");
    assert_eq!(cc_worker.worker_type, "claude_code");

    let codex_worker = parsed
        .workers
        .iter()
        .find(|w| w.id == "codex-worker")
        .expect("Worker");
    assert_eq!(codex_worker.worker_type, "codex");
}

/// Verify Queen configuration from YAML
#[test]
fn test_hello_swarm_queen_config() {
    let parsed = load_hello_swarm_yaml();
    let queen = parsed.queen.expect("Queen config");
    assert_eq!(queen.worker_id, "claude-code-worker");
}
