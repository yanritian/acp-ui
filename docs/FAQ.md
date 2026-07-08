# Frequently Asked Questions (FAQ)

## General

### What is Hermes Game Operator?

Hermes Game Operator is an AI-powered game development assistant integrated into ACP UI. It helps developers create, modify, and manage Godot Engine projects through natural language commands and automated workflows.

### How does it work?

1. You select a Godot project directory
2. You describe your task goal in natural language
3. The operator analyzes the project and generates an execution plan
4. You review and approve the plan
5. The operator executes the plan step by step
6. You monitor progress and approve file modifications
7. The operator completes the task and provides a summary

### Is it safe to use?

Yes! Hermes Game Operator includes multiple safety features:
- **PathGuard**: Validates all file operations stay within allowed boundaries
- **CommandGuard**: Whitelist-based command execution
- **Approval System**: Dangerous operations require user approval
- **File Backup**: Automatic backups before modifications
- **Event Tracking**: Complete audit trail of all operations

### What Godot versions are supported?

Currently supports Godot 4.x. Support for Godot 3.x is planned for future releases.

### Do I need to know GDScript?

No! The operator can generate and modify GDScript code automatically. However, understanding GDScript will help you review and understand the generated code.

## Installation

### How do I install Hermes Game Operator?

Hermes Game Operator is built into ACP UI. Simply:
1. Download ACP UI from the official website
2. Install the application
3. Launch and navigate to Game Operator (default homepage)

### What are the system requirements?

- **OS**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)
- **RAM**: 4 GB minimum, 8 GB recommended
- **Disk Space**: 500 MB for application + project space
- **Internet**: Required for Hermes Agent API calls

### Can I use it offline?

Partially. Project analysis works offline, but code generation requires internet access to connect to Hermes Agent.

## Usage

### How do I start a task?

1. Click "Browse" to select your Godot project directory
2. Enter your task goal in the text area
3. Click "Start Task"
4. Review the execution plan
5. Approve the plan
6. Monitor progress
7. Approve file modifications when prompted

### What kinds of tasks can I perform?

Common tasks include:
- Adding new features (e.g., double jump, sprint ability)
- Modifying existing behavior (e.g., change enemy speed)
- Creating new scenes or scripts
- Fixing bugs
- Refactoring code
- Optimizing performance

### Can I pause a task?

Yes! You can pause, resume, or stop tasks at any time:
- **Pause**: Temporarily stops execution
- **Resume**: Continues from where it paused
- **Stop**: Cancels the task completely

### What if I don't like the plan?

You can:
- **Reject**: Cancel the current plan and start over
- **Redirect**: Change the goal and generate a new plan
- **Modify**: Edit the plan steps before approving

### How do I approve file modifications?

When the operator wants to modify a file:
1. An approval request appears in the Approval Drawer
2. Review the diff (changes)
3. Click "Approve" to allow the modification
4. Or click "Reject" to deny it

### Can I see what files were modified?

Yes! After task completion, you can:
- View the event stream for file modification events
- Check the task summary for list of modified files
- Open files directly in your editor
- Compare with backups (.bak files)

## Troubleshooting

### Task fails to start

**Possible Causes**:
- Invalid project path
- Not a Godot project
- Insufficient permissions

**Solutions**:
- Verify the project path is correct
- Ensure `project.godot` exists in the directory
- Check file permissions

### No player controller found

**Possible Causes**:
- Player script uses non-standard naming
- Project structure is unusual

**Solutions**:
- Rename player script to include "player", "character", or "controller"
- Manually specify the player script path
- Restructure project to follow standard conventions

### Approval timeout

**Possible Causes**:
- Too slow to respond
- Network issues

**Solutions**:
- Respond to approval requests promptly
- Check internet connection
- Adjust timeout settings in preferences

### Code generation fails

**Possible Causes**:
- Hermes Agent API unavailable
- Invalid API key
- Rate limit exceeded

**Solutions**:
- Check internet connection
- Verify API key in settings
- Wait for rate limit to reset
- Try again later

### File modification rejected

**Possible Causes**:
- Path outside project directory
- Forbidden operation
- Security policy violation

**Solutions**:
- Check file path is within project
- Review approval request details
- Adjust security policy if needed

## Security

### What files can be modified?

Only files within the selected project directory can be modified. The operator cannot:
- Access system files
- Modify files outside the project
- Delete files without approval
- Execute arbitrary commands

### Are my API keys safe?

Yes! API keys are:
- Stored securely in system keychain
- Never logged or transmitted in plain text
- Only used for Hermes Agent API calls
- Can be rotated at any time

### Can the operator run arbitrary commands?

No! Commands are restricted by:
- Whitelist of allowed commands
- Approval requirements
- Security policies
- Path validation

### What happens if something goes wrong?

The operator provides:
- Automatic backups before modifications
- Rollback capability
- Detailed error messages
- Recovery suggestions
- Event history for debugging

## Advanced

### Can I customize the approval policy?

Yes! Access via Settings → Approval Policy:
- **Safe Default**: Level 2 for modifications
- **Permissive**: Level 1 for most operations
- **Strict**: Level 3 for all operations

### Can I use custom tools?

Yes! You can:
- Create custom skills in `skills/` directory
- Define custom tools in configuration
- Extend the operator with plugins

### How do I backup my project?

The operator automatically:
- Creates .bak files before modifications
- Maintains event history
- Provides rollback capability

You can also:
- Use Git for version control
- Create manual backups
- Export project regularly

### Can I integrate with Git?

Yes! The operator can:
- Read Git status
- Show uncommitted changes
- Suggest commits after modifications
- Create branches for experimental changes

### How do I report bugs?

Use GitHub Issues:
1. Go to the repository
2. Click "Issues"
3. Click "New Issue"
4. Provide detailed description
5. Include logs if available

### Where can I get help?

- **Documentation**: See docs/ directory
- **FAQ**: This document
- **GitHub Discussions**: Ask questions
- **Discord**: Real-time chat support
- **Email**: support@example.com

## Performance

### How long does analysis take?

Typical analysis times:
- Small project (< 50 files): 2-5 seconds
- Medium project (50-200 files): 5-15 seconds
- Large project (200+ files): 15-60 seconds

### Can I optimize performance?

Yes:
- Exclude unnecessary directories
- Use .godotignore for large assets
- Split large projects into modules
- Close other applications

### Does it use a lot of memory?

Memory usage:
- Idle: 200-300 MB
- Analyzing: 400-600 MB
- Executing: 500-800 MB

### Can I limit resource usage?

Yes, in Settings:
- Max concurrent tasks
- Memory limit
- CPU priority
- Network bandwidth

## Migration

### Can I import existing projects?

Yes! Simply:
1. Select the project directory
2. The operator will analyze it automatically
3. No special import process needed

### Can I export the operator's work?

Yes! The operator:
- Modifies files directly in your project
- Creates backups you can restore
- Generates summaries you can save
- Works with standard Godot files

### How do I upgrade to a new version?

1. Download the new version
2. Install over the existing version
3. Your projects and settings are preserved
4. No migration needed

## Comparison

### How is this different from other AI tools?

Hermes Game Operator:
- **Integrated**: Built into ACP UI
- **Safe**: Multiple safety features
- **Controllable**: Full user control
- **Transparent**: Complete event tracking
- **Specialized**: Focused on game development

### Why Godot first?

Godot was chosen because:
- Open source and free
- Growing community
- Excellent for indie developers
- Clean and readable GDScript
- Strong 2D/3D capabilities

### Will other engines be supported?

Yes! Future plans include:
- Unity (C#)
- Unreal (C++)
- Ren'Py (Python)
- Custom engines (plugin system)

## Legal

### Is it free to use?

Yes! ACP UI is free and open source under MIT License.

### Do I need an API key?

Yes, for code generation features. You can get one from:
- Anthropic (Claude)
- OpenAI (GPT)
- Other providers

### Can I use it commercially?

Yes! You can use it for:
- Personal projects
- Commercial games
- Client work
- Educational purposes

### Who owns the generated code?

You do! All generated code is yours to use as you wish.

---

## Still Have Questions?

- **Documentation**: See docs/ directory
- **GitHub Discussions**: Ask the community
- **Discord**: Join our server
- **Email**: support@example.com

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
