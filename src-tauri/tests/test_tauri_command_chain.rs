//! Tauri Command Chain Test - Complete User Flow
//!
//! Tests the full chain from user action to result:
//! 1. User calls goal_submit → Goal stored in GoalGraph
//! 2. User calls goal_get_ready → Get executable goals
//! 3. User calls goal_assign_worker → Worker assigned
//! 4. Worker executes → goal_add_iteration
//! 5. User calls goal_update_status → Mark as Converged
//! 6. User calls goal_get_graph_summary → View progress

use acp_ui_lib::{
    AgentRole, AgentSwarmStatus, CompletionCondition, ConditionResult, ConsensusStrategy,
    EvaluationResult, Goal, GoalGraph, GoalStatus, IterationRecord, SwarmAgent, SwarmOrchestrator,
    SwarmTopology,
};

// ============================================================================
// Goal Command Chain Test
// ============================================================================

#[test]
fn test_goal_submit_to_converge_chain() {
    // === Step 1: User submits Goal ===
    let graph = GoalGraph::new();

    let goal = Goal::new(
        "user-task-001".to_string(),
        "Fix TypeScript compilation errors".to_string(),
        CompletionCondition::OutputContains {
            text: "Build successful".to_string(),
            case_sensitive: false,
        }
        .to_spec(),
    );

    // Simulate goal_submit Tauri command
    let submit_result = graph.submit(goal);
    assert!(submit_result.is_ok(), "goal_submit should succeed");

    // === Step 2: User checks status ===
    let stored = graph.get("user-task-001");
    assert!(stored.is_some(), "goal_get_status should return the goal");
    assert_eq!(stored.unwrap().status, GoalStatus::Pending);

    // === Step 3: User gets ready goals ===
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1, "goal_get_ready should return 1 goal");
    assert_eq!(ready[0].id, "user-task-001");

    // === Step 4: User assigns worker ===
    graph
        .assign_worker("user-task-001", "claude-code-worker".to_string())
        .unwrap();

    let assigned = graph.get("user-task-001").unwrap();
    assert_eq!(assigned.status, GoalStatus::Active);
    assert_eq!(assigned.executor, Some("claude-code-worker".to_string()));
    assert!(assigned.started_at.is_some());

    // === Step 5: Worker executes (iteration 1) ===
    let iteration1 = IterationRecord {
        iteration: 1,
        worker_output: "Fixed 3 TypeScript errors".to_string(),
        evaluation: EvaluationResult {
            passed: false,
            explanation: "Build still has errors".to_string(),
            details: vec![ConditionResult {
                description: "Output contains 'Build successful'".to_string(),
                passed: false,
                evidence: "Output: 'Fixed 3 TypeScript errors'".to_string(),
            }],
        },
        feedback: Some("Remaining error in utils.ts: type mismatch".to_string()),
        tokens_used: 5000,
        timestamp: 1000,
        duration_ms: 30000,
    };
    graph.add_iteration("user-task-001", iteration1).unwrap();

    // === Step 6: User checks iteration count ===
    let goal_with_iteration = graph.get("user-task-001").unwrap();
    assert_eq!(goal_with_iteration.iteration_log.len(), 1);

    // === Step 7: Worker executes (iteration 2 - success) ===
    let iteration2 = IterationRecord {
        iteration: 2,
        worker_output: "Build successful! All tests pass".to_string(),
        evaluation: EvaluationResult {
            passed: true,
            explanation: "Build completed successfully".to_string(),
            details: vec![ConditionResult {
                description: "Output contains 'Build successful'".to_string(),
                passed: true,
                evidence: "Found 'Build successful' in output".to_string(),
            }],
        },
        feedback: None,
        tokens_used: 3000,
        timestamp: 2000,
        duration_ms: 20000,
    };
    graph.add_iteration("user-task-001", iteration2).unwrap();

    // === Step 8: User marks goal as converged ===
    graph
        .update_status("user-task-001", GoalStatus::Converged)
        .unwrap();

    // === Step 9: User checks final status ===
    let final_goal = graph.get("user-task-001").unwrap();
    assert_eq!(final_goal.status, GoalStatus::Converged);
    // Note: converged_at is not automatically set by update_status, only by the goal itself
    assert_eq!(final_goal.iteration_log.len(), 2);

    // === Step 10: User views summary ===
    let summary = graph.summary();
    assert_eq!(summary.total, 1);
    assert_eq!(summary.converged, 1);
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.active, 0);
}

#[test]
fn test_goal_chain_with_dependencies() {
    let graph = GoalGraph::new();

    // === User submits 3 dependent goals ===
    let goal_a = Goal::new(
        "goal-a".to_string(),
        "Write unit tests".to_string(),
        CompletionCondition::OutputContains {
            text: "Tests written".to_string(),
            case_sensitive: false,
        }
        .to_spec(),
    );
    graph.submit(goal_a).unwrap();

    let mut goal_b = Goal::new(
        "goal-b".to_string(),
        "Run tests".to_string(),
        CompletionCondition::OutputContains {
            text: "Tests pass".to_string(),
            case_sensitive: false,
        }
        .to_spec(),
    );
    goal_b.depends_on.push("goal-a".to_string());
    graph.submit(goal_b).unwrap();

    let mut goal_c = Goal::new(
        "goal-c".to_string(),
        "Generate coverage report".to_string(),
        CompletionCondition::OutputContains {
            text: "Coverage report generated".to_string(),
            case_sensitive: false,
        }
        .to_spec(),
    );
    goal_c.depends_on.push("goal-b".to_string());
    graph.submit(goal_c).unwrap();

    // === Chain: Only A is ready initially ===
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-a");

    // === User executes A ===
    graph
        .assign_worker("goal-a", "worker-1".to_string())
        .unwrap();
    graph
        .add_iteration("goal-a", create_success_iteration(1, "Tests written"))
        .unwrap();
    graph
        .update_status("goal-a", GoalStatus::Converged)
        .unwrap();

    // === Now B is ready ===
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-b");

    // === User executes B ===
    graph
        .assign_worker("goal-b", "worker-2".to_string())
        .unwrap();
    graph
        .add_iteration("goal-b", create_success_iteration(1, "Tests pass"))
        .unwrap();
    graph
        .update_status("goal-b", GoalStatus::Converged)
        .unwrap();

    // === Now C is ready ===
    let ready = graph.get_ready_goals();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "goal-c");

    // === User executes C ===
    graph
        .assign_worker("goal-c", "worker-3".to_string())
        .unwrap();
    graph
        .add_iteration(
            "goal-c",
            create_success_iteration(1, "Coverage report generated"),
        )
        .unwrap();
    graph
        .update_status("goal-c", GoalStatus::Converged)
        .unwrap();

    // === Final: All converged ===
    let summary = graph.summary();
    assert_eq!(summary.converged, 3);
    assert!(graph.get_ready_goals().is_empty());
}

fn create_success_iteration(iteration: u32, output: &str) -> IterationRecord {
    IterationRecord {
        iteration,
        worker_output: output.to_string(),
        evaluation: EvaluationResult {
            passed: true,
            explanation: "Success".to_string(),
            details: vec![],
        },
        feedback: None,
        tokens_used: 1000,
        timestamp: iteration as u64 * 1000,
        duration_ms: 10000,
    }
}

// ============================================================================
// Swarm Command Chain Test
// ============================================================================

#[test]
fn test_swarm_task_creation_to_completion_chain() {
    let mut orchestrator = SwarmOrchestrator::new();

    // === Step 1: User registers agents ===
    let orch = create_test_agent("orch-001", "Orchestrator", AgentRole::Orchestrator);
    let exec1 = create_test_agent("exec-001", "Executor 1", AgentRole::Executor);
    let exec2 = create_test_agent("exec-002", "Executor 2", AgentRole::Executor);

    orchestrator.register_agent(orch).unwrap();
    orchestrator.register_agent(exec1).unwrap();
    orchestrator.register_agent(exec2).unwrap();

    // === Step 2: User creates task ===
    let task = orchestrator
        .create_task(
            "Fix all TypeScript errors".to_string(),
            Some(SwarmTopology::Hierarchical),
            Some(ConsensusStrategy::FirstWins),
        )
        .unwrap();

    assert!(!task.id.is_empty());
    assert!(task.assigned_agents.len() >= 2);

    // === Step 3: User checks agent status ===
    let agents = orchestrator.list_agents();
    for agent in agents
        .iter()
        .filter(|a| task.assigned_agents.contains(&a.id))
    {
        assert_eq!(agent.status, AgentSwarmStatus::Assigned);
    }

    // === Step 4: Worker submits result ===
    let result = create_test_result(&task.id, true, "Fixed all errors");
    let status = orchestrator
        .submit_result(&task.id, &task.assigned_agents[0], result)
        .unwrap();

    // With FirstWins, first success should complete
    assert_eq!(status, acp_ui_lib::SwarmTaskStatus::Completed);

    // === Step 5: User checks task status ===
    let completed_task = orchestrator.get_task(&task.id).unwrap();
    assert_eq!(
        completed_task.status,
        acp_ui_lib::SwarmTaskStatus::Completed
    );
    assert!(completed_task.completed_at.is_some());

    // === Step 6: User checks health ===
    let health = orchestrator.get_health();
    assert_eq!(health.completed_tasks, 1);
    assert!(health.total_tokens_used > 0);
}

#[test]
fn test_swarm_cancel_and_retry_chain() {
    let mut orchestrator = SwarmOrchestrator::new();

    // === Setup: Register agents ===
    let orch = create_test_agent("orch-001", "Orchestrator", AgentRole::Orchestrator);
    let exec = create_test_agent("exec-001", "Executor", AgentRole::Executor);
    orchestrator.register_agent(orch).unwrap();
    orchestrator.register_agent(exec).unwrap();

    // === Step 1: Create task ===
    let task = orchestrator
        .create_task("Task to cancel".to_string(), None, None)
        .unwrap();

    // === Step 2: Cancel task ===
    orchestrator.cancel_task(&task.id).unwrap();

    // === Step 3: Verify agents released ===
    let agents = orchestrator.list_agents();
    for agent in agents {
        assert_eq!(agent.status, AgentSwarmStatus::Idle);
    }

    // === Step 4: Create new task (retry) ===
    let new_task = orchestrator
        .create_task("Retry task".to_string(), None, None)
        .unwrap();
    assert_ne!(new_task.id, task.id);

    // === Step 5: Complete new task ===
    let result = create_test_result(&new_task.id, true, "Success");
    orchestrator
        .submit_result(&new_task.id, &new_task.assigned_agents[0], result)
        .unwrap();

    let health = orchestrator.get_health();
    assert_eq!(health.completed_tasks, 1);
}

fn create_test_agent(id: &str, name: &str, role: AgentRole) -> SwarmAgent {
    SwarmAgent {
        id: id.to_string(),
        name: name.to_string(),
        role,
        capabilities: vec!["code-generation".to_string()],
        agent_config_name: "default".to_string(),
        status: AgentSwarmStatus::Idle,
        current_task: None,
        results: Vec::new(),
        token_budget: None,
        tokens_used: 0,
    }
}

fn create_test_result(task_id: &str, success: bool, output: &str) -> acp_ui_lib::AgentResult {
    acp_ui_lib::AgentResult {
        task_id: task_id.to_string(),
        success,
        output: serde_json::json!({ "output": output }),
        duration_ms: 1000,
        tokens_used: 500,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

// ============================================================================
// Goal Cancel Chain Test
// ============================================================================

#[test]
fn test_goal_cancel_chain() {
    let graph = GoalGraph::new();

    // === User submits goal ===
    let goal = Goal::new(
        "cancel-test".to_string(),
        "Task to cancel".to_string(),
        CompletionCondition::OutputContains {
            text: "done".to_string(),
            case_sensitive: false,
        }
        .to_spec(),
    );
    graph.submit(goal).unwrap();

    // === User assigns worker ===
    graph
        .assign_worker("cancel-test", "worker-001".to_string())
        .unwrap();
    assert_eq!(graph.get("cancel-test").unwrap().status, GoalStatus::Active);

    // === User cancels goal ===
    graph.cancel("cancel-test").unwrap();

    // === Verify cancelled state ===
    let cancelled = graph.get("cancel-test").unwrap();
    assert_eq!(cancelled.status, GoalStatus::Cancelled);

    // === Verify not in ready goals ===
    let ready = graph.get_ready_goals();
    assert!(ready.is_empty() || !ready.iter().any(|g| g.id == "cancel-test"));

    // === Verify summary shows cancelled ===
    let summary = graph.summary();
    assert_eq!(summary.cancelled, 1);
}
