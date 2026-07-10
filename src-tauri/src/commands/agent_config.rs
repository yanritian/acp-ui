use crate::agent_config_parser::{self, AgentConfigParsed, AgentConfigParser, ParseResult};
use crate::AppState;
use tauri::State;

// ===== Agent Config Parser Commands =====

#[tauri::command]
pub fn parse_agent_config(yaml_content: String) -> ParseResult {
    let parser = AgentConfigParser::new();
    parser.parse_yaml(&yaml_content)
}

#[tauri::command]
pub fn save_agent_config(
    config: AgentConfigParsed,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.save_to_db(&config, &conn)
}

#[tauri::command]
pub fn load_agent_config(
    name: String,
    state: State<AppState>,
) -> Result<Option<AgentConfigParsed>, String> {
    let db = state.database.lock().unwrap();
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.load_from_db(&name, &conn)
}

#[tauri::command]
pub fn list_agent_configs(state: State<AppState>) -> Result<Vec<(String, String)>, String> {
    let db = state.database.lock().unwrap();
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.list_configs(&conn)
}

#[tauri::command]
pub fn delete_agent_config(name: String, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.delete_config(&name, &conn)
}

#[tauri::command]
pub fn get_example_agent_config() -> String {
    agent_config_parser::get_example_config()
}
