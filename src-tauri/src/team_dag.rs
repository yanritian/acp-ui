//! Team Orchestration - DAG execution engine with SyncPoints
//!
//! Implements parallel/sequential/hybrid execution strategies for multi-agent teams

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

/// Execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Parallel,    // Execute all agents simultaneously
    Sequential,  // Execute agents one by one
    Hybrid,      // Mix of parallel and sequential based on dependencies
}

impl ExecutionStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStrategy::Parallel => "parallel",
            ExecutionStrategy::Sequential => "sequential",
            ExecutionStrategy::Hybrid => "hybrid",
        }
    }
}

/// Team member definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub agent_id: String,
    pub template_id: String,
    pub role: String,           // "frontend", "backend", "qa", "lead"
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,  // IDs of agents this one depends on
}

/// Sync point for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPoint {
    pub id: String,
    pub name: String,
    pub wait_for: Vec<String>,   // Agent IDs that must complete first
    pub trigger_action: String,  // Action to take when sync point reached
    pub timeout_ms: u64,         // Maximum wait time
}

/// DAG node for execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGNode {
    pub id: String,
    pub member_id: String,
    pub dependencies: Vec<String>,
    pub status: NodeStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    WaitingSync,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Pending => "pending",
            NodeStatus::WaitingSync => "waiting_sync",
            NodeStatus::Running => "running",
            NodeStatus::Completed => "completed",
            NodeStatus::Failed => "failed",
            NodeStatus::Skipped => "skipped",
        }
    }
}

/// Team execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub team_id: String,
    pub strategy: ExecutionStrategy,
    pub nodes: Vec<DAGNode>,
    pub sync_points: Vec<SyncPoint>,
    pub current_sync_point: Option<String>,
    pub status: ExecutionStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Planning,
    PendingSync,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStatus::Planning => "planning",
            ExecutionStatus::PendingSync => "pending_sync",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Paused => "paused",
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Cancelled => "cancelled",
        }
    }
}

/// DAG execution engine
pub struct DAGEngine {
    plans: HashMap<String, ExecutionPlan>,
    completed_nodes: HashMap<String, HashSet<String>>,
}

impl DAGEngine {
    pub fn new() -> Self {
        Self {
            plans: HashMap::new(),
            completed_nodes: HashMap::new(),
        }
    }

    /// Create execution plan from team definition
    pub fn create_plan(
        &self,
        team_id: &str,
        members: Vec<TeamMember>,
        sync_points: Vec<SyncPoint>,
        strategy: ExecutionStrategy,
    ) -> Result<ExecutionPlan, String> {
        // Validate DAG (no circular dependencies)
        self.validate_dag(&members)?;

        // Build nodes from members
        let nodes: Vec<DAGNode> = members.iter().map(|m| {
            DAGNode {
                id: uuid::Uuid::new_v4().to_string(),
                member_id: m.agent_id.clone(),
                dependencies: m.dependencies.clone(),
                status: NodeStatus::Pending,
                started_at: None,
                completed_at: None,
                result: None,
            }
        }).collect();

        let plan = ExecutionPlan {
            id: uuid::Uuid::new_v4().to_string(),
            team_id: team_id.to_string(),
            strategy,
            nodes,
            sync_points,
            current_sync_point: None,
            status: ExecutionStatus::Planning,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        Ok(plan)
    }

    /// Validate DAG topology (no circular dependencies)
    fn validate_dag(&self, members: &[TeamMember]) -> Result<(), String> {
        // Build adjacency list
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for member in members {
            graph.insert(member.agent_id.clone(), member.dependencies.clone());
        }

        // DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for member in members {
            if self.has_cycle(&member.agent_id, &graph, &mut visited, &mut rec_stack) {
                return Err("Circular dependency detected in team DAG".to_string());
            }
        }

        Ok(())
    }

    /// DFS cycle detection
    fn has_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true; // Cycle found
        }

        if visited.contains(node) {
            return false; // Already processed
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if self.has_cycle(dep, graph, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Get nodes ready to execute (dependencies satisfied)
    pub fn get_ready_nodes<'a>(&self, plan: &'a ExecutionPlan) -> Vec<&'a DAGNode> {
        let completed: HashSet<&'a String> = plan.nodes.iter()
            .filter(|n| n.status == NodeStatus::Completed)
            .map(|n| &n.member_id)
            .collect();

        plan.nodes.iter()
            .filter(|n| {
                n.status == NodeStatus::Pending &&
                n.dependencies.iter().all(|d| completed.contains(d))
            })
            .collect()
    }

    /// Start executing a node
    pub fn start_node(&mut self, plan_id: &str, node_id: &str) -> Result<(), String> {
        let plan = self.plans.get_mut(plan_id)
            .ok_or_else(|| format!("Plan {} not found", plan_id))?;

        let node = plan.nodes.iter_mut()
            .find(|n| n.id == node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        node.status = NodeStatus::Running;
        node.started_at = Some(Utc::now());

        Ok(())
    }

    /// Complete a node
    pub fn complete_node(&mut self, plan_id: &str, node_id: &str, result: String) -> Result<(), String> {
        let plan = self.plans.get_mut(plan_id)
            .ok_or_else(|| format!("Plan {} not found", plan_id))?;

        let node = plan.nodes.iter_mut()
            .find(|n| n.id == node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        node.status = NodeStatus::Completed;
        node.completed_at = Some(Utc::now());
        node.result = Some(result);

        // Track completed nodes
        let completed = self.completed_nodes.entry(plan_id.to_string())
            .or_default();
        completed.insert(node.member_id.clone());

        // Check sync points inline
        Self::check_sync_points_inline(plan)?;

        Ok(())
    }

    /// Check sync points (static inline version)
    fn check_sync_points_inline(plan: &mut ExecutionPlan) -> Result<(), String> {
        // Find sync point to trigger
        let sync_point_id = plan.sync_points.iter()
            .find(|sync_point| {
                sync_point.wait_for.iter().all(|agent_id| {
                    plan.nodes.iter()
                        .any(|n| n.member_id == *agent_id && n.status == NodeStatus::Completed)
                })
            })
            .map(|s| s.id.clone());

        if let Some(id) = sync_point_id {
            plan.current_sync_point = Some(id.clone());
            plan.status = ExecutionStatus::PendingSync;

            // Auto-continue
            for node in &mut plan.nodes {
                if node.status == NodeStatus::WaitingSync {
                    node.status = NodeStatus::Pending;
                }
            }

            plan.current_sync_point = None;
            plan.status = ExecutionStatus::Running;
        }

        Ok(())
    }

    /// Check if all nodes are completed
    pub fn is_plan_complete(&self, plan: &ExecutionPlan) -> bool {
        plan.nodes.iter().all(|n|
            n.status == NodeStatus::Completed || n.status == NodeStatus::Skipped
        )
    }

    /// Get execution progress
    pub fn get_progress(&self, plan: &ExecutionPlan) -> f64 {
        let total = plan.nodes.len();
        let completed = plan.nodes.iter()
            .filter(|n| n.status == NodeStatus::Completed)
            .count();

        if total == 0 {
            0.0
        } else {
            completed as f64 / total as f64
        }
    }

    /// Register a plan
    pub fn register_plan(&mut self, plan: ExecutionPlan) {
        self.plans.insert(plan.id.clone(), plan);
    }

    /// Get plan by ID
    pub fn get_plan(&self, plan_id: &str) -> Option<&ExecutionPlan> {
        self.plans.get(plan_id)
    }
}

impl Default for DAGEngine {
    fn default() -> Self {
        Self::new()
    }
}