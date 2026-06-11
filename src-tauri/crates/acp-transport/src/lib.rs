//! ACP Transport - Communication layer for ACP protocol
//!
//! Provides adapters for different transport mechanisms:
//! - Stdio (process communication)
//! - WebSocket (browser/dashboard)
//! - HTTP (REST API)

pub mod adapters;
pub mod stdio;
pub mod websocket;
pub mod http;

// Re-export main types
pub use adapters::{TransportAdapter, TransportConfig, TransportType};
pub use stdio::StdioAdapter;
pub use websocket::WebSocketAdapter;
pub use http::HttpAdapter;