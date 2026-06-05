//! 测试 Hermes 与阿里百炼云 API 连接

use hermes_agent::{AgentLoop, agent_builder::{build_agent_config, build_provider, bridge_tool_registry}};
use hermes_core::Message;
use hermes_config::load_config;
use hermes_tools::ToolRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Hermes + 阿里百炼云 API 测试 ===\n");

    // 加载配置
    let config_dir = Some("D:/dingsun/acp-ui/hermes");
    println!("加载配置目录: {:?}", config_dir);

    let gateway_config = match load_config(config_dir) {
        Ok(cfg) => {
            println!("✅ 配置加载成功");
            println!("   Model: {:?}", cfg.model);
            if let Some(providers) = cfg.llm_providers.get("alibaba-coding-plan") {
                println!("   API Key: {}...", &providers.api_key.as_ref().unwrap()[..10]);
                println!("   Base URL: {:?}", providers.base_url);
            }
            cfg
        }
        Err(e) => {
            println!("❌ 配置加载失败: {}", e);
            return;
        }
    };

    // 构建 Agent
    let model = "alibaba-coding-plan:qwen3.6-plus";
    println!("\n构建 Agent with model: {}", model);

    let agent_config = build_agent_config(&gateway_config, model, Some("test"));
    let llm_provider = build_provider(&gateway_config, model);
    let tools = ToolRegistry::new();
    let tool_registry = Arc::new(bridge_tool_registry(&tools));

    let agent_loop = AgentLoop::new(agent_config, tool_registry, llm_provider);

    // 执行简单任务
    println!("\n执行测试任务: 写一个简单的 Hello World 程序");
    let messages = vec![Message::user("写一个简单的 Rust Hello World 程序，只需要打印 Hello World 即可。")];

    println!("开始调用 API...\n");
    let result = agent_loop.run(messages, None).await;

    match result {
        Ok(agent_result) => {
            println!("✅ Agent 执行成功!");
            println!("消息数量: {}", agent_result.messages.len());
            for msg in &agent_result.messages {
                if msg.role == hermes_core::MessageRole::Assistant {
                    println!("\n=== Agent 响应 ===");
                    println!("{}", msg.content.as_ref().unwrap_or(&"无内容".to_string()));
                }
            }
        }
        Err(e) => {
            println!("❌ Agent 执行失败: {}", e);
        }
    }

    println!("\n=== 测试完成 ===");
}