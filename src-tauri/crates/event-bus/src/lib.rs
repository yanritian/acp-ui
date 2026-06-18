//! Event Bus - Unified publish/subscribe channel for Event Protocol
//!
//! Provides:
//! - Topic-based event publishing
//! - Subscriber registration and management
//! - Event queue for async processing
//! - WebSocket adapter for frontend push
//!
//! # Example
//!
//! ```rust,ignore
//! use event_bus::{EventBus, Subscriber};
//! use acp_core::AcpEvent;
//!
//! let bus = EventBus::new();
//!
//! // Subscribe to topic
//! let sub = Subscriber::new("sub-001", Box::new(|event| {
//!     println!("Received: {:?}", event);
//! }));
//! bus.subscribe("goal.status", sub);
//!
//! // Publish event
//! let event = AcpEvent::new("goal.converged").with_goal("goal-001");
//! bus.publish("goal.status", event);
//! ```

pub mod publisher;
pub mod subscriber;
pub mod topics;
pub mod websocket_adapter;

use acp_core::AcpEvent;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub use publisher::{EventPublisher, PublishError};
pub use subscriber::{Subscriber, SubscriberHandle, SubscriberError};
pub use topics::{EventTopic, TopicMatcher};
pub use websocket_adapter::{EventPusher, NullPusher, create_push_subscriber};

/// Event bus for publish/subscribe operations
pub struct EventBus {
    /// Subscriber registry: topic -> subscribers
    subscribers: Mutex<HashMap<String, Vec<SubscriberHandle>>>,
    /// Event queue for async processing
    event_queue: Mutex<VecDeque<AcpEvent>>,
    /// Maximum queue size
    max_queue_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(HashMap::new()),
            event_queue: Mutex::new(VecDeque::new()),
            max_queue_size: 1000,
        }
    }

    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    /// Publish an event to all subscribers of a topic
    pub fn publish(&self, topic: &str, event: AcpEvent) -> Result<usize, PublishError> {
        // Add to queue
        {
            let mut queue = self.event_queue.lock();
            if queue.len() >= self.max_queue_size {
                // Drop oldest events
                queue.pop_front();
            }
            queue.push_back(event.clone());
        }

        // Deliver to subscribers
        let mut delivered = 0;
        {
            let subscribers = self.subscribers.lock();

            // Exact match
            if let Some(subs) = subscribers.get(topic) {
                for handle in subs {
                    if handle.deliver(event.clone()) {
                        delivered += 1;
                    }
                }
            }

            // Wildcard match (topic ending with *)
            for (pattern, subs) in subscribers.iter() {
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    if topic.starts_with(prefix) {
                        for handle in subs {
                            if handle.deliver(event.clone()) {
                                delivered += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(delivered)
    }

    /// Subscribe to a topic
    pub fn subscribe(&self, topic: &str, subscriber: Subscriber) -> Result<String, SubscriberError> {
        let handle = SubscriberHandle::new(subscriber);
        let id = handle.id.clone();

        {
            let mut subscribers = self.subscribers.lock();
            subscribers.entry(topic.to_string()).or_default().push(handle);
        }

        Ok(id)
    }

    /// Unsubscribe from a topic
    pub fn unsubscribe(&self, topic: &str, subscriber_id: &str) -> Result<bool, SubscriberError> {
        let mut subscribers = self.subscribers.lock();

        if let Some(subs) = subscribers.get_mut(topic) {
            let removed = subs.iter().position(|h| h.id == subscriber_id);
            if let Some(idx) = removed {
                subs.remove(idx);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get pending events from queue
    pub fn pending_events(&self) -> Vec<AcpEvent> {
        let queue = self.event_queue.lock();
        queue.iter().cloned().collect()
    }

    /// Clear event queue
    pub fn clear_queue(&self) {
        let mut queue = self.event_queue.lock();
        queue.clear();
    }

    /// Get subscriber count for a topic
    pub fn subscriber_count(&self, topic: &str) -> usize {
        let subscribers = self.subscribers.lock();
        subscribers.get(topic).map(|s| s.len()).unwrap_or(0)
    }

    /// Get all topics with subscribers
    pub fn topics(&self) -> Vec<String> {
        let subscribers = self.subscribers.lock();
        subscribers.keys().cloned().collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared event bus reference
pub type SharedEventBus = Arc<EventBus>;

#[cfg(test)]
mod tests {
    use super::*;
    use acp_core::AcpEvent;

    #[test]
    fn event_bus_new() {
        let bus = EventBus::new();
        assert!(bus.pending_events().is_empty());
        assert!(bus.topics().is_empty());
    }

    #[test]
    fn event_bus_subscribe_publish() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        let sub = Subscriber::new("test-sub", Box::new(move |event| {
            received_clone.lock().push(event);
        }));

        bus.subscribe("goal.status", sub).unwrap();

        let event = AcpEvent::new("goal.converged").with_goal("goal-001");

        let delivered = bus.publish("goal.status", event.clone()).unwrap();
        assert_eq!(delivered, 1);

        let events = received.lock();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn event_bus_unsubscribe() {
        let bus = EventBus::new();

        let sub = Subscriber::new("test-sub", Box::new(|_| {}));
        let id = bus.subscribe("goal.status", sub).unwrap();

        assert_eq!(bus.subscriber_count("goal.status"), 1);

        bus.unsubscribe("goal.status", &id).unwrap();
        assert_eq!(bus.subscriber_count("goal.status"), 0);
    }

    #[test]
    fn event_bus_wildcard() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(0));
        let received_clone = received.clone();

        let sub = Subscriber::new("wildcard-sub", Box::new(move |_| {
            *received_clone.lock() += 1;
        }));

        bus.subscribe("goal.*", sub).unwrap();

        let event = AcpEvent::new("goal.converged").with_goal("goal-001");

        // Should match wildcard "goal.*"
        let delivered = bus.publish("goal.status", event.clone()).unwrap();
        assert_eq!(delivered, 1);

        let delivered = bus.publish("goal.created", event).unwrap();
        assert_eq!(delivered, 1);

        let count = *received.lock();
        assert_eq!(count, 2);
    }

    #[test]
    fn event_bus_queue_limit() {
        let bus = EventBus::new().with_max_queue_size(5);

        let event = AcpEvent::new("test.event");

        // Add 10 events
        for _ in 0..10 {
            bus.publish("test", event.clone()).ok();
        }

        // Should only keep 5 most recent
        let pending = bus.pending_events();
        assert!(pending.len() <= 5);
    }

    #[test]
    fn event_bus_no_subscribers() {
        let bus = EventBus::new();

        let event = AcpEvent::new("test.event");

        // Should still queue event even if no subscribers
        let delivered = bus.publish("nonexistent.topic", event).unwrap();
        assert_eq!(delivered, 0);

        // Event is in queue
        assert_eq!(bus.pending_events().len(), 1);
    }
}