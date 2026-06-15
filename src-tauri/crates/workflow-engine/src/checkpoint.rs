//! Checkpoint Manager - Save/Restore/Rollback workflow state
//!
//! Provides:
//! - Save checkpoint at any stage
//! - Restore from checkpoint
//! - Rollback to specific stage
//! - Auto-save at stage completion

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialize(String),

    #[error("Checkpoint not found: {0}")]
    NotFound(String),

    #[error("Invalid checkpoint: {0}")]
    Invalid(String),
}

/// Checkpoint data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint unique ID
    pub id: String,
    /// Workflow ID this checkpoint belongs to
    pub workflow_id: String,
    /// Stage ID at which checkpoint was taken
    pub stage_id: String,
    /// Stage index
    pub stage_index: usize,
    /// Workflow status at checkpoint time
    pub workflow_status: String,
    /// Stage statuses at checkpoint time
    pub stage_statuses: HashMap<String, String>,
    /// Stage outputs at checkpoint time
    pub stage_outputs: HashMap<String, Option<String>>,
    /// Timestamp (UNIX ms)
    pub timestamp: u64,
    /// Checkpoint metadata
    pub metadata: HashMap<String, String>,
    /// Checkpoint label (for human reference)
    pub label: Option<String>,
}

impl Checkpoint {
    pub fn new(workflow_id: &str, stage_id: &str, stage_index: usize) -> Self {
        Self {
            id: format!("cp-{}-{}-{}", workflow_id, stage_id, now_ms()),
            workflow_id: workflow_id.to_string(),
            stage_id: stage_id.to_string(),
            stage_index,
            workflow_status: "Running".to_string(),
            stage_statuses: HashMap::new(),
            stage_outputs: HashMap::new(),
            timestamp: now_ms(),
            metadata: HashMap::new(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Checkpoint manager for a workflow
pub struct CheckpointManager {
    /// Base directory for checkpoint storage
    base_dir: PathBuf,
    /// In-memory cache of checkpoints
    cache: HashMap<String, Checkpoint>,
    /// Auto-save enabled
    auto_save: bool,
    /// Maximum checkpoints to keep (older ones deleted)
    max_checkpoints: usize,
}

impl CheckpointManager {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
            auto_save: true,
            max_checkpoints: 10,
        }
    }

    pub fn with_auto_save(mut self, enabled: bool) -> Self {
        self.auto_save = enabled;
        self
    }

    pub fn with_max_checkpoints(mut self, max: usize) -> Self {
        self.max_checkpoints = max;
        self
    }

    /// Ensure checkpoint directory exists
    fn ensure_dir(&self) -> Result<PathBuf, CheckpointError> {
        let dir = self.base_dir.join("checkpoints");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Save checkpoint to disk
    fn save_to_disk(&self, checkpoint: &Checkpoint) -> Result<(), CheckpointError> {
        let dir = self.ensure_dir()?;
        let path = dir.join(format!("{}.json", checkpoint.id));

        let json = serde_json::to_string_pretty(checkpoint)
            .map_err(|e| CheckpointError::Serialize(e.to_string()))?;

        let mut file = fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Load checkpoint from disk
    fn load_from_disk(&self, id: &str) -> Result<Checkpoint, CheckpointError> {
        let dir = self.ensure_dir()?;
        let path = dir.join(format!("{}.json", id));

        if !path.exists() {
            return Err(CheckpointError::NotFound(id.to_string()));
        }

        let mut file = fs::File::open(&path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;

        let checkpoint: Checkpoint = serde_json::from_str(&json)
            .map_err(|e| CheckpointError::Invalid(e.to_string()))?;

        Ok(checkpoint)
    }

    /// Create and save a checkpoint
    pub fn save(&mut self, checkpoint: Checkpoint) -> Result<String, CheckpointError> {
        let id = checkpoint.id.clone();
        let workflow_id = checkpoint.workflow_id.clone();

        // Save to disk
        if self.auto_save {
            self.save_to_disk(&checkpoint)?;
        }

        // Cache in memory
        self.cache.insert(id.clone(), checkpoint);

        // Cleanup old checkpoints
        self.cleanup_old_checkpoints(&workflow_id)?;

        Ok(id)
    }

    /// Load checkpoint by ID
    pub fn load(&self, id: &str) -> Result<Checkpoint, CheckpointError> {
        // Try cache first
        if let Some(cp) = self.cache.get(id) {
            return Ok(cp.clone());
        }

        // Load from disk
        self.load_from_disk(id)
    }

    /// Get all checkpoints for a workflow
    pub fn list(&self, workflow_id: &str) -> Result<Vec<Checkpoint>, CheckpointError> {
        let dir = self.ensure_dir()?;
        let mut checkpoints = Vec::new();

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let mut file = fs::File::open(&path)?;
                let mut json = String::new();
                file.read_to_string(&mut json)?;

                if let Ok(cp) = serde_json::from_str::<Checkpoint>(&json) {
                    if cp.workflow_id == workflow_id {
                        checkpoints.push(cp);
                    }
                }
            }
        }

        // Sort by timestamp (oldest first)
        checkpoints.sort_by_key(|cp| cp.timestamp);

        Ok(checkpoints)
    }

    /// Get latest checkpoint for a workflow
    pub fn latest(&self, workflow_id: &str) -> Result<Option<Checkpoint>, CheckpointError> {
        let checkpoints = self.list(workflow_id)?;
        Ok(checkpoints.into_iter().last())
    }

    /// Delete a checkpoint
    pub fn delete(&mut self, id: &str) -> Result<(), CheckpointError> {
        // Remove from cache
        self.cache.remove(id);

        // Remove from disk
        let dir = self.ensure_dir()?;
        let path = dir.join(format!("{}.json", id));

        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }

    /// Delete all checkpoints for a workflow
    pub fn delete_all(&mut self, workflow_id: &str) -> Result<usize, CheckpointError> {
        let checkpoints = self.list(workflow_id)?;
        let count = checkpoints.len();

        for cp in checkpoints {
            self.delete(&cp.id)?;
        }

        Ok(count)
    }

    /// Rollback to a specific checkpoint (return stage index)
    pub fn rollback(&self, id: &str) -> Result<usize, CheckpointError> {
        let checkpoint = self.load(id)?;
        Ok(checkpoint.stage_index)
    }

    /// Rollback to latest checkpoint for a workflow
    pub fn rollback_latest(&self, workflow_id: &str) -> Result<Option<usize>, CheckpointError> {
        match self.latest(workflow_id)? {
            Some(cp) => Ok(Some(cp.stage_index)),
            None => Ok(None),
        }
    }

    /// Cleanup old checkpoints (keep only max_checkpoints most recent)
    fn cleanup_old_checkpoints(&mut self, workflow_id: &str) -> Result<(), CheckpointError> {
        let checkpoints = self.list(workflow_id)?;

        if checkpoints.len() > self.max_checkpoints {
            // Delete oldest checkpoints
            let to_delete = checkpoints.len() - self.max_checkpoints;
            for cp in checkpoints.into_iter().take(to_delete) {
                self.delete(&cp.id)?;
            }
        }

        Ok(())
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn checkpoint_new() {
        let cp = Checkpoint::new("wf-001", "stage-2", 1);
        assert_eq!(cp.workflow_id, "wf-001");
        assert_eq!(cp.stage_id, "stage-2");
        assert_eq!(cp.stage_index, 1);
    }

    #[test]
    fn checkpoint_with_label() {
        let cp = Checkpoint::new("wf-001", "stage-2", 1)
            .with_label("After data validation");

        assert_eq!(cp.label, Some("After data validation".to_string()));
    }

    #[test]
    fn checkpoint_manager_save_load() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir);

        let cp = Checkpoint::new("wf-001", "stage-2", 1)
            .with_label("Test checkpoint");

        let id = manager.save(cp).unwrap();

        // Load from cache
        let loaded = manager.load(&id).unwrap();
        assert_eq!(loaded.label, Some("Test checkpoint".to_string()));

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn checkpoint_manager_list() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-list-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir);

        manager.save(Checkpoint::new("wf-001", "stage-1", 0)).unwrap();
        // Sleep briefly to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.save(Checkpoint::new("wf-001", "stage-2", 1)).unwrap();

        let list = manager.list("wf-001").unwrap();
        assert_eq!(list.len(), 2);

        // Sorted by timestamp
        assert_eq!(list[0].stage_index, 0);
        assert_eq!(list[1].stage_index, 1);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn checkpoint_manager_latest() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-latest-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir);

        manager.save(Checkpoint::new("wf-001", "stage-1", 0)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.save(Checkpoint::new("wf-001", "stage-2", 1)).unwrap();

        let latest = manager.latest("wf-001").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().stage_index, 1);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn checkpoint_manager_delete() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-delete-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir);

        let id = manager.save(Checkpoint::new("wf-001", "stage-1", 0)).unwrap();
        assert!(manager.load(&id).is_ok());

        manager.delete(&id).unwrap();
        assert!(manager.load(&id).is_err());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn checkpoint_manager_rollback() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-rollback-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir);

        let id = manager.save(Checkpoint::new("wf-001", "stage-3", 2)).unwrap();

        let stage_index = manager.rollback(&id).unwrap();
        assert_eq!(stage_index, 2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn checkpoint_manager_max_checkpoints() {
        let temp_dir = env::temp_dir().join("acp-checkpoint-max-test");
        fs::create_dir_all(&temp_dir).ok();

        let mut manager = CheckpointManager::new(&temp_dir)
            .with_max_checkpoints(2);

        // Create 4 checkpoints
        for i in 0..4 {
            let stage_id = format!("stage-{}", i);
            manager.save(Checkpoint::new("wf-001", &stage_id, i)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Should only keep 2 most recent
        let list = manager.list("wf-001").unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].stage_index, 2); // stage-2 and stage-3 kept
        assert_eq!(list[1].stage_index, 3);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}