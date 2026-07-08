// End-to-End Integration Tests for Hermes Game Operator

use crate::operator::{
    GodotTaskExecutor, TaskExecutionReport,
    PathGuard, CommandGuard,
};
use std::path::PathBuf;

/// Test complete task execution workflow
#[tokio::test]
async fn test_complete_task_workflow() {
    // Setup: Create a minimal Godot project structure
    let test_project_path = PathBuf::from("test-godot-project");

    // Verify test project exists
    assert!(test_project_path.exists(), "Test project directory should exist");
    assert!(test_project_path.join("project.godot").exists(), "project.godot should exist");

    // Execute task
    let mut executor = GodotTaskExecutor::new(
        "test_task_001".to_string(),
        test_project_path,
        "给 Player 添加二段跳能力".to_string(),
    );

    let result = executor.execute().await;

    // Verify execution succeeded
    assert!(result.is_ok(), "Task execution should succeed");

    let report = result.unwrap();

    // Verify report
    assert_eq!(report.task_id, "test_task_001");
    assert!(report.success, "Task should complete successfully");
    assert!(report.total_steps > 0, "Should have execution steps");
    assert!(!report.events.is_empty(), "Should have events");

    // Verify events contain expected phases
    let event_types: Vec<_> = report.events.iter().map(|e| &e.event_type).collect();

    assert!(event_types.iter().any(|t| matches!(t, crate::operator::OperatorEventType::TaskStarted)));
    assert!(event_types.iter().any(|t| matches!(t, crate::operator::OperatorEventType::ProjectAnalyzed)));
    assert!(event_types.iter().any(|t| matches!(t, crate::operator::OperatorEventType::PlanReady)));
    assert!(event_types.iter().any(|t| matches!(t, crate::operator::OperatorEventType::TaskCompleted)));
}

/// Test project analysis
#[tokio::test]
async fn test_project_analysis() {
    let test_project_path = PathBuf::from("test-godot-project");

    let mut executor = GodotTaskExecutor::new(
        "test_task_002".to_string(),
        test_project_path,
        "Analyze project".to_string(),
    );

    let analysis = executor.analyze_project().await;

    assert!(analysis.is_ok(), "Project analysis should succeed");

    let result = analysis.unwrap();

    // Verify analysis results
    assert!(!result.project_name.is_empty(), "Project name should not be empty");
    assert!(!result.scripts.is_empty(), "Should find scripts");
    assert!(!result.scenes.is_empty(), "Should find scenes");

    // Verify specific files (matching actual test project structure)
    let has_player = result.scripts.iter().any(|s| s.contains("Player.gd"));
    let has_main = result.scenes.iter().any(|s| s.contains("Main.tscn") || s.contains("main.tscn"));

    assert!(has_player, "Should find Player.gd");
    assert!(has_main, "Should find Main.tscn or main.tscn");
}

/// Test plan generation
#[tokio::test]
async fn test_plan_generation() {
    let test_project_path = PathBuf::from("test-godot-project");

    let mut executor = GodotTaskExecutor::new(
        "test_task_003".to_string(),
        test_project_path,
        "给 Player 添加二段跳".to_string(),
    );

    let analysis = executor.analyze_project().await.unwrap();
    let plan = executor.generate_plan(&analysis).await;

    assert!(plan.is_ok(), "Plan generation should succeed");

    let result = plan.unwrap();

    assert_eq!(result.task_id, "test_task_003");
    assert!(!result.goal.is_empty(), "Goal should not be empty");
    assert!(!result.steps.is_empty(), "Should have steps");

    // Verify steps contain analysis and implementation
    let has_analysis = result.steps.iter().any(|s| s.description.contains("分析") || s.description.contains("Analyze"));
    let has_implementation = result.steps.iter().any(|s| s.description.contains("实现") || s.description.contains("Implement"));

    assert!(has_analysis, "Should have analysis step");
    assert!(has_implementation, "Should have implementation step");
}

/// Test path guard security
#[test]
fn test_path_guard() {
    let allowed_roots = vec![PathBuf::from("test-godot-project")];
    let guard = PathGuard::new(allowed_roots);

    // Test valid path
    let valid_path = PathBuf::from("test-godot-project/scripts/Player.gd");
    assert!(guard.validate_file(&valid_path).is_ok(), "Valid path should pass");

    // Test invalid path
    let invalid_path = PathBuf::from("/etc/passwd");
    assert!(guard.validate_file(&invalid_path).is_err(), "Invalid path should fail");
}

/// Test command guard security
#[test]
fn test_command_guard() {
    let guard = CommandGuard::new();

    // Test allowed command
    let allowed = guard.validate_command("godot --version");
    assert!(allowed.is_ok(), "Godot command should be allowed");

    // Test forbidden command
    let forbidden = guard.validate_command("rm -rf /");
    assert!(forbidden.is_err(), "rm command should be forbidden");
}
