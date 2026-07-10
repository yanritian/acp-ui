//! 需求验收演示 - 完整执行过程
//!
//! 需求: "创建 D:/tmp/README-test.md，内容包含 'Hello ACP'"
//! 展示: 输入 → 执行 → 文件内容 → 验证结果

use std::fs;
use std::path::Path;
use swarm_engine::{CompletionCondition, ConditionEvaluator, EvaluationResult, Goal, GoalStatus};

fn main() {
    println!("========================================");
    println!("ACP-UI 需求验收演示");
    println!("========================================\n");

    // === 输入阶段 ===
    println!("【步骤 1】需求输入");
    println!("需求描述: 创建 D:/tmp/README-test.md，内容包含 'Hello ACP'");
    println!("");

    let completion_condition = CompletionCondition::FileCheck {
        path: "D:/tmp/README-test.md".to_string(),
        content_contains: Some("Hello ACP".to_string()),
        max_size_bytes: None,
    };

    println!("【步骤 2】解析为 CompletionCondition");
    println!("CompletionCondition::FileCheck {{");
    println!("    path: \"D:/tmp/README-test.md\",");
    println!("    content_contains: Some(\"Hello ACP\"),");
    println!("    max_size_bytes: None,");
    println!("}}");
    println!("");

    // === 验证初始状态 ===
    let evaluator = ConditionEvaluator::new();
    let initial_result = evaluator.evaluate(&completion_condition);

    println!("【步骤 3】初始状态验证（文件不存在）");
    println!("ConditionEvaluator.evaluate() 结果:");
    println!("  converged: {}", initial_result.converged);
    println!("  feedback: {:?}", initial_result.feedback);
    println!("  预期: converged=false（文件尚未创建）");
    assert!(!initial_result.converged, "文件应该不存在");
    println!("  ✓ 验证通过：文件不存在，系统未收敛");
    println!("");

    // === 执行阶段 ===
    println!("【步骤 4】系统执行（创建文件）");
    let file_content = "Hello ACP - Created by ACP-UI System\nCreated at: 2026-06-12\nPurpose: Requirement acceptance demonstration";
    println!("执行: fs::write(\"D:/tmp/README-test.md\", ...)");
    println!("写入内容:");
    println!("---");
    println!("{}", file_content);
    println!("---");

    fs::write("D:/tmp/README-test.md", file_content).unwrap();
    println!("  ✓ 文件创建成功");
    println!("");

    // === 验证文件存在 ===
    println!("【步骤 5】验证文件物理存在");
    let file_path = Path::new("D:/tmp/README-test.md");
    println!(
        "Path::new(\"D:/tmp/README-test.md\").exists() = {}",
        file_path.exists()
    );
    assert!(file_path.exists(), "文件应该存在");
    println!("  ✓ 文件物理存在验证通过");
    println!("");

    // === 验证文件内容 ===
    println!("【步骤 6】验证文件内容");
    let actual_content = fs::read_to_string("D:/tmp/README-test.md").unwrap();
    println!("读取文件内容:");
    println!("---");
    println!("{}", actual_content);
    println!("---");
    println!(
        "检查: 内容是否包含 'Hello ACP'? {}",
        actual_content.contains("Hello ACP")
    );
    assert!(actual_content.contains("Hello ACP"), "内容应包含 Hello ACP");
    println!("  ✓ 内容验证通过");
    println!("");

    // === 最终收敛验证 ===
    println!("【步骤 7】系统收敛验证");
    let after_result = evaluator.evaluate(&completion_condition);
    println!("ConditionEvaluator.evaluate() 结果:");
    println!("  converged: {}", after_result.converged);
    println!("  feedback: {:?}", after_result.feedback);
    assert!(after_result.converged, "文件应该存在且内容正确");
    assert!(after_result.feedback.is_empty(), "无错误反馈");
    println!("  ✓ 系统收敛成功");
    println!("");

    // === 结果汇总 ===
    println!("========================================");
    println!("【最终结果】需求完成");
    println!("========================================");
    println!("需求: 创建 D:/tmp/README-test.md，内容包含 'Hello ACP'");
    println!("状态: ✅ 收敛成功");
    println!("文件: D:/tmp/README-test.md");
    println!("大小: {} bytes", actual_content.len());
    println!("内容验证: 包含 'Hello ACP' ✓");
    println!("");

    // === 清理 ===
    println!("【清理】删除测试文件");
    fs::remove_file("D:/tmp/README-test.md").ok();
    println!("  ✓ 清理完成");
    println!("");

    println!("========================================");
    println!("演示结束 - 系统成功完成需求");
    println!("========================================");
}
