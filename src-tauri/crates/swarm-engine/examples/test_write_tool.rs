//! AIWorkerExecutor 工具执行能力测试
//!
//! 验证 Claude CLI 是否能在 Rust 程序调用时执行 Write 工具

use swarm_engine::{
    Goal, CompletionCondition,
    ConditionEvaluator, ReconcileLoop, GoalOutcome,
    AIWorkerExecutor,
};
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("AIWorkerExecutor 工具执行测试");
    println!("========================================\n");

    // 创建一个简单的 Goal - 创建一个文件
    // 明确指定文件路径和内容
    let mut goal = Goal::new(
        "test-write-tool",
        "IMMEDIATELY use Write tool to create file D:/tmp/test-write-result.txt with content 'SUCCESS - File created by AIWorkerExecutor'",
        CompletionCondition::FileCheck {
            path: "D:/tmp/test-write-result.txt".to_string(),
            content_contains: Some("SUCCESS".to_string()),
            max_size_bytes: None,
        },
        "claude-worker",
    );
    goal.max_iterations = 5;

    println!("【Goal】创建文件 D:/tmp/test-write-result.txt");
    println!("  内容要求: 包含 'SUCCESS'");
    println!("  MaxIterations: {}", goal.max_iterations);

    // 创建 AIWorkerExecutor
    let executor = AIWorkerExecutor::new(
        "claude-worker",
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd"
    ).with_cwd("D:/dingsun/acp-ui/src-tauri");

    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    println!("\n【开始执行】");

    // 执行
    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n========================================");
    println!("【执行结果】");
    println!("========================================");

    // 检查文件
    let file_path = "D:/tmp/test-write-result.txt";
    if std::path::Path::new(file_path).exists() {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        let contains_success = content.contains("SUCCESS");
        println!("  文件存在: ✓");
        println!("  内容: {}", content);
        println!("  包含 'SUCCESS': {}", contains_success);
    } else {
        println!("  文件不存在: ✗");
    }

    match outcome {
        GoalOutcome::Converged { iterations, .. } => {
            println!("\n✅ Goal 收敛! {} 次迭代", iterations);
        },
        GoalOutcome::MaxIterReached { iterations, .. } => {
            println!("\n⚠️ 达到最大迭代 {}", iterations);
        },
        GoalOutcome::Failed(msg) => {
            println!("\n❌ 失败: {}", msg);
        },
        _ => println!("\n结果: {:?}", outcome),
    }

    println!("\n迭代历史:");
    for (i, record) in goal.iteration_log.iter().enumerate() {
        println!("  Iteration {}: feedback={}",
            i + 1,
            record.evaluation_result.feedback.chars().take(100).collect::<String>());
    }
}