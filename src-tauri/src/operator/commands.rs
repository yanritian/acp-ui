// Operator Tauri Commands
// Exposes Operator Control Plane to frontend

use crate::operator::{
    OperatorTask, OperatorTaskStatus, OperatorEvent, ApprovalRequest,
    TaskStateMachine, StartTaskRequest, StartTaskResponse,
    ApproveRequest, RedirectRequest, TaskSummary,
};
use crate::domains::games::godot::{GodotProjectAnalyzer, GodotProjectInfo};
use tauri::State;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// ============================================================================
// Application State
// ============================================================================

pub struct OperatorState {
    pub tasks: HashMap<String, OperatorTask>,
    pub state_machines: HashMap<String, TaskStateMachine>,
    pub events: HashMap<String, Vec<OperatorEvent>>,
    pub approvals: HashMap<String, Vec<ApprovalRequest>>,
}

impl OperatorState {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            state_machines: HashMap::new(),
            events: HashMap::new(),
            approvals: HashMap::new(),
        }
    }
}

// ============================================================================
// Task Management Commands
// ============================================================================

#[tauri::command]
pub async fn operator_start_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: StartTaskRequest,
) -> Result<StartTaskResponse, String> {
    let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());

    let task = OperatorTask {
        task_id: task_id.clone(),
        domain: request.domain.clone(),
        project_path: request.project_path.clone(),
        goal: request.goal.clone(),
        status: OperatorTaskStatus::Planning,
        mode: request.mode.unwrap_or(crate::operator::TaskMode::ProposeThenApply),
        approval_policy: request.approval_policy.unwrap_or(crate::operator::ApprovalPolicy::SafeDefault),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        completed_at: None,
        summary: None,
        error: None,
    };

    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.tasks.insert(task_id.clone(), task);

    // Create state machine and transition to Planning
    let mut state_machine = TaskStateMachine::new(task_id.clone());
    state_machine.start_task().map_err(|e| e.to_string())?; // Idle -> Planning
    state.state_machines.insert(task_id.clone(), state_machine);

    state.events.insert(task_id.clone(), Vec::new());
    state.approvals.insert(task_id.clone(), Vec::new());

    // Emit initial event
    let event = OperatorEvent {
        event_id: format!("evt_{}_0", task_id),
        task_id: task_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        event_type: crate::operator::OperatorEventType::TaskCreated,
        level: crate::operator::EventLevel::Info,
        title: "Task created".to_string(),
        message: Some(format!("Starting task: {}", request.goal)),
        source: "operator".to_string(),
        payload: None,
    };
    state.events.get_mut(&task_id).unwrap().push(event);

    // Emit planning started event
    let event = OperatorEvent {
        event_id: format!("evt_{}_1", task_id),
        task_id: task_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        event_type: crate::operator::OperatorEventType::PlanStarted,
        level: crate::operator::EventLevel::Info,
        title: "Planning started".to_string(),
        message: Some("Task is now in planning phase".to_string()),
        source: "operator".to_string(),
        payload: None,
    };
    state.events.get_mut(&task_id).unwrap().push(event);
    state.events.get_mut(&task_id).unwrap().push(event);

    Ok(StartTaskResponse {
        task_id,
        status: OperatorTaskStatus::Planning,
        event_stream: format!("operator://tasks/{}/events", task_id),
    })
}

#[tauri::command]
pub async fn operator_get_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<OperatorTask, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.tasks.get(&task_id)
        .cloned()
        .ok_or_else(|| format!("Task not found: {}", task_id))
}

#[tauri::command]
pub async fn operator_list_tasks(
    state: State<'_, Arc<Mutex<OperatorState>>>,
) -> Result<Vec<OperatorTask>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.tasks.values().cloned().collect())
}

#[tauri::command]
pub async fn operator_pause_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    let machine = state.state_machines.get_mut(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    machine.pause().map_err(|e| e.to_string())?;

    // Update task status
    if let Some(task) = state.tasks.get_mut(&task_id) {
        task.status = OperatorTaskStatus::Paused;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    Ok(())
}

#[tauri::command]
pub async fn operator_resume_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    let machine = state.state_machines.get_mut(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    machine.resume().map_err(|e| e.to_string())?;

    if let Some(task) = state.tasks.get_mut(&task_id) {
        task.status = OperatorTaskStatus::Running;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    Ok(())
}

#[tauri::command]
pub async fn operator_stop_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    let machine = state.state_machines.get_mut(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    machine.stop().map_err(|e| e.to_string())?;
    machine.cleanup_done().map_err(|e| e.to_string())?;

    if let Some(task) = state.tasks.get_mut(&task_id) {
        task.status = OperatorTaskStatus::Cancelled;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

#[tauri::command]
pub async fn operator_redirect_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: RedirectRequest,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    let machine = state.state_machines.get_mut(&request.task_id)
        .ok_or_else(|| format!("Task not found: {}", request.task_id))?;

    machine.redirect().map_err(|e| e.to_string())?;
    machine.replan().map_err(|e| e.to_string())?;

    if let Some(task) = state.tasks.get_mut(&request.task_id) {
        task.status = OperatorTaskStatus::Planning;
        task.goal = request.new_goal;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    Ok(())
}

#[tauri::command]
pub async fn operator_approve(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: ApproveRequest,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Find and update approval
    if let Some(approvals) = state.approvals.get_mut(&request.task_id) {
        if let Some(approval) = approvals.iter_mut().find(|a| a.approval_id == request.approval_id) {
            approval.decision = Some(request.decision.clone());
            approval.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            approval.resolved_by = Some("user".to_string());
        }
    }

    // Drive state machine based on decision
    let new_status = match request.decision {
        crate::operator::ApprovalDecision::Approve => {
            if let Some(machine) = state.state_machines.get_mut(&request.task_id) {
                machine.approve().map_err(|e| e.to_string())?; // WaitingApproval -> Running
            }
            OperatorTaskStatus::Running
        }
        crate::operator::ApprovalDecision::Reject => {
            if let Some(machine) = state.state_machines.get_mut(&request.task_id) {
                machine.reject().map_err(|e| e.to_string())?; // WaitingApproval -> Cancelled
            }
            OperatorTaskStatus::Cancelled
        }
        crate::operator::ApprovalDecision::RequestChanges => {
            if let Some(machine) = state.state_machines.get_mut(&request.task_id) {
                machine.replan().map_err(|e| e.to_string())?; // WaitingApproval -> Planning
            }
            OperatorTaskStatus::Planning
        }
    };

    // Update task status
    if let Some(task) = state.tasks.get_mut(&request.task_id) {
        task.status = new_status;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    // Emit approval event
    let event_type = match request.decision {
        crate::operator::ApprovalDecision::Approve => crate::operator::OperatorEventType::ApprovalGranted,
        crate::operator::ApprovalDecision::Reject => crate::operator::OperatorEventType::ApprovalRejected,
        crate::operator::ApprovalDecision::RequestChanges => crate::operator::OperatorEventType::ApprovalRequested,
    };

    let event = OperatorEvent {
        event_id: format!("evt_approve_{}", request.approval_id),
        task_id: request.task_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        event_type,
        level: crate::operator::EventLevel::Info,
        title: format!("Approval {:?}", request.decision),
        message: request.comment,
        source: "operator".to_string(),
        payload: None,
    };

    if let Some(events) = state.events.get_mut(&request.task_id) {
        events.push(event);
    }

    Ok(())
}

#[tauri::command]
pub async fn operator_get_pending_approvals(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<Vec<ApprovalRequest>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let approvals = state.approvals.get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    Ok(approvals.iter()
        .filter(|a| a.decision.is_none())
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn operator_list_events(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    limit: Option<usize>,
) -> Result<Vec<OperatorEvent>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let events = state.events.get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let limit = limit.unwrap_or(100);
    Ok(events.iter().rev().take(limit).cloned().collect())
}

#[tauri::command]
pub async fn operator_get_task_summary(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<TaskSummary, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let task = state.tasks.get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let events = state.events.get(&task_id).cloned().unwrap_or_default();

    // Calculate duration
    let duration_seconds = if let (Some(started), Some(completed)) = (&task.started_at, &task.completed_at) {
        let start = chrono::DateTime::parse_from_rfc3339(started).unwrap_or_default();
        let end = chrono::DateTime::parse_from_rfc3339(completed).unwrap_or_default();
        (end - start).num_seconds() as u64
    } else {
        0
    };

    Ok(TaskSummary {
        task_id: task_id.clone(),
        status: task.status.clone(),
        goal: task.goal.clone(),
        summary: task.summary.clone().unwrap_or_else(|| "Task completed".to_string()),
        files_changed: Vec::new(), // TODO: Track file changes
        files_created: Vec::new(),
        files_deleted: Vec::new(),
        duration_seconds,
        iterations: events.len() as u32,
        errors: Vec::new(),
        warnings: Vec::new(),
    })
}

// ============================================================================
// File Tool Commands
// ============================================================================

#[tauri::command]
pub async fn operator_file_read(
    path: String,
    allowed_roots: Vec<String>,
) -> Result<FileReadResult, String> {
    let path = std::path::Path::new(&path);
    let roots: Vec<std::path::PathBuf> = allowed_roots.iter().map(|s| std::path::PathBuf::from(s)).collect();
    let path_guard = PathGuard::new(roots);

    file_read(path, &path_guard).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn operator_file_patch(
    path: String,
    new_content: String,
    allowed_roots: Vec<String>,
    create_backup: bool,
) -> Result<FilePatchResult, String> {
    let path = std::path::Path::new(&path);
    let roots: Vec<std::path::PathBuf> = allowed_roots.iter().map(|s| std::path::PathBuf::from(s)).collect();
    let path_guard = PathGuard::new(roots);

    file_patch(path, &new_content, &path_guard, create_backup).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn operator_file_patch_preview(
    path: String,
    new_content: String,
    allowed_roots: Vec<String>,
) -> Result<FilePatch, String> {
    let path = std::path::Path::new(&path);
    let roots: Vec<std::path::PathBuf> = allowed_roots.iter().map(|s| std::path::PathBuf::from(s)).collect();
    let path_guard = PathGuard::new(roots);

    file_patch_preview(path, &new_content, &path_guard).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn operator_file_list(
    path: String,
    allowed_roots: Vec<String>,
) -> Result<FileListResult, String> {
    let path = std::path::Path::new(&path);
    let roots: Vec<std::path::PathBuf> = allowed_roots.iter().map(|s| std::path::PathBuf::from(s)).collect();
    let path_guard = PathGuard::new(roots);

    file_list(path, &path_guard).map_err(|e| e.to_string())
}

// ============================================================================
// Godot Commands
// ============================================================================

#[tauri::command]
pub async fn godot_detect_project(path: String) -> Result<bool, String> {
    let path = std::path::Path::new(&path);
    Ok(GodotProjectAnalyzer::detect_project(path))
}

#[tauri::command]
pub async fn godot_analyze_project(project_path: String) -> Result<GodotProjectInfo, String> {
    let path = std::path::Path::new(&project_path);
    GodotProjectAnalyzer::analyze_project(path).map_err(|e| e.to_string())
}
