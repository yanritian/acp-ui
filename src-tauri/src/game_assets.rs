// Game Asset Detection - Game asset type detection and analysis
//
// Phase 3 Week 4: Game Asset Generation
// Detects and categorizes game assets for optimization suggestions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Game asset type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAssetType {
    /// 2D sprite/image
    Sprite2D,
    /// Texture ( diffuse, normal, etc.)
    Texture,
    /// 3D model
    Model3D,
    /// Animation data
    Animation,
    /// Audio file
    Audio,
    /// Shader program
    Shader,
    /// Level/scene file
    Level,
    /// Font
    Font,
    /// Video
    Video,
    /// Script/behavior
    Script,
    /// Unknown
    Unknown,
}

impl GameAssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameAssetType::Sprite2D => "sprite2d",
            GameAssetType::Texture => "texture",
            GameAssetType::Model3D => "model3d",
            GameAssetType::Animation => "animation",
            GameAssetType::Audio => "audio",
            GameAssetType::Shader => "shader",
            GameAssetType::Level => "level",
            GameAssetType::Font => "font",
            GameAssetType::Video => "video",
            GameAssetType::Script => "script",
            GameAssetType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sprite" | "sprite2d" | "sprite_2d" => GameAssetType::Sprite2D,
            "texture" | "tex" => GameAssetType::Texture,
            "model" | "model3d" | "3dmodel" => GameAssetType::Model3D,
            "animation" | "anim" => GameAssetType::Animation,
            "audio" | "sound" | "music" => GameAssetType::Audio,
            "shader" | "glsl" | "hlsl" => GameAssetType::Shader,
            "level" | "scene" | "map" => GameAssetType::Level,
            "font" => GameAssetType::Font,
            "video" | "movie" => GameAssetType::Video,
            "script" | "behavior" => GameAssetType::Script,
            _ => GameAssetType::Unknown,
        }
    }
}

/// Game asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAsset {
    /// Asset name
    pub name: String,
    /// Asset type
    pub asset_type: GameAssetType,
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
    /// Asset format (extension)
    pub format: String,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<AssetOptimization>,
}

/// Asset optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetOptimization {
    /// Suggestion type
    pub suggestion_type: OptimizationType,
    /// Priority (1-5, 5 = critical)
    pub priority: u8,
    /// Description
    pub description: String,
    /// Estimated savings (bytes or percentage)
    pub estimated_savings: Option<EstimatedSaving>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Compress image/texture
    Compress,
    /// Resize/downscale
    Resize,
    /// Convert format
    ConvertFormat,
    /// Optimize mesh (reduce vertices)
    OptimizeMesh,
    /// Optimize audio (reduce bitrate)
    OptimizeAudio,
    /// Split into smaller parts
    Split,
    /// Remove unused data
    RemoveUnused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstimatedSaving {
    Bytes(u64),
    Percentage(f32),
}

/// Game Asset Detector
pub struct GameAssetDetector {
    /// Max file size threshold for optimization warnings (5MB)
    large_file_threshold: u64,
    /// Texture format preferences per platform
    texture_formats: HashMap<String, Vec<String>>,
}

use std::collections::HashMap;

impl GameAssetDetector {
    pub fn new() -> Self {
        Self {
            large_file_threshold: 5 * 1024 * 1024, // 5MB
            texture_formats: Self::default_texture_formats(),
        }
    }

    fn default_texture_formats() -> HashMap<String, Vec<String>> {
        HashMap::from([
            ("unity".to_string(), vec!["png".to_string(), "jpg".to_string(), "tga".to_string(), "dds".to_string()]),
            ("godot".to_string(), vec!["png".to_string(), "jpg".to_string(), "webp".to_string()]),
        ])
    }

    /// Detect assets in a game project directory
    pub fn detect_assets(&self, project_path: &PathBuf, framework: &str) -> Vec<GameAsset> {
        let mut assets = Vec::new();

        // Common asset directories
        let asset_dirs = if framework == "unity" {
            vec!["Assets"]
        } else if framework == "godot" {
            vec!["assets", "scenes", "scripts"]
        } else {
            vec!["assets", "resources", "data"]
        };

        for dir_name in asset_dirs {
            let dir = project_path.join(dir_name);
            if dir.exists() {
                self.scan_directory(&dir, &mut assets);
            }
        }

        assets
    }

    fn scan_directory(&self, dir: &PathBuf, assets: &mut Vec<GameAsset>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path, assets);
                } else if path.is_file() {
                    if let Some(asset) = self.detect_file_asset(&path) {
                        assets.push(asset);
                    }
                }
            }
        }
    }

    fn detect_file_asset(&self, path: &PathBuf) -> Option<GameAsset> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        let name = path.file_name()?.to_string_lossy().to_string();

        let asset_type = self.get_asset_type_from_extension(&ext);
        if asset_type == GameAssetType::Unknown {
            return None;
        }

        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let optimization_suggestions = self.get_optimization_suggestions(asset_type, size, &ext);

        Some(GameAsset {
            name,
            asset_type,
            path: path.clone(),
            size_bytes: size,
            format: ext,
            optimization_suggestions,
        })
    }

    fn get_asset_type_from_extension(&self, ext: &str) -> GameAssetType {
        match ext {
            // 2D Sprites/Textures
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "gif" | "webp" => GameAssetType::Sprite2D,
            // 3D Models
            "fbx" | "obj" | "gltf" | "glb" | "dae" | "blend" | "3ds" | "max" => GameAssetType::Model3D,
            // Unity/Godot specific
            "prefab" | "mat" | "mesh" => GameAssetType::Model3D,
            // Animations
            "anim" | "clip" | "fbx" => GameAssetType::Animation,
            // Audio
            "wav" | "mp3" | "ogg" | "aac" | "flac" | "m4a" => GameAssetType::Audio,
            // Shaders
            "shader" | "glsl" | "hlsl" | "cg" | "frag" | "vert" => GameAssetType::Shader,
            // Levels/Scenes
            "unity" | "tscn" | "scn" | "level" | "map" => GameAssetType::Level,
            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "fnt" => GameAssetType::Font,
            // Video
            "mp4" | "mov" | "avi" | "webm" | "mkv" => GameAssetType::Video,
            // Scripts
            "cs" | "gd" | "lua" | "js" => GameAssetType::Script,
            _ => GameAssetType::Unknown,
        }
    }

    pub fn get_optimization_suggestions(&self, asset_type: GameAssetType, size: u64, format: &str) -> Vec<AssetOptimization> {
        let mut suggestions = Vec::new();

        // Large file warning
        if size > self.large_file_threshold {
            suggestions.push(AssetOptimization {
                suggestion_type: OptimizationType::Compress,
                priority: 4,
                description: format!("Large file ({}MB) - consider compression", size / 1024 / 1024),
                estimated_savings: Some(EstimatedSaving::Percentage(30.0)),
            });
        }

        // Type-specific suggestions
        match asset_type {
            GameAssetType::Sprite2D | GameAssetType::Texture => {
                if format == "bmp" || format == "tga" {
                    suggestions.push(AssetOptimization {
                        suggestion_type: OptimizationType::ConvertFormat,
                        priority: 3,
                        description: "Convert to PNG or JPG for better compression".to_string(),
                        estimated_savings: Some(EstimatedSaving::Percentage(50.0)),
                    });
                }
                if size > 2 * 1024 * 1024 {
                    suggestions.push(AssetOptimization {
                        suggestion_type: OptimizationType::Resize,
                        priority: 2,
                        description: "Consider reducing resolution for mobile/WebGL".to_string(),
                        estimated_savings: Some(EstimatedSaving::Percentage(60.0)),
                    });
                }
            }
            GameAssetType::Model3D => {
                if size > 5 * 1024 * 1024 {
                    suggestions.push(AssetOptimization {
                        suggestion_type: OptimizationType::OptimizeMesh,
                        priority: 3,
                        description: "High polygon count - consider mesh optimization".to_string(),
                        estimated_savings: Some(EstimatedSaving::Percentage(40.0)),
                    });
                }
            }
            GameAssetType::Audio => {
                if format == "wav" && size > 1 * 1024 * 1024 {
                    suggestions.push(AssetOptimization {
                        suggestion_type: OptimizationType::OptimizeAudio,
                        priority: 3,
                        description: "Convert WAV to OGG or MP3 for compression".to_string(),
                        estimated_savings: Some(EstimatedSaving::Percentage(70.0)),
                    });
                }
            }
            GameAssetType::Video => {
                if size > 10 * 1024 * 1024 {
                    suggestions.push(AssetOptimization {
                        suggestion_type: OptimizationType::Compress,
                        priority: 5,
                        description: "Large video file - critical for mobile/WebGL".to_string(),
                        estimated_savings: Some(EstimatedSaving::Percentage(60.0)),
                    });
                }
            }
            _ => {}
        }

        suggestions
    }

    /// Get asset statistics
    pub fn get_asset_stats(&self, assets: &[GameAsset]) -> AssetStats {
        let mut type_counts: HashMap<GameAssetType, usize> = HashMap::new();
        let mut type_sizes: HashMap<GameAssetType, u64> = HashMap::new();
        let mut total_size = 0;

        for asset in assets {
            type_counts.entry(asset.asset_type).or_insert(0);
            type_counts.entry(asset.asset_type).and_modify(|c| *c += 1);

            type_sizes.entry(asset.asset_type).or_insert(0);
            type_sizes.entry(asset.asset_type).and_modify(|s| *s += asset.size_bytes);

            total_size += asset.size_bytes;
        }

        let optimization_count = assets.iter()
            .map(|a| a.optimization_suggestions.len())
            .sum();

        AssetStats {
            total_assets: assets.len(),
            total_size_bytes: total_size,
            type_counts,
            type_sizes,
            optimization_count,
        }
    }
}

impl Default for GameAssetDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStats {
    pub total_assets: usize,
    pub total_size_bytes: u64,
    pub type_counts: HashMap<GameAssetType, usize>,
    pub type_sizes: HashMap<GameAssetType, u64>,
    pub optimization_count: usize,
}

impl AssetStats {
    pub fn format_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{}KB", bytes / 1024)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{}MB", bytes / 1024 / 1024)
        } else {
            format!("{}GB", bytes / 1024 / 1024 / 1024)
        }
    }

    pub fn get_largest_type(&self) -> Option<(GameAssetType, u64)> {
        self.type_sizes.iter()
            .max_by_key(|(_, size)| *size)
            .map(|(t, s)| (*t, *s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_as_str() {
        assert_eq!(GameAssetType::Sprite2D.as_str(), "sprite2d");
        assert_eq!(GameAssetType::Model3D.as_str(), "model3d");
        assert_eq!(GameAssetType::Audio.as_str(), "audio");
    }

    #[test]
    fn test_asset_type_from_str() {
        assert_eq!(GameAssetType::from_str("sprite"), GameAssetType::Sprite2D);
        assert_eq!(GameAssetType::from_str("model"), GameAssetType::Model3D);
        assert_eq!(GameAssetType::from_str("unknown_type"), GameAssetType::Unknown);
    }

    #[test]
    fn test_detector_new() {
        let detector = GameAssetDetector::new();
        assert_eq!(detector.large_file_threshold, 5 * 1024 * 1024);
    }

    #[test]
    fn test_get_asset_type_from_extension() {
        let detector = GameAssetDetector::new();
        assert_eq!(detector.get_asset_type_from_extension("png"), GameAssetType::Sprite2D);
        assert_eq!(detector.get_asset_type_from_extension("fbx"), GameAssetType::Model3D);
        assert_eq!(detector.get_asset_type_from_extension("wav"), GameAssetType::Audio);
        assert_eq!(detector.get_asset_type_from_extension("shader"), GameAssetType::Shader);
        assert_eq!(detector.get_asset_type_from_extension("txt"), GameAssetType::Unknown);
    }

    #[test]
    fn test_optimization_suggestions_for_large_file() {
        let detector = GameAssetDetector::new();
        let suggestions = detector.get_optimization_suggestions(
            GameAssetType::Sprite2D,
            10 * 1024 * 1024, // 10MB
            "png"
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.suggestion_type == OptimizationType::Compress));
    }

    #[test]
    fn test_optimization_suggestions_for_bmp() {
        let detector = GameAssetDetector::new();
        let suggestions = detector.get_optimization_suggestions(
            GameAssetType::Sprite2D,
            500 * 1024, // 500KB
            "bmp"
        );
        assert!(suggestions.iter().any(|s| s.suggestion_type == OptimizationType::ConvertFormat));
    }

    #[test]
    fn test_optimization_suggestions_for_wav() {
        let detector = GameAssetDetector::new();
        let suggestions = detector.get_optimization_suggestions(
            GameAssetType::Audio,
            2 * 1024 * 1024, // 2MB
            "wav"
        );
        assert!(suggestions.iter().any(|s| s.suggestion_type == OptimizationType::OptimizeAudio));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(AssetStats::format_size(500), "500B");
        assert_eq!(AssetStats::format_size(2048), "2KB");
        assert_eq!(AssetStats::format_size(5 * 1024 * 1024), "5MB");
    }
}