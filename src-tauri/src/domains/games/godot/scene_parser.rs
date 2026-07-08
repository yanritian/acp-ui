// Godot Domain Pack - Scene Parser
// Parses Godot .tscn files to extract node structure

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotScene {
    pub path: String,
    pub name: String,
    pub root_node: GodotNode,
    pub nodes: Vec<GodotNode>,
    pub connections: Vec<GodotConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotNode {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
    pub properties: HashMap<String, String>,
    pub scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotConnection {
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct GodotSceneParser;

impl GodotSceneParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a .tscn file
    pub fn parse_scene(path: &Path) -> Result<GodotScene, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read scene: {}", e))?;

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut nodes = Vec::new();
        let mut connections = Vec::new();
        let mut current_node: Option<GodotNode> = None;
        let mut root_node = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("[ext_resource") || line.starts_with("[sub_resource") {
                continue;
            }

            if line.starts_with("[node]") {
                // Save previous node if exists
                if let Some(node) = current_node.take() {
                    if root_node.is_none() && node.parent.is_none() {
                        root_node = Some(node.clone());
                    }
                    nodes.push(node);
                }

                // Start new node
                let node = Self::parse_node_header(line);
                current_node = Some(node);
            } else if line.contains('=') {
                // Property assignment
                if let Some(ref mut node) = current_node {
                    let (key, value) = Self::parse_property(line);
                    if key == "script" {
                        node.scripts.push(value);
                    } else {
                        node.properties.insert(key, value);
                    }
                }
            } else if line.starts_with("[connection]") {
                // Save current node before connection
                if let Some(node) = current_node.take() {
                    if root_node.is_none() && node.parent.is_none() {
                        root_node = Some(node.clone());
                    }
                    nodes.push(node);
                }

                // Parse connection
                if let Some(conn) = Self::parse_connection(line) {
                    connections.push(conn);
                }
            }
        }

        // Save last node
        if let Some(node) = current_node {
            if root_node.is_none() && node.parent.is_none() {
                root_node = Some(node.clone());
            }
            nodes.push(node);
        }

        let root = root_node.unwrap_or_else(|| GodotNode {
            name: "Root".to_string(),
            node_type: "Node".to_string(),
            parent: None,
            properties: HashMap::new(),
            scripts: Vec::new(),
        });

        Ok(GodotScene {
            path: path.to_string_lossy().to_string(),
            name,
            root_node: root,
            nodes,
            connections,
        })
    }

    /// Find all character/player nodes in a scene
    pub fn find_character_nodes(scene: &GodotScene) -> Vec<&GodotNode> {
        let keywords = ["player", "character", "actor", "hero", "entity"];

        scene.nodes.iter()
            .filter(|node| {
                let name_lower = node.name.to_lowercase();
                let type_lower = node.node_type.to_lowercase();
                keywords.iter().any(|kw| {
                    name_lower.contains(kw) || type_lower.contains(kw)
                })
            })
            .collect()
    }

    /// Extract physics-related properties
    pub fn get_physics_properties(node: &GodotNode) -> HashMap<String, String> {
        let physics_keys = [
            "gravity", "mass", "linear_damp", "angular_damp",
            "jump_velocity", "speed", "max_speed", "acceleration",
        ];

        node.properties.iter()
            .filter(|(k, _)| physics_keys.iter().any(|pk| k.to_lowercase().contains(pk)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // ============================================================================
    // Helper Methods
    // ============================================================================

    fn parse_node_header(line: &str) -> GodotNode {
        let mut name = "Unknown".to_string();
        let mut node_type = "Node".to_string();
        let mut parent = None;

        // Parse: [node name="Player" type="CharacterBody2D" parent="."]
        for part in line.split_whitespace() {
            if part.starts_with("name=") {
                name = part.trim_start_matches("name=")
                    .trim_matches('"')
                    .to_string();
            } else if part.starts_with("type=") {
                node_type = part.trim_start_matches("type=")
                    .trim_matches('"')
                    .to_string();
            } else if part.starts_with("parent=") {
                parent = Some(part.trim_start_matches("parent=")
                    .trim_matches('"')
                    .to_string());
            }
        }

        GodotNode {
            name,
            node_type,
            parent,
            properties: HashMap::new(),
            scripts: Vec::new(),
        }
    }

    fn parse_property(line: &str) -> (String, String) {
        if let Some(idx) = line.find('=') {
            let key = line[..idx].trim().to_string();
            let value = line[idx + 1..].trim().to_string();
            (key, value)
        } else {
            (line.to_string(), String::new())
        }
    }

    fn parse_connection(_line: &str) -> Option<GodotConnection> {
        // Simplified: [connection signal="pressed" from="Button" to="Player" method="_on_button_pressed"]
        // TODO: Implement full connection parsing
        None
    }
}
