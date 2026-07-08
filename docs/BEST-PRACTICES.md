# Best Practices Guide

This guide provides best practices for using Hermes Game Operator effectively and safely.

## Task Design

### Write Clear Goals

**Good**:
```
Add double jump ability to the player character, allowing up to 2 jumps in the air.
```

**Bad**:
```
Make jumping better.
```

**Why**: Clear, specific goals help the operator generate accurate plans.

### Break Down Complex Tasks

**Good**:
1. "Analyze current player movement system"
2. "Add double jump logic to Player.gd"
3. "Test jump mechanics in main scene"

**Bad**:
```
Completely rewrite the player controller with new movement, jumping, and combat systems.
```

**Why**: Smaller tasks are easier to review, approve, and debug.

### Provide Context

**Good**:
```
Add sprint ability to player. Current speed is 300, sprint should be 500. 
Keep the existing jump mechanics unchanged.
```

**Bad**:
```
Add sprint.
```

**Why**: Context helps the operator understand constraints and requirements.

### Use Standard Naming

**Good**:
- `Player.gd` - Player controller
- `Enemy.gd` - Enemy behavior
- `Main.tscn` - Main scene

**Bad**:
- `CharacterController2D_v2_final.gd`
- `EnemyBehaviorScript.gd`
- `Scene1.tscn`

**Why**: Standard naming helps the operator identify relevant files.

## Project Structure

### Follow Godot Conventions

```
project/
├── project.godot
├── scenes/
│   ├── Main.tscn
│   └── Menu.tscn
├── scripts/
│   ├── Player.gd
│   └── Enemy.gd
└── assets/
    ├── sprites/
    └── audio/
```

### Use .godotignore

```
# Build outputs
build/
export/

# Temporary files
*.tmp
*.bak

# IDE files
.vscode/
.idea/

# OS files
.DS_Store
Thumbs.db
```

### Keep Projects Organized

- Group related files in directories
- Use consistent naming conventions
- Document complex systems
- Keep scenes modular

## Safety Practices

### Always Review Plans

Before approving:
1. ✅ Read all steps carefully
2. ✅ Check which files will be modified
3. ✅ Verify the approach makes sense
4. ✅ Consider edge cases

### Use Version Control

```bash
# Before starting task
git status
git add .
git commit -m "Backup before operator task"

# After task completion
git diff
git add .
git commit -m "Add feature X via operator"
```

### Test Incrementally

1. Complete small task
2. Test in Godot editor
3. Verify functionality
4. Commit changes
5. Continue to next task

### Keep Backups

The operator creates `.bak` files automatically, but also:
- Use Git for version history
- Create manual backups for critical files
- Export project regularly

## Approval Best Practices

### Review Diffs Carefully

```gdscript
# Before
const SPEED = 300.0

# After
const SPEED = 400.0  # Increased for better gameplay feel
```

Check:
- ✅ Are changes correct?
- ✅ Are comments helpful?
- ✅ Is code style consistent?
- ✅ Are there any unintended changes?

### Understand Approval Levels

**Level 0 (Silent)**: Safe read operations
- No approval needed
- Safe to auto-approve

**Level 1 (Notify)**: Can execute, show in progress
- Review in event stream
- Generally safe

**Level 2 (Approve)**: Must get user approval
- Review carefully
- Check diffs
- Consider impact

**Level 3 (Forbidden)**: Always blocked
- Dangerous operations
- Requires policy change
- Use with extreme caution

### Respond Promptly

Approval requests can timeout:
- Monitor approval drawer
- Respond within timeout period
- Ask for more time if needed

## Monitoring Best Practices

### Watch Event Stream

Key events to monitor:
- `task_started` - Task beginning
- `project_analyzed` - Analysis complete
- `plan_ready` - Plan generated
- `approval_requested` - Needs approval
- `file_modified` - File changed
- `task_completed` - Task finished

### Check Progress Regularly

For long tasks:
1. Monitor event stream
2. Check step progress
3. Verify file modifications
4. Approve pending requests

### Review Summaries

After completion:
- Read task summary
- Check modified files list
- Verify expected changes
- Note any warnings

## Code Quality

### Maintain Style Consistency

Follow project's existing style:
- Indentation (tabs vs spaces)
- Naming conventions
- Comment style
- Code organization

### Add Meaningful Comments

```gdscript
# Good: Explains why
var jump_count = 0  # Tracks number of jumps for double jump mechanic

# Bad: States obvious
var jump_count = 0  # Jump count variable
```

### Test Generated Code

Before approving:
1. ✅ Syntax is correct
2. ✅ Logic makes sense
3. ✅ No obvious bugs
4. ✅ Follows best practices

### Document Changes

After task completion:
- Update relevant documentation
- Add comments for complex logic
- Note any manual adjustments needed

## Performance Optimization

### Optimize Project Analysis

- Use `.godotignore` for large assets
- Exclude build directories
- Keep project structure clean
- Split large projects into modules

### Manage Memory

- Close unnecessary applications
- Limit concurrent tasks
- Clear cache periodically
- Restart if memory issues

### Network Efficiency

- Use stable internet connection
- Enable response caching
- Batch similar operations
- Minimize API calls

## Security Best Practices

### Validate Paths

- Always use project-relative paths
- Never use absolute system paths
- Check for path traversal attempts
- Validate all file operations

### Control Commands

- Use whitelist for allowed commands
- Review command details before approval
- Never execute unknown commands
- Log all command executions

### Protect Secrets

- Never commit API keys
- Use environment variables
- Rotate keys regularly
- Use secure storage

### Monitor Access

- Review event stream regularly
- Check for unauthorized operations
- Audit file modifications
- Report suspicious activity

## Workflow Best Practices

### Plan Before Executing

1. Define clear goal
2. Break into smaller tasks
3. Identify dependencies
4. Estimate complexity
5. Prepare backups

### Iterate Incrementally

```
Task 1: Analyze current system
Task 2: Implement core feature
Task 3: Add edge cases
Task 4: Optimize performance
Task 5: Add documentation
```

### Review and Refine

After each task:
1. Test functionality
2. Review code quality
3. Check for issues
4. Refactor if needed
5. Document changes

### Document Everything

- Task goals and outcomes
- Design decisions
- Code changes
- Testing results
- Known issues

## Common Patterns

### Adding New Features

1. Analyze existing system
2. Design feature architecture
3. Implement core logic
4. Add UI integration
5. Test thoroughly
6. Document feature

### Fixing Bugs

1. Reproduce bug
2. Identify root cause
3. Implement fix
4. Test fix
5. Add regression test
6. Document fix

### Refactoring Code

1. Identify code smells
2. Design refactoring plan
3. Refactor incrementally
4. Test after each change
5. Verify no regressions
6. Update documentation

### Creating New Scenes

1. Design scene structure
2. Create scene file
3. Add nodes and scripts
4. Connect signals
5. Test scene
6. Integrate with main game

## Advanced Techniques

### Custom Skills

Create custom skills for common tasks:

```
skills/
└── godot/
    └── custom-feature/
        └── SKILL.md
```

### Batch Operations

Execute multiple related tasks:
1. Create batch plan
2. Execute tasks sequentially
3. Review each result
4. Commit after each task

### Integration Testing

Test operator with real projects:
1. Use test project
2. Run common tasks
3. Verify results
4. Report issues

## Anti-Patterns

### Avoid These Mistakes

❌ **Vague Goals**: "Make it better"
✅ **Specific Goals**: "Increase player speed by 20%"

❌ **Large Tasks**: "Rewrite entire game"
✅ **Small Tasks**: "Add double jump"

❌ **No Testing**: Approve without testing
✅ **Test First**: Verify in editor

❌ **Ignoring Warnings**: Skip review
✅ **Review Always**: Check all changes

❌ **No Backups**: Work without version control
✅ **Use Git**: Track all changes

## Resources

- [User Manual](USER-MANUAL.md)
- [API Reference](api.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Security Guide](SECURITY.md)
- [FAQ](FAQ.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
