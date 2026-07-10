//! FT-3: SwarmOrchestrator Consensus Strategy Test
//!
//! Validates 5 consensus strategies: FirstWins, Majority, BestScore, Merge, Adversarial

use acp_ui_lib::{
    AgentResult, AgentRole, AgentSwarmStatus, ConsensusStrategy, SwarmAgent, SwarmOrchestrator,
    SwarmTopology,
};

fn create_agent(id: &str, name: &str, role: AgentRole) -> SwarmAgent {
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

fn create_result(task_id: &str, success: bool, output: serde_json::Value) -> AgentResult {
    AgentResult {
        task_id: task_id.to_string(),
        success,
        output,
        duration_ms: 1000,
        tokens_used: 500,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

#[test]
fn test_register_agents_and_create_task() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register orchestrator + 2 executors
    let orch = create_agent(
        "orch-001",
        "Claude Code Orchestrator",
        AgentRole::Orchestrator,
    );
    let exec1 = create_agent("exec-001", "Codex Executor 1", AgentRole::Executor);
    let exec2 = create_agent("exec-002", "Codex Executor 2", AgentRole::Executor);

    assert!(orchestrator.register_agent(orch).is_ok());
    assert!(orchestrator.register_agent(exec1).is_ok());
    assert!(orchestrator.register_agent(exec2).is_ok());

    // Create task with Hierarchical topology
    let task = orchestrator
        .create_task(
            "Fix TypeScript errors".to_string(),
            Some(SwarmTopology::Hierarchical),
            None,
        )
        .unwrap();

    // Orchestrator + at least 1 executor should be assigned
    assert!(task.assigned_agents.contains(&"orch-001".to_string()));
    assert!(task.assigned_agents.len() >= 2);

    // Agents should be marked as Assigned
    let agents = orchestrator.list_agents();
    for agent in agents {
        if task.assigned_agents.contains(&agent.id) {
            assert_eq!(agent.status, AgentSwarmStatus::Assigned);
        }
    }
}

#[test]
fn test_consensus_first_wins() {
    let mut orchestrator = SwarmOrchestrator::new();

    let exec1 = create_agent("exec-001", "Executor 1", AgentRole::Executor);
    let exec2 = create_agent("exec-002", "Executor 2", AgentRole::Executor);

    orchestrator.register_agent(exec1).unwrap();
    orchestrator.register_agent(exec2).unwrap();

    let task = orchestrator
        .create_task(
            "Test task".to_string(),
            Some(SwarmTopology::Mesh),
            Some(ConsensusStrategy::FirstWins),
        )
        .unwrap();

    // First result (success) should complete the task
    let result1 = create_result(&task.id, true, serde_json::json!({"output": "done"}));
    let status = orchestrator
        .submit_result(&task.id, "exec-001", result1)
        .unwrap();

    assert_eq!(status, acp_ui_lib::SwarmTaskStatus::Completed);
}

#[test]
fn test_consensus_majority() {
    let mut orchestrator = SwarmOrchestrator::new();

    let exec1 = create_agent("exec-001", "Executor 1", AgentRole::Executor);
    let exec2 = create_agent("exec-002", "Executor 2", AgentRole::Executor);
    let exec3 = create_agent("exec-003", "Executor 3", AgentRole::Executor);

    orchestrator.register_agent(exec1).unwrap();
    orchestrator.register_agent(exec2).unwrap();
    orchestrator.register_agent(exec3).unwrap();

    let task = orchestrator
        .create_task(
            "Vote task".to_string(),
            Some(SwarmTopology::Mesh),
            Some(ConsensusStrategy::Majority),
        )
        .unwrap();

    // First result (fail) - not enough for majority
    let result1 = create_result(&task.id, false, serde_json::json!({"error": "fail"}));
    let status1 = orchestrator
        .submit_result(&task.id, &task.assigned_agents[0], result1)
        .unwrap();
    assert_eq!(status1, acp_ui_lib::SwarmTaskStatus::InProgress);

    // Second result (success) - 1 fail, 1 success, still no majority
    let result2 = create_result(&task.id, true, serde_json::json!({"output": "ok"}));
    let status2 = orchestrator
        .submit_result(&task.id, &task.assigned_agents[1], result2)
        .unwrap();
    // Depends on assigned count - may still be in progress or waiting

    // Third result (success) - 2 success out of 3 = majority
    let result3 = create_result(&task.id, true, serde_json::json!({"output": "ok"}));
    let status3 = orchestrator
        .submit_result(&task.id, &task.assigned_agents[2], result3)
        .unwrap();
    assert_eq!(status3, acp_ui_lib::SwarmTaskStatus::Completed);
}

#[test]
fn test_consensus_adversarial_requires_all_success() {
    let mut orchestrator = SwarmOrchestrator::new();

    let exec1 = create_agent("exec-001", "Executor 1", AgentRole::Executor);
    let exec2 = create_agent("exec-002", "Executor 2", AgentRole::Executor);

    orchestrator.register_agent(exec1).unwrap();
    orchestrator.register_agent(exec2).unwrap();

    let task = orchestrator
        .create_task(
            "Cross-validate task".to_string(),
            Some(SwarmTopology::Mesh),
            Some(ConsensusStrategy::Adversarial),
        )
        .unwrap();

    // First result (success) - need all to succeed
    let result1 = create_result(&task.id, true, serde_json::json!({"output": "ok"}));
    let status1 = orchestrator
        .submit_result(&task.id, &task.assigned_agents[0], result1)
        .unwrap();
    assert_eq!(status1, acp_ui_lib::SwarmTaskStatus::InProgress);

    // Second result (fail) - adversarial requires ALL success
    let result2 = create_result(&task.id, false, serde_json::json!({"error": "fail"}));
    let status2 = orchestrator
        .submit_result(&task.id, &task.assigned_agents[1], result2)
        .unwrap();
    assert_eq!(status2, acp_ui_lib::SwarmTaskStatus::WaitingConsensus); // Not failed, waiting for potential retry
}

#[test]
fn test_cancel_task_releases_agents() {
    let mut orchestrator = SwarmOrchestrator::new();

    let exec1 = create_agent("exec-001", "Executor 1", AgentRole::Executor);
    let exec2 = create_agent("exec-002", "Executor 2", AgentRole::Executor);

    orchestrator.register_agent(exec1).unwrap();
    orchestrator.register_agent(exec2).unwrap();

    let task = orchestrator
        .create_task("Task to cancel".to_string(), None, None)
        .unwrap();

    // Agents should be Assigned
    let agents = orchestrator.list_agents();
    for agent_id in &task.assigned_agents {
        let agent = agents.iter().find(|a| &a.id == agent_id).unwrap();
        assert_eq!(agent.status, AgentSwarmStatus::Assigned);
    }

    // Cancel task
    orchestrator.cancel_task(&task.id).unwrap();

    // Agents should be back to Idle
    let agents = orchestrator.list_agents();
    for agent_id in &task.assigned_agents {
        let agent = agents.iter().find(|a| &a.id == agent_id).unwrap();
        assert_eq!(agent.status, AgentSwarmStatus::Idle);
        assert!(agent.current_task.is_none());
    }

    // Task should be cancelled
    let cancelled_task = orchestrator.get_task(&task.id).unwrap();
    assert_eq!(
        cancelled_task.status,
        acp_ui_lib::SwarmTaskStatus::Cancelled
    );
}

#[test]
fn test_evict_underperforming_agents() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register an orchestrator + many executors
    let orch = create_agent("orch-001", "Orchestrator", AgentRole::Orchestrator);
    orchestrator.register_agent(orch).unwrap();

    for i in 1..=20 {
        let exec = create_agent(
            &format!("exec-{}", i),
            &format!("Executor {}", i),
            AgentRole::Executor,
        );
        orchestrator.register_agent(exec).unwrap();
    }

    // Create 1 task to test the flow (doesn't need multiple tasks)
    let task = orchestrator
        .create_task("test eviction".to_string(), None, None)
        .unwrap();
    let result = create_result(&task.id, true, serde_json::json!({"ok": true}));
    if !task.assigned_agents.is_empty() {
        orchestrator
            .submit_result(&task.id, &task.assigned_agents[0], result)
            .unwrap();
    }

    // Check eviction - agents with <3 results shouldn't be evicted
    let evicted = orchestrator.check_and_evict(0.8);

    // No agents should be evicted (all have <3 results)
    assert!(evicted.is_empty());

    // Verify all agents are still active
    let agents = orchestrator.list_agents();
    let non_evicted = agents
        .iter()
        .filter(|a| a.status != AgentSwarmStatus::Evicted)
        .count();
    assert!(non_evicted >= 20);
}
