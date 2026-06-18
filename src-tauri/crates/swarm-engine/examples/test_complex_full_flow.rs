//! ComplexGoalExecutor 完整流程测试
//!
//! 测试复杂需求的完整闭环：
//! 1. 规划阶段：AI 分析需求并生成任务列表
//! 2. 分解阶段：转换为 GoalGraph
//! 3. 执行阶段：逐个执行子任务
//! 4. 验证阶段：检查所有文件

use swarm_engine::{
    ComplexGoalExecutor, ImplementationPlan,
};
use std::fs;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ComplexGoalExecutor 完整流程测试");
    println!("========================================\n");

    // 清理并准备目录
    let work_dir = "D:/tmp/complex-test";
    fs::create_dir_all(work_dir).ok();
    fs::create_dir_all(format!("{}/src", work_dir)).ok();

    println!("【需求】");
    println!("创建 MES 系统的核心模块：");
    println!("  - 生产计划模块");
    println!("  - 质量管理模块");
    println!("  - 库存管理模块\n");

    // 创建执行器
    let executor = ComplexGoalExecutor::new(
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd",
        work_dir,
    );

    println!("========================================");
    println!("【阶段 1：规划】分析需求并生成任务列表");
    println!("========================================\n");

    let requirement = "创建 MES 系统核心模块：production.ts (生产计划), quality.ts (质量管理), inventory.ts (库存管理)";

    let plan_result = executor.plan(requirement).await;

    match plan_result {
        Ok(plan) => {
            println!("✅ 规划成功！");
            println!("  分析: {}", plan.analysis.chars().take(50).collect::<String>());
            println!("  任务数量: {}", plan.subtasks.len());
            println!("  预估步骤: {}\n", plan.estimated_steps);

            for task in &plan.subtasks {
                println!("  任务 {}: {}", task.id, task.description.chars().take(40).collect::<String>());
                if !task.dependencies.is_empty() {
                    println!("    依赖: {:?}", task.dependencies);
                }
            }
            println!("");

            // ========================================
            // 阶段 2：分解
            // ========================================
            println!("========================================");
            println!("【阶段 2：分解】转换为 GoalGraph");
            println!("========================================\n");

            let mut graph = executor.decompose(&plan, format!("{}/src/", work_dir).as_str()).await;
            println!("  GoalGraph 节点数: {}", graph.len());

            let order = graph.topological_order();
            println!("  执行顺序: {:?}\n", order);

            // ========================================
            // 阶段 3：执行
            // ========================================
            println!("========================================");
            println!("【阶段 3：执行】逐个执行子任务");
            println!("========================================\n");

            let results = executor.execute_graph(&mut graph).await;

            println!("执行结果:");
            for (id, outcome) in &results {
                match outcome {
                    swarm_engine::GoalOutcome::Converged { .. } => {
                        println!("  ✅ {} - 成功", id);
                    },
                    swarm_engine::GoalOutcome::Failed { reason, .. } => {
                        println!("  ❌ {} - 失败: {}", id, reason.chars().take(30).collect::<String>());
                    },
                    _ => println!("  ⚠️ {} - {:?}", id, outcome),
                }
            }
            println!("");

            // ========================================
            // 阶段 4：验证
            // ========================================
            println!("========================================");
            println!("【阶段 4：验证】检查生成文件");
            println!("========================================\n");

            let files_to_check = [
                format!("{}/src/production.ts", work_dir),
                format!("{}/src/quality.ts", work_dir),
                format!("{}/src/inventory.ts", work_dir),
            ];

            let mut success_count = 0;
            for file in &files_to_check {
                if std::path::Path::new(file).exists() {
                    let content = fs::read_to_string(file).unwrap_or_default();
                    println!("  ✅ {} - {} bytes", file, content.len());
                    success_count += 1;
                } else {
                    println!("  ❌ {} - 不存在", file);
                }
            }
            println!("");

            // ========================================
            // 最终结论
            // ========================================
            println!("========================================");
            println!("【最终结论】");
            println!("========================================");

            if success_count == files_to_check.len() {
                println!("✅ ComplexGoalExecutor 完整闭环成功！");
                println!("  规划 → 分解 → 执行 → 验证 全流程通过");
            } else {
                println!("⚠️ 部分 success_count = {}/{}", success_count, files_to_check.len());
            }
        },
        Err(e) => {
            println!("❌ 规划失败: {}", e);
        }
    }

    println!("\n========================================");
    println!("测试完成");
    println!("========================================");
}