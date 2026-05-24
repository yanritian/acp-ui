//! 测试 spawn_and_execute_agent WebSocket 命令
//! 验证 Claude Code agent spawn 功能

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== 测试 spawn_and_execute_agent 命令 ===\n");

    // 连接 WebSocket
    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 测试 spawn_and_execute_agent 命令
    println!("测试 spawn Claude Code agent...");

    let test_request = r#"创建一个简单的 Rust 程序，计算并打印斐波那契数列的前10项。

使用以下格式输出：
### FILE: fibonacci.rs
```rust
fn fibonacci(n: u32) -> u32 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
fn main() {
    for i in 0..10 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
```"#;

    let spawn_cmd = json!({
        "id": "spawn-test-001",
        "type": "request",
        "command": "spawn_and_execute_agent",
        "payload": {
            "agent_name": "Claude Code",
            "request": test_request,
            "workspace": "D:/dingsun/erp_system"
        }
    });

    write.send(Message::Text(spawn_cmd.to_string().into()))
        .await
        .expect("发送失败");

    println!("命令已发送: {}", spawn_cmd);

    // 监听响应
    println!("\n等待响应...\n");

    let mut event_count = 0;
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            read.next()
        ).await {
            Ok(Some(msg)) => {
                match msg {
                    Ok(Message::Text(text)) => {
                        event_count += 1;
                        println!("[{}] 收到: {}", event_count, text);

                        // 解析响应
                        if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                            if response.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                                println!("\n✅ 响应成功!");
                            }
                        }

                        if event_count > 5 {
                            println!("\n停止监听");
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
                }
            }
            Ok(None) => {
                println!("连接结束");
                break;
            }
            Err(_) => {
                println!("超时");
                break;
            }
        }
    }

    println!("\n=== 测试完成 ===");
}