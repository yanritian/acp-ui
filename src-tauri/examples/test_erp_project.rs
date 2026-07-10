//! 完整 ERP 项目开发流程测试
//! 1. 生成详细需求文档
//! 2. 生成项目代码
//! 3. 校验代码正确性

use hermes_core::Message;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== ERP 项目开发流程测试 ===\n");

    let project_dir = PathBuf::from("D:/smallProject");
    fs::create_dir_all(&project_dir).unwrap();
    println!("项目目录: {}", project_dir.display());

    // 配置 Hermes
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    let agent_config =
        hermes_agent::agent_builder::build_agent_config(&gateway_config, &model, Some("erp-dev"));
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    println!("Hermes AgentLoop 已就绪\n");

    // ========== 第一步：生成 ERP 详细需求文档 ==========
    println!("========== 第一步：生成 ERP 详细需求文档 ==========\n");

    let requirements_prompt = r#"你是一位资深的企业管理系统架构师。请为一个中小企业 ERP 系统编写详细的需求文档。

系统技术栈：
- 后端：C# (.NET 8) + ASP.NET Core Web API
- 前端：Vue 3 + TypeScript + Vite
- 数据库：SQL Server
- 运行平台：Windows Desktop App (可选 Electron 或 WPF)

请输出完整的需求文档，包含以下章节：

### FILE: docs/requirements.md

内容应包括：
1. **系统概述** - 项目背景、目标用户、核心价值
2. **功能模块** - 详细列出各模块功能
   - 用户管理模块
   - 产品管理模块
   - 采购管理模块
   - 销售管理模块
   - 库存管理模块
   - 财务管理模块
   - 报表分析模块
3. **技术架构** - 系统架构图说明、技术选型理由
4. **数据库设计** - 核心表结构设计
5. **接口设计** - RESTful API 设计规范
6. **非功能需求** - 性能、安全、可用性要求

请用 Markdown 格式，中文编写，内容详实专业。"#;

    let messages = vec![Message::user(requirements_prompt)];
    println!("执行需求文档生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
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

    println!("需求文档生成完成 ({} 字符)\n", response.len());

    // 提取并写入需求文档
    let pattern =
        Regex::new(r"###\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]*```[\w]*\s*([\s\S]*?)```").unwrap();
    let mut docs_written = 0;

    for caps in pattern.captures_iter(&response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();

        // 如果是 md 文件，不需要代码块标记
        if file_path.ends_with(".md") {
            // 移除可能的 markdown 代码块标记
            let clean_content = content.replace("```markdown", "").replace("```", "");
            let full_path = project_dir.join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full_path, clean_content).unwrap();
            println!("✅ 写入需求文档: {}", file_path);
            docs_written += 1;
        }
    }

    // 如果正则没匹配到，直接写入整个响应
    if docs_written == 0 {
        let req_path = project_dir.join("docs/requirements.md");
        fs::create_dir_all(req_path.parent().unwrap()).unwrap();
        fs::write(&req_path, &response).unwrap();
        println!("✅ 写入需求文档: docs/requirements.md");
    }

    // 显示需求文档预览
    let req_content =
        fs::read_to_string(project_dir.join("docs/requirements.md")).unwrap_or_default();
    println!(
        "\n需求文档预览:\n{}\n",
        req_content.lines().take(30).collect::<Vec<_>>().join("\n")
    );

    // ========== 第二步：生成项目代码结构 ==========
    println!("========== 第二步：生成项目代码结构 ==========\n");

    // 先生成后端项目结构
    let backend_prompt = r#"你是 C# 后端开发专家。根据之前的 ERP 需求文档，生成 ASP.NET Core Web API 后端项目代码。

项目名称：ErpSystem
目录结构：backend/ErpSystem/

请生成以下核心文件：

### FILE: backend/ErpSystem/ErpSystem.csproj
.NET 8 项目配置文件

### FILE: backend/ErpSystem/Program.cs
程序入口，配置依赖注入、Swagger、数据库连接

### FILE: backend/ErpSystem/Models/User.cs
用户实体模型

### FILE: backend/ErpSystem/Models/Product.cs
产品实体模型

### FILE: backend/ErpSystem/Models/Order.cs
订单实体模型

### FILE: backend/ErpSystem/Controllers/UsersController.cs
用户管理 API 控制器

### FILE: backend/ErpSystem/Controllers/ProductsController.cs
产品管理 API 控制器

### FILE: backend/ErpSystem/appsettings.json
配置文件（数据库连接字符串）

代码要求：
1. 使用最新 C# 语法特性
2. 包含完整的 CRUD 操作
3. 使用 Entity Framework Core
4. 包含 Swagger API 文档
5. 中文注释"#;

    let messages = vec![Message::user(backend_prompt)];
    println!("执行后端代码生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let backend_response = result
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

    println!("后端代码生成完成 ({} 字符)\n", backend_response.len());

    // 写入后端文件
    let mut backend_files = 0;
    for caps in pattern.captures_iter(&backend_response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();

        let full_path = project_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        println!("✅ 写入后端文件: {}", file_path);
        backend_files += 1;
    }
    println!("后端文件数: {}\n", backend_files);

    // 生成前端项目结构
    let frontend_prompt = r#"你是 Vue 3 前端开发专家。根据 ERP 需求文档，生成 Vue 3 + TypeScript 前端项目代码。

项目名称：erp-frontend
目录结构：frontend/erp-frontend/

请生成以下核心文件：

### FILE: frontend/erp-frontend/package.json
项目依赖配置

### FILE: frontend/erp-frontend/vite.config.ts
Vite 构建配置

### FILE: frontend/erp-frontend/src/main.ts
Vue 应用入口

### FILE: frontend/erp-frontend/src/App.vue
根组件，包含导航菜单

### FILE: frontend/erp-frontend/src/views/ProductManagement.vue
产品管理页面组件

### FILE: frontend/erp-frontend/src/views/OrderManagement.vue
订单管理页面组件

### FILE: frontend/erp-frontend/src/api/index.ts
API 请求封装

### FILE: frontend/erp-frontend/src/router/index.ts
路由配置

代码要求：
1. 使用 Vue 3 Composition API
2. TypeScript 类型定义
3. Element Plus UI 组件库
4. Axios HTTP 请求
5. 中文界面"#;

    let messages = vec![Message::user(frontend_prompt)];
    println!("执行前端代码生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let frontend_response = result
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

    println!("前端代码生成完成 ({} 字符)\n", frontend_response.len());

    // 写入前端文件
    let mut frontend_files = 0;
    for caps in pattern.captures_iter(&frontend_response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();

        let full_path = project_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        println!("✅ 写入前端文件: {}", file_path);
        frontend_files += 1;
    }
    println!("前端文件数: {}\n", frontend_files);

    // ========== 第三步：代码校验 ==========
    println!("========== 第三步：代码校验 ==========\n");

    let mut validation_results = Vec::new();

    // 校验后端 C# 代码
    if let Ok(content) = fs::read_to_string(project_dir.join("backend/ErpSystem/Program.cs")) {
        let checks = [
            (
                "包含 WebApplication.CreateBuilder",
                content.contains("WebApplication.CreateBuilder"),
            ),
            (
                "包含 Swagger",
                content.contains("Swagger") || content.contains("AddSwaggerGen"),
            ),
            ("包含 Controller 映射", content.contains("MapControllers")),
            ("包含 Run", content.contains(".Run()")),
        ];
        println!("Program.cs 校验:");
        for (name, passed) in checks {
            println!("  {} {}", if passed { "✅" } else { "❌" }, name);
            validation_results.push(format!(
                "Program.cs - {}: {}",
                name,
                if passed { "通过" } else { "失败" }
            ));
        }
    }

    if let Ok(content) = fs::read_to_string(project_dir.join("backend/ErpSystem/Models/Product.cs"))
    {
        let checks = [
            ("包含 class 定义", content.contains("public class")),
            ("包含属性", content.contains("public")),
            ("包含 Id", content.contains("Id")),
            ("包含 Name", content.contains("Name")),
        ];
        println!("Product.cs 校验:");
        for (name, passed) in checks {
            println!("  {} {}", if passed { "✅" } else { "❌" }, name);
            validation_results.push(format!(
                "Product.cs - {}: {}",
                name,
                if passed { "通过" } else { "失败" }
            ));
        }
    }

    // 校验前端 Vue 代码
    if let Ok(content) = fs::read_to_string(project_dir.join("frontend/erp-frontend/package.json"))
    {
        let checks = [
            ("包含 vue", content.contains("vue")),
            ("包含 vite", content.contains("vite")),
            ("包含 typescript", content.contains("typescript")),
            ("包含 element-plus", content.contains("element-plus")),
        ];
        println!("package.json 校验:");
        for (name, passed) in checks {
            println!("  {} {}", if passed { "✅" } else { "❌" }, name);
            validation_results.push(format!(
                "package.json - {}: {}",
                name,
                if passed { "通过" } else { "失败" }
            ));
        }
    }

    if let Ok(content) = fs::read_to_string(project_dir.join("frontend/erp-frontend/src/main.ts")) {
        let checks = [
            ("包含 createApp", content.contains("createApp")),
            ("包含 App", content.contains("App")),
            ("包含 mount", content.contains("mount")),
        ];
        println!("main.ts 校验:");
        for (name, passed) in checks {
            println!("  {} {}", if passed { "✅" } else { "❌" }, name);
            validation_results.push(format!(
                "main.ts - {}: {}",
                name,
                if passed { "通过" } else { "失败" }
            ));
        }
    }

    // ========== 总结 ==========
    println!("\n========== 项目生成总结 ==========\n");

    let total_files = count_all_files(&project_dir);
    println!("项目目录: {}", project_dir.display());
    println!("总文件数: {}", total_files);
    println!("校验项目数: {}", validation_results.len());
    let passed = validation_results
        .iter()
        .filter(|r| r.contains("通过"))
        .count();
    println!("通过项目: {}", passed);

    // 显示文件结构
    println!("\n项目文件结构:");
    show_file_tree(&project_dir, 0);

    println!("\n=== ERP 项目开发流程测试完成 ===");
}

fn count_all_files(dir: &PathBuf) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_all_files(&path);
            }
        }
    }
    count
}

fn show_file_tree(dir: &PathBuf, depth: usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();
            let indent = "  ".repeat(depth);

            if path.is_file() {
                println!("{}📄 {}", indent, name);
            } else if path.is_dir() && depth < 3 {
                println!("{}📁 {}", indent, name);
                show_file_tree(&path, depth + 1);
            }
        }
    }
}
