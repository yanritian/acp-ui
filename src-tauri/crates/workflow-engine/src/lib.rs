//! Workflow Engine - Multi-stage orchestrated workflows
//!
//! Provides:
//! - DAG-based workflow execution
//! - Checkpoint and rollback support
//! - Stage management and progression
//!
//! # Example
//!
//! ```rust,ignore
//! use workflow_engine::{Workflow, WorkflowStage, DagExecutor, CheckpointManager};
//!
//! // Create workflow
//! let workflow = Workflow::new("wf-001", "Data Pipeline")
//!     .with_stage(WorkflowStage::new("extract", "Extract data"))
//!     .with_stage(WorkflowStage::new("transform", "Transform data")
//!         .with_dependency("extract"))
//!     .with_stage(WorkflowStage::new("load", "Load data")
//!         .with_dependency("transform"));
//!
//! // Save checkpoints
//! let checkpoint_mgr = CheckpointManager::new("./data/checkpoints");
//! ```

pub mod dag;
pub mod checkpoint;
pub mod stage;
pub mod workflow;

// Re-export main types
pub use workflow::{
    Workflow, WorkflowStatus, WorkflowStage, StageStatus,
    WorkflowSummary,
};
pub use dag::{DagExecutor, DagNode, DagResult};
pub use checkpoint::{Checkpoint, CheckpointManager, CheckpointError};
pub use stage::{StageExecutor, DefaultStageExecutor, StageTransition};