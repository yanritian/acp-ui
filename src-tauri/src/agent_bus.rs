//! Agent Communication Bus Module
//!
//! Enables agent-to-agent message routing for ACP-UI.
//! Currently agents can only communicate with the user - this module
//! provides direct agent-to-agent messaging, broadcast, and pub/sub patterns.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tauri::State;

use crate::AppState;

// ---------------------------------------------------------------------------
// Message Types
// ---------------------------------------------------------------------------

/// Message types for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMessageType {
    /// Task delegation from one agent to another
    Task,
    /// Task result/response
    Result,
    /// Information request
    Query,
    /// Information response
    Response,
    /// Status update (heartbeat, progress, etc.)
    Status,
    /// Synchronization point (barrier, checkpoint)
    Sync,
    /// Error notification
    Error,
}

impl std::fmt::Display for AgentMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMessageType::Task => write!(f, "task"),
            AgentMessageType::Result => write!(f, "result"),
            AgentMessageType::Query => write!(f, "query"),
            AgentMessageType::Response => write!(f, "response"),
            AgentMessageType::Status => write!(f, "status"),
            AgentMessageType::Sync => write!(f, "sync"),
            AgentMessageType::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for AgentMessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "task" => Ok(AgentMessageType::Task),
            "result" => Ok(AgentMessageType::Result),
            "query" => Ok(AgentMessageType::Query),
            "response" => Ok(AgentMessageType::Response),
            "status" => Ok(AgentMessageType::Status),
            "sync" => Ok(AgentMessageType::Sync),
            "error" => Ok(AgentMessageType::Error),
            other => Err(format!(
                "Unknown message type: '{}'. Valid: task, result, query, response, status, sync, error",
                other
            )),
        }
    }
}

/// Message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unique message ID
    pub id: String,
    /// Sender agent ID
    pub from_agent: String,
    /// Target agent ID, or "broadcast" for all agents
    pub to_agent: String,
    /// Message type classification
    pub message_type: AgentMessageType,
    /// Arbitrary JSON payload
    pub payload: serde_json::Value,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Message ID this is replying to (for threading)
    pub reply_to: Option<String>,
}

impl AgentMessage {
    /// Create a new message with auto-generated ID and timestamp
    pub fn new(
        from_agent: &str,
        to_agent: &str,
        message_type: AgentMessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            message_type,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            reply_to: None,
        }
    }

    /// Create a reply to an existing message
    pub fn reply(
        from_agent: &str,
        original: &AgentMessage,
        message_type: AgentMessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_agent: from_agent.to_string(),
            to_agent: original.from_agent.clone(),
            message_type,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            reply_to: Some(original.id.clone()),
        }
    }

    /// Check if this is a broadcast message
    pub fn is_broadcast(&self) -> bool {
        self.to_agent == "broadcast"
    }
}

// ---------------------------------------------------------------------------
// Agent Mailbox
// ---------------------------------------------------------------------------

/// Message mailbox for an agent
#[derive(Debug)]
pub struct AgentMailbox {
    /// Agent identifier
    pub agent_id: String,
    /// Incoming message queue
    pub inbox: VecDeque<AgentMessage>,
    /// Maximum inbox size (oldest messages dropped when exceeded)
    pub max_size: usize,
    /// When this agent registered
    pub registered_at: String,
    /// Total messages received by this agent
    pub messages_received: u64,
}

impl AgentMailbox {
    /// Create a new mailbox for an agent
    fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            inbox: VecDeque::new(),
            max_size: 1000, // Default max inbox size
            registered_at: chrono::Utc::now().to_rfc3339(),
            messages_received: 0,
        }
    }

    /// Add a message to the inbox
    fn push(&mut self, message: AgentMessage) {
        // Drop oldest if at capacity
        if self.inbox.len() >= self.max_size {
            self.inbox.pop_front();
        }
        self.inbox.push_back(message);
        self.messages_received += 1;
    }

    /// Drain messages from inbox (up to limit)
    fn drain(&mut self, limit: Option<usize>) -> Vec<AgentMessage> {
        match limit {
            Some(n) => {
                let count = std::cmp::min(n, self.inbox.len());
                self.inbox.drain(..count).collect()
            }
            None => self.inbox.drain(..).collect(),
        }
    }

    /// Peek at messages without draining
    fn peek(&self, limit: Option<usize>) -> Vec<&AgentMessage> {
        match limit {
            Some(n) => self.inbox.iter().take(n).collect(),
            None => self.inbox.iter().collect(),
        }
    }

    /// Get current inbox size
    fn count(&self) -> usize {
        self.inbox.len()
    }
}

// ---------------------------------------------------------------------------
// Agent Bus
// ---------------------------------------------------------------------------

/// Agent communication bus - routes messages between agents
pub struct AgentBus {
    /// Agent mailboxes indexed by agent_id
    mailboxes: HashMap<String, AgentMailbox>,
    /// Message history for audit/debugging
    message_history: Vec<AgentMessage>,
    /// Maximum history size
    max_history: usize,
    /// Topic subscriptions (topic -> list of agent_ids)
    subscribers: HashMap<String, Vec<String>>,
    /// Total messages sent
    total_sent: u64,
    /// Total messages delivered
    total_delivered: u64,
}

impl AgentBus {
    /// Create a new agent communication bus
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            message_history: Vec::new(),
            max_history: 10000, // Keep last 10k messages
            subscribers: HashMap::new(),
            total_sent: 0,
            total_delivered: 0,
        }
    }

    /// Register an agent's mailbox
    pub fn register_agent(&mut self, agent_id: &str) -> Result<(), String> {
        if self.mailboxes.contains_key(agent_id) {
            return Err(format!("Agent '{}' is already registered", agent_id));
        }

        self.mailboxes
            .insert(agent_id.to_string(), AgentMailbox::new(agent_id));
        println!("AgentBus: Registered agent '{}'", agent_id);
        Ok(())
    }

    /// Unregister an agent and clean up
    pub fn unregister_agent(&mut self, agent_id: &str) -> Result<(), String> {
        if self.mailboxes.remove(agent_id).is_none() {
            return Err(format!("Agent '{}' is not registered", agent_id));
        }

        // Remove from all topic subscriptions
        for subscribers in self.subscribers.values_mut() {
            subscribers.retain(|id| id != agent_id);
        }

        // Clean up empty subscription lists
        self.subscribers.retain(|_, v| !v.is_empty());

        println!("AgentBus: Unregistered agent '{}'", agent_id);
        Ok(())
    }

    /// Send a message from one agent to another
    pub fn send(&mut self, message: AgentMessage) -> Result<(), String> {
        // Validate sender is registered
        if !self.mailboxes.contains_key(&message.from_agent) {
            return Err(format!(
                "Sender '{}' is not registered",
                message.from_agent
            ));
        }

        self.total_sent += 1;

        // Record in history
        self.record_history(message.clone());

        if message.is_broadcast() {
            // Deliver to all agents except sender
            let recipients: Vec<String> = self
                .mailboxes
                .keys()
                .filter(|id| *id != &message.from_agent)
                .cloned()
                .collect();

            for recipient_id in recipients {
                if let Some(mailbox) = self.mailboxes.get_mut(&recipient_id) {
                    mailbox.push(message.clone());
                    self.total_delivered += 1;
                }
            }
        } else {
            // Deliver to specific agent
            let mailbox = self.mailboxes.get_mut(&message.to_agent).ok_or_else(|| {
                format!(
                    "Recipient '{}' is not registered",
                    message.to_agent
                )
            })?;

            mailbox.push(message);
            self.total_delivered += 1;
        }

        Ok(())
    }

    /// Receive messages for an agent (drains inbox)
    pub fn receive(
        &mut self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<AgentMessage>, String> {
        let mailbox = self
            .mailboxes
            .get_mut(agent_id)
            .ok_or_else(|| format!("Agent '{}' is not registered", agent_id))?;

        Ok(mailbox.drain(limit))
    }

    /// Peek at messages without draining
    pub fn peek(
        &self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<&AgentMessage>, String> {
        let mailbox = self
            .mailboxes
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' is not registered", agent_id))?;

        Ok(mailbox.peek(limit))
    }

    /// Get message count for an agent
    pub fn inbox_count(&self, agent_id: &str) -> usize {
        self.mailboxes
            .get(agent_id)
            .map(|m| m.count())
            .unwrap_or(0)
    }

    /// Subscribe to a topic (for pub/sub pattern)
    pub fn subscribe(&mut self, agent_id: &str, topic: &str) -> Result<(), String> {
        // Validate agent is registered
        if !self.mailboxes.contains_key(agent_id) {
            return Err(format!("Agent '{}' is not registered", agent_id));
        }

        let subscribers = self
            .subscribers
            .entry(topic.to_string())
            .or_default();

        if !subscribers.contains(&agent_id.to_string()) {
            subscribers.push(agent_id.to_string());
            println!("AgentBus: Agent '{}' subscribed to topic '{}'", agent_id, topic);
        }

        Ok(())
    }

    /// Unsubscribe from a topic
    pub fn unsubscribe(&mut self, agent_id: &str, topic: &str) -> Result<(), String> {
        if let Some(subscribers) = self.subscribers.get_mut(topic) {
            subscribers.retain(|id| id != agent_id);
            if subscribers.is_empty() {
                self.subscribers.remove(topic);
            }
        }
        Ok(())
    }

    /// Publish to a topic - sends message to all subscribers
    pub fn publish(
        &mut self,
        from_agent: &str,
        topic: &str,
        payload: serde_json::Value,
    ) -> Result<u32, String> {
        // Validate sender is registered
        if !self.mailboxes.contains_key(from_agent) {
            return Err(format!("Publisher '{}' is not registered", from_agent));
        }

        // Get subscribers for this topic (excluding sender)
        let subscribers = self
            .subscribers
            .get(topic)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|id| id != from_agent)
            .collect::<Vec<_>>();

        let recipient_count = subscribers.len() as u32;

        // Create and deliver messages
        for subscriber_id in subscribers {
            let message = AgentMessage::new(
                from_agent,
                &subscriber_id,
                AgentMessageType::Status, // Topic messages use Status type
                serde_json::json!({
                    "topic": topic,
                    "data": payload.clone()
                }),
            );

            // Record history first (avoids double mutable borrow with mailboxes)
            if self.message_history.len() >= self.max_history {
                let remove_count = self.max_history / 10;
                self.message_history.drain(..remove_count);
            }
            self.message_history.push(message.clone());
            self.total_sent += 1;

            if let Some(mailbox) = self.mailboxes.get_mut(&subscriber_id) {
                mailbox.push(message);
                self.total_delivered += 1;
            }
        }

        Ok(recipient_count)
    }

    /// Get message history (for audit/debugging)
    pub fn get_history(&self, limit: Option<usize>) -> Vec<&AgentMessage> {
        match limit {
            Some(n) => self.message_history.iter().rev().take(n).collect(),
            None => self.message_history.iter().rev().collect(),
        }
    }

    /// Get history filtered by agent (sent or received)
    pub fn get_agent_history(&self, agent_id: &str, limit: Option<usize>) -> Vec<&AgentMessage> {
        let filtered: Vec<&AgentMessage> = self
            .message_history
            .iter()
            .rev()
            .filter(|m| m.from_agent == agent_id || m.to_agent == agent_id || m.to_agent == "broadcast")
            .collect();

        match limit {
            Some(n) => filtered.into_iter().take(n).collect(),
            None => filtered,
        }
    }

    /// Get bus statistics
    pub fn get_stats(&self) -> AgentBusStats {
        let pending_messages: u64 = self
            .mailboxes
            .values()
            .map(|m| m.count() as u64)
            .sum();

        AgentBusStats {
            registered_agents: self.mailboxes.len() as u32,
            total_messages_sent: self.total_sent,
            total_messages_delivered: self.total_delivered,
            pending_messages,
            topics: self.subscribers.len() as u32,
        }
    }

    /// Get list of registered agents
    pub fn list_agents(&self) -> Vec<String> {
        self.mailboxes.keys().cloned().collect()
    }

    /// Get list of topics
    pub fn list_topics(&self) -> Vec<String> {
        self.subscribers.keys().cloned().collect()
    }

    /// Get subscribers for a topic
    pub fn get_topic_subscribers(&self, topic: &str) -> Vec<String> {
        self.subscribers
            .get(topic)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if an agent is registered
    pub fn is_registered(&self, agent_id: &str) -> bool {
        self.mailboxes.contains_key(agent_id)
    }

    /// Record a message in history (internal)
    fn record_history(&mut self, message: AgentMessage) {
        if self.message_history.len() >= self.max_history {
            // Remove oldest 10% when at capacity
            let remove_count = self.max_history / 10;
            self.message_history.drain(..remove_count);
        }
        self.message_history.push(message);
    }
}

impl Default for AgentBus {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// Agent bus statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBusStats {
    /// Number of registered agents
    pub registered_agents: u32,
    /// Total messages sent through the bus
    pub total_messages_sent: u64,
    /// Total messages successfully delivered
    pub total_messages_delivered: u64,
    /// Messages currently pending in mailboxes
    pub pending_messages: u64,
    /// Number of active topics
    pub topics: u32,
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Register an agent with the communication bus
#[tauri::command]
pub fn agent_bus_register(state: State<'_, AppState>, agent_id: String) -> Result<(), String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    bus.register_agent(&agent_id)
}

/// Unregister an agent from the communication bus
#[tauri::command]
pub fn agent_bus_unregister(state: State<'_, AppState>, agent_id: String) -> Result<(), String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    bus.unregister_agent(&agent_id)
}

/// Send a message from one agent to another
#[tauri::command]
pub fn agent_bus_send(
    state: State<'_, AppState>,
    from_agent: String,
    to_agent: String,
    message_type: String,
    payload: serde_json::Value,
) -> Result<(), String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;

    let msg_type = message_type.parse::<AgentMessageType>()?;

    let message = AgentMessage::new(&from_agent, &to_agent, msg_type, payload);

    bus.send(message)
}

/// Receive messages for an agent (drains inbox)
#[tauri::command]
pub fn agent_bus_receive(
    state: State<'_, AppState>,
    agent_id: String,
    limit: Option<usize>,
) -> Result<Vec<AgentMessage>, String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    bus.receive(&agent_id, limit)
}

/// Peek at messages without draining
#[tauri::command]
pub fn agent_bus_peek(
    state: State<'_, AppState>,
    agent_id: String,
    limit: Option<usize>,
) -> Result<Vec<AgentMessage>, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    let messages = bus.peek(&agent_id, limit)?;
    Ok(messages.into_iter().cloned().collect())
}

/// Get message count for an agent's inbox
#[tauri::command]
pub fn agent_bus_inbox_count(state: State<'_, AppState>, agent_id: String) -> Result<usize, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.inbox_count(&agent_id))
}

/// Subscribe an agent to a topic
#[tauri::command]
pub fn agent_bus_subscribe(
    state: State<'_, AppState>,
    agent_id: String,
    topic: String,
) -> Result<(), String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    bus.subscribe(&agent_id, &topic)
}

/// Publish a message to a topic
#[tauri::command]
pub fn agent_bus_publish(
    state: State<'_, AppState>,
    from_agent: String,
    topic: String,
    payload: serde_json::Value,
) -> Result<u32, String> {
    let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    bus.publish(&from_agent, &topic, payload)
}

/// Get bus statistics
#[tauri::command]
pub fn agent_bus_stats(state: State<'_, AppState>) -> Result<AgentBusStats, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.get_stats())
}

/// Get message history (for audit/debugging)
#[tauri::command]
pub fn agent_bus_history(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AgentMessage>, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.get_history(limit).into_iter().cloned().collect())
}

/// List all registered agents
#[tauri::command]
pub fn agent_bus_list_agents(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.list_agents())
}

/// List all active topics
#[tauri::command]
pub fn agent_bus_list_topics(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.list_topics())
}

/// Check if an agent is registered
#[tauri::command]
pub fn agent_bus_is_registered(state: State<'_, AppState>, agent_id: String) -> Result<bool, String> {
    let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
    Ok(bus.is_registered(&agent_id))
}
