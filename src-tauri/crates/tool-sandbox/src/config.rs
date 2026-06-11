//! Sandbox config - Tool执行沙箱配置
//!
//! 定义Tool执行的权限和限制：
//! - 允许/禁止的文件路径模式
//! - 允许/禁止的命令
//! - 资源限制（超时、内存等）

use serde::{Deserialize, Serialize};

/// Sandbox配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 是否启用沙箱
    pub enabled: bool,
    /// 允许的路径模式
    pub allowed_paths: Vec<String>,
    /// 禁止的路径模式
    pub denied_paths: Vec<String>,
    /// 允许的命令
    pub allowed_commands: Vec<String>,
    /// 禁止的命令
    pub denied_commands: Vec<String>,
    /// 默认超时（毫秒）
    pub default_timeout_ms: u64,
    /// 最大内存使用（字节）
    pub max_memory_bytes: Option<u64>,
    /// 是否允许网络访问
    pub allow_network: bool,
    /// 是否允许环境变量读取
    pub allow_env_read: bool,
    /// 是否允许环境变量写入
    pub allow_env_write: bool,
}

impl SandboxConfig {
    /// 创建默认配置（宽松模式）
    pub fn default_relaxed() -> Self {
        Self {
            enabled: true,
            allowed_paths: vec!["*".to_string()],
            denied_paths: vec![
                ".env".to_string(),
                ".env.local".to_string(),
                ".env.*.local".to_string(),
                "**/secrets/**".to_string(),
                "**/.credentials/**".to_string(),
            ],
            allowed_commands: vec!["*".to_string()],
            denied_commands: vec![
                "rm -rf /".to_string(),
                "sudo".to_string(),
                "chmod 777".to_string(),
            ],
            default_timeout_ms: 60_000,
            max_memory_bytes: None,
            allow_network: true,
            allow_env_read: true,
            allow_env_write: false,
        }
    }

    /// 创建严格配置（安全模式）
    pub fn default_strict() -> Self {
        Self {
            enabled: true,
            allowed_paths: vec!["./**".to_string()],
            denied_paths: vec![
                ".env".to_string(),
                ".env.*".to_string(),
                "**/secrets/**".to_string(),
                "**/.git/**".to_string(),
                "**/node_modules/**".to_string(),
            ],
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "echo".to_string(),
                "grep".to_string(),
                "npm".to_string(),
                "cargo".to_string(),
                "git".to_string(),
            ],
            denied_commands: vec![
                "rm".to_string(),
                "sudo".to_string(),
                "chmod".to_string(),
                "curl".to_string(),
            ],
            default_timeout_ms: 30_000,
            max_memory_bytes: Some(100_000_000), // 100MB
            allow_network: false,
            allow_env_read: false,
            allow_env_write: false,
        }
    }

    /// 检查路径是否被允许
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if !self.enabled {
            return true;
        }

        // 先检查禁止列表
        for pattern in &self.denied_paths {
            if self.matches_pattern(path, pattern) {
                return false;
            }
        }

        // 再检查允许列表
        for pattern in &self.allowed_paths {
            if self.matches_pattern(path, pattern) {
                return true;
            }
        }

        false
    }

    /// 检查命令是否被允许
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if !self.enabled {
            return true;
        }

        // 先检查禁止列表
        for pattern in &self.denied_commands {
            if command.contains(pattern) {
                return false;
            }
        }

        // 再检查允许列表
        for pattern in &self.allowed_commands {
            if pattern == "*" || command.starts_with(pattern) {
                return true;
            }
        }

        false
    }

    /// 匹配glob模式（简化实现）
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            return path.ends_with(suffix) || path.contains(suffix);
        }
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return path.starts_with(prefix);
        }
        path.contains(pattern)
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::default_relaxed()
    }
}