//! Denied patterns - 安全敏感文件拦截
//!
//! 定义不允许Tool访问的文件模式：
//! - .env 文件（包含API密钥等敏感信息）
//! - credentials 目录
//! - secrets 目录

use serde::{Deserialize, Serialize};

/// DeniedPatterns - 禁止访问的模式列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeniedPatterns {
    /// 禁止的文件模式
    patterns: Vec<DeniedPattern>,
}

/// 单个禁止模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeniedPattern {
    /// 模式名称
    pub name: String,
    /// glob模式
    pub pattern: String,
    /// 禁止原因
    pub reason: String,
    /// 严重级别
    pub severity: Severity,
}

/// 严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// 低 - 警告但不阻止
    Low,
    /// 中 - 阻止执行
    Medium,
    /// 高 - 阻止执行并记录日志
    High,
    /// 关键 - 阻止执行并通知用户
    Critical,
}

impl DeniedPatterns {
    /// 创建默认禁止模式列表
    pub fn default_patterns() -> Self {
        Self {
            patterns: vec![
                DeniedPattern {
                    name: "env-files".to_string(),
                    pattern: ".env".to_string(),
                    reason: "Environment files may contain API keys and secrets".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "env-local".to_string(),
                    pattern: ".env.local".to_string(),
                    reason: "Local environment overrides may contain sensitive data".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "env-production".to_string(),
                    pattern: ".env.production".to_string(),
                    reason: "Production environment files contain critical secrets".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "secrets-dir".to_string(),
                    pattern: "**/secrets/**".to_string(),
                    reason: "Secrets directory contains sensitive credentials".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "credentials-dir".to_string(),
                    pattern: "**/.credentials/**".to_string(),
                    reason: "Credentials directory contains authentication data".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "private-keys".to_string(),
                    pattern: "**/*.pem".to_string(),
                    reason: "PEM files may contain private keys".to_string(),
                    severity: Severity::High,
                },
                DeniedPattern {
                    name: "ssh-keys".to_string(),
                    pattern: "**/.ssh/**".to_string(),
                    reason: "SSH keys are sensitive authentication credentials".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "git-config".to_string(),
                    pattern: "**/.git/config".to_string(),
                    reason: "Git config may contain credentials".to_string(),
                    severity: Severity::Medium,
                },
            ],
        }
    }

    /// 检查路径是否被禁止
    pub fn is_denied(&self, path: &str) -> Option<&DeniedPattern> {
        for pattern in &self.patterns {
            if self.matches_pattern(path, &pattern.pattern) {
                return Some(pattern);
            }
        }
        None
    }

    /// 匹配glob模式
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            return path.ends_with(suffix) || path.contains(suffix);
        }
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return path.starts_with(prefix);
        }
        path.contains(pattern) || path == pattern
    }

    /// 添加禁止模式
    pub fn add_pattern(&mut self, pattern: DeniedPattern) {
        self.patterns.push(pattern);
    }

    /// 获取所有禁止模式
    pub fn list_patterns(&self) -> &[DeniedPattern] {
        &self.patterns
    }
}

impl Default for DeniedPatterns {
    fn default() -> Self {
        Self::default_patterns()
    }
}