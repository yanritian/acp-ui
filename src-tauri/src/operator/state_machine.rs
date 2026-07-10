// Operator State Machine - Task lifecycle management
// Implements the state transitions defined in the protocol

use crate::operator::types::{EventLevel, OperatorEvent, OperatorEventType, OperatorTaskStatus};
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct TaskStateMachine {
    task_id: String,
    current_status: OperatorTaskStatus,
    event_history: Arc<Mutex<Vec<OperatorEvent>>>,
    event_counter: u64,
}

impl TaskStateMachine {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            current_status: OperatorTaskStatus::Idle,
            event_history: Arc::new(Mutex::new(Vec::new())),
            event_counter: 0,
        }
    }

    pub(crate) fn restore(
        task_id: String,
        status: OperatorTaskStatus,
        event_history: Vec<OperatorEvent>,
    ) -> Self {
        Self {
            task_id,
            current_status: status,
            event_counter: event_history.len() as u64,
            event_history: Arc::new(Mutex::new(event_history)),
        }
    }

    pub(crate) fn resume_interrupted_planning(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Paused {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Planning,
                reason: "Can only restart interrupted planning from Paused state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Planning, "Interrupted planning resumed")
    }

    pub fn current_status(&self) -> &OperatorTaskStatus {
        &self.current_status
    }

    pub fn event_history(&self) -> Result<Vec<OperatorEvent>, StateError> {
        self.event_history
            .lock()
            .map(|guard| guard.clone())
            .map_err(|_| StateError::LockError {
                reason: "Failed to acquire lock on event history".to_string(),
            })
    }

    // ========================================================================
    // State Transitions
    // ========================================================================

    pub fn start_task(&mut self) -> Result<(), StateError> {
        self.transition(OperatorTaskStatus::Planning, "Task started")
    }

    pub fn plan_ready(&mut self) -> Result<(), StateError> {
        self.transition(
            OperatorTaskStatus::WaitingApproval,
            "Plan ready for approval",
        )
    }

    pub fn approve(&mut self) -> Result<(), StateError> {
        self.transition(OperatorTaskStatus::Running, "Plan approved")
    }

    pub fn await_approval(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Running {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::WaitingApproval,
                reason: "Can only request an execution approval from Running state".to_string(),
            });
        }
        self.transition(
            OperatorTaskStatus::WaitingApproval,
            "Generated changes are ready for approval",
        )
    }

    pub fn request_changes(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::WaitingApproval {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Planning,
                reason: "Can only request changes from WaitingApproval state".to_string(),
            });
        }
        self.transition(
            OperatorTaskStatus::Planning,
            "Operator requested a revised proposal",
        )
    }

    pub fn reject(&mut self) -> Result<(), StateError> {
        self.transition(OperatorTaskStatus::Cancelled, "Plan rejected")
    }

    pub fn pause(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Running {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Paused,
                reason: "Can only pause from Running state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Paused, "Task paused")
    }

    pub fn resume(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Paused {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Running,
                reason: "Can only resume from Paused state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Running, "Task resumed")
    }

    pub fn redirect(&mut self) -> Result<(), StateError> {
        if !matches!(
            self.current_status,
            OperatorTaskStatus::Planning
                | OperatorTaskStatus::WaitingApproval
                | OperatorTaskStatus::Running
                | OperatorTaskStatus::Paused
        ) {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Redirecting,
                reason: "Can only redirect from an active task state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Redirecting, "Task redirected")
    }

    pub fn replan(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Redirecting {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Planning,
                reason: "Can only replan from Redirecting state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Planning, "Replanning after redirect")
    }

    pub fn stop(&mut self) -> Result<(), StateError> {
        if !matches!(
            self.current_status,
            OperatorTaskStatus::Planning
                | OperatorTaskStatus::WaitingApproval
                | OperatorTaskStatus::Running
                | OperatorTaskStatus::Paused
                | OperatorTaskStatus::Redirecting
        ) {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Cancelling,
                reason: "Can only stop an active task".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Cancelling, "Task stopping")
    }

    pub fn cleanup_done(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Cancelling {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Cancelled,
                reason: "Can only cleanup from Cancelling state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Cancelled, "Cleanup done")
    }

    pub fn complete(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Running {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Completed,
                reason: "Can only complete from Running state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Completed, "Task completed")
    }

    pub fn fail(&mut self, error: &str) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Running
            && self.current_status != OperatorTaskStatus::Paused
            && self.current_status != OperatorTaskStatus::Planning
        {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Failed,
                reason: "Can only fail from Running, Paused, or Planning state".to_string(),
            });
        }
        self.transition(
            OperatorTaskStatus::Failed,
            &format!("Task failed: {}", error),
        )
    }

    pub fn retry(&mut self) -> Result<(), StateError> {
        if self.current_status != OperatorTaskStatus::Failed {
            return Err(StateError::InvalidTransition {
                from: self.current_status.clone(),
                to: OperatorTaskStatus::Planning,
                reason: "Can only retry from Failed state".to_string(),
            });
        }
        self.transition(OperatorTaskStatus::Planning, "Retrying failed task")
    }

    // ========================================================================
    // Event Emission
    // ========================================================================

    fn transition(
        &mut self,
        new_status: OperatorTaskStatus,
        message: &str,
    ) -> Result<(), StateError> {
        let event_type = self.status_to_event_type(&new_status);
        let event = OperatorEvent {
            event_id: format!("evt_{}_{}", self.task_id, self.event_counter),
            task_id: self.task_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            event_type,
            level: EventLevel::Info,
            title: message.to_string(),
            message: Some(message.to_string()),
            source: "state_machine".to_string(),
            payload: None,
            title_key: None,
            message_key: None,
            title_args: None,
            message_args: None,
        };

        {
            let mut history = self
                .event_history
                .lock()
                .map_err(|_| StateError::LockError {
                    reason: "Failed to acquire lock on event history".to_string(),
                })?;
            history.push(event);
        }
        self.event_counter += 1;
        self.current_status = new_status;

        Ok(())
    }

    fn status_to_event_type(&self, status: &OperatorTaskStatus) -> OperatorEventType {
        match status {
            OperatorTaskStatus::Idle => OperatorEventType::TaskCreated,
            OperatorTaskStatus::Planning => OperatorEventType::PlanStarted,
            OperatorTaskStatus::WaitingApproval => OperatorEventType::PlanReady,
            OperatorTaskStatus::Running => OperatorEventType::TaskStarted,
            OperatorTaskStatus::Paused => OperatorEventType::TaskPaused,
            OperatorTaskStatus::Redirecting => OperatorEventType::TaskRedirected,
            OperatorTaskStatus::Cancelling => OperatorEventType::TaskCancelling,
            OperatorTaskStatus::Cancelled => OperatorEventType::TaskCancelled,
            OperatorTaskStatus::Failed => OperatorEventType::TaskFailed,
            OperatorTaskStatus::Completed => OperatorEventType::TaskCompleted,
        }
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum StateError {
    InvalidTransition {
        from: OperatorTaskStatus,
        to: OperatorTaskStatus,
        reason: String,
    },
    LockError {
        reason: String,
    },
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::InvalidTransition { from, to, reason } => {
                write!(
                    f,
                    "Invalid state transition from {:?} to {:?}: {}",
                    from, to, reason
                )
            }
            StateError::LockError { reason } => {
                write!(f, "Lock error: {}", reason)
            }
        }
    }
}

impl std::error::Error for StateError {}
