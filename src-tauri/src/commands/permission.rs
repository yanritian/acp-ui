use crate::permission_checker::{self, PermissionChecker, PermissionConfig, PermissionResult};

// ===== Permission Commands =====

#[tauri::command]
pub fn check_permission(
    tool_name: String,
    args: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_tool(&tool_name, &args))
}

#[tauri::command]
pub fn check_bash_permission(
    command: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_bash(&command))
}

#[tauri::command]
pub fn check_path_permission(
    path: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_path(&path))
}

#[tauri::command]
pub fn get_default_permissions(agent_type: String) -> PermissionConfig {
    permission_checker::get_default_permissions(&agent_type)
}
