// Godot Domain Pack - Project Analyzer
// Analyzes Godot project structure and extracts metadata

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotProjectInfo {
    pub project_path: PathBuf,
    pub project_file: PathBuf,
    pub project_name: String,
    pub godot_version: Option<String>,
    pub main_scene: Option<String>,
    pub scripts: Vec<PathBuf>,
    pub scenes: Vec<PathBuf>,
    pub assets: Vec<PathBuf>,
    pub entry_scene: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GodotProjectAnalyzer;

impl GodotProjectAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Detect if a directory is a Godot project
    pub fn detect_project(path: &Path) -> bool {
        let project_file = path.join("project.godot");
        project_file.exists() && project_file.is_file()
    }

    /// Analyze a Godot project and extract metadata
    pub fn analyze_project(path: &Path) -> Result<GodotProjectInfo, GodotAnalyzerError> {
        if !Self::detect_project(path) {
            return Err(GodotAnalyzerError::NotAGodotProject(path.to_path_buf()));
        }

        let project_file = path.join("project.godot");
        let project_content = fs::read_to_string(&project_file)
            .map_err(|e| GodotAnalyzerError::IoError(e.to_string()))?;

        // Parse project.godot (simplified - real implementation would use a proper INI parser)
        let project_name = Self::extract_project_name(&project_content)
            .unwrap_or_else(|| "Unknown".to_string());
        let godot_version = Self::extract_godot_version(&project_content);
        let main_scene = Self::extract_main_scene(&project_content);

        // Scan for scripts, scenes, and assets
        let scripts = Self::find_files_with_extension(path, &["gd", "cs"])?;
        let scenes = Self::find_files_with_extension(path, &["tscn", "scn"])?;
        let assets = Self::find_asset_files(path)?;

        Ok(GodotProjectInfo {
            project_path: path.to_path_buf(),
            project_file,
            project_name,
            godot_version,
            main_scene: main_scene.clone(),
            scripts,
            scenes,
            assets,
            entry_scene: main_scene,
        })
    }

    /// Find player controller candidates
    pub fn find_player_controllers(project: &GodotProjectInfo) -> Vec<PathBuf> {
        let keywords = ["player", "character", "controller", "actor"];

        project.scripts.iter()
            .filter(|path| {
                if let Some(filename) = path.file_stem() {
                    let name = filename.to_string_lossy().to_lowercase();
                    keywords.iter().any(|kw| name.contains(kw))
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    // ============================================================================
    // Helper Methods
    // ============================================================================

    fn extract_project_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("config/name=") {
                return Some(line.trim_start_matches("config/name=").trim_matches('"').to_string());
            }
        }
        None
    }

    fn extract_godot_version(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("config/features=") {
                // Extract version from features array
                return Some(line.to_string());
            }
        }
        None
    }

    fn extract_main_scene(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("run/main_scene=") {
                return Some(line.trim_start_matches("run/main_scene=").trim_matches('"').to_string());
            }
        }
        None
    }

    fn find_files_with_extension(base_path: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, GodotAnalyzerError> {
        let mut files = Vec::new();

        fn walk_dir(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<(), GodotAnalyzerError> {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip common non-project directories
                        let dir_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        if ![".godot", ".import", "build", "dist"].contains(&dir_name) {
                            walk_dir(&path, extensions, files)?;
                        }
                    } else if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if extensions.iter().any(|e| *e == ext_str) {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        walk_dir(base_path, extensions, &mut files)?;
        Ok(files)
    }

    fn find_asset_files(base_path: &Path) -> Result<Vec<PathBuf>, GodotAnalyzerError> {
        let asset_extensions = ["png", "jpg", "jpeg", "svg", "wav", "mp3", "ogg", "tres", "res"];
        Self::find_files_with_extension(base_path, &asset_extensions)
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum GodotAnalyzerError {
    NotAGodotProject(PathBuf),
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for GodotAnalyzerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GodotAnalyzerError::NotAGodotProject(path) => {
                write!(f, "Not a Godot project: {}", path.display())
            }
            GodotAnalyzerError::IoError(msg) => write!(f, "IO error: {}", msg),
            GodotAnalyzerError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for GodotAnalyzerError {}
