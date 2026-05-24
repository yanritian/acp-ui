//! 测试 WebSocket 查询任务历史

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== 测试 WebSocket 查询任务历史 ===\n");

    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 查询任务历史
    println!("查询任务历史...");
    let history_cmd = json!({
        "id": "history-001",
        "type": "request",
        "command": "get_task_history",
        "payload": {
            "limit": 10
        }
    });

    write.send(Message::Text(history_cmd.to_string().into()))
        .await
        .expect("发送失败");

    // 等待响应
    if let Some(msg) = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        read.next()
    ).await.ok().flatten() {
        if let Ok(Message::Text(text)) = msg {
            println!("\n收到响应:\n{}", text);

            // 解析并显示任务列表
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                if response.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                    if let Some(data) = response.get("data") {
                        if let Some(tasks) = data.get("tasks").and_then(|v| v.as_array()) {
                            println!("\n=== 任务列表 ({}) ===", tasks.len());
                            for task in tasks {
                                let id = task.get("id").and_then(|v| v.as_str());
                                let name = task.get("name").and_then(|v| v.as_str());
                                let status = task.get("status").and_then(|v| v.as_str());
                                let source = task.get("source").and_then(|v| v.as_str());
                                println!("\n任务: {}", id.unwrap_or("?"));
                                println!("  名称: {}", name.unwrap_or("?"));
                                println!("  状态: {}", status.unwrap_or("?"));
                                println!("  来源: {}", source.unwrap_or("?"));
                            }
                        }
                    }
                }
            }
        }
    }

    // 查询统计
    println!("\n\n查询任务统计...");
    let stats_cmd = json!({
        "id": "stats-001",
        "type": "request",
        "command": "get_task_statistics"
    });

    write.send(Message::Text(stats_cmd.to_string().into()))
        .await
        .expect("发送失败");

    if let Some(msg) = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        read.next()
    ).await.ok().flatten() {
        if let Ok(Message::Text(text)) = msg {
            println!("\n收到统计响应:\n{}", text);
        }
    }

    println!("\n=== 测试完成 ===");
}