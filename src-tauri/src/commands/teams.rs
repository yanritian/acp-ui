use crate::AppState;
use crate::{circuit_breaker, self_healing, smart_router, team_dag};
use tauri::State;

// ===== Agent Teams Platform Commands =====

/// Analyze task complexity using Smart Router
#[tauri::command]
pub fn analyze_task_complexity(
    input: String,
    input_type: String,
) -> Result<smart_router::RouteDecision, String> {
    let analyzer = smart_router::TaskAnalyzer::new();
    let input_type_enum = match input_type.as_str() {
        "text" => smart_router::InputType::Text,
        "image" => smart_router::InputType::Image,
        "document" => smart_router::InputType::Document,
        "code" => smart_router::InputType::Code,
        "command" => smart_router::InputType::Command,
        _ => smart_router::InputType::Text,
    };
    Ok(analyzer.analyze(&input, input_type_enum))
}

/// Get circuit breaker status for an agent
#[tauri::command]
pub fn get_circuit_breaker_status(
    target: String,
    state: State<AppState>,
) -> Result<circuit_breaker::CircuitBreakerRecord, String> {
    let manager = state
        .circuit_breaker_manager
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(manager.get_breaker(&target))
}

/// Check if circuit breaker allows request
#[tauri::command]
pub fn is_circuit_breaker_allowed(target: String, state: State<AppState>) -> Result<bool, String> {
    let manager = state
        .circuit_breaker_manager
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(manager.is_allowed(&target))
}

/// Reset circuit breaker for an agent
#[tauri::command]
pub fn reset_circuit_breaker(target: String, state: State<AppState>) -> Result<(), String> {
    let manager = state
        .circuit_breaker_manager
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    manager.reset(&target);
    Ok(())
}

/// Get all circuit breaker states
#[tauri::command]
pub fn get_all_circuit_breakers(
    state: State<AppState>,
) -> Result<Vec<circuit_breaker::CircuitBreakerRecord>, String> {
    let manager = state
        .circuit_breaker_manager
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(manager.get_all_states())
}

/// Create DAG execution plan
#[tauri::command]
pub fn create_dag_plan(
    team_id: String,
    members: Vec<team_dag::TeamMember>,
    sync_points: Vec<team_dag::SyncPoint>,
    strategy: String,
    state: State<AppState>,
) -> Result<team_dag::ExecutionPlan, String> {
    let strategy_enum = match strategy.as_str() {
        "parallel" => team_dag::ExecutionStrategy::Parallel,
        "sequential" => team_dag::ExecutionStrategy::Sequential,
        "hybrid" => team_dag::ExecutionStrategy::Hybrid,
        _ => team_dag::ExecutionStrategy::Hybrid,
    };

    let mut engine = state
        .dag_engine
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let plan = engine.create_plan(&team_id, members, sync_points, strategy_enum)?;
    engine.register_plan(plan.clone());
    Ok(plan)
}

/// Get DAG plan progress
#[tauri::command]
pub fn get_dag_plan_progress(plan_id: String, state: State<AppState>) -> Result<f64, String> {
    let engine = state
        .dag_engine
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let plan = engine
        .get_plan(&plan_id)
        .ok_or_else(|| format!("Plan {} not found", plan_id))?;
    Ok(engine.get_progress(plan))
}

/// Check anomaly for a metric
#[tauri::command]
pub fn check_anomaly(
    metric_name: String,
    current_value: f64,
    state: State<AppState>,
) -> Result<Option<self_healing::AnomalyRecord>, String> {
    let detector = state
        .anomaly_detector
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(detector.check(&metric_name, current_value))
}

/// Update baseline for anomaly detection
#[tauri::command]
pub fn update_anomaly_baseline(
    metric_name: String,
    value: f64,
    state: State<AppState>,
) -> Result<(), String> {
    let mut detector = state
        .anomaly_detector
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    detector.update_baseline(&metric_name, value);
    Ok(())
}
