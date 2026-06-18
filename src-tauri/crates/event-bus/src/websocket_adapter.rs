//! WebSocket Adapter - Push EventBus events to frontend
//!
//! This module provides an abstraction for pushing EventBus events
//! to frontend via WebSocket or Tauri events.
//!
//! ## Architecture
//! ```
//! EventBus.publish() → EventPusher.push() → Frontend (WebSocket/Tauri)
//! ```
//!
//! ## Usage
//!
//! The main app implements `EventPusher` trait with Tauri emit logic:
//!
//! ```rust,ignore
//! // In src-tauri/src/event_pusher.rs:
//! impl EventPusher for TauriEventPusher {
//!     fn push(&self, event: &AcpEvent) {
//!         self.app_handle.emit("swarm:event", event).ok();
//!     }
//! }
//!
//! // Connect to EventBus:
//! let pusher = Arc::new(TauriEventPusher::new(app_handle));
//! let sub = Subscriber::new_push_subscriber(pusher);
//! bus.subscribe("goal.*", sub);
//! ```

use acp_core::AcpEvent;
use std::sync::Arc;

/// Trait for pushing events to frontend
///
/// Implement this trait in the main app with Tauri emit logic.
pub trait EventPusher: Send + Sync {
    /// Push an event to frontend
    fn push(&self, event: &AcpEvent);
}

/// Create a subscriber that pushes events via an EventPusher
pub fn create_push_subscriber(pusher: Arc<dyn EventPusher>) -> crate::subscriber::Subscriber {
    crate::subscriber::Subscriber::new("ws-push", Box::new(move |event| {
        pusher.push(&event);
    }))
}

/// Null pusher - does nothing (for testing)
pub struct NullPusher;

impl EventPusher for NullPusher {
    fn push(&self, _event: &AcpEvent) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventBus, Subscriber};

    #[test]
    fn test_null_pusher() {
        let pusher = Arc::new(NullPusher);
        let sub = create_push_subscriber(pusher);

        let bus = EventBus::new();
        bus.subscribe("test", sub).unwrap();

        let event = AcpEvent::new("test.event");
        let delivered = bus.publish("test", event).unwrap();
        assert_eq!(delivered, 1);
    }
}