//! ERP+MES+WMS 快速测试（优化配置）
//!
//! 使用更短超时 + 更大批次 + 精简需求

use swarm_engine::ComplexGoalExecutor;
use std::fs;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ERP+MES+WMS 快速测试（优化配置）");
    println!("========================================\n");

    // 准备工作目录
    let work_dir = "D:/tmp/erp-quick";
    fs::create_dir_all(work_dir).ok();

    println!("【需求】创建 5 个核心模块文件：");
    println!("  1. ERP: finance.ts (财务)");
    println!("  2. MES: production.ts (生产)");
    println!("  3. WMS: inventory.ts (库存)");
    println!("  4. App架构: app-design.md");
    println!("  5. 桌面端: desktop.md\n");

    // 创建执行器（优化配置）
    let executor = ComplexGoalExecutor::new(
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd",
        work_dir,
    )
    .with_batch_size(2)    // 每批 2 个（避免超时）
    .with_timeout(60);     // 60 秒超时

    // 检查是否有可恢复的进度
    if executor.has_progress() {
        println!("发现进度文件，尝试恢复...");
        if let Ok(progress) = executor.load_progress() {
            println!("  已完成: {} 个任务", progress.completed.len());
            println!("  待执行: {} 个任务", progress.pending.len());
        }
    }

    println!("========================================");
    println!("【阶段 1：规划】");
    println!("========================================\n");

    // 精简需求
    let requirement = "创建5个文件:
1. finance.ts - ERP财务模块(总账/应收应付/报表)
2. production.ts - MES生产模块(工单/排程/执行)
3. inventory.ts - WMS库存模块(库存查询/盘点)
4. app-design.md - Flutter App架构设计
5. desktop.md - WinForm桌面端设计";

    let plan_result = executor.plan(requirement).await;

    match plan_result {
        Ok(plan) => {
            println!("✅ 规划成功！任务数: {}", plan.subtasks.len());
            for (i, task) in plan.subtasks.iter().enumerate() {
                println!("  {}. {}", i + 1, task.description.chars().take(40).collect::<String>());
            }
            println!("");

            println!("========================================");
            println!("【阶段 2：分解】");
            println!("========================================\n");

            let mut graph = executor.decompose(&plan, work_dir).await;
            println!("  GoalGraph: {} 个节点", graph.len());

            println!("========================================");
            println!("【阶段 3：执行】(批量执行，60秒超时)");
            println!("========================================\n");

            let results = executor.execute_graph(&mut graph).await;

            // 统计
            let success = results.iter().filter(|(_, o)| matches!(o, swarm_engine::GoalOutcome::Converged { .. })).count();
            println!("\n执行结果: {} / {} 成功", success, results.len());

            println!("========================================");
            println!("【阶段 4：验证】");
            println!("========================================\n");

            // 检查文件
            let files = [
                format!("{}/finance.ts", work_dir),
                format!("{}/production.ts", work_dir),
                format!("{}/inventory.ts", work_dir),
                format!("{}/app-design.md", work_dir),
                format!("{}/desktop.md", work_dir),
            ];

            let mut total_lines = 0;
            let mut success_count = 0;

            for file in &files {
                if std::path::Path::new(file).exists() {
                    let content = fs::read_to_string(file).unwrap_or_default();
                    let lines = content.lines().count();
                    println!("  ✅ {} - {} 行", file.split('/').last().unwrap_or(file), lines);
                    total_lines += lines;
                    success_count += 1;
                } else {
                    println!("  ❌ {} - 未创建", file.split('/').last().unwrap_or(file));
                }
            }

            println!("\n总计: {} 个文件, {} 行代码", success_count, total_lines);

            println!("========================================");
            if success_count >= 3 && total_lines >= 200 {
                println!("✅ 测试成功！系统闭环验证通过");
            } else if success_count >= 1 {
                println!("⚠️ 部分成功，需继续优化");
            } else {
                println!("❌ 需要进一步调试");
            }
            println!("========================================");
        },
        Err(e) => {
            println!("❌ 规划失败: {}", e);
        }
    }
}