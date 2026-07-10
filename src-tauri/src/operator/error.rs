// Unified Error Handling for Hermes Game Operator

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Error Categories
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    Validation,
    Security,
    Network,
    FileSystem,
    Agent,
    State,
    Approval,
    Config,
    Unknown,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Security => write!(f, "Security"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::FileSystem => write!(f, "FileSystem"),
            ErrorCategory::Agent => write!(f, "Agent"),
            ErrorCategory::State => write!(f, "State"),
            ErrorCategory::Approval => write!(f, "Approval"),
            ErrorCategory::Config => write!(f, "Config"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

// ============================================================================
// Error Severity
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ============================================================================
// Operator Error
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorError {
    pub code: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub message: String,
    pub details: Option<String>,
    pub source: Option<String>,
    pub timestamp: String,
    pub task_id: Option<String>,
    pub recoverable: bool,
}

impl OperatorError {
    pub fn new(
        code: &str,
        category: ErrorCategory,
        severity: ErrorSeverity,
        message: &str,
    ) -> Self {
        Self {
            code: code.to_string(),
            category,
            severity,
            message: message.to_string(),
            details: None,
            source: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            task_id: None,
            recoverable: false,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    pub fn with_task_id(mut self, task_id: &str) -> Self {
        self.task_id = Some(task_id.to_string());
        self
    }

    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    /// Convert to user-friendly message
    pub fn to_user_message(&self) -> String {
        match self.severity {
            ErrorSeverity::Info => format!("ℹ️ {}", self.message),
            ErrorSeverity::Warning => format!("⚠️ {}", self.message),
            ErrorSeverity::Error => format!("❌ {}", self.message),
            ErrorSeverity::Critical => format!("🚨 {}", self.message),
        }
    }

    /// Convert to log message
    pub fn to_log_message(&self) -> String {
        format!(
            "[{}] [{}] [{}] {}: {}",
            self.timestamp, self.severity, self.category, self.code, self.message
        )
    }
}

impl fmt::Display for OperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_log_message())
    }
}

impl std::error::Error for OperatorError {}

// ============================================================================
// Error Codes
// ============================================================================

pub mod codes {
    // Validation errors (1xxx)
    pub const INVALID_PATH: &str = "1001";
    pub const INVALID_COMMAND: &str = "1002";
    pub const INVALID_INPUT: &str = "1003";
    pub const MISSING_REQUIRED_FIELD: &str = "1004";

    // Security errors (2xxx)
    pub const PATH_OUTSIDE_BOUNDARY: &str = "2001";
    pub const FORBIDDEN_OPERATION: &str = "2002";
    pub const PERMISSION_DENIED: &str = "2003";
    pub const COMMAND_NOT_ALLOWED: &str = "2004";

    // Network errors (3xxx)
    pub const API_UNAVAILABLE: &str = "3001";
    pub const NETWORK_TIMEOUT: &str = "3002";
    pub const CONNECTION_FAILED: &str = "3003";

    // FileSystem errors (4xxx)
    pub const FILE_NOT_FOUND: &str = "4001";
    pub const FILE_READ_ERROR: &str = "4002";
    pub const FILE_WRITE_ERROR: &str = "4003";
    pub const DIRECTORY_NOT_FOUND: &str = "4004";

    // Agent errors (5xxx)
    pub const AGENT_NOT_INITIALIZED: &str = "5001";
    pub const AGENT_EXECUTION_FAILED: &str = "5002";
    pub const AGENT_TIMEOUT: &str = "5003";

    // State errors (6xxx)
    pub const INVALID_STATE_TRANSITION: &str = "6001";
    pub const TASK_NOT_FOUND: &str = "6002";
    pub const TASK_ALREADY_COMPLETED: &str = "6003";

    // Approval errors (7xxx)
    pub const APPROVAL_NOT_FOUND: &str = "7001";
    pub const APPROVAL_TIMEOUT: &str = "7002";
    pub const APPROVAL_REJECTED: &str = "7003";

    // Config errors (8xxx)
    pub const CONFIG_INVALID: &str = "8001";
    pub const CONFIG_MISSING: &str = "8002";
}

// ============================================================================
// Error Context
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub task_id: Option<String>,
    pub step_id: Option<u32>,
    pub file_path: Option<String>,
    pub operation: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            task_id: None,
            step_id: None,
            file_path: None,
            operation: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_task_id(mut self, task_id: &str) -> Self {
        self.task_id = Some(task_id.to_string());
        self
    }

    pub fn with_step_id(mut self, step_id: u32) -> Self {
        self.step_id = Some(step_id);
        self
    }

    pub fn with_file_path(mut self, file_path: &str) -> Self {
        self.file_path = Some(file_path.to_string());
        self
    }

    pub fn with_operation(mut self, operation: &str) -> Self {
        self.operation = Some(operation.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Error Handler
// ============================================================================

pub struct ErrorHandler {
    errors: Vec<OperatorError>,
    max_errors: usize,
}

impl ErrorHandler {
    pub fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            max_errors,
        }
    }

    pub fn report(&mut self, error: OperatorError) {
        if self.errors.len() >= self.max_errors {
            self.errors.remove(0);
        }
        self.errors.push(error);
    }

    pub fn get_all(&self) -> &[OperatorError] {
        &self.errors
    }

    pub fn get_by_category(&self, category: ErrorCategory) -> Vec<&OperatorError> {
        self.errors
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn get_by_severity(&self, severity: ErrorSeverity) -> Vec<&OperatorError> {
        self.errors
            .iter()
            .filter(|e| e.severity == severity)
            .collect()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }

    pub fn count(&self) -> usize {
        self.errors.len()
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(1000)
    }
}
