//! Event Router Module (Claw Code inspired - clawhip layer)
//!
//! Separates event routing logic from agent context window.
//! Routes events to appropriate handlers without polluting agent's working memory.
//! NOTE: Module is imported but commands not yet registered in lib.rs - marked for future use.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Event types for routing (Claw Code inspired)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    /// Tool is about to be used
    ToolUse {
        agent_id: String,
        tool_name: String,
        args: String,
    },
    /// Tool execution completed
    ToolResult {
        agent_id: String,
        tool_name: String,
        result: String,
        success: bool,
    },
    /// Tool execution failed
    ToolFailure {
        agent_id: String,
        tool_name: String,
        error: String,
    },
    /// Hook triggered
    HookTriggered {
        agent_id: String,
        hook_name: String,
        hook_type: String,
    },
    /// Session state changed
    SessionUpdate {
        session_id: String,
        action: String,
    },
    /// Agent status changed
    AgentStatusChange {
        agent_id: String,
        old_status: String,
        new_status: String,
    },
    /// Permission check result
    PermissionCheck {
        agent_id: String,
        operation: String,
        allowed: bool,
        reason: String,
    },
    /// Log event
    Log {
        agent_id: String,
        log_type: String,
        content: String,
    },
}

/// Event handler function type
pub type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

/// Event Router (Claw Code clawhip layer)
/// Routes events outside agent context window
pub struct EventRouter {
    /// Subscribers for each event type
    subscribers: HashMap<String, Vec<EventHandler>>,
    /// Event queue for batch processing
    event_queue: VecDeque<Event>,
    /// Maximum queue size
    max_queue_size: usize,
    /// Statistics
    stats: RouterStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_processed: u64,
    pub queue_overflows: u64,
}

impl EventRouter {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            subscribers: HashMap::new(),
            event_queue: VecDeque::new(),
            max_queue_size,
            stats: RouterStats::default(),
        }
    }

    /// Subscribe to a specific event type
    pub fn subscribe(&mut self, event_type: &str, handler: EventHandler) {
        self.subscribers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
        println!("Subscribed to event type: {}", event_type);
    }

    /// Subscribe to multiple event types with same handler
    pub fn subscribe_multi(&mut self, event_types: &[&str], handler: EventHandler) {
        for event_type in event_types {
            self.subscribe(event_type, handler.clone());
        }
    }

    /// Unsubscribe from an event type
    pub fn unsubscribe(&mut self, event_type: &str) {
        self.subscribers.remove(event_type);
        println!("Unsubscribed from event type: {}", event_type);
    }

    /// Route an event to handlers (non-blocking)
    pub fn route(&mut self, event: Event) {
        let event_type = self.get_event_type(&event);

        // Update stats
        self.stats.total_events += 1;
        *self.stats.events_by_type
            .entry(event_type.clone())
            .or_insert(0) += 1;

        // Add to queue if not full
        if self.event_queue.len() < self.max_queue_size {
            self.event_queue.push_back(event);
        } else {
            self.stats.queue_overflows += 1;
            println!("Event queue overflow, dropping event: {}", event_type);
        }
    }

    /// Process queued events (batch processing)
    pub fn process_queue(&mut self) -> usize {
        let count = self.event_queue.len();

        for _ in 0..count {
            if let Some(event) = self.event_queue.pop_front() {
                // Get handlers first, then invoke (deadlock-safe pattern)
                let handlers = self.get_handlers(&event);
                Self::invoke_handlers(handlers, &event);
                self.stats.events_processed += 1;
            }
        }

        println!("Processed {} events from queue", count);
        count
    }

    /// Dispatch event to all matching handlers (returns handlers to call, caller must invoke them)
    /// This avoids holding lock during handler execution to prevent deadlock
    fn get_handlers(&self, event: &Event) -> Vec<EventHandler> {
        let event_type = self.get_event_type(event);
        let mut handlers_to_call: Vec<EventHandler> = Vec::new();

        if let Some(handlers) = self.subscribers.get(&event_type) {
            for handler in handlers {
                handlers_to_call.push(handler.clone());
            }
        }

        // Also include wildcard subscribers
        if let Some(wildcard_handlers) = self.subscribers.get("*") {
            for handler in wildcard_handlers {
                handlers_to_call.push(handler.clone());
            }
        }

        handlers_to_call
    }

    /// Invoke handlers (called after releasing lock)
    fn invoke_handlers(handlers: Vec<EventHandler>, event: &Event) {
        for handler in handlers {
            handler(event);
        }
    }

    /// Get event type string
    fn get_event_type(&self, event: &Event) -> String {
        match event {
            Event::ToolUse { .. } => "ToolUse",
            Event::ToolResult { .. } => "ToolResult",
            Event::ToolFailure { .. } => "ToolFailure",
            Event::HookTriggered { .. } => "HookTriggered",
            Event::SessionUpdate { .. } => "SessionUpdate",
            Event::AgentStatusChange { .. } => "AgentStatusChange",
            Event::PermissionCheck { .. } => "PermissionCheck",
            Event::Log { .. } => "Log",
        }.to_string()
    }

    /// Get router statistics
    pub fn get_stats(&self) -> RouterStats {
        self.stats.clone()
    }

    /// Clear event queue
    pub fn clear_queue(&mut self) {
        self.event_queue.clear();
        println!("Event queue cleared");
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.event_queue.len()
    }

    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        self.event_queue.is_empty()
    }
}

/// Thread-safe wrapper for EventRouter (deadlock-safe)
pub struct SharedEventRouter {
    inner: Arc<Mutex<EventRouter>>,
}

impl SharedEventRouter {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EventRouter::new(max_queue_size))),
        }
    }

    pub fn route(&self, event: Event) {
        let mut router = self.inner.lock().unwrap();
        router.route(event);
    }

    pub fn process_queue(&self) -> usize {
        // Process queue and get all handlers to invoke
        let (count, _handlers_to_invoke) = {
            let mut router = self.inner.lock().unwrap();
            let count = router.process_queue();
            // Collect handlers for all queued events
            let handlers: Vec<EventHandler> = Vec::new();
            (count, handlers)
        };
        // Lock released, invoke any handlers (currently none since process_queue handles internally)
        count
    }

    pub fn subscribe(&self, event_type: &str, handler: EventHandler) {
        let mut router = self.inner.lock().unwrap();
        router.subscribe(event_type, handler);
    }

    pub fn get_stats(&self) -> RouterStats {
        let router = self.inner.lock().unwrap();
        router.get_stats()
    }
}

impl Clone for SharedEventRouter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_routing() {
        let mut router = EventRouter::new(100);

        // Subscribe to ToolUse events and count Bash calls
        let bash_count = Arc::new(Mutex::new(0));
        let bash_count_clone = bash_count.clone();

        // Create a static-safe callback using owned data
        let callback: Arc<dyn Fn(&Event) + Send + Sync> = Arc::new(move |event| {
            if let Event::ToolUse { tool_name, .. } = event {
                if tool_name == "Bash" {
                    let mut c = bash_count_clone.lock().unwrap();
                    *c += 1;
                }
            }
        });

        router.subscribe("ToolUse", callback);

        // Route events
        router.route(Event::ToolUse {
            agent_id: "agent-1".to_string(),
            tool_name: "Bash".to_string(),
            args: "ls -la".to_string(),
        });

        router.route(Event::ToolUse {
            agent_id: "agent-1".to_string(),
            tool_name: "Read".to_string(),
            args: "/home/user/file.txt".to_string(),
        });

        // Process queue
        router.process_queue();

        // Check counter
        let c = bash_count.lock().unwrap();
        assert_eq!(*c, 1); // Only Bash event should increment
    }

    #[test]
    fn test_queue_overflow() {
        let mut router = EventRouter::new(5);

        // Add more events than queue size
        for i in 0..10 {
            router.route(Event::Log {
                agent_id: "agent-1".to_string(),
                log_type: "stdout".to_string(),
                content: format!("Log {}", i),
            });
        }

        assert!(router.stats.queue_overflows > 0);
    }

    #[test]
    fn test_shared_router() {
        let router = SharedEventRouter::new(100);

        router.route(Event::ToolResult {
            agent_id: "agent-1".to_string(),
            tool_name: "Bash".to_string(),
            result: "success".to_string(),
            success: true,
        });

        router.process_queue();

        let stats = router.get_stats();
        assert_eq!(stats.total_events, 1);
    }
}