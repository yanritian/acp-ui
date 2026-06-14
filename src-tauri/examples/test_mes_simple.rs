//! AI Worker MES 生产模块快速测试
//!
//! 需求：只创建生产管理模块

use swarm_engine::{
    Goal, CompletionCondition,
    ConditionEvaluator, ReconcileLoop, GoalOutcome,
    AIWorkerExecutor,
};
use std::fs;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("MES 生产模块快速测试");
    println!("========================================\n");

    fs::create_dir_all("D:/tmp/mes-simple/src").ok();

    println!("【需求】创建生产管理模块 D:/tmp/mes-simple/src/production.ts");
    println!("  包含 ProductionOrder 类型定义和相关函数\n");

    let mut goal = Goal::new(
        "mes-production",
        "创建 TypeScript 文件 D:/tmp/mes-simple/src/production.ts，包含 ProductionOrder 接口定义（包含 id, productName, quantity, status 字段），以及 createProductionOrder 和 updateStatus 函数",
        CompletionCondition::FileCheck {
            path: "D:/tmp/mes-simple/src/production.ts".to_string(),
            content_contains: Some("ProductionOrder".to_string()),
            max_size_bytes: None,
        },
        "claude-worker",
    );
    goal.max_iterations = 1;

    println!("【执行】发送任务给 Claude Code...\n");

    let executor = AIWorkerExecutor::new("claude-worker", "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd")
        .with_cwd("D:/tmp/mes-simple");
    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n【结果】Outcome: {:?}\n", outcome);

    // 验证
    if std::path::Path::new("D:/tmp/mes-simple/src/production.ts").exists() {
        let content = fs::read_to_string("D:/tmp/mes-simple/src/production.ts").unwrap();
        println!("✅ 文件创建成功 ({:.1} KB)", content.len() as f64 / 1024.0);
        println!("✅ 包含 ProductionOrder: {}", content.contains("ProductionOrder"));
        println!("\n内容预览:");
        println!("---");
        for line in content.lines().take(20) {
            println!("{}", line);
        }
        println!("---");
    } else {
        println!("❌ 文件未创建");
    }

    println!("\n========================================");
    match outcome {
        GoalOutcome::Converged { .. } => println!("✅ MES 生产模块测试成功"),
        _ => println!("结果: {:?}", outcome),
    }
}