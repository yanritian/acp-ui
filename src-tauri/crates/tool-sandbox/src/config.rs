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
        if let Some(suffix) = pattern.strip_prefix("**/") {
            return path.ends_with(suffix) || path.contains(suffix);
        }
        if let Some(prefix) = pattern.strip_suffix("/**") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_relaxed_allows_most_paths() {
        let config = SandboxConfig::default_relaxed();

        // Should allow normal paths
        assert!(config.is_path_allowed("src/main.rs"));
        assert!(config.is_path_allowed("Cargo.toml"));
        assert!(config.is_path_allowed("README.md"));

        // Should deny .env files
        assert!(!config.is_path_allowed(".env"));
        assert!(!config.is_path_allowed(".env.local"));
        assert!(!config.is_path_allowed("config/.env.production"));
    }

    #[test]
    fn default_relaxed_allows_most_commands() {
        let config = SandboxConfig::default_relaxed();

        // Should allow normal commands
        assert!(config.is_command_allowed("ls"));
        assert!(config.is_command_allowed("cargo build"));
        assert!(config.is_command_allowed("npm test"));

        // Should deny dangerous commands
        assert!(!config.is_command_allowed("rm -rf /"));
        assert!(!config.is_command_allowed("sudo rm"));
    }

    #[test]
    fn default_strict_restricts_paths() {
        let config = SandboxConfig::default_strict();

        // Should allow ./ paths
        assert!(config.is_path_allowed("./src/main.rs"));
        assert!(config.is_path_allowed("./Cargo.toml"));

        // Should deny .env
        assert!(!config.is_path_allowed(".env"));

        // Should deny absolute paths (not starting with ./)
        assert!(!config.is_path_allowed("/etc/passwd"));
        assert!(!config.is_path_allowed("C:\\Windows\\"));
    }

    #[test]
    fn default_strict_restricts_commands() {
        let config = SandboxConfig::default_strict();

        // Should allow listed commands
        assert!(config.is_command_allowed("ls"));
        assert!(config.is_command_allowed("cat"));
        assert!(config.is_command_allowed("npm"));
        assert!(config.is_command_allowed("cargo"));

        // Should deny unlisted commands
        assert!(!config.is_command_allowed("rm"));
        assert!(!config.is_command_allowed("curl"));
        assert!(!config.is_command_allowed("python"));
    }

    #[test]
    fn disabled_sandbox_allows_all() {
        let mut config = SandboxConfig::default_relaxed();
        config.enabled = false;

        // Everything should be allowed when sandbox is disabled
        assert!(config.is_path_allowed(".env"));
        assert!(config.is_path_allowed("/etc/passwd"));
        assert!(config.is_command_allowed("rm -rf /"));
        assert!(config.is_command_allowed("sudo"));
    }

    #[test]
    fn matches_pattern_glob_star() {
        let config = SandboxConfig::default_relaxed();

        // * should match anything
        assert!(config.is_path_allowed("anything"));
    }

    #[test]
    fn matches_pattern_double_star_prefix() {
        let config = SandboxConfig::default_strict();

        // **/secrets/** should match any path containing secrets/
        assert!(!config.is_path_allowed("config/secrets/api.key"));
        assert!(!config.is_path_allowed("secrets/password.txt"));
        assert!(!config.is_path_allowed("deep/nested/secrets/file"));
    }

    #[test]
    fn no_network_in_strict() {
        let config = SandboxConfig::default_strict();
        assert!(!config.allow_network);
    }

    #[test]
    fn network_in_relaxed() {
        let config = SandboxConfig::default_relaxed();
        assert!(config.allow_network);
    }
}