//! Condition Inference - Infer completion conditions from task description

use acp_core::CompletionConditionSpec;
use regex::Regex;

/// Condition inference engine
pub struct ConditionInference {
    /// Patterns for condition inference
    patterns: Vec<(Regex, CompletionConditionSpec)>,
}

impl ConditionInference {
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
        }
    }

    fn build_patterns() -> Vec<(Regex, CompletionConditionSpec)> {
        vec![
            // Build/compile tasks
            (
                Regex::new(r"(build|compile|make)").unwrap(),
                CompletionConditionSpec::CommandSuccess {
                    command: "cargo build".to_string(),
                },
            ),
            // Test tasks
            (
                Regex::new(r"(test|tests|testing)").unwrap(),
                CompletionConditionSpec::CommandSuccess {
                    command: "cargo test".to_string(),
                },
            ),
            // Lint/format tasks
            (
                Regex::new(r"(lint|format|style)").unwrap(),
                CompletionConditionSpec::CommandSuccess {
                    command: "cargo clippy".to_string(),
                },
            ),
            // File creation tasks
            (
                Regex::new(r"(create|add|write)\s+(?:a\s+)?file").unwrap(),
                CompletionConditionSpec::FileExists {
                    path: "output.txt".to_string(),
                },
            ),
            // Documentation tasks
            (
                Regex::new(r"(document|docs|readme)").unwrap(),
                CompletionConditionSpec::FileExists {
                    path: "README.md".to_string(),
                },
            ),
        ]
    }

    /// Infer completion condition from task description
    pub fn infer(&self, description: &str) -> CompletionConditionSpec {
        let lower = description.to_lowercase();

        for (pattern, condition) in &self.patterns {
            if pattern.is_match(&lower) {
                return condition.clone();
            }
        }

        // Default: command success with echo
        CompletionConditionSpec::CommandSuccess {
            command: "echo done".to_string(),
        }
    }

    /// Infer with custom command extraction
    pub fn infer_with_command(&self, description: &str) -> CompletionConditionSpec {
        // Try to extract specific command from description
        // Match both inline and trailing commands
        if let Some(remainder) = description.strip_prefix("Run `") {
            if let Some(end) = remainder.find("`") {
                let command = remainder[..end].to_string();
                return CompletionConditionSpec::CommandSuccess { command };
            }
        }

        // Alternative pattern: "execute `cmd`"
        let cmd_pattern = Regex::new(r"`([^`]+)`").unwrap();
        if let Some(caps) = cmd_pattern.captures(description) {
            return CompletionConditionSpec::CommandSuccess {
                command: caps[1].to_string(),
            };
        }

        self.infer(description)
    }

    /// Infer file-based condition with path extraction
    pub fn infer_with_path(&self, description: &str) -> CompletionConditionSpec {
        // Try to extract file path
        let file_pattern = Regex::new(r"(?:file|path):\s*(\S+)").unwrap();
        if let Some(caps) = file_pattern.captures(description) {
            return CompletionConditionSpec::FileExists {
                path: caps[1].to_string(),
            };
        }

        self.infer(description)
    }
}

impl Default for ConditionInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inference_new() {
        let inference = ConditionInference::new();
        assert!(!inference.patterns.is_empty());
    }

    #[test]
    fn inference_build_task() {
        let inference = ConditionInference::new();
        let cond = inference.infer("Build the project");

        assert!(matches!(cond, CompletionConditionSpec::CommandSuccess { .. }));
        if let CompletionConditionSpec::CommandSuccess { command } = cond {
            assert!(command.contains("build"));
        }
    }

    #[test]
    fn inference_test_task() {
        let inference = ConditionInference::new();
        let cond = inference.infer("Run all tests");

        assert!(matches!(cond, CompletionConditionSpec::CommandSuccess { .. }));
        if let CompletionConditionSpec::CommandSuccess { command } = cond {
            assert!(command.contains("test"));
        }
    }

    #[test]
    fn inference_file_creation() {
        let inference = ConditionInference::new();
        let cond = inference.infer("Create a file for output");

        assert!(matches!(cond, CompletionConditionSpec::FileExists { .. }));
    }

    #[test]
    fn inference_default() {
        let inference = ConditionInference::new();
        let cond = inference.infer("Some random task");

        assert!(matches!(cond, CompletionConditionSpec::CommandSuccess { .. }));
    }

    #[test]
    fn inference_with_command() {
        let inference = ConditionInference::new();
        let cond = inference.infer_with_command("Run `cargo build --release` for production");

        if let CompletionConditionSpec::CommandSuccess { command } = cond {
            assert_eq!(command, "cargo build --release");
        } else {
            panic!("Expected CommandSuccess");
        }
    }

    #[test]
    fn inference_with_path() {
        let inference = ConditionInference::new();
        let cond = inference.infer_with_path("Check file: src/main.rs");

        if let CompletionConditionSpec::FileExists { path } = cond {
            assert_eq!(path, "src/main.rs");
        } else {
            panic!("Expected FileExists");
        }
    }

    #[test]
    fn inference_documentation() {
        let inference = ConditionInference::new();
        let cond = inference.infer("Update documentation");

        assert!(matches!(cond, CompletionConditionSpec::FileExists { .. }));
    }
}