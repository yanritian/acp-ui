# Git Workspace Isolation

ACP-Swarm uses Git branch isolation to ensure each worker has a clean workspace.

## Mechanism

When a Goal is assigned to a worker, the system:

1. Creates a new branch: `worker/{worker-id}/{goal-id}`
2. Worker executes on this isolated branch
3. On convergence, branch is merged back to main
4. On failure, branch is cleaned up without merge

## Branch Naming

```
main
├── worker/claude-code-001/fix-typescript
├── worker/codex-002/run-tests
└── worker/claude-code-003/generate-docs
```

## Benefits

### 1. No Conflicts
Each worker operates independently without interfering with others.

### 2. Traceability
Git history shows exactly what each worker did for each goal.

### 3. Safe Rollback
Failed goals can be discarded without affecting main branch.

### 4. Review Process
Converged work can be reviewed before merging.

## Configuration

In your `.goal.yaml`:

```yaml
workers:
  - id: claude-code-001
    type: claude_code
    workspace_isolation: true  # Enable (default)
```

## Manual Control

```bash
# Create worker branch
git checkout -b worker/claude-code-001/goal-001 main

# Worker executes...
git add .
git commit -m "Goal converged: goal-001"

# Merge converged work
git checkout main
git merge worker/claude-code-001/goal-001

# Clean up
git branch -d worker/claude-code-001/goal-001
```

## Best Practices

### 1. Start from Clean Main
```bash
git checkout main
git pull origin main
# Ensure main is up-to-date before spawning worker branches
```

### 2. Regular Cleanup
```bash
# Remove abandoned worker branches
git branch --list 'worker/*' | xargs git branch -D
```

### 3. Conflict Resolution
If two workers modify the same file:
1. System detects conflict during merge
2. Queen worker resolves conflict
3. Manual intervention if needed

## Implementation

The `WorktreeManager` in `src/worktree.rs` handles this:

```rust
pub struct WorktreeManager {
    repo_path: PathBuf,
    worktree_base: PathBuf,
}

impl WorktreeManager {
    pub fn create_worktree(&self, goal_id: &str) -> Result<PathBuf, WorktreeError>;
    pub fn remove_worktree(&self, goal_id: &str) -> Result<(), WorktreeError>;
    pub fn list_worktrees(&self) -> Result<Vec<String>, WorktreeError>;
    pub fn cleanup_all(&self) -> Result<u32, WorktreeError>;
}
```