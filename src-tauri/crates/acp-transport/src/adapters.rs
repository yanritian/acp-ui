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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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