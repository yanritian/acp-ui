// Godot Domain Pack - Scene Parser
// Parses Godot .tscn files to extract node structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read scene: {}", e))?;

        let name = path
            .file_stem()
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

            if line.starts_with("[node") && line.ends_with(']') {
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
            } else if line.starts_with("[connection") && line.ends_with(']') {
                // Save current node before connection
                if let Some(node) = current_node.take() {
                    if root_node.is_none() && node.parent.is_none() {
                        root_node = Some(node.clone());
                    }
                    nodes.push(node);
                }

                // Parse connection
                connections.push(Self::parse_connection(line)?);
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

        scene
            .nodes
            .iter()
            .filter(|node| {
                let name_lower = node.name.to_lowercase();
                let type_lower = node.node_type.to_lowercase();
                keywords
                    .iter()
                    .any(|kw| name_lower.contains(kw) || type_lower.contains(kw))
            })
            .collect()
    }

    /// Extract physics-related properties
    pub fn get_physics_properties(node: &GodotNode) -> HashMap<String, String> {
        let physics_keys = [
            "gravity",
            "mass",
            "linear_damp",
            "angular_damp",
            "jump_velocity",
            "speed",
            "max_speed",
            "acceleration",
        ];

        node.properties
            .iter()
            .filter(|(k, _)| physics_keys.iter().any(|pk| k.to_lowercase().contains(pk)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // ============================================================================
    // Helper Methods
    // ============================================================================

    fn parse_node_header(line: &str) -> GodotNode {
        let attributes = Self::parse_attributes(line);
        let name = attributes
            .get("name")
            .cloned()
            .unwrap_or_else(|| "Root".to_string());
        let node_type = attributes
            .get("type")
            .cloned()
            .unwrap_or_else(|| "Node".to_string());
        let parent = attributes.get("parent").cloned();

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

    fn parse_connection(line: &str) -> Result<GodotConnection, String> {
        let attributes = Self::parse_attributes(line);
        let required = |key: &str| {
            attributes
                .get(key)
                .cloned()
                .ok_or_else(|| format!("Godot connection is missing '{}'", key))
        };
        Ok(GodotConnection {
            signal: required("signal")?,
            from: required("from")?,
            to: required("to")?,
            method: required("method")?,
        })
    }

    /// Parse Godot's section attributes without splitting inside quoted values.
    fn parse_attributes(line: &str) -> HashMap<String, String> {
        let inner = line
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']');
        let mut attributes = HashMap::new();
        let mut chars = inner.chars().peekable();

        while chars.peek().is_some() {
            while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
                chars.next();
            }
            while matches!(chars.peek(), Some('=')) {
                chars.next();
            }
            let mut key = String::new();
            while let Some(ch) = chars.peek().copied() {
                if ch == '=' || ch.is_whitespace() {
                    break;
                }
                key.push(ch);
                chars.next();
            }
            if key.is_empty() {
                chars.next();
                continue;
            }
            while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
                chars.next();
            }
            if chars.peek() != Some(&'=') {
                continue;
            }
            chars.next();
            while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
                chars.next();
            }

            let value = if chars.peek() == Some(&'"') {
                chars.next();
                let mut value = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    value.push(ch);
                }
                value
            } else {
                let mut value = String::new();
                while let Some(ch) = chars.peek().copied() {
                    if ch.is_whitespace() {
                        break;
                    }
                    value.push(ch);
                    chars.next();
                }
                value
            };
            attributes.insert(key, value);
        }

        attributes
    }
}

#[cfg(test)]
mod tests {
    use super::GodotSceneParser;
    use std::path::PathBuf;

    #[test]
    fn parses_nodes_properties_and_connections_from_fixture() {
        let scene = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test-godot-project")
            .join("scenes")
            .join("Main.tscn");
        let parsed = GodotSceneParser::parse_scene(&scene).expect("parse fixture scene");

        assert_eq!(parsed.root_node.name, "Main");
        assert_eq!(parsed.nodes.len(), 3);
        assert_eq!(parsed.nodes[1].name, "Player");
        assert_eq!(parsed.nodes[1].node_type, "CharacterBody2D");
        assert_eq!(parsed.nodes[1].parent.as_deref(), Some("."));
        assert!(parsed.nodes[1].scripts[0].contains("ExtResource"));
    }

    #[test]
    fn parses_connection_attributes_with_quoted_values() {
        let root = std::env::current_dir()
            .expect("current directory")
            .join(".tmp-tests")
            .join(format!("scene_parser_{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create parser test directory");
        let path = root.join("connected.tscn");
        std::fs::write(
            &path,
            "[gd_scene format=3]\n\n[node name=\"Main Scene\" type=\"Node2D\"]\n\n[connection signal=\"pressed\" from=\"Main Scene/Button\" to=\"Player\" method=\"_on_button_pressed\"]\n",
        )
        .expect("write parser fixture");

        let parsed = GodotSceneParser::parse_scene(&path).expect("parse connection fixture");
        assert_eq!(parsed.connections.len(), 1);
        assert_eq!(parsed.connections[0].signal, "pressed");
        assert_eq!(parsed.connections[0].from, "Main Scene/Button");
        assert_eq!(parsed.connections[0].method, "_on_button_pressed");

        let _ = std::fs::remove_dir_all(root);
    }
}
