# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with Hermes Game Operator.

## Quick Diagnosis

### Task Won't Start

**Symptoms**: Error message when clicking "Start Task"

**Check**:
1. ✅ Is the project path valid?
2. ✅ Does `project.godot` exist?
3. ✅ Do you have write permissions?
4. ✅ Is Hermes Agent configured?

**Solutions**:
```bash
# Verify project structure
ls -la /path/to/project/project.godot

# Check permissions
ls -ld /path/to/project

# Test Hermes Agent
curl -X POST https://api.anthropic.com/v1/messages \
  -H "x-api-key: YOUR_API_KEY" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-5-sonnet-20241022","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'
```

### Analysis Fails

**Symptoms**: "Project analysis failed" error

**Check**:
1. ✅ Is the project structure valid?
2. ✅ Are there too many files?
3. ✅ Is the Godot version supported?

**Solutions**:
- Simplify project structure
- Exclude unnecessary directories
- Update to Godot 4.x

### Plan Generation Fails

**Symptoms**: "Failed to generate plan" error

**Check**:
1. ✅ Is Hermes Agent API accessible?
2. ✅ Is API key valid?
3. ✅ Are you within rate limits?

**Solutions**:
- Check internet connection
- Verify API key in settings
- Wait for rate limit reset
- Try a simpler goal

## Common Errors

### Error 1001: Invalid Path

**Cause**: File path is invalid or inaccessible

**Solution**:
```bash
# Check path exists
test -d /path/to/project && echo "Valid"

# Normalize path (remove .., ., etc.)
realpath /path/to/project
```

### Error 2001: Path Outside Boundary

**Cause**: Attempting to access files outside project directory

**Solution**:
- Ensure all file paths are within project
- Don't use absolute paths to system directories
- Check for symlink attacks

### Error 2002: Forbidden Operation

**Cause**: Attempting a forbidden operation

**Solution**:
- Review operation details
- Check approval level requirements
- Adjust security policy if needed

### Error 3001: API Unavailable

**Cause**: Hermes Agent API not accessible

**Solution**:
```bash
# Test API connectivity
ping api.anthropic.com

# Check DNS resolution
nslookup api.anthropic.com

# Test with curl
curl -I https://api.anthropic.com
```

### Error 3002: Network Timeout

**Cause**: Network request timed out

**Solution**:
- Check internet connection
- Increase timeout in settings
- Try again later
- Use a different network

### Error 4001: File Not Found

**Cause**: File does not exist

**Solution**:
```bash
# Check file exists
test -f /path/to/file.gd && echo "Exists"

# List directory contents
ls -la /path/to/directory/
```

### Error 4002: File Read Error

**Cause**: Cannot read file

**Solution**:
```bash
# Check file permissions
ls -l /path/to/file.gd

# Test read access
cat /path/to/file.gd > /dev/null
```

### Error 4003: File Write Error

**Cause**: Cannot write to file

**Solution**:
```bash
# Check write permissions
touch /path/to/directory/test.txt && rm /path/to/directory/test.txt

# Check disk space
df -h /path/to/directory
```

### Error 5001: Agent Not Initialized

**Cause**: Hermes Agent not properly initialized

**Solution**:
- Restart ACP UI
- Check API key configuration
- Verify agent settings

### Error 5002: Execution Failed

**Cause**: Agent execution encountered an error

**Solution**:
- Review error details
- Check logs for more information
- Try a simpler task
- Report bug if persistent

### Error 6001: Invalid State Transition

**Cause**: Attempting invalid state change

**Solution**:
- Check current task state
- Follow valid state transitions
- Restart task if needed

### Error 7001: Approval Not Found

**Cause**: Approval request not found

**Solution**:
- Refresh approval list
- Check if already approved
- Restart task if needed

## Debugging

### Enable Debug Logging

**Settings** → **Advanced** → **Log Level** → **Debug**

**Log Locations**:
- Windows: `%APPDATA%\acp-ui\logs\`
- macOS: `~/Library/Application Support/acp-ui/logs/`
- Linux: `~/.config/acp-ui/logs/`

### View Event Stream

1. Open task details
2. Click "Event Stream" tab
3. Review chronological events
4. Expand events for details

### Check Task State

```typescript
// In browser console
const task = await window.__TAURI__.invoke('operator_get_task', {
  taskId: 'your-task-id'
});
console.log(task);
```

### Test File Operations

```typescript
// Test file read
const result = await window.__TAURI__.invoke('operator_file_read', {
  path: '/path/to/file.gd',
  allowedRoots: ['/path/to/project']
});
console.log(result);
```

## Performance Issues

### Slow Analysis

**Symptoms**: Analysis takes > 30 seconds

**Solutions**:
1. **Exclude Directories**: Add `.godotignore` file
2. **Reduce Scope**: Analyze specific subdirectories
3. **Upgrade Hardware**: More RAM/CPU
4. **Close Applications**: Free up resources

**Example `.godotignore`**:
```
build/
export/
*.import
.tmp/
```

### High Memory Usage

**Symptoms**: Application uses > 1 GB RAM

**Solutions**:
1. **Limit Concurrent Tasks**: Settings → Max Tasks
2. **Reduce Event Buffer**: Settings → Event Retention
3. **Restart Application**: Clear memory
4. **Upgrade RAM**: More memory

### Slow Code Generation

**Symptoms**: Code generation takes > 10 seconds

**Solutions**:
1. **Check Internet**: Faster connection
2. **Simplify Goal**: Break into smaller tasks
3. **Use Cache**: Enable response caching
4. **Alternative Model**: Try different AI model

## Security Issues

### Path Traversal Attempt

**Symptoms**: "Path outside boundary" error

**Diagnosis**:
```bash
# Check for path traversal
echo "/path/to/project/../../../etc/passwd" | grep -c "\.\."
```

**Solution**:
- Use canonical paths
- Validate all inputs
- Report suspicious activity

### Command Injection Attempt

**Symptoms**: "Command not allowed" error

**Diagnosis**:
```bash
# Check for command injection
echo "ls; rm -rf /" | grep -c ";"
```

**Solution**:
- Use parameterized commands
- Validate command syntax
- Report suspicious activity

### Unauthorized Access

**Symptoms**: "Permission denied" error

**Diagnosis**:
```bash
# Check file permissions
ls -l /path/to/file
```

**Solution**:
- Verify user permissions
- Check group membership
- Adjust file permissions

## Network Issues

### Cannot Connect to API

**Symptoms**: "API unavailable" error

**Diagnosis**:
```bash
# Test DNS
nslookup api.anthropic.com

# Test connectivity
ping api.anthropic.com

# Test HTTPS
curl -I https://api.anthropic.com
```

**Solutions**:
- Check firewall settings
- Verify proxy configuration
- Try different network
- Contact ISP if needed

### WebSocket Connection Fails

**Symptoms**: "WebSocket connection failed" error

**Diagnosis**:
```bash
# Test WebSocket endpoint
wscat -c wss://your-endpoint.com
```

**Solutions**:
- Check WebSocket URL
- Verify SSL certificate
- Check firewall rules
- Test with different client

## Recovery

### Restore from Backup

**Location**: Same directory as original file with `.bak` extension

**Process**:
```bash
# List backups
ls -la /path/to/project/scripts/*.bak

# Restore backup
cp /path/to/file.gd.bak /path/to/file.gd
```

### Recover Failed Task

**Process**:
1. Check event stream for last successful step
2. Restore files from backups
3. Restart task with same goal
4. Monitor for same failure point
5. Adjust goal if needed

### Reset Operator State

**Process**:
```bash
# Clear task history
rm -rf ~/.config/acp-ui/tasks/

# Clear event cache
rm -rf ~/.config/acp-ui/events/

# Reset configuration
rm ~/.config/acp-ui/config.json

# Restart application
```

## Getting Help

### Before Asking

1. **Check This Guide**: Most issues are covered here
2. **Search Issues**: Check GitHub Issues
3. **Review Logs**: Check debug logs
4. **Test Fresh**: Try with minimal project

### Providing Information

When asking for help, include:
- **Version**: ACP UI version
- **OS**: Operating system and version
- **Godot**: Godot version
- **Error**: Complete error message
- **Logs**: Relevant log excerpts
- **Steps**: Steps to reproduce

### Support Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and discussions
- **Discord**: Real-time chat support
- **Email**: support@example.com

---

## Quick Reference

### Common Commands

```bash
# Test project structure
ls -la /path/to/project/

# Check file permissions
ls -l /path/to/file

# Test API connectivity
curl -I https://api.anthropic.com

# View logs
tail -f ~/.config/acp-ui/logs/app.log

# Clear cache
rm -rf ~/.config/acp-ui/cache/
```

### Environment Variables

```bash
# API Key
export ANTHROPIC_API_KEY="your-key-here"

# Log Level
export ACP_LOG_LEVEL="debug"

# API Endpoint
export ACP_API_ENDPOINT="https://api.anthropic.com"
```

### Configuration Files

- **Config**: `~/.config/acp-ui/config.json`
- **Agents**: `~/.config/acp-ui/agents.json`
- **Logs**: `~/.config/acp-ui/logs/`
- **Cache**: `~/.config/acp-ui/cache/`

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
