// Codex CLI Adapter - Wraps Codex CLI for code development tasks

use crate::agent_adapter::{health_tracker::HealthTracker, types::*, AgentAdapter};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Codex CLI Adapter
pub struct CodexAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
        }
    }

    /// Codex capabilities
    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "coding".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.002,
                latency_ms: 4000,
            },
            Capability {
                name: "refactoring".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.001,
                latency_ms: 3000,
            },
            Capability {
                name: "code-generation".to_string(),
                proficiency: 0.82,
                cost_per_unit: 0.002,
                latency_ms: 5000,
            },
            Capability {
                name: "documentation".to_string(),
                proficiency: 0.75,
                cost_per_unit: 0.001,
                latency_ms: 2000,
            },
        ]
    }

    /// Estimate tokens from text
    fn estimate_tokens(&self, text: &str) -> u64 {
        let char_count = text.chars().count();
        (char_count / 3) as u64
    }

    /// Calculate cost from tokens
    fn calculate_cost(&self, tokens: u64) -> f32 {
        // Codex pricing: $2 per 1M tokens
        tokens as f32 * 0.002 / 1000.0
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Cli
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        if let Some(cwd) = &config.cwd {
            if !std::path::Path::new(cwd).is_absolute() {
                return Err(AgentError::ConfigurationError {
                    message: "cwd must be an absolute path".to_string(),
                });
            }
        }

        self.config = Some(config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        let output = tokio::process::Command::new("codex")
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(true),
            _ => Err(AgentError::ConfigurationError {
                message: "Codex CLI not found. Please install it first.".to_string(),
            }),
        }
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let mut health_tracker_clone = self.health_tracker.clone();
        if !health_tracker_clone.is_allowed() {
            return Err(AgentError::CircuitBreakerOpen {
                reason: "Too many failures, circuit breaker is open".to_string(),
            });
        }

        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Codex not configured. Call configure() first.".to_string(),
        })?;

        let start = Instant::now();

        let cwd = config.cwd.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });

        let mut child = Command::new("codex")
            .arg("-q")
            .arg(&task.description)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to start Codex: {}", e),
                retryable: true,
            })?;

        let stdout = child.stdout.take().ok_or(AgentError::ExecutionError {
            message: "Failed to capture stdout".to_string(),
            retryable: false,
        })?;

        let mut reader = BufReader::new(stdout);
        let mut output = String::new();

        let mut lines = reader.lines();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to read output: {}", e),
                retryable: false,
            })?
        {
            output.push_str(&line);
            output.push('\n');
        }

        let status = child.wait().await.map_err(|e| AgentError::ExecutionError {
            message: format!("Failed to wait for Codex: {}", e),
            retryable: true,
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if !status.success() {
            return Err(AgentError::ExecutionError {
                message: format!("Codex failed with status: {}", status),
                retryable: true,
            });
        }

        let input_tokens = self.estimate_tokens(&task.description);
        let output_tokens = self.estimate_tokens(&output);
        let total_tokens = input_tokens + output_tokens;
        let cost = self.calculate_cost(total_tokens);

        let token_usage = TokenUsage {
            input_tokens,
            output_tokens,
            total_tokens,
        };

        let result = AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(output),
            input_tokens,
            output_tokens,
            total_tokens,
            cost: ActualCost {
                amount: cost,
                currency: "USD".to_string(),
                token_usage: token_usage.clone(),
            },
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::new(),
        };

        Ok(result)
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> {
        Ok(())
    }

    fn status(&self) -> AgentStatus {
        AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        let input_estimate = self.estimate_tokens(&task.description);
        let output_estimate = 400;

        let total_estimate = input_estimate + output_estimate;
        let min_cost = self.calculate_cost(total_estimate);
        let max_cost = min_cost * 2.0;

        CostEstimate {
            min_cost,
            max_cost,
            currency: "USD".to_string(),
            breakdown: HashMap::from([
                ("input".to_string(), self.calculate_cost(input_estimate)),
                ("output".to_string(), self.calculate_cost(output_estimate)),
            ]),
            token_estimate: TokenEstimate {
                input_tokens: input_estimate,
                output_tokens: output_estimate,
                total_tokens: total_estimate,
            },
        }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history
            .iter()
            .rev()
            .take(last_n as usize)
            .cloned()
            .collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        match result.status {
            ResultStatus::Success => {
                self.health_tracker.record_success(result.duration_ms);
            }
            _ => {
                self.health_tracker.record_error();
            }
        }

        self.token_history.push(TokenUsage {
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
            total_tokens: result.total_tokens,
        });
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        true
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_adapter_new() {
        let adapter = CodexAdapter::new();
        assert_eq!(adapter.id(), "codex");
        assert_eq!(adapter.name(), "Codex CLI");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_capabilities() {
        let adapter = CodexAdapter::new();
        let caps = adapter.capabilities();
        assert!(!caps.is_empty());
        assert!(caps.iter().any(|c| c.name == "coding"));
    }

    #[test]
    fn test_cost_estimate() {
        let adapter = CodexAdapter::new();
        let task = AgentTask {
            id: "test".to_string(),
            description: "Write a function".to_string(),
            input: TaskInput::Text("Write a function".to_string()),
            constraints: TaskConstraints::default(),
            scene_type: None,
            platform: None,
        };

        let estimate = adapter.cost_estimate(&task);
        assert!(estimate.min_cost > 0.0);
        assert!(estimate.max_cost >= estimate.min_cost);
    }
}
