// hermes_smoke.rs - 独立验证 hermes AgentLoop + qwen
// 运行：cargo run --example hermes_smoke
//
// 这是 Week 1 Spike 1：验证 hermes 能不能跑通
// 不依赖 Tauri 应用，独立测试 hermes 引擎

use hermes_agent::{
    agent_builder::{bridge_tool_registry, build_agent_config, build_provider},
    AgentLoop,
};
use hermes_config::load_config;
use hermes_core::Message;
use hermes_tools::ToolRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hermes Smoke Test ===\n");

    // 1. 加载配置
    println!("[1/5] 加载配置...");
    let config_dir = Some("D:/dingsun/acp-ui/hermes");
    let gateway_config = load_config(config_dir)?;
    let model = gateway_config
        .model
        .clone()
        .unwrap_or_else(|| "alibaba-coding-plan:qwen3.6-plus".to_string());
    println!("  ✓ 配置加载成功");
    println!("  默认模型: {}", model);

    // 2. 构建 AgentConfig
    println!("\n[2/5] 构建 AgentConfig...");
    let agent_config = build_agent_config(&gateway_config, &model, Some("hermes-smoke"));
    println!("  ✓ AgentConfig 构建成功");
    println!("  max_turns: {}", agent_config.max_turns);

    // 3. 构建 LLM Provider
    println!("\n[3/5] 构建 LLM Provider...");
    let llm_provider = build_provider(&gateway_config, &model);
    println!("  ✓ LLM Provider 构建成功");

    // 4. 创建 AgentLoop（空工具注册表）
    println!("\n[4/5] 创建 AgentLoop...");
    let tools = ToolRegistry::new();
    let tool_registry = Arc::new(bridge_tool_registry(&tools));
    let agent_loop = AgentLoop::new(agent_config, tool_registry, llm_provider);
    println!("  ✓ AgentLoop 创建成功");

    // 5. 执行测试
    println!("\n[5/5] 执行测试 (发送 'hello' 给 qwen)...");
    let messages = vec![Message::user("你好，请用一句话介绍自己。")];

    match agent_loop.run(messages, None).await {
        Ok(result) => {
            println!("\n=== 成功！ ===");
            println!("回复消息数: {}", result.messages.len());

            // 提取 assistant 回复
            for msg in &result.messages {
                if msg.role == hermes_core::MessageRole::Assistant {
                    println!("\nAssistant 回复:");
                    println!(
                        "{}",
                        msg.content.clone().unwrap_or_else(|| "(空)".to_string())
                    );
                }
            }

            println!("\n✓ hermes + qwen 工作正常！");
            println!("  可以集成到 Tauri 应用中使用。");
        }
        Err(e) => {
            eprintln!("\n=== 失败！ ===");
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
