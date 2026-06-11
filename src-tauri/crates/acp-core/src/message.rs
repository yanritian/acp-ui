//! ACP Message - Unified message format

use serde::{Deserialize, Serialize};

/// ACP Message - Unified message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    /// Message type
    pub message_type: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: i64,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl AcpMessage {
    pub fn new(message_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            message_type: message_type.into(),
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_message_new() {
        let msg = AcpMessage::new("goal.submitted", serde_json::json!({"goal_id": "g001"}));

        assert_eq!(msg.message_type, "goal.submitted");
        assert!(msg.timestamp > 0);
        assert!(msg.metadata.is_none());
    }

    #[test]
    fn acp_message_serde_roundtrip() {
        let msg = AcpMessage::new("worker.heartbeat", serde_json::json!({
            "worker_id": "w001",
            "status": "idle"
        }));

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: AcpMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.message_type, msg.message_type);
        assert_eq!(decoded.payload, msg.payload);
        assert_eq!(decoded.timestamp, msg.timestamp);
    }

    #[test]
    fn acp_message_with_metadata() {
        let mut msg = AcpMessage::new("test", serde_json::json!({}));
        msg.metadata = Some(serde_json::json!({"source": "dashboard"}));

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: AcpMessage = serde_json::from_str(&json).unwrap();

        assert!(decoded.metadata.is_some());
    }
}