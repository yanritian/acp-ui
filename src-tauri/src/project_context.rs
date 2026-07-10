// Project Context - Context tracking for development projects
//
// Tracks:
// - Project metadata (language, framework, platform)
// - File structure and dependencies
// - Recent changes and patterns
// - Scene-specific context (web, mini-program, desktop, game)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Project context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project ID
    pub id: String,
    /// Project name
    pub name: String,
    /// Project root path
    pub root_path: PathBuf,
    /// Primary language
    pub language: ProjectLanguage,
    /// Framework
    pub framework: Option<ProjectFramework>,
    /// Scene type
    pub scene: Scene,
    /// Platform
    pub platform: Platform,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// File structure summary
    pub file_summary: FileSummary,
    /// Recent changes
    pub recent_changes: Vec<ChangeRecord>,
    /// Code patterns detected
    pub patterns: Vec<CodePattern>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl ProjectContext {
    pub fn new(id: String, name: String, root_path: PathBuf) -> Self {
        Self {
            id,
            name,
            root_path,
            language: ProjectLanguage::Unknown,
            framework: None,
            scene: Scene::Unknown,
            platform: Platform::Unknown,
            dependencies: Vec::new(),
            file_summary: FileSummary::default(),
            recent_changes: Vec::new(),
            patterns: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Detect project type from file structure
    pub fn detect_from_files(&mut self, files: &[FileInfo]) {
        // Detect language
        self.language = self.detect_language(files);

        // Detect framework
        self.framework = self.detect_framework(files, &self.language);

        // Detect scene
        self.scene = self.detect_scene(files);

        // Detect platform
        self.platform = self.detect_platform(files, &self.scene);

        // Update file summary
        self.file_summary = FileSummary::from_files(files);

        self.updated_at = Utc::now();
    }

    fn detect_language(&self, files: &[FileInfo]) -> ProjectLanguage {
        let extensions: HashMap<&str, ProjectLanguage> = HashMap::from([
            ("ts", ProjectLanguage::TypeScript),
            ("tsx", ProjectLanguage::TypeScript),
            ("js", ProjectLanguage::JavaScript),
            ("vue", ProjectLanguage::Vue),
            ("rs", ProjectLanguage::Rust),
            ("go", ProjectLanguage::Go),
            ("py", ProjectLanguage::Python),
            ("java", ProjectLanguage::Java),
            ("kt", ProjectLanguage::Kotlin),
            ("swift", ProjectLanguage::Swift),
            ("cs", ProjectLanguage::CSharp),
            ("cpp", ProjectLanguage::Cpp),
            ("c", ProjectLanguage::C),
        ]);

        // Count files by extension
        let counts: HashMap<ProjectLanguage, usize> = files
            .iter()
            .filter_map(|f| {
                let path = PathBuf::from(&f.path);
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                extensions.get(ext).cloned()
            })
            .fold(HashMap::new(), |mut acc, lang| {
                *acc.entry(lang).or_insert(0) += 1;
                acc
            });

        // Return most common language
        counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang.clone())
            .unwrap_or(ProjectLanguage::Unknown)
    }

    fn detect_framework(
        &self,
        files: &[FileInfo],
        language: &ProjectLanguage,
    ) -> Option<ProjectFramework> {
        // Check for config files
        for file in files {
            let path = PathBuf::from(&file.path);
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            match name {
                "package.json" => {
                    // Check for Vue/React/Angular
                    if files.iter().any(|f| f.path.ends_with(".vue")) {
                        return Some(ProjectFramework::Vue);
                    }
                    if files.iter().any(|f| f.path.contains("react")) {
                        return Some(ProjectFramework::React);
                    }
                    if files.iter().any(|f| f.path.contains("angular")) {
                        return Some(ProjectFramework::Angular);
                    }
                }
                "Cargo.toml" => {
                    // Check for Tauri
                    if files.iter().any(|f| f.path.contains("tauri.conf.json")) {
                        return Some(ProjectFramework::Tauri);
                    }
                    return Some(ProjectFramework::RustNative);
                }
                "go.mod" => return Some(ProjectFramework::GoNative),
                "requirements.txt" | "pyproject.toml" => {
                    return Some(ProjectFramework::PythonNative)
                }
                _ => {}
            }
        }

        // Language-based defaults
        match language {
            ProjectLanguage::Vue => Some(ProjectFramework::Vue),
            ProjectLanguage::TypeScript => Some(ProjectFramework::React),
            ProjectLanguage::Rust => Some(ProjectFramework::RustNative),
            ProjectLanguage::Go => Some(ProjectFramework::GoNative),
            ProjectLanguage::Python => Some(ProjectFramework::PythonNative),
            _ => None,
        }
    }

    fn detect_scene(&self, files: &[FileInfo]) -> Scene {
        // Check for Tauri indicators
        if files
            .iter()
            .any(|f| f.path.contains("tauri.conf.json") || f.path.contains("src-tauri"))
        {
            return Scene::Desktop;
        }

        // Check for Electron indicators
        if files
            .iter()
            .any(|f| f.path.contains("electron") || f.path.contains("electron-builder"))
        {
            return Scene::Desktop;
        }

        // Check for Flutter indicators
        if files
            .iter()
            .any(|f| f.path.contains("pubspec.yaml") || f.path.contains("flutter"))
        {
            return Scene::Desktop;
        }

        // Check for mini-program indicators
        if files
            .iter()
            .any(|f| f.path.contains("miniprogram") || f.path.contains("app.json"))
        {
            return Scene::MiniProgram;
        }

        // Check for game indicators
        if files.iter().any(|f| {
            f.path.contains("unity") || f.path.contains("godot") || f.path.contains("game")
        }) {
            return Scene::Game;
        }

        // Check for web indicators
        if files.iter().any(|f| {
            f.path.ends_with(".html")
                || f.path.ends_with(".vue")
                || f.path.contains("public")
                || f.path.contains("www")
        }) {
            return Scene::Web;
        }

        // Check for common web frameworks via package.json
        let has_web_framework = files.iter().any(|f| {
            f.path.contains("package.json")
                && (f.path.contains("vite")
                    || f.path.contains("next")
                    || f.path.contains("nuxt")
                    || f.path.contains("svelte"))
        });

        if has_web_framework {
            return Scene::Web;
        }

        Scene::Unknown
    }

    fn detect_platform(&self, files: &[FileInfo], scene: &Scene) -> Platform {
        match scene {
            Scene::Web => Platform::Web,
            Scene::MiniProgram => {
                // Check which platform
                if files.iter().any(|f| f.path.contains("wechat")) {
                    return Platform::MiniProgramWechat;
                }
                if files.iter().any(|f| f.path.contains("alipay")) {
                    return Platform::MiniProgramAlipay;
                }
                Platform::MiniProgramUnknown
            }
            Scene::Desktop => {
                if files.iter().any(|f| f.path.contains("windows")) {
                    return Platform::DesktopWindows;
                }
                if files.iter().any(|f| f.path.contains("macos")) {
                    return Platform::DesktopMacos;
                }
                Platform::DesktopCross
            }
            Scene::Game => {
                if files.iter().any(|f| f.path.contains("unity")) {
                    return Platform::GameUnity;
                }
                Platform::GameUnknown
            }
            Scene::Unknown => Platform::Unknown,
        }
    }

    /// Add a change record
    pub fn record_change(&mut self, change: ChangeRecord) {
        self.recent_changes.push(change);

        // Keep only last 100 changes
        if self.recent_changes.len() > 100 {
            self.recent_changes.remove(0);
        }

        self.updated_at = Utc::now();
    }

    /// Add a detected pattern
    pub fn add_pattern(&mut self, pattern: CodePattern) {
        if !self.patterns.iter().any(|p| p.name == pattern.name) {
            self.patterns.push(pattern);
            self.updated_at = Utc::now();
        }
    }

    /// Get context summary for routing
    pub fn get_summary(&self) -> ContextSummary {
        ContextSummary {
            language: self.language.clone(),
            framework: self.framework.clone(),
            scene: self.scene.clone(),
            platform: self.platform.clone(),
            file_count: self.file_summary.total_files,
            recent_change_count: self.recent_changes.len(),
            pattern_count: self.patterns.len(),
        }
    }
}

/// Project language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProjectLanguage {
    TypeScript,
    JavaScript,
    Vue,
    Rust,
    Go,
    Python,
    Java,
    Kotlin,
    Swift,
    CSharp,
    Cpp,
    C,
    Unknown,
}

/// Project framework
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectFramework {
    Vue,
    React,
    Angular,
    Tauri,
    Electron,
    Flutter,
    RustNative,
    GoNative,
    PythonNative,
    Unity,
    Godot,
    Unknown,
}

/// Scene type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Scene {
    Web,
    MiniProgram,
    Desktop,
    Game,
    Unknown,
}

/// Platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Web,
    MiniProgramWechat,
    MiniProgramAlipay,
    MiniProgramUnknown,
    DesktopWindows,
    DesktopMacos,
    DesktopLinux,
    DesktopCross,
    GameUnity,
    GameGodot,
    GameUnknown,
    Unknown,
}

/// Dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    Production,
    Development,
    Peer,
    Optional,
}

/// File summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSummary {
    pub total_files: usize,
    pub source_files: usize,
    pub test_files: usize,
    pub config_files: usize,
    pub doc_files: usize,
    pub total_lines: usize,
    pub largest_file: Option<String>,
    pub largest_file_lines: usize,
}

impl FileSummary {
    pub fn from_files(files: &[FileInfo]) -> Self {
        let total_files = files.len();
        let source_files = files.iter().filter(|f| f.is_source).count();
        let test_files = files.iter().filter(|f| f.is_test).count();
        let config_files = files.iter().filter(|f| f.is_config).count();
        let doc_files = files.iter().filter(|f| f.is_doc).count();
        let total_lines = files.iter().map(|f| f.line_count).sum();

        let largest_file = files
            .iter()
            .max_by_key(|f| f.line_count)
            .map(|f| f.path.clone());
        let largest_file_lines = files
            .iter()
            .max_by_key(|f| f.line_count)
            .map(|f| f.line_count)
            .unwrap_or(0);

        Self {
            total_files,
            source_files,
            test_files,
            config_files,
            doc_files,
            total_lines,
            largest_file,
            largest_file_lines,
        }
    }
}

/// File info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub line_count: usize,
    pub is_source: bool,
    pub is_test: bool,
    pub is_config: bool,
    pub is_doc: bool,
}

/// Change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub id: String,
    pub file_path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub timestamp: DateTime<Utc>,
    pub agent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Code pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub files: Vec<String>,
    pub frequency: usize,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    Component,  // Vue/React component
    Service,    // Service layer
    Repository, // Data access layer
    Controller, // API handler
    Model,      // Data model
    Utility,    // Helper functions
    Test,       // Test pattern
    Config,     // Configuration
    Hook,       // React/Vue hook
    Middleware, // Middleware
}

/// Context summary for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub language: ProjectLanguage,
    pub framework: Option<ProjectFramework>,
    pub scene: Scene,
    pub platform: Platform,
    pub file_count: usize,
    pub recent_change_count: usize,
    pub pattern_count: usize,
}

/// Project context manager
pub struct ProjectContextManager {
    contexts: HashMap<String, ProjectContext>,
    current_project: Option<String>,
}

impl ProjectContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            current_project: None,
        }
    }

    /// Create a new project context
    pub fn create_context(
        &mut self,
        id: String,
        name: String,
        root_path: PathBuf,
    ) -> ProjectContext {
        let context = ProjectContext::new(id.clone(), name, root_path);
        self.contexts.insert(id, context.clone());
        self.current_project = Some(context.id.clone());
        context
    }

    /// Get current project context
    pub fn get_current(&self) -> Option<&ProjectContext> {
        self.current_project
            .as_ref()
            .and_then(|id| self.contexts.get(id))
    }

    /// Get context by ID
    pub fn get_context(&self, id: &str) -> Option<&ProjectContext> {
        self.contexts.get(id)
    }

    /// Update context
    pub fn update_context(&mut self, id: &str, files: &[FileInfo]) -> Option<ProjectContext> {
        let context = self.contexts.get_mut(id)?;
        context.detect_from_files(files);
        Some(context.clone())
    }

    /// Set current project
    pub fn set_current(&mut self, id: &str) -> bool {
        if self.contexts.contains_key(id) {
            self.current_project = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// List all contexts
    pub fn list_contexts(&self) -> Vec<ProjectContext> {
        self.contexts.values().cloned().collect()
    }

    /// Remove context
    pub fn remove_context(&mut self, id: &str) -> Option<ProjectContext> {
        let removed = self.contexts.remove(id);
        if self.current_project.as_ref() == Some(&id.to_string()) {
            self.current_project = None;
        }
        removed
    }
}

impl Default for ProjectContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_context_new() {
        let ctx = ProjectContext::new(
            "proj-1".to_string(),
            "MyProject".to_string(),
            PathBuf::from("/tmp/myproject"),
        );
        assert_eq!(ctx.id, "proj-1");
        assert_eq!(ctx.name, "MyProject");
        assert_eq!(ctx.language, ProjectLanguage::Unknown);
    }

    #[test]
    fn test_detect_language_typescript() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        let files = vec![
            FileInfo {
                path: "src/main.ts".to_string(),
                line_count: 100,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "src/utils.ts".to_string(),
                line_count: 50,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "src/index.ts".to_string(),
                line_count: 30,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
        ];

        ctx.detect_from_files(&files);
        assert_eq!(ctx.language, ProjectLanguage::TypeScript);
    }

    #[test]
    fn test_detect_language_vue() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        let files = vec![
            FileInfo {
                path: "src/App.vue".to_string(),
                line_count: 100,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "src/components/Button.vue".to_string(),
                line_count: 50,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
        ];

        ctx.detect_from_files(&files);
        assert_eq!(ctx.language, ProjectLanguage::Vue);
        assert_eq!(ctx.framework, Some(ProjectFramework::Vue));
    }

    #[test]
    fn test_detect_framework_tauri() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        let files = vec![
            FileInfo {
                path: "src-tauri/Cargo.toml".to_string(),
                line_count: 30,
                is_source: false,
                is_test: false,
                is_config: true,
                is_doc: false,
            },
            FileInfo {
                path: "src-tauri/tauri.conf.json".to_string(),
                line_count: 50,
                is_source: false,
                is_test: false,
                is_config: true,
                is_doc: false,
            },
            FileInfo {
                path: "src/main.rs".to_string(),
                line_count: 100,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
        ];

        ctx.detect_from_files(&files);
        assert_eq!(ctx.language, ProjectLanguage::Rust);
        assert_eq!(ctx.framework, Some(ProjectFramework::Tauri));
        assert_eq!(ctx.scene, Scene::Desktop);
    }

    #[test]
    fn test_detect_scene_mini_program() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        let files = vec![
            FileInfo {
                path: "miniprogram/pages/index/index.js".to_string(),
                line_count: 100,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "miniprogram/app.json".to_string(),
                line_count: 50,
                is_source: false,
                is_test: false,
                is_config: true,
                is_doc: false,
            },
        ];

        ctx.detect_from_files(&files);
        assert_eq!(ctx.scene, Scene::MiniProgram);
    }

    #[test]
    fn test_file_summary() {
        let files = vec![
            FileInfo {
                path: "src/a.ts".to_string(),
                line_count: 100,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "src/b.ts".to_string(),
                line_count: 200,
                is_source: true,
                is_test: false,
                is_config: false,
                is_doc: false,
            },
            FileInfo {
                path: "test/a.test.ts".to_string(),
                line_count: 50,
                is_source: false,
                is_test: true,
                is_config: false,
                is_doc: false,
            },
        ];

        let summary = FileSummary::from_files(&files);
        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.source_files, 2);
        assert_eq!(summary.test_files, 1);
        assert_eq!(summary.total_lines, 350);
        assert_eq!(summary.largest_file, Some("src/b.ts".to_string()));
        assert_eq!(summary.largest_file_lines, 200);
    }

    #[test]
    fn test_record_change() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        ctx.record_change(ChangeRecord {
            id: "change-1".to_string(),
            file_path: "src/main.ts".to_string(),
            change_type: ChangeType::Modified,
            lines_added: 10,
            lines_removed: 5,
            timestamp: Utc::now(),
            agent_id: Some("claude-code".to_string()),
            description: Some("Added new function".to_string()),
        });

        assert_eq!(ctx.recent_changes.len(), 1);
    }

    #[test]
    fn test_context_manager() {
        let mut manager = ProjectContextManager::new();

        let ctx = manager.create_context(
            "proj-1".to_string(),
            "MyProject".to_string(),
            PathBuf::from("/tmp/myproject"),
        );
        assert_eq!(ctx.id, "proj-1");
        assert!(manager.get_current().is_some());

        let contexts = manager.list_contexts();
        assert_eq!(contexts.len(), 1);
    }

    #[test]
    fn test_get_summary() {
        let mut ctx = ProjectContext::new(
            "proj-1".to_string(),
            "Test".to_string(),
            PathBuf::from("/tmp"),
        );

        let files = vec![FileInfo {
            path: "src/App.vue".to_string(),
            line_count: 100,
            is_source: true,
            is_test: false,
            is_config: false,
            is_doc: false,
        }];

        ctx.detect_from_files(&files);
        let summary = ctx.get_summary();

        assert_eq!(summary.language, ProjectLanguage::Vue);
        assert_eq!(summary.framework, Some(ProjectFramework::Vue));
        assert_eq!(summary.scene, Scene::Web);
        assert_eq!(summary.file_count, 1);
    }
}
