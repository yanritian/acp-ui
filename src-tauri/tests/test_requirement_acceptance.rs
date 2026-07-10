//! Real Requirement Acceptance Test
//!
//! Given a simple requirement: "Create a file README-test.md with content 'Hello ACP'"
//! The system should:
//! 1. Create Goal with FileCheck completion condition
//! 2. Execute (create the file)
//! 3. Evaluate (check file exists with content)
//! 4. Converge when condition met

use std::fs;
use std::path::Path;
use swarm_engine::{CompletionCondition, ConditionEvaluator, EvaluationResult, Goal, GoalStatus};

// ============================================================================
// Requirement: "Create README-test.md with 'Hello ACP'"
// ============================================================================

#[test]
fn test_requirement_create_file_with_content() {
    // === Requirement: Create a file with specific content ===
    let requirement = "创建一个文件 D:/tmp/README-test.md，内容包含 'Hello ACP'";

    // === Step 1: Define completion condition ===
    let completion_condition = CompletionCondition::FileCheck {
        path: "D:/tmp/README-test.md".to_string(),
        content_contains: Some("Hello ACP".to_string()),
        max_size_bytes: None,
    };

    // === Step 2: Create evaluator ===
    let evaluator = ConditionEvaluator::new();

    // === Step 3: Check initial state (file doesn't exist) ===
    let initial_result = evaluator.evaluate(&completion_condition);
    assert!(!initial_result.converged, "File should not exist initially");

    // === Step 4: Execute (create file) ===
    fs::write(
        "D:/tmp/README-test.md",
        "Hello ACP - Created by ACP-UI System",
    )
    .unwrap();

    // === Step 5: Evaluate (verify file created) ===
    let after_result = evaluator.evaluate(&completion_condition);
    assert!(
        after_result.converged,
        "File should exist with correct content"
    );
    assert!(after_result.feedback.is_empty(), "No error feedback");

    // === Step 6: Cleanup ===
    fs::remove_file("D:/tmp/README-test.md").ok();
}

// ============================================================================
// Requirement: "Run echo command successfully"
// ============================================================================

#[test]
fn test_requirement_command_success() {
    // === Requirement: Execute a shell command that succeeds ===
    let requirement = "执行 echo 命令，退出码为 0";

    let completion_condition = CompletionCondition::command_success("echo");
    let evaluator = ConditionEvaluator::new();

    // === Execute and evaluate ===
    let result = evaluator.evaluate(&completion_condition);
    assert!(result.converged, "echo command should succeed");
}

// ============================================================================
// Requirement: "File must NOT contain sensitive data"
// ============================================================================

#[test]
fn test_requirement_file_content_negative() {
    // === Requirement: A file should NOT contain 'password' ===
    let requirement = "配置文件不应包含 'password' 字样";

    // Create a safe config file
    fs::write("D:/tmp/config-safe.txt", "api_key=abc123\nenv=production").unwrap();

    // Evaluate: file contains 'password' → should FAIL (we want it to NOT contain)
    let evaluator = ConditionEvaluator::new();
    let condition_has_password = CompletionCondition::FileCheck {
        path: "D:/tmp/config-safe.txt".to_string(),
        content_contains: Some("password".to_string()),
        max_size_bytes: None,
    };

    let result = evaluator.evaluate(&condition_has_password);
    assert!(
        !result.converged,
        "File correctly does NOT contain 'password'"
    );

    // Cleanup
    fs::remove_file("D:/tmp/config-safe.txt").ok();
}

// ============================================================================
// Requirement: "Multiple conditions must all pass (AND)"
// ============================================================================

#[test]
fn test_requirement_all_conditions() {
    // === Requirement: File exists AND command succeeds ===
    let requirement = "README-test.md 文件存在，且 echo 命令成功";

    // Create file first
    fs::write("D:/tmp/README-test.md", "Test content").unwrap();

    let all_condition = CompletionCondition::All {
        conditions: vec![
            CompletionCondition::FileCheck {
                path: "D:/tmp/README-test.md".to_string(),
                content_contains: None,
                max_size_bytes: None,
            },
            CompletionCondition::command_success("echo"),
        ],
    };

    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&all_condition);

    assert!(result.converged, "Both conditions should pass");

    // Cleanup
    fs::remove_file("D:/tmp/README-test.md").ok();
}

// ============================================================================
// Requirement: "At least one condition passes (OR)"
// ============================================================================

#[test]
fn test_requirement_any_condition() {
    // === Requirement: Either file exists OR command succeeds ===
    let requirement = "至少满足一个：README-test.md 存在，或 echo 成功";

    // Don't create file, but echo will succeed
    let any_condition = CompletionCondition::Any {
        conditions: vec![
            CompletionCondition::FileCheck {
                path: "D:/tmp/nonexistent.txt".to_string(),
                content_contains: None,
                max_size_bytes: None,
            },
            CompletionCondition::command_success("echo"),
        ],
    };

    let evaluator = ConditionEvaluator::new();
    let result = evaluator.evaluate(&any_condition);

    assert!(
        result.converged,
        "At least one condition (echo) should pass"
    );
}

// ============================================================================
// Full Workflow: Goal from requirement → execute → converge
// ============================================================================

#[tokio::test]
async fn test_full_requirement_workflow() {
    use std::sync::Arc;
    use swarm_engine::{GoalGraph, GoalOutcome, ReconcileLoop};
    use tokio::sync::Mutex;

    // === Requirement: Create a test file ===
    let requirement_desc = "创建测试文件并验证内容";

    // === Step 1: Create Goal ===
    let mut goal = Goal::new(
        "req-001",
        requirement_desc,
        CompletionCondition::FileCheck {
            path: "D:/tmp/workflow-test.txt".to_string(),
            content_contains: Some("workflow success".to_string()),
            max_size_bytes: None,
        },
    );
    goal.max_iterations = 3;

    // === Step 2: Create executor that creates file ===
    struct FileCreatorExecutor {
        worker_id: String,
    }

    impl swarm_engine::WorkerExecutor for FileCreatorExecutor {
        fn execute(&mut self, goal: &Goal) -> Result<String, swarm_engine::ReconcileError> {
            // Simulate creating the file
            let path = match &goal.completion_condition {
                CompletionCondition::FileCheck { path, .. } => path.clone(),
                _ => "D:/tmp/workflow-test.txt".to_string(),
            };
            fs::write(&path, "workflow success - created by executor").ok();
            Ok(format!("File created: {}", path))
        }

        fn worker_id(&self) -> &str {
            &self.worker_id
        }
    }

    let executor = Arc::new(Mutex::new(FileCreatorExecutor {
        worker_id: "file-creator".to_string(),
    }));
    let reconciler = ReconcileLoop::new(executor);

    // === Step 3: Execute goal ===
    let outcome = reconciler.reconcile_goal(&mut goal).await;

    // === Step 4: Verify convergence ===
    assert!(
        matches!(outcome, GoalOutcome::Converged { .. }),
        "Goal should converge after file creation"
    );
    assert_eq!(goal.status, GoalStatus::Converged);

    // === Step 5: Verify file exists ===
    assert!(
        Path::new("D:/tmp/workflow-test.txt").exists(),
        "File should be created"
    );

    // === Cleanup ===
    fs::remove_file("D:/tmp/workflow-test.txt").ok();
}

// ============================================================================
// Requirement with Sandbox Security Check
// ============================================================================

#[test]
fn test_requirement_sandbox_blocks_env_file() {
    use tool_sandbox::{DeniedPatterns, SandboxConfig};

    // === Requirement: Read .env file (should be BLOCKED) ===
    let requirement = "读取 .env 配置文件";

    // === Sandbox check ===
    let sandbox = SandboxConfig::default_strict();
    let patterns = DeniedPatterns::default_patterns();

    // === Verify blocked ===
    assert!(
        !sandbox.is_path_allowed(".env"),
        "Sandbox should block .env files"
    );
    assert!(
        patterns.is_denied(".env").is_some(),
        "DeniedPatterns should flag .env"
    );

    // === Result: Requirement CANNOT be fulfilled (security) ===
    // This is correct behavior - system protects sensitive files
}
