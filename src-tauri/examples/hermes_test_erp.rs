//! 使用 Hermes Native Executive Agent 测试 ERP 项目
//! Hermes AI 将分析代码、编写测试、验证项目

use hermes_core::Message;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("=== Hermes Native 测试 ERP 项目 ===\n");

    let project_dir = PathBuf::from("D:/smallProject");

    // 配置 Hermes
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    let agent_config = hermes_agent::agent_builder::build_agent_config(
        &gateway_config,
        &model,
        Some("erp-tester"),
    );
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    println!("Hermes AgentLoop 已就绪\n");

    // ========== 任务1：代码审查 ==========
    println!("========== 任务1：Hermes 审查生成的代码 ==========\n");

    // 读取生成的代码文件
    let backend_program =
        fs::read_to_string(project_dir.join("backend/ErpSystem/Program.cs")).unwrap_or_default();
    let backend_product =
        fs::read_to_string(project_dir.join("backend/ErpSystem/Models/Product.cs"))
            .unwrap_or_default();
    let backend_controller =
        fs::read_to_string(project_dir.join("backend/ErpSystem/Controllers/ProductsController.cs"))
            .unwrap_or_default();
    let frontend_vue =
        fs::read_to_string(project_dir.join("frontend/src/views/ProductManagement.vue"))
            .unwrap_or_default();

    let review_prompt = format!(
        r#"你是代码审查专家。请审查以下生成的 ERP 项目代码，指出潜在问题和改进建议。

## 后端 Program.cs
```csharp
{}
```

## 后端 Product.cs
```csharp
{}
```

## 后端 ProductsController.cs
```csharp
{}
```

## 前端 ProductManagement.vue
```vue
{}
```

请审查以下方面：
1. 代码规范和最佳实践
2. 安全性问题
3. 性能优化建议
4. 功能完整性
5. 前后端接口一致性

输出格式：
### 审查结果

**代码质量评分**: X/10

**发现的问题**:
1. ...
2. ...

**改进建议**:
1. ...
2. ...

**总体评价**: ...

"#,
        backend_program, backend_product, backend_controller, frontend_vue
    );

    let messages = vec![Message::user(&review_prompt)];
    println!("执行代码审查...\n");

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

    println!("代码审查结果:\n{}\n", response);

    // ========== 任务2：编写单元测试 ==========
    println!("========== 任务2：Hermes 编写单元测试 ==========\n");

    let test_prompt = r#"你是测试开发专家。请为 ERP 系统编写单元测试代码。

### FILE: backend/ErpSystem.Tests/ProductsControllerTests.cs
编写 NUnit 测试类，测试 ProductsController 的 CRUD 操作。

要求：
1. 使用 xUnit 或 NUnit 测试框架
2. 测试 GetAll、GetById、Create、Update、Delete 方法
3. 使用 Mock 模拟数据库上下文
4. 包含正常和异常场景测试
5. 中文注释

### FILE: frontend/tests/product-api.test.ts
编写 Vitest 测试文件，测试前端 API 调用。

要求：
1. 使用 Vitest 测试框架
2. 测试 API 请求和响应处理
3. Mock axios 响应
4. 中文注释"#;

    let messages = vec![Message::user(test_prompt)];
    println!("执行测试代码生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let test_response = result
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

    println!("测试代码生成完成 ({} 字符)\n", test_response.len());

    // 写入测试文件
    let pattern =
        Regex::new(r"###\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```").unwrap();
    let mut tests_written = 0;

    for caps in pattern.captures_iter(&test_response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();

        let full_path = project_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        println!("✅ 写入测试文件: {}", file_path);
        tests_written += 1;
    }
    println!("测试文件数: {}\n", tests_written);

    // ========== 任务3：验证项目结构完整性 ==========
    println!("========== 任务3：Hermes 验证项目完整性 ==========\n");

    // 收集所有文件列表
    let all_files = collect_all_files(&project_dir);
    let file_list = all_files
        .iter()
        .filter(|f| {
            f.ends_with(".cs") || f.ends_with(".vue") || f.ends_with(".ts") || f.ends_with(".json")
        })
        .map(|f| f.replace("D:/smallProject/", ""))
        .collect::<Vec<_>>()
        .join("\n");

    let verify_prompt = format!(
        r#"你是项目经理。请验证以下 ERP 项目结构是否完整。

## 当前项目文件列表
{}

## 标准 ERP 项目应包含的文件
后端：
- Program.cs (程序入口)
- appsettings.json (配置)
- Models/*.cs (实体模型)
- Controllers/*.cs (API控制器)
- Services/*.cs (业务逻辑) - 可选
- Tests/*.cs (单元测试)

前端：
- package.json
- vite.config.ts
- src/main.ts
- src/App.vue
- src/router/index.ts
- src/api/index.ts
- src/views/*.vue

请检查并输出：
### 验证结果

**已完成的文件**:
- ✅ ...

**缺失的文件**:
- ❌ ...

**建议补充**: ...

**完整性评分**: X%

"#,
        file_list
    );

    let messages = vec![Message::user(&verify_prompt)];
    println!("执行项目完整性验证...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let verify_response = result
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

    println!("项目完整性验证结果:\n{}\n", verify_response);

    // ========== 任务4：生成测试报告 ==========
    println!("========== 任务4：Hermes 生成测试报告 ==========\n");

    let report_prompt = format!(
        r#"你是测试报告撰写专家。请根据以下审查和验证结果，生成完整的测试报告。

## 代码审查结果
{}

## 项目完整性验证
{}

请生成格式化的测试报告：

### FILE: docs/test-report.md

# ERP 项目测试报告

## 1. 测试概述
- 测试时间
- 测试范围
- 测试方法

## 2. 代码审查结果
- 后端代码评分
- 前端代码评分
- 发现的问题列表
- 改进建议

## 3. 项目结构验证
- 完整性评分
- 缺失项列表
- 补充建议

## 4. 测试执行结果
- 单元测试覆盖情况
- 功能测试结果

## 5. 总体评价
- 项目可行性评估
- 下一步建议

## 6. 结论
- 是否可以交付
"#,
        response.lines().take(20).collect::<Vec<_>>().join("\n"),
        verify_response
            .lines()
            .take(20)
            .collect::<Vec<_>>()
            .join("\n")
    );

    let messages = vec![Message::user(&report_prompt)];
    println!("执行测试报告生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let report_response = result
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

    println!("测试报告生成完成\n");

    // 写入测试报告
    for caps in pattern.captures_iter(&report_response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();
        let full_path = project_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        println!("✅ 写入测试报告: {}", file_path);
    }

    println!("\n=== Hermes Native 测试完成 ===");
    println!("测试报告位置: D:/smallProject/docs/test-report.md");
}

fn collect_all_files(dir: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                files.extend(collect_all_files(&path));
            }
        }
    }
    files
}
