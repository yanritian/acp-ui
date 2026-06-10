//! ConditionEvaluator — evaluates CompletionConditions against execution results
//!
//! Each `CompletionCondition` variant maps to a concrete check:
//! - `CommandSuccess` → spawn a subprocess, check exit code
//! - `OutputContains` → string search
//! - `OutputMatches` → regex match
//! - `FileCheck` → filesystem check
//! - `HttpHealthCheck` → HTTP request
//! - `All` / `Any` → compound logic
//! - `QueenJudgment` → deferred to reconcile.rs (requires Queen worker)

use crate::goal::{
    CompletionCondition, ConditionResult, EvaluationResult,
};
use std::process::Command;

/// Evaluates completion conditions against worker output.
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a single completion condition against the worker's output.
    ///
    /// Returns an `EvaluationResult` with per-condition details.
    pub fn eval(&self, condition: &CompletionCondition, output: &str) -> EvaluationResult {
        match condition {
            CompletionCondition::CommandSuccess { command, expected_exit_code } => {
                self.eval_command(command, *expected_exit_code)
            }
            CompletionCondition::OutputContains { text, case_sensitive } => {
                self.eval_output_contains(output, text, *case_sensitive)
            }
            CompletionCondition::OutputMatches { pattern } => {
                self.eval_output_matches(output, pattern)
            }
            CompletionCondition::All { conditions } => {
                self.eval_all(conditions, output)
            }
            CompletionCondition::Any { conditions } => {
                self.eval_any(conditions, output)
            }
            CompletionCondition::FileCheck { path, must_exist, content_contains } => {
                self.eval_file_check(path, *must_exist, content_contains.as_deref())
            }
            CompletionCondition::HttpHealthCheck { url, expected_status } => {
                self.eval_http_health(url, *expected_status)
            }
            CompletionCondition::QueenJudgment { criteria } => {
                // Queen judgment requires a live worker — handled by ReconcileLoop
                EvaluationResult {
                    passed: false,
                    explanation: format!(
                        "QueenJudgment requires live Queen evaluation (criteria: {})",
                        criteria
                    ),
                    details: vec![ConditionResult {
                        description: format!("Queen judgment: {}", criteria),
                        passed: false,
                        evidence: "Requires Queen worker — deferred to reconcile loop".into(),
                    }],
                }
            }
        }
    }

    // --- Individual evaluators ---

    /// Commands that are never allowed in completion checks (security denylist).
    const DENIED_COMMANDS: &'static [&'static str] = &[
        "rm -rf /", "rm -rf /*", "mkfs", "dd if=", ":(){:|:&};:",
        "chmod -R 777 /", "wget", "curl", "nc ", "ncat ",
        "bash -i", "python -c", "perl -e", "ruby -e",
        "> /dev/sda", "mv / ", "format c:",
    ];

    fn eval_command(&self, command: &str, expected_exit_code: i32) -> EvaluationResult {
        // Security: deny dangerous commands
        let cmd_lower = command.to_lowercase();
        for denied in Self::DENIED_COMMANDS {
            if cmd_lower.contains(denied) {
                return EvaluationResult {
                    passed: false,
                    explanation: format!(
                        "Command '{}' was rejected by security denylist (matched: '{}')",
                        command, denied
                    ),
                    details: vec![ConditionResult {
                        description: format!("Command: {} (DENIED)", command),
                        passed: false,
                        evidence: format!("Security policy blocked: contains '{}'", denied),
                    }],
                };
            }
        }

        // Security: limit command length to prevent injection via long strings
        if command.len() > 512 {
            return EvaluationResult {
                passed: false,
                explanation: format!(
                    "Command rejected: length {} exceeds maximum 512 characters",
                    command.len()
                ),
                details: vec![ConditionResult {
                    description: "Command length check".into(),
                    passed: false,
                    evidence: format!("Command length: {} chars (max 512)", command.len()),
                }],
            };
        }

        let result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output();

        match result {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(-1);
                let passed = exit_code == expected_exit_code;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                EvaluationResult {
                    passed,
                    explanation: if passed {
                        format!("Command '{}' exited with code {}", command, exit_code)
                    } else {
                        format!(
                            "Command '{}' exited with code {} (expected {})",
                            command, exit_code, expected_exit_code
                        )
                    },
                    details: vec![ConditionResult {
                        description: format!("Command: {} (expect exit {})", command, expected_exit_code),
                        passed,
                        evidence: format!(
                            "exit_code={}, stdout={}, stderr={}",
                            exit_code,
                            stdout.chars().take(200).collect::<String>(),
                            stderr.chars().take(200).collect::<String>()
                        ),
                    }],
                }
            }
            Err(e) => EvaluationResult {
                passed: false,
                explanation: format!("Failed to execute command '{}': {}", command, e),
                details: vec![ConditionResult {
                    description: format!("Command: {}", command),
                    passed: false,
                    evidence: format!("Execution error: {}", e),
                }],
            },
        }
    }

    fn eval_output_contains(
        &self,
        output: &str,
        text: &str,
        case_sensitive: bool,
    ) -> EvaluationResult {
        let passed = if case_sensitive {
            output.contains(text)
        } else {
            output.to_lowercase().contains(&text.to_lowercase())
        };

        EvaluationResult {
            passed,
            explanation: if passed {
                format!("Output contains '{}' (case_sensitive={})", text, case_sensitive)
            } else {
                format!(
                    "Output does NOT contain '{}' (case_sensitive={})",
                    text, case_sensitive
                )
            },
            details: vec![ConditionResult {
                description: format!("Output contains '{}' (case_sensitive={})", text, case_sensitive),
                passed,
                evidence: format!("Output length: {} chars", output.len()),
            }],
        }
    }

    fn eval_output_matches(&self, output: &str, pattern: &str) -> EvaluationResult {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                let passed = re.is_match(output);
                EvaluationResult {
                    passed,
                    explanation: if passed {
                        format!("Output matches pattern '{}'", pattern)
                    } else {
                        format!("Output does NOT match pattern '{}'", pattern)
                    },
                    details: vec![ConditionResult {
                        description: format!("Regex: {}", pattern),
                        passed,
                        evidence: format!("Output length: {} chars", output.len()),
                    }],
                }
            }
            Err(e) => EvaluationResult {
                passed: false,
                explanation: format!("Invalid regex pattern '{}': {}", pattern, e),
                details: vec![ConditionResult {
                    description: format!("Regex: {}", pattern),
                    passed: false,
                    evidence: format!("Regex compilation error: {}", e),
                }],
            },
        }
    }

    fn eval_all(&self, conditions: &[CompletionCondition], output: &str) -> EvaluationResult {
        let mut details = Vec::new();
        let mut all_passed = true;

        for cond in conditions {
            let sub = self.eval(cond, output);
            if !sub.passed {
                all_passed = false;
            }
            details.extend(sub.details);
        }

        EvaluationResult {
            passed: all_passed,
            explanation: if all_passed {
                format!("All {} conditions passed", conditions.len())
            } else {
                let failed = details.iter().filter(|d| !d.passed).count();
                format!("{}/{} conditions failed", failed, conditions.len())
            },
            details,
        }
    }

    fn eval_any(&self, conditions: &[CompletionCondition], output: &str) -> EvaluationResult {
        let mut details = Vec::new();
        let mut any_passed = false;

        for cond in conditions {
            let sub = self.eval(cond, output);
            if sub.passed {
                any_passed = true;
            }
            details.extend(sub.details);
        }

        EvaluationResult {
            passed: any_passed,
            explanation: if any_passed {
                let passed_count = details.iter().filter(|d| d.passed).count();
                format!("{}/{} conditions passed", passed_count, conditions.len())
            } else {
                format!("None of {} conditions passed", conditions.len())
            },
            details,
        }
    }

    fn eval_file_check(
        &self,
        path: &str,
        must_exist: bool,
        content_contains: Option<&str>,
    ) -> EvaluationResult {
        let exists = std::path::Path::new(path).exists();

        if must_exist && !exists {
            return EvaluationResult {
                passed: false,
                explanation: format!("File '{}' does not exist (expected to exist)", path),
                details: vec![ConditionResult {
                    description: format!("File exists: {}", path),
                    passed: false,
                    evidence: "File not found".into(),
                }],
            };
        }

        if !must_exist && exists {
            return EvaluationResult {
                passed: false,
                explanation: format!("File '{}' exists (expected to NOT exist)", path),
                details: vec![ConditionResult {
                    description: format!("File does NOT exist: {}", path),
                    passed: false,
                    evidence: "File found".into(),
                }],
            };
        }

        // File existence check passed. Now check content if required.
        if let Some(needle) = content_contains {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let passed = content.contains(needle);
                    EvaluationResult {
                        passed,
                        explanation: if passed {
                            format!("File '{}' exists and contains '{}'", path, needle)
                        } else {
                            format!("File '{}' exists but does NOT contain '{}'", path, needle)
                        },
                        details: vec![
                            ConditionResult {
                                description: format!("File exists: {}", path),
                                passed: true,
                                evidence: format!("Size: {} bytes", content.len()),
                            },
                            ConditionResult {
                                description: format!("Contains: '{}'", needle),
                                passed,
                                evidence: format!("Content preview: {}...", content.chars().take(100).collect::<String>()),
                            },
                        ],
                    }
                }
                Err(e) => EvaluationResult {
                    passed: false,
                    explanation: format!("Cannot read file '{}': {}", path, e),
                    details: vec![ConditionResult {
                        description: format!("Read file: {}", path),
                        passed: false,
                        evidence: format!("IO error: {}", e),
                    }],
                },
            }
        } else {
            EvaluationResult {
                passed: true,
                explanation: format!("File '{}' exists as expected", path),
                details: vec![ConditionResult {
                    description: format!("File exists: {}", path),
                    passed: true,
                    evidence: "OK".into(),
                }],
            }
        }
    }

    fn eval_http_health(&self, url: &str, expected_status: u16) -> EvaluationResult {
        // Use curl as a fallback HTTP client (avoids pulling in reqwest dependency)
        let result = Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "10", url])
            .output();

        match result {
            Ok(output) => {
                let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let status_code: u16 = status_str.parse().unwrap_or(0);
                let passed = status_code == expected_status;

                EvaluationResult {
                    passed,
                    explanation: if passed {
                        format!("HTTP {} returned status {} (expected {})", url, status_code, expected_status)
                    } else {
                        format!("HTTP {} returned status {} (expected {})", url, status_code, expected_status)
                    },
                    details: vec![ConditionResult {
                        description: format!("HTTP GET {} → {}", url, expected_status),
                        passed,
                        evidence: format!("Actual status: {}", status_code),
                    }],
                }
            }
            Err(e) => EvaluationResult {
                passed: false,
                explanation: format!("HTTP health check failed for '{}': {}", url, e),
                details: vec![ConditionResult {
                    description: format!("HTTP GET {}", url),
                    passed: false,
                    evidence: format!("curl error: {}", e),
                }],
            },
        }
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
