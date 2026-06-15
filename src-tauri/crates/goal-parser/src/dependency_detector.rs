//! Dependency Detector - Detect task dependencies from natural language

use acp_core::GoalSpec;

/// Dependency detector for goal ordering
pub struct DependencyDetector;

impl DependencyDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect dependencies between goals based on description patterns
    pub fn detect_dependencies(&self, goals: &mut Vec<GoalSpec>) {
        // First collect all dependency mappings
        let mut deps_to_add: Vec<(usize, String)> = Vec::new();

        for i in 0..goals.len() {
            let goal = &goals[i];
            let desc_lower = goal.description.to_lowercase();

            // Check for "after task N" patterns
            if desc_lower.contains("after task") {
                // Extract task number
                let parts: Vec<&str> = goal.description.split_whitespace().collect();
                for (j, part) in parts.iter().enumerate() {
                    if *part == "task" && j + 1 < parts.len() {
                        if let Ok(task_num) = parts[j + 1].parse::<usize>() {
                            if task_num > 0 && task_num <= goals.len() && task_num - 1 != i {
                                deps_to_add.push((i, goals[task_num - 1].id.clone()));
                            }
                        }
                    }
                }
            }

            // Check for sequential dependency (implicit)
            if i > 0 {
                let prev_desc = goals[i - 1].description.to_lowercase();
                if self.is_sequential(&prev_desc, &desc_lower) {
                    deps_to_add.push((i, goals[i - 1].id.clone()));
                }
            }
        }

        // Now apply all dependencies
        for (idx, dep_id) in deps_to_add {
            goals[idx].depends_on.push(dep_id);
        }
    }

    /// Check if task2 is sequential continuation of task1
    fn is_sequential(&self, task1_lower: &str, task2_lower: &str) -> bool {
        // Keywords indicating sequence
        let sequential_keywords = ["then", "next", "after that", "following"];

        for keyword in &sequential_keywords {
            if task2_lower.contains(keyword) {
                return true;
            }
        }

        // Check for phase/stage indicators
        let phase1 = self.extract_phase(task1_lower);
        let phase2 = self.extract_phase(task2_lower);

        if let (Some(p1), Some(p2)) = (phase1, phase2) {
            return p2 == p1 + 1;
        }

        false
    }

    /// Extract phase number from description
    fn extract_phase(&self, desc_lower: &str) -> Option<u32> {
        let parts: Vec<&str> = desc_lower.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "phase" && i + 1 < parts.len() {
                if let Ok(num) = parts[i + 1].parse::<u32>() {
                    return Some(num);
                }
            }
        }
        None
    }

    /// Get all dependencies for a goal
    pub fn get_dependencies<'a>(&self, goal: &GoalSpec, all_goals: &'a [GoalSpec]) -> Vec<&'a GoalSpec> {
        all_goals
            .iter()
            .filter(|g| goal.depends_on.contains(&g.id))
            .collect()
    }

    /// Get goals that depend on this goal
    pub fn get_dependents<'a>(&self, goal: &GoalSpec, all_goals: &'a [GoalSpec]) -> Vec<&'a GoalSpec> {
        all_goals
            .iter()
            .filter(|g| g.depends_on.contains(&goal.id))
            .collect()
    }
}

impl Default for DependencyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acp_core::CompletionConditionSpec;

    fn make_goals(descriptions: &[&str]) -> Vec<GoalSpec> {
        descriptions
            .iter()
            .enumerate()
            .map(|(i, desc)| GoalSpec {
                id: format!("goal-{}", i),
                description: desc.to_string(),
                completion_condition: CompletionConditionSpec::CommandSuccess {
                    command: "echo".to_string(),
                },
                evaluator: "auto".to_string(),
                executor: None,
                depends_on: vec![],
                token_budget: 50_000,
                max_iterations: 5,
            })
            .collect()
    }

    #[test]
    fn detector_new() {
        let detector = DependencyDetector::new();
        assert!(true); // Detector exists
    }

    #[test]
    fn detector_no_dependencies() {
        let mut goals = make_goals(&["Task A", "Task B"]);
        let detector = DependencyDetector::new();
        detector.detect_dependencies(&mut goals);

        assert!(goals[0].depends_on.is_empty());
        assert!(goals[1].depends_on.is_empty());
    }

    #[test]
    fn detector_sequential_then() {
        let mut goals = make_goals(&["Setup project", "Then add tests"]);
        let detector = DependencyDetector::new();
        detector.detect_dependencies(&mut goals);

        assert!(goals[1].depends_on.contains(&goals[0].id));
    }

    #[test]
    fn detector_phase_sequence() {
        let mut goals = make_goals(&["Phase 1 Setup", "Phase 2 Develop"]);
        let detector = DependencyDetector::new();
        detector.detect_dependencies(&mut goals);

        assert!(goals[1].depends_on.contains(&goals[0].id));
    }

    #[test]
    fn detector_get_dependencies() {
        let goals = make_goals(&["A", "B"]);
        let goal_a_id = goals[0].id.clone();
        let mut goals = goals;
        goals[1].depends_on.push(goal_a_id);

        let detector = DependencyDetector::new();
        let deps = detector.get_dependencies(&goals[1], &goals);

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].id, goals[0].id);
    }

    #[test]
    fn detector_get_dependents() {
        let goals = make_goals(&["A", "B", "C"]);
        let goal_a_id = goals[0].id.clone();
        let mut goals = goals;
        goals[1].depends_on.push(goal_a_id.clone());
        goals[2].depends_on.push(goal_a_id);

        let detector = DependencyDetector::new();
        let dependents = detector.get_dependents(&goals[0], &goals);

        assert_eq!(dependents.len(), 2);
    }
}