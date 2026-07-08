# Hermes Game Operator User Manual

## 🎮 Introduction

Hermes Game Operator is an AI-powered game development assistant that helps you create, modify, and manage Godot Engine projects through natural language commands and automated workflows.

## 🚀 Getting Started

### Installation

1. Download ACP UI from the official website
2. Install the application
3. Launch ACP UI
4. Navigate to Game Operator (default homepage)

### First Task

1. **Select Project**: Click "Browse" and select your Godot project directory
2. **Enter Goal**: Describe what you want to achieve (e.g., "Add double jump to player")
3. **Start Task**: Click "Start Task" button
4. **Review Plan**: Examine the generated execution plan
5. **Approve**: Approve file modifications when prompted
6. **Monitor**: Watch the progress timeline
7. **Review Results**: Check the final summary

## 📋 Core Concepts

### Task Lifecycle

Every task goes through these states:

```
Idle → Planning → WaitingApproval → Running → Completed
                ↘ Cancelled
         Running → Paused → Running
         Running → Cancelling → Cancelled
         Running → Redirecting → Planning
         Running → Failed → Planning (retry)
```

### Event Stream

The event stream shows real-time progress:

- **Task Events**: Task creation, completion, failure
- **Analysis Events**: Project analysis progress
- **Plan Events**: Plan generation and approval
- **Execution Events**: Step execution progress
- **File Events**: File modifications and backups

### Approval System

Dangerous operations require approval:

- **Level 0 (Silent)**: Safe read operations
- **Level 1 (Notify)**: Can execute, show in progress
- **Level 2 (Approve)**: Must get user approval
- **Level 3 (Forbidden)**: Always blocked

## 🎯 Common Tasks

### Task 1: Add Double Jump

**Goal**: "Add double jump ability to the player character"

**Steps**:
1. Select your Godot project
2. Enter the goal
3. Start the task
4. Review the plan (should include analyzing player script)
5. Approve the modification
6. Wait for completion
7. Review the changes in Player.gd

**Expected Changes**:
```gdscript
# Added variables
var jump_count = 0
var max_jumps = 2

# Modified jump logic
if Input.is_action_just_pressed("jump"):
    if jump_count < max_jumps:
        velocity.y = JUMP_VELOCITY
        jump_count += 1

# Reset on landing
if is_on_floor():
    jump_count = 0
```

### Task 2: Modify Enemy Behavior

**Goal**: "Change enemy patrol speed from 100 to 150"

**Steps**:
1. Select project
2. Enter goal
3. Start task
4. Approve modification to Enemy.gd
5. Review changes

**Expected Changes**:
```gdscript
# Before
const SPEED = 100.0

# After
const SPEED = 150.0
```

### Task 3: Create New Scene

**Goal**: "Create a new menu scene with start and quit buttons"

**Steps**:
1. Select project
2. Enter goal
3. Start task
4. Approve scene creation
5. Approve script generation
6. Review new files

**Expected Files**:
- `scenes/Menu.tscn`
- `scripts/Menu.gd`

## 🎛️ Controls

### Task Control Bar

- **Pause**: Pause the current task
- **Resume**: Resume a paused task
- **Stop**: Stop and cancel the task
- **Redirect**: Change the task goal

### Progress Timeline

Shows chronological events:
- Timestamp
- Event type
- Event title
- Expandable details

### Plan Panel

Displays execution plan:
- Step number
- Description
- Status (pending/running/completed/failed)
- Files to modify

### Approval Drawer

Shows pending approvals:
- Action description
- Risk level
- File preview
- Approve/Reject buttons

## 🔒 Security

### Path Guard

All file operations are validated:
- Must be within project directory
- Cannot access system files
- Prevents directory traversal

### Command Guard

Shell commands are filtered:
- Whitelist of allowed commands
- Dangerous commands blocked
- Requires approval for modifications

### File Backup

Automatic backups before modifications:
- Original file saved as `.bak`
- Can be restored manually
- Kept until task completion

## 📊 Monitoring

### Event Types

**Task Events**:
- `task_started` - Task execution began
- `task_completed` - Task finished successfully
- `task_failed` - Task encountered an error
- `task_cancelled` - Task was cancelled

**Analysis Events**:
- `project_analyzed` - Project structure analyzed
- `scripts_found` - Scripts discovered
- `scenes_found` - Scenes discovered

**Plan Events**:
- `plan_generated` - Execution plan created
- `plan_approved` - User approved the plan
- `plan_rejected` - User rejected the plan

**Execution Events**:
- `step_started` - Step execution began
- `step_completed` - Step finished
- `step_failed` - Step encountered an error

**File Events**:
- `file_read` - File was read
- `file_modified` - File was modified
- `file_backed_up` - Backup created

### Statistics

View task statistics:
- Total steps
- Completed steps
- Failed steps
- Duration
- Files modified

## 🐛 Troubleshooting

### Issue: Task fails to start

**Possible Causes**:
- Project directory not found
- Not a valid Godot project
- Insufficient permissions

**Solutions**:
- Verify project path
- Check for `project.godot` file
- Ensure write permissions

### Issue: Plan generation fails

**Possible Causes**:
- Invalid project structure
- Missing required files
- Agent not available

**Solutions**:
- Verify project structure
- Check Godot version
- Restart the application

### Issue: File modification rejected

**Possible Causes**:
- Path outside project directory
- Forbidden operation
- Approval timeout

**Solutions**:
- Check file path
- Review approval request
- Respond to approval promptly

### Issue: Task paused unexpectedly

**Possible Causes**:
- User paused manually
- Error encountered
- Approval required

**Solutions**:
- Check event stream
- Resume if appropriate
- Approve pending requests

## 🎓 Advanced Usage

### Redirecting Tasks

Change task direction mid-execution:

1. Pause the current task
2. Click "Redirect"
3. Enter new goal
4. Review revised plan
5. Approve new direction

### Batch Operations

Execute multiple tasks sequentially:

1. Complete first task
2. Review results
3. Start next task
4. Continue as needed

### Custom Approval Policies

Configure approval levels:

- **Safe Default**: Level 2 for modifications
- **Permissive**: Level 1 for most operations
- **Strict**: Level 3 for all operations

Access via Settings → Approval Policy

## 📚 Examples

### Example 1: Simple Modification

**Goal**: "Change player speed from 300 to 400"

**Expected Result**:
```gdscript
# Player.gd
const SPEED = 400.0  # Changed from 300
```

### Example 2: Feature Addition

**Goal**: "Add sprint ability to player"

**Expected Result**:
```gdscript
# Player.gd
const SPRINT_SPEED = 500.0
var is_sprinting = false

if Input.is_action_pressed("sprint"):
    is_sprinting = true
    speed = SPRINT_SPEED
else:
    is_sprinting = false
    speed = SPEED
```

### Example 3: Bug Fix

**Goal**: "Fix player falling through floor"

**Expected Result**:
```gdscript
# Player.gd
func _physics_process(delta):
    # Fixed gravity calculation
    velocity += get_gravity() * delta
    move_and_slide()
```

## 🎯 Best Practices

### 1. Start Small

Begin with simple modifications before complex features.

### 2. Review Plans Carefully

Always review the execution plan before approving.

### 3. Test Incrementally

Test changes after each task completion.

### 4. Use Backups

Restore from backups if something goes wrong.

### 5. Monitor Events

Watch the event stream for real-time feedback.

### 6. Approve Promptly

Respond to approval requests quickly to avoid timeouts.

### 7. Keep Goals Clear

Write clear, specific task goals.

### 8. Review Diffs

Check file diffs before approving modifications.

## 🔗 Resources

- [API Documentation](api.md)
- [Test Plan](codex/test-plan.md)
- [Test Project](codex/test-godot-project.md)
- [Godot Documentation](https://docs.godotengine.org/)
- [GDScript Reference](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/index.html)

## 📞 Support

- **Documentation**: See docs/ directory
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: [Join our server](https://discord.gg/example)

## 🎊 Conclusion

Hermes Game Operator makes game development faster, safer, and more enjoyable. By combining AI-powered automation with user control and safety features, you can focus on creativity while the operator handles the implementation details.

Happy game development! 🎮

---

**Version**: 1.0.0  
**Last Updated**: 2026-07-08
