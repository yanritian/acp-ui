// Approval Queue - Manage approval requests for dangerous operations

use crate::operator::{ApprovalDecision, ApprovalLevel, ApprovalRequest};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// ============================================================================
// Approval Queue
// ============================================================================

pub struct ApprovalQueue {
    queue: Arc<Mutex<VecDeque<ApprovalRequest>>>,
    task_id: String,
}

impl ApprovalQueue {
    pub fn new(task_id: String) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            task_id,
        }
    }

    /// Add an approval request to the queue
    pub fn enqueue(&self, request: ApprovalRequest) -> Result<(), ApprovalQueueError> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        queue.push_back(request);
        Ok(())
    }

    /// Get the next pending approval request
    pub fn dequeue(&self) -> Result<Option<ApprovalRequest>, ApprovalQueueError> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        Ok(queue.pop_front())
    }

    /// Get all pending approval requests
    pub fn get_pending(&self) -> Result<Vec<ApprovalRequest>, ApprovalQueueError> {
        let queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        Ok(queue
            .iter()
            .filter(|r| r.decision.is_none())
            .cloned()
            .collect())
    }

    /// Get approval request by ID
    pub fn get_by_id(
        &self,
        approval_id: &str,
    ) -> Result<Option<ApprovalRequest>, ApprovalQueueError> {
        let queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        Ok(queue.iter().find(|r| r.approval_id == approval_id).cloned())
    }

    /// Resolve an approval request
    pub fn resolve(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        resolved_by: &str,
    ) -> Result<(), ApprovalQueueError> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        if let Some(request) = queue.iter_mut().find(|r| r.approval_id == approval_id) {
            request.decision = Some(decision);
            request.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            request.resolved_by = Some(resolved_by.to_string());
            Ok(())
        } else {
            Err(ApprovalQueueError::NotFound(approval_id.to_string()))
        }
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> Result<ApprovalQueueStats, ApprovalQueueError> {
        let queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        let total = queue.len();
        let pending = queue.iter().filter(|r| r.decision.is_none()).count();
        let approved = queue
            .iter()
            .filter(|r| matches!(r.decision, Some(ApprovalDecision::Approve)))
            .count();
        let rejected = queue
            .iter()
            .filter(|r| matches!(r.decision, Some(ApprovalDecision::Reject)))
            .count();

        Ok(ApprovalQueueStats {
            task_id: self.task_id.clone(),
            total,
            pending,
            approved,
            rejected,
        })
    }

    /// Clear resolved requests from the queue
    pub fn clear_resolved(&self) -> Result<usize, ApprovalQueueError> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|e| ApprovalQueueError::LockError(e.to_string()))?;

        let initial_len = queue.len();
        queue.retain(|r| r.decision.is_none());
        let cleared = initial_len - queue.len();

        Ok(cleared)
    }
}

// ============================================================================
// Statistics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalQueueStats {
    pub task_id: String,
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum ApprovalQueueError {
    LockError(String),
    NotFound(String),
    QueueFull,
}

impl std::fmt::Display for ApprovalQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalQueueError::LockError(msg) => write!(f, "Lock error: {}", msg),
            ApprovalQueueError::NotFound(id) => write!(f, "Approval request not found: {}", id),
            ApprovalQueueError::QueueFull => write!(f, "Approval queue is full"),
        }
    }
}

impl std::error::Error for ApprovalQueueError {}
