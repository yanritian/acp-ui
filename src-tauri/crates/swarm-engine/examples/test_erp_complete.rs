//! ERP+MES+WMS + 手机App + WinForm 桌面 完整系统测试
//!
//! 验证 ComplexGoalExecutor 处理真实复杂需求的能力

use swarm_engine::ComplexGoalExecutor;
use std::fs;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ERP+MES+WMS 综合系统完整测试");
    println!("========================================\n");

    // 准备工作目录
    let work_dir = "D:/tmp/erp-mes-wms-complete";
    fs::create_dir_all(work_dir).ok();
    fs::create_dir_all(format!("{}/src", work_dir)).ok();

    println!("【需求】");
    println!("给我做一个erp+mes+wms的综合系统与手机app，");
    println!("同时还要有winform做windows桌面系统\n");

    println!("包含模块:");
    println!("  ERP: 财务管理、人力资源、采购销售");
    println!("  MES: 生产计划、质量管理、设备监控");
    println!("  WMS: 库存管理、出入库、仓库布局");
    println!("  手机App: Flutter跨平台应用");
    println!("  WinForm: Windows桌面客户端\n");

    // 创建执行器
    let executor = ComplexGoalExecutor::new(
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd",
        work_dir,
    );

    println!("========================================");
    println!("【阶段 1：规划】AI 自动分解复杂需求");
    println!("========================================\n");

    let requirement = "创建ERP+MES+WMS综合系统:
1. ERP模块: 财务.ts, 人力.ts, 采购.ts
2. MES模块: 生产.ts, 质量.ts, 设备.ts
3. WMS模块: 库存.ts, 出入库.ts
4. 手机App: Flutter架构设计 flutter-app.md
5. WinForm桌面: winform-design.md
共10个文件";

    let plan_result = executor.plan(requirement).await;

    match plan_result {
        Ok(plan) => {
            println!("✅ 规划成功！AI 自动分解任务：");
            println!("  分析预览: {}\n", plan.analysis.chars().take(60).collect::<String>());
            println!("  任务总数: {}", plan.subtasks.len());
            println!("  预估步骤: {}\n", plan.estimated_steps);

            // 显示任务列表
            println!("【任务列表】");
            for (i, task) in plan.subtasks.iter().enumerate() {
                println!("  {}. {} - 依赖: {:?}",
                    i + 1,
                    task.description.chars().take(40).collect::<String>(),
                    task.dependencies);
            }
            println!("");

            // ========================================
            // 阶段 2：分解
            // ========================================
            println!("========================================");
            println!("【阶段 2：分解】构建执行图");
            println!("========================================\n");

            let mut graph = executor.decompose(&plan, work_dir).await;
            let order = graph.topological_order();

            println!("  GoalGraph 构建完成");
            println!("  节点数: {}", graph.len());
            println!("  拓扑排序: 前5个 {:?}", order.iter().take(5).collect::<Vec<_>>());
            println!("");

            // ========================================
            // 阶段 3：执行
            // ========================================
            println!("========================================");
            println!("【阶段 3：执行】AI 自动创建所有模块");
            println!("========================================\n");

            println!("开始执行（带重试机制，超时60秒/任务）...\n");

            let results = executor.execute_graph(&mut graph).await;

            // 统计结果
            let success_count = results.iter()
                .filter(|(_, outcome)| matches!(outcome, swarm_engine::GoalOutcome::Converged { .. }))
                .count();

            println!("\n执行统计:");
            println!("  成功: {}", success_count);
            println!("  失败: {}", results.len() - success_count);
            println!("  总计: {}", results.len());
            println!("");

            // ========================================
            // 阶段 4：验证
            // ========================================
            println!("========================================");
            println!("【阶段 4：验证】检查生成文件");
            println!("========================================\n");

            // 检查关键文件
            let key_files = [
                ("ERP财务模块", format!("{}/财务.ts", work_dir)),
                ("MES生产模块", format!("{}/生产.ts", work_dir)),
                ("WMS库存模块", format!("{}/库存.ts", work_dir)),
                ("Flutter设计", format!("{}/flutter-app.md", work_dir)),
                ("WinForm设计", format!("{}/winform-design.md", work_dir)),
            ];

            let mut file_count = 0;
            let mut total_lines = 0;

            for (name, path) in &key_files {
                if std::path::Path::new(path).exists() {
                    let content = fs::read_to_string(path).unwrap_or_default();
                    let lines = content.lines().count();
                    println!("  ✅ {} - {} 行 ({} bytes)", name, lines, content.len());
                    file_count += 1;
                    total_lines += lines;
                } else {
                    println!("  ❌ {} - 未创建", name);
                }
            }

            println!("\n  总计: {} 个文件, {} 行代码\n", file_count, total_lines);

            // ========================================
            // 最终结论
            // ========================================
            println!("========================================");
            println!("【最终结论】");
            println!("========================================");

            if file_count >= 3 && total_lines >= 500 {
                println!("✅ ComplexGoalExecutor 成功完成复杂系统！");
                println!("  从需求 → 规划 → 分解 → 执行 → 验证 全流程闭环");
                println!("  生成了 {} 个核心模块，共 {} 行代码", file_count, total_lines);
            } else if file_count >= 1 {
                println!("⚠️ 部分成功：{} 个文件创建", file_count);
                println!("  建议：增加超时时间或减少单次任务复杂度");
            } else {
                println!("❌ 执行失败，需要进一步调试");
            }
        },
        Err(e) => {
            println!("❌ 规划阶段失败: {}", e);
        }
    }

    println!("\n========================================");
    println!("测试完成");
    println!("========================================");
}