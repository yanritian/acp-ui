use crate::config::AgentsConfig;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.get_config())
        .ok_or_else(|| "Config manager not initialized".to_string())
}

#[tauri::command]
pub fn reload_config(state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.reload())
        .ok_or_else(|| "Config manager not initialized".to_string())?
}

#[tauri::command]
pub fn get_config_path(state: State<AppState>) -> Result<String, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.get_config_path().to_string_lossy().to_string())
        .ok_or_else(|| "Config manager not initialized".to_string())
}

#[tauri::command]
pub fn get_machine_id() -> Result<String, String> {
    // `machine-uid` is desktop-only (no support for iOS / Android). Telemetry
    // on mobile falls back to an anonymous id (the frontend handles a failure
    // here by leaving `machineId = null`).
    #[cfg(desktop)]
    {
        machine_uid::get().map_err(|e| format!("Failed to get machine ID: {}", e))
    }
    #[cfg(not(desktop))]
    {
        Err("machine id is not available on this platform".to_string())
    }
}
