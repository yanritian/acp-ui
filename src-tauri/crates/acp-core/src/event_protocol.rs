//! Event Protocol - State change events

use serde::{Deserialize, Serialize};

/// ACP Event - State change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpEvent {
    /// Event type
    pub event_type: String,
    /// Related goal ID (if applicable)
    pub goal_id: Option<String>,
    /// Related worker ID (if applicable)
    pub worker_id: Option<String>,
    /// Timestamp
    pub timestamp: i64,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

impl AcpEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            goal_id: None,
            worker_id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            data: None,
        }
    }

    pub fn with_goal(mut self, goal_id: impl Into<String>) -> Self {
        self.goal_id = Some(goal_id.into());
        self
    }

    pub fn with_worker(mut self, worker_id: impl Into<String>) -> Self {
        self.worker_id = Some(worker_id.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_event_new() {
        let event = AcpEvent::new("goal.submitted");

        assert_eq!(event.event_type, "goal.submitted");
        assert!(event.goal_id.is_none());
        assert!(event.worker_id.is_none());
        assert!(event.data.is_none());
        assert!(event.timestamp > 0);
    }

    #[test]
    fn acp_event_with_goal() {
        let event = AcpEvent::new("goal.converged").with_goal("goal-001");

        assert_eq!(event.goal_id, Some("goal-001".to_string()));
        assert!(event.worker_id.is_none());
    }

    #[test]
    fn acp_event_with_worker() {
        let event = AcpEvent::new("worker.registered").with_worker("worker-001");

        assert_eq!(event.worker_id, Some("worker-001".to_string()));
        assert!(event.goal_id.is_none());
    }

    #[test]
    fn acp_event_with_data() {
        let event = AcpEvent::new("goal.iterating")
            .with_goal("goal-001")
            .with_data(serde_json::json!({
                "iteration": 2,
                "feedback": "Still working on it"
            }));

        assert!(event.data.is_some());
        let data = event.data.unwrap();
        assert_eq!(data["iteration"], 2);
    }

    #[test]
    fn acp_event_serde_roundtrip() {
        let event = AcpEvent::new("queen.elected")
            .with_worker("worker-001")
            .with_data(serde_json::json!({"ttl_seconds": 30}));

        let json = serde_json::to_string(&event).unwrap();
        let decoded: AcpEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.event_type, event.event_type);
        assert_eq!(decoded.worker_id, event.worker_id);
        assert!(decoded.data.is_some());
    }

    #[test]
    fn acp_event_builder_chain() {
        let event = AcpEvent::new("test.event")
            .with_goal("g001")
            .with_worker("w001")
            .with_data(serde_json::json!({"key": "value"}));

        assert_eq!(event.goal_id, Some("g001".to_string()));
        assert_eq!(event.worker_id, Some("w001".to_string()));
        assert!(event.data.is_some());
    }
}