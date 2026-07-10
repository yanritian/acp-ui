//! FT-4: Chain Topology Pipeline Execution Test
//!
//! Validates Worker A → B → C sequential pipeline execution

use acp_ui_lib::{AgentRole, AgentSwarmStatus, SwarmAgent, SwarmOrchestrator};

fn create_worker(id: &str, name: &str) -> SwarmAgent {
    SwarmAgent {
        id: id.to_string(),
        name: name.to_string(),
        role: AgentRole::Executor,
        capabilities: vec!["code-generation".to_string()],
        agent_config_name: "default".to_string(),
        status: AgentSwarmStatus::Idle,
        current_task: None,
        results: Vec::new(),
        token_budget: None,
        tokens_used: 0,
    }
}

#[test]
fn test_chain_execute_assigns_workers_sequentially() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register 3 workers for pipeline
    let worker_a = create_worker("worker-a", "Stage 1 Worker");
    let worker_b = create_worker("worker-b", "Stage 2 Worker");
    let worker_c = create_worker("worker-c", "Stage 3 Worker");

    orchestrator.register_agent(worker_a).unwrap();
    orchestrator.register_agent(worker_b).unwrap();
    orchestrator.register_agent(worker_c).unwrap();

    // Execute chain
    let result = orchestrator
        .execute_chain(
            "Write a test suite for the new feature",
            &[
                "worker-a".to_string(),
                "worker-b".to_string(),
                "worker-c".to_string(),
            ],
        )
        .unwrap();

    // Should have 3 stages
    assert_eq!(result.total_stages, 3);
    assert_eq!(result.stages.len(), 3);

    // Stage 0 should use worker-a
    assert_eq!(result.stages[0].worker_id, "worker-a");
    assert_eq!(result.stages[0].stage_index, 0);

    // Stage 1 should use worker-b
    assert_eq!(result.stages[1].worker_id, "worker-b");
    assert_eq!(result.stages[1].stage_index, 1);

    // Stage 2 should use worker-c
    assert_eq!(result.stages[2].worker_id, "worker-c");
    assert_eq!(result.stages[2].stage_index, 2);

    // Final output should contain accumulated transformations
    assert!(!result.final_output.is_empty());
}

#[test]
fn test_chain_requires_idle_workers() {
    let mut orchestrator = SwarmOrchestrator::new();

    let worker_a = create_worker("worker-a", "Worker A");
    let worker_b = create_worker("worker-b", "Worker B");

    orchestrator.register_agent(worker_a).unwrap();
    orchestrator.register_agent(worker_b).unwrap();

    // Execute chain successfully
    let _ = orchestrator.execute_chain("Task 1", &["worker-a".to_string(), "worker-b".to_string()]);

    // Workers should be back to Idle after chain completes
    let agents = orchestrator.list_agents();
    for agent in agents {
        assert_eq!(agent.status, AgentSwarmStatus::Idle);
    }

    // Can execute another chain
    let result =
        orchestrator.execute_chain("Task 2", &["worker-a".to_string(), "worker-b".to_string()]);
    assert!(result.is_ok());
}

#[test]
fn test_chain_fails_with_missing_worker() {
    let mut orchestrator = SwarmOrchestrator::new();

    let worker_a = create_worker("worker-a", "Worker A");
    orchestrator.register_agent(worker_a).unwrap();

    // Try to execute chain with unregistered worker
    let result = orchestrator.execute_chain(
        "Task",
        &["worker-a".to_string(), "worker-nonexistent".to_string()],
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_chain_fails_with_busy_worker() {
    let mut orchestrator = SwarmOrchestrator::new();

    let worker_a = create_worker("worker-a", "Worker A");
    let worker_b = create_worker("worker-b", "Worker B");

    orchestrator.register_agent(worker_a).unwrap();
    orchestrator.register_agent(worker_b).unwrap();

    // Create a task that assigns worker_b
    let task = orchestrator
        .create_task("Busy task".to_string(), None, None)
        .unwrap();

    // If worker_b was assigned, try to use it in chain
    if task.assigned_agents.contains(&"worker-b".to_string()) {
        // Chain should fail because worker_b is not idle
        let result = orchestrator.execute_chain(
            "Chain task",
            &["worker-a".to_string(), "worker-b".to_string()],
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not idle"));
    }
}
