//! Event Topics - Standard topic names and matching utilities

use std::collections::HashSet;

/// Standard event topics for ACP-Swarm
pub struct EventTopic;

impl EventTopic {
    /// Goal events prefix
    pub const GOAL: &'static str = "goal";

    /// Goal created event
    pub const GOAL_CREATED: &'static str = "goal.created";

    /// Goal status changed event
    pub const GOAL_STATUS_CHANGED: &'static str = "goal.status";

    /// Goal converged event
    pub const GOAL_CONVERGED: &'static str = "goal.converged";

    /// Goal failed event
    pub const GOAL_FAILED: &'static str = "goal.failed";

    /// Worker events prefix
    pub const WORKER: &'static str = "worker";

    /// Worker registered event
    pub const WORKER_REGISTERED: &'static str = "worker.registered";

    /// Worker health update event
    pub const WORKER_HEALTH_UPDATE: &'static str = "worker.health";

    /// Worker unregistered event
    pub const WORKER_UNREGISTERED: &'static str = "worker.unregistered";

    /// Topology events prefix
    pub const TOPOLOGY: &'static str = "topology";

    /// Topology changed event
    pub const TOPOLOGY_CHANGED: &'static str = "topology.changed";

    /// Queen events prefix
    pub const QUEEN: &'static str = "queen";

    /// Queen elected event
    pub const QUEEN_ELECTED: &'static str = "queen.elected";

    /// Queen lease renewed event
    pub const QUEEN_LEASE_RENEWED: &'static str = "queen.lease";

    /// Swarm events prefix
    pub const SWARM: &'static str = "swarm";

    /// All events wildcard
    pub const ALL: &'static str = "*";

    /// Get all goal-related topics
    pub fn goal_topics() -> Vec<&'static str> {
        vec![
            Self::GOAL_CREATED,
            Self::GOAL_STATUS_CHANGED,
            Self::GOAL_CONVERGED,
            Self::GOAL_FAILED,
        ]
    }

    /// Get all worker-related topics
    pub fn worker_topics() -> Vec<&'static str> {
        vec![
            Self::WORKER_REGISTERED,
            Self::WORKER_HEALTH_UPDATE,
            Self::WORKER_UNREGISTERED,
        ]
    }

    /// Get all topology-related topics
    pub fn topology_topics() -> Vec<&'static str> {
        vec![Self::TOPOLOGY_CHANGED]
    }

    /// Get all queen-related topics
    pub fn queen_topics() -> Vec<&'static str> {
        vec![Self::QUEEN_ELECTED, Self::QUEEN_LEASE_RENEWED]
    }

    /// Get all standard topics
    pub fn all_topics() -> Vec<&'static str> {
        let mut topics = Vec::new();
        topics.extend(Self::goal_topics());
        topics.extend(Self::worker_topics());
        topics.extend(Self::topology_topics());
        topics.extend(Self::queen_topics());
        topics
    }
}

/// Topic matcher for wildcard support
pub struct TopicMatcher;

impl TopicMatcher {
    /// Check if a topic matches a pattern
    ///
    /// Patterns:
    /// - Exact match: "goal.created" matches "goal.created"
    /// - Prefix wildcard: "goal.*" matches "goal.created", "goal.status", etc.
    /// - Multi-level wildcard: "goal.**" matches "goal.created", "goal.status.changed"
    pub fn matches(pattern: &str, topic: &str) -> bool {
        // Exact match
        if pattern == topic {
            return true;
        }

        // Single-level wildcard (*)
        if pattern.ends_with('*') && !pattern.ends_with("**") {
            let prefix = &pattern[..pattern.len() - 1];
            return topic.starts_with(prefix) && topic[prefix.len()..].split('.').count() == 1;
        }

        // Multi-level wildcard (**)
        if pattern.ends_with("**") {
            let prefix = &pattern[..pattern.len() - 2];
            return topic.starts_with(prefix);
        }

        false
    }

    /// Get all matching topics from a set
    pub fn matching_topics<'a>(pattern: &str, topics: &'a HashSet<&'a str>) -> Vec<&'a str> {
        topics
            .iter()
            .filter(|topic| Self::matches(pattern, topic))
            .cloned()
            .collect()
    }

    /// Parse a pattern into components
    pub fn parse_pattern(pattern: &str) -> Vec<String> {
        pattern.split('.').map(|s| s.to_string()).collect()
    }

    /// Build a pattern from components
    pub fn build_pattern(components: &[&str]) -> String {
        components.join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_topic_constants() {
        assert_eq!(EventTopic::GOAL_CREATED, "goal.created");
        assert_eq!(EventTopic::WORKER_REGISTERED, "worker.registered");
        assert_eq!(EventTopic::TOPOLOGY_CHANGED, "topology.changed");
        assert_eq!(EventTopic::QUEEN_ELECTED, "queen.elected");
    }

    #[test]
    fn event_topic_collections() {
        let goal_topics = EventTopic::goal_topics();
        assert!(goal_topics.contains(&"goal.created"));

        let worker_topics = EventTopic::worker_topics();
        assert!(worker_topics.contains(&"worker.registered"));
    }

    #[test]
    fn topic_matcher_exact() {
        assert!(TopicMatcher::matches("goal.created", "goal.created"));
        assert!(!TopicMatcher::matches("goal.created", "goal.status"));
    }

    #[test]
    fn topic_matcher_single_wildcard() {
        assert!(TopicMatcher::matches("goal.*", "goal.created"));
        assert!(TopicMatcher::matches("goal.*", "goal.status"));
        assert!(!TopicMatcher::matches("goal.*", "goal.status.changed"));
    }

    #[test]
    fn topic_matcher_multi_wildcard() {
        assert!(TopicMatcher::matches("goal.**", "goal.created"));
        assert!(TopicMatcher::matches("goal.**", "goal.status.changed"));
        assert!(TopicMatcher::matches("**", "goal.created"));
        assert!(TopicMatcher::matches("**", "worker.registered"));
    }

    #[test]
    fn topic_matcher_matching_topics() {
        let topics: HashSet<&str> = [
            "goal.created",
            "goal.status",
            "worker.registered",
        ].into();

        let matching = TopicMatcher::matching_topics("goal.*", &topics);
        assert_eq!(matching.len(), 2);
        assert!(matching.contains(&"goal.created"));
        assert!(matching.contains(&"goal.status"));
    }

    #[test]
    fn topic_matcher_parse_pattern() {
        let components = TopicMatcher::parse_pattern("goal.created.test");
        assert_eq!(components, vec!["goal", "created", "test"]);
    }

    #[test]
    fn topic_matcher_build_pattern() {
        let pattern = TopicMatcher::build_pattern(&["goal", "created"]);
        assert_eq!(pattern, "goal.created");
    }
}