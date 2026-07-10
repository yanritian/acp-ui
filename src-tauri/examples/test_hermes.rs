//! 测试 Hermes Native 执行功能
//! 直接调用 Hermes AgentLoop 执行简单编码任务

use hermes_core::Message;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Hermes Native 执行测试 ===\n");

    // 配置 Hermes（使用项目 hermes/config.yaml）
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();

    println!("配置目录: {:?}", config_dir);
    println!("模型: {}", model);
    println!();

    // 测试配置加载
    println!("--- 测试 Hermes 配置加载 ---");
    match hermes_config::load_config(config_dir.as_deref()) {
        Ok(config) => {
            println!("✅ 配置加载成功");
            println!("   模型: {:?}", config.model);

            if let Some(provider) = config.llm_providers.get("alibaba-coding-plan") {
                let api_key_preview = provider
                    .api_key
                    .as_deref()
                    .unwrap_or("未设置")
                    .chars()
                    .take(10)
                    .collect::<String>()
                    + "...";
                println!("   API Key: {}", api_key_preview);
                println!(
                    "   Base URL: {}",
                    provider.base_url.as_deref().unwrap_or_default()
                );
            }
        }
        Err(e) => {
            println!("❌ 配置加载失败: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n--- 测试 Hermes AgentLoop 构建 ---");

    // 加载配置
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    // 构建 AgentConfig
    let agent_config =
        hermes_agent::agent_builder::build_agent_config(&gateway_config, &model, Some("test"));
    println!("✅ AgentConfig 构建成功");
    println!("   max_turns: {}", agent_config.max_turns);
    println!("   stream: {}", agent_config.stream);

    // 构建 Provider
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    println!("✅ LLM Provider 构建成功");

    // 创建空的 ToolRegistry
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    println!(
        "✅ ToolRegistry 构建成功 (工具数量: {})",
        tool_registry.names().len()
    );

    // 创建 AgentLoop
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);
    println!("✅ AgentLoop 构建成功\n");

    // 构建测试消息
    let test_prompt = r#"你是编码智能体。请用 Rust 写一个计算斐波那契数列的程序。

重要：使用 ### FILE: 路径 格式标记文件：
### FILE: fibonacci.rs
```rust
fn main() {
    // 你的代码
}
```

要求：
1. 计算前10个斐波那契数
2. 打印输出结果
3. 代码简洁清晰"#;

    let messages = vec![Message::user(test_prompt)];

    println!("--- 测试 Hermes AgentLoop 执行 ---");
    println!(
        "提示词预览:\n{}\n",
        test_prompt.lines().take(5).collect::<Vec<_>>().join("\n")
    );

    // 执行
    println!("开始执行 AgentLoop.run()...\n");
    match agent_loop.run(messages, None).await {
        Ok(result) => {
            println!("✅ AgentLoop 执行成功!");
            println!("   消息数量: {}", result.messages.len());
            println!("   总 turns: {}", result.total_turns);

            // 提取响应内容
            let response = result
                .messages
                .iter()
                .rev()
                .find_map(|m| {
                    if m.role == hermes_core::MessageRole::Assistant {
                        m.content.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            println!("\n--- AI 响应内容 ---");
            for line in response.lines().take(40) {
                println!("{}", line);
            }
            if response.lines().count() > 40 {
                println!("... (共 {} 行)", response.lines().count());
            }

            // 检查是否生成了文件标记
            if response.contains("### FILE:") {
                println!("\n✅ 检测到文件生成标记!");
            }
        }
        Err(e) => {
            println!("❌ AgentLoop 执行失败: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== 测试完成 ===");
}
