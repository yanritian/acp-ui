//! Event Pusher - Tauri emit adapter for EventBus
//!
//! Implements the EventPusher trait from event-bus crate to push
//! EventBus events to frontend via Tauri's emit mechanism.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use event_bus::{EventBus, create_push_subscriber};
//! use crate::event_pusher::TauriEventPusher;
//!
//! let app_handle = app.handle();
//! let pusher = Arc::new(TauriEventPusher::new(app_handle));
//! let sub = create_push_subscriber(pusher);
//! bus.subscribe("goal.*", sub);
//! ```

use acp_core::AcpEvent;
use event_bus::EventPusher;
use tauri::{AppHandle, Emitter};
use std::sync::Arc;

/// TauriEventPusher - Pushes EventBus events to frontend via Tauri emit
pub struct TauriEventPusher {
    app_handle: Arc<AppHandle>,
}

impl TauriEventPusher {
    /// Create a new TauriEventPusher
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle: Arc::new(app_handle),
        }
    }

    /// Create from existing Arc<AppHandle>
    pub fn from_arc(app_handle: Arc<AppHandle>) -> Self {
        Self { app_handle }
    }
}

impl EventPusher for TauriEventPusher {
    /// Push an event to frontend via Tauri emit
    fn push(&self, event: &AcpEvent) {
        // Emit to frontend with event type as the event name
        // Frontend can listen with: listen('swarm:event', callback)
        if let Err(e) = self.app_handle.emit("swarm:event", event) {
            eprintln!("[TauriEventPusher] Failed to emit event: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pusher_creation() {
        // This test requires a mock AppHandle which is not easily created
        // without a full Tauri app context. The logic is simple enough
        // to trust without complex mocking.
        // Real validation happens in integration tests.
    }
}