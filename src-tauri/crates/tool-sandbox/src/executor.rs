//! Tool executor - 安全Tool执行器

use crate::config::SandboxConfig;
use crate::patterns::DeniedPatterns;
use std::process::Command;

/// ToolExecutor - 在沙箱环境中执行Tool
pub struct ToolExecutor {
    config: SandboxConfig,
    patterns: DeniedPatterns,
}

impl ToolExecutor {
    pub fn new(config: SandboxConfig, patterns: DeniedPatterns) -> Self {
        Self { config, patterns }
    }

    pub fn with_defaults() -> Self {
        Self::new(SandboxConfig::default(), DeniedPatterns::default())
    }

    /// 执行命令（带沙箱检查）
    pub fn execute(&self, command: &str, args: &[String], cwd: Option<&str>) -> Result<String, String> {
        // 1. 检查命令是否允许
        if !self.config.is_command_allowed(command) {
            return Err(format!("Command '{}' is not allowed by sandbox", command));
        }

        // 2. 检查工作目录路径是否允许
        if let Some(dir) = cwd {
            if !self.config.is_path_allowed(dir) {
                return Err(format!("Path '{}' is not allowed by sandbox", dir));
            }
        }

        // 3. 执行命令
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// 检查文件路径是否安全
    pub fn check_path(&self, path: &str) -> Result<(), String> {
        // 检查是否被禁止模式匹配
        if let Some(pattern) = self.patterns.is_denied(path) {
            return Err(format!(
                "Path '{}' matches denied pattern '{}' (severity: {:?}, reason: {})",
                path, pattern.pattern, pattern.severity, pattern.reason
            ));
        }

        // 检查沙箱配置
        if !self.config.is_path_allowed(path) {
            return Err(format!("Path '{}' is not allowed by sandbox config", path));
        }

        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// 获取禁止模式
    pub fn patterns(&self) -> &DeniedPatterns {
        &self.patterns
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::with_defaults()
    }
}