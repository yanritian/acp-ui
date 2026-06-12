//! Transport adapters - 传输层适配器抽象

use serde::{Deserialize, Serialize};

/// TransportAdapter trait - 传输适配器接口
pub trait TransportAdapter: Send + Sync {
    /// 发送消息
    fn send(&self, message: &str) -> Result<(), String>;

    /// 接收消息
    fn receive(&self) -> Result<String, String>;

    /// 关闭连接
    fn close(&self) -> Result<(), String>;

    /// 是否连接
    fn is_connected(&self) -> bool;
}

/// TransportConfig - 传输配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportConfig {
    /// 传输类型
    pub transport_type: TransportType,
    /// 目标地址
    pub address: String,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 是否启用TLS
    pub enable_tls: bool,
}

/// TransportType - 传输类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransportType {
    /// 标准输入输出
    Stdio,
    /// WebSocket
    WebSocket,
    /// HTTP
    Http,
    /// Tauri内部事件
    TauriEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_config_new() {
        let config = TransportConfig {
            transport_type: TransportType::Stdio,
            address: "localhost:8080".to_string(),
            timeout_ms: 30000,
            enable_tls: false,
        };
        assert_eq!(config.address, "localhost:8080");
        assert_eq!(config.timeout_ms, 30000);
    }

    #[test]
    fn transport_config_serde_roundtrip() {
        let config = TransportConfig {
            transport_type: TransportType::WebSocket,
            address: "ws://example.com".to_string(),
            timeout_ms: 60000,
            enable_tls: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: TransportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.transport_type, TransportType::WebSocket);
        assert_eq!(decoded.address, "ws://example.com");
        assert_eq!(decoded.timeout_ms, 60000);
        assert!(decoded.enable_tls);
    }

    #[test]
    fn transport_type_serde_roundtrip() {
        let types = vec![TransportType::Stdio, TransportType::WebSocket, TransportType::Http, TransportType::TauriEvent];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let decoded: TransportType = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, t);
        }
    }
}