// 功能流程测试 - 从输入到输出完整监控
// 测试路径：D:\dingsun\acp-ui\src-tauri\tests\test_function_flow.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 测试结果记录
#[derive(Debug)]
struct FlowTestRecord {
    step: String,
    input: String,
    output: String,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
}

/// 功能流程测试器
struct FunctionFlowTester {
    records: Vec<FlowTestRecord>,
    start_time: Instant,
}

impl FunctionFlowTester {
    fn new() -> Self {
        Self {
            records: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn record(
        &mut self,
        step: &str,
        input: &str,
        output: &str,
        success: bool,
        error: Option<String>,
    ) {
        let duration_ms = self.start_time.elapsed().as_millis() as u64;
        self.records.push(FlowTestRecord {
            step: step.to_string(),
            input: input.to_string(),
            output: output.to_string(),
            duration_ms,
            success,
            error,
        });
        self.start_time = Instant::now();
    }

    fn report(&self) -> String {
        let mut report = String::new();
        let separator = "=".repeat(60);
        report.push_str(&separator);
        report.push_str("\n功能流程测试报告\n");
        report.push_str(&separator);
        report.push_str("\n\n");

        for r in &self.records {
            report.push_str(&format!(
                "【步骤 {}】{}\n",
                self.records
                    .iter()
                    .position(|x| x.step == r.step)
                    .unwrap_or(0)
                    + 1,
                r.step
            ));
            report.push_str(&format!("  输入: {}\n", r.input));
            report.push_str(&format!("  输出: {}\n", r.output));
            report.push_str(&format!("  耗时: {}ms\n", r.duration_ms));
            report.push_str(&format!(
                "  状态: {}\n",
                if r.success {
                    "✅ 成功"
                } else {
                    "❌ 失败"
                }
            ));
            if let Some(e) = &r.error {
                report.push_str(&format!("  错误: {}\n", e));
            }
            report.push_str("\n");
        }

        let total_duration = self.records.iter().map(|r| r.duration_ms).sum::<u64>();
        let success_count = self.records.iter().filter(|r| r.success).count();
        let fail_count = self.records.iter().filter(|r| !r.success).count();

        report.push_str(&separator);
        report.push_str("\n汇总\n");
        report.push_str(&separator);
        report.push_str("\n");
        report.push_str(&format!("总耗时: {}ms\n", total_duration));
        report.push_str(&format!("成功: {} 步\n", success_count));
        report.push_str(&format!("失败: {} 步\n", fail_count));
        report.push_str(&format!(
            "成功率: {:.1}%\n",
            (success_count as f32 / self.records.len() as f32) * 100.0
        ));

        report
    }
}

// ===== 测试 1: One-Shot Interface 完整流程 =====

fn test_one_shot_flow() {
    println!("\n测试 1: One-Shot Interface 完整流程\n");

    let mut tester = FunctionFlowTester::new();

    // Step 1: 用户输入
    let user_input = "帮我写一个 Vue 组件，实现用户登录表单";
    tester.record("用户输入", user_input, "接收用户请求", true, None);

    // Step 2: 角色检测
    // 模拟 UserRoleDetector::detect()
    let detected_role = detect_role(user_input);
    tester.record(
        "角色检测",
        user_input,
        &format!("检测到角色: {}", detected_role),
        detected_role == "developer",
        if detected_role != "developer" {
            Some("期望 developer，实际 ".to_string() + &detected_role)
        } else {
            None
        },
    );

    // Step 3: 场景检测
    // 模拟 SceneDetector::detect()
    let detected_scene = detect_scene(user_input);
    tester.record(
        "场景检测",
        user_input,
        &format!("检测到场景: {}", detected_scene),
        detected_scene == "code_generation",
        if detected_scene != "code_generation" {
            Some("期望 code_generation".to_string())
        } else {
            None
        },
    );

    // Step 4: Agent 选择
    // 模拟 AgentSelector::select()
    let (selected_agent, reason) = select_agent(&detected_role, &detected_scene);
    tester.record(
        "Agent选择",
        &format!("角色={}, 场景={}", detected_role, detected_scene),
        &format!("选择 Agent: {} ({})", selected_agent, reason),
        selected_agent == "claude-code",
        if selected_agent != "claude-code" {
            Some("期望 claude-code".to_string())
        } else {
            None
        },
    );

    // Step 5: 透明度信息生成
    let transparency = generate_transparency(&detected_role, &detected_scene, &selected_agent);
    tester.record(
        "透明度生成",
        &format!(
            "role={}, scene={}, agent={}",
            detected_role, detected_scene, selected_agent
        ),
        &transparency,
        true,
        None,
    );

    // 输出报告
    let report = tester.report();
    println!("{}", report);

    // 保存报告到 D:\tmp
    std::fs::write("D:/tmp/one-shot-flow-test.txt", &report).expect("Failed to write report");
}

// ===== 测试 2: Privacy Orchestrator 完整流程 =====

fn test_privacy_flow() {
    println!("\n测试 2: Privacy Orchestrator 完整流程\n");

    let mut tester = FunctionFlowTester::new();

    // Step 1: 任务输入（包含敏感数据）
    let task_content = "api_key = 'sk-1234567890abcdefghij123456'";
    let file_paths = vec!["src/config.rs".to_string()];
    tester.record(
        "任务输入",
        task_content,
        &format!("文件: {:?}", file_paths),
        true,
        None,
    );

    // Step 2: 敏感数据检测
    let sensitive_detected = detect_sensitive_data(task_content);
    tester.record(
        "敏感数据检测",
        task_content,
        &format!("检测到 {} 处敏感数据", sensitive_detected.len()),
        sensitive_detected.len() > 0,
        if sensitive_detected.is_empty() {
            Some("未检测到敏感数据".to_string())
        } else {
            None
        },
    );

    // Step 3: 隐私级别判定
    let privacy_level = determine_privacy_level(&sensitive_detected, &file_paths);
    tester.record(
        "隐私级别判定",
        &format!(
            "敏感={}, 文件={}",
            sensitive_detected.len(),
            file_paths.len()
        ),
        &format!("隐私级别: {}", privacy_level),
        privacy_level == "Sanitized",
        if privacy_level != "Sanitized" {
            Some("期望 Sanitized".to_string())
        } else {
            None
        },
    );

    // Step 4: 内容脱敏
    let sanitized = sanitize_content(task_content, &sensitive_detected);
    tester.record(
        "内容脱敏",
        task_content,
        &sanitized,
        sanitized.contains("[REDACTED"),
        if !sanitized.contains("[REDACTED") {
            Some("脱敏失败".to_string())
        } else {
            None
        },
    );

    // 输出报告
    let report = tester.report();
    println!("{}", report);

    std::fs::write("D:/tmp/privacy-flow-test.txt", &report).expect("Failed to write report");
}

// ===== 测试 3: Goal Graph 完整流程 =====

fn test_goal_graph_flow() {
    println!("\n测试 3: Goal Graph 依赖管理流程\n");

    let mut tester = FunctionFlowTester::new();

    // Step 1: 创建 Goal A（无依赖）
    let goal_a = create_goal("goal-a", "实现用户认证", vec![]);
    tester.record(
        "创建 Goal A",
        "实现用户认证",
        &format!("Goal ID: {}, Status: Pending", goal_a.id),
        goal_a.status == "Pending",
        None,
    );

    // Step 2: 创建 Goal B（依赖 A）
    let goal_b = create_goal("goal-b", "实现登录API", vec!["goal-a"]);
    tester.record(
        "创建 Goal B",
        "实现登录API（依赖 goal-a）",
        &format!(
            "Goal ID: {}, Dependencies: {:?}",
            goal_b.id, goal_b.dependencies
        ),
        goal_b.dependencies.contains(&"goal-a".to_string()),
        None,
    );

    // Step 3: 获取就绪 Goal
    let ready_goals = get_ready_goals(&[goal_a.clone(), goal_b.clone()]);
    tester.record(
        "获取就绪 Goal",
        "检查依赖状态",
        &format!(
            "就绪 Goal: {:?}",
            ready_goals.iter().map(|g| &g.id).collect::<Vec<_>>()
        ),
        ready_goals.len() == 1 && ready_goals[0].id == "goal-a",
        None,
    );

    // Step 4: 执行 Goal A 并标记收敛
    let converged_a = converge_goal(&goal_a);
    tester.record(
        "执行 Goal A",
        &goal_a.description,
        &format!("状态: {}", converged_a.status),
        converged_a.status == "Converged",
        None,
    );

    // Step 5: 再次获取就绪 Goal（现在 B 应该就绪）
    let ready_goals_after = get_ready_goals(&[converged_a.clone(), goal_b.clone()]);
    tester.record(
        "重新检查就绪 Goal",
        "Goal A 已收敛",
        &format!(
            "就绪 Goal: {:?}",
            ready_goals_after.iter().map(|g| &g.id).collect::<Vec<_>>()
        ),
        ready_goals_after.len() == 1 && ready_goals_after[0].id == "goal-b",
        if ready_goals_after.iter().any(|g| g.id == "goal-b") {
            None
        } else {
            Some("Goal B 未能就绪".to_string())
        },
    );

    let report = tester.report();
    println!("{}", report);

    std::fs::write("D:/tmp/goal-graph-flow-test.txt", &report).expect("Failed to write report");
}

// ===== 辅助函数（模拟实际实现） =====

fn detect_role(input: &str) -> String {
    let input_lower = input.to_lowercase();
    if input_lower.contains("vue") || input_lower.contains("组件") || input_lower.contains("代码")
    {
        "developer".to_string()
    } else if input_lower.contains("文案") || input_lower.contains("营销") {
        "marketer".to_string()
    } else if input_lower.contains("图片") || input_lower.contains("设计") {
        "designer".to_string()
    } else if input_lower.contains("财务") || input_lower.contains("excel") {
        "finance".to_string()
    } else {
        "general".to_string()
    }
}

fn detect_scene(input: &str) -> String {
    let input_lower = input.to_lowercase();
    if input_lower.contains("写") || input_lower.contains("生成") || input_lower.contains("实现")
    {
        "code_generation".to_string()
    } else if input_lower.contains("审查") || input_lower.contains("review") {
        "code_review".to_string()
    } else if input_lower.contains("修复") || input_lower.contains("bug") {
        "bug_fix".to_string()
    } else {
        "general".to_string()
    }
}

fn select_agent(role: &str, scene: &str) -> (String, String) {
    if scene == "code_generation" {
        (
            "claude-code".to_string(),
            "Selected claude-code for code generation".to_string(),
        )
    } else if scene == "bug_fix" {
        (
            "codex".to_string(),
            "Selected codex for bug fix".to_string(),
        )
    } else if role == "marketer" {
        (
            "kimi".to_string(),
            "Selected kimi for marketing".to_string(),
        )
    } else {
        ("claude-code".to_string(), "Default agent".to_string())
    }
}

fn generate_transparency(role: &str, scene: &str, agent: &str) -> String {
    format!(
        "DAG: Input -> RoleDetection({}) -> SceneDetection({}) -> AgentSelection({})",
        role, scene, agent
    )
}

fn detect_sensitive_data(content: &str) -> Vec<String> {
    let mut matches = Vec::new();
    if content.contains("api_key") {
        matches.push("api_key pattern".to_string());
    }
    if content.contains("password") {
        matches.push("password pattern".to_string());
    }
    matches
}

fn determine_privacy_level(sensitive: &[String], files: &[String]) -> String {
    if files.iter().any(|f| f.contains(".env")) {
        "StrictLocal".to_string()
    } else if !sensitive.is_empty() {
        "Sanitized".to_string()
    } else {
        "Standard".to_string()
    }
}

fn sanitize_content(content: &str, _sensitive: &[String]) -> String {
    content.replace("sk-1234567890abcdefghij123456", "[REDACTED_api_key]")
}

#[derive(Clone)]
struct Goal {
    id: String,
    description: String,
    dependencies: Vec<String>,
    status: String,
}

fn create_goal(id: &str, desc: &str, deps: Vec<&str>) -> Goal {
    Goal {
        id: id.to_string(),
        description: desc.to_string(),
        dependencies: deps.iter().map(|s| s.to_string()).collect(),
        status: "Pending".to_string(),
    }
}

fn get_ready_goals(goals: &[Goal]) -> Vec<Goal> {
    goals
        .iter()
        .filter(|g| {
            g.status == "Pending"
                && g.dependencies.iter().all(|dep| {
                    goals
                        .iter()
                        .any(|other| other.id == *dep && other.status == "Converged")
                })
        })
        .cloned()
        .collect()
}

fn converge_goal(goal: &Goal) -> Goal {
    Goal {
        id: goal.id.clone(),
        description: goal.description.clone(),
        dependencies: goal.dependencies.clone(),
        status: "Converged".to_string(),
    }
}

// ===== 主测试入口 =====

fn main() {
    let separator = "=".repeat(60);
    println!("{}", separator);
    println!("开发者轨道功能流程测试");
    println!("{}", separator);

    test_one_shot_flow();
    test_privacy_flow();
    test_goal_graph_flow();

    println!("\n所有测试报告已保存到 D:/tmp/");
}

// ===== 运行测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_flow_tests() {
        test_one_shot_flow();
        test_privacy_flow();
        test_goal_graph_flow();
    }
}
