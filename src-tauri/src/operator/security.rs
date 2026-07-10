// Security Module - Path and command guards for Hermes Game Operator

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

// ============================================================================
// Path Guard
// ============================================================================

/// Validates that file operations stay within allowed boundaries
pub struct PathGuard {
    allowed_roots: Vec<PathBuf>,
}

impl PathGuard {
    pub fn new(allowed_roots: Vec<PathBuf>) -> Self {
        Self { allowed_roots }
    }

    /// Check if a path is within any allowed root
    pub fn validate_path(&self, path: &Path) -> Result<(), PathGuardError> {
        // For non-existent paths, validate parent directory exists and check against allowed roots
        let canonical = if path.exists() {
            path.canonicalize()
                .map_err(|_| PathGuardError::InvalidPath(path.to_path_buf()))?
        } else {
            // Path doesn't exist yet - validate parent directory
            let parent = path
                .parent()
                .ok_or_else(|| PathGuardError::InvalidPath(path.to_path_buf()))?;

            if !parent.exists() {
                return Err(PathGuardError::ParentNotFound(path.to_path_buf()));
            }

            let canonical_parent = parent
                .canonicalize()
                .map_err(|_| PathGuardError::InvalidPath(parent.to_path_buf()))?;

            // Check parent is within allowed boundaries
            let parent_valid = self.allowed_roots.iter().any(|root| {
                if let Ok(canonical_root) = root.canonicalize() {
                    canonical_parent.starts_with(&canonical_root)
                } else {
                    false
                }
            });

            if !parent_valid {
                return Err(PathGuardError::PathOutsideBoundary {
                    path: path.to_path_buf(),
                    allowed: self.allowed_roots.clone(),
                });
            }

            // Return success - parent is valid, so child path will be valid when created
            return Ok(());
        };

        // Check if the canonical path starts with any allowed root
        for root in &self.allowed_roots {
            if let Ok(canonical_root) = root.canonicalize() {
                if canonical.starts_with(&canonical_root) {
                    return Ok(());
                }
            }
        }

        Err(PathGuardError::PathOutsideBoundary {
            path: canonical,
            allowed: self.allowed_roots.clone(),
        })
    }

    /// Check if path is a file (not directory)
    pub fn validate_file(&self, path: &Path) -> Result<(), PathGuardError> {
        self.validate_path(path)?;

        if !path.is_file() {
            return Err(PathGuardError::NotAFile(path.to_path_buf()));
        }

        Ok(())
    }

    /// Check if path is a directory
    pub fn validate_directory(&self, path: &Path) -> Result<(), PathGuardError> {
        self.validate_path(path)?;

        if !path.is_dir() {
            return Err(PathGuardError::NotADirectory(path.to_path_buf()));
        }

        Ok(())
    }

    /// Check if writing to path is allowed
    pub fn validate_write(&self, path: &Path) -> Result<(), PathGuardError> {
        self.validate_path(path)?;

        // Prevent writing to sensitive locations
        let forbidden_dirs = [".git", "node_modules", "target", "build", "dist"];

        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if forbidden_dirs.contains(&name_str) {
                        return Err(PathGuardError::ForbiddenDirectory(name_str.to_string()));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum PathGuardError {
    InvalidPath(PathBuf),
    ParentNotFound(PathBuf),
    PathOutsideBoundary {
        path: PathBuf,
        allowed: Vec<PathBuf>,
    },
    NotAFile(PathBuf),
    NotADirectory(PathBuf),
    ForbiddenDirectory(String),
}

impl std::fmt::Display for PathGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathGuardError::InvalidPath(p) => write!(f, "Invalid path: {}", p.display()),
            PathGuardError::ParentNotFound(p) => {
                write!(f, "Parent directory not found: {}", p.display())
            }
            PathGuardError::PathOutsideBoundary { path, allowed } => {
                write!(
                    f,
                    "Path {} is outside allowed boundaries: {:?}",
                    path.display(),
                    allowed
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                )
            }
            PathGuardError::NotAFile(p) => write!(f, "Not a file: {}", p.display()),
            PathGuardError::NotADirectory(p) => write!(f, "Not a directory: {}", p.display()),
            PathGuardError::ForbiddenDirectory(d) => {
                write!(f, "Cannot write to forbidden directory: {}", d)
            }
        }
    }
}

impl std::error::Error for PathGuardError {}

// ============================================================================
// Command Guard
// ============================================================================

/// Validates shell commands against an allowlist
pub struct CommandGuard {
    allowed_commands: Vec<String>,
    forbidden_commands: Vec<String>,
}

impl CommandGuard {
    pub fn new() -> Self {
        Self {
            allowed_commands: vec![
                "godot".to_string(),
                "ls".to_string(),
                "dir".to_string(),
                "cat".to_string(),
                "type".to_string(),
                "find".to_string(),
                "grep".to_string(),
            ],
            forbidden_commands: vec![
                "rm".to_string(),
                "del".to_string(),
                "rmdir".to_string(),
                "format".to_string(),
                "mkfs".to_string(),
                "dd".to_string(),
                "sudo".to_string(),
                "su".to_string(),
                "powershell".to_string(),
                "cmd".to_string(),
            ],
        }
    }

    /// Validate a command string
    pub fn validate_command(&self, command: &str) -> Result<ValidatedCommand, CommandGuardError> {
        // Prevent injection by checking for dangerous characters first
        let dangerous_patterns = [";", "|", "&", "$", "`", "'", "\"", "\n", "\r", "..", "~"];
        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return Err(CommandGuardError::InjectionAttempt(pattern.to_string()));
            }
        }

        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return Err(CommandGuardError::EmptyCommand);
        }

        let base_command = parts[0];

        // Check if command is forbidden
        if self.forbidden_commands.contains(&base_command.to_string()) {
            return Err(CommandGuardError::ForbiddenCommand(
                base_command.to_string(),
            ));
        }

        // Check if command is allowed
        if !self.allowed_commands.contains(&base_command.to_string()) {
            return Err(CommandGuardError::UnknownCommand(base_command.to_string()));
        }

        // Validate arguments - check for path traversal attempts
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        for arg in &args {
            if arg.contains("..") || arg.contains("~") {
                return Err(CommandGuardError::InjectionAttempt(
                    "path traversal".to_string(),
                ));
            }
        }

        Ok(ValidatedCommand {
            command: base_command.to_string(),
            args,
            requires_approval: self.requires_approval(base_command),
        })
    }

    /// Check if command requires user approval
    fn requires_approval(&self, command: &str) -> bool {
        // Commands that modify state require approval
        let approval_required = ["godot"];
        approval_required.contains(&command)
    }

    /// Add a command to the allowlist
    pub fn allow_command(&mut self, command: String) {
        if !self.allowed_commands.contains(&command) {
            self.allowed_commands.push(command);
        }
    }

    /// Add a command to the forbidden list
    pub fn forbid_command(&mut self, command: String) {
        if !self.forbidden_commands.contains(&command) {
            self.forbidden_commands.push(command);
        }
    }
}

#[derive(Debug)]
pub struct ValidatedCommand {
    pub command: String,
    pub args: Vec<String>,
    pub requires_approval: bool,
}

#[derive(Debug)]
pub enum CommandGuardError {
    EmptyCommand,
    ForbiddenCommand(String),
    UnknownCommand(String),
    InjectionAttempt(String),
}

impl std::fmt::Display for CommandGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandGuardError::EmptyCommand => write!(f, "Command is empty"),
            CommandGuardError::ForbiddenCommand(cmd) => {
                write!(f, "Command is forbidden: {}", cmd)
            }
            CommandGuardError::UnknownCommand(cmd) => {
                write!(f, "Command is not in allowlist: {}", cmd)
            }
            CommandGuardError::InjectionAttempt(pattern) => {
                write!(f, "Injection attempt detected: {}", pattern)
            }
        }
    }
}

impl std::error::Error for CommandGuardError {}

// ============================================================================
// Remote Operator Access Policy
// ============================================================================

#[derive(Clone)]
pub struct RemoteAccessPolicyConfig {
    pub token: Option<String>,
    pub allowed_origins: Vec<String>,
    pub allowed_clients: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub allowed_project_roots: Vec<PathBuf>,
}

impl Default for RemoteAccessPolicyConfig {
    fn default() -> Self {
        Self {
            token: None,
            allowed_origins: vec![
                "http://tauri.localhost".to_string(),
                "https://tauri.localhost".to_string(),
                "tauri://localhost".to_string(),
                "http://127.0.0.1:1420".to_string(),
                "http://localhost:1420".to_string(),
            ],
            allowed_clients: Vec::new(),
            allowed_domains: Vec::new(),
            allowed_project_roots: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct RemoteAccessPolicy {
    token_digest: Option<[u8; 32]>,
    allowed_origins: HashSet<String>,
    allowed_clients: HashSet<String>,
    allowed_domains: HashSet<String>,
    allowed_project_roots: Vec<PathBuf>,
}

impl RemoteAccessPolicy {
    pub fn new(config: RemoteAccessPolicyConfig) -> Result<Self, RemoteSecurityConfigError> {
        let token_digest = match config.token {
            Some(token) if token.trim().is_empty() => {
                return Err(RemoteSecurityConfigError::EmptyToken)
            }
            Some(token) => Some(Sha256::digest(token.trim().as_bytes()).into()),
            None => None,
        };

        let mut allowed_project_roots = Vec::with_capacity(config.allowed_project_roots.len());
        for root in config.allowed_project_roots {
            let canonical = root
                .canonicalize()
                .map_err(|_| RemoteSecurityConfigError::InvalidProjectRoot(root.clone()))?;
            if !canonical.is_dir() {
                return Err(RemoteSecurityConfigError::InvalidProjectRoot(root));
            }
            if !allowed_project_roots.contains(&canonical) {
                allowed_project_roots.push(canonical);
            }
        }

        Ok(Self {
            token_digest,
            allowed_origins: normalize_values(config.allowed_origins, normalize_origin),
            allowed_clients: normalize_values(config.allowed_clients, normalize_value),
            allowed_domains: normalize_values(config.allowed_domains, normalize_value),
            allowed_project_roots,
        })
    }

    pub fn requires_authentication(&self) -> bool {
        self.token_digest.is_some()
    }

    pub fn allowed_origins(&self) -> impl Iterator<Item = &str> {
        self.allowed_origins.iter().map(String::as_str)
    }

    pub fn authenticate(&self, authorization: Option<&str>) -> Result<(), RemoteAccessDenied> {
        let Some(expected) = self.token_digest.as_ref() else {
            return Ok(());
        };
        let Some(provided) = authorization.and_then(|value| {
            let (scheme, token) = value.split_once(' ')?;
            scheme.eq_ignore_ascii_case("bearer").then_some(token)
        }) else {
            return Err(if authorization.is_none() {
                RemoteAccessDenied::MissingBearerToken
            } else {
                RemoteAccessDenied::InvalidBearerToken
            });
        };

        let provided_digest: [u8; 32] = Sha256::digest(provided.as_bytes()).into();
        if bool::from(expected.as_slice().ct_eq(provided_digest.as_slice())) {
            Ok(())
        } else {
            Err(RemoteAccessDenied::InvalidBearerToken)
        }
    }

    pub fn authorize_origin(&self, origin: Option<&str>) -> Result<(), RemoteAccessDenied> {
        let Some(origin) = origin else {
            return Ok(());
        };
        let origin = normalize_origin(origin);
        if !origin.is_empty() && self.allowed_origins.contains(&origin) {
            Ok(())
        } else {
            Err(RemoteAccessDenied::OriginNotAllowed)
        }
    }

    pub fn authorize_client(&self, client: Option<&str>) -> Result<(), RemoteAccessDenied> {
        if self.allowed_clients.is_empty() {
            return Ok(());
        }
        match client.map(normalize_value) {
            Some(client) if self.allowed_clients.contains(&client) => Ok(()),
            _ => Err(RemoteAccessDenied::ClientNotAllowed),
        }
    }

    pub fn authorize_domain(&self, domain: Option<&str>) -> Result<(), RemoteAccessDenied> {
        if self.allowed_domains.is_empty() {
            return Ok(());
        }
        match domain.map(normalize_value) {
            Some(domain) if self.allowed_domains.contains(&domain) => Ok(()),
            _ => Err(RemoteAccessDenied::DomainNotAllowed),
        }
    }

    pub fn authorize_project_path(&self, path: Option<&Path>) -> Result<(), RemoteAccessDenied> {
        if self.allowed_project_roots.is_empty() {
            return Ok(());
        }
        let path = path.ok_or(RemoteAccessDenied::ProjectRootNotAllowed)?;
        self.canonicalize_authorized_project_path(path).map(|_| ())
    }

    pub fn canonicalize_authorized_project_path(
        &self,
        path: &Path,
    ) -> Result<PathBuf, RemoteAccessDenied> {
        let canonical = path
            .canonicalize()
            .ok()
            .filter(|path| path.is_dir())
            .ok_or(RemoteAccessDenied::ProjectRootNotAllowed)?;
        if self.allowed_project_roots.is_empty() {
            return Ok(canonical);
        }
        if self
            .allowed_project_roots
            .iter()
            .any(|root| canonical.starts_with(root))
        {
            Ok(canonical)
        } else {
            Err(RemoteAccessDenied::ProjectRootNotAllowed)
        }
    }
}

impl fmt::Debug for RemoteAccessPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteAccessPolicy")
            .field(
                "token",
                &self
                    .token_digest
                    .as_ref()
                    .map(|_| "[REDACTED]")
                    .unwrap_or("none"),
            )
            .field("allowed_origins", &self.allowed_origins)
            .field("allowed_clients", &self.allowed_clients)
            .field("allowed_domains", &self.allowed_domains)
            .field("allowed_project_roots", &self.allowed_project_roots)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteSecurityConfigError {
    EmptyToken,
    InvalidProjectRoot(PathBuf),
}

impl fmt::Display for RemoteSecurityConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyToken => write!(f, "Remote Operator token cannot be empty"),
            Self::InvalidProjectRoot(path) => write!(
                f,
                "Remote Operator project root is not an existing directory: {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for RemoteSecurityConfigError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteAccessDenied {
    MissingBearerToken,
    InvalidBearerToken,
    OriginNotAllowed,
    ClientNotAllowed,
    DomainNotAllowed,
    ProjectRootNotAllowed,
}

impl RemoteAccessDenied {
    pub fn code(self) -> &'static str {
        match self {
            Self::MissingBearerToken | Self::InvalidBearerToken => "unauthorized",
            Self::OriginNotAllowed => "origin_not_allowed",
            Self::ClientNotAllowed => "client_not_allowed",
            Self::DomainNotAllowed => "domain_not_allowed",
            Self::ProjectRootNotAllowed => "project_root_not_allowed",
        }
    }
}

impl fmt::Display for RemoteAccessDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Remote Operator access denied: {}", self.code())
    }
}

impl std::error::Error for RemoteAccessDenied {}

fn normalize_values(values: Vec<String>, normalize: fn(&str) -> String) -> HashSet<String> {
    values
        .into_iter()
        .map(|value| normalize(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn normalize_value(value: &str) -> String {
    value.trim().to_string()
}

fn normalize_origin(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_config() -> RemoteAccessPolicyConfig {
        RemoteAccessPolicyConfig {
            token: Some("correct-horse-battery-staple".to_string()),
            allowed_origins: vec!["http://127.0.0.1:5173".to_string()],
            allowed_clients: vec!["vscode-main".to_string()],
            allowed_domains: vec!["game.godot".to_string()],
            allowed_project_roots: Vec::new(),
        }
    }

    #[test]
    fn remote_policy_requires_bearer_token_and_redacts_debug_output() {
        let policy = RemoteAccessPolicy::new(policy_config()).expect("valid policy");

        assert_eq!(
            policy.authenticate(None),
            Err(RemoteAccessDenied::MissingBearerToken)
        );
        assert_eq!(
            policy.authenticate(Some("Bearer wrong-token")),
            Err(RemoteAccessDenied::InvalidBearerToken)
        );
        assert_eq!(
            policy.authenticate(Some("Basic correct-horse-battery-staple")),
            Err(RemoteAccessDenied::InvalidBearerToken)
        );
        assert_eq!(
            policy.authenticate(Some("Bearer correct-horse-battery-staple")),
            Ok(())
        );
        assert_eq!(
            policy.authenticate(Some("bearer correct-horse-battery-staple")),
            Ok(())
        );

        let debug = format!("{policy:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("correct-horse-battery-staple"));
    }

    #[test]
    fn remote_policy_enforces_origin_client_and_domain_allowlists() {
        let policy = RemoteAccessPolicy::new(policy_config()).expect("valid policy");

        assert_eq!(policy.authorize_origin(None), Ok(()));
        assert_eq!(
            policy.authorize_origin(Some("http://127.0.0.1:5173/")),
            Ok(())
        );
        assert_eq!(
            policy.authorize_origin(Some("https://attacker.example")),
            Err(RemoteAccessDenied::OriginNotAllowed)
        );
        assert_eq!(policy.authorize_client(Some("vscode-main")), Ok(()));
        assert_eq!(
            policy.authorize_client(None),
            Err(RemoteAccessDenied::ClientNotAllowed)
        );
        assert_eq!(policy.authorize_domain(Some("game.godot")), Ok(()));
        assert_eq!(
            policy.authorize_domain(Some("shell.host")),
            Err(RemoteAccessDenied::DomainNotAllowed)
        );
    }

    #[test]
    fn remote_policy_enforces_canonical_project_root_boundaries() {
        let allowed = tempfile::tempdir().expect("allowed root");
        let nested = allowed.path().join("nested");
        std::fs::create_dir(&nested).expect("nested project");
        let denied = tempfile::tempdir().expect("denied root");
        let mut config = policy_config();
        config.allowed_project_roots = vec![allowed.path().to_path_buf()];
        let policy = RemoteAccessPolicy::new(config).expect("valid policy");

        assert_eq!(policy.authorize_project_path(Some(&nested)), Ok(()));
        assert_eq!(
            policy.authorize_project_path(Some(denied.path())),
            Err(RemoteAccessDenied::ProjectRootNotAllowed)
        );
        assert_eq!(
            policy.authorize_project_path(None),
            Err(RemoteAccessDenied::ProjectRootNotAllowed)
        );
    }

    #[test]
    fn local_default_policy_does_not_require_remote_credentials_or_scope() {
        let policy = RemoteAccessPolicy::new(RemoteAccessPolicyConfig::default())
            .expect("default local policy");

        assert_eq!(policy.authenticate(None), Ok(()));
        assert_eq!(policy.authorize_origin(None), Ok(()));
        assert_eq!(policy.authorize_client(None), Ok(()));
        assert_eq!(policy.authorize_domain(None), Ok(()));
        assert_eq!(policy.authorize_project_path(None), Ok(()));
    }

    #[test]
    fn remote_policy_rejects_empty_token_without_exposing_it() {
        let mut config = policy_config();
        config.token = Some("   ".to_string());

        let error = RemoteAccessPolicy::new(config).expect_err("empty token must fail");
        assert_eq!(error, RemoteSecurityConfigError::EmptyToken);
        assert!(!format!("{error:?}").contains("correct-horse-battery-staple"));
    }

    #[test]
    fn remote_policy_stores_only_a_fixed_size_token_digest() {
        let policy = RemoteAccessPolicy::new(policy_config()).expect("valid policy");

        assert_eq!(
            policy.token_digest.as_ref().map(|digest| digest.len()),
            Some(32)
        );
        assert!(!format!("{policy:?}").contains("correct-horse-battery-staple"));
    }
}
