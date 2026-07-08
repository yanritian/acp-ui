# Troubleshooting Guide

## Common Issues

### Issue 1: Task Won't Start

**Symptoms**:
- Click "Start Task" but nothing happens
- Error message appears
- Task status remains "Idle"

**Possible Causes**:
1. Invalid project path
2. Project is not a Godot project
3. Insufficient permissions
4. API not available

**Solutions**:

#### Check Project Path
```bash
# Verify path exists
ls -la /path/to/project

# Check for project.godot
ls /path/to/project/project.godot
```

#### Verify Godot Project
```bash
# Use API to detect
curl http://localhost:1420/api/godot/detect \
  -d '{"path":"/path/to/project"}'
```

#### Check Permissions
```bash
# Verify write permissions
touch /path/to/project/test.txt && rm /path/to/project/test.txt
```

#### Restart Application
```bash
# Stop application
pkill -f hermes-operator

# Start application
npm run tauri dev
```

---

### Issue 2: Plan Generation Fails

**Symptoms**:
- Task starts but plan never generates
- Status stuck at "Planning"
- Error in event stream

**Possible Causes**:
1. API key not configured
2. Network connectivity issues
3. Invalid goal description
4. Project too large

**Solutions**:

#### Verify API Key
```bash
# Check configuration
cat ~/.config/hermes-operator/config.json | grep api_key

# Test API connection
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: YOUR_API_KEY" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-5-sonnet-20241022","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'
```

#### Check Network
```bash
# Test connectivity
ping api.anthropic.com

# Check DNS
nslookup api.anthropic.com
```

#### Simplify Goal
```
# Bad: Too complex
"Rewrite the entire game with new mechanics, better graphics, and improved performance"

# Good: Simple and specific
"Add double jump to player"
```

#### Check Project Size
```bash
# Count files
find /path/to/project -type f | wc -l

# If > 1000 files, consider:
# - Exclude unnecessary directories
# - Split into smaller projects
# - Use .godotignore
```

---

### Issue 3: File Modification Rejected

**Symptoms**:
- Approval request appears
- Cannot approve
- Error message

**Possible Causes**:
1. Path outside project directory
2. Forbidden file type
3. Approval timeout
4. Permission denied

**Solutions**:

#### Check Path
```bash
# Verify path is within project
realpath /path/to/file.gd | grep /path/to/project
```

#### Check File Type
```bash
# Verify file is allowed
# Allowed: .gd, .tscn, .tres
# Forbidden: .import, .godot
```

#### Restart Approval Process
```bash
# Stop task
curl -X POST http://localhost:1420/api/operator/stop \
  -d '{"task_id":"task_123"}'

# Start new task
curl -X POST http://localhost:1420/api/operator/start \
  -d '{"domain":"game.godot","project_path":"/path","goal":"Add feature"}'
```

---

### Issue 4: Performance Issues

**Symptoms**:
- Slow response times
- High memory usage
- Application freezes

**Possible Causes**:
1. Large project analysis
2. Too many concurrent tasks
3. Memory leaks
4. Network latency

**Solutions**:

#### Optimize Project
```bash
# Create .godotignore
cat > /path/to/project/.godotignore << EOF
.import/
build/
export/
*.tmp
EOF
```

#### Limit Concurrent Tasks
```bash
# Check current tasks
curl http://localhost:1420/api/operator/tasks

# Stop unnecessary tasks
curl -X POST http://localhost:1420/api/operator/stop \
  -d '{"task_id":"task_123"}'
```

#### Clear Cache
```bash
# Clear application cache
rm -rf ~/.config/hermes-operator/cache/*

# Restart application
npm run tauri dev
```

#### Monitor Resources
```bash
# Check memory usage
top -p $(pgrep -f hermes-operator)

# Check CPU usage
ps aux | grep hermes-operator
```

---

### Issue 5: Rust Compilation Errors

**Symptoms**:
- cargo check fails
- Missing dependencies
- Linker errors

**Possible Causes**:
1. Rust toolchain not installed
2. Missing Visual Studio Build Tools
3. PATH conflicts
4. Library path issues

**Solutions**:

#### Install Rust
```bash
# Download and install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

#### Install Visual Studio Build Tools
1. Download Visual Studio Build Tools
2. Install "Desktop development with C++"
3. Restart computer

#### Fix PATH
```bash
# Use Developer Command Prompt
# Start Menu → Visual Studio 2019 → Developer Command Prompt

# Or set PATH manually
export PATH="/c/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/14.29.30133/bin/Hostx64/x64:$PATH"
```

#### Set Library Path
```bash
# Set LIB environment variable
export LIB="C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\ucrt\x64"

# Set INCLUDE environment variable
export INCLUDE="C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.19041.0\um"
```

---

## Debugging

### Enable Debug Logging

```bash
# Set log level
export HERMES_LOG_LEVEL=debug

# Start application
npm run tauri dev

# View logs
tail -f ~/.config/hermes-operator/logs/app.log
```

### Check Event Stream

```bash
# Get recent events
curl http://localhost:1420/api/operator/events?task_id=task_123&limit=100

# Filter by type
curl http://localhost:1420/api/operator/events?task_id=task_123&type=error
```

### Inspect Database

```bash
# Open database
sqlite3 ~/.config/hermes-operator/data.db

# Check tasks
SELECT * FROM tasks ORDER BY created_at DESC LIMIT 10;

# Check events
SELECT * FROM events WHERE task_id = 'task_123' ORDER BY timestamp DESC LIMIT 20;

# Check approvals
SELECT * FROM approvals WHERE task_id = 'task_123';
```

### Test API Endpoints

```bash
# Health check
curl http://localhost:1420/health

# List tasks
curl http://localhost:1420/api/operator/tasks

# Get task details
curl http://localhost:1420/api/operator/tasks/task_123

# Get task events
curl http://localhost:1420/api/operator/tasks/task_123/events
```

---

## Getting Help

### Documentation

- [User Manual](USER-MANUAL.md)
- [FAQ](FAQ.md)
- [API Reference](API.md)
- [Troubleshooting](TROUBLESHOOTING.md)

### Community

- **GitHub Issues**: https://github.com/yourusername/acp-ui/issues
- **Discord**: https://discord.gg/hermes
- **Email**: support@example.com

### Providing Information

When reporting issues, include:

1. **Version**: `hermes-operator --version`
2. **OS**: Windows/macOS/Linux version
3. **Logs**: `~/.config/hermes-operator/logs/app.log`
4. **Steps to reproduce**: Detailed steps
5. **Expected behavior**: What should happen
6. **Actual behavior**: What actually happens
7. **Screenshots**: If applicable

---

## Quick Fixes

### Application Won't Start

```bash
# Kill all instances
pkill -9 -f hermes-operator

# Clear cache
rm -rf ~/.config/hermes-operator/cache

# Restart
npm run tauri dev
```

### Task Stuck

```bash
# Stop task
curl -X POST http://localhost:1420/api/operator/stop \
  -d '{"task_id":"task_123"}'

# Wait for cleanup
sleep 5

# Verify stopped
curl http://localhost:1420/api/operator/tasks/task_123
```

### Approval Timeout

```bash
# Check pending approvals
curl http://localhost:1420/api/operator/approvals?task_id=task_123

# Approve or reject
curl -X POST http://localhost:1420/api/operator/approve \
  -d '{"task_id":"task_123","approval_id":"appr_001","decision":"approve"}'
```

### Configuration Reset

```bash
# Backup configuration
cp -r ~/.config/hermes-operator ~/.config/hermes-operator.backup

# Reset configuration
rm -rf ~/.config/hermes-operator/config.json

# Restart application
npm run tauri dev
```

---

## Prevention

### Regular Maintenance

1. **Update Dependencies**: Weekly
   ```bash
   npm update
   cargo update
   ```

2. **Clear Cache**: Monthly
   ```bash
   rm -rf ~/.config/hermes-operator/cache/*
   ```

3. **Backup Data**: Daily
   ```bash
   ./backup.sh
   ```

4. **Monitor Logs**: Daily
   ```bash
   tail -100 ~/.config/hermes-operator/logs/app.log
   ```

### Best Practices

1. **Use Small Goals**: Break complex tasks into smaller ones
2. **Test Frequently**: Test after each modification
3. **Version Control**: Use Git for all projects
4. **Backup Regularly**: Backup before major changes
5. **Monitor Resources**: Check memory and CPU usage

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
