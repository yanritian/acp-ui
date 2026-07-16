// Operator Tauri Commands
// Exposes Operator Control Plane to frontend

use crate::domains::games::godot::{GodotProjectAnalyzer, GodotProjectInfo};
use crate::operator::{
    file_tools::{
        file_list, file_patch_preview, file_read, FileListResult, FilePatch, FilePatchResult,
        FileReadResult,
    },
    ApprovalDecision, ApprovalLevel, ApprovalPolicy, ApprovalPreview, ApprovalRequest,
    ApproveRequest, EventLevel, FileDiffPreview, GodotValidationResult, GodotValidationStatus,
    OperatorEvent, OperatorEventType, OperatorTask, OperatorTaskStatus, PatchOperation, PathGuard,
    PreparedPatchSet, RedirectRequest, StartTaskRequest, StartTaskResponse, TaskMode,
    TaskStateMachine, TaskSummary,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::State;

// ============================================================================
// Application State
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeRecord {
    pub path: String,
    pub change_type: String, // "modified", "created", "deleted"
    pub timestamp: String,
}

pub struct OperatorState {
    pub tasks: HashMap<String, OperatorTask>,
    pub state_machines: HashMap<String, TaskStateMachine>,
    pub events: HashMap<String, Vec<OperatorEvent>>,
    pub approvals: HashMap<String, Vec<ApprovalRequest>>,
    pub file_changes: HashMap<String, Vec<FileChangeRecord>>,
    pub pending_patches: HashMap<String, PreparedPatchSet>,
    pub(crate) pending_validations: HashMap<String, PendingGodotValidation>,
    pub(crate) godot_validation_discovery: GodotValidationDiscovery,
    pub(crate) recovery_origins: HashMap<String, OperatorTaskStatus>,
    pub(crate) persistence: Option<crate::operator::OperatorStateStore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GodotValidationDiscovery {
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PendingGodotValidation {
    pub(crate) project_root: PathBuf,
    pub(crate) patch_id: String,
    pub(crate) backup_root: String,
    pub(crate) files_applied: usize,
}

impl OperatorState {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            state_machines: HashMap::new(),
            events: HashMap::new(),
            approvals: HashMap::new(),
            file_changes: HashMap::new(),
            pending_patches: HashMap::new(),
            pending_validations: HashMap::new(),
            godot_validation_discovery: if cfg!(test) {
                GodotValidationDiscovery::Disabled
            } else {
                GodotValidationDiscovery::Auto
            },
            recovery_origins: HashMap::new(),
            persistence: None,
        }
    }

    pub fn new_persistent(db_path: impl AsRef<Path>) -> Result<Self, String> {
        let store = crate::operator::OperatorStateStore::open(db_path)?;
        let loaded = store.load()?;
        let mut state = Self {
            tasks: loaded.tasks,
            state_machines: HashMap::new(),
            events: loaded.events,
            approvals: loaded.approvals,
            file_changes: loaded.file_changes,
            pending_patches: loaded.pending_patches,
            pending_validations: loaded.pending_validations,
            godot_validation_discovery: if cfg!(test) {
                GodotValidationDiscovery::Disabled
            } else {
                GodotValidationDiscovery::Auto
            },
            recovery_origins: loaded.recovery_origins,
            persistence: Some(store),
        };
        let mut recovery_changed = false;

        for failure in loaded.patch_failures {
            recovery_changed = true;
            state.pending_validations.remove(&failure.task_id);
            state.recovery_origins.remove(&failure.task_id);
            let now = chrono::Utc::now().to_rfc3339();
            if let Some(task) = state.tasks.get_mut(&failure.task_id) {
                task.status = OperatorTaskStatus::Failed;
                task.updated_at = now.clone();
                task.completed_at = Some(now.clone());
                task.error = Some(format!(
                    "Pending patch '{}' could not be recovered: {}",
                    failure.patch_id, failure.error
                ));
                task.summary = Some(
                    "The pending patch became stale or invalid while the application was stopped."
                        .to_string(),
                );
            }
            if let Some(approvals) = state.approvals.get_mut(&failure.task_id) {
                for approval in approvals.iter_mut().filter(|approval| {
                    approval.action == PATCH_APPLY_ACTION && approval.decision.is_none()
                }) {
                    approval.decision = Some(ApprovalDecision::RequestChanges);
                    approval.resolved_at = Some(now.clone());
                    approval.resolved_by = Some("operator_recovery".to_string());
                }
            }
            push_operator_event(
                &mut state,
                &failure.task_id,
                OperatorEventType::TaskFailed,
                EventLevel::Error,
                "Pending patch recovery failed",
                Some(failure.error),
                "operator_recovery",
                Some(json!({ "patch_id": failure.patch_id })),
            );
        }

        let task_ids = state.tasks.keys().cloned().collect::<Vec<_>>();
        for task_id in &task_ids {
            let status = state
                .tasks
                .get(task_id)
                .map(|task| task.status.clone())
                .ok_or_else(|| format!("Recovered task disappeared: {}", task_id))?;
            match status {
                OperatorTaskStatus::Idle
                | OperatorTaskStatus::Planning
                | OperatorTaskStatus::Running
                | OperatorTaskStatus::Redirecting => {
                    recovery_changed = true;
                    state
                        .recovery_origins
                        .entry(task_id.clone())
                        .or_insert_with(|| status.clone());
                    if let Some(task) = state.tasks.get_mut(task_id) {
                        task.status = OperatorTaskStatus::Paused;
                        task.updated_at = chrono::Utc::now().to_rfc3339();
                        task.summary = Some(format!(
                            "Task was interrupted while {:?}; explicit resume is required.",
                            status
                        ));
                    }
                    push_operator_event(
                        &mut state,
                        task_id,
                        OperatorEventType::TaskPaused,
                        EventLevel::Warning,
                        "Task interrupted by application restart",
                        Some(
                            "No agent or validation process was restarted automatically."
                                .to_string(),
                        ),
                        "operator_recovery",
                        Some(json!({ "recovered_from": status })),
                    );
                }
                OperatorTaskStatus::Cancelling => {
                    recovery_changed = true;
                    state.recovery_origins.remove(task_id);
                    if let Some(task) = state.tasks.get_mut(task_id) {
                        let now = chrono::Utc::now().to_rfc3339();
                        task.status = OperatorTaskStatus::Cancelled;
                        task.updated_at = now.clone();
                        task.completed_at = Some(now);
                        task.summary = Some(
                            "Cancellation completed while recovering from application restart."
                                .to_string(),
                        );
                    }
                    push_operator_event(
                        &mut state,
                        task_id,
                        OperatorEventType::TaskCancelled,
                        EventLevel::Info,
                        "Interrupted cancellation completed",
                        None,
                        "operator_recovery",
                        None,
                    );
                }
                OperatorTaskStatus::WaitingApproval
                | OperatorTaskStatus::Paused
                | OperatorTaskStatus::Cancelled
                | OperatorTaskStatus::Failed
                | OperatorTaskStatus::Completed => {}
            }
        }

        for task_id in task_ids {
            let task = state
                .tasks
                .get(&task_id)
                .ok_or_else(|| format!("Recovered task disappeared: {}", task_id))?;
            let events = state.events.get(&task_id).cloned().unwrap_or_default();
            state.state_machines.insert(
                task_id.clone(),
                TaskStateMachine::restore(task_id, task.status.clone(), events),
            );
        }

        if recovery_changed {
            state.persist()?;
        }
        Ok(state)
    }

    pub fn new_persistent_default() -> Result<Self, String> {
        Self::new_persistent(crate::operator::default_operator_state_db_path()?)
    }

    pub(crate) fn persist(&self) -> Result<(), String> {
        match &self.persistence {
            Some(store) => store.save(self),
            None => Ok(()),
        }
    }
}

const PATCH_APPLY_ACTION: &str = "operator.patch.apply";

pub(crate) fn check_expected_revision(
    state: &OperatorState,
    task_id: &str,
    expected_revision: Option<u64>,
) -> Result<u64, String> {
    let task = state
        .tasks
        .get(task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;
    if let Some(expected) = expected_revision {
        if expected != task.revision {
            return Err(format!(
                "REVISION_CONFLICT: task '{}' is at revision {}, expected {}",
                task_id, task.revision, expected
            ));
        }
    }
    Ok(task.revision)
}

pub(crate) fn start_task_in_state(
    state: &mut OperatorState,
    request: StartTaskRequest,
) -> Result<StartTaskResponse, String> {
    if request.domain.trim().is_empty() {
        return Err("Domain cannot be empty".to_string());
    }
    if request.goal.trim().is_empty() {
        return Err("Goal cannot be empty".to_string());
    }
    if request.project_path.trim().is_empty() {
        return Err("Project path cannot be empty".to_string());
    }
    if state.persistence.is_some() {
        let project_path = PathBuf::from(&request.project_path);
        if !project_path.is_absolute() || !is_d_drive_path(&project_path) {
            return Err(format!(
                "Persistent Operator tasks require an absolute D: project path, got '{}'",
                request.project_path
            ));
        }
    }

    let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());
    let now = chrono::Utc::now().to_rfc3339();
    let mode = request.mode.unwrap_or(TaskMode::ProposeThenApply);
    let approval_policy = request
        .approval_policy
        .unwrap_or(ApprovalPolicy::SafeDefault);

    let task = OperatorTask {
        task_id: task_id.clone(),
        revision: 0, // Initial revision
        domain: request.domain.clone(),
        project_path: request.project_path.clone(),
        goal: request.goal.clone(),
        status: OperatorTaskStatus::Planning,
        mode,
        approval_policy,
        created_at: now.clone(),
        updated_at: now.clone(),
        started_at: Some(now),
        completed_at: None,
        checkpoint_id: None,
        memory_snapshot_id: None,
        summary: None,
        error: None,
    };

    state.tasks.insert(task_id.clone(), task);

    let mut state_machine = TaskStateMachine::new(task_id.clone());
    state_machine.start_task().map_err(|e| e.to_string())?;
    state.state_machines.insert(task_id.clone(), state_machine);

    state.events.insert(task_id.clone(), Vec::new());
    state.approvals.insert(task_id.clone(), Vec::new());
    state.file_changes.insert(task_id.clone(), Vec::new());

    push_operator_event(
        state,
        &task_id,
        OperatorEventType::TaskCreated,
        EventLevel::Info,
        "Task created",
        Some(format!("Starting task: {}", request.goal)),
        "operator",
        None,
    );
    push_operator_event(
        state,
        &task_id,
        OperatorEventType::PlanStarted,
        EventLevel::Info,
        "Planning started",
        Some("Task is now in planning phase".to_string()),
        "operator",
        None,
    );

    if request.domain == "game.godot" {
        if let Err(error) = advance_godot_task_to_plan(state, &task_id) {
            fail_task_in_state(state, &task_id, &error)?;
        }
    }

    let status = state
        .tasks
        .get(&task_id)
        .map(|task| task.status.clone())
        .unwrap_or(OperatorTaskStatus::Failed);
    let revision = state
        .tasks
        .get(&task_id)
        .map(|task| task.revision)
        .unwrap_or(0);
    let event_stream = format!("operator://tasks/{}/events", task_id);

    let response = StartTaskResponse {
        task_id,
        revision,
        status,
        event_stream,
    };
    if let Err(error) = state.persist() {
        state.tasks.remove(&response.task_id);
        state.state_machines.remove(&response.task_id);
        state.events.remove(&response.task_id);
        state.approvals.remove(&response.task_id);
        state.file_changes.remove(&response.task_id);
        state.pending_patches.remove(&response.task_id);
        state.pending_validations.remove(&response.task_id);
        state.recovery_origins.remove(&response.task_id);
        return Err(error);
    }
    Ok(response)
}

pub(crate) fn approve_task_in_state(
    state: &mut OperatorState,
    request: ApproveRequest,
) -> Result<(), String> {
    approve_task_in_state_with_cli_path(state, request, discover_hermes_cli_path())
}

pub(crate) fn approve_task_in_state_with_cli_path(
    state: &mut OperatorState,
    request: ApproveRequest,
    hermes_cli_path: Option<PathBuf>,
) -> Result<(), String> {
    check_expected_revision(state, &request.task_id, request.expected_revision)?;
    let approval = resolve_approval_record(state, &request)?;

    if approval.action == PATCH_APPLY_ACTION {
        let validation_pending = resolve_patch_approval_in_state(
            state,
            &request.task_id,
            approval,
            request.decision,
            request.comment,
        )?;
        if validation_pending {
            let godot_executable = configured_godot_executable(state);
            run_pending_godot_validation_in_state(state, &request.task_id, godot_executable)?;
        }
        state.persist()?;
        return Ok(());
    }

    match request.decision {
        ApprovalDecision::Approve => {
            {
                let machine = state
                    .state_machines
                    .get_mut(&request.task_id)
                    .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
                machine.approve().map_err(|e| e.to_string())?;
            }
            update_task_status(state, &request.task_id, OperatorTaskStatus::Running, None);
            push_operator_event(
                state,
                &request.task_id,
                OperatorEventType::ApprovalGranted,
                EventLevel::Info,
                "Approval granted",
                request.comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );
            run_approved_task_in_state(state, &request.task_id, hermes_cli_path)?;
        }
        ApprovalDecision::Reject => {
            {
                let machine = state
                    .state_machines
                    .get_mut(&request.task_id)
                    .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
                machine.reject().map_err(|e| e.to_string())?;
            }
            update_task_status(state, &request.task_id, OperatorTaskStatus::Cancelled, None);
            if let Some(task) = state.tasks.get_mut(&request.task_id) {
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.summary = Some("Task cancelled by approval rejection".to_string());
            }
            push_operator_event(
                state,
                &request.task_id,
                OperatorEventType::ApprovalRejected,
                EventLevel::Info,
                "Approval rejected",
                request.comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );
        }
        ApprovalDecision::RequestChanges => {
            resolve_plan_changes_request(
                state,
                &request.task_id,
                &approval.approval_id,
                request.comment,
            )?;
        }
    }

    state.persist()
}

#[derive(Clone)]
struct ApprovedTaskRun {
    task_id: String,
    hermes_cli_path: Option<PathBuf>,
    validate_godot: bool,
    godot_executable: Option<PathBuf>,
}

fn approve_task_in_state_for_background(
    state: &mut OperatorState,
    request: ApproveRequest,
    hermes_cli_path: Option<PathBuf>,
) -> Result<Option<ApprovedTaskRun>, String> {
    let approval = resolve_approval_record(state, &request)?;

    if approval.action == PATCH_APPLY_ACTION {
        let validation_pending = resolve_patch_approval_in_state(
            state,
            &request.task_id,
            approval,
            request.decision,
            request.comment,
        )?;
        if validation_pending {
            let godot_executable = configured_godot_executable(state);
            if godot_executable.is_none() {
                finish_godot_validation_in_state(
                    state,
                    &request.task_id,
                    GodotValidationResult::skipped(
                        "No D: Godot executable was found. Set GODOT_BIN to a D: Godot 4 executable.",
                    ),
                )?;
                return Ok(None);
            }
            return Ok(Some(ApprovedTaskRun {
                task_id: request.task_id,
                hermes_cli_path: None,
                validate_godot: true,
                godot_executable,
            }));
        }
        return Ok(None);
    }

    match request.decision {
        ApprovalDecision::Approve => {
            {
                let machine = state
                    .state_machines
                    .get_mut(&request.task_id)
                    .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
                machine.approve().map_err(|e| e.to_string())?;
            }
            update_task_status(state, &request.task_id, OperatorTaskStatus::Running, None);
            push_operator_event(
                state,
                &request.task_id,
                OperatorEventType::ApprovalGranted,
                EventLevel::Info,
                "Approval granted",
                request.comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );

            Ok(Some(ApprovedTaskRun {
                task_id: request.task_id,
                hermes_cli_path,
                validate_godot: false,
                godot_executable: None,
            }))
        }
        ApprovalDecision::Reject => {
            {
                let machine = state
                    .state_machines
                    .get_mut(&request.task_id)
                    .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
                machine.reject().map_err(|e| e.to_string())?;
            }
            update_task_status(state, &request.task_id, OperatorTaskStatus::Cancelled, None);
            if let Some(task) = state.tasks.get_mut(&request.task_id) {
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.summary = Some("Task cancelled by approval rejection".to_string());
            }
            push_operator_event(
                state,
                &request.task_id,
                OperatorEventType::ApprovalRejected,
                EventLevel::Info,
                "Approval rejected",
                request.comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );
            Ok(None)
        }
        ApprovalDecision::RequestChanges => {
            resolve_plan_changes_request(
                state,
                &request.task_id,
                &approval.approval_id,
                request.comment,
            )?;
            Ok(None)
        }
    }
}

fn resolve_approval_record(
    state: &mut OperatorState,
    request: &ApproveRequest,
) -> Result<ApprovalRequest, String> {
    let approval_index = {
        let approvals = state
            .approvals
            .get(&request.task_id)
            .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
        let index = approvals
            .iter()
            .position(|approval| approval.approval_id == request.approval_id)
            .ok_or_else(|| format!("Approval not found: {}", request.approval_id))?;
        let approval = &approvals[index];
        if approval.decision.is_some() {
            return Err(format!(
                "Approval already resolved: {}",
                request.approval_id
            ));
        }
        if approval.action == PATCH_APPLY_ACTION
            && matches!(&request.decision, ApprovalDecision::Approve)
            && !state.pending_patches.contains_key(&request.task_id)
        {
            return Err(format!(
                "Pending patch not found for task: {}",
                request.task_id
            ));
        }
        index
    };

    let approval = state
        .approvals
        .get_mut(&request.task_id)
        .and_then(|approvals| approvals.get_mut(approval_index))
        .ok_or_else(|| format!("Approval not found: {}", request.approval_id))?;
    approval.decision = Some(request.decision.clone());
    approval.resolved_at = Some(chrono::Utc::now().to_rfc3339());
    approval.resolved_by = Some("user".to_string());
    Ok(approval.clone())
}

fn resolve_plan_changes_request(
    state: &mut OperatorState,
    task_id: &str,
    approval_id: &str,
    comment: Option<String>,
) -> Result<(), String> {
    {
        let machine = state
            .state_machines
            .get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        machine
            .request_changes()
            .map_err(|error| error.to_string())?;
    }
    update_task_status(state, task_id, OperatorTaskStatus::Planning, None);
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ApprovalRejected,
        EventLevel::Info,
        "Plan changes requested",
        comment,
        "operator",
        Some(json!({ "approval_id": approval_id })),
    );
    let domain = state
        .tasks
        .get(task_id)
        .map(|task| task.domain.clone())
        .ok_or_else(|| format!("Task not found: {}", task_id))?;
    if domain == "game.godot" {
        advance_godot_task_to_plan(state, task_id)?;
    }
    Ok(())
}

fn resolve_patch_approval_in_state(
    state: &mut OperatorState,
    task_id: &str,
    approval: ApprovalRequest,
    decision: ApprovalDecision,
    comment: Option<String>,
) -> Result<bool, String> {
    match decision {
        ApprovalDecision::Approve => {
            {
                let machine = state
                    .state_machines
                    .get_mut(task_id)
                    .ok_or_else(|| format!("Task not found: {}", task_id))?;
                machine.approve().map_err(|error| error.to_string())?;
            }
            update_task_status(state, task_id, OperatorTaskStatus::Running, None);
            push_operator_event(
                state,
                task_id,
                OperatorEventType::ApprovalGranted,
                EventLevel::Info,
                "Patch approval granted",
                comment,
                "operator",
                Some(json!({
                    "approval_id": approval.approval_id,
                    "action": PATCH_APPLY_ACTION,
                })),
            );

            let prepared = state
                .pending_patches
                .remove(task_id)
                .ok_or_else(|| format!("Pending patch not found for task: {}", task_id))?;
            let project_root = prepared.project_root.clone();
            let validation = prepared.validation.clone();
            let backup_root = hermes_game_backup_path(task_id, &approval.approval_id);
            let applied = match crate::operator::apply_prepared_patch(&prepared, &backup_root) {
                Ok(applied) => applied,
                Err(error) => {
                    let error = error.to_string();
                    fail_task_in_state(state, task_id, &error)?;
                    return Err(error);
                }
            };

            for file in &applied.files {
                let absolute_path = project_root.join(Path::new(&file.path));
                let change_type = match file.operation {
                    PatchOperation::Create => "created",
                    PatchOperation::Replace => "modified",
                };
                record_file_change(
                    state,
                    task_id,
                    &absolute_path.to_string_lossy(),
                    change_type,
                );
            }
            push_operator_event(
                state,
                task_id,
                OperatorEventType::FilePatchApplied,
                EventLevel::Info,
                "Approved patch applied",
                Some(format!(
                    "Applied {} file change(s) with backups under {}.",
                    applied.files.len(),
                    applied.backup_root
                )),
                "operator_patch_engine",
                Some(json!({
                    "patch_id": applied.patch_id.clone(),
                    "files": applied.files.clone(),
                    "backup_root": applied.backup_root.clone(),
                    "validation": validation.clone(),
                })),
            );
            state.pending_validations.insert(
                task_id.to_string(),
                PendingGodotValidation {
                    project_root,
                    patch_id: applied.patch_id.clone(),
                    backup_root: applied.backup_root.clone(),
                    files_applied: applied.files.len(),
                },
            );
            if let Some(task) = state.tasks.get_mut(task_id) {
                task.status = OperatorTaskStatus::Running;
                task.updated_at = chrono::Utc::now().to_rfc3339();
                task.completed_at = None;
                task.summary = Some(format!(
                    "Applied {} approved Hermes Game file change(s); awaiting Godot validation. Backup: {}.",
                    applied.files.len(),
                    applied.backup_root
                ));
                task.error = None;
            }
            return Ok(true);
        }
        ApprovalDecision::Reject => {
            state.pending_patches.remove(task_id);
            {
                let machine = state
                    .state_machines
                    .get_mut(task_id)
                    .ok_or_else(|| format!("Task not found: {}", task_id))?;
                machine.reject().map_err(|error| error.to_string())?;
            }
            update_task_status(state, task_id, OperatorTaskStatus::Cancelled, None);
            if let Some(task) = state.tasks.get_mut(task_id) {
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.summary =
                    Some("Generated patch rejected; project files were not changed.".to_string());
            }
            push_operator_event(
                state,
                task_id,
                OperatorEventType::ApprovalRejected,
                EventLevel::Info,
                "Patch rejected",
                comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );
            return Ok(false);
        }
        ApprovalDecision::RequestChanges => {
            state.pending_patches.remove(task_id);
            {
                let machine = state
                    .state_machines
                    .get_mut(task_id)
                    .ok_or_else(|| format!("Task not found: {}", task_id))?;
                machine
                    .request_changes()
                    .map_err(|error| error.to_string())?;
            }
            update_task_status(state, task_id, OperatorTaskStatus::Planning, None);
            push_operator_event(
                state,
                task_id,
                OperatorEventType::ApprovalRejected,
                EventLevel::Info,
                "Patch changes requested",
                comment,
                "operator",
                Some(json!({ "approval_id": approval.approval_id })),
            );
            advance_godot_task_to_plan(state, task_id)?;
            return Ok(false);
        }
    }
}

fn discover_hermes_cli_path() -> Option<PathBuf> {
    d_drive_path_from_env("HERMES_GAME_CLI_PATH")
        .or_else(|| d_drive_path_from_env("HERMES_CLI_PATH"))
        .or_else(discover_known_d_drive_hermes_cli_path)
}

fn discover_known_d_drive_hermes_cli_path() -> Option<PathBuf> {
    let candidates = [
        workspace_root().join("bin").join("hermes-game.exe"),
        PathBuf::from("D:/dev-tools/hermes-game/target/release/hermes-game.exe"),
        PathBuf::from("D:/tmp/hermes-official/hermes.exe"),
    ];

    candidates
        .into_iter()
        .find(|path| is_d_drive_path(path) && path.is_file())
}

fn is_hermes_game_cli_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let full_path = path.to_string_lossy().to_ascii_lowercase();

    file_name.contains("hermes-game") || full_path.contains("hermes-game")
}

fn hermes_cli_not_found_message() -> String {
    "Hermes CLI not found on D:. Set HERMES_GAME_CLI_PATH/HERMES_CLI_PATH to a D: executable or build D:\\dev-tools\\hermes-game\\target\\release\\hermes-game.exe.".to_string()
}

fn hermes_connection_status_for_cli_path(cli_path: PathBuf) -> HermesConnectionStatus {
    let available = if is_hermes_game_cli_path(&cli_path) {
        let bridge = HermesGameBridge::new(cli_path.clone(), PathBuf::new(), String::new());
        bridge.is_available()
    } else {
        let bridge = HermesCliBridge::new(cli_path.clone(), PathBuf::new(), String::new());
        bridge.is_available()
    };

    if available {
        HermesConnectionStatus {
            available: true,
            version: executable_version(&cli_path),
            path: Some(cli_path.to_string_lossy().to_string()),
            error: None,
        }
    } else {
        HermesConnectionStatus {
            available: false,
            version: None,
            path: Some(cli_path.to_string_lossy().to_string()),
            error: Some("Hermes CLI found but not executable".to_string()),
        }
    }
}

fn advance_godot_task_to_plan(state: &mut OperatorState, task_id: &str) -> Result<(), String> {
    let (project_path, goal) = {
        let task = state
            .tasks
            .get(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        (
            std::path::PathBuf::from(&task.project_path),
            task.goal.clone(),
        )
    };

    push_operator_event(
        state,
        task_id,
        OperatorEventType::ProjectAnalyzing,
        EventLevel::Info,
        "Analyzing Godot project",
        Some(project_path.to_string_lossy().to_string()),
        "operator_local_planner",
        None,
    );

    let analysis =
        GodotProjectAnalyzer::analyze_project(&project_path).map_err(|e| e.to_string())?;
    let player_controllers = GodotProjectAnalyzer::find_player_controllers(&analysis);

    push_operator_event(
        state,
        task_id,
        OperatorEventType::ProjectAnalyzed,
        EventLevel::Info,
        "Project analyzed",
        Some(format!(
            "{} scripts, {} scenes, {} player controllers",
            analysis.scripts.len(),
            analysis.scenes.len(),
            player_controllers.len()
        )),
        "operator_local_planner",
        Some(json!({
            "project_name": analysis.project_name,
            "godot_version": analysis.godot_version,
            "main_scene": analysis.main_scene,
            "scripts": stringify_paths(&analysis.scripts),
            "scenes": stringify_paths(&analysis.scenes),
            "player_controllers": stringify_paths(&player_controllers),
        })),
    );

    push_operator_event(
        state,
        task_id,
        OperatorEventType::PlanGenerating,
        EventLevel::Info,
        "Generating execution plan",
        Some(goal.clone()),
        "operator_local_planner",
        None,
    );

    let steps = build_godot_plan_steps(&goal, &analysis, &player_controllers);
    let files = collect_plan_files(&steps);

    push_operator_event(
        state,
        task_id,
        OperatorEventType::PlanReady,
        EventLevel::Info,
        "Plan ready",
        Some(format!("Generated {} steps for review", steps.len())),
        "operator_local_planner",
        Some(json!({ "steps": steps })),
    );

    {
        let machine = state
            .state_machines
            .get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        machine.plan_ready().map_err(|e| e.to_string())?;
    }
    update_task_status(state, task_id, OperatorTaskStatus::WaitingApproval, None);

    let approval_id = format!(
        "approval_{}_plan_{}",
        sanitize_operator_identifier(task_id),
        uuid::Uuid::new_v4().simple()
    );
    let approval = ApprovalRequest {
        approval_id: approval_id.clone(),
        task_id: task_id.to_string(),
        task_revision: state.tasks.get(task_id).map(|task| task.revision).unwrap_or(0),
        level: ApprovalLevel::Approve,
        action: "operator.plan.generate_patch".to_string(),
        title: "Review generated Godot plan".to_string(),
        reason: "The operator generated a project-aware plan and needs approval before execution.".to_string(),
        risk: Some("This approval may start a proposal-only Agent run. Project files still require a separate structured patch approval before any write.".to_string()),
        preview: Some(ApprovalPreview {
            files: Some(files),
            diff_id: None,
            command: None,
            diffs: None,
        }),
        options: vec![
            ApprovalDecision::Approve,
            ApprovalDecision::Reject,
            ApprovalDecision::RequestChanges,
        ],
        created_at: chrono::Utc::now().to_rfc3339(),
        resolved_at: None,
        decision: None,
        resolved_by: None,
    };
    state
        .approvals
        .entry(task_id.to_string())
        .or_default()
        .push(approval);

    push_operator_event(
        state,
        task_id,
        OperatorEventType::ApprovalRequested,
        EventLevel::Info,
        "Approval requested",
        Some(
            "Review the generated plan before asking Hermes Game for a structured patch."
                .to_string(),
        ),
        "operator",
        Some(json!({ "approval_id": approval_id })),
    );

    Ok(())
}

fn run_approved_task_in_state(
    state: &mut OperatorState,
    task_id: &str,
    hermes_cli_path: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(cli_path) = hermes_cli_path {
        let (project_path, goal, domain) = {
            let task = state
                .tasks
                .get(task_id)
                .ok_or_else(|| format!("Task not found: {}", task_id))?;
            (
                std::path::PathBuf::from(&task.project_path),
                task.goal.clone(),
                task.domain.clone(),
            )
        };

        if domain.starts_with("game.") && !is_hermes_game_cli_path(&cli_path) {
            push_operator_event(
                state,
                task_id,
                OperatorEventType::HookFailed,
                EventLevel::Warning,
                "Generic Hermes CLI blocked for game project writes",
                Some(
                    "Controlled game tasks require Hermes Game structured proposals; using the no-write local fallback."
                        .to_string(),
                ),
                "operator_patch_engine",
                None,
            );
            return run_local_approved_task_in_state(state, task_id);
        }

        if is_hermes_game_cli_path(&cli_path) {
            let bridge =
                HermesGameBridge::new(cli_path.clone(), project_path.clone(), task_id.to_string());

            if bridge.is_available() {
                match run_hermes_game_approved_task_in_state(state, task_id, bridge, &goal) {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        fail_task_in_state(state, task_id, &error)?;
                        return Err(error);
                    }
                }
            }

            push_operator_event(
                state,
                task_id,
                OperatorEventType::HookFailed,
                EventLevel::Warning,
                "Hermes Game CLI unavailable",
                Some(format!(
                    "hermes-game was found at '{}' but failed the availability check; using local fallback.",
                    cli_path.to_string_lossy()
                )),
                "operator",
                None,
            );

            return run_local_approved_task_in_state(state, task_id);
        }

        let bridge = HermesCliBridge::new(cli_path.clone(), project_path, task_id.to_string());

        if bridge.is_available() {
            match run_hermes_approved_task_in_state(state, task_id, bridge, &goal) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    fail_task_in_state(state, task_id, &error)?;
                    return Err(error);
                }
            }
        }

        push_operator_event(
            state,
            task_id,
            OperatorEventType::HookFailed,
            EventLevel::Warning,
            "Hermes CLI unavailable",
            Some(format!(
                "Hermes CLI was found at '{}' but failed the availability check; using local fallback.",
                cli_path.to_string_lossy()
            )),
            "operator",
            None,
        );
    }

    run_local_approved_task_in_state(state, task_id)
}

fn spawn_approved_task_runner(
    state: Arc<Mutex<OperatorState>>,
    run: ApprovedTaskRun,
) -> thread::JoinHandle<()> {
    let task_id = run.task_id.clone();
    thread::spawn(move || {
        if let Err(error) = run_approved_task_with_shared_state(state.clone(), run) {
            if let Ok(mut state) = state.lock() {
                if !is_task_terminal(&state, &task_id) {
                    let _ = fail_task_in_state(&mut state, &task_id, &error);
                }
            }
        }
        if let Ok(state) = state.lock() {
            if let Err(error) = state.persist() {
                eprintln!(
                    "Failed to persist Operator task '{}' after background run: {}",
                    task_id, error
                );
            }
        }
    })
}

fn run_approved_task_with_shared_state(
    state: Arc<Mutex<OperatorState>>,
    run: ApprovedTaskRun,
) -> Result<(), String> {
    if run.validate_godot {
        return run_pending_godot_validation_with_shared_state(
            state,
            &run.task_id,
            run.godot_executable,
        );
    }

    if let Some(cli_path) = run.hermes_cli_path.clone() {
        let (project_path, goal, domain) = {
            let state = state.lock().map_err(|e| e.to_string())?;
            let task = state
                .tasks
                .get(&run.task_id)
                .ok_or_else(|| format!("Task not found: {}", run.task_id))?;
            (
                PathBuf::from(&task.project_path),
                task.goal.clone(),
                task.domain.clone(),
            )
        };

        if domain.starts_with("game.") && !is_hermes_game_cli_path(&cli_path) {
            let mut state = state.lock().map_err(|e| e.to_string())?;
            push_operator_event(
                &mut state,
                &run.task_id,
                OperatorEventType::HookFailed,
                EventLevel::Warning,
                "Generic Hermes CLI blocked for game project writes",
                Some(
                    "Controlled game tasks require Hermes Game structured proposals; using the no-write local fallback."
                        .to_string(),
                ),
                "operator_patch_engine",
                None,
            );
            return run_local_approved_task_in_state(&mut state, &run.task_id);
        }

        if is_hermes_game_cli_path(&cli_path) {
            let bridge = HermesGameBridge::new(cli_path.clone(), project_path, run.task_id.clone());

            if bridge.is_available() {
                return run_hermes_game_approved_task_with_shared_state(
                    state,
                    &run.task_id,
                    bridge,
                    &goal,
                );
            }

            let mut state = state.lock().map_err(|e| e.to_string())?;
            push_operator_event(
                &mut state,
                &run.task_id,
                OperatorEventType::HookFailed,
                EventLevel::Warning,
                "Hermes Game CLI unavailable",
                Some(format!(
                    "hermes-game was found at '{}' but failed the availability check; using local fallback.",
                    cli_path.to_string_lossy()
                )),
                "operator",
                None,
            );
            return run_local_approved_task_in_state(&mut state, &run.task_id);
        }
    }

    let mut state = state.lock().map_err(|e| e.to_string())?;
    run_approved_task_in_state(&mut state, &run.task_id, run.hermes_cli_path)
}

fn run_pending_godot_validation_in_state(
    state: &mut OperatorState,
    task_id: &str,
    executable: Option<PathBuf>,
) -> Result<(), String> {
    let pending = state
        .pending_validations
        .get(task_id)
        .cloned()
        .ok_or_else(|| format!("Pending Godot validation not found for task: {}", task_id))?;
    let Some(executable) = executable else {
        return finish_godot_validation_in_state(
            state,
            task_id,
            GodotValidationResult::skipped(
                "No D: Godot executable was found. Set GODOT_BIN to a D: Godot 4 executable.",
            ),
        );
    };

    push_validation_started(state, task_id, &pending, &executable);
    match crate::operator::run_godot_validation(&executable, &pending.project_root, || false) {
        Ok(result) => finish_godot_validation_in_state(state, task_id, result),
        Err(error) => fail_godot_validation_start_in_state(state, task_id, &error.to_string()),
    }
}

fn run_pending_godot_validation_with_shared_state(
    state: Arc<Mutex<OperatorState>>,
    task_id: &str,
    executable: Option<PathBuf>,
) -> Result<(), String> {
    let (pending, expected_goal) = {
        let state = state.lock().map_err(|error| error.to_string())?;
        let pending = state
            .pending_validations
            .get(task_id)
            .cloned()
            .ok_or_else(|| format!("Pending Godot validation not found for task: {}", task_id))?;
        let goal = state
            .tasks
            .get(task_id)
            .map(|task| task.goal.clone())
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        (pending, goal)
    };

    let Some(executable) = executable else {
        let mut state = state.lock().map_err(|error| error.to_string())?;
        if !state.pending_validations.contains_key(task_id)
            || !state
                .tasks
                .get(task_id)
                .map(|task| {
                    task.status == OperatorTaskStatus::Running && task.goal == expected_goal
                })
                .unwrap_or(false)
        {
            return Ok(());
        }
        return finish_godot_validation_in_state(
            &mut state,
            task_id,
            GodotValidationResult::skipped(
                "No D: Godot executable was found. Set GODOT_BIN to a D: Godot 4 executable.",
            ),
        );
    };

    {
        let mut state = state.lock().map_err(|error| error.to_string())?;
        if !state.pending_validations.contains_key(task_id)
            || !state
                .tasks
                .get(task_id)
                .map(|task| {
                    task.status == OperatorTaskStatus::Running && task.goal == expected_goal
                })
                .unwrap_or(false)
        {
            return Ok(());
        }
        push_validation_started(&mut state, task_id, &pending, &executable);
    }

    let cancel_state = state.clone();
    let cancel_task_id = task_id.to_string();
    let cancel_goal = expected_goal.clone();
    let result =
        crate::operator::run_godot_validation(&executable, &pending.project_root, move || {
            cancel_state
                .lock()
                .map(|state| {
                    state
                        .tasks
                        .get(&cancel_task_id)
                        .map(|task| {
                            matches!(
                                task.status,
                                OperatorTaskStatus::Cancelling
                                    | OperatorTaskStatus::Cancelled
                                    | OperatorTaskStatus::Paused
                            ) || task.goal != cancel_goal
                        })
                        .unwrap_or(true)
                })
                .unwrap_or(true)
        });

    let mut state = state.lock().map_err(|error| error.to_string())?;
    if is_task_goal_changed(&state, task_id, &expected_goal) {
        push_operator_event(
            &mut state,
            task_id,
            OperatorEventType::ValidationSkipped,
            EventLevel::Warning,
            "Godot validation superseded",
            Some("Validation stopped because the operator redirected the task.".to_string()),
            "godot_validation",
            None,
        );
        return Ok(());
    }
    if is_task_paused(&state, task_id) {
        push_operator_event(
            &mut state,
            task_id,
            OperatorEventType::ValidationSkipped,
            EventLevel::Warning,
            "Godot validation paused",
            Some("Validation stopped. Resume will rerun it for the applied patch.".to_string()),
            "godot_validation",
            None,
        );
        return Ok(());
    }
    if is_task_cancelling_or_cancelled(&state, task_id) {
        push_operator_event(
            &mut state,
            task_id,
            OperatorEventType::ValidationSkipped,
            EventLevel::Warning,
            "Godot validation cancelled",
            Some("Validation stopped because the operator cancelled the task.".to_string()),
            "godot_validation",
            None,
        );
        return Ok(());
    }

    match result {
        Ok(result) => finish_godot_validation_in_state(&mut state, task_id, result),
        Err(error) => fail_godot_validation_start_in_state(&mut state, task_id, &error.to_string()),
    }
}

fn push_validation_started(
    state: &mut OperatorState,
    task_id: &str,
    pending: &PendingGodotValidation,
    executable: &Path,
) {
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ValidationStarted,
        EventLevel::Info,
        "Godot validation started",
        Some("Running ACP's fixed Godot 4 headless validation template.".to_string()),
        "godot_validation",
        Some(json!({
            "executable": executable.to_string_lossy(),
            "project_root": pending.project_root.to_string_lossy(),
            "patch_id": pending.patch_id,
            "command": "godot --headless --editor --path <project> --quit-after 1",
        })),
    );
}

fn finish_godot_validation_in_state(
    state: &mut OperatorState,
    task_id: &str,
    result: GodotValidationResult,
) -> Result<(), String> {
    let pending = state
        .pending_validations
        .get(task_id)
        .cloned()
        .ok_or_else(|| format!("Pending Godot validation not found for task: {}", task_id))?;
    let payload = godot_validation_event_payload(&pending, &result);

    match result.status {
        GodotValidationStatus::Passed | GodotValidationStatus::Skipped => {
            let (event_type, level, title) = if result.status == GodotValidationStatus::Passed {
                (
                    OperatorEventType::ValidationPassed,
                    EventLevel::Info,
                    "Godot validation passed",
                )
            } else {
                (
                    OperatorEventType::ValidationSkipped,
                    EventLevel::Warning,
                    "Godot validation skipped",
                )
            };
            push_operator_event(
                state,
                task_id,
                event_type,
                level,
                title,
                Some(result.message.clone()),
                "godot_validation",
                Some(payload),
            );
            push_operator_event(
                state,
                task_id,
                OperatorEventType::TaskCompleting,
                EventLevel::Info,
                "Task completing",
                Some("Finalizing the approved patch after Godot validation.".to_string()),
                "operator_patch_engine",
                None,
            );
            {
                let machine = state
                    .state_machines
                    .get_mut(task_id)
                    .ok_or_else(|| format!("Task not found: {}", task_id))?;
                machine.complete().map_err(|error| error.to_string())?;
            }
            state.pending_validations.remove(task_id);
            let checkpoint_id = format!(
                "checkpoint_{}_{}",
                sanitize_operator_identifier(task_id),
                uuid::Uuid::new_v4().simple()
            );
            let memory_snapshot_id = format!(
                "memory_{}_{}",
                sanitize_operator_identifier(task_id),
                uuid::Uuid::new_v4().simple()
            );
            if let Some(task) = state.tasks.get_mut(task_id) {
                task.status = OperatorTaskStatus::Completed;
                task.updated_at = chrono::Utc::now().to_rfc3339();
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.checkpoint_id = Some(checkpoint_id.clone());
                task.memory_snapshot_id = Some(memory_snapshot_id.clone());
                task.error = None;
                task.summary = Some(format!(
                    "Applied {} approved Hermes Game file change(s). Godot validation: {:?}. Backup: {}.",
                    pending.files_applied, result.status, pending.backup_root
                ));
            }
            push_operator_event(
                state,
                task_id,
                OperatorEventType::MemoryWritten,
                EventLevel::Info,
                "Task memory snapshot written",
                Some("Recorded validated project facts, applied patch and final validation result.".to_string()),
                "operator_memory",
                Some(json!({
                    "checkpoint_id": checkpoint_id,
                    "memory_snapshot_id": memory_snapshot_id,
                    "source": "validated_task_completion"
                })),
            );
            push_operator_event(
                state,
                task_id,
                OperatorEventType::TaskCompleted,
                EventLevel::Info,
                "Task completed",
                Some(
                    "The approved structured patch was applied and the validation stage finished."
                        .to_string(),
                ),
                "operator_patch_engine",
                Some(json!({
                    "patch_id": pending.patch_id,
                    "validation_status": result.status,
                })),
            );
            Ok(())
        }
        GodotValidationStatus::Failed | GodotValidationStatus::TimedOut => {
            push_operator_event(
                state,
                task_id,
                OperatorEventType::ValidationFailed,
                EventLevel::Error,
                "Godot validation failed",
                Some(result.message.clone()),
                "godot_validation",
                Some(payload),
            );
            let error = format!(
                "{}; the operator-approved files remain applied and can be reviewed or restored from {}",
                result.message, pending.backup_root
            );
            fail_task_in_state(state, task_id, &error)?;
            Err(error)
        }
        GodotValidationStatus::Cancelled => {
            let error =
                "Godot validation ended as cancelled without a matching operator state".to_string();
            push_operator_event(
                state,
                task_id,
                OperatorEventType::ValidationFailed,
                EventLevel::Error,
                "Godot validation cancelled unexpectedly",
                Some(error.clone()),
                "godot_validation",
                Some(payload),
            );
            fail_task_in_state(state, task_id, &error)?;
            Err(error)
        }
    }
}

fn fail_godot_validation_start_in_state(
    state: &mut OperatorState,
    task_id: &str,
    error: &str,
) -> Result<(), String> {
    let backup_root = state
        .pending_validations
        .get(task_id)
        .map(|pending| pending.backup_root.clone())
        .unwrap_or_default();
    let error = format!(
        "Godot validation could not start: {}; the operator-approved files remain applied and can be reviewed or restored from {}",
        error, backup_root
    );
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ValidationFailed,
        EventLevel::Error,
        "Godot validation failed to start",
        Some(error.clone()),
        "godot_validation",
        None,
    );
    fail_task_in_state(state, task_id, &error)?;
    Err(error)
}

fn godot_validation_event_payload(
    pending: &PendingGodotValidation,
    result: &GodotValidationResult,
) -> serde_json::Value {
    json!({
        "patch_id": pending.patch_id,
        "backup_root": pending.backup_root,
        "status": result.status,
        "executable": result.executable,
        "args": result.args,
        "exit_code": result.exit_code,
        "duration_ms": result.duration_ms,
        "output_truncated": result.output_truncated,
        "stdout": truncate_event_output(&result.stdout),
        "stderr": truncate_event_output(&result.stderr),
    })
}

fn truncate_event_output(output: &str) -> String {
    const LIMIT: usize = 8 * 1024;
    if output.len() <= LIMIT {
        return output.to_string();
    }
    let mut end = LIMIT;
    while !output.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[ACP truncated validation event output]",
        &output[..end]
    )
}

fn configured_godot_executable(state: &OperatorState) -> Option<PathBuf> {
    match state.godot_validation_discovery {
        GodotValidationDiscovery::Auto => crate::operator::discover_godot_executable(),
        GodotValidationDiscovery::Disabled => None,
    }
}

fn run_hermes_game_approved_task_with_shared_state(
    state: Arc<Mutex<OperatorState>>,
    task_id: &str,
    bridge: HermesGameBridge,
    goal: &str,
) -> Result<(), String> {
    let artifact_path = hermes_game_artifact_path(task_id);
    let artifact_path_display = artifact_path.to_string_lossy().to_string();

    {
        let mut state = state.lock().map_err(|e| e.to_string())?;
        push_operator_event(
            &mut state,
            task_id,
            OperatorEventType::TaskStarted,
            EventLevel::Info,
            "Hermes Game execution started",
            Some("Delegating approved game-development task to D: hermes-game.".to_string()),
            "hermes_game_cli",
            Some(json!({
                "artifact_path": artifact_path_display,
                "mode": "proposal_artifact"
            })),
        );
        push_operator_event(
            &mut state,
            task_id,
            OperatorEventType::ToolCallStarted,
            EventLevel::Info,
            "hermes-game codegen",
            Some("Generating a reviewable implementation proposal artifact.".to_string()),
            "hermes_game_cli",
            Some(json!({
                "command": "hermes-game --engine godot codegen controlled-proposal-request --output <artifact> --no-tools --prompt-file <request>",
                "artifact_path": artifact_path_display
            })),
        );
    }

    let cancel_state = state.clone();
    let cancel_task_id = task_id.to_string();
    let expected_goal = goal.to_string();
    let result = bridge.generate_code_proposal_with_cancel(goal, &artifact_path, move || {
        cancel_state
            .lock()
            .map(|state| {
                state
                    .tasks
                    .get(&cancel_task_id)
                    .map(|task| {
                        matches!(
                            task.status,
                            OperatorTaskStatus::Cancelling
                                | OperatorTaskStatus::Cancelled
                                | OperatorTaskStatus::Paused
                        ) || task.goal != expected_goal
                    })
                    .unwrap_or(true)
            })
            .unwrap_or(true)
    });

    match result {
        Ok(result) => {
            let mut state = state.lock().map_err(|e| e.to_string())?;
            if is_task_goal_changed(&state, task_id, goal) {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskRedirected,
                    EventLevel::Info,
                    "Hermes Game result discarded",
                    Some(
                        "A newer operator direction replaced this run before its result was applied."
                            .to_string(),
                    ),
                    "hermes_game_cli",
                    Some(json!({ "artifact_path": artifact_path_display })),
                );
                return Ok(());
            }

            if is_task_paused(&state, task_id) {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskPaused,
                    EventLevel::Info,
                    "Hermes Game result held",
                    Some(
                        "The proposal result was discarded because the task was paused before completion."
                            .to_string(),
                    ),
                    "hermes_game_cli",
                    Some(json!({ "artifact_path": artifact_path_display })),
                );
                return Ok(());
            }

            if is_task_cancelling_or_cancelled(&state, task_id) {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskCancelled,
                    EventLevel::Info,
                    "Hermes Game execution cancelled",
                    Some(
                        "The proposal result was discarded because the task was cancelled."
                            .to_string(),
                    ),
                    "hermes_game_cli",
                    Some(json!({ "artifact_path": artifact_path_display })),
                );
                return Ok(());
            }

            queue_hermes_game_patch_approval(&mut state, task_id, &result)
        }
        Err(error) => {
            let mut state = state.lock().map_err(|e| e.to_string())?;
            if is_task_goal_changed(&state, task_id, goal) {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskRedirected,
                    EventLevel::Info,
                    "Hermes Game execution superseded",
                    Some("The running hermes-game process was stopped after the task was redirected.".to_string()),
                    "hermes_game_cli",
                    None,
                );
                return Ok(());
            }

            if is_task_paused(&state, task_id) {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskPaused,
                    EventLevel::Info,
                    "Hermes Game execution paused",
                    Some("The running hermes-game process was stopped and can be restarted on resume.".to_string()),
                    "hermes_game_cli",
                    None,
                );
                return Ok(());
            }

            if is_task_cancelling_or_cancelled(&state, task_id)
                || error.to_string().contains("cancelled")
            {
                push_operator_event(
                    &mut state,
                    task_id,
                    OperatorEventType::TaskCancelled,
                    EventLevel::Info,
                    "Hermes Game execution cancelled",
                    Some("The running hermes-game process was stopped.".to_string()),
                    "hermes_game_cli",
                    None,
                );
                return Ok(());
            }

            let error = error.to_string();
            fail_task_in_state(&mut state, task_id, &error)
        }
    }
}

fn run_hermes_game_approved_task_in_state(
    state: &mut OperatorState,
    task_id: &str,
    bridge: HermesGameBridge,
    goal: &str,
) -> Result<(), String> {
    let artifact_path = hermes_game_artifact_path(task_id);
    let artifact_path_display = artifact_path.to_string_lossy().to_string();

    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskStarted,
        EventLevel::Info,
        "Hermes Game execution started",
        Some("Delegating approved game-development task to D: hermes-game.".to_string()),
        "hermes_game_cli",
        Some(json!({
            "artifact_path": artifact_path_display,
            "mode": "proposal_artifact"
        })),
    );
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ToolCallStarted,
        EventLevel::Info,
        "hermes-game codegen",
        Some("Generating a reviewable implementation proposal artifact.".to_string()),
        "hermes_game_cli",
        Some(json!({
            "command": "hermes-game --engine godot codegen controlled-proposal-request --output <artifact> --no-tools --prompt-file <request>",
            "artifact_path": artifact_path_display
        })),
    );

    let result = bridge
        .generate_code_proposal(goal, &artifact_path)
        .map_err(|e| e.to_string())?;

    queue_hermes_game_patch_approval(state, task_id, &result)
}

fn queue_hermes_game_patch_approval(
    state: &mut OperatorState,
    task_id: &str,
    result: &crate::operator::HermesGameRunResult,
) -> Result<(), String> {
    let project_path = state
        .tasks
        .get(task_id)
        .map(|task| PathBuf::from(&task.project_path))
        .ok_or_else(|| format!("Task not found: {}", task_id))?;
    let artifact = fs::read_to_string(&result.artifact_path).map_err(|error| {
        format!(
            "Hermes Game structured patch artifact cannot be read: {}",
            error
        )
    })?;
    record_file_change(state, task_id, &result.artifact_path, "artifact");
    let prepared = crate::operator::prepare_structured_patch(&artifact, &project_path)
        .map_err(|error| format!("Hermes Game structured patch rejected: {}", error))?;

    let patch_id = prepared.patch_id.clone();
    let files = prepared
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<Vec<_>>();
    let diffs = prepared
        .changes
        .iter()
        .map(|change| FileDiffPreview {
            path: change.path.clone(),
            operation: match change.operation {
                PatchOperation::Create => "create",
                PatchOperation::Replace => "replace",
            }
            .to_string(),
            diff: change.diff.clone(),
        })
        .collect::<Vec<_>>();
    let summary = prepared.summary.clone();
    let validation = prepared.validation.clone();
    let approval_id = format!(
        "approval_{}_patch_{}",
        sanitize_operator_identifier(task_id),
        uuid::Uuid::new_v4().simple()
    );

    {
        let machine = state
            .state_machines
            .get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        machine
            .await_approval()
            .map_err(|error| error.to_string())?;
    }
    state.pending_patches.insert(task_id.to_string(), prepared);
    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::WaitingApproval;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.completed_at = None;
        task.summary = Some(format!(
            "Hermes Game proposed {} structured file change(s). Project files remain unchanged until patch approval.",
            files.len()
        ));
    }

    push_operator_event(
        state,
        task_id,
        OperatorEventType::FilePatchProposed,
        EventLevel::Info,
        "Hermes Game structured patch generated",
        Some(summary.clone()),
        "hermes_game_cli",
        Some(json!({
            "artifact_path": result.artifact_path,
            "patch_id": patch_id.clone(),
            "files": files.clone(),
            "validation": validation.clone(),
            "proposal_only": true,
            "tools_enabled": false,
        })),
    );
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ToolCallSucceeded,
        EventLevel::Info,
        "hermes-game proposal validated",
        Some("The strict JSON patch is ready for a separate apply approval.".to_string()),
        "hermes_game_cli",
        Some(json!({ "patch_id": patch_id.clone() })),
    );

    state
        .approvals
        .entry(task_id.to_string())
        .or_default()
        .push(ApprovalRequest {
            approval_id: approval_id.clone(),
            task_id: task_id.to_string(),
            task_revision: state.tasks.get(task_id).map(|task| task.revision).unwrap_or(0),
            level: ApprovalLevel::Approve,
            action: PATCH_APPLY_ACTION.to_string(),
            title: format!("Review {} proposed file change(s)", files.len()),
            reason: summary,
            risk: Some(
                "Approval writes complete text files. ACP will recheck original hashes, create isolated backups, and roll back a failed batch."
                    .to_string(),
            ),
            preview: Some(ApprovalPreview {
                files: Some(files),
                diff_id: Some(patch_id.clone()),
                command: None,
                diffs: Some(diffs),
            }),
            options: vec![
                ApprovalDecision::Approve,
                ApprovalDecision::Reject,
                ApprovalDecision::RequestChanges,
            ],
            created_at: chrono::Utc::now().to_rfc3339(),
            resolved_at: None,
            decision: None,
            resolved_by: None,
        });
    push_operator_event(
        state,
        task_id,
        OperatorEventType::ApprovalRequested,
        EventLevel::Info,
        "Patch approval requested",
        Some("Review every file diff before applying the structured patch.".to_string()),
        "operator_patch_engine",
        Some(json!({
            "approval_id": approval_id,
            "patch_id": patch_id,
            "action": PATCH_APPLY_ACTION,
        })),
    );

    Ok(())
}

fn hermes_game_artifact_path(task_id: &str) -> PathBuf {
    let safe_task_id = sanitize_operator_identifier(task_id);

    workspace_root()
        .join(".operator")
        .join("hermes-runs")
        .join(safe_task_id)
        .join(format!("proposal_{}.json", uuid::Uuid::new_v4().simple()))
}

fn hermes_game_backup_path(task_id: &str, approval_id: &str) -> PathBuf {
    workspace_root()
        .join(".operator")
        .join("hermes-runs")
        .join(sanitize_operator_identifier(task_id))
        .join("backups")
        .join(sanitize_operator_identifier(approval_id))
}

fn sanitize_operator_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn run_hermes_approved_task_in_state(
    state: &mut OperatorState,
    task_id: &str,
    bridge: HermesCliBridge,
    goal: &str,
) -> Result<(), String> {
    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskStarted,
        EventLevel::Info,
        "Hermes CLI execution started",
        Some("Delegating approved task to Hermes CLI.".to_string()),
        "operator",
        None,
    );

    let events = bridge.execute_task(goal).map_err(|e| e.to_string())?;
    let mut completed = None;
    let mut errors = Vec::new();
    let mut modified_files = Vec::new();

    for hermes_event in events {
        match &hermes_event {
            HermesEvent::FileModified { path, .. } => {
                modified_files.push(path.clone());
                record_file_change(state, task_id, path, "modified");
            }
            HermesEvent::Error { message } => {
                errors.push(message.clone());
            }
            HermesEvent::Completed { success } => {
                completed = Some(*success);
            }
            _ => {}
        }

        let operator_event = hermes_event.to_operator_event(task_id);
        push_existing_operator_event(state, task_id, operator_event);
    }

    if !errors.is_empty() {
        return Err(format!(
            "Hermes CLI execution failed: {}",
            errors.join("; ")
        ));
    }

    match completed {
        Some(true) => {
            let machine = state
                .state_machines
                .get_mut(task_id)
                .ok_or_else(|| format!("Task not found: {}", task_id))?;
            machine.complete().map_err(|e| e.to_string())?;

            if let Some(task) = state.tasks.get_mut(task_id) {
                task.status = OperatorTaskStatus::Completed;
                task.updated_at = chrono::Utc::now().to_rfc3339();
                task.completed_at = Some(chrono::Utc::now().to_rfc3339());
                task.summary = Some(format!(
                    "Hermes CLI execution completed for '{}'. Recorded {} modified file(s).",
                    task.goal,
                    modified_files.len()
                ));
            }

            Ok(())
        }
        Some(false) => Err("Hermes CLI execution failed".to_string()),
        None => Err("Hermes CLI execution ended without a completion event".to_string()),
    }
}

fn run_local_approved_task_in_state(
    state: &mut OperatorState,
    task_id: &str,
) -> Result<(), String> {
    // When Hermes Game CLI is unavailable, the task must NOT complete successfully.
    // Per audit requirement P0: "Hermes Game 不可用 -> blocked 或 failed"
    // This prevents fake success when no execution backend is available.

    let plan_steps = state
        .events
        .get(task_id)
        .and_then(|events| {
            events
                .iter()
                .rev()
                .find(|event| matches!(event.event_type, OperatorEventType::PlanReady))
        })
        .and_then(|event| event.payload.as_ref())
        .and_then(|payload| payload.get("steps"))
        .and_then(|steps| steps.as_array())
        .cloned()
        .unwrap_or_default();

    // Emit EXECUTOR_UNAVAILABLE error event
    push_operator_event(
        state,
        task_id,
        OperatorEventType::HookFailed,
        EventLevel::Error,
        "No Hermes execution backend available",
        Some(
            "EXECUTOR_UNAVAILABLE: Controlled game tasks require Hermes Game structured proposals. \
             No files were modified. Configure ACP_HERMES_GAME_CLI_PATH or place hermes-game.exe in bin/."
                .to_string(),
        ),
        "operator_patch_engine",
        Some(json!({
            "error_code": "EXECUTOR_UNAVAILABLE",
            "plan_steps_count": plan_steps.len()
        })),
    );

    // Mark task as Failed instead of Completed
    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Failed;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        task.error = Some("EXECUTOR_UNAVAILABLE: Hermes Game CLI not available".to_string());
        task.summary = Some(format!(
            "Task blocked: Hermes execution backend unavailable. {} plan steps were recorded but not executed.",
            plan_steps.len()
        ));
    }

    // Emit TaskFailed event, NOT TaskCompleted
    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskFailed,
        EventLevel::Error,
        "Task failed: executor unavailable",
        Some("Hermes Game CLI is required for controlled game tasks.".to_string()),
        "operator",
        Some(json!({
            "error_code": "EXECUTOR_UNAVAILABLE",
            "steps_recorded": plan_steps.len()
        })),
    );

    // Persist the failure state
    state.persist()
}

fn record_file_change(state: &mut OperatorState, task_id: &str, path: &str, change_type: &str) {
    let change_record = FileChangeRecord {
        path: path.to_string(),
        change_type: change_type.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    state
        .file_changes
        .entry(task_id.to_string())
        .or_default()
        .push(change_record);
}

fn cancel_task_in_state(state: &mut OperatorState, task_id: &str) -> Result<(), String> {
    let status = state
        .tasks
        .get(task_id)
        .map(|task| task.status.clone())
        .ok_or_else(|| format!("Task not found: {}", task_id))?;
    let should_cleanup = matches!(
        status,
        OperatorTaskStatus::Planning
            | OperatorTaskStatus::WaitingApproval
            | OperatorTaskStatus::Running
            | OperatorTaskStatus::Paused
            | OperatorTaskStatus::Redirecting
            | OperatorTaskStatus::Cancelling
    );

    match status {
        OperatorTaskStatus::Cancelled => return Ok(()),
        OperatorTaskStatus::Planning
        | OperatorTaskStatus::WaitingApproval
        | OperatorTaskStatus::Running
        | OperatorTaskStatus::Paused
        | OperatorTaskStatus::Redirecting => {
            let machine = state
                .state_machines
                .get_mut(task_id)
                .ok_or_else(|| format!("Task not found: {}", task_id))?;
            machine.stop().map_err(|e| e.to_string())?;
            push_operator_event(
                state,
                task_id,
                OperatorEventType::TaskCancelling,
                EventLevel::Info,
                "Task cancelling",
                Some("Cancellation requested by operator.".to_string()),
                "operator",
                None,
            );
        }
        OperatorTaskStatus::Cancelling => {}
        _ => {
            return Err(format!(
                "Task cannot be stopped from status {:?}: {}",
                status, task_id
            ));
        }
    }

    if should_cleanup {
        let machine = state
            .state_machines
            .get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        machine.cleanup_done().map_err(|e| e.to_string())?;
    }

    supersede_pending_approvals(state, task_id, "operator_stop");
    state.pending_patches.remove(task_id);
    state.pending_validations.remove(task_id);

    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Cancelled;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        task.summary = Some("Task cancelled by operator stop request".to_string());
    }

    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskCancelled,
        EventLevel::Info,
        "Task cancelled",
        Some("Operator stop request completed.".to_string()),
        "operator",
        None,
    );

    Ok(())
}

fn is_task_cancelling_or_cancelled(state: &OperatorState, task_id: &str) -> bool {
    state
        .tasks
        .get(task_id)
        .map(|task| {
            matches!(
                task.status,
                OperatorTaskStatus::Cancelling | OperatorTaskStatus::Cancelled
            )
        })
        .unwrap_or(true)
}

fn is_task_goal_changed(state: &OperatorState, task_id: &str, expected_goal: &str) -> bool {
    state
        .tasks
        .get(task_id)
        .map(|task| task.goal != expected_goal)
        .unwrap_or(true)
}

fn is_task_paused(state: &OperatorState, task_id: &str) -> bool {
    state
        .tasks
        .get(task_id)
        .map(|task| matches!(task.status, OperatorTaskStatus::Paused))
        .unwrap_or(false)
}

fn is_task_terminal(state: &OperatorState, task_id: &str) -> bool {
    state
        .tasks
        .get(task_id)
        .map(|task| {
            matches!(
                task.status,
                OperatorTaskStatus::Completed
                    | OperatorTaskStatus::Cancelled
                    | OperatorTaskStatus::Failed
            )
        })
        .unwrap_or(true)
}

fn fail_task_in_state(state: &mut OperatorState, task_id: &str, error: &str) -> Result<(), String> {
    state.pending_validations.remove(task_id);
    if let Some(machine) = state.state_machines.get_mut(task_id) {
        let _ = machine.fail(error);
    }
    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Failed;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        task.error = Some(error.to_string());
        task.summary = Some(format!("Task failed: {}", error));
    }
    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskFailed,
        EventLevel::Error,
        "Task failed",
        Some(error.to_string()),
        "operator",
        None,
    );
    Ok(())
}

fn update_task_status(
    state: &mut OperatorState,
    task_id: &str,
    status: OperatorTaskStatus,
    summary: Option<String>,
) {
    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = status;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        if let Some(summary) = summary {
            task.summary = Some(summary);
        }
    }
}

fn push_operator_event(
    state: &mut OperatorState,
    task_id: &str,
    event_type: OperatorEventType,
    level: EventLevel,
    title: &str,
    message: Option<String>,
    source: &str,
    payload: Option<serde_json::Value>,
) {
    let next_index = state
        .events
        .get(task_id)
        .map(|events| events.len())
        .unwrap_or(0);
    let task_revision = state
        .tasks
        .get_mut(task_id)
        .map(|task| {
            task.revision = task.revision.saturating_add(1);
            task.revision
        })
        .unwrap_or(0);

    // Map event type to i18n key
    let title_key = match &event_type {
        OperatorEventType::TaskCreated => Some("operatorEvent.taskCreated".to_string()),
        OperatorEventType::TaskStarted => Some("operatorEvent.taskStarted".to_string()),
        OperatorEventType::ProjectAnalyzing => Some("operatorEvent.projectAnalyzing".to_string()),
        OperatorEventType::ProjectAnalyzed => Some("operatorEvent.projectAnalyzed".to_string()),
        OperatorEventType::PlanGenerating => Some("operatorEvent.planGenerating".to_string()),
        OperatorEventType::PlanReady => Some("operatorEvent.planReady".to_string()),
        OperatorEventType::ApprovalRequested => Some("operatorEvent.approvalRequested".to_string()),
        OperatorEventType::ApprovalGranted => Some("operatorEvent.approvalGranted".to_string()),
        OperatorEventType::ApprovalRejected => Some("operatorEvent.approvalRejected".to_string()),
        OperatorEventType::StepExecuting => Some("operatorEvent.stepExecuting".to_string()),
        OperatorEventType::StepCompleted => Some("operatorEvent.stepCompleted".to_string()),
        OperatorEventType::StepFailed => Some("operatorEvent.stepFailed".to_string()),
        OperatorEventType::FileModified => Some("operatorEvent.fileModified".to_string()),
        OperatorEventType::TaskCompleting => Some("operatorEvent.taskCompleting".to_string()),
        OperatorEventType::TaskCompleted => Some("operatorEvent.taskCompleted".to_string()),
        OperatorEventType::TaskFailed => Some("operatorEvent.taskFailed".to_string()),
        OperatorEventType::TaskCancelled => Some("operatorEvent.taskCancelled".to_string()),
        OperatorEventType::TaskPaused => Some("operatorEvent.taskPaused".to_string()),
        OperatorEventType::TaskResumed => Some("operatorEvent.taskResumed".to_string()),
        OperatorEventType::TaskRedirected => Some("operatorEvent.taskRedirected".to_string()),
        _ => None,
    };

    let event = OperatorEvent {
        event_id: format!("evt_{}_{}", task_id, next_index),
        task_id: task_id.to_string(),
        sequence: next_index as u64,
        task_revision,
        timestamp: chrono::Utc::now().to_rfc3339(),
        event_type,
        level,
        title: title.to_string(),
        message,
        source: source.to_string(),
        payload,
        title_key,
        message_key: None,
        title_args: None,
        message_args: None,
    };
    state
        .events
        .entry(task_id.to_string())
        .or_default()
        .push(event);
}

fn push_existing_operator_event(
    state: &mut OperatorState,
    task_id: &str,
    mut event: OperatorEvent,
) {
    let next_index = state
        .events
        .get(task_id)
        .map(|events| events.len())
        .unwrap_or(0);
    let task_revision = state
        .tasks
        .get(task_id)
        .map(|task| task.revision)
        .unwrap_or(event.task_revision);
    event.event_id = format!("evt_{}_{}", task_id, next_index);
    event.task_id = task_id.to_string();
    event.sequence = next_index as u64;
    event.task_revision = task_revision;
    state
        .events
        .entry(task_id.to_string())
        .or_default()
        .push(event);
}

fn build_godot_plan_steps(
    goal: &str,
    analysis: &GodotProjectInfo,
    player_controllers: &[std::path::PathBuf],
) -> Vec<serde_json::Value> {
    let target_files = if player_controllers.is_empty() {
        analysis.scripts.iter().take(3).cloned().collect::<Vec<_>>()
    } else {
        player_controllers.to_vec()
    };
    let files = stringify_paths(&target_files);

    vec![
        json!({
            "id": 1,
            "description": "Inspect Godot project structure and entry scene",
            "files": stringify_paths(&analysis.scenes),
            "estimatedTimeSeconds": 10,
            "requiresApproval": false,
        }),
        json!({
            "id": 2,
            "description": "Inspect player/controller scripts related to the requested change",
            "files": files,
            "estimatedTimeSeconds": 20,
            "requiresApproval": false,
        }),
        json!({
            "id": 3,
            "description": format!("Prepare implementation for: {}", goal),
            "files": stringify_paths(&target_files),
            "estimatedTimeSeconds": 60,
            "requiresApproval": true,
        }),
        json!({
            "id": 4,
            "description": "Run validation after applying the change",
            "files": Vec::<String>::new(),
            "estimatedTimeSeconds": 30,
            "requiresApproval": false,
        }),
    ]
}

fn collect_plan_files(steps: &[serde_json::Value]) -> Vec<String> {
    let mut files = Vec::new();
    for step in steps {
        if let Some(step_files) = step.get("files").and_then(|value| value.as_array()) {
            for file in step_files {
                if let Some(file) = file.as_str() {
                    let file = file.to_string();
                    if !files.contains(&file) {
                        files.push(file);
                    }
                }
            }
        }
    }
    files
}

fn stringify_paths(paths: &[std::path::PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

// ============================================================================
// Task Management Commands
// ============================================================================

#[tauri::command]
pub async fn operator_start_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: StartTaskRequest,
) -> Result<StartTaskResponse, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    start_task_in_state(&mut state, request)
}

#[tauri::command]
pub async fn operator_get_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<OperatorTask, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state
        .tasks
        .get(&task_id)
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

pub(crate) fn pause_task_in_state_with_revision(
    state: &mut OperatorState,
    task_id: &str,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    check_expected_revision(state, task_id, expected_revision)?;
    let is_validation = state.pending_validations.contains_key(task_id);
    let machine = state
        .state_machines
        .get_mut(task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    machine.pause().map_err(|e| e.to_string())?;

    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Paused;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskPaused,
        EventLevel::Info,
        "Task paused",
        Some(if is_validation {
            "Operator paused Godot validation. Resume will rerun validation for the already applied patch."
                .to_string()
        } else {
            "Operator paused task execution. Resume will restart the current run from the latest goal."
                .to_string()
        }),
        "operator",
        None,
    );

    state.persist()
}

pub(crate) fn pause_task_in_state(
    state: &mut OperatorState,
    task_id: &str,
) -> Result<(), String> {
    pause_task_in_state_with_revision(state, task_id, None)
}

fn resume_task_in_state_for_background(
    state: &mut OperatorState,
    task_id: &str,
    hermes_cli_path: Option<PathBuf>,
    expected_revision: Option<u64>,
) -> Result<ApprovedTaskRun, String> {
    check_expected_revision(state, task_id, expected_revision)?;
    let validate_godot = state.pending_validations.contains_key(task_id);
    let godot_executable = if validate_godot {
        configured_godot_executable(state)
    } else {
        None
    };
    let machine = state
        .state_machines
        .get_mut(task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    machine.resume().map_err(|e| e.to_string())?;

    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Running;
        task.updated_at = chrono::Utc::now().to_rfc3339();
    }

    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskResumed,
        EventLevel::Info,
        "Task resumed",
        Some("Operator resumed task execution.".to_string()),
        "operator",
        None,
    );
    state.recovery_origins.remove(task_id);

    Ok(ApprovedTaskRun {
        task_id: task_id.to_string(),
        hermes_cli_path: if validate_godot {
            None
        } else {
            hermes_cli_path
        },
        validate_godot,
        godot_executable,
    })
}

fn resume_interrupted_planning_in_state(
    state: &mut OperatorState,
    task_id: &str,
) -> Result<bool, String> {
    let Some(origin) = state.recovery_origins.get(task_id).cloned() else {
        return Ok(false);
    };
    if !matches!(
        origin,
        OperatorTaskStatus::Idle | OperatorTaskStatus::Planning | OperatorTaskStatus::Redirecting
    ) {
        return Ok(false);
    }

    let domain = state
        .tasks
        .get(task_id)
        .map(|task| task.domain.clone())
        .ok_or_else(|| format!("Task not found: {}", task_id))?;
    if domain != "game.godot" {
        return Err(format!(
            "Interrupted planning recovery is not implemented for domain '{}'",
            domain
        ));
    }

    {
        let machine = state
            .state_machines
            .get_mut(task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        machine
            .resume_interrupted_planning()
            .map_err(|error| error.to_string())?;
    }
    if let Some(task) = state.tasks.get_mut(task_id) {
        task.status = OperatorTaskStatus::Planning;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.error = None;
        task.summary = Some("Interrupted planning was resumed by the operator.".to_string());
    }
    push_operator_event(
        state,
        task_id,
        OperatorEventType::TaskResumed,
        EventLevel::Info,
        "Interrupted planning resumed",
        Some("Project analysis and plan approval will be regenerated.".to_string()),
        "operator_recovery",
        Some(json!({ "recovered_from": origin })),
    );

    if let Err(error) = advance_godot_task_to_plan(state, task_id) {
        state.recovery_origins.remove(task_id);
        fail_task_in_state(state, task_id, &error)?;
        return Err(error);
    }
    state.recovery_origins.remove(task_id);
    Ok(true)
}

fn supersede_pending_approvals(state: &mut OperatorState, task_id: &str, resolved_by: &str) {
    let resolved_at = chrono::Utc::now().to_rfc3339();
    if let Some(approvals) = state.approvals.get_mut(task_id) {
        approvals
            .iter_mut()
            .filter(|approval| approval.decision.is_none())
            .for_each(|approval| {
                approval.decision = Some(ApprovalDecision::RequestChanges);
                approval.resolved_at = Some(resolved_at.clone());
                approval.resolved_by = Some(resolved_by.to_string());
            });
    }
}

pub(crate) fn redirect_task_in_state(
    state: &mut OperatorState,
    request: RedirectRequest,
) -> Result<(), String> {
    check_expected_revision(state, &request.task_id, request.expected_revision)?;
    let (old_goal, domain) = {
        let task = state
            .tasks
            .get(&request.task_id)
            .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
        (task.goal.clone(), task.domain.clone())
    };
    let new_goal = request.new_goal.clone();

    {
        let machine = state
            .state_machines
            .get_mut(&request.task_id)
            .ok_or_else(|| format!("Task not found: {}", request.task_id))?;
        machine.redirect().map_err(|e| e.to_string())?;
        machine.replan().map_err(|e| e.to_string())?;
    }

    supersede_pending_approvals(state, &request.task_id, "operator_redirect");
    state.pending_patches.remove(&request.task_id);
    state.pending_validations.remove(&request.task_id);

    if let Some(task) = state.tasks.get_mut(&request.task_id) {
        task.status = OperatorTaskStatus::Planning;
        task.goal = new_goal.clone();
        task.updated_at = chrono::Utc::now().to_rfc3339();
        task.error = None;
        task.summary = Some(format!(
            "Task redirected from '{}' to '{}'.",
            old_goal, new_goal
        ));
    }

    push_operator_event(
        state,
        &request.task_id,
        OperatorEventType::TaskRedirected,
        EventLevel::Info,
        "Task redirected",
        Some("Operator changed the task direction and requested a new plan.".to_string()),
        "operator",
        Some(json!({
            "old_goal": old_goal,
            "new_goal": new_goal,
            "preserve_completed_work": request.preserve_completed_work,
        })),
    );
    push_operator_event(
        state,
        &request.task_id,
        OperatorEventType::PlanStarted,
        EventLevel::Info,
        "Replanning started",
        Some("Generating a new plan for the redirected goal.".to_string()),
        "operator",
        None,
    );

    if domain == "game.godot" {
        if let Err(error) = advance_godot_task_to_plan(state, &request.task_id) {
            fail_task_in_state(state, &request.task_id, &error)?;
            state.persist()?;
            return Err(error);
        }
    }

    state.persist()
}

pub(crate) fn resume_task_in_shared_state_for_background(
    shared_state: Arc<Mutex<OperatorState>>,
    task_id: String,
) -> Result<(), String> {
    resume_task_in_shared_state_for_background_with_revision(shared_state, task_id, None)
}

pub(crate) fn resume_task_in_shared_state_for_background_with_revision(
    shared_state: Arc<Mutex<OperatorState>>,
    task_id: String,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    let run = {
        let mut state = shared_state.lock().map_err(|e| e.to_string())?;
        match resume_interrupted_planning_in_state(&mut state, &task_id) {
            Ok(true) => {
                state.persist()?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                let persist_error = state.persist().err();
                return Err(match persist_error {
                    Some(persist_error) => format!(
                        "{}; additionally failed to persist recovery failure: {}",
                        error, persist_error
                    ),
                    None => error,
                });
            }
        }
        let run = resume_task_in_state_for_background(
            &mut state,
            &task_id,
            discover_hermes_cli_path(),
            expected_revision,
        )?;
        state.persist()?;
        run
    };

    if run.validate_godot && run.godot_executable.is_none() {
        let mut state = shared_state.lock().map_err(|error| error.to_string())?;
        finish_godot_validation_in_state(
            &mut state,
            &task_id,
            GodotValidationResult::skipped(
                "No D: Godot executable was found. Set GODOT_BIN to a D: Godot 4 executable.",
            ),
        )?;
        return state.persist();
    }

    spawn_approved_task_runner(shared_state, run);

    Ok(())
}

pub(crate) fn approve_task_in_shared_state_for_background(
    shared_state: Arc<Mutex<OperatorState>>,
    request: ApproveRequest,
) -> Result<(), String> {
    let run = {
        let mut state = shared_state.lock().map_err(|e| e.to_string())?;
        let run =
            approve_task_in_state_for_background(&mut state, request, discover_hermes_cli_path())?;
        state.persist()?;
        run
    };

    if let Some(run) = run {
        spawn_approved_task_runner(shared_state, run);
    }

    Ok(())
}

pub(crate) fn stop_task_in_state_with_revision(
    state: &mut OperatorState,
    task_id: &str,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    check_expected_revision(state, task_id, expected_revision)?;
    cancel_task_in_state(state, task_id)?;
    state.recovery_origins.remove(task_id);
    state.persist()
}

pub(crate) fn stop_task_in_state(state: &mut OperatorState, task_id: &str) -> Result<(), String> {
    stop_task_in_state_with_revision(state, task_id, None)
}

#[tauri::command]
pub async fn operator_pause_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    pause_task_in_state_with_revision(&mut state, &task_id, expected_revision)
}

#[tauri::command]
pub async fn operator_resume_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    resume_task_in_shared_state_for_background_with_revision(state.inner().clone(), task_id, expected_revision)
}

#[tauri::command]
pub async fn operator_stop_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    expected_revision: Option<u64>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    stop_task_in_state_with_revision(&mut state, &task_id, expected_revision)
}

#[tauri::command]
pub async fn operator_redirect_task(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: RedirectRequest,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    redirect_task_in_state(&mut state, request)
}

#[tauri::command]
pub async fn operator_approve(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    request: ApproveRequest,
) -> Result<(), String> {
    approve_task_in_shared_state_for_background(state.inner().clone(), request)
}

#[tauri::command]
pub async fn operator_get_pending_approvals(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<Vec<ApprovalRequest>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let approvals = state
        .approvals
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    Ok(approvals
        .iter()
        .filter(|a| a.decision.is_none())
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn operator_list_events(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    limit: Option<usize>,
    after_sequence: Option<u64>,
) -> Result<Vec<OperatorEvent>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let events = state
        .events
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    // Return events in sequence ascending order (consistent with HTTP endpoint)
    let filtered: Vec<OperatorEvent> = match after_sequence {
        Some(seq) => events.iter().filter(|e| e.sequence > seq).cloned().collect(),
        None => events.clone(),
    };

    let limit = limit.unwrap_or(100);
    Ok(filtered.into_iter().take(limit).collect())
}

#[tauri::command]
pub async fn operator_get_task_summary(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<TaskSummary, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let events = state.events.get(&task_id).cloned().unwrap_or_default();
    let file_changes = state
        .file_changes
        .get(&task_id)
        .cloned()
        .unwrap_or_default();

    // Categorize file changes
    let files_changed: Vec<String> = file_changes
        .iter()
        .filter(|c| c.change_type == "modified")
        .map(|c| c.path.clone())
        .collect();
    let files_created: Vec<String> = file_changes
        .iter()
        .filter(|c| c.change_type == "created")
        .map(|c| c.path.clone())
        .collect();
    let files_deleted: Vec<String> = file_changes
        .iter()
        .filter(|c| c.change_type == "deleted")
        .map(|c| c.path.clone())
        .collect();

    // Calculate duration
    let duration_seconds =
        if let (Some(started), Some(completed)) = (&task.started_at, &task.completed_at) {
            let start = chrono::DateTime::parse_from_rfc3339(started).unwrap_or_default();
            let end = chrono::DateTime::parse_from_rfc3339(completed).unwrap_or_default();
            (end - start).num_seconds() as u64
        } else {
            0
        };

    // Extract errors and warnings from events
    let errors: Vec<String> = events
        .iter()
        .filter(|e| matches!(e.level, crate::operator::EventLevel::Error))
        .map(|e| e.title.clone())
        .collect();
    let warnings: Vec<String> = events
        .iter()
        .filter(|e| matches!(e.level, crate::operator::EventLevel::Warning))
        .map(|e| e.title.clone())
        .collect();

    Ok(TaskSummary {
        task_id: task_id.clone(),
        status: task.status.clone(),
        goal: task.goal.clone(),
        summary: task
            .summary
            .clone()
            .unwrap_or_else(|| "Task completed".to_string()),
        files_changed,
        files_created,
        files_deleted,
        duration_seconds,
        iterations: events.len() as u32,
        errors,
        warnings,
    })
}

// ============================================================================
// File Tool Commands
// ============================================================================

#[tauri::command]
pub async fn operator_file_read(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    path: String,
) -> Result<FileReadResult, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    // Get project_path from task state (not from frontend)
    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let path = std::path::Path::new(&path);
    let roots = vec![std::path::PathBuf::from(&task.project_path)];
    let path_guard = PathGuard::new(roots);

    file_read(path, &path_guard).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn operator_file_patch(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    path: String,
    new_content: String,
    create_backup: bool,
) -> Result<FilePatchResult, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if !state.tasks.contains_key(&task_id) {
        return Err(format!("Task not found: {}", task_id));
    }
    let _ = (path, new_content, create_backup);
    push_operator_event(
        &mut state,
        &task_id,
        OperatorEventType::ToolCallFailed,
        EventLevel::Warning,
        "Direct file patch rejected",
        Some(
            "Direct writes are disabled. Generate a structured patch and resolve its separate apply approval."
                .to_string(),
        ),
        "operator_patch_engine",
        Some(json!({ "required_action": PATCH_APPLY_ACTION })),
    );
    state.persist()?;
    Err(
        "Direct file patch is disabled; use the structured proposal and patch approval flow."
            .to_string(),
    )
}

#[tauri::command]
pub async fn operator_file_patch_preview(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    path: String,
    new_content: String,
) -> Result<FilePatch, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    // Get project_path from task state (not from frontend)
    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let path = std::path::Path::new(&path);
    let roots = vec![std::path::PathBuf::from(&task.project_path)];
    let path_guard = PathGuard::new(roots);

    file_patch_preview(path, &new_content, &path_guard).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn operator_file_list(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    path: String,
) -> Result<FileListResult, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    // Get project_path from task state (not from frontend)
    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let path = std::path::Path::new(&path);
    let roots = vec![std::path::PathBuf::from(&task.project_path)];
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

// ============================================================================
// Hermes CLI Commands
// ============================================================================

use crate::operator::hermes_cli_bridge::{
    HermesAnalysisResult, HermesCliBridge, HermesConnectionStatus, HermesEvent, HermesPlan,
    HermesPlanStep,
};
use crate::operator::hermes_game_bridge::HermesGameBridge;
use crate::operator::hermes_process::{
    d_drive_path_from_env, executable_version, is_d_drive_path, workspace_root,
};

#[tauri::command]
pub async fn hermes_check_connection() -> Result<HermesConnectionStatus, String> {
    let cli_path = discover_hermes_cli_path().ok_or_else(hermes_cli_not_found_message)?;
    Ok(hermes_connection_status_for_cli_path(cli_path))
}

#[tauri::command]
pub async fn hermes_analyze_project(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
) -> Result<HermesAnalysisResult, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let cli_path = discover_hermes_cli_path().ok_or_else(hermes_cli_not_found_message)?;
    if is_hermes_game_cli_path(&cli_path) {
        let analysis = GodotProjectAnalyzer::analyze_project(Path::new(&task.project_path))
            .map_err(|e| e.to_string())?;
        let player_controllers = GodotProjectAnalyzer::find_player_controllers(&analysis);

        return Ok(HermesAnalysisResult {
            project_name: analysis.project_name,
            godot_version: analysis.godot_version.unwrap_or_default(),
            scripts: stringify_paths(&analysis.scripts),
            scenes: stringify_paths(&analysis.scenes),
            player_controllers: stringify_paths(&player_controllers),
            analysis_time_ms: 0,
        });
    }

    let bridge = HermesCliBridge::new(cli_path, PathBuf::from(&task.project_path), task_id.clone());

    bridge.analyze_project().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hermes_generate_plan(
    state: State<'_, Arc<Mutex<OperatorState>>>,
    task_id: String,
    goal: String,
) -> Result<HermesPlan, String> {
    let state = state.lock().map_err(|e| e.to_string())?;

    let task = state
        .tasks
        .get(&task_id)
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    let cli_path = discover_hermes_cli_path().ok_or_else(hermes_cli_not_found_message)?;
    if is_hermes_game_cli_path(&cli_path) {
        let analysis = GodotProjectAnalyzer::analyze_project(Path::new(&task.project_path))
            .map_err(|e| e.to_string())?;
        let player_controllers = GodotProjectAnalyzer::find_player_controllers(&analysis);
        let steps = build_godot_plan_steps(&goal, &analysis, &player_controllers)
            .into_iter()
            .map(|step| HermesPlanStep {
                id: step.get("id").and_then(|value| value.as_u64()).unwrap_or(0) as u32,
                description: step
                    .get("description")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                files: step
                    .get("files")
                    .and_then(|value| value.as_array())
                    .map(|files| {
                        files
                            .iter()
                            .filter_map(|file| file.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                estimated_time_seconds: step
                    .get("estimatedTimeSeconds")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0),
                requires_approval: step
                    .get("requiresApproval")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false),
            })
            .collect::<Vec<_>>();
        let total_estimated_time_seconds =
            steps.iter().map(|step| step.estimated_time_seconds).sum();

        return Ok(HermesPlan {
            task_id,
            goal,
            steps,
            total_estimated_time_seconds,
        });
    }

    let bridge = HermesCliBridge::new(cli_path, PathBuf::from(&task.project_path), task_id.clone());

    bridge.generate_plan(&goal).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::current_dir()
                .expect("current dir")
                .join(".tmp-tests")
                .join(format!("{}_{}_{}", name, std::process::id(), unique));
            std::fs::create_dir_all(&path).expect("create test dir");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if is_d_drive_path(&self.path) {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }

    #[test]
    fn test_operator_state_disables_host_godot_discovery() {
        let state = OperatorState::new();

        assert_eq!(
            state.godot_validation_discovery,
            GodotValidationDiscovery::Disabled
        );
        assert_eq!(configured_godot_executable(&state), None);
    }

    #[test]
    fn persistent_waiting_approval_survives_restart() {
        let temp_dir = TestDir::new("persistent_waiting_approval");
        let project = create_godot_test_project(temp_dir.path());
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let task_id = {
            let mut state = OperatorState::new_persistent(&db_path).expect("persistent state");
            start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
            .task_id
        };

        let recovered = OperatorState::new_persistent(&db_path).expect("recover state");
        assert_eq!(
            recovered.tasks[&task_id].status,
            OperatorTaskStatus::WaitingApproval
        );
        assert_eq!(
            recovered.approvals[&task_id]
                .iter()
                .filter(|approval| approval.decision.is_none())
                .count(),
            1
        );
        assert!(!recovered.recovery_origins.contains_key(&task_id));
    }

    #[test]
    fn operator_events_have_cursors_and_stale_writes_are_rejected() {
        let temp_dir = TestDir::new("operator_revision_conflict");
        let project = create_godot_test_project(temp_dir.path());
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("start task");
        let initial_revision = state.tasks[&response.task_id].revision;
        let original_approval = state.approvals[&response.task_id][0].approval_id.clone();

        redirect_task_in_state(
            &mut state,
            RedirectRequest {
                task_id: response.task_id.clone(),
                new_goal: "Add a wall jump instead".to_string(),
                preserve_completed_work: true,
                expected_revision: Some(initial_revision),
            },
        )
        .expect("redirect with current revision");

        let stale_error = approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: original_approval,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: Some(initial_revision),
            },
            None,
        )
        .expect_err("stale approval must be rejected");
        assert!(stale_error.contains("REVISION_CONFLICT"));

        let events = &state.events[&response.task_id];
        for (index, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, index as u64);
            assert!(event.task_revision > 0);
        }
        assert_eq!(
            events.last().map(|event| event.task_revision),
            Some(state.tasks[&response.task_id].revision)
        );
    }

    #[test]
    fn running_task_recovers_paused_and_keeps_origin_across_restarts() {
        let temp_dir = TestDir::new("persistent_running_recovery");
        let project = create_godot_test_project(temp_dir.path());
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let task_id = {
            let mut state = OperatorState::new_persistent(&db_path).expect("persistent state");
            let task_id = start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
            .task_id;
            state
                .state_machines
                .get_mut(&task_id)
                .expect("machine")
                .approve()
                .expect("approve machine");
            state.tasks.get_mut(&task_id).expect("task").status = OperatorTaskStatus::Running;
            push_operator_event(
                &mut state,
                &task_id,
                OperatorEventType::TaskStarted,
                EventLevel::Info,
                "Simulated running checkpoint",
                None,
                "test",
                None,
            );
            state.persist().expect("persist running task");
            task_id
        };

        let first_recovery = OperatorState::new_persistent(&db_path).expect("first recovery");
        assert_eq!(
            first_recovery.tasks[&task_id].status,
            OperatorTaskStatus::Paused
        );
        assert_eq!(
            first_recovery.recovery_origins[&task_id],
            OperatorTaskStatus::Running
        );
        let recovery_event_count = first_recovery.events[&task_id]
            .iter()
            .filter(|event| event.source == "operator_recovery")
            .count();
        drop(first_recovery);

        let second_recovery = OperatorState::new_persistent(&db_path).expect("second recovery");
        assert_eq!(
            second_recovery.recovery_origins[&task_id],
            OperatorTaskStatus::Running
        );
        assert_eq!(
            second_recovery.events[&task_id]
                .iter()
                .filter(|event| event.source == "operator_recovery")
                .count(),
            recovery_event_count
        );
    }

    #[test]
    fn interrupted_planning_resume_regenerates_plan_approval() {
        let temp_dir = TestDir::new("persistent_planning_recovery");
        let project = create_godot_test_project(temp_dir.path());
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let task_id = {
            let mut state = OperatorState::new_persistent(&db_path).expect("persistent state");
            let task_id = start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
            .task_id;
            state.tasks.get_mut(&task_id).expect("task").status = OperatorTaskStatus::Planning;
            state
                .approvals
                .get_mut(&task_id)
                .expect("approvals")
                .clear();
            let events = state.events[&task_id].clone();
            state.state_machines.insert(
                task_id.clone(),
                TaskStateMachine::restore(task_id.clone(), OperatorTaskStatus::Planning, events),
            );
            state.persist().expect("persist planning task");
            task_id
        };

        let shared = Arc::new(Mutex::new(
            OperatorState::new_persistent(&db_path).expect("recover planning"),
        ));
        resume_task_in_shared_state_for_background(shared.clone(), task_id.clone())
            .expect("resume interrupted planning");
        {
            let state = shared.lock().expect("state lock");
            assert_eq!(
                state.tasks[&task_id].status,
                OperatorTaskStatus::WaitingApproval
            );
            assert!(!state.recovery_origins.contains_key(&task_id));
            assert_eq!(
                state.approvals[&task_id]
                    .iter()
                    .filter(|approval| approval.decision.is_none())
                    .count(),
                1
            );
        }
        let recovered = OperatorState::new_persistent(&db_path).expect("reload resumed planning");
        assert_eq!(
            recovered.tasks[&task_id].status,
            OperatorTaskStatus::WaitingApproval
        );
    }

    #[test]
    fn pending_validation_only_restarts_after_operator_resume() {
        let temp_dir = TestDir::new("persistent_validation_recovery");
        let project = create_godot_test_project(temp_dir.path());
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let task_id = {
            let mut state = OperatorState::new_persistent(&db_path).expect("persistent state");
            let task_id = start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
            .task_id;
            state
                .state_machines
                .get_mut(&task_id)
                .expect("machine")
                .approve()
                .expect("approve machine");
            state.tasks.get_mut(&task_id).expect("task").status = OperatorTaskStatus::Running;
            state.pending_validations.insert(
                task_id.clone(),
                PendingGodotValidation {
                    project_root: project.clone(),
                    patch_id: "patch_0123456789abcdef".to_string(),
                    backup_root: temp_dir
                        .path()
                        .join("validation-backup")
                        .to_string_lossy()
                        .to_string(),
                    files_applied: 1,
                },
            );
            state.persist().expect("persist pending validation");
            task_id
        };

        let recovered = OperatorState::new_persistent(&db_path).expect("recover validation");
        assert_eq!(recovered.tasks[&task_id].status, OperatorTaskStatus::Paused);
        assert!(recovered.pending_validations.contains_key(&task_id));
        let shared = Arc::new(Mutex::new(recovered));

        resume_task_in_shared_state_for_background(shared.clone(), task_id.clone())
            .expect("resume validation");
        let state = shared.lock().expect("state lock");
        assert_eq!(state.tasks[&task_id].status, OperatorTaskStatus::Completed);
        assert!(!state.pending_validations.contains_key(&task_id));
        assert!(state.events[&task_id]
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationSkipped)));
    }

    #[test]
    fn pending_patch_survives_restart_and_keeps_second_approval() {
        let temp_dir = TestDir::new("persistent_patch_approval");
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let shared = Arc::new(Mutex::new(
            OperatorState::new_persistent(&db_path).expect("persistent state"),
        ));
        let response = {
            let mut state = shared.lock().expect("state lock");
            start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
        };
        let plan_approval_id = {
            let state = shared.lock().expect("state lock");
            state.approvals[&response.task_id][0].approval_id.clone()
        };
        let run = {
            let mut state = shared.lock().expect("state lock");
            let run = approve_task_in_state_for_background(
                &mut state,
                ApproveRequest {
                    task_id: response.task_id.clone(),
                    approval_id: plan_approval_id,
                    decision: ApprovalDecision::Approve,
                    comment: Some("generate persistent proposal".to_string()),
                    expected_revision: None,
                },
                Some(fake_cli),
            )
            .expect("approve plan")
            .expect("background run");
            state.persist().expect("persist approved run");
            run
        };
        spawn_approved_task_runner(shared.clone(), run)
            .join()
            .expect("proposal runner");
        let patch_approval_id = {
            let state = shared.lock().expect("state lock");
            assert_eq!(
                state.tasks[&response.task_id].status,
                OperatorTaskStatus::WaitingApproval
            );
            assert!(state.pending_patches.contains_key(&response.task_id));
            state.approvals[&response.task_id]
                .iter()
                .find(|approval| {
                    approval.action == PATCH_APPLY_ACTION && approval.decision.is_none()
                })
                .expect("patch approval")
                .approval_id
                .clone()
        };
        drop(shared);

        let recovered = OperatorState::new_persistent(&db_path).expect("recover pending patch");
        assert!(recovered.pending_patches.contains_key(&response.task_id));
        assert!(recovered.approvals[&response.task_id]
            .iter()
            .any(
                |approval| approval.approval_id == patch_approval_id && approval.decision.is_none()
            ));
        let recovered = Arc::new(Mutex::new(recovered));
        approve_task_in_shared_state_for_background(
            recovered.clone(),
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval_id,
                decision: ApprovalDecision::Approve,
                comment: Some("apply recovered patch".to_string()),
                expected_revision: None,
            },
        )
        .expect("approve recovered patch");

        let backup_root = {
            let state = recovered.lock().expect("state lock");
            assert_eq!(
                state.tasks[&response.task_id].status,
                OperatorTaskStatus::Completed
            );
            assert!(!state.pending_patches.contains_key(&response.task_id));
            assert!(std::fs::read_to_string(&player)
                .expect("read applied player")
                .contains("jumps = 2"));
            state.events[&response.task_id]
                .iter()
                .find(|event| matches!(event.event_type, OperatorEventType::FilePatchApplied))
                .and_then(|event| event.payload.as_ref())
                .and_then(|payload| payload.get("backup_root"))
                .and_then(|value| value.as_str())
                .map(PathBuf::from)
                .expect("backup root in applied event")
        };
        assert!(is_d_drive_path(&backup_root));
        assert!(backup_root.exists());
        drop(recovered);

        let final_state = OperatorState::new_persistent(&db_path).expect("reload completed task");
        assert_eq!(
            final_state.tasks[&response.task_id].status,
            OperatorTaskStatus::Completed
        );
        assert!(final_state.events[&response.task_id]
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationSkipped)));
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn stale_pending_patch_is_failed_and_approval_is_closed_on_recovery() {
        let temp_dir = TestDir::new("persistent_stale_patch");
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let db_path = temp_dir.path().join("operator-state.sqlite3");
        let (task_id, patch_approval_id) = {
            let mut state = OperatorState::new_persistent(&db_path).expect("persistent state");
            let task_id = start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("start task")
            .task_id;
            let now = chrono::Utc::now().to_rfc3339();
            let plan_approval = state
                .approvals
                .get_mut(&task_id)
                .and_then(|approvals| approvals.first_mut())
                .expect("plan approval");
            plan_approval.decision = Some(ApprovalDecision::Approve);
            plan_approval.resolved_at = Some(now.clone());
            plan_approval.resolved_by = Some("test".to_string());

            let artifact = serde_json::to_string(&json!({
                "version": 1,
                "summary": "Stale recovery test",
                "changes": [{
                    "path": "scripts/player.gd",
                    "operation": "replace",
                    "content": "extends Node\nvar jumps = 2\n"
                }],
                "validation": []
            }))
            .expect("serialize patch");
            let patch = crate::operator::prepare_structured_patch(&artifact, &project)
                .expect("prepare patch");
            let patch_id = patch.patch_id.clone();
            let approval_id = format!("approval_{}_stale_patch", task_id);
            state.pending_patches.insert(task_id.clone(), patch);
            state
                .approvals
                .get_mut(&task_id)
                .expect("approvals")
                .push(ApprovalRequest {
                    approval_id: approval_id.clone(),
                    task_id: task_id.clone(),
                    task_revision: state.tasks.get(&task_id).map(|task| task.revision).unwrap_or(0),
                    level: ApprovalLevel::Approve,
                    action: PATCH_APPLY_ACTION.to_string(),
                    title: "Review stale patch".to_string(),
                    reason: "Apply generated patch".to_string(),
                    risk: Some("Modifies project files".to_string()),
                    preview: Some(ApprovalPreview {
                        files: Some(vec!["scripts/player.gd".to_string()]),
                        diff_id: Some(patch_id),
                        command: None,
                        diffs: None,
                    }),
                    options: vec![
                        ApprovalDecision::Approve,
                        ApprovalDecision::Reject,
                        ApprovalDecision::RequestChanges,
                    ],
                    created_at: now,
                    resolved_at: None,
                    decision: None,
                    resolved_by: None,
                });
            state.persist().expect("persist pending patch");
            (task_id, approval_id)
        };

        std::fs::write(&player, "extends Node\nvar jumps = 99\n")
            .expect("operator edit while stopped");
        let recovered = OperatorState::new_persistent(&db_path).expect("recover stale patch");
        assert_eq!(recovered.tasks[&task_id].status, OperatorTaskStatus::Failed);
        assert!(!recovered.pending_patches.contains_key(&task_id));
        let patch_approval = recovered.approvals[&task_id]
            .iter()
            .find(|approval| approval.approval_id == patch_approval_id)
            .expect("recovered patch approval");
        assert!(matches!(
            patch_approval.decision.as_ref(),
            Some(ApprovalDecision::RequestChanges)
        ));
        assert_eq!(
            patch_approval.resolved_by.as_deref(),
            Some("operator_recovery")
        );
        assert!(recovered.events[&task_id].iter().any(|event| {
            event.source == "operator_recovery"
                && matches!(event.event_type, OperatorEventType::TaskFailed)
        }));
        assert!(std::fs::read_to_string(&player)
            .expect("read operator edit")
            .contains("jumps = 99"));
    }

    fn godot_request(mode: TaskMode) -> StartTaskRequest {
        StartTaskRequest {
            domain: "game.godot".to_string(),
            project_path: "test-godot-project".to_string(),
            goal: "Add double jump to Player".to_string(),
            mode: Some(mode),
            approval_policy: Some(ApprovalPolicy::SafeDefault),
        }
    }

    fn godot_request_for_project(mode: TaskMode, project_path: &Path) -> StartTaskRequest {
        let mut request = godot_request(mode);
        request.project_path = project_path.to_string_lossy().to_string();
        request
    }

    #[test]
    #[ignore = "requires the real D: Hermes Game CLI and Godot executable"]
    fn real_hermes_godot_closed_loop_applies_and_validates_a_patch() {
        let cli = std::env::var_os("ACP_REAL_HERMES_GAME_CLI")
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root().join("bin").join("hermes-game.exe"));
        let godot = std::env::var_os("ACP_REAL_GODOT_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("D:/dev-tools/godot/4.7-stable/Godot_v4.7-stable_win64_console.exe"));
        assert!(cli.is_file(), "real Hermes Game CLI missing: {}", cli.display());
        assert!(godot.is_file(), "real Godot executable missing: {}", godot.display());

        let temp_dir = TestDir::new("real_hermes_godot_closed_loop");
        let project = create_godot_test_project(temp_dir.path());
        let original = std::fs::read_to_string(project.join("scripts/player.gd"))
            .expect("read original player");
        let mut state = OperatorState::new();
        state.godot_validation_discovery = GodotValidationDiscovery::Auto;
        std::env::set_var("GODOT_BIN", &godot);

        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("start real task");
        let plan_approval = state.approvals[&response.task_id][0].approval_id.clone();
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: plan_approval,
                decision: ApprovalDecision::Approve,
                comment: Some("real Hermes proposal approval".to_string()),
                expected_revision: None,
            },
            Some(cli),
        )
        .expect("real Hermes proposal");

        let patch_approval = state.approvals[&response.task_id]
            .iter()
            .find(|approval| approval.action == PATCH_APPLY_ACTION && approval.decision.is_none())
            .map(|approval| approval.approval_id.clone())
            .expect("real patch approval");
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval,
                decision: ApprovalDecision::Approve,
                comment: Some("real patch approval".to_string()),
                expected_revision: None,
            },
            None,
        )
        .expect("real patch apply and Godot validation");

        assert_eq!(state.tasks[&response.task_id].status, OperatorTaskStatus::Completed);
        assert_ne!(
            std::fs::read_to_string(project.join("scripts/player.gd")).expect("read changed player"),
            original
        );
        assert!(state.events[&response.task_id]
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationPassed)));
        cleanup_task_artifacts(&response.task_id);
    }

    fn create_godot_test_project(dir: &Path) -> PathBuf {
        let project = dir.join("godot-project");
        std::fs::create_dir_all(project.join("scripts")).expect("create Godot scripts");
        std::fs::write(
            project.join("project.godot"),
            "[application]\nconfig/name=\"Operator Patch Test\"\nrun/main_scene=\"res://main.tscn\"\n",
        )
        .expect("write Godot project marker");
        std::fs::write(
            project.join("main.tscn"),
            "[gd_scene format=3]\n\n[node name=\"Main\" type=\"Node\"]\n",
        )
        .expect("write main scene");
        std::fs::write(
            project.join("scripts/player.gd"),
            "extends Node\nvar jumps = 1\n",
        )
        .expect("write player script");
        project
    }

    fn cleanup_task_artifacts(task_id: &str) {
        let path = workspace_root()
            .join(".operator")
            .join("hermes-runs")
            .join(sanitize_operator_identifier(task_id));
        if is_d_drive_path(&path) {
            let _ = std::fs::remove_dir_all(path);
        }
    }

    fn prepare_applied_patch_waiting_validation(
        name: &str,
    ) -> (TestDir, OperatorState, StartTaskResponse, PathBuf) {
        let temp_dir = TestDir::new(name);
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("task should start");
        let plan_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("plan approval id");
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: plan_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("generate structured patch");
        let patch_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.action == PATCH_APPLY_ACTION)
            })
            .map(|approval| approval.approval_id.clone())
            .expect("patch approval id");
        let request = ApproveRequest {
            task_id: response.task_id.clone(),
            approval_id: patch_approval_id,
            decision: ApprovalDecision::Approve,
            comment: Some("apply before synthetic validation".to_string()),
            expected_revision: None,
        };
        let approval =
            resolve_approval_record(&mut state, &request).expect("resolve patch approval");
        let validation_pending = resolve_patch_approval_in_state(
            &mut state,
            &request.task_id,
            approval,
            request.decision,
            request.comment,
        )
        .expect("apply approved patch");
        assert!(validation_pending);
        assert!(state.pending_validations.contains_key(&response.task_id));

        (temp_dir, state, response, player)
    }

    fn synthetic_validation_result(status: GodotValidationStatus) -> GodotValidationResult {
        GodotValidationResult {
            status,
            executable: Some("D:/test/Godot.exe".to_string()),
            args: vec!["--headless".to_string(), "--editor".to_string()],
            exit_code: Some(if status == GodotValidationStatus::Passed {
                0
            } else {
                1
            }),
            stdout: "Godot validation output".to_string(),
            stderr: String::new(),
            output_truncated: false,
            duration_ms: 25,
            message: if status == GodotValidationStatus::Passed {
                "Godot headless validation passed".to_string()
            } else {
                "Godot headless validation failed".to_string()
            },
        }
    }

    fn create_fake_hermes_cli(dir: &Path) -> PathBuf {
        #[cfg(windows)]
        {
            let cmd_path = dir.join("hermes.cmd");
            std::fs::write(
                &cmd_path,
                r#"@echo off
if "%~1"=="--version" (
  echo hermes-test 1.0.0
  exit /b 0
)

if "%~1"=="execute" (
  echo {"type":"task_started","task_id":"fake-task"}
  echo {"type":"file_modified","task_id":"fake-task","path":"scripts/Player.gd","diff":"@@ fake diff"}
  echo {"type":"tool_result","task_id":"fake-task","tool":"godot_check","output":{"ok":true}}
  exit /b 0
)

echo unexpected args: %* 1>&2
exit /b 1
"#,
            )
            .expect("write fake hermes cmd");

            cmd_path
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            let sh_path = dir.join("hermes");
            std::fs::write(
                &sh_path,
                r#"#!/usr/bin/env sh
if [ "$1" = "--version" ]; then
  echo "hermes-test 1.0.0"
  exit 0
fi

if [ "$1" = "execute" ]; then
  echo '{"type":"task_started","task_id":"fake-task"}'
  echo '{"type":"file_modified","task_id":"fake-task","path":"scripts/Player.gd","diff":"@@ fake diff"}'
  echo '{"type":"tool_result","task_id":"fake-task","tool":"godot_check","output":{"ok":true}}'
  exit 0
fi

echo "unexpected args: $*" >&2
exit 1
"#,
            )
            .expect("write fake hermes shell");
            let mut permissions = std::fs::metadata(&sh_path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&sh_path, permissions).expect("chmod fake hermes");

            sh_path
        }
    }

    fn create_fake_hermes_game_cli(dir: &Path) -> PathBuf {
        #[cfg(windows)]
        {
            let cmd_path = dir.join("hermes-game.cmd");
            std::fs::write(
                &cmd_path,
                r#"@echo off
if "%~1"=="--version" (
  echo hermes-game-test 0.4.0
  exit /b 0
)

if "%~1"=="--engine" if "%~2"=="godot" if "%~3"=="codegen" if "%~4"=="--help" (
  echo --no-tools --prompt-file
  exit /b 0
)

if "%~1"=="--engine" if "%~2"=="godot" if "%~3"=="codegen" (
  if not "%~7"=="--no-tools" (
    echo proposal mode must disable tools 1>&2
    exit /b 2
  )
  if not exist "%~dp6" mkdir "%~dp6"
  echo {"version":1,"summary":"Add a controlled double jump","changes":[{"path":"scripts/player.gd","operation":"replace","content":"extends Node\nvar jumps = 2\n"}],"validation":["Open the main scene"]}> "%~6"
  echo generated proposal
  exit /b 0
)

echo unexpected args: %* 1>&2
exit /b 1
"#,
            )
            .expect("write fake hermes-game cmd");

            cmd_path
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            let sh_path = dir.join("hermes-game");
            std::fs::write(
                &sh_path,
                r#"#!/usr/bin/env sh
if [ "$1" = "--version" ]; then
  echo "hermes-game-test 0.4.0"
  exit 0
fi

if [ "$1" = "--engine" ] && [ "$2" = "godot" ] && [ "$3" = "codegen" ] && [ "$4" = "--help" ]; then
  echo "--no-tools --prompt-file"
  exit 0
fi

if [ "$1" = "--engine" ] && [ "$2" = "godot" ] && [ "$3" = "codegen" ]; then
  if [ "$7" != "--no-tools" ]; then
    echo "proposal mode must disable tools" >&2
    exit 2
  fi
  mkdir -p "$(dirname "$6")"
  printf '%s' '{"version":1,"summary":"Add a controlled double jump","changes":[{"path":"scripts/player.gd","operation":"replace","content":"extends Node\nvar jumps = 2\n"}],"validation":["Open the main scene"]}' > "$6"
  echo "generated proposal"
  exit 0
fi

echo "unexpected args: $*" >&2
exit 1
"#,
            )
            .expect("write fake hermes-game shell");
            let mut permissions = std::fs::metadata(&sh_path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&sh_path, permissions).expect("chmod fake hermes-game");

            sh_path
        }
    }

    fn create_slow_hermes_game_cli(dir: &Path) -> PathBuf {
        #[cfg(windows)]
        {
            let cmd_path = dir.join("hermes-game-slow.cmd");
            std::fs::write(
                &cmd_path,
                r#"@echo off
if "%~1"=="--version" (
  echo hermes-game-test 0.4.0
  exit /b 0
)

if "%~1"=="--engine" if "%~2"=="godot" if "%~3"=="codegen" if "%~4"=="--help" (
  echo --no-tools --prompt-file
  exit /b 0
)

if "%~1"=="--engine" if "%~2"=="godot" if "%~3"=="codegen" (
:loop
  goto loop
)

echo unexpected args: %* 1>&2
exit /b 1
"#,
            )
            .expect("write slow hermes-game cmd");

            cmd_path
        }

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;

            let sh_path = dir.join("hermes-game-slow");
            std::fs::write(
                &sh_path,
                r#"#!/usr/bin/env sh
if [ "$1" = "--version" ]; then
  echo "hermes-game-test 0.4.0"
  exit 0
fi

if [ "$1" = "--engine" ] && [ "$2" = "godot" ] && [ "$3" = "codegen" ] && [ "$4" = "--help" ]; then
  echo "--no-tools --prompt-file"
  exit 0
fi

if [ "$1" = "--engine" ] && [ "$2" = "godot" ] && [ "$3" = "codegen" ]; then
  while true; do :; done
fi

echo "unexpected args: $*" >&2
exit 1
"#,
            )
            .expect("write slow hermes-game shell");
            let mut permissions = std::fs::metadata(&sh_path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&sh_path, permissions).expect("chmod slow hermes-game");

            sh_path
        }
    }

    #[test]
    fn start_task_drives_godot_planning_to_approval() {
        let mut state = OperatorState::new();

        let response = start_task_in_state(&mut state, godot_request(TaskMode::ProposeThenApply))
            .expect("task should start");

        assert_eq!(response.status, OperatorTaskStatus::WaitingApproval);

        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::WaitingApproval);

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, OperatorEventType::ProjectAnalyzed)));
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, OperatorEventType::PlanReady)));
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, OperatorEventType::ApprovalRequested)));

        let plan_ready = events
            .iter()
            .find(|e| matches!(e.event_type, OperatorEventType::PlanReady))
            .expect("plan ready event");
        let steps = plan_ready
            .payload
            .as_ref()
            .and_then(|payload| payload.get("steps"))
            .and_then(|steps| steps.as_array())
            .expect("plan steps payload");
        assert!(!steps.is_empty());

        let approvals = state
            .approvals
            .get(&response.task_id)
            .expect("approvals stored");
        assert_eq!(approvals.iter().filter(|a| a.decision.is_none()).count(), 1);
    }

    #[test]
    fn requesting_plan_changes_creates_a_fresh_plan_approval() {
        let mut state = OperatorState::new();
        let response = start_task_in_state(&mut state, godot_request(TaskMode::ProposeThenApply))
            .expect("task should start");
        let first_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("first plan approval");

        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: first_approval_id.clone(),
                decision: ApprovalDecision::RequestChanges,
                comment: Some("revise the implementation plan".to_string()),
                expected_revision: None,
            },
            None,
        )
        .expect("request changes should replan");

        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::WaitingApproval)
        );
        let approvals = state
            .approvals
            .get(&response.task_id)
            .expect("approvals stored");
        assert_eq!(
            approvals
                .iter()
                .filter(|approval| approval.decision.is_none())
                .count(),
            1
        );
        assert_eq!(
            approvals
                .iter()
                .filter(|approval| approval.decision.is_some())
                .count(),
            1
        );
        let revised_approval_id = approvals
            .iter()
            .find(|approval| approval.decision.is_none())
            .map(|approval| approval.approval_id.clone())
            .expect("revised plan approval");
        assert_ne!(revised_approval_id, first_approval_id);
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: revised_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            None,
        )
        .expect("fresh approval should remain actionable");
        // Without Hermes CLI, task should be Failed (not Completed)
        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::Failed)
        );
        assert!(
            state.tasks.get(&response.task_id).and_then(|task| task.error.as_ref()).map(|e| e.contains("EXECUTOR_UNAVAILABLE")).unwrap_or(false),
            "Task error should contain EXECUTOR_UNAVAILABLE"
        );
    }

    #[test]
    fn approving_ready_task_completes_local_control_loop() {
        let mut state = OperatorState::new();
        let response = start_task_in_state(&mut state, godot_request(TaskMode::ProposeThenApply))
            .expect("task should start");

        let approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("approval id");

        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id,
                decision: ApprovalDecision::Approve,
                comment: Some("go".to_string()),
                expected_revision: None,
            },
            None,
        )
        .expect("approval should fail task");

        // Without Hermes CLI, task should be Failed (not Completed)
        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::Failed);
        assert!(task.completed_at.is_some());
        assert!(task.error.as_ref().unwrap().contains("EXECUTOR_UNAVAILABLE"));

        let events = state.events.get(&response.task_id).expect("events stored");
        // Should have EXECUTOR_UNAVAILABLE error event
        assert!(events.iter().any(|e| {
            e.event_type == OperatorEventType::HookFailed
                && e.message.as_ref().map(|m| m.contains("EXECUTOR_UNAVAILABLE")).unwrap_or(false)
        }));
        // Should have TaskFailed, NOT TaskCompleted
        assert!(events
            .iter()
            .any(|e| matches!(e.event_type, OperatorEventType::TaskFailed)));
        assert!(!events
            .iter()
            .any(|e| matches!(e.event_type, OperatorEventType::TaskCompleted)));
    }

    #[test]
    fn controlled_game_task_blocks_generic_hermes_direct_writes() {
        let temp_dir = TestDir::new("fake_hermes_cli");
        let fake_cli = create_fake_hermes_cli(temp_dir.path());
        let mut state = OperatorState::new();
        let response = start_task_in_state(&mut state, godot_request(TaskMode::ProposeThenApply))
            .expect("task should start");

        let approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("approval id");

        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id,
                decision: ApprovalDecision::Approve,
                comment: Some("run hermes".to_string()),
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("approval should fail with executor unavailable");

        // Task should be Failed (not Completed) when Hermes is unavailable
        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::Failed);
        assert!(task.error.as_ref().unwrap().contains("EXECUTOR_UNAVAILABLE"));

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events.iter().any(|event| {
            event.source == "operator_patch_engine"
                && matches!(event.event_type, OperatorEventType::HookFailed)
        }));
        assert!(!events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::FileModified)));
        // Should have TaskFailed, NOT TaskCompleted
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskFailed)));
        assert!(!events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskCompleted)));

        let changes = state
            .file_changes
            .get(&response.task_id)
            .expect("file changes stored");
        assert!(changes.is_empty());
    }

    #[test]
    fn approving_ready_task_uses_hermes_game_cli_when_available() {
        let temp_dir = TestDir::new("fake_hermes_game_cli");
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("task should start");

        let approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("approval id");

        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id,
                decision: ApprovalDecision::Approve,
                comment: Some("run hermes-game".to_string()),
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("approval should execute via hermes-game cli");

        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::WaitingApproval);
        assert!(task
            .summary
            .as_deref()
            .unwrap_or_default()
            .contains("structured file change"));
        assert_eq!(
            std::fs::read_to_string(&player).expect("read untouched player"),
            "extends Node\nvar jumps = 1\n"
        );

        let patch_approval = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.action == PATCH_APPLY_ACTION)
            })
            .cloned()
            .expect("patch approval");
        let preview = patch_approval.preview.as_ref().expect("patch preview");
        assert_eq!(
            preview.files.as_deref(),
            Some(&["scripts/player.gd".to_string()][..])
        );
        assert!(preview
            .diffs
            .as_ref()
            .and_then(|diffs| diffs.first())
            .map(|diff| diff.diff.contains("+var jumps = 2"))
            .unwrap_or(false));

        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval.approval_id,
                decision: ApprovalDecision::Approve,
                comment: Some("apply reviewed patch".to_string()),
                expected_revision: None,
            },
            None,
        )
        .expect("patch approval should apply files");

        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::Completed);
        assert!(std::fs::read_to_string(&player)
            .expect("read applied player")
            .contains("jumps = 2"));
        assert!(!state.pending_patches.contains_key(&response.task_id));

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events.iter().any(|event| event.source == "hermes_game_cli"));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::FilePatchProposed)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::FilePatchApplied)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationSkipped)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskCompleted)));

        let changes = state
            .file_changes
            .get(&response.task_id)
            .expect("file changes stored");
        let artifact = changes
            .iter()
            .find(|change| change.change_type == "artifact")
            .map(|change| PathBuf::from(&change.path))
            .expect("artifact change recorded");

        assert!(is_d_drive_path(&artifact));
        assert!(artifact.exists());
        assert!(std::fs::read_to_string(&artifact)
            .expect("artifact content")
            .contains("\"version\":1"));

        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn passed_godot_validation_completes_the_applied_patch_task() {
        let (_temp_dir, mut state, response, player) =
            prepare_applied_patch_waiting_validation("passed_godot_validation");
        let pending = state
            .pending_validations
            .get(&response.task_id)
            .cloned()
            .expect("pending validation");
        push_validation_started(
            &mut state,
            &response.task_id,
            &pending,
            Path::new("D:/test/Godot.exe"),
        );

        finish_godot_validation_in_state(
            &mut state,
            &response.task_id,
            synthetic_validation_result(GodotValidationStatus::Passed),
        )
        .expect("finish validation");

        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::Completed)
        );
        assert!(!state.pending_validations.contains_key(&response.task_id));
        assert!(std::fs::read_to_string(player)
            .expect("read applied player")
            .contains("jumps = 2"));
        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationStarted)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::ValidationPassed)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskCompleted)));
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn failed_godot_validation_keeps_applied_files_and_backup_visible() {
        let (_temp_dir, mut state, response, player) =
            prepare_applied_patch_waiting_validation("failed_godot_validation");

        let error = finish_godot_validation_in_state(
            &mut state,
            &response.task_id,
            synthetic_validation_result(GodotValidationStatus::Failed),
        )
        .expect_err("failed validation must fail task");

        assert!(error.contains("remain applied"));
        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::Failed)
        );
        assert!(!state.pending_validations.contains_key(&response.task_id));
        assert!(std::fs::read_to_string(player)
            .expect("read applied player")
            .contains("jumps = 2"));
        let events = state.events.get(&response.task_id).expect("events stored");
        let backup_root = events
            .iter()
            .find(|event| matches!(event.event_type, OperatorEventType::ValidationFailed))
            .and_then(|event| event.payload.as_ref())
            .and_then(|payload| payload.get("backup_root"))
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
            .expect("validation failure backup root");
        assert_eq!(
            std::fs::read_to_string(backup_root.join("scripts/player.gd")).expect("read backup"),
            "extends Node\nvar jumps = 1\n"
        );
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn patch_approval_refuses_to_overwrite_an_operator_edit() {
        let temp_dir = TestDir::new("stale_hermes_game_patch");
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("task should start");
        let plan_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("plan approval id");
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: plan_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("generate structured patch");

        let patch_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.action == PATCH_APPLY_ACTION)
            })
            .map(|approval| approval.approval_id.clone())
            .expect("patch approval id");
        std::fs::write(&player, "extends Node\nvar jumps = 3 # operator edit\n")
            .expect("operator edit");

        let error = approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            None,
        )
        .expect_err("stale patch must fail");

        assert!(error.contains("stale patch target"));
        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::Failed)
        );
        assert!(std::fs::read_to_string(player)
            .expect("read operator edit")
            .contains("operator edit"));
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn stopping_while_patch_waits_invalidates_the_pending_write() {
        let temp_dir = TestDir::new("stop_pending_hermes_game_patch");
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("task should start");
        let plan_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("plan approval id");
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: plan_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("generate structured patch");
        let patch_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.action == PATCH_APPLY_ACTION)
            })
            .map(|approval| approval.approval_id.clone())
            .expect("patch approval id");

        stop_task_in_state(&mut state, &response.task_id).expect("stop waiting task");

        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::Cancelled)
        );
        assert!(!state.pending_patches.contains_key(&response.task_id));
        assert!(approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            None,
        )
        .is_err());
        assert_eq!(
            std::fs::read_to_string(player).expect("read untouched player"),
            "extends Node\nvar jumps = 1\n"
        );
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn missing_pending_patch_does_not_consume_the_approval() {
        let temp_dir = TestDir::new("missing_pending_hermes_game_patch");
        let fake_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let player = project.join("scripts/player.gd");
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            godot_request_for_project(TaskMode::ProposeThenApply, &project),
        )
        .expect("task should start");
        let plan_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| approvals.first())
            .map(|approval| approval.approval_id.clone())
            .expect("plan approval id");
        approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: plan_approval_id,
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            Some(fake_cli),
        )
        .expect("generate structured patch");
        let patch_approval_id = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.action == PATCH_APPLY_ACTION)
            })
            .map(|approval| approval.approval_id.clone())
            .expect("patch approval id");
        state.pending_patches.remove(&response.task_id);

        let error = approve_task_in_state_with_cli_path(
            &mut state,
            ApproveRequest {
                task_id: response.task_id.clone(),
                approval_id: patch_approval_id.clone(),
                decision: ApprovalDecision::Approve,
                comment: None,
                expected_revision: None,
            },
            None,
        )
        .expect_err("missing patch payload must block approval resolution");

        assert!(error.contains("Pending patch not found"));
        assert_eq!(
            state.tasks.get(&response.task_id).map(|task| &task.status),
            Some(&OperatorTaskStatus::WaitingApproval)
        );
        let patch_approval = state
            .approvals
            .get(&response.task_id)
            .and_then(|approvals| {
                approvals
                    .iter()
                    .find(|approval| approval.approval_id == patch_approval_id)
            })
            .expect("patch approval remains stored");
        assert!(patch_approval.decision.is_none());
        assert!(patch_approval.resolved_at.is_none());
        assert_eq!(
            std::fs::read_to_string(player).expect("read untouched player"),
            "extends Node\nvar jumps = 1\n"
        );
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn background_hermes_game_task_can_be_cancelled() {
        let temp_dir = TestDir::new("slow_hermes_game_cli");
        let fake_cli = create_slow_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let state = Arc::new(Mutex::new(OperatorState::new()));

        let response = {
            let mut state = state.lock().expect("state lock");
            start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("task should start")
        };

        let approval_id = {
            let state = state.lock().expect("state lock");
            state
                .approvals
                .get(&response.task_id)
                .and_then(|approvals| approvals.first())
                .map(|approval| approval.approval_id.clone())
                .expect("approval id")
        };

        let run = {
            let mut state = state.lock().expect("state lock");
            approve_task_in_state_for_background(
                &mut state,
                ApproveRequest {
                    task_id: response.task_id.clone(),
                    approval_id,
                    decision: ApprovalDecision::Approve,
                    comment: Some("run slow hermes-game".to_string()),
                    expected_revision: None,
                },
                Some(fake_cli),
            )
            .expect("approval should prepare background run")
            .expect("approved run")
        };

        let handle = spawn_approved_task_runner(state.clone(), run);
        std::thread::sleep(std::time::Duration::from_millis(250));

        {
            let mut state = state.lock().expect("state lock");
            cancel_task_in_state(&mut state, &response.task_id).expect("task should cancel");
        }

        handle.join().expect("background runner join");

        let state = state.lock().expect("state lock");
        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::Cancelled);
        assert!(task.completed_at.is_some());

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskCancelled)));
        assert!(events.iter().any(|event| event.source == "hermes_game_cli"
            && matches!(event.event_type, OperatorEventType::TaskCancelled)));
        drop(state);
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn background_hermes_game_task_can_be_redirected() {
        let temp_dir = TestDir::new("redirect_slow_hermes_game_cli");
        let fake_cli = create_slow_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let state = Arc::new(Mutex::new(OperatorState::new()));

        let response = {
            let mut state = state.lock().expect("state lock");
            start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("task should start")
        };

        let approval_id = {
            let state = state.lock().expect("state lock");
            state
                .approvals
                .get(&response.task_id)
                .and_then(|approvals| approvals.first())
                .map(|approval| approval.approval_id.clone())
                .expect("approval id")
        };

        let run = {
            let mut state = state.lock().expect("state lock");
            approve_task_in_state_for_background(
                &mut state,
                ApproveRequest {
                    task_id: response.task_id.clone(),
                    approval_id,
                    decision: ApprovalDecision::Approve,
                    comment: Some("run slow hermes-game".to_string()),
                    expected_revision: None,
                },
                Some(fake_cli),
            )
            .expect("approval should prepare background run")
            .expect("approved run")
        };

        let handle = spawn_approved_task_runner(state.clone(), run);
        std::thread::sleep(std::time::Duration::from_millis(250));

        {
            let mut state = state.lock().expect("state lock");
            redirect_task_in_state(
                &mut state,
                RedirectRequest {
                    task_id: response.task_id.clone(),
                    new_goal: "Add wall jump to Player".to_string(),
                    preserve_completed_work: true,
                    expected_revision: None,
                },
            )
            .expect("task should redirect");
        }

        handle.join().expect("background runner join");

        let state = state.lock().expect("state lock");
        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::WaitingApproval);
        assert_eq!(task.goal, "Add wall jump to Player");

        let pending_approvals = state
            .approvals
            .get(&response.task_id)
            .expect("approvals stored")
            .iter()
            .filter(|approval| approval.decision.is_none())
            .count();
        assert_eq!(pending_approvals, 1);

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events.iter().any(|event| event.source == "operator"
            && matches!(event.event_type, OperatorEventType::TaskRedirected)));
        assert!(events.iter().any(|event| event.source == "hermes_game_cli"
            && matches!(event.event_type, OperatorEventType::TaskRedirected)));
        drop(state);
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn background_hermes_game_task_can_pause_and_resume() {
        let temp_dir = TestDir::new("pause_resume_hermes_game_cli");
        let slow_cli = create_slow_hermes_game_cli(temp_dir.path());
        let fast_cli = create_fake_hermes_game_cli(temp_dir.path());
        let project = create_godot_test_project(temp_dir.path());
        let state = Arc::new(Mutex::new(OperatorState::new()));

        let response = {
            let mut state = state.lock().expect("state lock");
            start_task_in_state(
                &mut state,
                godot_request_for_project(TaskMode::ProposeThenApply, &project),
            )
            .expect("task should start")
        };

        let approval_id = {
            let state = state.lock().expect("state lock");
            state
                .approvals
                .get(&response.task_id)
                .and_then(|approvals| approvals.first())
                .map(|approval| approval.approval_id.clone())
                .expect("approval id")
        };

        let run = {
            let mut state = state.lock().expect("state lock");
            approve_task_in_state_for_background(
                &mut state,
                ApproveRequest {
                    task_id: response.task_id.clone(),
                    approval_id,
                    decision: ApprovalDecision::Approve,
                    comment: Some("run slow hermes-game".to_string()),
                    expected_revision: None,
                },
                Some(slow_cli),
            )
            .expect("approval should prepare background run")
            .expect("approved run")
        };

        let handle = spawn_approved_task_runner(state.clone(), run);
        std::thread::sleep(std::time::Duration::from_millis(250));

        {
            let mut state = state.lock().expect("state lock");
            pause_task_in_state(&mut state, &response.task_id).expect("task should pause");
        }

        handle.join().expect("background runner join");

        {
            let state = state.lock().expect("state lock");
            let task = state.tasks.get(&response.task_id).expect("task stored");
            assert_eq!(task.status, OperatorTaskStatus::Paused);
        }

        let resumed_run = {
            let mut state = state.lock().expect("state lock");
            resume_task_in_state_for_background(
                &mut state,
                &response.task_id,
                Some(fast_cli),
                None,
            )
                .expect("task should resume")
        };
        let resumed_handle = spawn_approved_task_runner(state.clone(), resumed_run);
        resumed_handle
            .join()
            .expect("resumed background runner join");

        {
            let mut state = state.lock().expect("state lock");
            let task = state.tasks.get(&response.task_id).expect("task stored");
            assert_eq!(task.status, OperatorTaskStatus::WaitingApproval);
            let patch_approval_id = state
                .approvals
                .get(&response.task_id)
                .and_then(|approvals| {
                    approvals.iter().find(|approval| {
                        approval.action == PATCH_APPLY_ACTION && approval.decision.is_none()
                    })
                })
                .map(|approval| approval.approval_id.clone())
                .expect("patch approval id");
            approve_task_in_state_with_cli_path(
                &mut state,
                ApproveRequest {
                    task_id: response.task_id.clone(),
                    approval_id: patch_approval_id,
                    decision: ApprovalDecision::Approve,
                    comment: Some("apply resumed proposal".to_string()),
                    expected_revision: None,
                },
                None,
            )
            .expect("apply resumed patch");
        }

        let state = state.lock().expect("state lock");
        let task = state.tasks.get(&response.task_id).expect("task stored");
        assert_eq!(task.status, OperatorTaskStatus::Completed);

        let events = state.events.get(&response.task_id).expect("events stored");
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskPaused)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskResumed)));
        assert!(events
            .iter()
            .any(|event| matches!(event.event_type, OperatorEventType::TaskCompleted)));
        drop(state);
        cleanup_task_artifacts(&response.task_id);
    }

    #[test]
    fn hermes_connection_status_accepts_explicit_cli_path() {
        let temp_dir = TestDir::new("fake_hermes_connection");
        let fake_cli = create_fake_hermes_cli(temp_dir.path());

        let status = hermes_connection_status_for_cli_path(fake_cli.clone());

        assert!(status.available);
        assert_eq!(status.version.as_deref(), Some("hermes-test 1.0.0"));
        assert_eq!(status.path, Some(fake_cli.to_string_lossy().to_string()));
        assert!(status.error.is_none());
    }

    #[test]
    fn operator_events_serialize_type_for_frontend() {
        let event = OperatorEvent {
            event_id: "evt_1".to_string(),
            task_id: "task_1".to_string(),
            sequence: 0,
            task_revision: 0,
            timestamp: "2026-07-09T00:00:00Z".to_string(),
            event_type: OperatorEventType::PlanReady,
            level: EventLevel::Info,
            title: "Plan ready".to_string(),
            message: None,
            source: "test".to_string(),
            payload: None,
            title_key: None,
            message_key: None,
            title_args: None,
            message_args: None,
        };

        let json = serde_json::to_value(event).expect("serializes");
        assert_eq!(
            json.get("type").and_then(|v| v.as_str()),
            Some("plan_ready")
        );
        assert!(json.get("event_type").is_none());
    }
}
