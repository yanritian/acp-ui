//! ERP+MES+WMS 综合系统需求测试
//!
//! 验证 ACP-UI 能否自动处理复杂需求：
//! "给我做一个erp+mes+wms的综合系统与手机app，同时还要有winform做windows桌面系统"
//!
//! 测试策略：先测试需求分解和基础架构创建（收敛条件为关键文件存在）

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
    println!("ACP-UI 复杂需求闭环测试");
    println!("========================================\n");

    println!("【需求】");
    println!("给我做一个erp+mes+wms的综合系统与手机app，");
    println!("同时还要有winform做windows桌面系统\n");

    // 准备目录
    std::fs::create_dir_all("D:/tmp/erp-mes-wms-system").ok();

    // 创建 Goal - 首先验证需求分解能力
    // 第一阶段：创建基础架构文档
    let mut goal = Goal::new(
        "erp-mes-wms-architecture",
        "Create comprehensive architecture design for ERP+MES+WMS integrated system.

TASK: Create architecture document D:/tmp/erp-mes-wms-system/ARCHITECTURE.md

The document MUST include:
1. System Overview - ERP/MES/WMS modules description
2. Mobile App Architecture - Flutter/React Native design
3. Windows Desktop (WinForm) Architecture
4. Integration Layer - API gateway, message queue
5. Database Schema Overview - tables for each module
6. Technology Stack Recommendations

Use Write tool to create the file immediately.",
        CompletionCondition::FileCheck {
            path: "D:/tmp/erp-mes-wms-system/ARCHITECTURE.md".to_string(),
            content_contains: Some("ERP+MES+WMS".to_string()),
            max_size_bytes: None,
        },
    );
    goal.executor = Some("claude-worker".to_string());
    goal.max_iterations = 3;
    goal.token_budget = Some(50000);

    println!("【Goal 配置】");
    println!("  ID: {}", goal.id);
    println!("  MaxIterations: {}", goal.max_iterations);
    println!("  TokenBudget: {:?}", goal.token_budget);
    println!("  验收条件: ARCHITECTURE.md 包含 'ERP+MES+WMS'\n");

    // 创建 AIWorkerExecutor
    let executor = AIWorkerExecutor::new(
        "claude-worker",
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd"
    ).with_cwd("D:/tmp");

    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    println!("========================================");
    println!("【开始执行 - AI 自动完成】");
    println!("========================================\n");

    // 执行
    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n========================================");
    println!("【执行结果】");
    println!("========================================\n");

    // 验证文件
    let file_path = "D:/tmp/erp-mes-wms-system/ARCHITECTURE.md";
    if std::path::Path::new(file_path).exists() {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        let contains_keyword = content.contains("ERP+MES+WMS");
        println!("  文件存在: ✓");
        println!("  文件大小: {} bytes", content.len());
        println!("  包含 'ERP+MES+WMS': {}", contains_keyword);

        println!("\n【架构文档内容预览】");
        println!("----------------------------------------");
        for line in content.lines().take(30) {
            println!("  {}", line);
        }
        println!("----------------------------------------");
    } else {
        println!("  文件不存在: ✗");
    }

    match outcome {
        GoalOutcome::Converged { iterations, .. } => {
            println!("\n✅ Goal 收敛! {} 次迭代", iterations);
            println!("  ACP-UI 成功自动完成架构设计");
        },
        GoalOutcome::MaxIterReached { iterations, .. } => {
            println!("\n⚠️ 达到最大迭代 {}", iterations);
        },
        GoalOutcome::Failed { reason, .. } => {
            println!("\n❌ 失败: {}", reason);
        },
        _ => println!("\n结果: {:?}", outcome),
    }

    println!("\n迭代历史:");
    for (i, record) in goal.iteration_log.iter().enumerate() {
        println!("  Iteration {}: converged={}",
            i + 1, record.evaluation.passed);
    }

    println!("\n========================================");
    println!("【闭环验证结论】");
    println!("========================================");

    // 检查 ACP-UI 是否闭环
    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&goal.completion_condition);

    if result.converged {
        println!("✅ ACP-UI 已闭环 - 需求自动处理成功");
        println!("  系统能够: 理解需求 → 分解任务 → 生成架构 → 验证收敛");
    } else {
        println!("❌ ACP-UI 未闭环 - 需要继续改进");
        println!("  反馈: {}", result.feedback);
    }
}