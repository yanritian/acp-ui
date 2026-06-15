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
//! # Unified Types (from acp-core)
//!
//! The following types are re-exported from acp-core for unified usage:
//! - `GoalStatus`, `IterationRecord`, `EvaluationResult`, `GoalOutcome`

pub mod goal;
pub mod goal_evaluator;
pub mod goal_graph;
pub mod reconcile;
pub mod topology;
pub mod queen_lease;
pub mod skill_registry;
pub mod complex_executor;

// Re-export from acp-core (unified types)
pub use acp_core::{
    GoalStatus as UnifiedGoalStatus,
    IterationRecord as UnifiedIterationRecord,
    EvaluationResult as UnifiedEvaluationResult,
    ConditionResult as UnifiedConditionResult,
    GoalOutcome as UnifiedGoalOutcome,
    GoalGraphSummary as UnifiedGoalGraphSummary,
    AcpEvent,
};

// Re-export main types
pub use goal::{
    Goal, GoalStatus, CompletionCondition, Evaluator,
    IterationRecord, EvaluationResult, GoalOutcome
};
pub use goal_evaluator::ConditionEvaluator;
pub use goal_graph::GoalGraph;
pub use reconcile::{ReconcileLoop, WorkerExecutor, EchoExecutor, CommandExecutor, AIWorkerExecutor, ReconcileError};
pub use topology::{Topology, StarTopology, StarTopologyAsync, ChainTopology, ChainTopologyAsync};
pub use queen_lease::{QueenLease, QueenElectionManager, WorkerId, ValidationResult};
pub use skill_registry::{SkillRouter, SkillDeclaration, WorkerCapabilitiesDeclaration};
pub use complex_executor::{ComplexGoalExecutor, ImplementationPlan, SubTaskSpec, ComplexError};