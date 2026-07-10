// Operator Module - Hermes Game Operator Control Plane
// This module implements the core operator protocol and state machine

pub mod agent_bridge;
pub mod approval_queue;
pub mod commands;
pub mod error;
pub mod file_tools;
pub mod godot_validation;
pub mod hermes_cli_bridge;
pub mod hermes_game_bridge;
pub mod hermes_process;
pub mod hermes_runtime;
pub mod persistence;
pub mod security;
pub mod state_machine;
pub mod structured_patch;
pub mod task_executor;
pub mod types;

#[cfg(test)]
pub mod e2e_tests;

// Re-export commonly used types
pub use agent_bridge::{
    ExecutionPlan, HermesAgentBridge, PlanStep, ProjectAnalysisResult, StepResult, StepStatus,
};
pub use approval_queue::*;
pub use commands::*;
pub use error::*;
pub use file_tools::*;
pub use godot_validation::*;
pub use hermes_cli_bridge::*;
pub use hermes_game_bridge::*;
pub use hermes_process::*;
pub use hermes_runtime::*;
pub use persistence::*;
pub use security::{CommandGuard, CommandGuardError, PathGuard, PathGuardError, ValidatedCommand};
pub use state_machine::{StateError, TaskStateMachine};
pub use structured_patch::*;
pub use task_executor::*;
pub use types::*;
