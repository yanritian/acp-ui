//! Hook Executor Module (Claw Code inspired)
//!
//! Handles hook execution with timeout, abort signals, and structured output parsing.
//! Provides Tauri commands for hook registration, execution, and management.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tauri::State;

use crate::operator::{is_d_drive_path, workspace_root};
use crate::AppState;

const MAX_HOOK_TIMEOUT_MS: u64 = 120_000;
const MAX_HOOK_OUTPUT_BYTES: usize = 64 * 1024;

/// Hook execution result (Claw Code inspired)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub should_block: bool,
    pub message: Option<String>,
    /// NEW: Modified input for tool execution (Claw Code)
    pub updated_input: Option<String>,
    /// NEW: Abort signal for stopping execution (Claw Code)
    pub abort_signal: Option<HookAbortSignal>,
}

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookConfig {
    pub name: String,
    pub script_path: String,
    pub hook_type: HookType,
    pub timeout_ms: Option<u64>,
    pub block_on_failure: Option<bool>,
}

/// Hook type (Claw Code inspired)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure, // NEW: triggered when tool execution fails
    Stop,
}

/// Hook abort signal (Claw Code inspired)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookAbortSignal {
    pub aborted: bool,
    pub reason: Option<String>,
}

impl HookAbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: false,
            reason: None,
        }
    }

    pub fn abort(&mut self, reason: Option<String>) {
        self.aborted = true;
        self.reason = reason;
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
}

impl Default for HookAbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Hooks Executor - executes hook scripts for agent operations
pub struct HooksExecutor {
    hooks: HashMap<String, HookConfig>,
    agent_hooks: HashMap<String, Vec<String>>, // agent_id -> hook_names
}

impl HooksExecutor {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            agent_hooks: HashMap::new(),
        }
    }

    /// Register a hook
    pub fn register_hook(&mut self, config: HookConfig) {
        let name = config.name.clone();
        self.hooks.insert(name.clone(), config);
        println!("Registered hook: {}", name);
    }

    /// Register hooks for an agent
    pub fn register_agent_hooks(&mut self, agent_id: &str, hook_names: &[String]) {
        let valid_hooks: Vec<String> = hook_names
            .iter()
            .filter(|name| self.hooks.contains_key(*name))
            .cloned()
            .collect();

        let hook_count = valid_hooks.len();
        self.agent_hooks.insert(agent_id.to_string(), valid_hooks);
        println!("Registered {} hooks for agent: {}", hook_count, agent_id);
    }

    /// Unregister all hooks for an agent
    pub fn unregister_agent_hooks(&mut self, agent_id: &str) {
        self.agent_hooks.remove(agent_id);
        println!("Unregistered all hooks for agent: {}", agent_id);
    }

    /// Get hooks for an agent
    pub fn get_agent_hooks(&self, agent_id: &str) -> Vec<&HookConfig> {
        self.agent_hooks
            .get(agent_id)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.hooks.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Execute PreToolUse hooks
    pub fn execute_pre_tool_use(
        &self,
        agent_id: &str,
        tool_name: &str,
        tool_args: &str,
    ) -> HookResult {
        let agent_hooks = self.get_agent_hooks(agent_id);

        for hook_config in agent_hooks
            .iter()
            .filter(|h| h.hook_type == HookType::PreToolUse)
        {
            let result = self.execute_hook(hook_config, agent_id, tool_name, Some(tool_args), None);

            // If hook returns should_block, stop execution
            if result.should_block {
                return result;
            }

            // If hook fails and block_on_failure is true
            if !result.success && hook_config.block_on_failure.unwrap_or(false) {
                return HookResult {
                    success: false,
                    output: result.output,
                    error: result.error,
                    should_block: true,
                    message: Some(format!(
                        "Hook '{}' failed, blocking operation",
                        hook_config.name
                    )),
                    updated_input: None,
                    abort_signal: None,
                };
            }
        }

        // All hooks passed
        HookResult {
            success: true,
            output: None,
            error: None,
            should_block: false,
            message: Some("All PreToolUse hooks passed".to_string()),
            updated_input: None,
            abort_signal: None,
        }
    }

    /// Execute PostToolUse hooks
    pub fn execute_post_tool_use(
        &self,
        agent_id: &str,
        tool_name: &str,
        tool_args: &str,
        tool_result: &str,
    ) -> HookResult {
        let agent_hooks = self.get_agent_hooks(agent_id);

        let mut all_outputs = Vec::new();
        let mut all_errors = Vec::new();

        for hook_config in agent_hooks
            .iter()
            .filter(|h| h.hook_type == HookType::PostToolUse)
        {
            let result = self.execute_hook(
                hook_config,
                agent_id,
                tool_name,
                Some(tool_args),
                Some(tool_result),
            );

            if let Some(output) = result.output {
                all_outputs.push(format!("[{}] {}", hook_config.name, output));
            }
            if let Some(error) = result.error {
                all_errors.push(format!("[{}] {}", hook_config.name, error));
            }
        }

        HookResult {
            success: all_errors.is_empty(),
            output: if all_outputs.is_empty() {
                None
            } else {
                Some(all_outputs.join("\n"))
            },
            error: if all_errors.is_empty() {
                None
            } else {
                Some(all_errors.join("\n"))
            },
            should_block: false,
            message: Some(format!(
                "Executed {} PostToolUse hooks",
                agent_hooks
                    .iter()
                    .filter(|h| h.hook_type == HookType::PostToolUse)
                    .count()
            )),
            updated_input: None,
            abort_signal: None,
        }
    }

    /// Execute PostToolUseFailure hooks (Claw Code inspired)
    /// Triggered when a tool execution fails
    pub fn execute_post_tool_use_failure(
        &self,
        agent_id: &str,
        tool_name: &str,
        tool_args: &str,
        tool_error: &str,
    ) -> HookResult {
        let agent_hooks = self.get_agent_hooks(agent_id);

        let mut all_outputs = Vec::new();
        let mut all_errors = Vec::new();
        let mut abort_signal = HookAbortSignal::new();

        for hook_config in agent_hooks
            .iter()
            .filter(|h| h.hook_type == HookType::PostToolUseFailure)
        {
            let result = self.execute_hook_with_error(
                hook_config,
                agent_id,
                tool_name,
                Some(tool_args),
                Some(tool_error),
            );

            if let Some(output) = result.output {
                all_outputs.push(format!("[{}] {}", hook_config.name, output));
            }
            if let Some(error) = result.error {
                all_errors.push(format!("[{}] {}", hook_config.name, error));
            }

            // Check for abort signal from hook
            if let Some(signal) = &result.abort_signal {
                if signal.is_aborted() {
                    abort_signal.abort(signal.reason.clone());
                    break;
                }
            }
        }

        HookResult {
            success: all_errors.is_empty(),
            output: if all_outputs.is_empty() {
                None
            } else {
                Some(all_outputs.join("\n"))
            },
            error: if all_errors.is_empty() {
                None
            } else {
                Some(all_errors.join("\n"))
            },
            should_block: false,
            message: Some(format!(
                "Executed {} PostToolUseFailure hooks",
                agent_hooks
                    .iter()
                    .filter(|h| h.hook_type == HookType::PostToolUseFailure)
                    .count()
            )),
            updated_input: None,
            abort_signal: if abort_signal.is_aborted() {
                Some(abort_signal)
            } else {
                None
            },
        }
    }

    /// Execute a single hook
    fn execute_hook(
        &self,
        hook_config: &HookConfig,
        agent_id: &str,
        tool_name: &str,
        tool_args: Option<&str>,
        tool_result: Option<&str>,
    ) -> HookResult {
        let script_path = match validate_hook_script_path(&hook_config.script_path) {
            Ok(path) => path,
            Err(error) => {
                return HookResult {
                    success: false,
                    output: None,
                    error: Some(error),
                    should_block: hook_config.block_on_failure.unwrap_or(false),
                    message: None,
                    updated_input: None,
                    abort_signal: None,
                }
            }
        };

        // Check if script exists
        if !script_path.exists() {
            return HookResult {
                success: false,
                output: None,
                error: Some(format!(
                    "Hook script not found: {}",
                    hook_config.script_path
                )),
                should_block: false,
                message: None,
                updated_input: None,
                abort_signal: None,
            };
        }

        // Build command
        let mut cmd = Command::new(&script_path);

        // Pass environment variables
        cmd.env("AGENT_ID", agent_id);
        cmd.env("TOOL_NAME", tool_name);
        if let Some(args) = tool_args {
            cmd.env("TOOL_ARGS", args);
        }
        if let Some(result) = tool_result {
            cmd.env("TOOL_RESULT", result);
        }
        cmd.env("HOOK_NAME", &hook_config.name);
        cmd.env(
            "HOOK_TYPE",
            match hook_config.hook_type {
                HookType::PreToolUse => "pre",
                HookType::PostToolUse => "post",
                HookType::PostToolUseFailure => "postFailure",
                HookType::Stop => "stop",
            },
        );

        // Execute with timeout (actually enforced)
        let timeout_ms = hook_config
            .timeout_ms
            .unwrap_or(30000)
            .clamp(1, MAX_HOOK_TIMEOUT_MS);

        #[cfg(desktop)]
        let output_result: Result<Output, std::io::Error> = {
            // Capture stdout/stderr
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            // Spawn and wait with timeout using polling approach
            match cmd.spawn() {
                Ok(mut child) => {
                    let start = std::time::Instant::now();
                    let timeout_duration = std::time::Duration::from_millis(timeout_ms);

                    // Poll for completion with timeout check
                    loop {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                // Process completed - collect output
                                let stdout = child
                                    .stdout
                                    .take()
                                    .map(|mut s| {
                                        let mut buf = Vec::new();
                                        let _ = s.read_to_end(&mut buf);
                                        buf
                                    })
                                    .unwrap_or_default();
                                let stderr = child
                                    .stderr
                                    .take()
                                    .map(|mut s| {
                                        let mut buf = Vec::new();
                                        let _ = s.read_to_end(&mut buf);
                                        buf
                                    })
                                    .unwrap_or_default();
                                break Ok(Output {
                                    status,
                                    stdout,
                                    stderr,
                                });
                            }
                            Ok(None) => {
                                // Still running - check timeout
                                if start.elapsed() > timeout_duration {
                                    let _ = child.kill();
                                    let _ = child.wait();
                                    break Err(std::io::Error::new(
                                        std::io::ErrorKind::TimedOut,
                                        format!("Hook timed out after {}ms", timeout_ms),
                                    ));
                                }
                                // Sleep briefly before checking again
                                std::thread::sleep(std::time::Duration::from_millis(50));
                            }
                            Err(e) => break Err(e),
                        }
                    }
                }
                Err(e) => Err(e),
            }
        };

        #[cfg(not(desktop))]
        let output_result: Result<Output, std::io::Error> = {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Hooks not supported on non-desktop platforms",
            ))
        };

        match output_result {
            Ok(output) => {
                let stdout = truncate_output(&String::from_utf8_lossy(&output.stdout));
                let stderr = truncate_output(&String::from_utf8_lossy(&output.stderr));

                // Parse output for block/deny signals using structured protocol
                // Require dedicated signal prefix to avoid false positives from normal output
                let should_block = stdout.contains("__HOOK_SIGNAL:BLOCK__")
                    || stdout.contains("__HOOK_SIGNAL:DENY__")
                    || stderr.contains("__HOOK_SIGNAL:BLOCK__")
                    || stderr.contains("__HOOK_SIGNAL:DENY__");

                // Parse for updated input (structured format)
                let updated_input = if stdout.contains("__HOOK_INPUT:") {
                    // Extract content between __HOOK_INPUT: and end marker __END__
                    if let Some(start) = stdout.find("__HOOK_INPUT:") {
                        let rest = &stdout[start + "__HOOK_INPUT:".len()..];
                        if let Some(end) = rest.find("__END__") {
                            Some(rest[..end].trim().to_string())
                        } else {
                            Some(rest.trim().to_string())
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Parse for abort signal (structured format)
                let abort_signal = if stdout.contains("__HOOK_ABORT:") {
                    if let Some(start) = stdout.find("__HOOK_ABORT:") {
                        let rest = &stdout[start + "__HOOK_ABORT:".len()..];
                        if let Some(end) = rest.find("__END__") {
                            Some(HookAbortSignal {
                                aborted: true,
                                reason: Some(rest[..end].trim().to_string()),
                            })
                        } else {
                            Some(HookAbortSignal {
                                aborted: true,
                                reason: Some(rest.trim().to_string()),
                            })
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                HookResult {
                    success: output.status.success(),
                    output: if stdout.is_empty() {
                        None
                    } else {
                        Some(stdout)
                    },
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                    should_block,
                    message: None,
                    updated_input,
                    abort_signal,
                }
            }
            Err(e) => HookResult {
                success: false,
                output: None,
                error: Some(format!("Failed to execute hook: {}", e)),
                should_block: hook_config.block_on_failure.unwrap_or(false),
                message: None,
                updated_input: None,
                abort_signal: None,
            },
        }
    }

    /// Execute a single hook with error context (for PostToolUseFailure)
    fn execute_hook_with_error(
        &self,
        hook_config: &HookConfig,
        agent_id: &str,
        tool_name: &str,
        tool_args: Option<&str>,
        tool_error: Option<&str>,
    ) -> HookResult {
        let script_path = match validate_hook_script_path(&hook_config.script_path) {
            Ok(path) => path,
            Err(error) => {
                return HookResult {
                    success: false,
                    output: None,
                    error: Some(error),
                    should_block: hook_config.block_on_failure.unwrap_or(false),
                    message: None,
                    updated_input: None,
                    abort_signal: None,
                }
            }
        };

        // Check if script exists
        if !script_path.exists() {
            return HookResult {
                success: false,
                output: None,
                error: Some(format!(
                    "Hook script not found: {}",
                    hook_config.script_path
                )),
                should_block: false,
                message: None,
                updated_input: None,
                abort_signal: None,
            };
        }

        // Build command
        let mut cmd = Command::new(&script_path);

        // Pass environment variables
        cmd.env("AGENT_ID", agent_id);
        cmd.env("TOOL_NAME", tool_name);
        if let Some(args) = tool_args {
            cmd.env("TOOL_ARGS", args);
        }
        if let Some(error) = tool_error {
            cmd.env("TOOL_ERROR", error);
        }
        cmd.env("HOOK_NAME", &hook_config.name);
        cmd.env("HOOK_TYPE", "postFailure");

        // Execute command
        match cmd.output() {
            Ok(output) => {
                let stdout = truncate_output(&String::from_utf8_lossy(&output.stdout));
                let stderr = truncate_output(&String::from_utf8_lossy(&output.stderr));

                // Parse for abort signal
                let abort_signal = if stdout.contains("ABORT:") {
                    let reason = stdout
                        .split("ABORT:")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    Some(HookAbortSignal {
                        aborted: true,
                        reason: Some(reason),
                    })
                } else {
                    None
                };

                HookResult {
                    success: output.status.success(),
                    output: if stdout.is_empty() {
                        None
                    } else {
                        Some(stdout)
                    },
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                    should_block: false,
                    message: None,
                    updated_input: None,
                    abort_signal,
                }
            }
            Err(e) => HookResult {
                success: false,
                output: None,
                error: Some(format!("Failed to execute failure hook: {}", e)),
                should_block: false,
                message: None,
                updated_input: None,
                abort_signal: None,
            },
        }
    }

    /// Validate agent has hook registered (isolation check)
    pub fn validate_hook_access(&self, agent_id: &str, hook_name: &str) -> bool {
        self.agent_hooks
            .get(agent_id)
            .map(|hooks| hooks.contains(&hook_name.to_string()))
            .unwrap_or(false)
    }

    /// List all registered hooks
    pub fn list_hooks(&self) -> Vec<&HookConfig> {
        self.hooks.values().collect()
    }
}

fn validate_hook_script_path(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if !path.is_absolute() || !is_d_drive_path(&path) {
        return Err(format!(
            "Hook script must be an absolute D: path, got '{}'",
            raw
        ));
    }
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| format!("Cannot inspect hook script '{}': {}", raw, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Hook script is not a regular file: {}",
            path.display()
        ));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot canonicalize hook script '{}': {}", raw, error))?;
    let canonical_workspace = workspace_root()
        .canonicalize()
        .unwrap_or_else(|_| workspace_root());
    if !canonical.starts_with(&canonical_workspace) {
        return Err(format!(
            "Hook script must stay inside the D: workspace: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn truncate_output(value: &str) -> String {
    if value.len() <= MAX_HOOK_OUTPUT_BYTES {
        return value.to_string();
    }
    let mut end = MAX_HOOK_OUTPUT_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[hook output truncated]", &value[..end])
}

impl Default for HooksExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Example hook scripts (Claw Code inspired)
pub fn get_example_hooks() -> Vec<HookConfig> {
    vec![
        HookConfig {
            name: "pre-commit-check".to_string(),
            script_path: workspace_root()
                .join("hooks/pre-tool-use.cmd")
                .to_string_lossy()
                .to_string(),
            hook_type: HookType::PreToolUse,
            timeout_ms: Some(5000),
            block_on_failure: Some(true),
        },
        HookConfig {
            name: "post-format".to_string(),
            script_path: workspace_root()
                .join("hooks/post-tool-use.cmd")
                .to_string_lossy()
                .to_string(),
            hook_type: HookType::PostToolUse,
            timeout_ms: Some(10000),
            block_on_failure: Some(false),
        },
        HookConfig {
            name: "failure-handler".to_string(),
            script_path: workspace_root()
                .join("hooks/post-tool-use-failure.cmd")
                .to_string_lossy()
                .to_string(),
            hook_type: HookType::PostToolUseFailure,
            timeout_ms: Some(30000),
            block_on_failure: Some(false),
        },
    ]
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Register a new hook
#[tauri::command]
pub fn hook_register(
    state: State<'_, AppState>,
    name: String,
    script_path: String,
    hook_type: HookType,
    timeout_ms: Option<u64>,
    block_on_failure: Option<bool>,
) -> Result<String, String> {
    validate_hook_script_path(&script_path)?;
    let config = HookConfig {
        name: name.clone(),
        script_path,
        hook_type,
        timeout_ms,
        block_on_failure,
    };
    state
        .hooks_executor
        .lock()
        .map_err(|e| e.to_string())?
        .register_hook(config);
    Ok(format!("Hook '{}' registered", name))
}

/// Register hooks for a specific agent
#[tauri::command]
pub fn hook_register_for_agent(
    state: State<'_, AppState>,
    agent_id: String,
    hook_names: Vec<String>,
) -> Result<String, String> {
    state
        .hooks_executor
        .lock()
        .map_err(|e| e.to_string())?
        .register_agent_hooks(&agent_id, &hook_names);
    Ok(format!(
        "Registered {} hooks for agent '{}'",
        hook_names.len(),
        agent_id
    ))
}

/// Unregister all hooks for an agent
#[tauri::command]
pub fn hook_unregister_agent(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<String, String> {
    state
        .hooks_executor
        .lock()
        .map_err(|e| e.to_string())?
        .unregister_agent_hooks(&agent_id);
    Ok(format!("Unregistered all hooks for agent '{}'", agent_id))
}

/// Execute PreToolUse hooks for a tool invocation
#[tauri::command]
pub fn hook_execute_pre_tool(
    state: State<'_, AppState>,
    agent_id: String,
    tool_name: String,
    tool_args: String,
) -> Result<HookResult, String> {
    let executor = state.hooks_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor.execute_pre_tool_use(&agent_id, &tool_name, &tool_args))
}

/// Execute PostToolUse hooks after a tool invocation
#[tauri::command]
pub fn hook_execute_post_tool(
    state: State<'_, AppState>,
    agent_id: String,
    tool_name: String,
    tool_args: String,
    tool_result: String,
) -> Result<HookResult, String> {
    let executor = state.hooks_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor.execute_post_tool_use(&agent_id, &tool_name, &tool_args, &tool_result))
}

/// Execute PostToolUseFailure hooks when a tool fails
#[tauri::command]
pub fn hook_execute_post_tool_failure(
    state: State<'_, AppState>,
    agent_id: String,
    tool_name: String,
    tool_args: String,
    tool_error: String,
) -> Result<HookResult, String> {
    let executor = state.hooks_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor.execute_post_tool_use_failure(&agent_id, &tool_name, &tool_args, &tool_error))
}

/// List all registered hooks
#[tauri::command]
pub fn hook_list(state: State<'_, AppState>) -> Result<Vec<HookConfig>, String> {
    let executor = state.hooks_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor.list_hooks().into_iter().cloned().collect())
}

/// Get hooks registered for a specific agent
#[tauri::command]
pub fn hook_get_agent_hooks(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Vec<HookConfig>, String> {
    let executor = state.hooks_executor.lock().map_err(|e| e.to_string())?;
    Ok(executor
        .get_agent_hooks(&agent_id)
        .into_iter()
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_hook_fixture_executes_without_blocking() {
        let mut executor = HooksExecutor::new();
        for hook in get_example_hooks() {
            executor.register_hook(hook);
        }
        executor.register_agent_hooks("fixture-agent", &["pre-commit-check".to_string()]);
        let result = executor.execute_pre_tool_use(
            "fixture-agent",
            "godot.analyze",
            "{\"project_path\":\"D:/dingsun/acp-ui/test-godot-project\"}",
        );
        assert!(result.success, "hook failed: {:?}", result.error);
        assert!(!result.should_block);
    }

    #[test]
    fn hook_registry_rejects_non_d_drive_scripts() {
        let result = validate_hook_script_path("C:/not-allowed.cmd");
        assert!(result.is_err());
    }
}
