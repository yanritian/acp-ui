// Operator Module - Hermes Game Operator Control Plane
// This module implements the core operator protocol and state machine

pub mod types;
pub mod state_machine;
pub mod commands;
pub mod security;
pub mod file_tools;
pub mod agent_bridge;
pub mod task_executor;
pub mod hermes_runtime;
pub mod hermes_cli_bridge;
pub mod approval_queue;
pub mod error;

#[cfg(test)]
pub mod e2e_tests;

// Re-export commonly used types
pub use types::*;
pub use state_machine::{TaskStateMachine, StateError};
pub use commands::*;
pub use security::{PathGuard, PathGuardError, CommandGuard, CommandGuardError, ValidatedCommand};
pub use file_tools::*;
pub use agent_bridge::*;
pub use task_executor::*;
pub use hermes_runtime::*;
pub use hermes_cli_bridge::*;
pub use approval_queue::*;
pub use error::*;
