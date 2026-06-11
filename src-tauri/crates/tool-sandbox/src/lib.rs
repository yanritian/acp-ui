//! Tool Sandbox - Secure tool execution environment
//!
//! Provides:
//! - Tool execution sandbox
//! - Denied patterns (e.g., .env files)
//! - Permission checks
//! - Resource limits

pub mod config;
pub mod executor;
pub mod permissions;
pub mod patterns;

// Re-export main types
pub use config::SandboxConfig;
pub use executor::ToolExecutor;
pub use permissions::PermissionChecker;
pub use patterns::{DeniedPatterns, DeniedPattern, Severity};