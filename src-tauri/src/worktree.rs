//! Git Worktree Isolation - 为每个 Goal 创建独立工作区
//!
//! 功能：
//! - 创建临时 Git worktree
//! - 隔离文件修改，避免冲突
//! - 执行完成后清理 worktree

#![allow(dead_code)] // Reserved for future worktree isolation feature

use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("Failed to create worktree: {0}")]
    CreateFailed(String),

    #[error("Failed to remove worktree: {0}")]
    RemoveFailed(String),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Worktree path invalid: {0}")]
    InvalidPath(String),
}

/// Git Worktree 管理器
pub struct WorktreeManager {
    /// 主仓库路径
    repo_path: PathBuf,
    /// worktree 基础路径
    worktree_base: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_path: impl Into<PathBuf>, worktree_base: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            worktree_base: worktree_base.into(),
        }
    }

    /// 为 Goal 创建独立 worktree
    pub fn create_worktree(&self, goal_id: &str) -> Result<PathBuf, WorktreeError> {
        let worktree_path = self.worktree_base.join(goal_id);

        // 确保 worktree 基础目录存在
        if !self.worktree_base.exists() {
            std::fs::create_dir_all(&self.worktree_base)
                .map_err(|e| WorktreeError::InvalidPath(e.to_string()))?;
        }

        // 创建 worktree
        let output = Command::new("git")
            .args(["worktree", "add", &worktree_path.to_string_lossy(), "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| WorktreeError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 如果 worktree 已存在，直接返回路径
            if stderr.contains("already exists") || worktree_path.exists() {
                return Ok(worktree_path);
            }
            return Err(WorktreeError::CreateFailed(stderr.to_string()));
        }

        Ok(worktree_path)
    }

    /// 移除 worktree
    pub fn remove_worktree(&self, goal_id: &str) -> Result<(), WorktreeError> {
        let worktree_path = self.worktree_base.join(goal_id);

        // 使用 git worktree remove
        let output = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                &worktree_path.to_string_lossy(),
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| WorktreeError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 如果不存在，忽略
            if stderr.contains("not a working tree") {
                return Ok(());
            }
            return Err(WorktreeError::RemoveFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// 列出所有 worktree
    pub fn list_worktrees(&self) -> Result<Vec<String>, WorktreeError> {
        let output = Command::new("git")
            .args(["worktree", "list"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| WorktreeError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(WorktreeError::GitCommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let worktrees: Vec<String> = stdout
            .lines()
            .skip(1) // 跳过主仓库行
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.first().map(|s| s.to_string())
            })
            .collect();

        Ok(worktrees)
    }

    /// 清理所有 goal worktree
    pub fn cleanup_all(&self) -> Result<u32, WorktreeError> {
        let worktrees = self.list_worktrees()?;
        let mut cleaned = 0;

        let base_path_str = self.worktree_base.to_string_lossy().to_string();

        for wt_path in worktrees {
            // 只清理 worktree_base 下的 worktree
            if wt_path.starts_with(&base_path_str) {
                // 提取 goal_id
                let goal_id = wt_path
                    .strip_prefix(&base_path_str)
                    .map(|s| s.trim_start_matches('/').trim_start_matches('\\'))
                    .unwrap_or("");

                if !goal_id.is_empty() {
                    self.remove_worktree(goal_id)?;
                    cleaned += 1;
                }
            }
        }

        Ok(cleaned)
    }

    /// 获取 worktree 路径
    pub fn get_worktree_path(&self, goal_id: &str) -> PathBuf {
        self.worktree_base.join(goal_id)
    }

    /// 检查 worktree 是否存在
    pub fn worktree_exists(&self, goal_id: &str) -> bool {
        self.worktree_base.join(goal_id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn worktree_manager_new() {
        let manager = WorktreeManager::new("/repo", "/worktrees");
        assert_eq!(manager.repo_path, PathBuf::from("/repo"));
        assert_eq!(manager.worktree_base, PathBuf::from("/worktrees"));
    }

    #[test]
    fn worktree_get_path() {
        let manager = WorktreeManager::new("/repo", "/worktrees");
        let path = manager.get_worktree_path("goal-001");
        assert_eq!(path, PathBuf::from("/worktrees/goal-001"));
    }

    #[test]
    fn worktree_exists_check() {
        let manager = WorktreeManager::new("/repo", "/nonexistent");
        assert!(!manager.worktree_exists("goal-001"));
    }

    #[test]
    fn worktree_manager_from_current_repo() {
        let manager = WorktreeManager::new(".", "./worktrees");
        assert!(manager.repo_path.is_relative());
    }
}
