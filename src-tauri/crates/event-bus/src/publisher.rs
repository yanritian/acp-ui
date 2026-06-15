//! Event Publisher - Publish events to the bus

use acp_core::AcpEvent;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("Failed to deliver event: {0}")]
    DeliveryFailed(String),

    #[error("Queue overflow")]
    QueueOverflow,
}

/// Event publisher for publishing events to topics
pub struct EventPublisher {
    /// Default topic prefix
    default_prefix: Option<String>,
}

impl EventPublisher {
    pub fn new() -> Self {
        Self { default_prefix: None }
    }

    pub fn with_default_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.default_prefix = Some(prefix.into());
        self
    }

    /// Build topic name with prefix
    pub fn build_topic(&self, topic: &str) -> String {
        match &self.default_prefix {
            Some(prefix) => format!("{}.{}", prefix, topic),
            None => topic.to_string(),
        }
    }

    /// Create GoalCreated event
    pub fn goal_created(goal_id: &str, description: &str) -> AcpEvent {
        AcpEvent::new("goal.created")
            .with_goal(goal_id)
            .with_data(json!({"description": description}))
    }

    /// Create GoalStatusChanged event
    pub fn goal_status_changed(goal_id: &str, old_status: &str, new_status: &str) -> AcpEvent {
        AcpEvent::new("goal.status_changed")
            .with_goal(goal_id)
            .with_data(json!({"old_status": old_status, "new_status": new_status}))
    }

    /// Create GoalConverged event
    pub fn goal_converged(goal_id: &str, iterations: u32) -> AcpEvent {
        AcpEvent::new("goal.converged")
            .with_goal(goal_id)
            .with_data(json!({"iterations": iterations}))
    }

    /// Create GoalFailed event
    pub fn goal_failed(goal_id: &str, reason: &str) -> AcpEvent {
        AcpEvent::new("goal.failed")
            .with_goal(goal_id)
            .with_data(json!({"reason": reason}))
    }

    /// Create WorkerRegistered event
    pub fn worker_registered(worker_id: &str, worker_type: &str) -> AcpEvent {
        AcpEvent::new("worker.registered")
            .with_worker(worker_id)
            .with_data(json!({"worker_type": worker_type}))
    }

    /// Create WorkerHealthUpdate event
    pub fn worker_health_update(worker_id: &str, health: &str) -> AcpEvent {
        AcpEvent::new("worker.health_update")
            .with_worker(worker_id)
            .with_data(json!({"health": health}))
    }

    /// Create TopologyChanged event
    pub fn topology_changed(topology_type: &str, goal_count: usize) -> AcpEvent {
        AcpEvent::new("topology.changed")
            .with_data(json!({"topology_type": topology_type, "goal_count": goal_count}))
    }

    /// Create QueenElected event
    pub fn queen_elected(queen_id: &str, ttl_seconds: u64) -> AcpEvent {
        AcpEvent::new("queen.elected")
            .with_worker(queen_id)
            .with_data(json!({"ttl_seconds": ttl_seconds}))
    }
}

impl Default for EventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publisher_new() {
        let publisher = EventPublisher::new();
        assert!(publisher.default_prefix.is_none());
    }

    #[test]
    fn publisher_with_prefix() {
        let publisher = EventPublisher::new().with_default_prefix("acp");
        assert_eq!(publisher.build_topic("goal"), "acp.goal");
    }

    #[test]
    fn publisher_build_topic_no_prefix() {
        let publisher = EventPublisher::new();
        assert_eq!(publisher.build_topic("goal.status"), "goal.status");
    }

    #[test]
    fn publisher_goal_created() {
        let event = EventPublisher::goal_created("goal-001", "Test goal");
        assert_eq!(event.event_type, "goal.created");
        assert_eq!(event.goal_id, Some("goal-001".to_string()));
    }

    #[test]
    fn publisher_goal_status_changed() {
        let event = EventPublisher::goal_status_changed("goal-001", "pending", "active");
        assert_eq!(event.event_type, "goal.status_changed");
        assert_eq!(event.goal_id, Some("goal-001".to_string()));
    }

    #[test]
    fn publisher_worker_registered() {
        let event = EventPublisher::worker_registered("worker-001", "claude_code");
        assert_eq!(event.event_type, "worker.registered");
        assert_eq!(event.worker_id, Some("worker-001".to_string()));
    }

    #[test]
    fn publisher_worker_health_update() {
        let event = EventPublisher::worker_health_update("worker-001", "healthy");
        assert_eq!(event.event_type, "worker.health_update");
        assert_eq!(event.worker_id, Some("worker-001".to_string()));
    }

    #[test]
    fn publisher_topology_changed() {
        let event = EventPublisher::topology_changed("star", 5);
        assert_eq!(event.event_type, "topology.changed");
    }

    #[test]
    fn publisher_queen_elected() {
        let event = EventPublisher::queen_elected("queen-001", 30);
        assert_eq!(event.event_type, "queen.elected");
        assert_eq!(event.worker_id, Some("queen-001".to_string()));
    }
}