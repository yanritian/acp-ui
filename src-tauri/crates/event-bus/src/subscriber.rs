//! Event Subscriber - Subscribe to events and receive notifications

use acp_core::AcpEvent;
use parking_lot::Mutex;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubscriberError {
    #[error("Subscriber not found: {0}")]
    NotFound(String),

    #[error("Invalid subscriber: {0}")]
    Invalid(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
}

/// Event handler function type
pub type EventHandler = Box<dyn Fn(AcpEvent) + Send + Sync>;

/// Subscriber for receiving events
pub struct Subscriber {
    /// Subscriber ID
    pub id: String,
    /// Event handler
    handler: EventHandler,
    /// Active flag
    active: bool,
}

impl Subscriber {
    pub fn new(id: impl Into<String>, handler: EventHandler) -> Self {
        Self {
            id: id.into(),
            handler,
            active: true,
        }
    }

    /// Handle an event
    pub fn handle(&self, event: AcpEvent) {
        if self.active {
            (self.handler)(event);
        }
    }

    /// Activate subscriber
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate subscriber (pause receiving events)
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Subscriber handle for managing subscriptions
pub struct SubscriberHandle {
    /// Subscriber ID
    pub id: String,
    /// Inner subscriber (shared for async access)
    inner: Arc<Mutex<Subscriber>>,
}

impl SubscriberHandle {
    pub fn new(subscriber: Subscriber) -> Self {
        let id = subscriber.id.clone();
        Self {
            id,
            inner: Arc::new(Mutex::new(subscriber)),
        }
    }

    /// Deliver event to subscriber
    pub fn deliver(&self, event: AcpEvent) -> bool {
        let sub = self.inner.lock();
        if sub.is_active() {
            sub.handle(event);
            true
        } else {
            false
        }
    }

    /// Activate subscriber
    pub fn activate(&self) {
        self.inner.lock().activate();
    }

    /// Deactivate subscriber
    pub fn deactivate(&self) {
        self.inner.lock().deactivate();
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.inner.lock().is_active()
    }
}

impl Clone for SubscriberHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            inner: self.inner.clone(),
        }
    }
}

/// Subscriber builder for easy creation
pub struct SubscriberBuilder {
    id: Option<String>,
    handler: Option<EventHandler>,
    active: bool,
}

impl SubscriberBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            handler: None,
            active: true,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_handler(mut self, handler: EventHandler) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn build(self) -> Result<Subscriber, SubscriberError> {
        let id = self.id.ok_or_else(|| SubscriberError::Invalid("ID required".into()))?;
        let handler = self.handler.ok_or_else(|| SubscriberError::Invalid("Handler required".into()))?;

        let mut sub = Subscriber::new(id, handler);
        sub.active = self.active;
        Ok(sub)
    }
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_new() {
        let sub = Subscriber::new("sub-001", Box::new(|_| {}));
        assert_eq!(sub.id, "sub-001");
        assert!(sub.is_active());
    }

    #[test]
    fn subscriber_activate_deactivate() {
        let mut sub = Subscriber::new("sub-001", Box::new(|_| {}));
        assert!(sub.is_active());

        sub.deactivate();
        assert!(!sub.is_active());

        sub.activate();
        assert!(sub.is_active());
    }

    #[test]
    fn subscriber_handle() {
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();

        let sub = Subscriber::new("sub-001", Box::new(move |_| {
            *received_clone.lock() = true;
        }));

        let event = AcpEvent::new("goal.created").with_goal("goal-001");

        sub.handle(event);
        assert!(*received.lock());
    }

    #[test]
    fn subscriber_handle_inactive() {
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();

        let mut sub = Subscriber::new("sub-001", Box::new(move |_| {
            *received_clone.lock() = true;
        }));

        sub.deactivate();

        let event = AcpEvent::new("goal.created").with_goal("goal-001");

        sub.handle(event);
        assert!(!*received.lock()); // Not received because inactive
    }

    #[test]
    fn subscriber_handle_new() {
        let sub = Subscriber::new("sub-001", Box::new(|_| {}));
        let handle = SubscriberHandle::new(sub);
        assert_eq!(handle.id, "sub-001");
    }

    #[test]
    fn subscriber_handle_deliver() {
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();

        let sub = Subscriber::new("sub-001", Box::new(move |_| {
            *received_clone.lock() = true;
        }));
        let handle = SubscriberHandle::new(sub);

        let event = AcpEvent::new("goal.created").with_goal("goal-001");

        assert!(handle.deliver(event));
        assert!(*received.lock());
    }

    #[test]
    fn subscriber_handle_clone() {
        let sub = Subscriber::new("sub-001", Box::new(|_| {}));
        let handle1 = SubscriberHandle::new(sub);
        let handle2 = handle1.clone();

        assert_eq!(handle1.id, handle2.id);
    }

    #[test]
    fn subscriber_builder() {
        let sub = SubscriberBuilder::new()
            .with_id("sub-001")
            .with_handler(Box::new(|_| {}))
            .build()
            .unwrap();

        assert_eq!(sub.id, "sub-001");
    }

    #[test]
    fn subscriber_builder_missing_id() {
        let result = SubscriberBuilder::new()
            .with_handler(Box::new(|_| {}))
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn subscriber_builder_missing_handler() {
        let result = SubscriberBuilder::new()
            .with_id("sub-001")
            .build();

        assert!(result.is_err());
    }
}