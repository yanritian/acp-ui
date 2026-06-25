pub mod config;
pub mod agent_lifecycle;
pub mod task_history;
pub mod memory;
pub mod self_healing;
pub mod self_evolution;
pub mod gateway;
pub mod websocket_cmds;
pub mod log_stream_cmds;
pub mod permission;
pub mod agent_config;
pub mod executive;
pub mod teams;
pub mod self_optimizing;  // Self-Optimizing Router + Project Context (Phase 1 Week 2)
pub mod godot;            // Godot Game Development Commands (Phase 2)

// Re-export all public items from submodules for convenient access
pub use config::*;
pub use agent_lifecycle::*;
pub use task_history::*;
pub use memory::*;
pub use self_healing::*;
pub use self_evolution::*;
pub use gateway::*;
pub use websocket_cmds::*;
pub use log_stream_cmds::*;
pub use permission::*;
pub use agent_config::*;
pub use executive::*;
pub use teams::*;
pub use self_optimizing::*;
pub use godot::*;
