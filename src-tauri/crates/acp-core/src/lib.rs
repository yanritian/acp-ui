//! ACP Core - Protocol definitions and message types
//!
//! This crate contains the core ACP protocol definitions with zero internal dependencies.
//! It defines the message types, worker protocol, goal protocol, and event protocol.
//!
//! # Unified Goal Types
//!
//! This crate provides unified Goal types for both src-tauri/src/ and swarm-engine crate:
//! - `GoalSpec` - Goal submission specification
//! - `GoalStatus` - Execution status enum
//! - `IterationRecord` - Iteration feedback record
//! - `EvaluationResult` - Condition evaluation result
//! - `GoalOutcome` - Final execution outcome
//! - `GoalGraphSummary` - Graph status summary
//! - `AcpEvent` - Event Protocol event type
//! - `GoalYamlFile` - YAML file parser for .goal.yaml files

pub mod message;
pub mod worker_protocol;
pub mod goal_protocol;
pub mod event_protocol;

// Re-export main types
pub use message::AcpMessage;
pub use worker_protocol::{WorkerCapabilities, WorkerStatus, WorkerHeartbeat};
pub use goal_protocol::{
    GoalSpec,
    GoalYamlFile,
    QueenConfig,
    WorkerDefinition,
    SkillDefinition,
    CompletionConditionSpec,
    EvaluatorSpec,
    GoalStatus,
    IterationRecord,
    EvaluationResult,
    ConditionResult,
    GoalOutcome,
    GoalGraphSummary,
};
pub use event_protocol::AcpEvent;