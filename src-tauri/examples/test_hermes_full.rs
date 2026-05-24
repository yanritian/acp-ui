//! 完整测试 Hermes Native 执行 + 文件写入
//! 测试 Executive Agent 的完整工作流程

use std::sync::Arc;
use std::fs;
use regex::Regex;
use hermes_core::Message;

#[tokio::main]
async fn main() {
    println!("=== Hermes Native 完整测试 ===\n");

    // 工作目录
    let workspace = std::path::PathBuf::from("D:/dingsun/acp-ui/test-workspace");

    // 创建工作目录
    if !workspace.exists() {
        fs::create_dir_all(&workspace).unwrap();
    }
    println!("工作目录: {}", workspace.display());

    // 配置 Hermes
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();

    // 加载配置
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    // 构建 Agent
    let agent_config = hermes_agent::agent_builder::build_agent_config(
        &gateway_config, &model, Some("test"),
    );
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    println!("AgentLoop 构建完成\n");

    // 编码需求
    let request = r#"请创建以下两个 Rust 文件：

1. ### FILE: fibonacci.rs
   - 计算斐波那契数列前10个数
   - 使用迭代方式实现
   - 打印结果

2. ### FILE: main.rs
   - 调用 fibonacci.rs 中的函数
   - 主程序入口

代码要简洁，用中文注释。"#;

    println!("需求: {}\n", request.lines().take(3).collect::<Vec<_>>().join("\n"));

    // 执行
    println!("执行 AgentLoop...\n");
    let messages = vec![Message::user(request)];

    let result = agent_loop.run(messages, None).await.unwrap();

    // 提取响应
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

    // 提取文件并写入
    println!("--- 提取并写入文件 ---");
    // 支持多种格式: ### FILE: xxx 或 ### 📄 `xxx`
    let pattern = Regex::new(r"###\s*(?:FILE:|📄\s*`)([^\n`]+)\s*```[\w]*\s*([\s\S]*?)```").unwrap();
    let mut files_written = 0;

    for caps in pattern.captures_iter(&response) {
        let file_path = caps[1].trim();
        let code = caps[2].trim();

        let full_path = workspace.join(file_path);

        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }

        fs::write(&full_path, code).unwrap();
        let lines = code.lines().count();

        println!("✅ 写入文件: {} ({})", file_path, lines);
        files_written += 1;
    }

    println!("\n共写入 {} 个文件\n", files_written);

    // 验证文件
    println!("--- 验证生成的文件 ---");
    for entry in fs::read_dir(&workspace).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(&path).unwrap();
            println!("\n文件: {}", path.display());
            println!("内容预览:\n{}\n", content.lines().take(5).collect::<Vec<_>>().join("\n"));
        }
    }

    // 尝试编译
    println!("--- 尝试编译 ---");
    let cargo_toml = workspace.join("Cargo.toml");
    fs::write(&cargo_toml, r#"
[package]
name = "test-fibonacci"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).unwrap();

    println!("写入 Cargo.toml\n");

    // 编译
    let compile_result = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&workspace)
        .output();

    match compile_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ 编译成功!");
                println!("{}", String::from_utf8_lossy(&output.stdout));

                // 运行
                println!("\n--- 运行程序 ---");
                let run_result = std::process::Command::new("cargo")
                    .args(["run", "--release"])
                    .current_dir(&workspace)
                    .output();

                if let Ok(run_output) = run_result {
                    println!("程序输出:\n{}", String::from_utf8_lossy(&run_output.stdout));
                }
            } else {
                println!("❌ 编译失败:\n{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("无法执行编译: {}", e);
        }
    }

    println!("\n=== 测试完成 ===");
}