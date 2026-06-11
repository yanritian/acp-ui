//! Queen Lease - Queen选举与租约机制
//!
//! 根据RFC-001定义的Queen Lease机制：
//! - 30秒租约 + 10秒续约
//! - 租约过期后自动选举新Queen
//! - 防脑裂：Worker只接受租约有效的Queen指令

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Worker ID类型
pub type WorkerId = String;

/// Queen Lease - 租约机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueenLease {
    /// Queen Worker ID
    pub queen_id: WorkerId,
    /// 租约授予时间（毫秒时间戳）
    pub granted_at: u64,
    /// 租约TTL（秒）
    pub ttl_seconds: u64,
    /// 续约间隔（秒）
    pub renew_interval: u64,
}

impl QueenLease {
    /// 创建新租约
    pub fn new(queen_id: impl Into<WorkerId>) -> Self {
        Self {
            queen_id: queen_id.into(),
            granted_at: Utc::now().timestamp_millis() as u64,
            ttl_seconds: 30,
            renew_interval: 10,
        }
    }

    /// 获取租约过期时间（毫秒时间戳）
    pub fn expires_at_ms(&self) -> u64 {
        self.granted_at + self.ttl_seconds * 1000
    }

    /// 检查租约是否有效
    pub fn check_validity(&self) -> bool {
        let now_ms = Utc::now().timestamp_millis() as u64;
        now_ms < self.expires_at_ms()
    }

    /// 续约租约
    pub fn renew(&mut self) -> bool {
        if !self.check_validity() {
            return false;
        }
        self.granted_at = Utc::now().timestamp_millis() as u64;
        true
    }
}

/// Queen Election Manager - Queen选举管理器
pub struct QueenElectionManager {
    /// 当前租约
    current_lease: Option<QueenLease>,
    /// 所有Worker列表
    workers: HashMap<WorkerId, WorkerInfo>,
    /// 选举候选人优先级（基于能力评分）
    priority_scores: HashMap<WorkerId, f32>,
}

/// Worker信息
#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub worker_id: WorkerId,
    pub worker_type: String,
    pub last_heartbeat: u64,
    pub is_alive: bool,
}

impl QueenElectionManager {
    /// 创建选举管理器
    pub fn new() -> Self {
        Self {
            current_lease: None,
            workers: HashMap::new(),
            priority_scores: HashMap::new(),
        }
    }

    /// 注册Worker
    pub fn register_worker(&mut self, worker_id: WorkerId, worker_type: String) {
        let now_ms = Utc::now().timestamp_millis() as u64;
        self.workers.insert(worker_id.clone(), WorkerInfo {
            worker_id: worker_id.clone(),
            worker_type,
            last_heartbeat: now_ms,
            is_alive: true,
        });
        // 默认优先级
        self.priority_scores.insert(worker_id, 1.0);
    }

    /// 更新Worker心跳
    pub fn update_heartbeat(&mut self, worker_id: &str) {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.last_heartbeat = Utc::now().timestamp_millis() as u64;
            worker.is_alive = true;
        }
    }

    /// 检查Worker存活状态
    pub fn check_workers_alive(&mut self, timeout_ms: u64) {
        let now_ms = Utc::now().timestamp_millis() as u64;
        for worker in self.workers.values_mut() {
            if now_ms - worker.last_heartbeat > timeout_ms {
                worker.is_alive = false;
            }
        }
    }

    /// 获取当前Queen
    pub fn get_current_queen(&self) -> Option<&QueenLease> {
        self.current_lease.as_ref()
    }

    /// 检查当前Queen是否有效
    pub fn is_queen_valid(&self) -> bool {
        if let Some(lease) = &self.current_lease {
            lease.check_validity()
        } else {
            false
        }
    }

    /// 选举新Queen
    pub fn elect_new_queen(&mut self) -> Option<QueenLease> {
        // 选择存活且优先级最高的Worker作为Queen
        let best_worker = self
            .workers
            .iter()
            .filter(|(_, info)| info.is_alive)
            .max_by(|(id1, _), (id2, _)| {
                let score1 = self.priority_scores.get(id1.as_str()).unwrap_or(&0.0);
                let score2 = self.priority_scores.get(id2.as_str()).unwrap_or(&0.0);
                score1.partial_cmp(score2).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((worker_id, _)) = best_worker {
            let lease = QueenLease::new(worker_id.clone());
            self.current_lease = Some(lease.clone());
            Some(lease)
        } else {
            None
        }
    }

    /// 续约当前Queen租约
    pub fn renew_queen_lease(&mut self) -> bool {
        if let Some(lease) = &mut self.current_lease {
            lease.renew()
        } else {
            false
        }
    }

    /// 检查并自动选举（如果租约过期）
    pub fn auto_elect_if_expired(&mut self) -> Option<QueenLease> {
        if !self.is_queen_valid() {
            self.elect_new_queen()
        } else {
            None
        }
    }

    /// 更新Worker优先级评分
    pub fn update_priority(&mut self, worker_id: &str, score: f32) {
        self.priority_scores.insert(worker_id.to_string(), score);
    }
}

impl Default for QueenElectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// 有效
    Valid,
    /// 租约过期
    LeaseExpired,
    /// 非当前Queen
    NotCurrentQueen {
        current_queen: WorkerId,
        sender: WorkerId,
    },
    /// 无Queen
    NoQueen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queen_lease_new_is_valid() {
        let lease = QueenLease::new("queen-001");
        assert!(lease.check_validity());
    }

    #[test]
    fn queen_lease_expires_at() {
        let lease = QueenLease::new("queen-001");
        assert!(lease.expires_at_ms() > lease.granted_at);
        assert_eq!(lease.ttl_seconds, 30);
    }

    #[test]
    fn queen_lease_renew() {
        let mut lease = QueenLease::new("queen-001");
        let original_granted = lease.granted_at;

        // Should be able to renew while valid
        assert!(lease.renew());

        // granted_at should be updated
        assert!(lease.granted_at >= original_granted);
    }

    #[test]
    fn election_manager_register_and_elect() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());

        let lease = manager.elect_new_queen();
        assert!(lease.is_some());
        assert_eq!(lease.unwrap().queen_id, "worker-001");
    }

    #[test]
    fn election_manager_no_workers() {
        let mut manager = QueenElectionManager::new();
        let lease = manager.elect_new_queen();
        assert!(lease.is_none());
    }

    #[test]
    fn election_manager_priority_election() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());
        manager.register_worker("worker-002".to_string(), "claude_code".to_string());

        // Set priorities
        manager.update_priority("worker-001", 0.5);
        manager.update_priority("worker-002", 0.9);

        // Higher priority worker should be elected
        let lease = manager.elect_new_queen();
        assert!(lease.is_some());
        assert_eq!(lease.unwrap().queen_id, "worker-002");
    }

    #[test]
    fn election_manager_auto_elect() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());

        // No current lease -> auto_elect should trigger election
        let lease = manager.auto_elect_if_expired();
        assert!(lease.is_some());
    }

    #[test]
    fn election_manager_is_queen_valid() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());

        // Initially no queen
        assert!(!manager.is_queen_valid());

        // After election
        manager.elect_new_queen();
        assert!(manager.is_queen_valid());
    }

    #[test]
    fn election_manager_renew_lease() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());
        manager.elect_new_queen();

        // Should be able to renew
        assert!(manager.renew_queen_lease());
    }

    #[test]
    fn election_manager_update_heartbeat() {
        let mut manager = QueenElectionManager::new();
        manager.register_worker("worker-001".to_string(), "codex".to_string());

        // Get current heartbeat
        let initial = manager.workers.get("worker-001").unwrap().last_heartbeat;

        // Update heartbeat
        manager.update_heartbeat("worker-001");

        // Should be updated
        let updated = manager.workers.get("worker-001").unwrap().last_heartbeat;
        assert!(updated >= initial);
    }
}