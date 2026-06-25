// Comprehensive Game Integration Tests
// Phase 4 Day 12-14: End-to-end testing and documentation

#[cfg(test)]
mod integration_tests {
    use crate::game_detector::{GameDetector, GameEngine};
    use crate::game_engine::GameEngineManager;
    use crate::game_launcher::GameLauncher;
    use crate::game_process_monitor::ProcessMonitor;
    use crate::game_error_handler::{GameErrorHandler, create_error, GameErrorType, ErrorContext};
    use std::path::PathBuf;
    use tempfile::tempdir;

    // ===== Detection Tests =====

    #[test]
    fn test_detect_godot_project() {
        let dir = tempdir().unwrap();
        let project_file = dir.path().join("project.godot");
        std::fs::write(&project_file, r#"[application]
config/name="Test Game"
config/features=PackedStringArray("4.2", "GL Compatibility")
"#).unwrap();

        let info = GameDetector::detect(dir.path()).unwrap();
        assert_eq!(info.engine, GameEngine::Godot);
        assert_eq!(info.project_name, "Test Game");
    }

    #[test]
    fn test_detect_unity_project() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Assets")).unwrap();
        std::fs::create_dir_all(dir.path().join("ProjectSettings")).unwrap();

        let version_file = dir.path().join("ProjectSettings/ProjectVersion.txt");
        std::fs::write(&version_file, "m_EditorVersion: 2021.3.0f1\n").unwrap();

        let info = GameDetector::detect(dir.path()).unwrap();
        assert_eq!(info.engine, GameEngine::Unity);
    }

    #[test]
    fn test_detect_invalid_project() {
        let dir = tempdir().unwrap();
        let result = GameDetector::detect(dir.path());
        assert!(result.is_err());
    }

    // ===== Engine Tests =====

    #[test]
    fn test_engine_manager_creation() {
        let _manager = GameEngineManager::new();
        // Should not panic
    }

    // ===== Launcher Tests =====

    #[test]
    fn test_launcher_creation() {
        let _launcher = GameLauncher::new();
        // Should not panic
    }

    #[test]
    fn test_launcher_no_running_games() {
        let launcher = GameLauncher::new();
        let running = launcher.get_all_running();
        assert_eq!(running.len(), 0);
    }

    // ===== Process Monitor Tests =====

    #[test]
    fn test_process_monitor_creation() {
        let _monitor = ProcessMonitor::new(100);
        // Should not panic
    }

    #[test]
    fn test_process_monitor_thresholds() {
        let mut monitor = ProcessMonitor::new(100);
        monitor.set_memory_threshold(1024.0);
        monitor.set_cpu_threshold(80.0);
        // Should not panic
    }

    // ===== Error Handler Tests =====

    #[test]
    fn test_error_handler_creation() {
        let _handler = GameErrorHandler::new();
        // Should not panic
    }

    #[test]
    fn test_error_handler_known_issues() {
        let mut handler = GameErrorHandler::new();

        let error = create_error(
            GameErrorType::MissingDependencies,
            "Export template not found for platform",
            None,
            true,
        );

        let context = ErrorContext {
            engine: Some("Godot".to_string()),
            project_path: None,
            build_target: None,
            process_id: None,
        };

        let enhanced = handler.handle_error(error, &context);
        assert!(enhanced.suggestion.is_some());
    }

    #[test]
    fn test_error_history() {
        let mut handler = GameErrorHandler::new();

        for i in 0..10 {
            let error = create_error(
                GameErrorType::BuildFailed,
                format!("Build error {}", i),
                None,
                true,
            );

            handler.handle_error(error, &ErrorContext {
                engine: None,
                project_path: None,
                build_target: None,
                process_id: None,
            });
        }

        assert_eq!(handler.get_error_history().len(), 10);
    }

    // ===== Integration Tests =====

    #[test]
    fn test_full_workflow_detection() {
        // Create a mock Godot project
        let dir = tempdir().unwrap();
        let project_file = dir.path().join("project.godot");
        std::fs::write(&project_file, r#"[application]
config/name="Integration Test Game"
config/features=PackedStringArray("4.2", "GL Compatibility")
"#).unwrap();

        // Step 1: Detect
        let info = GameDetector::detect(dir.path()).unwrap();
        assert_eq!(info.engine, GameEngine::Godot);

        // Step 2: Create engine manager
        let _manager = GameEngineManager::new();

        // Step 3: Create launcher
        let _launcher = GameLauncher::new();

        // Step 4: Create monitor
        let _monitor = ProcessMonitor::new(100);

        // Step 5: Create error handler
        let _handler = GameErrorHandler::new();
    }

    #[test]
    fn test_error_recovery_workflow() {
        let mut handler = GameErrorHandler::new();

        // Simulate multiple errors
        let errors = vec![
            (GameErrorType::MissingDependencies, "Export template missing"),
            (GameErrorType::BuildFailed, "Compilation failed"),
            (GameErrorType::LaunchFailed, "Executable not found"),
        ];

        for (error_type, message) in errors {
            let error = create_error(error_type, message, None, true);
            let context = ErrorContext {
                engine: Some("Godot".to_string()),
                project_path: Some("/tmp/test".to_string()),
                build_target: Some("windows".to_string()),
                process_id: None,
            };
            handler.handle_error(error, &context);
        }

        // Verify history
        assert_eq!(handler.get_error_history().len(), 3);

        // Get frequent errors
        let frequent = handler.get_frequent_errors(5);
        assert!(frequent.len() > 0);
    }
}

// ===== Performance Tests =====

#[cfg(test)]
mod performance_tests {
    use crate::game_process_monitor::ProcessMonitor;
    use std::time::Instant;

    #[test]
    fn test_monitor_performance() {
        let mut monitor = ProcessMonitor::new(10000);

        let start = Instant::now();

        // Record 1000 metrics
        for i in 0..1000 {
            let metrics = crate::game_process_monitor::ProcessMetrics {
                process_id: 1234,
                memory_mb: 500.0 + i as f64,
                cpu_percent: 50.0,
                thread_count: 4,
                handle_count: 100,
                timestamp: i as u64,
            };
            monitor.record_metrics(metrics);
        }

        let elapsed = start.elapsed();

        // Should complete in less than 100ms
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_summary_calculation_performance() {
        let mut monitor = ProcessMonitor::new(10000);

        // Record 1000 metrics
        for i in 0..1000 {
            let metrics = crate::game_process_monitor::ProcessMetrics {
                process_id: 1234,
                memory_mb: 500.0,
                cpu_percent: 50.0,
                thread_count: 4,
                handle_count: 100,
                timestamp: i as u64,
            };
            monitor.record_metrics(metrics);
        }

        let start = Instant::now();
        let _summary = monitor.get_summary();
        let elapsed = start.elapsed();

        // Should complete in less than 10ms
        assert!(elapsed.as_millis() < 10);
    }
}
