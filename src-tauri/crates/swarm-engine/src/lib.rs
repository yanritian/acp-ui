//! Swarm Engine - Goal-driven orchestration engine
//!
//! Core components:
//! - Goal: Target state with completion conditions
//! - GoalGraph: Dependency DAG for multi-Goal coordination
//! - Reconcile Loop: Iterative execution with feedback
//! - Topology: Star/Chain/Pipeline execution patterns
//! - SkillRouter: Worker capability routing
//! - ComplexGoalExecutor: Handles complex system requirements with planning/decomposition
//!
//! # Unified Types (from acp-core, C-1 solution)
//!
//! The following types are re-exported from acp-core for unified usage:
//! - `GoalRuntime` (aliased as `Goal`) - Authoritative Goal execution type
//! - `GoalSpec` - Goal submission specification
//! - `GoalStatus`, `IterationRecord`, `EvaluationResult`, `GoalOutcome`
//!
//! **Note**: The local Goal struct has been replaced with acp-core's GoalRuntime.
//! Use `Goal` (alias) from this crate or `GoalRuntime` from acp-core.

pub mod goal;
pub mod goal_evaluator;
pub mod goal_graph;
pub mod reconcile;
pub mod topology;
pub mod queen_lease;
pub mod skill_registry;
pub mod complex_executor;

// Re-export from acp-core (unified types - C-1 solution)
pub use acp_core::{
    GoalRuntime as Goal,  // Alias for backward compatibility
    GoalSpec,             // Submission spec
    GoalStatus,
    IterationRecord,
    EvaluationResult,
    ConditionResult,
    GoalOutcome,
    GoalGraphSummary,
    AcpEvent,
};

// Re-export swarm-engine specific types
pub use goal::CompletionCondition;
pub use goal::Evaluator;
pub use goal_evaluator::{ConditionEvaluator, ConditionEvaluationResult};
pub use goal_graph::GoalGraph;
pub use reconcile::{ReconcileLoop, WorkerExecutor, EchoExecutor, CommandExecutor, AIWorkerExecutor, ReconcileError};
pub use topology::{Topology, StarTopology, StarTopologyAsync, ChainTopology, ChainTopologyAsync};
pub use queen_lease::{QueenLease, QueenElectionManager, WorkerId, ValidationResult};
pub use skill_registry::{SkillRouter, SkillDeclaration, WorkerCapabilitiesDeclaration};
pub use complex_executor::{ComplexGoalExecutor, ImplementationPlan, SubTaskSpec, ComplexError};