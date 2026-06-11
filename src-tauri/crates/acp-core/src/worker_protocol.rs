//! Worker Protocol - Agent registration and communication

use serde::{Deserialize, Serialize};

/// Worker capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// Worker ID
    pub worker_id: String,
    /// Worker type (e.g., "claude-code", "codex")
    pub worker_type: String,
    /// Worker version
    pub version: String,
    /// Skills with proficiency scores
    pub skills: Vec<SkillDeclaration>,
    /// Maximum concurrent tasks
    pub max_concurrency: u32,
    /// Maximum context tokens
    pub max_context_tokens: u64,
}

/// Skill declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDeclaration {
    pub name: String,
    pub proficiency: f32,
    pub avg_duration_ms: u64,
}

/// Worker status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Disconnected,
    Error,
}

/// Worker heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub status: WorkerStatus,
    pub current_load: u32,
    pub active_goals: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_capabilities_new() {
        let caps = WorkerCapabilities {
            worker_id: "worker-001".to_string(),
            worker_type: "codex".to_string(),
            version: "1.0.0".to_string(),
            skills: vec![],
            max_concurrency: 2,
            max_context_tokens: 200_000,
        };

        assert_eq!(caps.worker_id, "worker-001");
        assert_eq!(caps.worker_type, "codex");
    }

    #[test]
    fn worker_capabilities_serde_roundtrip() {
        let caps = WorkerCapabilities {
            worker_id: "worker-001".to_string(),
            worker_type: "claude_code".to_string(),
            version: "2.0.0".to_string(),
            skills: vec![
                SkillDeclaration {
                    name: "code-generation".to_string(),
                    proficiency: 0.9,
                    avg_duration_ms: 30000,
                },
            ],
            max_concurrency: 3,
            max_context_tokens: 100_000,
        };

        let json = serde_json::to_string(&caps).unwrap();
        let decoded: WorkerCapabilities = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.worker_id, caps.worker_id);
        assert_eq!(decoded.skills.len(), 1);
        assert_eq!(decoded.skills[0].name, "code-generation");
    }

    #[test]
    fn worker_status_serde() {
        let status = WorkerStatus::Busy;

        let json = serde_json::to_string(&status).unwrap();
        let decoded: WorkerStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, WorkerStatus::Busy);
    }

    #[test]
    fn worker_status_all_variants() {
        // Test all status variants serialize correctly
        let statuses = vec![WorkerStatus::Idle, WorkerStatus::Busy, WorkerStatus::Disconnected, WorkerStatus::Error];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let decoded: WorkerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, status);
        }
    }

    #[test]
    fn worker_heartbeat_serde_roundtrip() {
        let heartbeat = WorkerHeartbeat {
            worker_id: "worker-001".to_string(),
            status: WorkerStatus::Busy,
            current_load: 2,
            active_goals: vec!["goal-001".to_string(), "goal-002".to_string()],
        };

        let json = serde_json::to_string(&heartbeat).unwrap();
        let decoded: WorkerHeartbeat = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.worker_id, heartbeat.worker_id);
        assert_eq!(decoded.status, WorkerStatus::Busy);
        assert_eq!(decoded.active_goals.len(), 2);
    }
}