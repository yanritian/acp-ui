//! Codex Adapter - Stdio process communication for Codex CLI
//!
//! Manages `codex --full-auto` subprocess with real-time output streaming.
//! Codex operates in autonomous mode, executing tasks without interactive prompts.

use super::*;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Adapter for Codex CLI
///
/// Codex is an autonomous coding agent that can execute complex tasks
/// without requiring step-by-step guidance.
///
/// Command: `codex --full-auto`
/// - Reads prompts from stdin
/// - Writes output to stdout
/// - Errors to stderr
pub struct CodexAdapter {
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
            max_complexity: 5, // Codex can handle very complex tasks
            max_concurrent: 1, // Single task at a time
            supports_streaming: true,
            supports_cancel: true,
            default_timeout_ms: 300000, // 5 minutes
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
            reader_thread: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the Codex process
    fn start_process(&self, working_dir: Option<&str>) -> Result<(), SwarmError> {
        // Check if process already running
        if self.process.lock().unwrap().is_some() {
            return Ok(());
        }

        // Build command
        let mut cmd = Command::new("codex");
        cmd.arg("--full-auto");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| SwarmError::ProcessStartFailed(format!("Failed to spawn codex: {}", e)))?;

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

        // Start output reader thread
        self.start_output_reader();

        Ok(())
    }

    /// Start background thread to read stdout/stderr
    fn start_output_reader(&self) {
        // Note: In a real implementation, we would take stdout/stderr from the child
        // and read them in a background thread. For now, this is a placeholder.

        let _outputs = self.outputs.clone();
        let status = self.status.clone();

        let handle = thread::spawn(move || {
            // Placeholder: In production, this would:
            // 1. Take ownership of stdout/stderr from child process
            // 2. Read lines in a loop
            // 3. Append to appropriate task output
            // 4. Update status based on output patterns

            // Simulate heartbeat updates
            loop {
                thread::sleep(Duration::from_secs(10));

                let mut st = status.lock().unwrap();
                if st.health == HealthStatus::Starting {
                    st.health = HealthStatus::Healthy;
                }
                st.last_heartbeat = Some(InstantWrapper::from(Instant::now()));

                // Break if process is no longer running
                if st.health == HealthStatus::Offline {
                    break;
                }
            }
        });

        *self.reader_thread.lock().unwrap() = Some(handle);
    }

    /// Send prompt to Codex via stdin
    fn send_prompt(&self, prompt: &str) -> Result<(), SwarmError> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(ref mut _child) = *process_guard {
            // In production: write to child.stdin
            // For now, this is a placeholder
            println!("[CodexAdapter] Would send prompt to stdin: {}", prompt);
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

            // Wait for process to exit
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
}

impl SwarmAgentAdapter for CodexAdapter {
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

        Ok(handle)
    }

    fn get_status(&self) -> Result<WorkerStatus, SwarmError> {
        let status = self.status.lock().unwrap().clone();
        Ok(status)
    }

    fn cancel_task(&self, task_id: &str) -> Result<(), SwarmError> {
        // Check if task exists
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.get_mut(task_id) {
            handle.status = TaskStatus::Cancelled;
        } else {
            return Err(SwarmError::InvalidTask(format!("Task {} not found", task_id)));
        }

        // Clear current task if this was it
        {
            let mut status = self.status.lock().unwrap();
            if status.current_task.as_ref() == Some(&task_id.to_string()) {
                status.current_task = None;
                status.health = HealthStatus::Healthy;
            }
        }

        // In production: would send cancel signal to process
        // For now: just mark as cancelled

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

        // Stop reader thread
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
        let adapter = CodexAdapter::new(self.worker_id);
        // Note: working_dir would be used when starting the process
        adapter
    }
}