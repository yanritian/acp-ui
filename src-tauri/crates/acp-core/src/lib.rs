//! ACP Core - Protocol definitions and message types
//!
//! This crate contains the core ACP protocol definitions with zero internal dependencies.
//! It defines the message types, worker protocol, goal protocol, and event protocol.

pub mod message;
pub mod worker_protocol;
pub mod goal_protocol;
pub mod event_protocol;

// Re-export main types
pub use message::AcpMessage;
pub use worker_protocol::{WorkerCapabilities, WorkerStatus, WorkerHeartbeat};
pub use goal_protocol::{GoalSpec, CompletionConditionSpec};
pub use event_protocol::AcpEvent;