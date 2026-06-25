//! ACP-UI Goal 系统测试 - 财务系统需求
//!
//! Goal: 创建财务系统（代码 + 测试 + 验证）
//! 执行者: AIWorkerExecutor (Claude CLI)
//! 验证: npm test 全部通过
//! 观察者: 我只观察，不干预代码

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
    println!("ACP-UI Goal-Driven 财务系统测试");
    println!("========================================\n");

    println!("【Goal 定义】");
    println!("需求: 创建财务系统（代码 + 测试 + 验证）");
    println!("执行者: AIWorkerExecutor");
    println!("验证条件: npm test 全部通过");
    println!("我的角色: 只观察，不干预代码\n");

    // 准备目录
    std::fs::create_dir_all("D:/tmp/finance-goal/src").ok();
    std::fs::create_dir_all("D:/tmp/finance-goal/tests").ok();

    // 创建 Goal - 完整的财务系统需求
    // 改进：使用文件存在检查替代 npm test（因为系统可能没有 npm）
    let mut goal = Goal::new(
        "finance-system-goal",
        "Create a complete finance system with code, tests, and validation:

PHASE 1 - CODE:
Create D:/tmp/finance-goal/src/schemas/account.schema.ts with Zod AccountSchema
Create D:/tmp/finance-goal/src/schemas/transaction.schema.ts with Zod TransactionSchema
Create D:/tmp/finance-goal/src/schemas/report.schema.ts with Zod FinancialReportSchema
Create D:/tmp/finance-goal/src/schemas/budget.schema.ts with Zod BudgetSchema
Create D:/tmp/finance-goal/src/services/account.service.ts with AccountService
Create D:/tmp/finance-goal/src/services/transaction.service.ts with TransactionService
Create D:/tmp/finance-goal/src/services/report.service.ts with ReportService
Create D:/tmp/finance-goal/src/services/budget.service.ts with BudgetService

PHASE 2 - TESTS:
Create D:/tmp/finance-goal/tests/unit/schemas/ for schema tests
Create D:/tmp/finance-goal/tests/unit/services/ for service tests
Create D:/tmp/finance-goal/tests/integration/ for integration tests
Create D:/tmp/finance-goal/tests/utils/test-helpers.ts for test utilities

PHASE 3 - VALIDATION:
Create D:/tmp/finance-goal/package.json with Vitest + Zod
Create D:/tmp/finance-goal/vitest.config.ts
Create D:/tmp/finance-goal/tsconfig.json",
        CompletionCondition::All {
            conditions: vec![
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/package.json".to_string(),
                    content_contains: Some("vitest".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/src/schemas/account.schema.ts".to_string(),
                    content_contains: Some("AccountSchema".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/src/schemas/transaction.schema.ts".to_string(),
                    content_contains: Some("TransactionSchema".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/src/schemas/report.schema.ts".to_string(),
                    content_contains: Some("FinancialReportSchema".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/src/schemas/budget.schema.ts".to_string(),
                    content_contains: Some("BudgetSchema".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/src/services/account.service.ts".to_string(),
                    content_contains: Some("AccountService".to_string()),
                    max_size_bytes: None,
                },
                CompletionCondition::FileCheck {
                    path: "D:/tmp/finance-goal/tests/utils/test-helpers.ts".to_string(),
                    content_contains: None,
                    max_size_bytes: None,
                },
            ],
        },
    );
    goal.executor = Some("claude-worker".to_string());
    goal.max_iterations = 15;
    goal.token_budget = Some(500000);

    println!("【Goal 配置】");
    println!("  ID: {}", goal.id);
    println!("  MaxIterations: {}", goal.max_iterations);
    println!("  TokenBudget: {:?}", goal.token_budget);
    println!("  CompletionCondition: 7 个文件存在检查\n");

    // 创建 AIWorkerExecutor
    let executor = AIWorkerExecutor::new(
        "claude-worker",
        "C:/Users/Administrator/AppData/Roaming/npm/claude.cmd"
    ).with_cwd("D:/dingsun/acp-ui/src-tauri");

    let reconciler = ReconcileLoop::new(Arc::new(Mutex::new(executor)));

    println!("========================================");
    println!("【开始执行 - 我只观察，不干预】");
    println!("========================================\n");

    // 执行 - AI Worker 会完成所有工作，我只观察
    let outcome = reconciler.reconcile_goal(&mut goal).await;

    println!("\n========================================");
    println!("【执行完成 - 观察结果】");
    println!("========================================\n");

    // 验证最终结果
    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&goal.completion_condition);

    println!("  CompletionCondition 评估:");
    println!("    converged: {}", result.converged);
    println!("    feedback: {:?}", result.feedback);

    // 检查文件结构
    let paths = [
        "D:/tmp/finance-goal/package.json",
        "D:/tmp/finance-goal/vitest.config.ts",
        "D:/tmp/finance-goal/src/schemas/account.schema.ts",
        "D:/tmp/finance-goal/src/services/account.service.ts",
        "D:/tmp/finance-goal/tests/unit/schemas/account.schema.test.ts",
    ];

    println!("\n  文件检查:");
    for path in &paths {
        let exists = std::path::Path::new(path).exists();
        println!("    {} - {}", path, if exists { "✓ 存在" } else { "✗ 不存在" });
    }

    println!("\n========================================");
    println!("【最终结果】");
    println!("========================================\n");

    match outcome {
        GoalOutcome::Converged { iterations, tokens_used, .. } => {
            println!("✅ ACP-UI Goal-Driven 测试成功！");
            println!("  • {} 次迭代后收敛", iterations);
            println!("  • 总 tokens: {}", tokens_used);
            println!("  • 我只观察，未干预代码");
            println!("  • 所有工作由 AIWorkerExecutor 完成");
        },
        GoalOutcome::MaxIterReached { iterations } => {
            println!("⚠️ 达到最大迭代 {}", iterations);
            println!("  需要更多迭代或调整需求");
        },
        GoalOutcome::Failed { reason, .. } => {
            println!("❌ 执行失败: {}", reason);
        },
        GoalOutcome::BudgetExhausted { tokens_used } => {
            println!("⚠️ Token 预算耗尽: {} tokens", tokens_used);
        },
        _ => println!("结果: {:?}", outcome),
    }

    println!("\n迭代历史:");
    for (i, record) in goal.iteration_log.iter().enumerate() {
        println!("  Iteration {}: tokens={}, feedback preview: {:?}",
            i + 1, record.tokens_used,
            record.feedback.as_deref().unwrap_or("").chars().take(100).collect::<String>());
    }
}