//! 查询 ACP-UI 数据库内容
use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    let db_path = PathBuf::from("C:/Users/Administrator/AppData/Local/acp-ui/history.db");

    println!("数据库路径: {}", db_path.display());

    if !db_path.exists() {
        println!("数据库文件不存在!");
        return;
    }

    let conn = Connection::open(&db_path).expect("无法打开数据库");

    // 查询表
    println!("\n=== 数据库表 ===");
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    println!("表: {:?}", tables);

    // 查询 tasks 表
    println!("\n=== 任务记录 ===");
    let mut stmt = conn
        .prepare(
            "
        SELECT id, name, status, source, created_at, completed_at, error_message
        FROM tasks
        ORDER BY created_at DESC
        LIMIT 10
    ",
        )
        .unwrap();

    let tasks = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })
        .unwrap();

    for task in tasks.filter_map(|t| t.ok()) {
        println!("\n任务 ID: {}", task.0);
        println!("名称: {}", task.1);
        println!("状态: {}", task.2);
        println!("来源: {}", task.3);
        println!("创建时间: {}", task.4);
        println!("完成时间: {:?}", task.5);
        println!("错误: {:?}", task.6);
    }

    // 查询任务总数
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .unwrap();
    println!("\n总任务数: {}", count);

    // 查询 executive_sessions
    println!("\n=== Executive Sessions ===");
    let exec_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM executive_sessions", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    println!("Executive Sessions 数: {}", exec_count);

    // 查询 sessions
    println!("\n=== Sessions ===");
    let sess_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap_or(0);
    println!("Sessions 数: {}", sess_count);

    // 查询 logs
    println!("\n=== Logs ===");
    let log_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))
        .unwrap_or(0);
    println!("Logs 数: {}", log_count);

    // 查询 runtime_events
    println!("\n=== Runtime Events ===");
    let event_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM runtime_events", [], |row| row.get(0))
        .unwrap_or(0);
    println!("Runtime Events 数: {}", event_count);
}
