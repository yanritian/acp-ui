//! GoalGraph — dependency management for multiple Goals
//!
//! Manages a DAG of goals with dependencies. Provides:
//! - `ready_goals()` — goals whose dependencies are all Converged
//! - `all_converged()` — check if the entire graph is done
//! - `has_blocked_goals()` — detect deadlock (goals waiting on non-converging deps)
//! - `topological_order()` — linearize the graph for sequential execution

#![allow(dead_code)] // Some methods reserved for future graph queries

use crate::goal::{Goal, GoalStatus};
use std::collections::{HashMap, VecDeque};

/// A directed acyclic graph of Goals with dependency tracking.
pub struct GoalGraph {
    /// All goals in the graph, keyed by goal ID
    goals: HashMap<String, Goal>,
    /// Dependency map: goal_id → [dependency_ids]
    /// A goal can only execute when all its dependencies have converged.
    dependencies: HashMap<String, Vec<String>>,
    /// Reverse dependency map: goal_id → [dependent_ids]
    /// Used for efficient "what depends on me?" queries.
    dependents: HashMap<String, Vec<String>>,
}

impl GoalGraph {
    /// Create an empty goal graph.
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Add a goal to the graph.
    ///
    /// The goal's `dependencies` field is used to populate the dependency maps.
    /// Returns an error if a dependency references a non-existent goal.
    pub fn add_goal(&mut self, goal: Goal) -> Result<(), String> {
        let id = goal.id.clone();
        let deps = goal.depends_on.clone();

        // Validate dependencies exist (warn on forward references)
        for dep in &deps {
            if !self.goals.contains_key(dep) && *dep != id {
                eprintln!(
                    "[GoalGraph] Warning: goal '{}' has forward dependency on '{}' (not yet in graph)",
                    id, dep
                );
            }
        }

        // Update reverse dependency map
        for dep in &deps {
            self.dependents
                .entry(dep.clone())
                .or_default()
                .push(id.clone());
        }

        self.dependencies.insert(id.clone(), deps);
        self.goals.insert(id, goal);

        Ok(())
    }

    /// Get a reference to a goal by ID.
    pub fn get_goal(&self, id: &str) -> Option<&Goal> {
        self.goals.get(id)
    }

    /// Get a mutable reference to a goal by ID.
    pub fn get_goal_mut(&mut self, id: &str) -> Option<&mut Goal> {
        self.goals.get_mut(id)
    }

    /// Get all goals.
    pub fn all_goals(&self) -> Vec<&Goal> {
        self.goals.values().collect()
    }

    /// Get all goal IDs.
    pub fn goal_ids(&self) -> Vec<&str> {
        self.goals.keys().map(|s| s.as_str()).collect()
    }

    /// Get the dependencies of a goal.
    pub fn dependencies_of(&self, id: &str) -> Vec<&str> {
        self.dependencies
            .get(id)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the dependents of a goal (goals that depend on it).
    pub fn dependents_of(&self, id: &str) -> Vec<&str> {
        self.dependents
            .get(id)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Find all goals whose dependencies are all Converged (or have no dependencies).
    ///
    /// These are the goals ready for execution.
    pub fn ready_goals(&self) -> Vec<&Goal> {
        self.goals
            .values()
            .filter(|goal| {
                // Must be pending or iterating (not already active/converged/failed)
                match &goal.status {
                    GoalStatus::Pending | GoalStatus::Iterating { .. } => {}
                    _ => return false,
                }

                // All dependencies must be converged
                let deps = self.dependencies.get(&goal.id).cloned().unwrap_or_default();
                deps.iter().all(|dep_id| {
                    self.goals
                        .get(dep_id)
                        .map(|g| g.status == GoalStatus::Converged)
                        .unwrap_or(false)
                })
            })
            .collect()
    }

    /// Check if all goals in the graph have converged.
    pub fn all_converged(&self) -> bool {
        self.goals
            .values()
            .all(|g| g.status == GoalStatus::Converged)
    }

    /// Check if any goals are permanently blocked (waiting on a Failed/Cancelled dependency).
    ///
    /// Returns the IDs of blocked goals.
    pub fn blocked_goals(&self) -> Vec<String> {
        self.goals
            .values()
            .filter_map(|goal| {
                match &goal.status {
                    GoalStatus::Pending | GoalStatus::Iterating { .. } => {}
                    _ => return None,
                }

                let deps = self.dependencies.get(&goal.id).cloned().unwrap_or_default();
                let has_failed_dep = deps.iter().any(|dep_id| {
                    self.goals.get(dep_id).is_some_and(|g| {
                        matches!(
                            g.status,
                            GoalStatus::Failed { .. }
                                | GoalStatus::Cancelled
                                | GoalStatus::BudgetExhausted
                        )
                    })
                });

                if has_failed_dep {
                    Some(goal.id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Detect deadlocks: goals that are pending but can never be ready.
    ///
    /// A goal is deadlocked if:
    /// - It has unmet dependencies
    /// - None of its dependencies can make progress (they're all pending/active/etc.)
    pub fn has_blocked_goals(&self) -> bool {
        !self.blocked_goals().is_empty()
    }

    /// Return a topological ordering of all goals.
    ///
    /// Uses Kahn's algorithm. Returns an error if the graph contains a cycle.
    pub fn topological_order(&self) -> Result<Vec<String>, String> {
        // Calculate in-degree for each node
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for id in self.goals.keys() {
            in_degree.insert(id.clone(), 0);
        }
        // in_degree[g] = number of dependencies of g
        for (id, deps) in &self.dependencies {
            in_degree.insert(id.clone(), deps.len());
        }

        // Start with nodes that have no dependencies
        let mut queue: VecDeque<String> = VecDeque::new();
        for (id, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(id.clone());
            }
        }

        let mut order = Vec::new();

        while let Some(id) = queue.pop_front() {
            order.push(id.clone());

            // For each goal that depends on `id`, reduce its in-degree
            if let Some(dependents) = self.dependents.get(&id) {
                for dep_id in dependents {
                    if let Some(deg) = in_degree.get_mut(dep_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        if order.len() != self.goals.len() {
            return Err(format!(
                "Cycle detected in GoalGraph: ordered {} of {} goals",
                order.len(),
                self.goals.len()
            ));
        }

        Ok(order)
    }

    /// Get a summary of the graph's execution state.
    pub fn summary(&self) -> GoalGraphSummary {
        let mut summary = GoalGraphSummary {
            total: self.goals.len(),
            converged: 0,
            active: 0,
            pending: 0,
            failed: 0,
            iterating: 0,
            blocked: 0,
        };

        for goal in self.goals.values() {
            match &goal.status {
                GoalStatus::Converged => summary.converged += 1,
                GoalStatus::Active | GoalStatus::Evaluating => summary.active += 1,
                GoalStatus::Pending => summary.pending += 1,
                GoalStatus::Failed { .. } => summary.failed += 1,
                GoalStatus::Iterating { .. } => summary.iterating += 1,
                GoalStatus::BudgetExhausted
                | GoalStatus::Cancelled
                | GoalStatus::MaxIterReached => summary.blocked += 1,
            }
        }

        summary.blocked += self.blocked_goals().len();

        summary
    }

    /// Remove a goal from the graph (and clean up dependency references).
    pub fn remove_goal(&mut self, id: &str) -> Option<Goal> {
        if let Some(goal) = self.goals.remove(id) {
            // Save deps BEFORE removing the entry, so we can clean up reverse mappings
            let deps = self.dependencies.remove(id).unwrap_or_default();

            // Remove this goal from the dependents list of each of its dependencies
            for dep in &deps {
                if let Some(rev) = self.dependents.get_mut(dep) {
                    rev.retain(|r| r != id);
                }
            }

            // Remove its entry in the dependents map (no one depends on this goal anymore)
            self.dependents.remove(id);

            // Remove references in other goals' dependency lists
            for deps in self.dependencies.values_mut() {
                deps.retain(|d| d != id);
            }

            Some(goal)
        } else {
            None
        }
    }
}

impl Default for GoalGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for a GoalGraph.
#[derive(Debug, Clone)]
pub struct GoalGraphSummary {
    pub total: usize,
    pub converged: usize,
    pub active: usize,
    pub pending: usize,
    pub failed: usize,
    pub iterating: usize,
    pub blocked: usize,
}

impl std::fmt::Display for GoalGraphSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GoalGraph: {}/{} converged, {} active, {} pending, {} iterating, {} failed, {} blocked",
            self.converged, self.total, self.active, self.pending,
            self.iterating, self.failed, self.blocked
        )
    }
}
