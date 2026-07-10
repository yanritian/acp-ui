// Audit Logging Module
// Implements comprehensive audit trail for security and compliance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Audit Event Type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Authentication,
    Authorization,
    TaskCreated,
    TaskStarted,
    TaskPaused,
    TaskResumed,
    TaskStopped,
    TaskRedirected,
    TaskCompleted,
    TaskFailed,
    ApprovalRequested,
    ApprovalGranted,
    ApprovalRejected,
    FileModified,
    FileDeleted,
    UserLogin,
    UserLogout,
    UserCreated,
    UserDeleted,
    RoleAssigned,
    PermissionChanged,
    SystemConfig,
    SecurityViolation,
}

/// Audit Event Severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub user_id: Option<String>,
    pub task_id: Option<String>,
    pub project_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub outcome: AuditOutcome,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Audit Outcome
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        action: String,
        resource: String,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            id: format!("audit_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now(),
            event_type,
            severity,
            user_id: None,
            task_id: None,
            project_id: None,
            action,
            resource,
            outcome,
            details: None,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_project_id(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

/// Audit Log Configuration
#[derive(Debug, Clone)]
pub struct AuditLogConfig {
    pub max_events: usize,
    pub retention_days: u32,
    pub log_file: Option<PathBuf>,
    pub enable_console: bool,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            retention_days: 90,
            log_file: None,
            enable_console: true,
        }
    }
}

/// Audit Log
pub struct AuditLog {
    config: AuditLogConfig,
    events: VecDeque<AuditEvent>,
}

impl AuditLog {
    pub fn new(config: AuditLogConfig) -> Self {
        Self {
            config,
            events: VecDeque::new(),
        }
    }

    /// Log an audit event
    pub fn log(&mut self, event: AuditEvent) {
        if self.config.enable_console {
            println!(
                "[AUDIT] {} {:?} {:?} - {} {} ({:?})",
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                event.severity,
                event.event_type,
                event.action,
                event.resource,
                event.outcome
            );
        }

        if let Some(log_file) = &self.config.log_file {
            // TODO: Implement file logging
            let _ = log_file;
        }

        self.events.push_back(event);

        // Enforce max events limit
        while self.events.len() > self.config.max_events {
            self.events.pop_front();
        }
    }

    /// Get all events
    pub fn get_events(&self) -> Vec<&AuditEvent> {
        self.events.iter().collect()
    }

    /// Get events by user
    pub fn get_events_by_user(&self, user_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.user_id.as_deref() == Some(user_id))
            .collect()
    }

    /// Get events by task
    pub fn get_events_by_task(&self, task_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.task_id.as_deref() == Some(task_id))
            .collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &AuditEventType) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: &AuditSeverity) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| &e.severity == severity)
            .collect()
    }

    /// Get events by date range
    pub fn get_events_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Clear old events (retention policy)
    pub fn clear_old_events(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        self.events.retain(|e| e.timestamp >= cutoff);
    }

    /// Export audit log to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> AuditStatistics {
        let total = self.events.len();
        let by_severity = self.get_severity_counts();
        let by_type = self.get_event_type_counts();
        let by_outcome = self.get_outcome_counts();

        AuditStatistics {
            total_events: total,
            by_severity,
            by_type,
            by_outcome,
        }
    }

    fn get_severity_counts(&self) -> std::collections::HashMap<AuditSeverity, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in &self.events {
            *counts.entry(event.severity.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn get_event_type_counts(&self) -> std::collections::HashMap<AuditEventType, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in &self.events {
            *counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn get_outcome_counts(&self) -> std::collections::HashMap<AuditOutcome, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in &self.events {
            *counts.entry(event.outcome.clone()).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(AuditLogConfig::default())
    }
}

/// Audit Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub by_severity: std::collections::HashMap<AuditSeverity, usize>,
    pub by_type: std::collections::HashMap<AuditEventType, usize>,
    pub by_outcome: std::collections::HashMap<AuditOutcome, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::TaskCreated,
            AuditSeverity::Info,
            "create".to_string(),
            "task_001".to_string(),
            AuditOutcome::Success,
        );

        assert_eq!(event.event_type, AuditEventType::TaskCreated);
        assert_eq!(event.severity, AuditSeverity::Info);
        assert_eq!(event.outcome, AuditOutcome::Success);
    }

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new(AuditLogConfig::default());

        let event1 = AuditEvent::new(
            AuditEventType::TaskCreated,
            AuditSeverity::Info,
            "create".to_string(),
            "task_001".to_string(),
            AuditOutcome::Success,
        );

        let event2 = AuditEvent::new(
            AuditEventType::TaskFailed,
            AuditSeverity::Error,
            "execute".to_string(),
            "task_002".to_string(),
            AuditOutcome::Failure,
        );

        log.log(event1);
        log.log(event2);

        assert_eq!(log.get_events().len(), 2);
    }

    #[test]
    fn test_audit_log_filtering() {
        let mut log = AuditLog::new(AuditLogConfig::default());

        let event1 = AuditEvent::new(
            AuditEventType::TaskCreated,
            AuditSeverity::Info,
            "create".to_string(),
            "task_001".to_string(),
            AuditOutcome::Success,
        )
        .with_user_id("user_001".to_string());

        let event2 = AuditEvent::new(
            AuditEventType::TaskStarted,
            AuditSeverity::Info,
            "start".to_string(),
            "task_001".to_string(),
            AuditOutcome::Success,
        )
        .with_user_id("user_002".to_string());

        log.log(event1);
        log.log(event2);

        let user1_events = log.get_events_by_user("user_001");
        assert_eq!(user1_events.len(), 1);

        let task_events = log.get_events_by_task("task_001");
        assert_eq!(task_events.len(), 2);
    }

    #[test]
    fn test_audit_log_max_events() {
        let config = AuditLogConfig {
            max_events: 5,
            ..Default::default()
        };
        let mut log = AuditLog::new(config);

        for i in 0..10 {
            let event = AuditEvent::new(
                AuditEventType::TaskCreated,
                AuditSeverity::Info,
                "create".to_string(),
                format!("task_{}", i),
                AuditOutcome::Success,
            );
            log.log(event);
        }

        assert_eq!(log.get_events().len(), 5);
    }

    #[test]
    fn test_audit_statistics() {
        let mut log = AuditLog::new(AuditLogConfig::default());

        log.log(AuditEvent::new(
            AuditEventType::TaskCreated,
            AuditSeverity::Info,
            "create".to_string(),
            "task_001".to_string(),
            AuditOutcome::Success,
        ));

        log.log(AuditEvent::new(
            AuditEventType::TaskFailed,
            AuditSeverity::Error,
            "execute".to_string(),
            "task_002".to_string(),
            AuditOutcome::Failure,
        ));

        let stats = log.get_statistics();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.by_severity.get(&AuditSeverity::Info), Some(&1));
        assert_eq!(stats.by_severity.get(&AuditSeverity::Error), Some(&1));
    }
}