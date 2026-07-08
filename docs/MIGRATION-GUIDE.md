# Migration Guide

This guide helps you migrate from previous versions of Hermes Game Operator or other tools.

## From v0.x to v1.0.0

### Breaking Changes

#### 1. Operator Protocol

**Old (v0.x)**:
```typescript
const task = await OperatorApi.startTask({
  domain: 'godot',
  goal: 'Add feature'
})
```

**New (v1.0.0)**:
```typescript
const task = await OperatorApi.startTask({
  domain: 'game.godot',  // Changed format
  project_path: '/path/to/project',  // New required field
  goal: 'Add feature',
  mode: 'propose_then_apply',  // New required field
  approval_policy: 'safe_default'  // New required field
})
```

**Migration Steps**:
1. Update domain format: `godot` → `game.godot`
2. Add `project_path` parameter
3. Add `mode` parameter
4. Add `approval_policy` parameter

---

#### 2. Event Stream Format

**Old (v0.x)**:
```json
{
  "type": "file_changed",
  "data": {
    "path": "scripts/Player.gd",
    "changes": 5
  }
}
```

**New (v1.0.0)**:
```json
{
  "type": "file_modified",
  "level": "info",
  "title": "File modified",
  "payload": {
    "path": "scripts/Player.gd",
    "lines_changed": 5
  }
}
```

**Migration Steps**:
1. Update event type names
2. Move event data to `payload` field
3. Add `level` and `title` fields

---

#### 3. Approval System

**Old (v0.x)**:
```typescript
await OperatorApi.approve({
  task_id: "task_123",
  approval_id: "appr_001",
  approved: true
})
```

**New (v1.0.0)**:
```typescript
await OperatorApi.approve({
  task_id: "task_123",
  approval_id: "appr_001",
  decision: "approve",  // Changed from boolean
  comment: "Looks good"  // New optional field
})
```

**Migration Steps**:
1. Change `approved: boolean` to `decision: string`
2. Use `"approve"`, `"reject"`, or `"request_changes"`
3. Optionally add `comment` field

---

#### 4. Configuration

**Old (v0.x)**:
```json
{
  "version": "0.9.0",
  "api_key": "key123",
  "timeout": 30
}
```

**New (v1.0.0)**:
```json
{
  "version": "1.0.0",
  "api": {
    "key": "key123",
    "endpoint": "https://api.anthropic.com",
    "timeout": 30
  },
  "operator": {
    "mode": "propose_then_apply",
    "approval_policy": "safe_default"
  }
}
```

**Migration Steps**:
1. Restructure API configuration under `api` object
2. Add operator configuration
3. Update version number

---

### Automatic Migration

The application includes an automatic migration tool:

```bash
# Run migration
npm run migrate

# Or use the CLI
npx hermes-migrate --from 0.9.0 --to 1.0.0
```

**What it migrates**:
- ✅ Configuration files
- ✅ Task history
- ✅ Event logs
- ✅ User preferences
- ✅ Agent configurations

**What it doesn't migrate**:
- ❌ Custom skills (manual update needed)
- ❌ API keys (re-enter recommended)
- ❌ Custom scripts (manual update needed)

---

## From Other Tools

### From Claude Code

If you're currently using Claude Code for game development:

#### Migration Steps

1. **Export Your Work**
   ```bash
   # Save your current work
   git add .
   git commit -m "Backup before migration"
   ```

2. **Install Hermes Game Operator**
   ```bash
   # Download and install
   # https://github.com/yourusername/acp-ui/releases
   ```

3. **Configure API Key**
   ```bash
   # Set your API key
   export ANTHROPIC_API_KEY="your-key-here"
   ```

4. **Import Projects**
   - Open Hermes Game Operator
   - Select your Godot project
   - Start using the operator

#### Feature Comparison

| Feature | Claude Code | Hermes Game Operator |
|---------|-------------|----------------------|
| Game-specific | ❌ | ✅ |
| Visual UI | ❌ | ✅ |
| Approval system | Manual | ✅ |
| Event tracking | Basic | ✅ |
| Memory management | Basic | ✅ |
| Multi-task | ❌ | ✅ |

---

### From GitHub Copilot

If you're currently using GitHub Copilot:

#### Migration Steps

1. **Continue Using Copilot**
   - Hermes Game Operator doesn't replace Copilot
   - Use both together for best results

2. **Complementary Features**
   - Copilot: Inline code suggestions
   - Hermes: Task-level automation

3. **Workflow**
   - Use Hermes for planning and large changes
   - Use Copilot for inline editing

#### Integration

```typescript
// Use both tools together
// Hermes for task planning
const plan = await OperatorApi.generatePlan(task)

// Copilot for code completion
// (In your editor)
```

---

### From Cursor

If you're currently using Cursor:

#### Migration Steps

1. **Export Settings**
   - Save your Cursor settings
   - Note your preferences

2. **Configure Hermes**
   - Set similar preferences in Hermes
   - Customize to your workflow

3. **Gradual Migration**
   - Start with small tasks
   - Build confidence
   - Migrate larger tasks

#### Feature Comparison

| Feature | Cursor | Hermes Game Operator |
|---------|--------|----------------------|
| AI-powered | ✅ | ✅ |
| Game-specific | ❌ | ✅ |
| Task management | Basic | ✅ |
| Approval system | Manual | ✅ |
| Event tracking | Basic | ✅ |
| Desktop app | ✅ | ✅ |

---

### From Manual Development

If you're currently developing manually:

#### Benefits of Migration

1. **Faster Development**
   - Automated code generation
   - Intelligent suggestions
   - Task automation

2. **Better Quality**
   - Consistent code style
   - Best practices enforced
   - Error prevention

3. **Safer Operations**
   - Approval system
   - Automatic backups
   - Audit trail

#### Migration Steps

1. **Start Small**
   - Begin with simple tasks
   - Test in non-critical projects
   - Build confidence

2. **Learn the Workflow**
   - Understand the operator
   - Practice with test projects
   - Review documentation

3. **Gradual Adoption**
   - Use for new features
   - Use for bug fixes
   - Use for refactoring

---

## Data Migration

### Export from Old Version

```bash
# Export configuration
hermes export --config > config.json

# Export task history
hermes export --tasks > tasks.json

# Export events
hermes export --events > events.json
```

### Import to New Version

```bash
# Import configuration
hermes import --config config.json

# Import task history
hermes import --tasks tasks.json

# Import events
hermes import --events events.json
```

---

## API Migration

### Old API (v0.x)

```typescript
// Start task
const task = await api.startTask({
  domain: 'godot',
  goal: 'Add feature'
})

// Get task
const task = await api.getTask(taskId)

// Approve
await api.approve({
  task_id: taskId,
  approval_id: approvalId,
  approved: true
})
```

### New API (v1.0.0)

```typescript
// Start task
const task = await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: '/path/to/project',
  goal: 'Add feature',
  mode: 'propose_then_apply',
  approval_policy: 'safe_default'
})

// Get task
const task = await OperatorApi.getTask(taskId)

// Approve
await OperatorApi.approve({
  task_id: taskId,
  approval_id: approvalId,
  decision: 'approve',
  comment: 'Looks good'
})
```

---

## Configuration Migration

### Old Configuration

```json
{
  "version": "0.9.0",
  "api_key": "sk-ant-...",
  "timeout": 30,
  "model": "claude-3-5-sonnet-20241022"
}
```

### New Configuration

```json
{
  "version": "1.0.0",
  "api": {
    "key": "sk-ant-...",
    "endpoint": "https://api.anthropic.com",
    "timeout": 30,
    "model": "claude-3-5-sonnet-20241022"
  },
  "operator": {
    "mode": "propose_then_apply",
    "approval_policy": "safe_default",
    "max_concurrent_tasks": 3
  },
  "security": {
    "path_guard_enabled": true,
    "command_guard_enabled": true,
    "allowed_roots": []
  }
}
```

---

## Troubleshooting

### Migration Fails

**Problem**: Migration tool fails

**Solution**:
```bash
# Manual migration
1. Backup old configuration
2. Install new version
3. Manually update configuration
4. Test with sample project
```

### API Key Issues

**Problem**: API key not working after migration

**Solution**:
```bash
# Re-enter API key
Settings → API Keys → Add New

# Or via environment variable
export ANTHROPIC_API_KEY="your-key-here"
```

### Task History Lost

**Problem**: Task history not migrated

**Solution**:
```bash
# Export from old version
hermes export --tasks > tasks.json

# Import to new version
hermes import --tasks tasks.json
```

---

## Rollback Procedure

If migration causes issues:

### Desktop Application

1. **Uninstall new version**
   ```bash
   # Windows
   Control Panel → Programs → Uninstall
   
   # macOS
   Drag to Trash
   
   # Linux
   sudo apt remove hermes-operator
   ```

2. **Restore from backup**
   ```bash
   # Restore configuration
   cp ~/.config/hermes-operator.backup/* ~/.config/hermes-operator/
   ```

3. **Install old version**
   - Download previous version
   - Install normally

### Web Application

Web version automatically rolls back when you access the previous deployment.

---

## Best Practices

### 1. Backup Before Migration

```bash
# Backup everything
cp -r ~/.config/hermes-operator ~/.config/hermes-operator.backup
git add .
git commit -m "Backup before migration"
```

### 2. Test in Staging

- Use a test project
- Verify all features work
- Check performance

### 3. Migrate Gradually

- Start with small projects
- Build confidence
- Migrate larger projects

### 4. Document Changes

- Keep a migration log
- Note any issues
- Track resolution

---

## Resources

- [Upgrade Guide](UPGRADE.md)
- [User Manual](USER-MANUAL.md)
- [API Reference](api.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Support](mailto:support@example.com)

---

**Migration Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08
