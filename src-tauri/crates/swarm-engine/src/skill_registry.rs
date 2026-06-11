//! Skill Registry - Worker能力声明与Skill路由
//!
//! 根据RFC-001定义的Worker Protocol扩展：
//! - Worker声明能力（Skills with proficiency scores）
//! - SkillRouter根据能力评分路由到最佳Worker

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDeclaration {
    /// Skill名称
    pub name: String,
    /// 熟练度评分（0.0-1.0）
    pub proficiency: f32,
    /// 平均执行时长（毫秒）
    pub avg_duration_ms: u64,
    /// 成功次数
    pub success_count: u32,
    /// 总执行次数
    pub total_count: u32,
}

impl SkillDeclaration {
    pub fn new(name: impl Into<String>, proficiency: f32) -> Self {
        Self {
            name: name.into(),
            proficiency,
            avg_duration_ms: 0,
            success_count: 0,
            total_count: 0,
        }
    }

    /// 计算有效熟练度（考虑历史成功率）
    pub fn effective_proficiency(&self) -> f32 {
        if self.total_count == 0 {
            self.proficiency
        } else {
            let success_rate = self.success_count as f32 / self.total_count as f32;
            // EWMA: 原始评分 * 0.3 + 实际成功率 * 0.7
            self.proficiency * 0.3 + success_rate * 0.7
        }
    }

    /// 记录执行结果
    pub fn record_execution(&mut self, success: bool, duration_ms: u64) {
        self.total_count += 1;
        if success {
            self.success_count += 1;
        }
        // 更新平均时长
        let total_duration = self.avg_duration_ms * (self.total_count - 1) as u64 + duration_ms;
        self.avg_duration_ms = total_duration / self.total_count as u64;
    }
}

/// Worker能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilitiesDeclaration {
    /// Worker ID
    pub worker_id: String,
    /// Worker类型
    pub worker_type: String,
    /// Skills列表
    pub skills: Vec<SkillDeclaration>,
    /// 最大并发数
    pub max_concurrency: u32,
    /// 最大上下文Token数
    pub max_context_tokens: u64,
    /// 当前负载
    pub current_load: u32,
}

impl WorkerCapabilitiesDeclaration {
    pub fn new(worker_id: impl Into<String>, worker_type: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            worker_type: worker_type.into(),
            skills: Vec::new(),
            max_concurrency: 2,
            max_context_tokens: 200_000,
            current_load: 0,
        }
    }

    /// 添加Skill
    pub fn add_skill(&mut self, skill: SkillDeclaration) {
        self.skills.push(skill);
    }

    /// 检查是否可用
    pub fn is_available(&self) -> bool {
        self.current_load < self.max_concurrency
    }

    /// 获取特定Skill的熟练度
    pub fn get_skill_proficiency(&self, skill_name: &str) -> Option<f32> {
        self.skills
            .iter()
            .find(|s| s.name == skill_name)
            .map(|s| s.effective_proficiency())
    }
}

/// SkillRouter - Skill路由器
pub struct SkillRouter {
    /// Worker能力注册表
    workers: HashMap<String, WorkerCapabilitiesDeclaration>,
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    /// 注册Worker能力
    pub fn register(&mut self, capabilities: WorkerCapabilitiesDeclaration) {
        self.workers.insert(capabilities.worker_id.clone(), capabilities);
    }

    /// 取消注册Worker
    pub fn unregister(&mut self, worker_id: &str) {
        self.workers.remove(worker_id);
    }

    /// 获取Worker能力
    pub fn get_worker(&self, worker_id: &str) -> Option<&WorkerCapabilitiesDeclaration> {
        self.workers.get(worker_id)
    }

    /// 获取所有Worker
    pub fn list_workers(&self) -> Vec<&WorkerCapabilitiesDeclaration> {
        self.workers.values().collect()
    }

    /// 路由Skill到最佳Worker
    ///
    /// 选择标准：
    /// 1. Worker必须可用（current_load < max_concurrency）
    /// 2. Worker必须声明了该Skill
    /// 3. 选择熟练度最高的Worker
    pub fn route(&self, skill_name: &str) -> Option<String> {
        let mut best_worker: Option<(String, f32)> = None;

        for worker in self.workers.values() {
            // 检查是否可用
            if !worker.is_available() {
                continue;
            }

            // 检查是否有该Skill
            let proficiency = worker.get_skill_proficiency(skill_name);
            if let Some(p) = proficiency {
                // 比较熟练度
                match best_worker {
                    None => best_worker = Some((worker.worker_id.clone(), p)),
                    Some((_, best_p)) if p > best_p => {
                        best_worker = Some((worker.worker_id.clone(), p));
                    }
                    _ => {}
                }
            }
        }

        best_worker.map(|(id, _)| id)
    }

    /// 批量路由多个Skill
    pub fn route_batch(&self, skill_names: &[String]) -> HashMap<String, Option<String>> {
        skill_names
            .iter()
            .map(|name| (name.clone(), self.route(name)))
            .collect()
    }

    /// 更新Worker负载
    pub fn update_load(&mut self, worker_id: &str, load: u32) {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.current_load = load;
        }
    }

    /// 记录执行结果（用于自动校准熟练度）
    pub fn record_execution(&mut self, worker_id: &str, skill_name: &str, success: bool, duration_ms: u64) {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            if let Some(skill) = worker.skills.iter_mut().find(|s| s.name == skill_name) {
                skill.record_execution(success, duration_ms);
            }
        }
    }
}

impl Default for SkillRouter {
    fn default() -> Self {
        Self::new()
    }
}