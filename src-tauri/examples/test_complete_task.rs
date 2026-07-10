//! 完整的任务执行测试 - 发送任务，等待结果，验证文件生成

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    println!("=== 完整任务执行测试 ===\n");

    // 连接 WebSocket
    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url).await.expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 设置工作目录
    let workspace = "D:/dingsun/test_workspace";
    println!("工作目录: {}", workspace);

    // 1. 初始化 Executive Agent
    println!("\n1. 初始化 Hermes Native Executive Agent...");
    let init_cmd = json!({
        "id": "init-test",
        "type": "request",
        "command": "init_executive_agent",
        "payload": {
            "workspace": workspace,
            "hermesConfigDir": "D:/dingsun/acp-ui/hermes",
            "hermesModel": "alibaba-coding-plan:qwen3.6-plus"
        }
    });

    write
        .send(Message::Text(init_cmd.to_string().into()))
        .await
        .expect("发送初始化命令失败");

    // 等待初始化响应
    if let Some(msg) = tokio::time::timeout(Duration::from_secs(10), read.next())
        .await
        .ok()
        .flatten()
    {
        if let Ok(Message::Text(text)) = msg {
            println!("初始化响应: {}", text);
        }
    }

    // 2. 发送开发任务
    println!("\n2. 发送开发任务...");
    let task_request = r#"创建一个简单的 Rust Hello World 程序。

要求：
1. 在 src/main.rs 中编写代码
2. 打印 "Hello, Hermes Agent!"
3. 同时打印当前时间

请使用以下格式输出：
### FILE: src/main.rs
```rust
// 代码内容
```"#;

    let execute_cmd = json!({
        "id": "exec-hello",
        "type": "request",
        "command": "execute_development_task",
        "payload": {
            "request": task_request
        }
    });

    write
        .send(Message::Text(execute_cmd.to_string().into()))
        .await
        .expect("发送执行命令失败");

    println!(
        "✅ 任务已发送: {}",
        task_request.lines().take(3).collect::<Vec<_>>().join("\n")
    );

    // 3. 监听事件和结果
    println!("\n3. 监听执行结果...\n");

    let mut task_completed = false;
    let mut event_count = 0;
    let max_wait = Duration::from_secs(120);
    let start_time = std::time::Instant::now();

    while !task_completed && start_time.elapsed() < max_wait {
        match tokio::time::timeout(Duration::from_secs(30), read.next()).await {
            Ok(Some(msg)) => {
                match msg {
                    Ok(Message::Text(text)) => {
                        event_count += 1;
                        println!(
                            "[事件 {}] {}",
                            event_count,
                            text.chars().take(200).collect::<String>()
                        );

                        // 检查是否完成
                        if text.contains("task-completed")
                            || text.contains("\"status\":\"completed\"")
                            || text.contains("files") && text.contains("GeneratedFile")
                        {
                            println!("\n✅ 任务执行完成！");
                            task_completed = true;

                            // 解析结果
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(files) = value.get("data").and_then(|d| d.get("files"))
                                {
                                    println!("\n生成的文件:");
                                    for file in files.as_array().unwrap_or(&vec![]) {
                                        if let Some(path) = file.get("relative_path") {
                                            println!("  - {}", path);
                                        }
                                        if let Some(lines) = file.get("lines") {
                                            println!("    行数: {}", lines);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("WebSocket 关闭");
                        break;
                    }
                    Err(e) => {
                        println!("错误: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            Ok(None) => {
                println!("连接结束");
                break;
            }
            Err(_) => {
                println!("等待超时...");
            }
        }

        if event_count > 20 {
            println!("事件数量超过限制，停止监听");
            break;
        }
    }

    if !task_completed {
        println!("\n⚠️ 任务未在预期时间内完成");
    }

    // 4. 检查生成的文件
    println!("\n4. 检查生成的文件...");
    let workspace_path = std::path::Path::new(workspace);
    if workspace_path.exists() {
        println!("工作目录存在: {}", workspace);

        // 列出所有文件
        if let Ok(entries) = std::fs::read_dir(workspace_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                println!("  文件: {}", path.display());

                // 如果是 Rust 文件，显示内容
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        println!("    内容预览 (前10行):");
                        for line in content.lines().take(10) {
                            println!("      {}", line);
                        }
                    }
                }
            }
        }
    } else {
        println!("⚠️ 工作目录不存在: {}", workspace);
    }

    println!("\n=== 测试完成 ===");
}
