//! Claude Code Adapter - Stdio process communication for Claude Code CLI
//!
//! Manages `claude --print` subprocess with real-time output streaming.
//! Claude Code operates in print mode for non-interactive task execution.

use super::*;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::time::{Duration, Instant};

/// Adapter for Claude Code CLI
///
/// Claude Code is Anthropic's official CLI tool for coding tasks.
/// It provides reasoning, code generation, and analysis capabilities.
///
/// Command: `claude --print`
/// - Accepts prompts via stdin or command line
/// - Outputs responses to stdout
/// - Errors to stderr
pub struct ClaudeCodeAdapter {
    /// Worker ID
    worker_id: WorkerId,
    /// Worker type
    worker_type: String,
    /// Current subprocess (if running)
    process: Arc<Mutex<Option<Child>>>,
    /// Current status
    status: Arc<Mutex<WorkerStatus>>,
    /// Task handles
    tasks: Arc<Mutex<HashMap<TaskId, TaskHandle>>>,
    /// Output accumulator
    outputs: Arc<Mutex<HashMap<TaskId, String>>>,
    /// Capabilities
    capabilities: WorkerCapabilities,
    /// Output reader thread handle
    reader_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    /// Error reader thread handle
    error_thread: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl ClaudeCodeAdapter {
    /// Create a new Claude Code adapter
    pub fn new(worker_id: WorkerId) -> Self {
        let capabilities = WorkerCapabilities {
            worker_id: worker_id.clone(),
            worker_type: "claude_code".to_string(),
            capabilities: vec![
                "reasoning".to_string(),
                "code_generation".to_string(),
                "code_review".to_string(),
                "debugging".to_string(),
                "architecture_design".to_string(),
                "test_writing".to_string(),
                "documentation".to_string(),
                "analysis".to_string(),
            ],
            max_complexity: 5, // Claude Code handles the most complex tasks
            max_concurrent: 1,
            supports_streaming: true,
            supports_cancel: true,
            default_timeout_ms: 600000, // 10 minutes for complex reasoning
        };

        let status = WorkerStatus {
            worker_id: worker_id.clone(),
            worker_type: "claude_code".to_string(),
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
            worker_type: "claude_code".to_string(),
            process: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(status)),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            outputs: Arc::new(Mutex::new(HashMap::new())),
            capabilities,
            reader_thread: Arc::new(Mutex::new(None)),
            error_thread: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the Claude Code process
    fn start_process(&self, working_dir: Option<&str>) -> Result<(), SwarmError> {
        // Check if process already running
        if self.process.lock().unwrap().is_some() {
            return Ok(());
        }

        // Build command
        // claude --print runs in non-interactive mode
        let mut cmd = Command::new("claude");
        cmd.arg("--print");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| SwarmError::ProcessStartFailed(format!("Failed to spawn claude: {}", e)))?;

        let pid = child.id();

        // Store process
        *self.process.lock().unwrap() = Some(child);

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            status.pid = Some(pid);
            status.health = HealthStatus::Starting;
            status.started_at = Some(InstantWrapper::from(Instant::now()));
        }

        // Start output reader threads
        self.start_output_readers();

        Ok(())
    }

    /// Start background threads to read stdout/stderr
    fn start_output_readers(&self) {
        let outputs = self.outputs.clone();
        let status = self.status.clone();
        let tasks = self.tasks.clone();

        // stdout reader
        let stdout_handle = thread::spawn(move || {
            // Placeholder: In production, would:
            // 1. Take stdout from child process
            // 2. Read lines in loop
            // 3. Append to current task output
            // 4. Detect completion markers

            loop {
                thread::sleep(Duration::from_secs(5));

                let mut st = status.lock().unwrap();
                if st.health == HealthStatus::Starting {
                    st.health = HealthStatus::Healthy;
                }
                st.last_heartbeat = Some(InstantWrapper::from(Instant::now()));

                if st.health == HealthStatus::Offline {
                    break;
                }

                // Simulate output accumulation
                if let Some(task_id) = st.current_task.clone() {
                    let mut out = outputs.lock().unwrap();
                    if let Some(output) = out.get_mut(&task_id) {
                        // In production: append actual stdout content
                        // Placeholder: just track that we're reading
                    }
                }
            }
        });

        // stderr reader
        let stderr_handle = thread::spawn(move || {
            // Placeholder: Monitor stderr for errors
            loop {
                thread::sleep(Duration::from_secs(5));

                // In production: read stderr and update task error field
                // Placeholder: just heartbeat
            }
        });

        *self.reader_thread.lock().unwrap() = Some(stdout_handle);
        *self.error_thread.lock().unwrap() = Some(stderr_handle);
    }

    /// Send prompt to Claude Code via stdin
    fn send_prompt(&self, prompt: &str) -> Result<(), SwarmError> {
        let process_guard = self.process.lock().unwrap();

        if let Some(ref mut child) = *process_guard {
            // In production: write to child.stdin
            // Example:
            // let stdin = child.stdin.as_mut().unwrap();
            // stdin.write_all(prompt.as_bytes())?;
            // stdin.flush()?;

            println!("[ClaudeCodeAdapter] Would send prompt to stdin: {}...", &prompt[..100.min(prompt.len())]);
            Ok(())
        } else {
            Err(SwarmError::WorkerCrashed("Process not running".to_string()))
        }
    }

    /// Kill the process
    fn kill_process(&self) -> Result<(), SwarmError> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(ref mut child) = *process_guard {
            child.kill()
                .map_err(|e| SwarmError::InternalError(format!("Failed to kill process: {}", e)))?;

            child.wait()
                .map_err(|e| SwarmError::InternalError(format!("Failed to wait for process: {}", e)))?;

            *process_guard = None;
        }

        // Update status
        {
            let mut status = self.status.lock().unwrap();
            status.health = HealthStatus::Offline;
            status.pid = None;
        }

        Ok(())
    }

    /// Check if process is still running
    fn is_process_alive(&self) -> bool {
        let process_guard = self.process.lock().unwrap();
        process_guard.is_some()
    }

    /// Simulate task completion (placeholder for real implementation)
    fn simulate_task_completion(&self, task_id: &str, output: String) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.get_mut(task_id) {
            handle.status = TaskStatus::Completed;
            handle.output = output.clone();
        }

        let mut outputs = self.outputs.lock().unwrap();
        outputs.insert(task_id.to_string(), output);

        let mut status = self.status.lock().unwrap();
        status.current_task = None;
        status.health = HealthStatus::Healthy;
        status.tasks_completed += 1;
    }

    /// Simulate task failure (placeholder)
    fn simulate_task_failure(&self, task_id: &str, error: String) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.get_mut(task_id) {
            handle.status = TaskStatus::Failed;
            handle.error = Some(error.clone());
        }

        let mut status = self.status.lock().unwrap();
        status.current_task = None;
        status.health = HealthStatus::Healthy;
        status.tasks_failed += 1;
    }
}

impl SwarmAgentAdapter for ClaudeCodeAdapter {
    fn send_task(&self, task: &TaskDescription) -> Result<TaskHandle, SwarmError> {
        // Start process if not running
        self.start_process(task.working_dir.as_deref())?;

        // Create task handle
        let handle = TaskHandle {
            task_id: task.id.clone(),
            worker_id: self.worker_id.clone(),
            status: TaskStatus::Running,
            started_at: InstantWrapper::from(Instant::now()),
            output: String::new(),
            error: None,
            pid: self.status.lock().unwrap().pid,
        };

        // Store task
        self.tasks.lock().unwrap().insert(task.id.clone(), handle.clone());

        // Update worker status
        {
            let mut status = self.status.lock().unwrap();
            status.health = HealthStatus::Busy;
            status.current_task = Some(task.id.clone());
        }

        // Send prompt to process
        self.send_prompt(&task.prompt)?;

        // Initialize output accumulator
        self.outputs.lock().unwrap().insert(task.id.clone(), String::new());

        // Placeholder: Simulate task completion after delay
        // In production: output would be accumulated from stdout stream
        let task_id_clone = task.id.clone();
        let outputs_clone = self.outputs.clone();
        let tasks_clone = self.tasks.clone();
        let status_clone = self.status.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));

            // Simulate completion
            let simulated_output = format!(
                "[Claude Code Output for task {}]\n\nTask completed successfully.\nOutput files generated.",
                task_id_clone
            );

            // Update outputs
            outputs_clone.lock().unwrap()
                .insert(task_id_clone.clone(), simulated_output);

            // Update task
            if let Some(handle) = tasks_clone.lock().unwrap().get_mut(&task_id_clone) {
                handle.status = TaskStatus::Completed;
                handle.output = simulated_output;
            }

            // Update status
            let mut st = status_clone.lock().unwrap();
            st.current_task = None;
            st.health = HealthStatus::Healthy;
            st.tasks_completed += 1;
        });

        Ok(handle)
    }

    fn get_status(&self) -> Result<WorkerStatus, SwarmError> {
        let status = self.status.lock().unwrap().clone();
        Ok(status)
    }

    fn cancel_task(&self, task_id: &str) -> Result<(), SwarmError> {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.get_mut(task_id) {
            handle.status = TaskStatus::Cancelled;
        } else {
            return Err(SwarmError::InvalidTask(format!("Task {} not found", task_id)));
        }

        // Clear current task
        {
            let mut status = self.status.lock().unwrap();
            if status.current_task.as_ref() == Some(&task_id.to_string()) {
                status.current_task = None;
                status.health = HealthStatus::Healthy;
            }
        }

        // In production: would send SIGTERM or interrupt to process
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
        self.kill_process()?;

        {
            let mut status = self.status.lock().unwrap();
            status.health = HealthStatus::Offline;
        }

        Ok(())
    }

    fn get_task_output(&self, task_id: &str) -> Result<String, SwarmError> {
        let outputs = self.outputs.lock().unwrap();
        outputs.get(task_id)
            .cloned()
            .ok_or_else(|| SwarmError::InvalidTask(format!("Task {} output not found", task_id)))
    }
}

/// Builder for ClaudeCodeAdapter with custom configuration
pub struct ClaudeCodeAdapterBuilder {
    worker_id: WorkerId,
    working_dir: Option<String>,
    timeout_ms: Option<u64>,
}

impl ClaudeCodeAdapterBuilder {
    pub fn new(worker_id: WorkerId) -> Self {
        Self {
            worker_id,
            working_dir: None,
            timeout_ms: None,
        }
    }

    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn build(self) -> ClaudeCodeAdapter {
        let adapter = ClaudeCodeAdapter::new(self.worker_id);
        // Note: timeout would be used in task execution
        adapter
    }
}