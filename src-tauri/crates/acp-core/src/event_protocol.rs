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