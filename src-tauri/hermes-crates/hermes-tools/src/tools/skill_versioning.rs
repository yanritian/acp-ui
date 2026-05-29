//! Skill Versioning Module
//!
//! Manages version history for skills, supporting:
//! - Snapshot creation on each evolution
//! - Version listing
//! - Rollback to any previous version

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single version snapshot of a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersionSnapshot {
    /// Semantic version (e.g., "1.0.3")
    pub semver: String,
    /// Full skill content at this version
    pub content: String,
    /// When this version was created
    pub evolved_at: String,
    /// Why this version was created (evolution reason or "initial")
    pub reason: String,
    /// Generation number (incremented on each evolution)
    pub generation: u32,
    /// Content hash for deduplication
    pub content_hash: String,
}

/// Version history for a single skill
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillVersionHistory {
    /// Skill name
    pub skill_name: String,
    /// Ordered list of versions (oldest first)
    pub versions: Vec<SkillVersionSnapshot>,
}

/// In-memory version store (production would persist to SQLite/filesystem)
#[derive(Debug, Default)]
pub struct VersionStore {
    histories: HashMap<String, SkillVersionHistory>,
}

impl VersionStore {
    pub fn new() -> Self {
        Self {
            histories: HashMap::new(),
        }
    }

    /// Record a new version snapshot
    pub fn record_version(
        &mut self,
        skill_name: &str,
        semver: &str,
        content: &str,
        reason: &str,
        generation: u32,
    ) -> &SkillVersionSnapshot {
        let history = self
            .histories
            .entry(skill_name.to_string())
            .or_insert_with(|| SkillVersionHistory {
                skill_name: skill_name.to_string(),
                versions: Vec::new(),
            });

        let snapshot = SkillVersionSnapshot {
            semver: semver.to_string(),
            content: content.to_string(),
            evolved_at: Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            generation,
            content_hash: compute_hash(content),
        };

        history.versions.push(snapshot);
        history.versions.last().unwrap()
    }

    /// List all versions for a skill
    pub fn list_versions(&self, skill_name: &str) -> Option<&SkillVersionHistory> {
        self.histories.get(skill_name)
    }

    /// Get a specific version by semver
    pub fn get_version(
        &self,
        skill_name: &str,
        semver: &str,
    ) -> Option<&SkillVersionSnapshot> {
        self.histories
            .get(skill_name)
            .and_then(|h| h.versions.iter().find(|v| v.semver == semver))
    }

    /// Get the latest version
    pub fn get_latest(&self, skill_name: &str) -> Option<&SkillVersionSnapshot> {
        self.histories
            .get(skill_name)
            .and_then(|h| h.versions.last())
    }

    /// Rollback to a specific version, returning the content to restore
    pub fn rollback(
        &mut self,
        skill_name: &str,
        target_semver: &str,
    ) -> Result<SkillVersionSnapshot, VersionError> {
        let history = self
            .histories
            .get(skill_name)
            .ok_or_else(|| VersionError::SkillNotFound(skill_name.to_string()))?;

        let target = history
            .versions
            .iter()
            .find(|v| v.semver == target_semver)
            .ok_or_else(|| {
                VersionError::VersionNotFound {
                    skill: skill_name.to_string(),
                    version: target_semver.to_string(),
                }
            })?
            .clone();

        // Record the rollback as a new version
        let rollback_reason = format!("rollback to v{}", target_semver);
        let new_semver = bump_patch_version(
            history
                .versions
                .last()
                .map(|v| v.semver.as_str())
                .unwrap_or("1.0.0"),
        );
        let generation = target.generation + 1;

        self.record_version(
            skill_name,
            &new_semver,
            &target.content,
            &rollback_reason,
            generation,
        );

        Ok(target)
    }

    /// Get total version count for a skill
    pub fn version_count(&self, skill_name: &str) -> usize {
        self.histories
            .get(skill_name)
            .map(|h| h.versions.len())
            .unwrap_or(0)
    }

    /// List all tracked skill names
    pub fn tracked_skills(&self) -> Vec<&str> {
        self.histories.keys().map(|s| s.as_str()).collect()
    }
}

/// Version operation errors
#[derive(Debug, Clone)]
pub enum VersionError {
    SkillNotFound(String),
    VersionNotFound { skill: String, version: String },
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::SkillNotFound(name) => {
                write!(f, "No version history for skill '{}'", name)
            }
            VersionError::VersionNotFound { skill, version } => {
                write!(f, "Version '{}' not found for skill '{}'", version, skill)
            }
        }
    }
}

impl std::error::Error for VersionError {}

/// Bump patch version (1.0.0 -> 1.0.1)
fn bump_patch_version(semver: &str) -> String {
    let parts: Vec<&str> = semver.split('.').collect();
    if parts.len() == 3 {
        let patch = parts[2].parse::<u32>().unwrap_or(0) + 1;
        format!("{}.{}.{}", parts[0], parts[1], patch)
    } else {
        "1.0.1".to_string()
    }
}

/// Compute content hash
fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_list_versions() {
        let mut store = VersionStore::new();

        store.record_version("web-research", "1.0.0", "# Web Research\nv1", "initial", 0);
        store.record_version("web-research", "1.0.1", "# Web Research\nv2", "evolution", 1);

        let history = store.list_versions("web-research").unwrap();
        assert_eq!(history.versions.len(), 2);
        assert_eq!(history.versions[0].semver, "1.0.0");
        assert_eq!(history.versions[1].semver, "1.0.1");
    }

    #[test]
    fn test_get_latest() {
        let mut store = VersionStore::new();
        store.record_version("test-skill", "1.0.0", "content v1", "initial", 0);
        store.record_version("test-skill", "1.0.1", "content v2", "evolution", 1);

        let latest = store.get_latest("test-skill").unwrap();
        assert_eq!(latest.semver, "1.0.1");
        assert_eq!(latest.content, "content v2");
    }

    #[test]
    fn test_rollback() {
        let mut store = VersionStore::new();
        store.record_version("my-skill", "1.0.0", "original content", "initial", 0);
        store.record_version("my-skill", "1.0.1", "evolved content", "high failure", 1);
        store.record_version("my-skill", "1.0.2", "more evolved", "low rating", 2);

        // Rollback to 1.0.0
        let result = store.rollback("my-skill", "1.0.0");
        assert!(result.is_ok());
        let restored = result.unwrap();
        assert_eq!(restored.content, "original content");

        // A new version should have been recorded
        assert_eq!(store.version_count("my-skill"), 4);
        let latest = store.get_latest("my-skill").unwrap();
        assert_eq!(latest.content, "original content");
        assert!(latest.reason.contains("rollback"));
    }

    #[test]
    fn test_rollback_nonexistent_version() {
        let mut store = VersionStore::new();
        store.record_version("my-skill", "1.0.0", "content", "initial", 0);

        let result = store.rollback("my-skill", "9.9.9");
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_nonexistent_skill() {
        let mut store = VersionStore::new();
        let result = store.rollback("no-such-skill", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_count() {
        let mut store = VersionStore::new();
        assert_eq!(store.version_count("test"), 0);

        store.record_version("test", "1.0.0", "v1", "initial", 0);
        assert_eq!(store.version_count("test"), 1);

        store.record_version("test", "1.0.1", "v2", "evolution", 1);
        assert_eq!(store.version_count("test"), 2);
    }

    #[test]
    fn test_tracked_skills() {
        let mut store = VersionStore::new();
        store.record_version("skill-a", "1.0.0", "a", "initial", 0);
        store.record_version("skill-b", "1.0.0", "b", "initial", 0);

        let skills = store.tracked_skills();
        assert_eq!(skills.len(), 2);
        assert!(skills.contains(&"skill-a"));
        assert!(skills.contains(&"skill-b"));
    }

    #[test]
    fn test_get_specific_version() {
        let mut store = VersionStore::new();
        store.record_version("test", "1.0.0", "content v1", "initial", 0);
        store.record_version("test", "1.0.1", "content v2", "evolution", 1);

        let v1 = store.get_version("test", "1.0.0").unwrap();
        assert_eq!(v1.content, "content v1");

        assert!(store.get_version("test", "2.0.0").is_none());
    }

    #[test]
    fn test_bump_patch_version() {
        assert_eq!(bump_patch_version("1.0.0"), "1.0.1");
        assert_eq!(bump_patch_version("2.3.9"), "2.3.10");
        assert_eq!(bump_patch_version("invalid"), "1.0.1");
    }

    #[test]
    fn test_content_hash_dedup() {
        let mut store = VersionStore::new();
        store.record_version("test", "1.0.0", "same content", "initial", 0);
        store.record_version("test", "1.0.1", "same content", "evolution", 1);

        let h1 = &store.get_version("test", "1.0.0").unwrap().content_hash;
        let h2 = &store.get_version("test", "1.0.1").unwrap().content_hash;
        assert_eq!(h1, h2);
    }
}
