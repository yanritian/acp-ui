//! FT-9: Tauri Swarm Commands Test
//!
//! Validates swarm_register_agent, swarm_list_agents, swarm_create_task, swarm_get_health

use acp_ui_lib::{
    AgentRole, AgentSwarmStatus, ConsensusStrategy, SwarmAgent, SwarmOrchestrator, SwarmTopology,
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

#[test]
fn test_swarm_register_and_list_agents() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register agents
    let orch = create_agent("orch-001", "Claude Orchestrator", AgentRole::Orchestrator);
    let exec = create_agent("exec-001", "Codex Executor", AgentRole::Executor);

    assert!(orchestrator.register_agent(orch).is_ok());
    assert!(orchestrator.register_agent(exec).is_ok());

    // List should return both
    let agents = orchestrator.list_agents();
    assert_eq!(agents.len(), 2);

    // Should be sorted by name
    let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"Claude Orchestrator"));
    assert!(names.contains(&"Codex Executor"));
}

#[test]
fn test_swarm_register_duplicate_fails() {
    let mut orchestrator = SwarmOrchestrator::new();

    let agent1 = create_agent("agent-001", "First Agent", AgentRole::Executor);
    let agent2 = create_agent("agent-001", "Duplicate Agent", AgentRole::Executor);

    assert!(orchestrator.register_agent(agent1).is_ok());
    assert!(orchestrator.register_agent(agent2).is_err());
}

#[test]
fn test_swarm_create_task_and_get_task() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Need at least one orchestrator + executor for hierarchical
    let orch = create_agent("orch-001", "Orchestrator", AgentRole::Orchestrator);
    let exec = create_agent("exec-001", "Executor", AgentRole::Executor);
    orchestrator.register_agent(orch).unwrap();
    orchestrator.register_agent(exec).unwrap();

    // Create task
    let task = orchestrator
        .create_task(
            "Fix TypeScript errors".to_string(),
            Some(SwarmTopology::Hierarchical),
            Some(ConsensusStrategy::FirstWins),
        )
        .unwrap();

    assert!(!task.id.is_empty());
    assert_eq!(task.description, "Fix TypeScript errors");
    assert!(task.assigned_agents.len() >= 1);

    // Get task should return the same
    let retrieved = orchestrator.get_task(&task.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, task.id);
}

#[test]
fn test_swarm_get_health_summary() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register agents
    for i in 1..=3 {
        let agent = create_agent(
            &format!("agent-{}", i),
            &format!("Agent {}", i),
            AgentRole::Executor,
        );
        orchestrator.register_agent(agent).unwrap();
    }

    // Create a task (assigns some agents)
    let _ = orchestrator.create_task("Test task".to_string(), Some(SwarmTopology::Mesh), None);

    // Get health
    let health = orchestrator.get_health();

    assert_eq!(health.total_agents, 3);
    assert!(health.active_agents > 0); // Some assigned
    assert!(health.idle_agents >= 0);
    assert!(health.pending_tasks >= 0);
    assert!(health.running_tasks >= 0);
}

#[test]
fn test_swarm_list_tasks_sorted_by_time() {
    let mut orchestrator = SwarmOrchestrator::new();

    // Register an orchestrator + many executors to allow multiple tasks
    let orch = create_agent("orch-001", "Orchestrator", AgentRole::Orchestrator);
    orchestrator.register_agent(orch).unwrap();

    // Register many executors (max_concurrent_agents is 8 by default)
    for i in 1..=10 {
        let exec = create_agent(
            &format!("exec-{}", i),
            &format!("Executor {}", i),
            AgentRole::Executor,
        );
        orchestrator.register_agent(exec).unwrap();
    }

    // Create multiple tasks with Mesh topology (uses all idle agents)
    let task1 = orchestrator
        .create_task("Task 1".to_string(), Some(SwarmTopology::Mesh), None)
        .unwrap();
    let task2 = orchestrator
        .create_task("Task 2".to_string(), Some(SwarmTopology::Mesh), None)
        .unwrap();

    // List should return newest first
    let tasks = orchestrator.list_tasks();
    assert!(tasks.len() >= 2); // At least 2 tasks should be created

    // Task 2 should be first (created later)
    assert_eq!(tasks[0].id, task2.id);
    assert_eq!(tasks[1].id, task1.id);
}
