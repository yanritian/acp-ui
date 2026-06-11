//! WebSocket transport - WebSocket传输层

use crate::adapters::{TransportAdapter, TransportConfig};

/// WebSocketAdapter - WebSocket连接适配器
pub struct WebSocketAdapter {
    url: String,
    connected: bool,
}

impl WebSocketAdapter {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            connected: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        // 简化实现：标记为已连接
        self.connected = true;
        Ok(())
    }
}

impl TransportAdapter for WebSocketAdapter {
    fn send(&self, message: &str) -> Result<(), String> {
        if self.connected {
            // 简化实现：返回成功
            Ok(())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    fn receive(&self) -> Result<String, String> {
        if self.connected {
            // 简化实现：返回空字符串
            Ok(String::new())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    fn close(&self) -> Result<(), String> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Default for WebSocketAdapter {
    fn default() -> Self {
        Self::new("ws://localhost:1421")
    }
}