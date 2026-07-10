//! 完整工作流程测试 - Hermes Native 执行 + 文件写入 + 编译运行

use hermes_core::Message;
use regex::Regex;
use std::fs;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Hermes Native 完整工作流程测试 ===\n");

    // 工作目录
    let workspace = std::path::PathBuf::from("D:/dingsun/acp-ui/test-workspace/src");

    // 创建工作目录
    fs::create_dir_all(&workspace).unwrap();
    println!("工作目录: {}", workspace.display());

    // 配置 Hermes
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();

    // 加载配置并构建 Agent
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();
    let agent_config =
        hermes_agent::agent_builder::build_agent_config(&gateway_config, &model, Some("test"));
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    println!("AgentLoop 构建完成\n");

    // 编码需求
    let request = r#"请创建一个 Rust 程序，计算斐波那契数列前10个数并打印结果。

严格使用以下格式：

### FILE: fibonacci.rs
```rust
// 计算斐波那契数列的函数
```

### FILE: main.rs
```rust
// 主程序入口
```

要求：
1. fibonacci.rs 包含生成斐波那契数列的函数
2. main.rs 调用 fibonacci 函数并打印结果
3. 使用中文注释"#;

    println!(
        "需求: {}\n",
        request.lines().take(2).collect::<Vec<_>>().join("\n")
    );

    // 执行
    println!("执行 AgentLoop...\n");
    let messages = vec![Message::user(request)];
    let result = agent_loop.run(messages, None).await.unwrap();

    println!("✅ AgentLoop 执行成功 (turns: {})\n", result.total_turns);

    // 提取响应
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

    println!(
        "AI 响应:\n{}\n",
        response.lines().take(20).collect::<Vec<_>>().join("\n")
    );

    // 提取文件并写入
    println!("--- 提取并写入文件 ---");
    let pattern =
        Regex::new(r"###\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]*```[\w]*\s*([\s\S]*?)```").unwrap();
    let mut files_written = 0;

    for caps in pattern.captures_iter(&response) {
        let file_path = caps[1].trim();
        let code = caps[2].trim();

        let full_path = workspace.join(file_path);
        fs::write(&full_path, code).unwrap();
        let lines = code.lines().count();

        println!("✅ 写入: {} ({})", file_path, lines);
        files_written += 1;
    }

    if files_written == 0 {
        println!("❌ 未提取到任何文件");
        return;
    }

    // 创建 Cargo.toml
    let cargo_toml = workspace.parent().unwrap().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"[package]
name = "fibonacci-test"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();
    println!("✅ 写入 Cargo.toml\n");

    // 编译
    println!("--- 编译程序 ---");
    let compile_result = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(workspace.parent().unwrap())
        .output();

    match compile_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 编译成功!\n");

                // 运行
                println!("--- 运行程序 ---");
                let run_result = std::process::Command::new("cargo")
                    .args(["run", "--release"])
                    .current_dir(workspace.parent().unwrap())
                    .output();

                if let Ok(run_output) = run_result {
                    if run_output.status.success() {
                        println!("✅ 运行成功!");
                        println!(
                            "\n程序输出:\n{}\n",
                            String::from_utf8_lossy(&run_output.stdout)
                        );
                    } else {
                        println!("运行失败:\n{}", String::from_utf8_lossy(&run_output.stderr));
                    }
                }
            } else {
                println!("❌ 编译失败:\n{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => println!("无法执行编译: {}", e),
    }

    println!("=== 测试完成 ===");
}
