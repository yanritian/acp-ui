// Simple test for game detection
use std::path::PathBuf;

fn main() {
    let test_path = PathBuf::from("D:/tmp/test-game");
    
    // Check if project.godot exists
    let project_file = test_path.join("project.godot");
    if project_file.exists() {
        println!("✅ Godot project detected at: {}", test_path.display());
        
        // Read project name
        let content = std::fs::read_to_string(&project_file).unwrap();
        if let Some(name_line) = content.lines().find(|l| l.contains("config/name=")) {
            let name = name_line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"');
            println!("✅ Project name: {}", name);
        }
        
        // Check version
        if content.contains("4.2") {
            println!("✅ Godot version: 4.2");
        }
        
        // Check main scene
        let main_scene = test_path.join("main.tscn");
        if main_scene.exists() {
            println!("✅ Main scene found: main.tscn");
        }
        
        println!("\n🎉 Game detection test PASSED!");
    } else {
        println!("❌ Project not found");
    }
}
