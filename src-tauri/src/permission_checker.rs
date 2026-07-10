//! Permission Checker Module (Claw Code inspired)
//!
//! Provides 5-level permission hierarchy for agent operations.
//! Includes standard Autonomy Level classification (Level 0-4) per MVP Blueprint.
//! NOTE: Some methods imported but not yet exposed via Tauri commands.

#![allow(dead_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Autonomy Level classification (MVP Blueprint standard)
/// Defines how much autonomous action an agent can take without human approval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AutonomyLevel {
    /// Level 0: Answer-only - Agent can only read and answer, no actions
    Level0AnswerOnly,
    /// Level 1: Draft-only - Agent drafts outputs, humans commit all changes
    Level1DraftOnly,
    /// Level 2: Approval-gated - Agent proposes actions, pauses for approval before side effects
    #[default]
    Level2ApprovalGated,
    /// Level 3: Policy-bounded - Agent executes low-risk actions inside explicit policy
    Level3PolicyBounded,
    /// Level 4: Autonomous - Agent pursues measurable goals across checkpoints and budgets
    Level4Autonomous,
}

impl AutonomyLevel {
    /// Get description of this autonomy level
    pub fn description(&self) -> &'static str {
        match self {
            AutonomyLevel::Level0AnswerOnly => {
                "Manual approval for all actions. Agent reads and answers only."
            }
            AutonomyLevel::Level1DraftOnly => {
                "Approval for write/external actions. Agent drafts, humans commit."
            }
            AutonomyLevel::Level2ApprovalGated => {
                "Approval only for destructive/external actions. Agent can read/write within scope."
            }
            AutonomyLevel::Level3PolicyBounded => {
                "Budget-based gates only. Agent executes within policy boundaries."
            }
            AutonomyLevel::Level4Autonomous => {
                "Full autonomy with logging. Agent pursues goals with budgets and checkpoints."
            }
        }
    }

    /// Check if planning mode is required for this level
    pub fn requires_planning_mode(&self) -> bool {
        matches!(
            self,
            AutonomyLevel::Level0AnswerOnly
                | AutonomyLevel::Level1DraftOnly
                | AutonomyLevel::Level2ApprovalGated
        )
    }

    /// Check if mutation tools are blocked in this level
    pub fn blocks_mutation_tools(&self) -> bool {
        matches!(
            self,
            AutonomyLevel::Level0AnswerOnly | AutonomyLevel::Level1DraftOnly
        )
    }
}

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

/// Map PermissionMode to AutonomyLevel for standard classification
impl PermissionMode {
    pub fn to_autonomy_level(&self) -> AutonomyLevel {
        match self {
            PermissionMode::ReadOnly => AutonomyLevel::Level0AnswerOnly,
            PermissionMode::WorkspaceWrite => AutonomyLevel::Level1DraftOnly,
            PermissionMode::Prompt => AutonomyLevel::Level2ApprovalGated,
            PermissionMode::Allow => AutonomyLevel::Level3PolicyBounded,
            PermissionMode::DangerFullAccess => AutonomyLevel::Level4Autonomous,
        }
    }
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
    /// Autonomy level (MVP Blueprint standard Level 0-4)
    pub autonomy_level: Option<AutonomyLevel>,
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
    /// Planning mode flag - blocks all mutation tools during planning
    /// When true: write_workspace, destructive_action, external_send tools are blocked
    pub is_planning_mode: bool,
}

impl PermissionConfig {
    /// Get effective autonomy level (explicit or derived from mode)
    pub fn get_autonomy_level(&self) -> AutonomyLevel {
        self.autonomy_level
            .clone()
            .unwrap_or_else(|| self.mode.to_autonomy_level())
    }

    /// Check if mutation tools should be blocked based on autonomy level or planning mode
    pub fn should_block_mutation(&self) -> bool {
        self.is_planning_mode || self.get_autonomy_level().blocks_mutation_tools()
    }
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
        let allow_patterns: Vec<(Regex, String)> = config
            .allow
            .iter()
            .filter_map(|rule| {
                Regex::new(&rule.pattern)
                    .ok()
                    .map(|r| (r, rule.pattern.clone()))
            })
            .collect();

        let deny_patterns: Vec<(Regex, String)> = config
            .deny
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

    /// Check if a tool/command operation is allowed (Claw Code hierarchy + MVP Blueprint)
    pub fn check_tool(&self, tool_name: &str, args: &str) -> PermissionResult {
        // MVP Blueprint: Check planning mode and autonomy level first
        if self.config.should_block_mutation() {
            // Block mutation tools in planning mode or low autonomy levels
            if self.is_mutation_tool(tool_name) {
                return PermissionResult {
                    allowed: false,
                    reason: if self.config.is_planning_mode {
                        "Planning mode: mutation tools blocked until approval".to_string()
                    } else {
                        format!(
                            "Autonomy level {:?}: mutation tools require approval",
                            self.config.get_autonomy_level()
                        )
                    },
                    matched_rule: Some("planning-mode-block".to_string()),
                };
            }
        }

        // Apply permission mode hierarchy (Claw Code)
        match self.config.mode {
            PermissionMode::ReadOnly => {
                // Only allow read operations - exact match or proper namespace prefix
                // "Read" or "Read:*" only, NOT "Readme" or "Reader"
                let is_read_tool =
                    tool_name == "Read" || (tool_name.starts_with("Read:") && tool_name.len() > 5); // "Read:something" valid
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
                            reason: format!(
                                "WorkspaceWrite mode: path '{}' outside workspace '{}'",
                                args, cwd
                            ),
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
            if pattern.is_match(&operation) || pattern.is_match(tool_name) || pattern.is_match(args)
            {
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
            if pattern.is_match(&operation) || pattern.is_match(tool_name) || pattern.is_match(args)
            {
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

    /// Check if tool is a mutation tool (should be blocked in planning mode)
    /// MVP Blueprint: mutation tools include write, destructive, external_send
    fn is_mutation_tool(&self, tool_name: &str) -> bool {
        // Mutation tool categories (from MVP Blueprint checklist)
        let mutation_tools = [
            // Write operations
            "Write",
            "Write:",
            "Edit",
            "Edit:",
            "write_file",
            "edit_file",
            "create_file",
            // Destructive operations
            "Delete",
            "Delete:",
            "delete_file",
            "rm",
            // External send operations
            "Send",
            "Send:",
            "send_message",
            "sendMessage",
            // Shell execution (can have mutation side effects)
            "Bash",
            "bash_command",
            "execute",
            "shell",
            // Database mutations
            "Update",
            "Update:",
            "Insert",
            "Insert:",
            // Permission changes
            "Chmod",
            "chmod",
            "Chown",
            "chown",
        ];

        for pattern in &mutation_tools {
            if tool_name == *pattern || tool_name.starts_with(pattern) {
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

/// Default permission configurations for different agent types (Claw Code hierarchy + MVP Blueprint)
pub fn get_default_permissions(agent_type: &str) -> PermissionConfig {
    match agent_type {
        "code-reviewer" => PermissionConfig {
            mode: PermissionMode::ReadOnly,
            autonomy_level: Some(AutonomyLevel::Level0AnswerOnly),
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
            is_planning_mode: true, // Reviewers work in planning mode
        },
        "test-validator" => PermissionConfig {
            mode: PermissionMode::WorkspaceWrite,
            autonomy_level: Some(AutonomyLevel::Level1DraftOnly),
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
            is_planning_mode: false,
        },
        "browser-control" => PermissionConfig {
            mode: PermissionMode::Allow,
            autonomy_level: Some(AutonomyLevel::Level3PolicyBounded),
            deny_by_default: true,
            allow: vec![PermissionRule {
                pattern: "tool:Bash:.*browser.*".to_string(),
                allow: true,
                description: Some("Can control browser".to_string()),
            }],
            deny: vec![PermissionRule {
                pattern: "tool:Bash:rm.*".to_string(),
                allow: false,
                description: Some("Cannot delete files".to_string()),
            }],
            cwd: None,
            ask_override: false,
            is_planning_mode: false,
        },
        "admin" => PermissionConfig {
            mode: PermissionMode::DangerFullAccess,
            autonomy_level: Some(AutonomyLevel::Level4Autonomous),
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            cwd: None,
            ask_override: true,
            is_planning_mode: false,
        },
        "interactive" => PermissionConfig {
            mode: PermissionMode::Prompt,
            autonomy_level: Some(AutonomyLevel::Level2ApprovalGated),
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            cwd: None,
            ask_override: true,
            is_planning_mode: true, // Interactive mode starts in planning
        },
        "planner" => PermissionConfig {
            mode: PermissionMode::ReadOnly,
            autonomy_level: Some(AutonomyLevel::Level0AnswerOnly),
            deny_by_default: true,
            allow: vec![
                PermissionRule {
                    pattern: "tool:Read:.*".to_string(),
                    allow: true,
                    description: Some("Can read any file".to_string()),
                },
                PermissionRule {
                    pattern: "tool:Search:.*".to_string(),
                    allow: true,
                    description: Some("Can search content".to_string()),
                },
            ],
            deny: vec![],
            cwd: None,
            ask_override: false,
            is_planning_mode: true, // Planner always in planning mode
        },
        _ => PermissionConfig {
            mode: PermissionMode::Allow,
            autonomy_level: Some(AutonomyLevel::Level2ApprovalGated),
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
            is_planning_mode: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonomy_level_classification() {
        // Test autonomy level descriptions
        assert_eq!(
            AutonomyLevel::Level0AnswerOnly.description(),
            "Manual approval for all actions. Agent reads and answers only."
        );
        assert!(AutonomyLevel::Level0AnswerOnly.requires_planning_mode());
        assert!(AutonomyLevel::Level0AnswerOnly.blocks_mutation_tools());

        assert!(!AutonomyLevel::Level3PolicyBounded.blocks_mutation_tools());
        assert!(!AutonomyLevel::Level4Autonomous.requires_planning_mode());
    }

    #[test]
    fn test_permission_mode_to_autonomy() {
        assert_eq!(
            PermissionMode::ReadOnly.to_autonomy_level(),
            AutonomyLevel::Level0AnswerOnly
        );
        assert_eq!(
            PermissionMode::WorkspaceWrite.to_autonomy_level(),
            AutonomyLevel::Level1DraftOnly
        );
        assert_eq!(
            PermissionMode::Prompt.to_autonomy_level(),
            AutonomyLevel::Level2ApprovalGated
        );
        assert_eq!(
            PermissionMode::Allow.to_autonomy_level(),
            AutonomyLevel::Level3PolicyBounded
        );
        assert_eq!(
            PermissionMode::DangerFullAccess.to_autonomy_level(),
            AutonomyLevel::Level4Autonomous
        );
    }

    #[test]
    fn test_planning_mode_blocks_mutation() {
        let config = PermissionConfig {
            mode: PermissionMode::ReadOnly,
            autonomy_level: Some(AutonomyLevel::Level0AnswerOnly),
            cwd: None,
            allow: vec![],
            deny: vec![],
            deny_by_default: false,
            ask_override: false,
            is_planning_mode: true,
        };
        let checker = PermissionChecker::new(config).unwrap();

        // Mutation tools should be blocked
        let result = checker.check_tool("Write", "/test.txt");
        assert!(!result.allowed);
        assert!(result.reason.contains("Planning mode"));

        let result = checker.check_tool("Edit", "/test.txt");
        assert!(!result.allowed);
    }

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
            autonomy_level: None,
            cwd: None,
            allow: vec![],
            deny: vec![],
            deny_by_default: false,
            ask_override: false,
            is_planning_mode: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_tool("Read", "/home/user/file.txt");
        assert!(result.allowed);

        let result = checker.check_tool("Write", "/home/user/file.txt");
        assert!(!result.allowed);

        // Test DangerFullAccess mode
        let config = PermissionConfig {
            mode: PermissionMode::DangerFullAccess,
            autonomy_level: Some(AutonomyLevel::Level4Autonomous),
            cwd: None,
            allow: vec![],
            deny: vec![],
            deny_by_default: false,
            ask_override: false,
            is_planning_mode: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_bash("rm -rf /");
        assert!(result.allowed); // Full access skips dangerous checks
    }

    #[test]
    fn test_path_restriction() {
        let config = PermissionConfig {
            mode: PermissionMode::Allow,
            autonomy_level: None,
            cwd: Some("/home/user/project".to_string()),
            deny_by_default: false,
            allow: vec![],
            deny: vec![],
            ask_override: false,
            is_planning_mode: false,
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
            autonomy_level: None,
            deny_by_default: false,
            allow: vec![PermissionRule {
                pattern: "Bash:.*".to_string(),
                allow: true,
                description: Some("Allow all bash".to_string()),
            }],
            deny: vec![PermissionRule {
                pattern: "Bash:rm.*".to_string(),
                allow: false,
                description: Some("Deny rm".to_string()),
            }],
            cwd: None,
            ask_override: false,
            is_planning_mode: false,
        };
        let checker = PermissionChecker::new(config).unwrap();

        let result = checker.check_bash("ls -la");
        assert!(result.allowed);

        let result = checker.check_bash("rm test.txt");
        assert!(!result.allowed);
    }
}
