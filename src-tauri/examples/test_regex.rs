//! 简化测试 - 调试正则匹配问题

use std::sync::Arc;
use regex::Regex;
use hermes_core::Message;

#[tokio::main]
async fn main() {
    println!("=== 测试 AI 输出格式解析 ===\n");

    // 测试已知格式的解析
    let test_responses = vec![
        // 格式1: ### FILE: xxx
        r#"### FILE: test.rs
```rust
fn main() {}
```"#,
        // 格式2: ### 📄 FILE: `xxx`
        r#"### 📄 FILE: `fibonacci.rs`
```rust
fn fib() {}
```"#,
        // 格式3: ### 📄 `xxx`
        r#"### 📄 `main.rs`
```rust
fn main() {}
```"#,
    ];

    // 测试多种正则表达式
    let patterns = vec![
        // 原始
        (r"###\s*FILE:\s*([^\n]+)\s*```[\w]*\s*([\s\S]*?)```", "原始格式"),
        // 支持表情符号
        (r"###\s*(?:📄)?\s*FILE:\s*[`]?([^`\n]+)[`]?\s*```[\w]*\s*([\s\S]*?)```", "带表情符号"),
        // 更宽松
        (r"###.*?[`]?([^`\n]+\.rs)[`]?\s*```[\w]*\s*([\s\S]*?)```", "宽松匹配"),
    ];

    for (pattern_str, name) in patterns {
        println!("测试正则: {}", name);
        let pattern = Regex::new(pattern_str).unwrap();

        for (i, response) in test_responses.iter().enumerate() {
            println!("  格式{}: ", i + 1);
            if let Some(caps) = pattern.captures(response) {
                println!("    ✅ 匹配成功");
                println!("    文件名: {}", caps[1].trim());
                println!("    代码长度: {} 字符", caps[2].trim().len());
            } else {
                println!("    ❌ 未匹配");
            }
        }
        println!();
    }

    // 实际测试 Hermes
    println!("=== 实际 Hermes 测试 ===\n");

    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    let agent_config = hermes_agent::agent_builder::build_agent_config(&gateway_config, &model, Some("test"));
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    // 明确要求使用 ### FILE: 格式
    let request = r#"创建一个 Rust 程序计算斐波那契数列前10个数。

请严格使用以下格式输出代码：

### FILE: fibonacci.rs
```rust
// 代码内容
```

### FILE: main.rs
```rust
// 代码内容
```

不要使用其他格式，只使用 ### FILE: 文件名 格式。"#;

    let messages = vec![Message::user(request)];
    let result = agent_loop.run(messages, None).await.unwrap();

    let response = result.messages.iter()
        .rev()
        .find_map(|m| {
            if m.role == hermes_core::MessageRole::Assistant {
                m.content.clone()
            } else {
                None
            }
        })
        .unwrap_or_default();

    println!("AI 响应:\n{}\n", response);

    // 使用宽松正则提取
    let pattern = Regex::new(r"###.*?FILE:\s*([^\n]+)\s*```[\w]*\s*([\s\S]*?)```").unwrap();

    println!("提取文件:");
    for caps in pattern.captures_iter(&response) {
        println!("  文件: {}", caps[1].trim());
        println!("  代码预览: {}...", caps[2].trim().lines().next().unwrap_or(""));
    }
}