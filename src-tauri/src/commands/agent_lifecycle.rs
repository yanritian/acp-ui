use crate::AppState;
use crate::agent::{AgentInstance, AgentStatus};
use crate::config::{AgentConfig, AgentTransport, AgentsConfig};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn spawn_agent(
    name: String,
    state: State<AppState>,
    app_handle: AppHandle,
) -> Result<AgentInstance, String> {
    let config_manager = state.config_manager.read();
    let config = config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .get_config();

    let agent_config = config
        .agents
        .get(&name)
        .ok_or_else(|| format!("Agent '{}' not found in config", name))?;

    let hooks = agent_config.hooks.clone();

    let instance = state
        .agent_manager
        .spawn_agent(name, agent_config, app_handle)?;

    // Register agent in the communication bus
    if let Ok(mut bus) = state.agent_bus.lock() {
        let _ = bus.register_agent(&instance.id);
    }

    // Register hooks for the agent if any are configured
    if !hooks.is_empty() {
        if let Ok(mut executor) = state.hooks_executor.lock() {
            executor.register_agent_hooks(&instance.id, &hooks);
        }
    }

    Ok(instance)
}

#[tauri::command]
pub fn send_to_agent(agent_id: String, message: String, state: State<AppState>) -> Result<(), String> {
    state.agent_manager.send_message(&agent_id, &message)
}

#[tauri::command]
pub fn kill_agent(agent_id: String, state: State<AppState>) -> Result<(), String> {
    // Unregister hooks for this agent before killing
    if let Ok(mut executor) = state.hooks_executor.lock() {
        executor.unregister_agent_hooks(&agent_id);
    }

    // Unregister agent from the communication bus
    if let Ok(mut bus) = state.agent_bus.lock() {
        let _ = bus.unregister_agent(&agent_id);
    }

    state.agent_manager.kill_agent(&agent_id)
}

#[tauri::command]
pub fn list_running_agents(state: State<AppState>) -> Vec<String> {
    state.agent_manager.list_running_agents()
}

#[tauri::command]
pub fn get_running_agents_info(state: State<AppState>) -> Vec<AgentInstance> {
    state.agent_manager.get_running_agents_info()
}

#[tauri::command]
pub fn get_agent_status(agent_id: String, state: State<AppState>) -> Result<AgentStatus, String> {
    state.agent_manager.get_agent_status(&agent_id)
}

#[tauri::command]
pub fn pause_agent(agent_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.pause_agent(&agent_id, &app_handle)
}

#[tauri::command]
pub fn resume_agent(agent_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.resume_agent(&agent_id, &app_handle)
}

#[tauri::command]
pub fn inject_message(agent_id: String, message: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.inject_message(&agent_id, &message, &app_handle)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_agent(
    name: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    state: State<AppState>,
) -> Result<AgentsConfig, String> {
    let agent_config = build_agent_config(command, args, env, transport, url, headers)?;
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .add_agent(name, agent_config)
}

#[tauri::command]
pub fn remove_agent(name: String, state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .remove_agent(&name)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_agent(
    name: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    state: State<AppState>,
) -> Result<AgentsConfig, String> {
    let agent_config = build_agent_config(command, args, env, transport, url, headers)?;
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .update_agent(name, agent_config)
}

/// Build an `AgentConfig` from the loosely-typed Tauri command arguments,
/// applying validation rules per transport kind.
pub fn build_agent_config(
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
) -> Result<AgentConfig, String> {
    let transport_kind = match transport.as_deref() {
        None | Some("") | Some("stdio") => AgentTransport::Stdio,
        Some("websocket") | Some("ws") | Some("wss") => AgentTransport::Websocket,
        Some("http") | Some("https") => AgentTransport::Http,
        Some(other) => return Err(format!("Unknown transport: {}", other)),
    };

    // Defense in depth: stdio agents can't run on mobile (no subprocess).
    // The frontend already filters them out, but reject here too so a
    // malicious renderer or synced config can't smuggle one through the
    // IPC boundary.
    #[cfg(not(desktop))]
    if matches!(transport_kind, AgentTransport::Stdio) {
        return Err("stdio agents are not supported on this platform".to_string());
    }

    match transport_kind {
        AgentTransport::Stdio => {
            let command = command
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "stdio agent requires a command".to_string())?;
            Ok(AgentConfig {
                transport: AgentTransport::Stdio,
                command: Some(command),
                args: Some(args.unwrap_or_default()),
                env: env.unwrap_or_default(),
                url: None,
                headers: None,
                cwd: None,
                mcp_servers: Vec::new(),
                skills: Vec::new(),
                hooks: Vec::new(),
                capabilities: Vec::new(),
            })
        }
        AgentTransport::Websocket | AgentTransport::Http => {
            let url = url
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "remote agent requires a url".to_string())?;
            // Sanity-check scheme matches transport so users get an early error.
            let lower = url.to_ascii_lowercase();
            let scheme_ok = match transport_kind {
                AgentTransport::Websocket => {
                    lower.starts_with("ws://") || lower.starts_with("wss://")
                }
                AgentTransport::Http => {
                    lower.starts_with("http://") || lower.starts_with("https://")
                }
                _ => true,
            };
            if !scheme_ok {
                return Err(format!(
                    "URL scheme does not match transport '{:?}': {}",
                    transport_kind, url
                ));
            }
            let headers = headers.filter(|h| !h.is_empty());
            Ok(AgentConfig {
                transport: transport_kind,
                command: None,
                args: None,
                env: std::collections::HashMap::new(),
                url: Some(url),
                headers,
                cwd: None,
                mcp_servers: Vec::new(),
                skills: Vec::new(),
                hooks: Vec::new(),
                capabilities: Vec::new(),
            })
        }
    }
}
