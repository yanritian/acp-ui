//! Workflow Engine - Multi-stage orchestrated workflows
//!
//! Provides:
//! - DAG-based workflow execution
//! - Checkpoint and rollback support
//! - Stage management and progression

pub mod dag;
pub mod checkpoint;
pub mod stage;
pub mod workflow;

// Re-export main types
pub use workflow::{Workflow, WorkflowStatus, WorkflowStage};
pub use dag::DagExecutor;
pub use checkpoint::CheckpointManager;