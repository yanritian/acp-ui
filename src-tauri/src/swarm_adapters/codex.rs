//! Codex Adapter - Stdio process communication for Codex CLI
//!
//! Manages `codex --full-auto` subprocess with real-time output streaming.
//! Codex operates in autonomous mode, executing tasks without interactive prompts.
//!
//! ## Process Model
//!
//! Each task spawns a fresh `codex` process:
//! 1. Prompt is passed as a command-line argument (codex accepts the prompt directly)
//! 2. stdout/stderr are read in background threads
//! 3. When the process exits, the task is marked Completed/Failed

use super::*;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Adapter for Codex CLI
///
/// Codex is an autonomous coding agent that can execute complex tasks.
///
/// Command: `codex -q "prompt" --full-auto`
/// - Reads prompt from command line argument (-q for quiet/quick mode)
/// - Writes output to stdout
/// - Errors to stderr
/// - Exits when execution is complete
pub struct CodexAdapter {
    /// Worker ID
    worker_id: WorkerId,
    /// Worker type
    worker_type: String,
    /// Current subprocess (if running) — for cancel / health check
    process: Arc<Mutex<Option<Child>>>,
    /// Current status
    status: Arc<Mutex<WorkerStatus>>,
    /// Task handles
    tasks: Arc<Mutex<HashMap<TaskId, TaskHandle>>>,
    /// Output accumulator
    outputs: Arc<Mutex<HashMap<TaskId, String>>>,
    /// Capabilities
    capabilities: WorkerCapabilities,
}

impl CodexAdapter {
    /// Create a new Codex adapter
    pub fn new(worker_id: WorkerId) -> Self {
        let capabilities = WorkerCapabilities {
            worker_id: worker_id.clone(),
            worker_type: "codex".to_string(),
            capabilities: vec![
                "code_generation".to_string(),
                "file_editing".to_string(),
                "test_writing".to_string(),
                "debugging".to_string(),
                "refactoring".to_string(),
                "git_operations".to_string(),
            ],
            max_complexity: 5,
            max_concurrent: 1,
            supports_streaming: true,
            supports_cancel: true,
            default_timeout_ms: 300_000, // 5 minutes
        };

        let status = WorkerStatus {
            worker_id: worker_id.clone(),
            worker_type: "codex".to_string(),
            health: HealthStatus::Offline,
            pid: None,
            memory_bytes: None,
            cpu_percent: None,
            tasks_completed: 0,
            tasks_failed: 0,
            current_task: None,
            started_at: None,
            last_heartbeat: None,
        };

        Self {
            worker_id,
            worker_type: "codex".to_string(),
            process: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(status)),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            outputs: Arc::new(Mutex::new(HashMap::new())),
            capabilities,
        }
    }

    /// Spawn a fresh `codex` process with the given prompt.
    fn spawn_codex_process(
        &self,
        prompt: &str,
        working_dir: Option<&str>,
    ) -> Result<Child, SwarmError> {
        let mut cmd = Command::new("codex");
        cmd.arg("-q"); // quiet/non-interactive
        cmd.arg("--full-auto");
        cmd.arg(prompt); // pass prompt as CLI argument
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        cmd.spawn()
            .map_err(|e| SwarmError::ProcessStartFailed(format!("Failed to spawn codex: {}", e)))
    }

    /// Kill the currently tracked process (if any)
    fn kill_process(&self) -> Result<(), SwarmError> {
        let mut process_guard = self
            .process
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("process lock poisoned: {}", e)))?;

        if let Some(ref mut child) = *process_guard {
            let _ = child.kill();
            let _ = child.wait();
            *process_guard = None;
        }

        let mut status = self
            .status
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("status lock poisoned: {}", e)))?;
        status.health = HealthStatus::Offline;
        status.pid = None;

        Ok(())
    }
}

impl SwarmAgentAdapter for CodexAdapter {
    fn send_task(&self, task: &TaskDescription) -> Result<TaskHandle, SwarmError> {
        // 1. Spawn a fresh codex process
        let mut child = self.spawn_codex_process(&task.prompt, task.working_dir.as_deref())?;
        let pid = child.id();

        // 2. Take stdout and stderr handles
        let stdout_handle = child
            .stdout
            .take()
            .ok_or_else(|| SwarmError::ProcessStartFailed("stdout pipe not available".into()))?;
        let stderr_handle = child
            .stderr
            .take()
            .ok_or_else(|| SwarmError::ProcessStartFailed("stderr pipe not available".into()))?;

        // 3. Store the child
        *self
            .process
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("process lock poisoned: {}", e)))? =
            Some(child);

        // 4. Create task handle
        let handle = TaskHandle {
            task_id: task.id.clone(),
            worker_id: self.worker_id.clone(),
            status: TaskStatus::Running,
            started_at: InstantWrapper::now(),
            output: String::new(),
            error: None,
            pid: Some(pid),
        };

        self.tasks
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("tasks lock poisoned: {}", e)))?
            .insert(task.id.clone(), handle.clone());

        // 5. Update worker status
        {
            let mut st = self
                .status
                .lock()
                .map_err(|e| SwarmError::InternalError(format!("status lock poisoned: {}", e)))?;
            st.health = HealthStatus::Busy;
            st.current_task = Some(task.id.clone());
            st.pid = Some(pid);
            st.started_at = Some(InstantWrapper::now());
            st.last_heartbeat = Some(InstantWrapper::now());
        }

        // 6. Initialize output accumulator
        self.outputs
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("outputs lock poisoned: {}", e)))?
            .insert(task.id.clone(), String::new());

        // 7. Spawn background thread to read stdout + wait for process exit
        let task_id = task.id.clone();
        let outputs = Arc::clone(&self.outputs);
        let tasks = Arc::clone(&self.tasks);
        let status = Arc::clone(&self.status);

        thread::spawn(move || {
            // Read stdout lines and accumulate
            let reader = BufReader::new(stdout_handle);
            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        if let Ok(mut out) = outputs.lock() {
                            if let Some(buf) = out.get_mut(&task_id) {
                                buf.push_str(&text);
                                buf.push('\n');
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // stdout closed — read remaining stderr
            let err_reader = BufReader::new(stderr_handle);
            let stderr_text: String = err_reader
                .lines()
                .filter_map(|l| l.ok())
                .collect::<Vec<_>>()
                .join("\n");

            // Update task with final output
            if let Ok(mut tasks_guard) = tasks.lock() {
                if let Some(handle) = tasks_guard.get_mut(&task_id) {
                    let final_output = outputs
                        .lock()
                        .ok()
                        .and_then(|o| o.get(&task_id).cloned())
                        .unwrap_or_default();
                    handle.output = final_output;

                    if !stderr_text.is_empty() {
                        handle.error = Some(stderr_text);
                        handle.status = TaskStatus::Failed;
                    } else {
                        handle.status = TaskStatus::Completed;
                    }
                }
            }

            // Update worker status
            if let Ok(mut st) = status.lock() {
                st.current_task = None;
                st.health = HealthStatus::Healthy;
                st.last_heartbeat = Some(InstantWrapper::now());
                let failed = tasks
                    .lock()
                    .ok()
                    .and_then(|t| t.get(&task_id).map(|h| h.status == TaskStatus::Failed))
                    .unwrap_or(false);
                if failed {
                    st.tasks_failed += 1;
                } else {
                    st.tasks_completed += 1;
                }
            }
        });

        Ok(handle)
    }

    fn get_status(&self) -> Result<WorkerStatus, SwarmError> {
        let status = self
            .status
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("status lock poisoned: {}", e)))?
            .clone();
        Ok(status)
    }

    fn cancel_task(&self, task_id: &str) -> Result<(), SwarmError> {
        // Mark task as cancelled
        {
            let mut tasks = self
                .tasks
                .lock()
                .map_err(|e| SwarmError::InternalError(format!("tasks lock poisoned: {}", e)))?;
            if let Some(handle) = tasks.get_mut(task_id) {
                handle.status = TaskStatus::Cancelled;
            } else {
                return Err(SwarmError::WorkerNotFound(task_id.to_string()));
            }
        }

        // Kill the process
        self.kill_process()?;

        // Clear current task from worker status
        {
            let mut st = self
                .status
                .lock()
                .map_err(|e| SwarmError::InternalError(format!("status lock poisoned: {}", e)))?;
            if st.current_task.as_ref() == Some(&task_id.to_string()) {
                st.current_task = None;
                st.health = HealthStatus::Healthy;
            }
        }

        Ok(())
    }

    fn capabilities(&self) -> WorkerCapabilities {
        self.capabilities.clone()
    }

    fn health_check(&self) -> bool {
        let status = self.status.lock().unwrap();
        matches!(status.health, HealthStatus::Healthy | HealthStatus::Busy)
    }

    fn worker_id(&self) -> &WorkerId {
        &self.worker_id
    }

    fn worker_type(&self) -> &str {
        &self.worker_type
    }

    fn shutdown(&self) -> Result<(), SwarmError> {
        self.kill_process()
    }

    fn get_task_output(&self, task_id: &str) -> Result<String, SwarmError> {
        let outputs = self
            .outputs
            .lock()
            .map_err(|e| SwarmError::InternalError(format!("outputs lock poisoned: {}", e)))?;
        outputs
            .get(task_id)
            .cloned()
            .ok_or_else(|| SwarmError::WorkerNotFound(format!("Task {} output not found", task_id)))
    }
}

/// Builder for CodexAdapter with custom configuration
pub struct CodexAdapterBuilder {
    worker_id: WorkerId,
    working_dir: Option<String>,
}

impl CodexAdapterBuilder {
    pub fn new(worker_id: WorkerId) -> Self {
        Self {
            worker_id,
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn build(self) -> CodexAdapter {
        CodexAdapter::new(self.worker_id)
    }
}
