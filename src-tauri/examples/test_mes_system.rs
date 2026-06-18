//! AI Worker Executor MES 系统需求测试
//!
//! 需求：创建 MES (制造执行系统) 基础模块

use swarm_engine::{
    Goal, CompletionCondition,
    ConditionEvaluator, ReconcileLoop, GoalOutcome,
    AIWorkerExecutor,
};
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("AI Worker Executor MES 系统测试");
    println!("========================================\n");

    // === 需求 ===
    println!("【需求】创建 MES 制造执行系统基础模块：");
    println!("  1. 在 D:/tmp/mes/ 创建项目结构");
    println!("  2. 创建 README.md 说明系统功能");
    println!("  3. 创建 src/production.ts 生产管理模块");
    println!("  4. 创建 src/inventory.ts 库存管理模块");
    println!("  5. 创建 src/quality.ts 质量管理模块");

    // 确保目录存在
    fs::create_dir_all("D:/tmp/mes/src").ok();

    // === Step 1: 创建Goal ===
    println!("\n【步骤 1】创建Goal");

    let mut goal = Goal::new(
        "create-mes-system",
        "创建 MES 制造执行系统基础模块：
        1. 在 D:/tmp/mes/ 目录创建项目结构
        2. 创建 README.md 文件，说明 MES 系统的核心功能模块
        3. 创建 src/production.ts 生产管理模块（包含 ProductionOrder, ProductionStatus 类型定义）
        4. 创建 src/inventory.ts 库存管理模块（包含 Material, Inventory 类型定义）
        5. 创建 src/quality.ts 质量管理模块（包含 QualityCheck, QualityStatus 类型定义）
        所有 TypeScript 文件需要有正确的类型定义和导出",
        CompletionCondition::FileCheck {
            path: "D:/tmp/mes/src/production.ts".to_string(),
            content_contains: Some("ProductionOrder".to_string()),
            max_size_bytes: None,
        },
    );
    goal.max_iterations = 2;

    println!("  Goal ID: {}", goal.id);
    println!("  Description: 创建 MES 系统基础模块");
    println!("  Condition: FileCheck(path='D:/tmp/mes/src/production.ts', contains='ProductionOrder')");

    // === Step 2: 执行 ===
    println!("\n【步骤 2】创建AIWorkerExecutor并执行");

    let executor = AIWorkerExecutor::new("claude-worker", "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd")
        .with_cwd("D:/tmp/mes");
    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    println!("  正在发送任务给 Claude Code...\n");

    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n  AI执行完成");
    println!("  Outcome: {:?}", outcome);

    // === Step 3: 验证 ===
    println!("\n【步骤 3】验证结果");

    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&goal.completion_condition);

    println!("  Goal Condition converged: {}", result.converged);

    // 检查所有文件
    println!("\n【文件检查】");

    let files = [
        ("D:/tmp/mes/README.md", "MES"),
        ("D:/tmp/mes/src/production.ts", "ProductionOrder"),
        ("D:/tmp/mes/src/inventory.ts", "Material"),
        ("D:/tmp/mes/src/quality.ts", "QualityCheck"),
    ];

    let mut all_exist = true;
    for (path, expected) in &files {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).unwrap();
            let contains = content.contains(expected);
            println!("  ✓ {} 存在 ({:.1} KB) - 包含 '{}': {}", path, content.len() as f64 / 1024.0, expected, contains);
            if !contains {
                all_exist = false;
            }
        } else {
            println!("  ❌ {} 不存在", path);
            all_exist = false;
        }
    }

    // === 结果 ===
    println!("\n========================================");
    println!("【最终结果】");
    println!("========================================\n");

    match outcome {
        GoalOutcome::Converged { iterations, .. } => {
            if all_exist {
                println!("✅ MES 系统测试成功！");
                println!("  • Claude Code 理解并执行了复杂系统需求");
                println!("  • Goal 在 {} 次迭代后收敛", iterations);
                println!("  • MES 系统基础模块创建成功");
                println!("  • 所有 TypeScript 类型定义文件生成");
            } else {
                println!("⚠️ Goal 收敛但部分文件缺失");
            }
        },
        GoalOutcome::Failed { reason, .. } => {
            println!("执行失败: {}", reason);
        },
        GoalOutcome::MaxIterReached { iterations, .. } => {
            println!("⚠️ 达到最大迭代次数 {}，但部分工作已完成", iterations);
            // 检查是否有文件生成
            if Path::new("D:/tmp/mes/src/production.ts").exists() {
                println!("  生产模块文件已创建");
            }
        },
        _ => {
            println!("⚠️ 其他结果: {:?}", outcome);
        }
    }

    println!("\n文件保留在 D:/tmp/mes/ 供查看");
}