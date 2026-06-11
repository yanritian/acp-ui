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