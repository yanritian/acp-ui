// Game Launcher
// Phase 3 Day 8: Launch and manage running games

use crate::game_detector::{GameDetector, GameEngine, GameInfo, GameSize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

// ===== Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub cwd: String,
    pub executable: Option<String>,  // Path to executable (auto-detect if not provided)
    pub args: Option<Vec<String>>,   // Additional arguments
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResponse {
    pub success: bool,
    pub game_id: Option<String>,
    pub process_id: Option<u32>,
    pub engine: String,
    pub mode: LaunchMode,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    Embedded,      // WebView embedded (small games)
    External,      // External process (large games)
    Editor,        // Open in game editor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub game_id: String,
    pub engine: String,
    pub process_id: u32,
    pub is_running: bool,
    pub started_at: u64,
    pub elapsed_ms: u64,
    pub memory_mb: Option<u64>,
}

// ===== Game Launcher =====

pub struct GameLauncher {
    running_games: Arc<Mutex<HashMap<String, RunningGame>>>,
}

struct RunningGame {
    game_id: String,
    engine: GameEngine,
    child: Child,
    started_at: std::time::Instant,
}

impl GameLauncher {
    pub fn new() -> Self {
        Self {
            running_games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Launch a game
    pub async fn launch_game(&self, request: LaunchRequest) -> Result<LaunchResponse, String> {
        let cwd = PathBuf::from(&request.cwd);

        // Detect game
        let info = GameDetector::detect(&cwd)
            .map_err(|e| format!("Failed to detect game: {}", e))?;

        // Determine launch mode based on game size
        let mode = match info.size {
            GameSize::Small => LaunchMode::Embedded,
            GameSize::Large => LaunchMode::External,
        };

        // Find executable
        let executable = if let Some(exe) = &request.executable {
            PathBuf::from(exe)
        } else {
            self.find_executable(&cwd, &info.engine)?
        };

        if !executable.exists() {
            return Err(format!("Executable not found: {}", executable.display()));
        }

        // Launch game
        let mut cmd = Command::new(&executable);

        // Add arguments
        if let Some(args) = &request.args {
            cmd.args(args);
        }

        // Start process
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to launch game: {}", e))?;

        let process_id = child.id();
        let game_id = format!("game-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());

        // Store running game
        let running = RunningGame {
            game_id: game_id.clone(),
            engine: info.engine.clone(),
            child,
            started_at: std::time::Instant::now(),
        };

        let mut games = self.running_games.lock().unwrap();
        games.insert(game_id.clone(), running);

        let engine_name = match info.engine {
            GameEngine::Godot => "Godot",
            GameEngine::Unity => "Unity",
            GameEngine::Unknown => "Unknown",
        };

        Ok(LaunchResponse {
            success: true,
            game_id: Some(game_id),
            process_id: Some(process_id),
            engine: engine_name.to_string(),
            mode,
            message: format!("Game launched successfully (PID: {})", process_id),
            error: None,
        })
    }

    /// Find game executable
    fn find_executable(&self, cwd: &PathBuf, engine: &GameEngine) -> Result<PathBuf, String> {
        let search_dirs = vec![
            cwd.join("export"),
            cwd.join("build"),
            cwd.join("bin"),
            cwd.clone(),
        ];

        let extensions = match engine {
            GameEngine::Godot => vec!["exe", "x86_64", "app"],
            GameEngine::Unity => vec!["exe", "x86_64", "app"],
            GameEngine::Unknown => vec!["exe"],
        };

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if extensions.contains(&ext) {
                                return Ok(path);
                            }
                        }
                    }
                }
            }
        }

        Err("No executable found. Please build the game first or specify executable path.".to_string())
    }

    /// Check if game is still running
    pub fn is_running(&self, game_id: &str) -> bool {
        let mut games = self.running_games.lock().unwrap();

        if let Some(game) = games.get_mut(game_id) {
            match game.child.try_wait() {
                Ok(Some(_)) => false,  // Process exited
                Ok(None) => true,      // Still running
                Err(_) => false,       // Error checking
            }
        } else {
            false
        }
    }

    /// Stop a running game
    pub fn stop_game(&self, game_id: &str) -> Result<(), String> {
        let mut games = self.running_games.lock().unwrap();

        if let Some(mut game) = games.remove(game_id) {
            game.child.kill()
                .map_err(|e| format!("Failed to stop game: {}", e))?;
            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    /// Get status of a running game
    pub fn get_status(&self, game_id: &str) -> Option<GameStatus> {
        let mut games = self.running_games.lock().unwrap();

        if let Some(game) = games.get_mut(game_id) {
            let elapsed = game.started_at.elapsed().as_millis() as u64;
            let is_running = match game.child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            };

            Some(GameStatus {
                game_id: game_id.to_string(),
                engine: match game.engine {
                    GameEngine::Godot => "Godot".to_string(),
                    GameEngine::Unity => "Unity".to_string(),
                    GameEngine::Unknown => "Unknown".to_string(),
                },
                process_id: game.child.id(),
                is_running,
                started_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                elapsed_ms: elapsed,
                memory_mb: None,  // TODO: Implement memory tracking
            })
        } else {
            None
        }
    }

    /// Get all running games
    pub fn get_all_running(&self) -> Vec<GameStatus> {
        let games = self.running_games.lock().unwrap();
        games.keys()
            .filter_map(|id| self.get_status(id))
            .collect()
    }

    /// Clean up stopped games
    pub fn cleanup_stopped(&self) {
        let mut games = self.running_games.lock().unwrap();
        let mut stopped = Vec::new();

        for (id, game) in games.iter_mut() {
            if let Ok(Some(_)) = game.child.try_wait() {
                stopped.push(id.clone());
            }
        }

        for id in stopped {
            games.remove(&id);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_mode() {
        let small = GameSize::Small;
        let mode = match small {
            GameSize::Small => LaunchMode::Embedded,
            GameSize::Large => LaunchMode::External,
        };
        assert!(matches!(mode, LaunchMode::Embedded));
    }

    #[test]
    fn test_game_status() {
        let status = GameStatus {
            game_id: "test".to_string(),
            engine: "Godot".to_string(),
            process_id: 1234,
            is_running: true,
            started_at: 0,
            elapsed_ms: 5000,
            memory_mb: Some(100),
        };

        assert_eq!(status.process_id, 1234);
        assert!(status.is_running);
    }
}
