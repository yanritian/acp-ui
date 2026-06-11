//! FT-6: Tool Sandbox Integration Test
//!
//! Validates sandbox blocking sensitive files and dangerous commands

use tool_sandbox::{SandboxConfig, DeniedPatterns, Severity};

#[test]
fn test_strict_sandbox_blocks_env_files() {
    let config = SandboxConfig::default_strict();
    let patterns = DeniedPatterns::default_patterns();

    // .env should be blocked by both config and patterns
    assert!(!config.is_path_allowed(".env"));
    assert!(patterns.is_denied(".env").is_some());

    // .env.local should be blocked
    assert!(!config.is_path_allowed(".env.local"));
    assert!(patterns.is_denied(".env.local").is_some());

    // Verify severity
    let pattern = patterns.is_denied(".env").unwrap();
    assert_eq!(pattern.severity, Severity::Critical);
}

#[test]
fn test_strict_sandbox_blocks_secrets_directory() {
    let patterns = DeniedPatterns::default_patterns();

    // Any path with secrets/ should be blocked
    assert!(patterns.is_denied("config/secrets/api.key").is_some());
    assert!(patterns.is_denied("secrets/db_password.txt").is_some());
    assert!(patterns.is_denied("deep/nested/secrets/file.json").is_some());
}

#[test]
fn test_strict_sandbox_blocks_ssh_keys() {
    let patterns = DeniedPatterns::default_patterns();

    // SSH directory should be blocked
    assert!(patterns.is_denied(".ssh/id_rsa").is_some());
    assert!(patterns.is_denied("home/user/.ssh/config").is_some());
}

#[test]
fn test_relaxed_sandbox_blocks_dangerous_commands() {
    let config = SandboxConfig::default_relaxed();

    // Normal commands allowed
    assert!(config.is_command_allowed("npm test"));
    assert!(config.is_command_allowed("cargo build"));
    assert!(config.is_command_allowed("git status"));

    // Dangerous commands blocked
    assert!(!config.is_command_allowed("rm -rf /"));
    assert!(!config.is_command_allowed("sudo rm"));
}

#[test]
fn test_strict_sandbox_restricts_to_project_directory() {
    let config = SandboxConfig::default_strict();

    // ./ paths allowed
    assert!(config.is_path_allowed("./src/main.rs"));
    assert!(config.is_path_allowed("./Cargo.toml"));

    // Absolute paths blocked (not starting with ./)
    assert!(!config.is_path_allowed("/etc/passwd"));
    assert!(!config.is_path_allowed("C:\\Windows\\System32"));
}

#[test]
fn test_sandbox_disabled_allows_all() {
    let mut config = SandboxConfig::default_strict();
    config.enabled = false;

    // Everything allowed when disabled
    assert!(config.is_path_allowed(".env"));
    assert!(config.is_path_allowed("/etc/passwd"));
    assert!(config.is_command_allowed("rm -rf /"));
}