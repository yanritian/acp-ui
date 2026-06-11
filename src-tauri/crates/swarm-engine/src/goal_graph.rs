//! GoalGraph - 多Goal协调的依赖DAG
//!
//! 根据RFC-001定义，GoalGraph管理Goal之间的依赖关系，
//! 提供 ready_goals()、has_blocked_goals()、all_converged() 等方法。

use crate::goal::{Goal, GoalStatus};
use std::collections::{HashMap, HashSet, VecDeque};

/// GoalGraph - Goal依赖图
#[derive(Debug, Clone)]
pub struct GoalGraph {
    /// Goal集合
    goals: HashMap<String, Goal>,
    /// 依赖关系：goal_id → [依赖的goal_id]
    dependencies: HashMap<String, Vec<String>>,
    /// 反向依赖：goal_id → [依赖我的goal_id]
    reverse_deps: HashMap<String, Vec<String>>,
}

impl GoalGraph {
    /// 创建空GoalGraph
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            dependencies: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// 添加Goal
    pub fn add_goal(&mut self, goal: Goal) {
        let goal_id = goal.id.clone();
        let depends_on = goal.depends_on.clone();

        // 存储Goal
        self.goals.insert(goal_id.clone(), goal);

        // 存储依赖关系
        self.dependencies.insert(goal_id.clone(), depends_on.clone());

        // 构建反向依赖
        for dep_id in depends_on {
            self.reverse_deps.entry(dep_id).or_default().push(goal_id.clone());
        }
    }

    /// 获取Goal
    pub fn get_goal(&self, goal_id: &str) -> Option<&Goal> {
        self.goals.get(goal_id)
    }

    /// 获取所有Goal
    pub fn all_goals(&self) -> Vec<&Goal> {
        self.goals.values().collect()
    }

    /// 获取当前可执行的Goal（无依赖或所有依赖已Converged）
    pub fn ready_goals(&self) -> Vec<&Goal> {
        let converged_ids: HashSet<&str> = self
            .goals
            .values()
            .filter(|g| g.status == GoalStatus::Converged)
            .map(|g| g.id.as_str())
            .collect();

        self.goals
            .values()
            .filter(|g| {
                g.status == GoalStatus::Pending
                    && g.depends_on.iter().all(|dep| converged_ids.contains(dep.as_str()))
            })
            .collect()
    }

    /// 检查是否有Goal失败导致下游阻塞
    pub fn has_blocked_goals(&self) -> Vec<(String, Vec<String>)> {
        let failed_ids: HashSet<&str> = self
            .goals
            .values()
            .filter(|g| g.status == GoalStatus::Failed || g.status == GoalStatus::Cancelled)
            .map(|g| g.id.as_str())
            .collect();

        self.goals
            .values()
            .filter(|g| g.status == GoalStatus::Pending)
            .filter_map(|g| {
                let blocking_deps: Vec<String> = g
                    .depends_on
                    .iter()
                    .filter(|dep| failed_ids.contains(dep.as_str()))
                    .cloned()
                    .collect();

                if !blocking_deps.is_empty() {
                    Some((g.id.clone(), blocking_deps))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 整个图是否全部Converged
    pub fn all_converged(&self) -> bool {
        self.goals
            .values()
            .all(|g| g.status.is_success())
    }

    /// 获取拓扑排序（按依赖顺序）
    pub fn topological_order(&self) -> Vec<String> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // 找到所有无依赖的Goal作为起点
        for (goal_id, deps) in &self.dependencies {
            if deps.is_empty() {
                queue.push_back(goal_id.clone());
            }
        }

        while let Some(goal_id) = queue.pop_front() {
            if visited.contains(&goal_id) {
                continue;
            }
            visited.insert(goal_id.clone());
            order.push(goal_id.clone());

            // 添加所有依赖此Goal且其他依赖都已访问的Goal
            if let Some(rev_deps) = self.reverse_deps.get(&goal_id) {
                for dep_id in rev_deps {
                    if visited.contains(dep_id) {
                        continue;
                    }
                    // 检查是否所有依赖都已访问
                    if let Some(all_deps) = self.dependencies.get(dep_id) {
                        if all_deps.iter().all(|d| visited.contains(d)) {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        // 添加剩余未访问的Goal（可能有循环依赖）
        for goal_id in self.goals.keys() {
            if !visited.contains(goal_id) {
                order.push(goal_id.clone());
            }
        }

        order
    }

    /// 更新Goal状态
    pub fn update_goal_status(&mut self, goal_id: &str, status: GoalStatus) {
        if let Some(goal) = self.goals.get_mut(goal_id) {
            goal.status = status;
        }
    }

    /// 获取Goal数量
    pub fn len(&self) -> usize {
        self.goals.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }
}

impl Default for GoalGraph {
    fn default() -> Self {
        Self::new()
    }
}