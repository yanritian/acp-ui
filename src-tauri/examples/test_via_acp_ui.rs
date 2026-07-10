//! 通过 WebSocket 向 ACP-UI 发送 Hermes 测试任务
//! 完整正确的命令格式

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    println!("=== 通过 ACP-UI WebSocket 测试 Hermes ===\n");

    // 连接 WebSocket (默认端口 1421)
    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url).await.expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 发送初始化命令（完整格式 - 包含type字段）
    println!("发送初始化命令...");
    let init_cmd = json!({
        "id": "init-001",
        "type": "command",  // RemoteRequest需要type字段
        "command": "init_executive_agent",
        "payload": {
            "workspace": "D:/smallProject",
            "hermesConfigDir": "D:/dingsun/acp-ui/hermes",
            "hermesModel": "alibaba-coding-plan:qwen3.6-plus"
        }
    });

    write
        .send(Message::Text(init_cmd.to_string().into()))
        .await
        .expect("发送失败");

    println!("初始化命令已发送: {}\n", init_cmd);

    // 等待响应
    println!("等待响应...");
    if let Some(msg) = tokio::time::timeout(std::time::Duration::from_secs(10), read.next())
        .await
        .ok()
        .flatten()
    {
        match msg {
            Ok(Message::Text(text)) => {
                println!("✅ 收到响应: {}\n", text);
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket被关闭");
            }
            _ => println!("收到其他类型消息\n"),
        }
    } else {
        println!("⚠ 未收到响应（可能没有token要求）\n");
    }

    // 发送测试任务
    println!("发送测试任务...");
    let test_request = r#"创建一个简单的 Rust 程序，计算并打印 1 到 10 的累加和。

使用以下格式输出文件：
### FILE: sum.rs
```rust
fn main() {
    let sum = (1..=10).sum::<i32>();
    println!("1到10的累加和: {}", sum);
}
```"#;

    let execute_cmd = json!({
        "id": "exec-001",
        "type": "command",
        "command": "execute_development_task",
        "payload": {
            "request": test_request
        }
    });

    write
        .send(Message::Text(execute_cmd.to_string().into()))
        .await
        .expect("发送失败");

    println!("✅ 测试任务已发送\n");
    println!("任务内容:\n{}\n", test_request);

    // 监听事件
    println!("监听 WebSocket 事件...\n");

    let mut event_count = 0;
    loop {
        match tokio::time::timeout(std::time::Duration::from_secs(60), read.next()).await {
            Ok(Some(msg)) => match msg {
                Ok(Message::Text(text)) => {
                    event_count += 1;
                    println!("[{}] 收到: {}", event_count, text);
                    if text.contains("task-completed") || text.contains("\"ok\":true") {
                        println!("\n✅ 任务完成!");
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket关闭");
                    break;
                }
                Err(e) => {
                    println!("错误: {}", e);
                    break;
                }
                _ => {}
            },
            Ok(None) => {
                println!("连接结束");
                break;
            }
            Err(_) => {
                println!("超时，停止监听");
                break;
            }
        }

        if event_count > 10 {
            break;
        }
    }

    println!("\n=== 测试完成 ===");
    println!("请检查 ACP-UI 前端和数据库");
}
