// Game Error Handler
// Phase 4 Day 11: Comprehensive error handling and user experience

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Error Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameErrorType {
    DetectionFailed,
    BuildFailed,
    LaunchFailed,
    ProcessCrash,
    MissingDependencies,
    InvalidProject,
    PermissionDenied,
    DiskSpaceInsufficient,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameError {
    pub error_type: GameErrorType,
    pub message: String,
    pub details: Option<String>,
    pub suggestion: Option<String>,
    pub recoverable: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub engine: Option<String>,
    pub project_path: Option<String>,
    pub build_target: Option<String>,
    pub process_id: Option<u32>,
}

// ===== Error Handler =====

pub struct GameErrorHandler {
    error_history: Vec<GameError>,
    known_issues: HashMap<String, ErrorSolution>,
}

#[derive(Debug, Clone)]
pub struct ErrorSolution {
    pub description: String,
    pub steps: Vec<String>,
    pub commands: Vec<String>,
}

impl GameErrorHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            error_history: Vec::new(),
            known_issues: HashMap::new(),
        };
        handler.init_known_issues();
        handler
    }

    fn init_known_issues(&mut self) {
        // Godot export template missing
        self.known_issues.insert(
            "export_template_missing".to_string(),
            ErrorSolution {
                description: "Godot export templates are not installed".to_string(),
                steps: vec![
                    "Open Godot Editor".to_string(),
                    "Go to Editor → Manage Export Templates".to_string(),
                    "Click 'Download and Install'".to_string(),
                    "Wait for download to complete".to_string(),
                ],
                commands: vec!["godot --install-export-templates".to_string()],
            },
        );

        // Unity not found
        self.known_issues.insert(
            "unity_not_found".to_string(),
            ErrorSolution {
                description: "Unity Editor is not installed or not in PATH".to_string(),
                steps: vec![
                    "Install Unity Hub from https://unity.com/download".to_string(),
                    "Install Unity Editor via Unity Hub".to_string(),
                    "Set UNITY_PATH environment variable".to_string(),
                    "Example: UNITY_PATH=C:\\Program Files\\Unity\\Hub\\Editor\\2022.3.0f1\\Editor\\Unity.exe".to_string(),
                ],
                commands: vec![
                    "set UNITY_PATH=\"C:\\path\\to\\Unity.exe\"".to_string(),
                ],
            },
        );

        // Godot not found
        self.known_issues.insert(
            "godot_not_found".to_string(),
            ErrorSolution {
                description: "Godot Engine is not installed or not in PATH".to_string(),
                steps: vec![
                    "Download Godot from https://godotengine.org/download".to_string(),
                    "Extract to a permanent location".to_string(),
                    "Add to PATH environment variable".to_string(),
                    "Or set GODOT_PATH environment variable".to_string(),
                ],
                commands: vec!["set GODOT_PATH=\"C:\\path\\to\\godot.exe\"".to_string()],
            },
        );

        // Compilation error
        self.known_issues.insert(
            "compilation_error".to_string(),
            ErrorSolution {
                description: "Script compilation failed".to_string(),
                steps: vec![
                    "Check the build logs for specific errors".to_string(),
                    "Open the project in the game editor".to_string(),
                    "Fix compilation errors in the script files".to_string(),
                    "Try building again".to_string(),
                ],
                commands: vec![],
            },
        );

        // Disk space insufficient
        self.known_issues.insert(
            "disk_space_insufficient".to_string(),
            ErrorSolution {
                description: "Not enough disk space for build".to_string(),
                steps: vec![
                    "Free up disk space (recommended: 2GB+)".to_string(),
                    "Delete temporary files".to_string(),
                    "Move large files to another drive".to_string(),
                    "Try building to a different output directory".to_string(),
                ],
                commands: vec!["cleanmgr".to_string()],
            },
        );
    }

    pub fn handle_error(&mut self, error: GameError, context: &ErrorContext) -> GameError {
        // Try to find a known solution
        let error_key = self.identify_error(&error, context);

        let mut enhanced_error = error.clone();

        if let Some(solution) = self.known_issues.get(&error_key) {
            enhanced_error.suggestion = Some(format!(
                "{}\n\nSolution:\n{}\n\nCommands:\n{}",
                solution.description,
                solution
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{}. {}", i + 1, s))
                    .collect::<Vec<_>>()
                    .join("\n"),
                if solution.commands.is_empty() {
                    "None".to_string()
                } else {
                    solution.commands.join("\n")
                }
            ));
        }

        // Store in history
        self.error_history.push(enhanced_error.clone());

        enhanced_error
    }

    fn identify_error(&self, error: &GameError, context: &ErrorContext) -> String {
        let message_lower = error.message.to_lowercase();

        // Check for common patterns
        if message_lower.contains("export template") && message_lower.contains("not found") {
            return "export_template_missing".to_string();
        }

        if message_lower.contains("unity") && message_lower.contains("not found") {
            return "unity_not_found".to_string();
        }

        if message_lower.contains("godot") && message_lower.contains("not found") {
            return "godot_not_found".to_string();
        }

        if message_lower.contains("compilation") || message_lower.contains("compile error") {
            return "compilation_error".to_string();
        }

        if message_lower.contains("disk space") || message_lower.contains("no space left") {
            return "disk_space_insufficient".to_string();
        }

        "unknown".to_string()
    }

    pub fn get_error_history(&self) -> &[GameError] {
        &self.error_history
    }

    pub fn clear_history(&mut self) {
        self.error_history.clear();
    }

    pub fn get_frequent_errors(&self, limit: usize) -> Vec<(String, usize)> {
        let mut error_counts: HashMap<String, usize> = HashMap::new();

        for error in &self.error_history {
            let key = format!("{:?}", error.error_type);
            *error_counts.entry(key).or_insert(0) += 1;
        }

        let mut sorted: Vec<_> = error_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }
}

// ===== Helper Functions =====

pub fn create_error(
    error_type: GameErrorType,
    message: impl Into<String>,
    details: Option<String>,
    recoverable: bool,
) -> GameError {
    GameError {
        error_type,
        message: message.into(),
        details,
        suggestion: None,
        recoverable,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handler() {
        let mut handler = GameErrorHandler::new();

        let error = create_error(
            GameErrorType::MissingDependencies,
            "Export template not found",
            None,
            true,
        );

        let context = ErrorContext {
            engine: Some("Godot".to_string()),
            project_path: None,
            build_target: None,
            process_id: None,
        };

        let enhanced = handler.handle_error(error, &context);
        assert!(enhanced.suggestion.is_some());
    }

    #[test]
    fn test_error_history() {
        let mut handler = GameErrorHandler::new();

        for i in 0..5 {
            let error = create_error(
                GameErrorType::BuildFailed,
                format!("Build error {}", i),
                None,
                true,
            );
            handler.handle_error(
                error,
                &ErrorContext {
                    engine: None,
                    project_path: None,
                    build_target: None,
                    process_id: None,
                },
            );
        }

        assert_eq!(handler.get_error_history().len(), 5);
    }
}
