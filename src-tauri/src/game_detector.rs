// Game Detector - Detect game engine and type
// Phase 1 Day 3: Godot Integration

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Game engine type
#[derive(Debug, Clone, PartialEq)]
pub enum GameEngine {
    Godot,
    Unity,
    Unknown,
}

/// Game size category
#[derive(Debug, Clone, PartialEq)]
pub enum GameSize {
    Small, // < 50MB, can use WebView
    Large, // > 50MB, need external process
}

/// Game information
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub engine: GameEngine,
    pub size: GameSize,
    pub scenes: Vec<String>,
    pub project_name: String,
    pub version: Option<String>,
    pub path: PathBuf,
}

/// Game detector
pub struct GameDetector;

impl GameDetector {
    /// Detect game in directory
    pub fn detect(cwd: &Path) -> Result<GameInfo, GameDetectorError> {
        // Try Godot first
        if Self::is_godot_project(cwd) {
            return Self::detect_godot(cwd);
        }

        // Try Unity
        if Self::is_unity_project(cwd) {
            return Self::detect_unity(cwd);
        }

        Err(GameDetectorError::UnknownGameEngine)
    }

    /// Check if directory is a Godot project
    fn is_godot_project(cwd: &Path) -> bool {
        cwd.join("project.godot").exists()
    }

    /// Check if directory is a Unity project
    fn is_unity_project(cwd: &Path) -> bool {
        cwd.join("Assets").exists() && cwd.join("ProjectSettings").exists()
    }

    /// Detect Godot project details
    fn detect_godot(cwd: &Path) -> Result<GameInfo, GameDetectorError> {
        let project_file = cwd.join("project.godot");
        let content = fs::read_to_string(&project_file)
            .map_err(|e| GameDetectorError::ReadError(e.to_string()))?;

        // Parse project name
        let project_name =
            Self::parse_godot_project_name(&content).unwrap_or_else(|| "Godot Project".to_string());

        // Parse version
        let version = Self::parse_godot_version(&content);

        // Get scenes
        let scenes = Self::find_godot_scenes(cwd);

        // Calculate size
        let size_bytes = Self::calculate_directory_size(cwd);
        let size = if size_bytes < 50 * 1024 * 1024 {
            GameSize::Small
        } else {
            GameSize::Large
        };

        Ok(GameInfo {
            engine: GameEngine::Godot,
            size,
            scenes,
            project_name,
            version,
            path: cwd.to_path_buf(),
        })
    }

    /// Detect Unity project details
    fn detect_unity(cwd: &Path) -> Result<GameInfo, GameDetectorError> {
        // Parse project name from ProjectSettings/ProjectVersion.txt
        let version_file = cwd.join("ProjectSettings/ProjectVersion.txt");
        let version = if version_file.exists() {
            let content = fs::read_to_string(&version_file)
                .map_err(|e| GameDetectorError::ReadError(e.to_string()))?;
            Self::parse_unity_version(&content)
        } else {
            None
        };

        // Get scenes from EditorBuildSettings.asset
        let scenes = Self::find_unity_scenes(cwd);

        // Calculate size
        let size_bytes = Self::calculate_directory_size(cwd);
        let size = if size_bytes < 50 * 1024 * 1024 {
            GameSize::Small
        } else {
            GameSize::Large
        };

        Ok(GameInfo {
            engine: GameEngine::Unity,
            size,
            scenes,
            project_name: "Unity Project".to_string(),
            version,
            path: cwd.to_path_buf(),
        })
    }

    /// Parse Godot project name from project.godot
    fn parse_godot_project_name(content: &str) -> Option<String> {
        let re = Regex::new(r#"config/name="([^"]+)""#).ok()?;
        re.captures(content).map(|caps| caps[1].to_string())
    }

    /// Parse Godot version from project.godot
    fn parse_godot_version(content: &str) -> Option<String> {
        let re = Regex::new(r#"PackedStringArray\("(\d+\.\d+)"#).ok()?;
        re.captures(content).map(|caps| caps[1].to_string())
    }

    /// Parse Unity version from ProjectVersion.txt
    fn parse_unity_version(content: &str) -> Option<String> {
        let re = Regex::new(r"m_EditorVersion:\s*(\S+)").ok()?;
        re.captures(content).map(|caps| caps[1].to_string())
    }

    /// Find Godot scenes (.tscn files)
    fn find_godot_scenes(cwd: &Path) -> Vec<String> {
        let mut scenes = Vec::new();
        Self::find_scenes_recursive(cwd, cwd, &mut scenes);
        scenes
    }

    fn find_scenes_recursive(base: &Path, dir: &Path, scenes: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::find_scenes_recursive(base, &path, scenes);
                } else if path.extension().and_then(|s| s.to_str()) == Some("tscn") {
                    if let Ok(relative) = path.strip_prefix(base) {
                        scenes.push(relative.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    /// Find Unity scenes from EditorBuildSettings.asset
    fn find_unity_scenes(cwd: &Path) -> Vec<String> {
        let build_settings = cwd.join("ProjectSettings/EditorBuildSettings.asset");
        if !build_settings.exists() {
            return Vec::new();
        }

        let content = match fs::read_to_string(&build_settings) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut scenes = Vec::new();
        let re = match Regex::new(r"path:\s*(Assets/Scenes/[^\s]+\.unity)") {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        for caps in re.captures_iter(&content) {
            scenes.push(caps[1].to_string());
        }

        scenes
    }

    /// Calculate total directory size in bytes
    fn calculate_directory_size(dir: &Path) -> u64 {
        let mut size = 0;

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip Library, Temp, obj directories (Unity cache)
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name == "Library" || name == "Temp" || name == "obj" {
                            continue;
                        }
                    }
                    size += Self::calculate_directory_size(&path);
                } else if let Ok(metadata) = entry.metadata() {
                    size += metadata.len();
                }
            }
        }

        size
    }
}

/// Game detector errors
#[derive(Debug)]
pub enum GameDetectorError {
    UnknownGameEngine,
    ReadError(String),
    ParseError(String),
}

impl std::fmt::Display for GameDetectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDetectorError::UnknownGameEngine => write!(f, "Unknown game engine"),
            GameDetectorError::ReadError(e) => write!(f, "Read error: {}", e),
            GameDetectorError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for GameDetectorError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_godot_project() {
        let dir = tempdir().unwrap();
        let project_file = dir.path().join("project.godot");
        fs::write(
            &project_file,
            r#"[application]
config/name="Test Game"
config/features=PackedStringArray("4.2", "GL Compatibility")
"#,
        )
        .unwrap();

        let info = GameDetector::detect(dir.path()).unwrap();
        assert_eq!(info.engine, GameEngine::Godot);
        assert_eq!(info.project_name, "Test Game");
        assert_eq!(info.version, Some("4.2".to_string()));
    }

    #[test]
    fn test_detect_unity_project() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Assets")).unwrap();
        fs::create_dir_all(dir.path().join("ProjectSettings")).unwrap();

        let version_file = dir.path().join("ProjectSettings/ProjectVersion.txt");
        fs::write(&version_file, "m_EditorVersion: 2021.3.0f1\n").unwrap();

        let info = GameDetector::detect(dir.path()).unwrap();
        assert_eq!(info.engine, GameEngine::Unity);
        assert_eq!(info.version, Some("2021.3.0f1".to_string()));
    }
}
