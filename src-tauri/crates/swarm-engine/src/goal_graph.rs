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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::CompletionCondition;

    fn make_goal(id: &str, depends_on: Vec<String>) -> Goal {
        let mut goal = Goal::new(id, "Test goal", CompletionCondition::command_success("echo"), "worker-1");
        goal.depends_on = depends_on;
        goal
    }

    #[test]
    fn empty_graph() {
        let graph = GoalGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert!(graph.ready_goals().is_empty());
        assert!(graph.all_converged()); // vacuous truth
        assert!(graph.topological_order().is_empty());
    }

    #[test]
    fn single_goal_no_deps() {
        let mut graph = GoalGraph::new();
        graph.add_goal(make_goal("goal-a", vec![]));

        assert!(!graph.is_empty());
        assert_eq!(graph.len(), 1);

        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-a");
    }

    #[test]
    fn dependency_chain() {
        let mut graph = GoalGraph::new();
        // B depends on A
        graph.add_goal(make_goal("goal-a", vec![]));
        graph.add_goal(make_goal("goal-b", vec!["goal-a".to_string()]));

        // Initially only A is ready
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-a");

        // Mark A as converged
        graph.update_goal_status("goal-a", GoalStatus::Converged);

        // Now B is ready
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-b");

        // Topological order: A before B
        let order = graph.topological_order();
        assert_eq!(order[0], "goal-a");
        assert_eq!(order[1], "goal-b");
    }

    #[test]
    fn blocked_by_failed_dependency() {
        let mut graph = GoalGraph::new();
        graph.add_goal(make_goal("goal-a", vec![]));
        graph.add_goal(make_goal("goal-b", vec!["goal-a".to_string()]));

        // Mark A as failed
        graph.update_goal_status("goal-a", GoalStatus::Failed);

        // B should be blocked
        let blocked = graph.has_blocked_goals();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].0, "goal-b");
        assert!(blocked[0].1.contains(&"goal-a".to_string()));
    }

    #[test]
    fn all_converged() {
        let mut graph = GoalGraph::new();
        graph.add_goal(make_goal("goal-a", vec![]));
        graph.add_goal(make_goal("goal-b", vec!["goal-a".to_string()]));

        assert!(!graph.all_converged());

        graph.update_goal_status("goal-a", GoalStatus::Converged);
        assert!(!graph.all_converged());

        graph.update_goal_status("goal-b", GoalStatus::Converged);
        assert!(graph.all_converged());
    }

    #[test]
    fn parallel_goals() {
        let mut graph = GoalGraph::new();
        // A and B both have no dependencies
        graph.add_goal(make_goal("goal-a", vec![]));
        graph.add_goal(make_goal("goal-b", vec![]));

        // Both should be ready
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn complex_dependency_graph() {
        let mut graph = GoalGraph::new();
        // D depends on B and C, B and C depend on A
        graph.add_goal(make_goal("goal-a", vec![]));
        graph.add_goal(make_goal("goal-b", vec!["goal-a".to_string()]));
        graph.add_goal(make_goal("goal-c", vec!["goal-a".to_string()]));
        graph.add_goal(make_goal("goal-d", vec!["goal-b".to_string(), "goal-c".to_string()]));

        // Only A ready initially
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-a");

        // Converge A -> B and C ready
        graph.update_goal_status("goal-a", GoalStatus::Converged);
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 2);

        // Converge B -> only C ready (D needs both B and C)
        graph.update_goal_status("goal-b", GoalStatus::Converged);
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-c");

        // Converge C -> D ready
        graph.update_goal_status("goal-c", GoalStatus::Converged);
        let ready = graph.ready_goals();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "goal-d");
    }
}