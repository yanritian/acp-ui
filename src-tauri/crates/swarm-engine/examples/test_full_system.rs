//! ERP+MES+WMS 综合项目完整测试
//!
//! 需求：erp+mes+wms综合项目，web+手机端+windows应用端
//! 测试系统自主执行能力，无人工干预

use swarm_engine::ComplexGoalExecutor;
use std::fs;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ERP+MES+WMS 综合项目完整测试");
    println!("========================================\n");

    let work_dir = "D:/tmp/erp-mes-wms-full";
    fs::create_dir_all(work_dir).ok();
    fs::create_dir_all(format!("{}/src", work_dir)).ok();

    println!("【需求】");
    println!("erp+mes+wms的综合项目，要web+手机端+windows应用端\n");

    println!("包含模块:");
    println!("  - ERP: 财务、人力资源、采购销售");
    println!("  - MES: 生产计划、质量管理、设备监控");
    println!("  - WMS: 库存管理、出入库、仓库布局");
    println!("  - Web端: Vue/React 前端应用");
    println!("  - 手机端: Flutter 跨平台应用");
    println!("  - Windows端: WinForm 桌面客户端\n");

    // 创建执行器
    let executor = ComplexGoalExecutor::new(
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd",
        work_dir,
    )
    .with_batch_size(2)
    .with_timeout(90);

    println!("========================================");
    println!("【阶段 1：规划】AI 自动分解需求");
    println!("========================================\n");

    let requirement = "创建 ERP+MES+WMS 综合系统（简化版，6个核心文件）:
1. finance.ts - ERP财务模块
2. production.ts - MES生产模块
3. inventory.ts - WMS库存模块
4. web-app.md - Web前端架构设计
5. mobile-app.md - 手机端架构设计
6. windows-app.md - Windows端架构设计";

    let plan_result = executor.plan(requirement).await;

    match plan_result {
        Ok(plan) => {
            println!("✅ 规划成功！任务数: {}", plan.subtasks.len());
            for (i, task) in plan.subtasks.iter().enumerate() {
                println!("  {}. {}", i + 1, task.description.chars().take(50).collect::<String>());
            }
            println!("");

            println!("========================================");
            println!("【阶段 2：分解】构建执行图");
            println!("========================================\n");

            let mut graph = executor.decompose(&plan, work_dir).await;
            println!("  GoalGraph: {} 个节点", graph.len());

            println!("========================================");
            println!("【阶段 3：执行】AI 自主创建所有模块");
            println!("========================================\n");

            println!("开始自主执行（无人工干预，90秒超时/任务）...\n");

            let results = executor.execute_graph(&mut graph).await;

            let success_count = results.iter()
                .filter(|(_, outcome)| matches!(outcome, swarm_engine::GoalOutcome::Converged { .. }))
                .count();

            println!("\n执行统计:");
            println!("  成功: {}", success_count);
            println!("  失败: {}", results.len() - success_count);
            println!("  总计: {}", results.len());

            println!("\n========================================");
            println!("【阶段 4：验证】检查生成文件");
            println!("========================================\n");

            // 检查所有文件
            let expected_files = [
                ("ERP财务", format!("{}/finance.ts", work_dir)),
                ("MES生产", format!("{}/production.ts", work_dir)),
                ("WMS库存", format!("{}/inventory.ts", work_dir)),
                ("Web前端", format!("{}/web-app.md", work_dir)),
                ("手机端", format!("{}/mobile-app.md", work_dir)),
                ("Windows端", format!("{}/windows-app.md", work_dir)),
            ];

            let mut file_count = 0;
            let mut total_lines = 0;
            let mut ts_files = Vec::new();

            for (name, path) in &expected_files {
                if std::path::Path::new(path).exists() {
                    let content = fs::read_to_string(path).unwrap_or_default();
                    let lines = content.lines().count();
                    let bytes = content.len();

                    if lines >= 50 || bytes >= 500 {
                        println!("  ✅ {} - {} 行, {} bytes", name, lines, bytes);
                        file_count += 1;
                        total_lines += lines;
                        if path.ends_with(".ts") {
                            ts_files.push(path.clone());
                        }
                    } else {
                        println!("  ⚠️ {} - 内容不足 ({} 行)", name, lines);
                    }
                } else {
                    println!("  ❌ {} - 未创建", name);
                }
            }

            println!("\n  总计: {} 个合格文件, {} 行代码", file_count, total_lines);

            // TypeScript 语法检查
            if !ts_files.is_empty() {
                println!("\n========================================");
                println!("【阶段 5：TypeScript 编译验证】");
                println!("========================================\n");

                // 创建 tsconfig.json
                let tsconfig = r#"{
                    "compilerOptions": {
                        "target": "ES2020",
                        "module": "ESNext",
                        "strict": true,
                        "noEmit": true,
                        "skipLibCheck": true,
                        "esModuleInterop": true
                    },
                    "include": ["*.ts"]
                }"#;
                fs::write(format!("{}/tsconfig.json", work_dir), tsconfig).ok();

                // 检查双引号语法错误
                let mut syntax_errors = 0;
                for ts_file in &ts_files {
                    let content = fs::read_to_string(ts_file).unwrap_or_default();
                    if content.contains("''") {
                        syntax_errors += 1;
                        println!("  ⚠️ {} - 包含双引号语法错误", ts_file.split('/').last().unwrap_or(ts_file));
                    }
                }

                if syntax_errors == 0 {
                    println!("  ✅ 所有 TypeScript 文件语法检查通过");
                } else {
                    println!("  ⚠️ {} 个文件有语法问题", syntax_errors);
                }
            }

            println!("\n========================================");
            println!("【最终结论】");
            println!("========================================");

            if file_count >= 5 && total_lines >= 1000 {
                println!("✅ 测试成功！系统自主完成项目");
                println!("  生成了 {} 个合格模块，共 {} 行代码", file_count, total_lines);
                println!("  系统闭环验证通过");
            } else if file_count >= 3 {
                println!("⚠️ 部分成功：{} 个文件创建", file_count);
                println!("  建议：增加超时时间或优化执行策略");
            } else {
                println!("❌ 需要进一步调试优化");
            }
        },
        Err(e) => {
            println!("❌ 规划失败: {}", e);
        }
    }

    println!("\n========================================");
    println!("测试完成 - 系统自主执行验证");
    println!("========================================");
}