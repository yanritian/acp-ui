//! DAG Executor - Topological ordering and parallel execution
//!
//! Provides:
//! - Topological sort for dependency ordering
//! - Parallel execution of ready nodes
//! - Cycle detection

use std::collections::{HashMap, HashSet, VecDeque};

/// DAG node with execution context
pub trait DagNode {
    fn id(&self) -> &str;
    fn dependencies(&self) -> &[String];
}

/// DAG execution result
#[derive(Debug, Clone)]
pub struct DagResult<T> {
    /// Nodes that completed successfully
    pub completed: Vec<T>,
    /// Nodes that failed with error
    pub failed: Vec<(T, String)>,
    /// Nodes skipped due to dependency failure
    pub skipped: Vec<T>,
    /// Execution order (successful nodes)
    pub order: Vec<String>,
}

/// DAG executor for dependency-based execution
pub struct DagExecutor {
    /// Maximum parallel executions
    max_parallel: usize,
}

impl DagExecutor {
    pub fn new() -> Self {
        Self { max_parallel: 4 }
    }

    pub fn with_max_parallel(mut self, max: usize) -> Self {
        self.max_parallel = max;
        self
    }

    /// Validate DAG for cycles and missing dependencies
    pub fn validate<T: DagNode>(&self, nodes: &[T]) -> Result<(), String> {
        let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id()).collect();

        // Check for missing dependencies
        for node in nodes {
            for dep in node.dependencies() {
                if !node_ids.contains(dep.as_str()) {
                    return Err(format!(
                        "Node '{}' depends on non-existent node '{}'",
                        node.id(),
                        dep
                    ));
                }
            }
        }

        // Check for cycles using topological sort
        self.topological_order(nodes)?;

        Ok(())
    }

    /// Compute topological order (Kahn's algorithm)
    pub fn topological_order<T: DagNode>(&self, nodes: &[T]) -> Result<Vec<String>, String> {
        // Build adjacency map: node_id -> [dependents]
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize
        for node in nodes {
            dependents.entry(node.id().to_string()).or_default();
            in_degree.insert(node.id().to_string(), node.dependencies().len());
        }

        // Build reverse dependencies
        for node in nodes {
            for dep in node.dependencies() {
                dependents.entry(dep.clone()).or_default().push(node.id().to_string());
            }
        }

        // Start with nodes with no dependencies
        let mut queue: VecDeque<String> = VecDeque::new();
        for (id, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(id.clone());
            }
        }

        let mut order = Vec::new();

        while let Some(id) = queue.pop_front() {
            order.push(id.clone());

            if let Some(deps) = dependents.get(&id) {
                for dep_id in deps {
                    if let Some(deg) = in_degree.get_mut(dep_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        // Check for cycle
        if order.len() != nodes.len() {
            let cycle_nodes: Vec<&str> = nodes
                .iter()
                .filter(|n| !order.contains(&n.id().to_string()))
                .map(|n| n.id())
                .collect();
            return Err(format!(
                "Cycle detected: {} nodes not reachable ({})",
                cycle_nodes.len(),
                cycle_nodes.join(", ")
            ));
        }

        Ok(order)
    }

    /// Get nodes ready to execute (dependencies completed)
    pub fn ready_nodes<'a, T: DagNode>(&self, nodes: &'a [T], completed: &[&str]) -> Vec<&'a T> {
        nodes
            .iter()
            .filter(|n| {
                // All dependencies must be completed
                n.dependencies().iter().all(|dep| completed.contains(&dep.as_str()))
            })
            .collect()
    }

    /// Execute DAG with custom executor function
    pub fn execute<T: DagNode + Clone, F>(
        &self,
        nodes: Vec<T>,
        executor: F,
    ) -> DagResult<T>
    where
        F: Fn(&T) -> Result<String, String>,
    {
        let mut completed_ids: HashSet<String> = HashSet::new();
        let mut failed_ids: HashSet<String> = HashSet::new();
        let mut result = DagResult {
            completed: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            order: Vec::new(),
        };

        // Validate DAG
        if let Err(e) = self.validate(&nodes) {
            // All nodes failed due to invalid DAG
            for node in nodes {
                result.failed.push((node, e.clone()));
            }
            return result;
        }

        let mut pending: Vec<T> = nodes;

        while !pending.is_empty() {
            // Find ready nodes
            let completed_refs: Vec<&str> = completed_ids.iter().map(|s| s.as_str()).collect();
            let ready_ids: Vec<String> = pending
                .iter()
                .filter(|n| {
                    n.dependencies().iter().all(|dep| completed_refs.contains(&dep.as_str()))
                })
                .map(|n| n.id().to_string())
                .collect();

            if ready_ids.is_empty() {
                // No ready nodes but pending remains -> blocked by failed dependencies
                let blocked_ids: Vec<String> = pending
                    .iter()
                    .filter(|n| {
                        n.dependencies().iter().any(|dep| failed_ids.contains(dep))
                    })
                    .map(|n| n.id().to_string())
                    .collect();

                for id in &blocked_ids {
                    if let Some(node) = pending.iter().find(|n| n.id() == *id) {
                        result.skipped.push(node.clone());
                    }
                }
                pending.retain(|n| !blocked_ids.contains(&n.id().to_string()));

                if pending.is_empty() {
                    break;
                }

                // Still pending but not blocked -> must be cycle (should have been caught)
                for node in pending {
                    result.failed.push((node, "Unexpected blocking".into()));
                }
                break;
            }

            // Execute ready nodes (respect max_parallel)
            let batch_ids: Vec<String> = ready_ids.into_iter().take(self.max_parallel).collect();

            for id in batch_ids {
                // Find node in pending
                let node_idx = pending.iter().position(|n| n.id() == id);
                if let Some(idx) = node_idx {
                    let node = &pending[idx];
                    match executor(node) {
                        Ok(_output) => {
                            completed_ids.insert(node.id().to_string());
                            result.order.push(node.id().to_string());
                            result.completed.push(node.clone());
                            pending.remove(idx);
                        }
                        Err(e) => {
                            failed_ids.insert(node.id().to_string());
                            result.failed.push((node.clone(), e));
                            pending.remove(idx);
                        }
                    }
                }
            }
        }

        result
    }
}

impl Default for DagExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestNode {
        id: String,
        deps: Vec<String>,
    }

    impl TestNode {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                deps: Vec::new(),
            }
        }

        fn with_dep(mut self, dep: &str) -> Self {
            self.deps.push(dep.to_string());
            self
        }
    }

    impl DagNode for TestNode {
        fn id(&self) -> &str {
            &self.id
        }

        fn dependencies(&self) -> &[String] {
            &self.deps
        }
    }

    #[test]
    fn dag_executor_new() {
        let exec = DagExecutor::new();
        assert_eq!(exec.max_parallel, 4);
    }

    #[test]
    fn topological_order_simple() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a"),
            TestNode::new("b").with_dep("a"),
            TestNode::new("c").with_dep("b"),
        ];

        let order = exec.topological_order(&nodes).unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_order_parallel() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a"),
            TestNode::new("b"),
            TestNode::new("c").with_dep("a").with_dep("b"),
        ];

        let order = exec.topological_order(&nodes).unwrap();
        // a and b are both ready (no deps), c after both
        assert!(order.contains(&"a".to_string()));
        assert!(order.contains(&"b".to_string()));
        assert!(order.contains(&"c".to_string()));
        // c must come after a and b
        let a_idx = order.iter().position(|s| s == "a").unwrap();
        let b_idx = order.iter().position(|s| s == "b").unwrap();
        let c_idx = order.iter().position(|s| s == "c").unwrap();
        assert!(c_idx > a_idx);
        assert!(c_idx > b_idx);
    }

    #[test]
    fn detect_cycle() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a").with_dep("b"),
            TestNode::new("b").with_dep("a"),
        ];

        let result = exec.topological_order(&nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cycle"));
    }

    #[test]
    fn detect_missing_dependency() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a").with_dep("nonexistent"),
        ];

        let result = exec.validate(&nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non-existent"));
    }

    #[test]
    fn ready_nodes() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a"),
            TestNode::new("b").with_dep("a"),
            TestNode::new("c").with_dep("b"),
        ];

        // Initially 'a' is ready (no deps), 'b' and 'c' are not
        let ready = exec.ready_nodes(&nodes, &[]);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id(), "a");

        // After 'a' completed, both 'a' (still satisfies) and 'b' are "ready"
        // but 'a' is already completed, so we filter by pending nodes in practice
        let ready = exec.ready_nodes(&nodes, &["a"]);
        // In practice, caller should filter pending nodes before calling
        // This returns nodes whose dependencies are satisfied
        assert!(ready.len() >= 1);
        assert!(ready.iter().any(|n| n.id() == "b"));

        // After 'a' and 'b' completed, 'c' is ready
        let ready = exec.ready_nodes(&nodes, &["a", "b"]);
        assert!(ready.len() >= 1);
        assert!(ready.iter().any(|n| n.id() == "c"));
    }

    #[test]
    fn execute_simple() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a"),
            TestNode::new("b").with_dep("a"),
        ];

        let result = exec.execute(nodes, |n| Ok(format!("{} executed", n.id())));

        assert_eq!(result.completed.len(), 2);
        assert!(result.failed.is_empty());
        assert!(result.skipped.is_empty());
        assert_eq!(result.order, vec!["a", "b"]);
    }

    #[test]
    fn execute_with_failure() {
        let exec = DagExecutor::new();
        let nodes = vec![
            TestNode::new("a"),
            TestNode::new("b").with_dep("a"),
        ];

        let result = exec.execute(nodes, |n| {
            if n.id() == "a" {
                Err("a failed".into())
            } else {
                Ok("ok".into())
            }
        });

        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.skipped.len(), 1); // b skipped due to a failure
    }
}