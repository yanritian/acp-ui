// Security Module - Path and command guards for Hermes Game Operator

use std::path::{Path, PathBuf};

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
            let parent = path.parent()
                .ok_or_else(|| PathGuardError::InvalidPath(path.to_path_buf()))?;

            if !parent.exists() {
                return Err(PathGuardError::ParentNotFound(path.to_path_buf()));
            }

            let canonical_parent = parent.canonicalize()
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
            PathGuardError::ParentNotFound(p) => write!(f, "Parent directory not found: {}", p.display()),
            PathGuardError::PathOutsideBoundary { path, allowed } => {
                write!(f, "Path {} is outside allowed boundaries: {:?}",
                    path.display(),
                    allowed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>())
            }
            PathGuardError::NotAFile(p) => write!(f, "Not a file: {}", p.display()),
            PathGuardError::NotADirectory(p) => write!(f, "Not a directory: {}", p.display()),
            PathGuardError::ForbiddenDirectory(d) => write!(f, "Cannot write to forbidden directory: {}", d),
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
            return Err(CommandGuardError::ForbiddenCommand(base_command.to_string()));
        }

        // Check if command is allowed
        if !self.allowed_commands.contains(&base_command.to_string()) {
            return Err(CommandGuardError::UnknownCommand(base_command.to_string()));
        }

        // Validate arguments - check for path traversal attempts
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        for arg in &args {
            if arg.contains("..") || arg.contains("~") {
                return Err(CommandGuardError::InjectionAttempt("path traversal".to_string()));
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
