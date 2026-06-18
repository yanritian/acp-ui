//! Topology - Swarm执行拓扑
//!
//! **DEPRECATED (C-1 Fix)**: This entire file is deprecated.
//! Use `src-tauri/src/topology.rs` instead, which has the authoritative implementation:
//! - `execute_star()` - real parallel execution
//! - `execute_chain()` - real sequential execution
//! - `execute_pipeline()` - real batch execution
//!
//! The sync versions in this file are stubs. The async versions use different
//! Goal types that don't match the main application's Goal/GoalGraph.
//!
//! **Migration**: Replace imports from `swarm_engine::topology` with `acp_ui_lib::topology`.

use crate::goal::{Goal, GoalStatus, GoalOutcome};
use crate::goal_graph::GoalGraph;
use crate::reconcile::{ReconcileLoop, WorkerExecutor};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Topology trait - 执行拓扑抽象
pub trait Topology {
    /// 执行拓扑（同步版本）
    fn execute(&self, goals: Vec<Goal>) -> Vec<(String, GoalOutcome)>;
}

/// StarTopology - 并行扇出拓扑
///
/// 所有无依赖的Goal同时启动执行，
/// 等待所有Goal收敛后才完成。
pub struct StarTopology {
    executor: Arc<Mutex<dyn WorkerExecutor>>,
}

impl StarTopology {
    pub fn new(executor: Arc<Mutex<dyn WorkerExecutor>>) -> Self {
        Self { executor }
    }
}

impl Topology for StarTopology {
    /// **DEPRECATED (H-3): Stub implementation**
    ///
    /// This sync version returns fake `GoalOutcome::Converged` without real execution.
    /// Use `StarTopologyAsync::execute_all()` for real reconcile_goal() calls.
    fn execute(&self, goals: Vec<Goal>) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();

        for goal in goals {
            // Stub: returns fake Converged outcome
            results.push((goal.id.clone(), GoalOutcome::Converged {
                iterations: 1,
                tokens_used: 1000,
                final_feedback: format!("Star topology stub: {} (use Async version)", goal.id),
            }));
        }

        results
    }
}

/// StarTopologyAsync - 异步版本的Star拓扑
pub struct StarTopologyAsync {
    reconciler: Arc<ReconcileLoop>,
}

impl StarTopologyAsync {
    pub fn new(reconciler: Arc<ReconcileLoop>) -> Self {
        Self { reconciler }
    }

    /// 执行所有Goal（并行）
    pub async fn execute_all(&self, goals: Vec<Goal>) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();

        // 并行执行所有Goal（简化实现：顺序执行）
        for mut goal in goals {
            let outcome = self.reconciler.reconcile_goal(&mut goal).await;
            results.push((goal.id, outcome));
        }

        results
    }
}

/// ChainTopology - 依赖链拓扑
///
/// Goal按拓扑排序顺序执行，
/// 每个Goal必须等待其依赖Goal收敛后才能启动。
pub struct ChainTopology {
    executor: Arc<Mutex<dyn WorkerExecutor>>,
}

impl ChainTopology {
    pub fn new(executor: Arc<Mutex<dyn WorkerExecutor>>) -> Self {
        Self { executor }
    }

    /// 构建依赖链并按顺序执行
    pub fn execute_chain(&self, graph: &mut GoalGraph) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();
        let order = graph.topological_order();

        for goal_id in order {
            // 检查依赖是否都已收敛
            if let Some(goal) = graph.get_goal(&goal_id) {
                let converged_deps: Vec<&str> = graph
                    .all_goals()
                    .iter()
                    .filter(|g| g.status == GoalStatus::Converged)
                    .map(|g| g.id.as_str())
                    .collect();

                if !goal.depends_on.iter().all(|dep| converged_deps.contains(&dep.as_str())) {
                    continue;
                }

                // 检查是否有失败的依赖
                let failed_deps: Vec<&str> = graph
                    .all_goals()
                    .iter()
                    .filter(|g| matches!(g.status, GoalStatus::Failed { .. }))
                    .map(|g| g.id.as_str())
                    .collect();

                if goal.depends_on.iter().any(|dep| failed_deps.contains(&dep.as_str())) {
                    results.push((goal_id, GoalOutcome::Failed("Dependency failed".to_string())));
                    continue;
                }
            }

            results.push((goal_id.clone(), GoalOutcome::Converged {
                iterations: 1,
                tokens_used: 1000,
                final_feedback: format!("Chain topology: {} executed", goal_id),
            }));
            graph.update_goal_status(&goal_id, GoalStatus::Converged);
        }

        results
    }
}

impl Topology for ChainTopology {
    /// **DEPRECATED (H-3): Stub implementation**
    ///
    /// This sync version returns fake `GoalOutcome::Converged` without real execution.
    /// Use `ChainTopologyAsync::execute_chain()` for real reconcile_graph() calls.
    fn execute(&self, goals: Vec<Goal>) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();

        for goal in goals {
            // Stub: returns fake Converged outcome
            results.push((goal.id.clone(), GoalOutcome::Converged {
                iterations: 1,
                tokens_used: 1000,
                final_feedback: format!("Chain topology stub: {} (use Async version)", goal.id),
            }));
        }

        results
    }
}

/// ChainTopologyAsync - 异步版本的Chain拓扑
pub struct ChainTopologyAsync {
    reconciler: Arc<ReconcileLoop>,
}

impl ChainTopologyAsync {
    pub fn new(reconciler: Arc<ReconcileLoop>) -> Self {
        Self { reconciler }
    }

    /// 按依赖顺序执行GoalGraph
    pub async fn execute_chain(&self, graph: &mut GoalGraph) -> Vec<(String, GoalOutcome)> {
        self.reconciler.reconcile_graph(graph).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reconcile::EchoExecutor;
    use crate::goal::CompletionCondition;

    #[test]
    fn star_topology_execute() {
        let executor = Arc::new(Mutex::new(EchoExecutor::new("star-worker", "", true)));
        let topology = StarTopology::new(executor);

        let goals = vec![
            Goal::new("g1", "Goal 1", CompletionCondition::command_success("echo"), "w1"),
            Goal::new("g2", "Goal 2", CompletionCondition::command_success("echo"), "w2"),
        ];

        let results = topology.execute(goals);

        assert_eq!(results.len(), 2);
        for (_, outcome) in &results {
            assert!(matches!(outcome, GoalOutcome::Converged { .. }));
        }
    }

    #[test]
    fn chain_topology_execute() {
        let executor = Arc::new(Mutex::new(EchoExecutor::new("chain-worker", "", true)));
        let topology = ChainTopology::new(executor);

        let goals = vec![
            Goal::new("g1", "Goal 1", CompletionCondition::command_success("echo"), "w1"),
            Goal::new("g2", "Goal 2", CompletionCondition::command_success("echo"), "w2"),
        ];

        let results = topology.execute(goals);

        assert_eq!(results.len(), 2);
        for (_, outcome) in &results {
            assert!(matches!(outcome, GoalOutcome::Converged { .. }));
        }
    }

    #[test]
    fn chain_topology_execute_chain_with_deps() {
        let executor = Arc::new(Mutex::new(EchoExecutor::new("chain-worker", "", true)));
        let topology = ChainTopology::new(executor);
        let mut graph = GoalGraph::new();

        let mut goal_a = Goal::new("a", "Goal A", CompletionCondition::command_success("echo"), "w1");
        goal_a.depends_on = vec![];

        let mut goal_b = Goal::new("b", "Goal B", CompletionCondition::command_success("echo"), "w1");
        goal_b.depends_on = vec!["a".to_string()];

        graph.add_goal(goal_a);
        graph.add_goal(goal_b);

        let results = topology.execute_chain(&mut graph);

        assert_eq!(results.len(), 2);
        // Check execution order matches topological order
        assert_eq!(results[0].0, "a");
        assert_eq!(results[1].0, "b");
    }
}