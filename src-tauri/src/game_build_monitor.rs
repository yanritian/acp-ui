// Game Build Monitor
// Phase 2 Day 7: Real-time build progress monitoring

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// ===== Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildProgress {
    pub id: String,
    pub engine: String,
    pub stage: BuildStage,
    pub progress: f32,  // 0.0 to 1.0
    pub message: String,
    pub started_at: u64,
    pub elapsed_ms: u64,
    pub estimated_remaining_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuildStage {
    Detecting,
    Preparing,
    Compiling,
    Linking,
    Finalizing,
    Complete,
    Failed,
}

impl BuildStage {
    pub fn progress_range(&self) -> (f32, f32) {
        match self {
            BuildStage::Detecting => (0.0, 0.1),
            BuildStage::Preparing => (0.1, 0.2),
            BuildStage::Compiling => (0.2, 0.7),
            BuildStage::Linking => (0.7, 0.9),
            BuildStage::Finalizing => (0.9, 1.0),
            BuildStage::Complete => (1.0, 1.0),
            BuildStage::Failed => (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLog {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

// ===== Build Monitor =====

pub struct BuildMonitor {
    builds: Arc<Mutex<HashMap<String, BuildState>>>,
}

struct BuildState {
    progress: BuildProgress,
    logs: Vec<BuildLog>,
    start_time: Instant,
}

impl BuildMonitor {
    pub fn new() -> Self {
        Self {
            builds: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start monitoring a new build
    pub fn start_build(&self, build_id: String, engine: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let progress = BuildProgress {
            id: build_id.clone(),
            engine,
            stage: BuildStage::Detecting,
            progress: 0.0,
            message: "Starting build...".to_string(),
            started_at: now,
            elapsed_ms: 0,
            estimated_remaining_ms: None,
        };

        let state = BuildState {
            progress,
            logs: Vec::new(),
            start_time: Instant::now(),
        };

        let mut builds = self.builds.lock().unwrap();
        builds.insert(build_id, state);
    }

    /// Update build progress
    pub fn update_progress(
        &self,
        build_id: &str,
        stage: BuildStage,
        message: String,
    ) -> Option<BuildProgress> {
        let mut builds = self.builds.lock().unwrap();

        if let Some(state) = builds.get_mut(build_id) {
            let elapsed = state.start_time.elapsed().as_millis() as u64;

            // Calculate progress based on stage
            let (start, end) = stage.progress_range();
            let stage_progress = match stage {
                BuildStage::Detecting => 0.5,
                BuildStage::Preparing => 0.5,
                BuildStage::Compiling => 0.5, // Middle of compilation stage
                BuildStage::Linking => 0.7,
                BuildStage::Finalizing => 0.9,
                BuildStage::Complete => 1.0,
                BuildStage::Failed => 0.0,
            };

            let progress_value = start + (end - start) * stage_progress;

            state.progress.stage = stage;
            state.progress.progress = progress_value;
            state.progress.message = message.clone();
            state.progress.elapsed_ms = elapsed;

            // Estimate remaining time
            if progress_value > 0.0 && progress_value < 1.0 {
                let total_estimated = (elapsed as f64 / progress_value as f64) as u64;
                state.progress.estimated_remaining_ms = Some(total_estimated.saturating_sub(elapsed));
            }

            // Add log entry
            state.logs.push(BuildLog {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                level: if stage == BuildStage::Failed {
                    LogLevel::Error
                } else {
                    LogLevel::Info
                },
                message,
            });

            Some(state.progress.clone())
        } else {
            None
        }
    }

    /// Add a log entry
    pub fn add_log(&self, build_id: &str, level: LogLevel, message: String) {
        let mut builds = self.builds.lock().unwrap();
        if let Some(state) = builds.get_mut(build_id) {
            state.logs.push(BuildLog {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                level,
                message,
            });
        }
    }

    /// Get current progress
    pub fn get_progress(&self, build_id: &str) -> Option<BuildProgress> {
        let builds = self.builds.lock().unwrap();
        builds.get(build_id).map(|s| s.progress.clone())
    }

    /// Get build logs
    pub fn get_logs(&self, build_id: &str) -> Option<Vec<BuildLog>> {
        let builds = self.builds.lock().unwrap();
        builds.get(build_id).map(|s| s.logs.clone())
    }

    /// Complete build
    pub fn complete_build(&self, build_id: &str, success: bool) -> Option<BuildProgress> {
        let stage = if success {
            BuildStage::Complete
        } else {
            BuildStage::Failed
        };

        self.update_progress(
            build_id,
            stage,
            if success {
                "Build completed successfully".to_string()
            } else {
                "Build failed".to_string()
            },
        )
    }

    /// Remove build from monitoring
    pub fn remove_build(&self, build_id: &str) {
        let mut builds = self.builds.lock().unwrap();
        builds.remove(build_id);
    }

    /// Get all active builds
    pub fn get_all_builds(&self) -> Vec<BuildProgress> {
        let builds = self.builds.lock().unwrap();
        builds.values().map(|s| s.progress.clone()).collect()
    }
}

// ===== Helper Functions =====

/// Generate a unique build ID
pub fn generate_build_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("build-{}", timestamp)
}

/// Parse Godot build output to extract progress
pub fn parse_godot_output(line: &str) -> Option<(BuildStage, String)> {
    let line_lower = line.to_lowercase();

    if line_lower.contains("detecting") || line_lower.contains("scanning") {
        Some((BuildStage::Detecting, line.to_string()))
    } else if line_lower.contains("preparing") || line_lower.contains("loading") {
        Some((BuildStage::Preparing, line.to_string()))
    } else if line_lower.contains("compiling") || line_lower.contains("building") {
        Some((BuildStage::Compiling, line.to_string()))
    } else if line_lower.contains("linking") {
        Some((BuildStage::Linking, line.to_string()))
    } else if line_lower.contains("finalizing") || line_lower.contains("packaging") {
        Some((BuildStage::Finalizing, line.to_string()))
    } else if line_lower.contains("error") || line_lower.contains("failed") {
        Some((BuildStage::Failed, line.to_string()))
    } else {
        None
    }
}

/// Parse Unity build output to extract progress
pub fn parse_unity_output(line: &str) -> Option<(BuildStage, String)> {
    let line_lower = line.to_lowercase();

    if line_lower.contains("compiling scripts") {
        Some((BuildStage::Compiling, line.to_string()))
    } else if line_lower.contains("building scenes") {
        Some((BuildStage::Compiling, line.to_string()))
    } else if line_lower.contains("linking") {
        Some((BuildStage::Linking, line.to_string()))
    } else if line_lower.contains("packaging") || line_lower.contains("creating") {
        Some((BuildStage::Finalizing, line.to_string()))
    } else if line_lower.contains("error") || line_lower.contains("failed") {
        Some((BuildStage::Failed, line.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_monitor_lifecycle() {
        let monitor = BuildMonitor::new();
        let build_id = generate_build_id();

        // Start build
        monitor.start_build(build_id.clone(), "Godot".to_string());

        // Check initial progress
        let progress = monitor.get_progress(&build_id).unwrap();
        assert_eq!(progress.stage, BuildStage::Detecting);
        assert_eq!(progress.progress, 0.0);

        // Update progress
        monitor.update_progress(
            &build_id,
            BuildStage::Compiling,
            "Compiling scripts...".to_string(),
        );

        let progress = monitor.get_progress(&build_id).unwrap();
        assert_eq!(progress.stage, BuildStage::Compiling);
        assert!(progress.progress > 0.2);
        assert!(progress.progress < 0.7);

        // Complete build
        monitor.complete_build(&build_id, true);
        let progress = monitor.get_progress(&build_id).unwrap();
        assert_eq!(progress.stage, BuildStage::Complete);
        assert_eq!(progress.progress, 1.0);
    }

    #[test]
    fn test_parse_godot_output() {
        let result = parse_godot_output("Compiling scripts...");
        assert!(result.is_some());
        let (stage, _) = result.unwrap();
        assert_eq!(stage, BuildStage::Compiling);
    }

    #[test]
    fn test_parse_unity_output() {
        let result = parse_unity_output("Linking assemblies...");
        assert!(result.is_some());
        let (stage, _) = result.unwrap();
        assert_eq!(stage, BuildStage::Linking);
    }
}
