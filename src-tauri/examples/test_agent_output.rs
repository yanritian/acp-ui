//! 长时间监听 Claude Code agent 输出
//! 验证 spawn_and_execute_agent WebSocket 命令

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== 测试 Claude Code Agent 输出监听 ===\n");

    // 连接 WebSocket
    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 测试简单的请求
    let test_request = r#"请告诉我你能做什么，不需要生成文件"#;

    let spawn_cmd = json!({
        "id": "spawn-test-002",
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

    println!("命令已发送，等待 agent 输出...\n");

    // 监听所有消息
    let mut event_count = 0;
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(120),  // 等待 120 秒
            read.next()
        ).await {
            Ok(Some(msg)) => {
                match msg {
                    Ok(Message::Text(text)) => {
                        event_count += 1;
                        println!("\n[{}] 收到消息:", event_count);

                        // 尝试解析并格式化输出
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            let msg_type = parsed.get("type").and_then(|v| v.as_str());
                            println!("类型: {}", msg_type.unwrap_or("未知"));

                            if msg_type == Some("agent-message") {
                                let data = parsed.get("data");
                                if let Some(d) = data {
                                    if let Some(msg) = d.get("message") {
                                        println!("\n=== Agent 输出 ===");
                                        println!("{}", msg);
                                        println!("=== 输出结束 ===\n");
                                    }
                                }
                            }

                            // 检查是否完成
                            if msg_type == Some("agent-closed") {
                                println!("\n✅ Agent 已关闭");
                                break;
                            }

                            // 显示原始数据（截取）
                            let data_str = serde_json::to_string_pretty(&parsed).unwrap_or_default();
                            if data_str.len() > 500 {
                                println!("数据: {}...(已截取)", &data_str[..500]);
                            } else {
                                println!("数据: {}", data_str);
                            }
                        } else {
                            println!("原始: {}", text);
                        }

                        if event_count > 50 {
                            println!("\n消息数量超过限制，停止监听");
                            break;
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
                println!("超时 (120秒)");
                break;
            }
        }
    }

    println!("\n=== 测试完成 ===");
}