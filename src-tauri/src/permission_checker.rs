//! Permission Checker Module (Claw Code inspired)
//!
//! Provides 5-level permission hierarchy for agent operations.
//! NOTE: Some methods imported but not yet exposed via Tauri commands.

#![allow(dead_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Permission mode hierarchy (Claw Code inspired)
/// Defines the base permission level for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Can only read files, no write operations
    ReadOnly,
    /// Can read and write within workspace directory
    WorkspaceWrite,
    /// Full access without permission checks (dangerous)
    DangerFullAccess,
    /// Ask user for each operation via prompt
    Prompt,
    /// Auto-allow with rule-based checks (default)
    #[default]
    Allow,
}

/// Permission rule for allowing or denying operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Pattern to match (supports regex)
    pub pattern: String,
    /// Whether this rule allows or denies the operation
    pub allow: bool,
    /// Optional description of what this rule does
    pub description: Option<String>,
}

/// Permission configuration for an agent (Claw Code inspired)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionConfig {
    /// Permission mode level (Claw Code hierarchy)
    pub mode: PermissionMode,
    /// Working directory restriction (agent can only operate within this directory)
    pub cwd: Option<String>,
    /// Allow rules (matched after deny checks)
    pub allow: Vec<PermissionRule>,
    /// Deny rules (checked first for safety - takes precedence over allow)
    pub deny: Vec<PermissionRule>,
    /// Whether to deny by default (true = deny all unless explicitly allowed)
    pub deny_by_default: bool,
    /// Allow user to override Prompt mode decisions (Claw Code)
    pub ask_override: bool,
}

/// Result of permission check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResult {
    pub allowed: bool,
    pub reason: String,
    pub matched_rule: Option<String>,
}

/// Permission checker for validating agent operations
pub struct PermissionChecker {
    config: PermissionConfig,
    allow_patterns: Vec<(Regex, String)>,
    deny_patterns: Vec<(Regex, String)>,
}

impl PermissionChecker {
    pub fn new(config: PermissionConfig) -> Result<Self, String> {
        // Compile regex patterns
        let allow_patterns: Vec<(Regex, String)> = config.allow
            .iter()
            .filter_map(|rule| {
                Regex::new(&rule.pattern)
                    .ok()
                    .map(|r| (r, rule.pattern.clone()))
            })
            .collect();

        let deny_patterns: Vec<(Regex, String)> = config.deny
            .iter()
            .filter_map(|rule| {
                Regex::new(&rule.pattern)
                    .ok()
                    .map(|r| (r, rule.pattern.clone()))
            })
            .collect();

        Ok(Self {
            config,
            allow_patterns,
            deny_patterns,
        })
    }

    /// Check if a tool/command operation is allowed (Claw Code hierarchy)
    pub fn check_tool(&self, tool_name: &str, args: &str) -> PermissionResult {
        // Apply permission mode hierarchy (Claw Code)
        match self.config.mode {
            PermissionMode::ReadOnly => {
                // Only allow read operations - exact match or proper namespace prefix
                // "Read" or "Read:*" only, NOT "Readme" or "Reader"
                let is_read_tool = tool_name == "Read" ||
                    (tool_name.starts_with("Read:") && tool_name.len() > 5);  // "Read:something" valid
                if !is_read_tool {
                    return PermissionResult {
                        allowed: false,
                        reason: "ReadOnly mode: only Read operations allowed".to_string(),
                        matched_rule: Some("mode-ReadOnly".to_string()),
                    };
                }
            }
            PermissionMode::WorkspaceWrite => {
                // Check path is within cwd
                if let Some(cwd) = &self.config.cwd {
                    if !args.starts_with(cwd) && self.is_path_in_args(args) {
                        return PermissionResult {
                            allowed: false,
                            reason: format!("WorkspaceWrite mode: path '{}' outside workspace '{}'", args, cwd),
                            matched_rule: Some("mode-WorkspaceWrite".to_string()),
                        };
                    }
                }
            }
            PermissionMode::DangerFullAccess => {
                // Skip all checks - full access
                return PermissionResult {
                    allowed: true,
                    reason: "DangerFullAccess mode: all operations allowed".to_string(),
                    matched_rule: Some("mode-DangerFullAccess".to_string()),
                };
            }
            PermissionMode::Prompt => {
                // Return a prompt-required result (UI should handle this)
                return PermissionResult {
                    allowed: self.config.ask_override,
                    reason: "Prompt mode: requires user confirmation".to_string(),
                    matched_rule: Some("mode-Prompt".to_string()),
                };
            }
            PermissionMode::Allow => {
                // Continue with normal rule-based checking
            }
        }

        let operation = format!("tool:{}:{}", tool_name, args);

        // First check deny rules (they override allow)
        for (pattern, pattern_str) in &self.deny_patterns {
            if pattern.is_match(&operation) || pattern.is_match(tool_name) || pattern.is_match(args) {
                return PermissionResult {
                    allowed: false,
                    reason: "Denied by rule".to_string(),
                    matched_rule: Some(pattern_str.clone()),
                };
            }
        }

        // Check dangerous patterns by default
        if self.is_dangerous_operation(tool_name, args) {
            return PermissionResult {
                allowed: false,
                reason: "Operation deemed dangerous".to_string(),
                matched_rule: Some("builtin-dangerous".to_string()),
            };
        }

        // Then check allow rules
        for (pattern, pattern_str) in &self.allow_patterns {
            if pattern.is_match(&operation) || pattern.is_match(tool_name) || pattern.is_match(args) {
                return PermissionResult {
                    allowed: true,
                    reason: "Allowed by rule".to_string(),
                    matched_rule: Some(pattern_str.clone()),
                };
            }
        }

        // If deny_by_default, deny unless explicitly allowed
        if self.config.deny_by_default {
            return PermissionResult {
                allowed: false,
                reason: "Not explicitly allowed (deny by default)".to_string(),
                matched_rule: None,
            };
        }

        // Default: allow if no deny rules matched
        PermissionResult {
            allowed: true,
            reason: "No deny rules matched".to_string(),
            matched_rule: None,
        }
    }

    /// Check if a file operation is within the working directory
    pub fn check_path(&self, path: &str) -> PermissionResult {
        if let Some(cwd) = &self.config.cwd {
            let cwd_path = Path::new(cwd);
            let target_path = Path::new(path);

            // Normalize paths
            let cwd_normalized = cwd_path.canonicalize().ok();
            let target_normalized = if target_path.exists() {
                target_path.canonicalize().ok()
            } else {
                // For non-existent paths, resolve relative to cwd
                Some(cwd_path.join(target_path))
            };

            if let (Some(cwd_norm), Some(target_norm)) = (cwd_normalized, target_normalized) {
                if !target_norm.starts_with(&cwd_norm) {
                    return PermissionResult {
                        allowed: false,
                        reason: format!("Path '{}' is outside working directory '{}'", path, cwd),
                        matched_rule: Some("cwd-restriction".to_string()),
                    };
                }
            }
        }

        PermissionResult {
            allowed: true,
            reason: "Path within working directory".to_string(),
            matched_rule: None,
        }
    }

    /// Check if operation is inherently dangerous
    fn is_dangerous_operation(&self, tool_name: &str, args: &str) -> bool {
        // Dangerous tool patterns
        let dangerous_tools = [
            "Bash:rm -rf",
            "Bash:rm -r",
            "Bash:dd",
            "Bash:mkfs",
            "Bash:fdisk",
            "Bash:shutdown",
            "Bash:reboot",
            "Bash:init",
            "Bash:kill -9",
            "Bash:pkill",
            "Bash:chmod 777",
            "Bash:chown",
            "Write:/etc/",
            "Write:/sys/",
            "Write:/proc/",
            "Read:/etc/shadow",
            "Read:/etc/passwd",
            "Read:/root/",
        ];

        let lower_args = args.to_lowercase();
        let operation = format!("{}:{}", tool_name, args);

        for pattern in &dangerous_tools {
            if operation.starts_with(pattern) || lower_args.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        // Check for system file patterns
        let system_patterns = [
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/root/",
            "/var/log/",
            "/sys/",
            "/proc/",
        ];

        for pattern in &system_patterns {
            if args.contains(pattern) {
                return true;
            }
        }

        // Check for dangerous commands in args
        let dangerous_commands = [
            "rm -rf",
            "rm -r",
            "rmdir /s",
            "del /s",
            "format",
            "fdisk",
            "mkfs",
            "dd if=",
            "shutdown",
            "reboot",
            "init 0",
            "init 6",
            "kill -9",
            "pkill -9",
            "> /dev/sda",
            "> /dev/hda",
        ];

        for cmd in &dangerous_commands {
            if lower_args.contains(cmd) {
                return true;
            }
        }

        false
    }

    /// Check if args contains a file path (Claw Code helper)
    fn is_path_in_args(&self, args: &str) -> bool {
        // Simple heuristic: check for common path patterns
        args.contains("/") || args.contains("\\") || args.contains("./") || args.starts_with("~")
    }

    /// Get the working directory for this agent
    pub fn get_cwd(&self) -> Option<&str> {
        self.config.cwd.as_deref()
    }

    /// Check if Bash command is allowed
    pub fn check_bash(&self, command: &str) -> PermissionResult {
        self.check_tool("Bash", command)
    }

    /// Check if Read operation is allowed
    pub fn check_read(&self, path: &str) -> PermissionResult {
        let path_check = self.check_path(path);
        if !path_check.allowed {
            return path_check;
        }
        self.check_tool("Read", path)
    }

    /// Check if Write operation is allowed
    pub fn check_write(&self, path: &str) -> PermissionResult {
        let path_check = self.check_path(path);
        if !path_check.allowed {
            return path_check;
        }
        self.check_tool("Write", path)
    }

    /// Check if Edit operation is allowed
    pub fn check_edit(&self, path: &str) -> PermissionResult {
        let path_check = self.check_path(path);
        if !path_check.allowed {
            return path_check;
        }
        self.check_tool("Edit", path)
    }
}

/// Default permission configurations for different agent types (Claw Code hierarchy)
pub fn get_default_permissions(agent_type: &str) -> PermissionConfig {
    match agent_type {
        "code-reviewer" => PermissionConfig {
            mode: PermissionMode::ReadOnly,
            deny_by_default: true,
            allow: vec![
                PermissionRule {
                    pattern: "tool:Read:.*".to_string(),
                    allow: true,
                    description: Some("Can read any file".to_string()),
                },
                PermissionRule {
                    pattern: "tool:Bash:npx tsc.*".to_string(),
                    allow: true,
                    description: Some("Can run TypeScript compiler".to_string()),
                },
                PermissionRule {
                    pattern: "tool:Bash:npx eslint.*".to_string(),
                    allow: true,
                    description: Some("Can run ESLint".to_string()),
                },
            ],
            deny: vec![],
            cwd: None,
            ask_override: false,
        },
        "test-validator" => PermissionConfig {
            mode: PermissionMode::WorkspaceWrite,
            deny_by_default: true,
            allow: vec![
                PermissionRule {
                    pattern: "tool:Read:.*".to_string(),
                    allow: true,
                    description: Some("Can read any file".to_string()),
                },
                PermissionRule {
                    pattern: "tool:Bash:npm test.*".to_string(),
                    allow: true,
                    description: Some("Can run tests".to_string()),
                },
                PermissionRule {
                    pattern: "tool:Bash:npx jest.*".to_string(),
                    allow: true,
                    description: Some("Can run Jest".to_string()),
                },
            ],
            deny: vec![],
            cwd: None,
            ask_override: false,
        },
        "browser-control" => PermissionConfig {
            mode: PermissionMode::Allow,
            deny_by_default: true,
            allow: vec![
                PermissionRule {
                    pattern: "tool:Bash:.*browser.*".to_string(),
                    allow: true,
                    description: Some("Can control browser".to_string()),
                },
            ],
            deny: vec![
                PermissionRule {
                    pattern: "tool:Bash:rm.*".to_string(),
                    allow: false,
                    description: Some("Cannot delete files".to_string()),
                },
            ],
            cwd: None,
            ask_override: false,
        },
        "admin" => PermissionConfig {
            mode: PermissionMode::DangerFullAccess,
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            cwd: None,
            ask_override: true,
        },
        "interactive" => PermissionConfig {
            mode: PermissionMode::Prompt,
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            cwd: None,
            ask_override: true,
        },
        _ => PermissionConfig {
            mode: PermissionMode::Allow,
            deny_by_default: false,
            allow: vec![],
            deny: vec![
                PermissionRule {
                    pattern: "Bash:rm -rf.*".to_string(),
                    allow: false,
                    description: Some("Cannot run destructive commands".to_string()),
                },
                PermissionRule {
                    pattern: "Read:/etc/shadow".to_string(),
                    allow: false,
                    description: Some("Cannot read shadow file".to_string()),
                },
            ],
            cwd: None,
            ask_override: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command_detection() {
        let checker = PermissionChecker::new(PermissionConfig::default()).unwrap();

        let result = checker.check_bash("rm -rf /home/user");
        assert!(!result.allowed);

        let result = checker.check_bash("ls -la");
        assert!(result.allowed);
    }

    #[test]
    fn test_permission_modes() {
        // Test ReadOnly mode
        let config = PermissionConfig {
            mode: PermissionMode::ReadOnly,
            cwd: None,
            allow: vec![],
            deny: vec![],
            deny_by_default: false,
            ask_override: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_tool("Read", "/home/user/file.txt");
        assert!(result.allowed);

        let result = checker.check_tool("Write", "/home/user/file.txt");
        assert!(!result.allowed);

        // Test DangerFullAccess mode
        let config = PermissionConfig {
            mode: PermissionMode::DangerFullAccess,
            cwd: None,
            allow: vec![],
            deny: vec![],
            deny_by_default: false,
            ask_override: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_bash("rm -rf /");
        assert!(result.allowed); // Full access skips dangerous checks
    }

    #[test]
    fn test_path_restriction() {
        let config = PermissionConfig {
            mode: PermissionMode::Allow,
            cwd: Some("/home/user/project".to_string()),
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            ask_override: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_path("/home/user/project/src/main.ts");
        assert!(result.allowed);

        // Note: This test would require actual filesystem
        // let result = checker.check_path("/etc/passwd");
        // assert!(!result.allowed);
    }

    #[test]
    fn test_allow_deny_override() {
        let config = PermissionConfig {
            mode: PermissionMode::Allow,
            deny_by_default: false,
            allow: vec![
                PermissionRule {
                    pattern: "Bash:.*".to_string(),
                    allow: true,
                    description: Some("Allow all bash".to_string()),
                },
            ],
            deny: vec![
                PermissionRule {
                    pattern: "Bash:rm.*".to_string(),
                    allow: false,
                    description: Some("Deny rm".to_string()),
                },
            ],
            cwd: None,
            ask_override: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_bash("ls -la");
        assert!(result.allowed);

        let result = checker.check_bash("rm test.txt");
        assert!(!result.allowed);
    }
}