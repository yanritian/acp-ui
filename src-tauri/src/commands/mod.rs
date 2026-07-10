pub mod agent_config;
pub mod agent_lifecycle;
pub mod config;
pub mod executive;
pub mod game_export; // Game Export Commands (Phase 2 Day 6)
pub mod game_launcher_cmds;
pub mod gateway;
pub mod godot; // Godot Game Development Commands (Phase 2)
pub mod log_stream_cmds;
pub mod memory;
pub mod permission;
pub mod renpy; // Ren'Py Visual Novel Commands (Phase 3)
pub mod self_evolution;
pub mod self_healing;
pub mod self_optimizing; // Self-Optimizing Router + Project Context (Phase 1 Week 2)
pub mod task_history;
pub mod teams;
pub mod unity; // Unity Game Development Commands (Phase 1 Day 4)
pub mod websocket_cmds; // Game Launcher Commands (Phase 3 Day 8)

// Re-export all public items from submodules for convenient access
pub use agent_config::*;
pub use agent_lifecycle::*;
pub use config::*;
pub use executive::*;
pub use game_export::*;
pub use game_launcher_cmds::*;
pub use gateway::*;
pub use godot::*;
pub use log_stream_cmds::*;
pub use memory::*;
pub use permission::*;
pub use renpy::*;
pub use self_evolution::*;
pub use self_healing::*;
pub use self_optimizing::*;
pub use task_history::*;
pub use teams::*;
pub use unity::*;
pub use websocket_cmds::*;
