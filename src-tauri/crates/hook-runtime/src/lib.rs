//! Hook Runtime - Hook execution system
//!
//! Provides:
//! - PreToolUse / PostToolUse hooks
//! - Stop hooks for session end
//! - Hook registration and execution

pub mod types;
pub mod executor;
pub mod registry;

// Re-export main types
pub use types::{Hook, HookType, HookResult};
pub use executor::HookExecutor;
pub use registry::HookRegistry;