// Hermes CLI Bridge - Connects Operator to Hermes CLI subprocess
// This module spawns Hermes CLI as a subprocess and parses its output

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use serde::{Deserialize, Serialize};
use crate::operator::{OperatorEvent, OperatorEventType, EventLevel};

// ============================================================================
// Hermes CLI Bridge
// ============================================================================

pub struct HermesCliBridge {
    cli_path: PathBuf,
    project_path: PathBuf,
    task_id: String,
}

impl HermesCliBridge {
    pub fn new(cli_path: PathBuf, project_path: PathBuf, task_id: String) -> Self {
        Self {
            cli_path,
            project_path,
            task_id,
        }
    }

    /// Check if Hermes CLI is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.cli_path)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Execute a task using Hermes CLI
    /// Returns a receiver for streaming events
    pub fn execute_task(&self, goal: &str) -> Result<Receiver<HermesEvent>, HermesCliError> {
        let (tx, rx) = channel();

        let cli_path = self.cli_path.clone();
        let project_path = self.project_path.clone();
        let task_id = self.task_id.clone();
        let goal = goal.to_string();

        thread::spawn(move || {
            if let Err(e) = Self::run_hermes_process(&cli_path, &project_path, &task_id, &goal, tx.clone()) {
                let _ = tx.send(HermesEvent::Error { message: e.to_string() });
            }
        });

        Ok(rx)
    }

    fn run_hermes_process(
        cli_path: &PathBuf,
        project_path: &PathBuf,
        task_id: &str,
        goal: &str,
        tx: Sender<HermesEvent>,
    ) -> Result<(), HermesCliError> {
        let mut child = Command::new(cli_path)
            .arg("execute")
            .arg("--goal")
            .arg(goal)
            .arg("--project")
            .arg(project_path)
            .arg("--output-format")
            .arg("json")
            .arg("--task-id")
            .arg(task_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| HermesCliError::ProcessError(e.to_string()))?;

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let reader = BufReader::new(stdout);

        // Parse JSON events from stdout
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.starts_with("{") {
                        if let Ok(event) = serde_json::from_str::<HermesEvent>(&line) {
                            let _ = tx.send(event);
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(HermesEvent::Error { message: format!("Read error: {}", e) });
                    break;
                }
            }
        }

        let status = child.wait()
            .map_err(|e| HermesCliError::ProcessError(e.to_string()))?;

        if status.success() {
            let _ = tx.send(HermesEvent::Completed { success: true });
        } else {
            let _ = tx.send(HermesEvent::Completed { success: false });
        }

        Ok(())
    }

    /// Analyze project using Hermes CLI
    pub fn analyze_project(&self) -> Result<HermesAnalysisResult, HermesCliError> {
        let output = Command::new(&self.cli_path)
            .arg("analyze")
            .arg("--project")
            .arg(&self.project_path)
            .arg("--output-format")
            .arg("json")
            .output()
            .map_err(|e| HermesCliError::ProcessError(e.to_string()))?;

        if !output.status.success() {
            return Err(HermesCliError::AnalysisFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        serde_json::from_slice(&output.stdout)
            .map_err(|e| HermesCliError::ParseError(e.to_string()))
    }

    /// Generate plan using Hermes CLI
    pub fn generate_plan(&self, goal: &str) -> Result<HermesPlan, HermesCliError> {
        let output = Command::new(&self.cli_path)
            .arg("plan")
            .arg("--goal")
            .arg(goal)
            .arg("--project")
            .arg(&self.project_path)
            .arg("--output-format")
            .arg("json")
            .output()
            .map_err(|e| HermesCliError::ProcessError(e.to_string()))?;

        if !output.status.success() {
            return Err(HermesCliError::PlanningFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        serde_json::from_slice(&output.stdout)
            .map_err(|e| HermesCliError::ParseError(e.to_string()))
    }

    /// Execute a single step using Hermes CLI
    pub fn execute_step(&self, step_id: u32, approve: bool) -> Result<HermesStepResult, HermesCliError> {
        let mut cmd = Command::new(&self.cli_path);
        cmd.arg("step")
            .arg("--step-id")
            .arg(step_id.to_string())
            .arg("--project")
            .arg(&self.project_path);

        if approve {
            cmd.arg("--approve");
        }

        let output = cmd.output()
            .map_err(|e| HermesCliError::ProcessError(e.to_string()))?;

        if !output.status.success() {
            return Err(HermesCliError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        serde_json::from_slice(&output.stdout)
            .map_err(|e| HermesCliError::ParseError(e.to_string()))
    }
}

// ============================================================================
// Event Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HermesEvent {
    #[serde(rename = "task_started")]
    TaskStarted { task_id: String },

    #[serde(rename = "phase_changed")]
    PhaseChanged { task_id: String, phase: String },

    #[serde(rename = "progress")]
    Progress { task_id: String, message: String, percentage: u8 },

    #[serde(rename = "file_read")]
    FileRead { task_id: String, path: String },

    #[serde(rename = "file_modified")]
    FileModified { task_id: String, path: String, diff: Option<String> },

    #[serde(rename = "tool_call")]
    ToolCall { task_id: String, tool: String, input: serde_json::Value },

    #[serde(rename = "tool_result")]
    ToolResult { task_id: String, tool: String, output: serde_json::Value },

    #[serde(rename = "approval_required")]
    ApprovalRequired { task_id: String, approval_id: String, message: String },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "completed")]
    Completed { success: bool },
}

impl HermesEvent {
    /// Convert to OperatorEvent
    pub fn to_operator_event(&self, task_id: &str) -> OperatorEvent {
        let (event_type, title, message) = match self {
            HermesEvent::TaskStarted { .. } => {
                (OperatorEventType::TaskStarted, "Task started".to_string(), Some("Task execution started".to_string()))
            }
            HermesEvent::PhaseChanged { phase, .. } => {
                (OperatorEventType::StepStarted, format!("Phase: {}", phase), Some(format!("Entered {} phase", phase)))
            }
            HermesEvent::Progress { message, percentage, .. } => {
                (OperatorEventType::StepExecuting, message.clone(), Some(format!("{}% - {}", percentage, message)))
            }
            HermesEvent::FileRead { path, .. } => {
                (OperatorEventType::FileRead, format!("Read: {}", path), Some(format!("Reading file: {}", path)))
            }
            HermesEvent::FileModified { path, diff, .. } => {
                (OperatorEventType::FileModified, format!("Modified: {}", path), diff.clone())
            }
            HermesEvent::ToolCall { tool, input, .. } => {
                (OperatorEventType::ToolCallStarted, format!("Call: {}", tool), Some(format!("{:?}", input)))
            }
            HermesEvent::ToolResult { tool, output, .. } => {
                (OperatorEventType::ToolCallSucceeded, format!("Result: {}", tool), Some(format!("{:?}", output)))
            }
            HermesEvent::ApprovalRequired { approval_id, message, .. } => {
                (OperatorEventType::ApprovalRequested, format!("Approval: {}", approval_id), Some(message.clone()))
            }
            HermesEvent::Error { message } => {
                (OperatorEventType::TaskFailed, "Error".to_string(), Some(message.clone()))
            }
            HermesEvent::Completed { success } => {
                if *success {
                    (OperatorEventType::TaskCompleted, "Completed".to_string(), Some("Task completed successfully".to_string()))
                } else {
                    (OperatorEventType::TaskFailed, "Failed".to_string(), Some("Task failed".to_string()))
                }
            }
        };

        OperatorEvent {
            event_id: format!("evt_{}_{}", task_id, chrono::Utc::now().timestamp_millis()),
            task_id: task_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            level: EventLevel::Info,
            title,
            message,
            source: "hermes_cli".to_string(),
            payload: Some(serde_json::to_value(self).unwrap_or(serde_json::Value::Null)),
        }
    }
}

// ============================================================================
// Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConnectionStatus {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesAnalysisResult {
    pub project_name: String,
    pub godot_version: String,
    pub scripts: Vec<String>,
    pub scenes: Vec<String>,
    pub player_controllers: Vec<String>,
    pub analysis_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesPlan {
    pub task_id: String,
    pub goal: String,
    pub steps: Vec<HermesPlanStep>,
    pub total_estimated_time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesPlanStep {
    pub id: u32,
    pub description: String,
    pub files: Vec<String>,
    pub estimated_time_seconds: u64,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesStepResult {
    pub step_id: u32,
    pub success: bool,
    pub file_changes: Vec<HermesFileChange>,
    pub output: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesFileChange {
    pub path: String,
    pub action: String,
    pub diff: Option<String>,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum HermesCliError {
    ProcessError(String),
    ParseError(String),
    AnalysisFailed(String),
    PlanningFailed(String),
    ExecutionFailed(String),
    Timeout,
}

impl std::fmt::Display for HermesCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HermesCliError::ProcessError(msg) => write!(f, "Process error: {}", msg),
            HermesCliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            HermesCliError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            HermesCliError::PlanningFailed(msg) => write!(f, "Planning failed: {}", msg),
            HermesCliError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            HermesCliError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for HermesCliError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_event_to_operator_event() {
        let event = HermesEvent::Progress {
            task_id: "test_task".to_string(),
            message: "Analyzing project".to_string(),
            percentage: 50,
        };

        let op_event = event.to_operator_event("test_task");
        assert_eq!(op_event.task_id, "test_task");
        assert!(op_event.title.contains("Analyzing"));
    }

    #[test]
    fn test_hermes_event_serialization() {
        let event = HermesEvent::FileModified {
            task_id: "test".to_string(),
            path: "scripts/player.gd".to_string(),
            diff: Some("@@ -1 +1 @@".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("file_modified"));
    }
}