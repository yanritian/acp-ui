//! 通过 WebSocket 发送 ERP 开发任务到 ACP-UI
//! 使用 Hermes Native (阿里云 Coding Plan) 执行

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== 发送 ERP 开发任务到 ACP-UI ===\n");

    // 连接 WebSocket
    let ws_url = "ws://127.0.0.1:1421";
    println!("连接 WebSocket: {}", ws_url);

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("无法连接 WebSocket");

    println!("✅ WebSocket 连接成功\n");

    let (mut write, mut read) = ws_stream.split();

    // 设置工作目录
    let workspace = "D:/dingsun/erp_project";
    println!("工作目录: {}", workspace);

    // 1. 初始化 Executive Agent
    println!("\n1. 初始化 Hermes Native Executive Agent...");
    let init_cmd = json!({
        "id": "init-erp",
        "type": "request",
        "command": "init_executive_agent",
        "payload": {
            "workspace": workspace,
            "hermesConfigDir": "D:/dingsun/acp-ui/hermes",
            "hermesModel": "alibaba-coding-plan:qwen3.6-plus"
        }
    });

    write.send(Message::Text(init_cmd.to_string().into()))
        .await
        .expect("发送失败");

    // 等待初始化响应
    if let Some(msg) = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        read.next()
    ).await.ok().flatten() {
        if let Ok(Message::Text(text)) = msg {
            println!("初始化响应: {}", text);
        }
    }

    // 2. 发送 ERP 开发任务
    println!("\n2. 发送 ERP 开发任务...");
    let erp_request = r#"开发一个简单的 ERP 系统原型，包含以下核心功能：

## 需求说明
1. 用户管理模块：用户登录、权限管理
2. 产品管理模块：产品信息 CRUD
3. 订单管理模块：订单创建、查询、状态更新
4. 库存管理模块：库存查询、入库、出库

## 技术要求
- 使用 C# + Vue + Windows App 架构
- 后端使用 ASP.NET Core Web API
- 前端使用 Vue 3 + TypeScript
- 数据库使用 SQLite

## 输出格式
请使用 ### FILE: 路径 格式输出每个文件，包括：
- 后端 API 控制器
- 数据模型
- 前端组件
- 配置文件"#;

    let exec_cmd = json!({
        "id": "exec-erp",
        "type": "request",
        "command": "execute_development_task",
        "payload": {
            "request": erp_request
        }
    });

    write.send(Message::Text(exec_cmd.to_string().into()))
        .await
        .expect("发送失败");

    println!("任务已发送，等待执行...\n");

    // 监听执行事件
    let mut event_count = 0;
    loop {
        match tokio::time::timeout(
            std::time::Duration::from_secs(300),  // 等待 5 分钟
            read.next()
        ).await {
            Ok(Some(msg)) => {
                match msg {
                    Ok(Message::Text(text)) => {
                        event_count += 1;
                        println!("\n[{}] 收到事件:", event_count);

                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            let msg_type = parsed.get("type").and_then(|v| v.as_str());
                            println!("类型: {}", msg_type.unwrap_or("未知"));

                            if msg_type == Some("task-completed") {
                                println!("\n✅ 任务完成!");
                                if let Some(data) = parsed.get("data") {
                                    println!("结果: {}", serde_json::to_string_pretty(data).unwrap_or_default());
                                }
                                break;
                            }

                            if msg_type == Some("file-created") {
                                if let Some(data) = parsed.get("data") {
                                    let path = data.get("path").and_then(|v| v.as_str());
                                    let lines = data.get("lines");
                                    println!("📁 创建文件: {} (? 行)", path.unwrap_or("?"));
                                }
                            }

                            if msg_type == Some("agent-message") {
                                if let Some(data) = parsed.get("data") {
                                    if let Some(msg) = data.get("message") {
                                        println!("💬 {}", msg);
                                    }
                                }
                            }
                        }

                        if event_count > 100 {
                            println!("\n事件数量过多，停止监听");
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
                println!("\n超时 (300秒)");
                break;
            }
        }
    }

    println!("\n=== 测试完成 ===");
    println!("请检查工作目录: {}", workspace);
}