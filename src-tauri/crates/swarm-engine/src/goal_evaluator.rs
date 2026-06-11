//! ConditionEvaluator - 评估CompletionCondition是否满足
//!
//! 根据RFC-001定义的8种CompletionCondition类型，提供对应的评估方法。

use crate::goal::{CompletionCondition, EvaluationResult};
use std::process::Command;
use std::path::Path;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Regex parse error: {0}")]
    RegexError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),
}

/// ConditionEvaluator - 评估完成条件
pub struct ConditionEvaluator {
    /// 默认工作目录
    default_cwd: Option<String>,
}

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self { default_cwd: None }
    }

    pub fn with_cwd(cwd: impl Into<String>) -> Self {
        Self { default_cwd: Some(cwd.into()) }
    }

    /// 评估任意CompletionCondition
    pub fn evaluate(&self, condition: &CompletionCondition) -> EvaluationResult {
        match condition {
            CompletionCondition::CommandSuccess { command, args, cwd } => {
                self.eval_command(command, args, cwd.as_ref().or(self.default_cwd.as_ref()))
            }
            CompletionCondition::OutputContains { command, pattern, case_sensitive } => {
                self.eval_output_contains(command, pattern, *case_sensitive)
            }
            CompletionCondition::OutputMatches { command, regex } => {
                self.eval_output_matches(command, regex)
            }
            CompletionCondition::FileCheck { path, content_contains, max_size_bytes } => {
                self.eval_file_check(path, content_contains.as_ref(), *max_size_bytes)
            }
            CompletionCondition::HttpHealthCheck { url, method, expected_status } => {
                self.eval_http_health_check(url, method, *expected_status)
            }
            CompletionCondition::All { conditions } => {
                self.eval_all(conditions)
            }
            CompletionCondition::Any { conditions } => {
                self.eval_any(conditions)
            }
            CompletionCondition::QueenJudgment { criteria } => {
                // QueenJudgment需要人工或Queen评估，返回待评估状态
                EvaluationResult {
                    converged: false,
                    feedback: format!("QueenJudgment pending: {}", criteria),
                    tokens_used: 0,
                }
            }
        }
    }

    /// 评估命令成功条件（退出码为0）
    pub fn eval_command(&self, command: &str, args: &[String], cwd: Option<&String>) -> EvaluationResult {
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    EvaluationResult {
                        converged: true,
                        feedback: String::new(),
                        tokens_used: 0,
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    EvaluationResult {
                        converged: false,
                        feedback: format!("Command failed with code {}: {}",
                            output.status.code().unwrap_or(-1), stderr),
                        tokens_used: 0,
                    }
                }
            }
            Err(e) => {
                EvaluationResult {
                    converged: false,
                    feedback: format!("Failed to execute command '{}': {}", command, e),
                    tokens_used: 0,
                }
            }
        }
    }

    /// 评估输出包含条件
    pub fn eval_output_contains(&self, command: &str, pattern: &str, case_sensitive: bool) -> EvaluationResult {
        // 首先执行命令
        let output = match self.run_command(command, &[], None) {
            Ok(o) => o,
            Err(e) => return EvaluationResult {
                converged: false,
                feedback: e.to_string(),
                tokens_used: 0,
            },
        };

        let output_str = String::from_utf8_lossy(&output.stdout);
        let search_str = if case_sensitive {
            output_str.to_string()
        } else {
            output_str.to_lowercase()
        };
        let pattern_str = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        if search_str.contains(&pattern_str) {
            EvaluationResult {
                converged: true,
                feedback: String::new(),
                tokens_used: 0,
            }
        } else {
            EvaluationResult {
                converged: false,
                feedback: format!("Output does not contain '{}'", pattern),
                tokens_used: 0,
            }
        }
    }

    /// 评估输出匹配正则条件
    pub fn eval_output_matches(&self, command: &str, regex: &str) -> EvaluationResult {
        let output = match self.run_command(command, &[], None) {
            Ok(o) => o,
            Err(e) => return EvaluationResult {
                converged: false,
                feedback: e.to_string(),
                tokens_used: 0,
            },
        };

        let re = match Regex::new(regex) {
            Ok(r) => r,
            Err(e) => return EvaluationResult {
                converged: false,
                feedback: format!("Invalid regex '{}': {}", regex, e),
                tokens_used: 0,
            },
        };

        let output_str = String::from_utf8_lossy(&output.stdout);

        if re.is_match(&output_str) {
            EvaluationResult {
                converged: true,
                feedback: String::new(),
                tokens_used: 0,
            }
        } else {
            EvaluationResult {
                converged: false,
                feedback: format!("Output does not match regex '{}'", regex),
                tokens_used: 0,
            }
        }
    }

    /// 评估文件检查条件
    pub fn eval_file_check(&self, path: &str, content_contains: Option<&String>, max_size_bytes: Option<u64>) -> EvaluationResult {
        let file_path = Path::new(path);

        if !file_path.exists() {
            return EvaluationResult {
                converged: false,
                feedback: format!("File '{}' does not exist", path),
                tokens_used: 0,
            };
        }

        // 检查大小限制
        if let Some(max_size) = max_size_bytes {
            if let Ok(metadata) = file_path.metadata() {
                if metadata.len() > max_size {
                    return EvaluationResult {
                        converged: false,
                        feedback: format!("File '{}' exceeds size limit ({} > {} bytes)",
                            path, metadata.len(), max_size),
                        tokens_used: 0,
                    };
                }
            }
        }

        // 检查内容
        if let Some(content_pattern) = content_contains {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    if content.contains(content_pattern) {
                        EvaluationResult {
                            converged: true,
                            feedback: String::new(),
                            tokens_used: 0,
                        }
                    } else {
                        EvaluationResult {
                            converged: false,
                            feedback: format!("File '{}' does not contain '{}'", path, content_pattern),
                            tokens_used: 0,
                        }
                    }
                }
                Err(e) => EvaluationResult {
                    converged: false,
                    feedback: format!("Failed to read file '{}': {}", path, e),
                    tokens_used: 0,
                },
            }
        } else {
            // 只检查存在性
            EvaluationResult {
                converged: true,
                feedback: String::new(),
                tokens_used: 0,
            }
        }
    }

    /// 评估HTTP健康检查条件
    pub fn eval_http_health_check(&self, url: &str, method: &str, expected_status: Option<u16>) -> EvaluationResult {
        // 使用ureq进行简单HTTP请求
        let expected = expected_status.unwrap_or(200);

        let response = match method {
            "GET" => ureq::get(url).call(),
            "POST" => ureq::post(url).call(),
            _ => ureq::get(url).call(),
        };

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status == expected {
                    EvaluationResult {
                        converged: true,
                        feedback: String::new(),
                        tokens_used: 0,
                    }
                } else {
                    EvaluationResult {
                        converged: false,
                        feedback: format!("HTTP {} returned {} (expected {})", method, status, expected),
                        tokens_used: 0,
                    }
                }
            }
            Err(e) => EvaluationResult {
                converged: false,
                feedback: format!("HTTP request to '{}' failed: {}", url, e),
                tokens_used: 0,
            },
        }
    }

    /// 评估All条件（所有子条件都必须满足）
    pub fn eval_all(&self, conditions: &[CompletionCondition]) -> EvaluationResult {
        let mut total_tokens = 0u64;
        let mut failed_feedbacks = Vec::new();

        for condition in conditions {
            let result = self.evaluate(condition);
            total_tokens += result.tokens_used;

            if !result.converged {
                failed_feedbacks.push(result.feedback);
            }
        }

        if failed_feedbacks.is_empty() {
            EvaluationResult {
                converged: true,
                feedback: String::new(),
                tokens_used: total_tokens,
            }
        } else {
            EvaluationResult {
                converged: false,
                feedback: format!("All condition failed: {}", failed_feedbacks.join("; ")),
                tokens_used: total_tokens,
            }
        }
    }

    /// 评估Any条件（任一子条件满足即可）
    pub fn eval_any(&self, conditions: &[CompletionCondition]) -> EvaluationResult {
        let mut total_tokens = 0u64;
        let mut feedbacks = Vec::new();

        for condition in conditions {
            let result = self.evaluate(condition);
            total_tokens += result.tokens_used;

            if result.converged {
                return EvaluationResult {
                    converged: true,
                    feedback: String::new(),
                    tokens_used: total_tokens,
                };
            }
            feedbacks.push(result.feedback);
        }

        EvaluationResult {
            converged: false,
            feedback: format!("Any condition failed: {}", feedbacks.join("; ")),
            tokens_used: total_tokens,
        }
    }

    /// 辅助方法：执行命令并返回输出
    fn run_command(&self, command: &str, args: &[String], cwd: Option<&String>) -> Result<std::process::Output, EvaluationError> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        cmd.output().map_err(|e| EvaluationError::CommandFailed(e.to_string()))
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}