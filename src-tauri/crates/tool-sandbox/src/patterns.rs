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
                // More specific patterns first (order matters!)
                DeniedPattern {
                    name: "env-production".to_string(),
                    pattern: ".env.production".to_string(),
                    reason: "Production environment files contain critical secrets".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "env-local".to_string(),
                    pattern: ".env.local".to_string(),
                    reason: "Local environment overrides may contain sensitive data".to_string(),
                    severity: Severity::Critical,
                },
                DeniedPattern {
                    name: "env-files".to_string(),
                    pattern: ".env".to_string(),
                    reason: "Environment files may contain API keys and secrets".to_string(),
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
        // Handle **/dir/** patterns (path contains dir/ anywhere)
        if pattern.starts_with("**/") && pattern.ends_with("/**") {
            let middle = &pattern[3..pattern.len()-3];
            return path.contains(&format!("{}/", middle)) || path.contains(middle);
        }
        // Handle **/*.ext patterns (path ends with .ext)
        if pattern.starts_with("**/*.") {
            let ext = &pattern[5..];
            return path.ends_with(ext);
        }
        // Handle **/name patterns (path ends with name or contains name)
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            return path.ends_with(suffix) || path.contains(&format!("/{suffix}"));
        }
        // Handle dir/** patterns (path starts with dir/)
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return path.starts_with(prefix) || path.starts_with(&format!("{}/", prefix));
        }
        // Exact match or substring match
        path == pattern || path.contains(pattern)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_patterns_count() {
        let patterns = DeniedPatterns::default_patterns();
        assert_eq!(patterns.list_patterns().len(), 8);
    }

    #[test]
    fn is_denied_env_file() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied(".env");
        assert!(result.is_some());
        let pattern = result.unwrap();
        assert_eq!(pattern.name, "env-files");
        assert_eq!(pattern.severity, Severity::Critical);
    }

    #[test]
    fn is_denied_env_local() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied(".env.local");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "env-local");
    }

    #[test]
    fn is_denied_env_production() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied(".env.production");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "env-production");
    }

    #[test]
    fn is_denied_secrets_directory() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied("config/secrets/api.key");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "secrets-dir");
    }

    #[test]
    fn is_denied_credentials_directory() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied("app/.credentials/db.json");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "credentials-dir");
    }

    #[test]
    fn is_denied_ssh_keys() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied(".ssh/id_rsa");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "ssh-keys");
    }

    #[test]
    fn is_denied_pem_file() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied("certs/server.pem");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "private-keys");
        assert_eq!(result.unwrap().severity, Severity::High);
    }

    #[test]
    fn is_not_denied_normal_file() {
        let patterns = DeniedPatterns::default_patterns();

        let result = patterns.is_denied("src/main.rs");
        assert!(result.is_none());

        let result = patterns.is_denied("README.md");
        assert!(result.is_none());

        let result = patterns.is_denied("Cargo.toml");
        assert!(result.is_none());
    }

    #[test]
    fn add_custom_pattern() {
        let mut patterns = DeniedPatterns::default_patterns();
        patterns.add_pattern(DeniedPattern {
            name: "custom-block".to_string(),
            pattern: "*.log".to_string(),
            reason: "Log files should not be accessed".to_string(),
            severity: Severity::Medium,
        });

        assert_eq!(patterns.list_patterns().len(), 9);
    }

    #[test]
    fn severity_levels() {
        assert_ne!(Severity::Low, Severity::Medium);
        assert_ne!(Severity::Medium, Severity::High);
        assert_ne!(Severity::High, Severity::Critical);
    }
}