use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    let db_path = PathBuf::from("C:/Users/Administrator/AppData/Local/acp-ui/history.db");
    println!("数据库路径: {}", db_path.display());

    let conn = Connection::open(&db_path).expect("Failed to open database");

    // Query executive_sessions
    println!("\n=== Executive Sessions ===");
    let sessions: Vec<(String, String, String, String, Option<String>, Option<String>, Option<String>, String, Option<String>)> = conn
        .prepare("SELECT id, request, workspace, status, summary, files_json, logs_json, created_at, completed_at FROM executive_sessions ORDER BY created_at DESC LIMIT 5")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
            ))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    for session in sessions {
        println!("\nSession ID: {}", session.0);
        println!("Request: {}", session.1);
        println!("Workspace: {}", session.2);
        println!("Status: {}", session.3);
        println!("Summary: {:?}", session.4);
        println!("Files JSON: {:?}", session.5);
        println!("Logs JSON: {:?}", session.6);
        println!("Created At: {}", session.7);
        println!("Completed At: {:?}", session.8);
    }

    // Query thinking_chunks
    println!("\n=== Thinking Chunks ===");
    let chunks: Vec<(String, String, String, i32, i64, String)> = conn
        .prepare("SELECT id, task_id, content, depth, duration_ms, created_at FROM thinking_chunks ORDER BY created_at DESC LIMIT 10")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    println!("Thinking Chunks 数: {}", chunks.len());
    for chunk in chunks {
        println!("\nChunk ID: {}", chunk.0);
        println!("Task ID: {}", chunk.1);
        println!("Content: {}", chunk.2.chars().take(100).collect::<String>());
        println!("Depth: {}", chunk.3);
        println!("Duration MS: {}", chunk.4);
        println!("Created At: {}", chunk.5);
    }

    // Query tool_calls
    println!("\n=== Tool Calls ===");
    let tools: Vec<(String, String, String, Option<String>, Option<String>, String, Option<i64>, Option<String>, String, Option<String>)> = conn
        .prepare("SELECT id, task_id, tool_name, arguments_json, result, status, duration_ms, error, created_at, completed_at FROM tool_calls ORDER BY created_at DESC LIMIT 10")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    println!("Tool Calls 数: {}", tools.len());
    for tool in tools {
        println!("\nTool Call ID: {}", tool.0);
        println!("Task ID: {}", tool.1);
        println!("Tool Name: {}", tool.2);
        println!("Arguments: {:?}", tool.3);
        println!("Result: {:?}", tool.4);
        println!("Status: {}", tool.5);
        println!("Duration MS: {:?}", tool.6);
        println!("Error: {:?}", tool.7);
        println!("Created At: {}", tool.8);
        println!("Completed At: {:?}", tool.9);
    }
}