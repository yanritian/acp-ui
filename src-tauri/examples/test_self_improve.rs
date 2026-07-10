//! 使用 ACP-UI Goal 系统完成 ComplexGoalExecutor 改进
//!
//! Goal: 完善 ComplexGoalExecutor 的规划 prompt
//! 执行者: AIWorkerExecutor (Claude CLI)
//! 验证: 测试通过

use std::sync::Arc;
use swarm_engine::{
    AIWorkerExecutor, CompletionCondition, ConditionEvaluator, Goal, GoalOutcome, ReconcileLoop,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ACP-UI 自我改进测试");
    println!("========================================\n");

    println!("【Goal】完善 ComplexGoalExecutor 的规划 prompt");
    println!("  - 让 AI 输出结构化 JSON 格式");
    println!("  - 添加智能 fallback 分解");
    println!("  - 确保 65 个测试通过\n");

    // 创建 Goal - 描述要完成的任务
    let mut goal = Goal::new(
        "improve-complex-executor",
        "修改 D:/dingsun/acp-ui/src-tauri/crates/swarm-engine/src/complex_executor.rs 文件，\
        改进 build_planning_prompt 函数使其输出 JSON 格式，\
        改进 parse_plan 函数添加 JSON 解析和智能 fallback，\
        改进 build_execution_prompt_from_info 添加文件路径提示，\
        确保所有测试通过 (cargo test --package swarm-engine)",
        CompletionCondition::CommandSuccess {
            command: "cargo".to_string(),
            args: vec![
                "test".to_string(),
                "--package".to_string(),
                "swarm-engine".to_string(),
            ],
            cwd: Some("D:/dingsun/acp-ui/src-tauri".to_string()),
        },
    );
    goal.max_iterations = 3;

    println!("【步骤 1】创建 Goal");
    println!("  ID: {}", goal.id);
    println!("  Condition: cargo test --package swarm-engine passes\n");

    // 创建 AIWorkerExecutor
    let executor = AIWorkerExecutor::new(
        "claude-worker",
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd",
    )
    .with_cwd("D:/dingsun/acp-ui/src-tauri");

    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    println!("【步骤 2】AIWorkerExecutor 执行 Goal");
    println!("  我（Claude）只观察，不干预...\n");

    // 执行 - AI Worker 会完成所有工作
    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n【步骤 3】观察结果");
    println!("  Outcome: {:?}", outcome);

    // 验证
    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&goal.completion_condition);

    println!("  测试通过: {}", result.converged);

    println!("\n========================================");
    match outcome {
        GoalOutcome::Converged { iterations, .. } => {
            println!("✅ ACP-UI 自我改进成功！");
            println!("  • Goal 由 AIWorkerExecutor 完成");
            println!("  • {} 次迭代后收敛", iterations);
            println!("  • 我只观察，未干预代码");
        }
        GoalOutcome::MaxIterReached { iterations, .. } => {
            println!("⚠️ 达到最大迭代 {}，需要更多迭代", iterations);
        }
        GoalOutcome::Failed { reason, .. } => {
            println!("执行失败: {}", reason);
        }
        _ => println!("结果: {:?}", outcome),
    }
}
