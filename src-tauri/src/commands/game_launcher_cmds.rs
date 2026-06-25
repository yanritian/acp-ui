// Game Launcher Commands
// Phase 3 Day 8: Tauri commands for game launcher

use crate::game_launcher::{GameLauncher, LaunchRequest, LaunchResponse, GameStatus};

#[tauri::command]
pub async fn game_launch(request: LaunchRequest) -> Result<LaunchResponse, String> {
    let launcher = GameLauncher::new();
    launcher.launch_game(request).await
}

#[tauri::command]
pub async fn game_stop(game_id: String) -> Result<(), String> {
    let launcher = GameLauncher::new();
    launcher.stop_game(&game_id)
}

#[tauri::command]
pub async fn game_get_status(game_id: String) -> Result<Option<GameStatus>, String> {
    let launcher = GameLauncher::new();
    Ok(launcher.get_status(&game_id))
}

#[tauri::command]
pub async fn game_list_running() -> Result<Vec<GameStatus>, String> {
    let launcher = GameLauncher::new();
    Ok(launcher.get_all_running())
}
