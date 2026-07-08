# Known Issues

This document lists known issues in Hermes Game Operator v1.0.0.

## Critical Issues

### None

No critical issues currently known.

## High Priority

### 1. Rust Toolchain Required

**Severity**: High  
**Status**: Known  
**Workaround**: Install Rust manually

**Description**:
The operator requires a complete Rust toolchain to compile and run the backend. Without it, the application cannot be built or executed.

**Impact**:
- Cannot build from source
- Cannot run cargo check
- Limited development capability

**Workaround**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or on Windows
# Download from https://rustup.rs/

# Verify installation
rustc --version
cargo --version
```

**Expected Fix**: v1.0.1  
**Priority**: P0

---

### 2. API Key Required for Code Generation

**Severity**: High  
**Status**: Known  
**Workaround**: Use offline features only

**Description**:
Code generation features require an API key for Hermes Agent. Without it, only project analysis and planning work.

**Impact**:
- Cannot generate code
- Cannot modify files
- Limited to analysis and planning

**Workaround**:
```bash
# Set API key
export ANTHROPIC_API_KEY="your-key-here"

# Or configure in settings
Settings → API Keys → Add New
```

**Expected Fix**: Documentation update  
**Priority**: P0

---

## Medium Priority

### 3. Large Projects May Be Slow

**Severity**: Medium  
**Status**: Known  
**Workaround**: Use .godotignore

**Description**:
Analyzing large projects (500+ files) can take 30-60 seconds. Very large projects (1000+ files) may timeout.

**Impact**:
- Slow analysis
- Potential timeouts
- Poor user experience

**Workaround**:
```ini
# .godotignore
build/
export/
*.import
.tmp/
.cache/
assets/large/
```

**Expected Fix**: v1.1.0  
**Priority**: P1

---

### 4. WebSocket Connection Drops

**Severity**: Medium  
**Status**: Known  
**Workaround**: Restart connection

**Description**:
WebSocket connections may drop after 30 minutes of inactivity due to NAT/proxy timeouts.

**Impact**:
- Lost real-time updates
- Need to reconnect
- Inconsistent state

**Workaround**:
```typescript
// Reconnect on disconnect
ws.onclose = () => {
  setTimeout(() => {
    connect();
  }, 5000);
};
```

**Expected Fix**: v1.0.1  
**Priority**: P1

---

### 5. Approval Timeout

**Severity**: Medium  
**Status**: Known  
**Workaround**: Respond quickly

**Description**:
Approval requests timeout after 5 minutes if not responded to. This can interrupt task execution.

**Impact**:
- Task paused
- Need to restart
- Lost progress

**Workaround**:
- Monitor approval drawer
- Respond within timeout
- Adjust timeout in settings

**Expected Fix**: v1.1.0  
**Priority**: P1

---

## Low Priority

### 6. Limited Godot Version Support

**Severity**: Low  
**Status**: Known  
**Workaround**: Use Godot 4.x

**Description**:
Currently only Godot 4.x is fully supported. Godot 3.x projects may have compatibility issues.

**Impact**:
- Godot 3.x not fully supported
- Some features may not work
- Limited compatibility

**Workaround**:
- Upgrade to Godot 4.x
- Use compatible features only

**Expected Fix**: v1.2.0  
**Priority**: P2

---

### 7. Memory Usage High

**Severity**: Low  
**Status**: Known  
**Workaround**: Limit concurrent tasks

**Description**:
Memory usage can exceed 500MB with multiple concurrent tasks or large event histories.

**Impact**:
- High memory usage
- Potential crashes
- Slow performance

**Workaround**:
```typescript
// Limit concurrent tasks
Settings → Performance → Max Concurrent Tasks: 2

// Clear old events
Settings → Performance → Event Retention: 1000
```

**Expected Fix**: v1.1.0  
**Priority**: P2

---

### 8. UI Rendering Lag

**Severity**: Low  
**Status**: Known  
**Workaround**: Reduce event display

**Description**:
Displaying 1000+ events in the timeline can cause UI lag and slow rendering.

**Impact**:
- Slow UI
- Poor performance
- Bad user experience

**Workaround**:
```typescript
// Limit displayed events
const displayedEvents = events.slice(0, 100);
```

**Expected Fix**: v1.1.0  
**Priority**: P2

---

### 9. Backup File Accumulation

**Severity**: Low  
**Status**: Known  
**Workaround**: Manual cleanup

**Description**:
Backup files (.bak) accumulate over time and are not automatically cleaned up.

**Impact**:
- Disk space usage
- Cluttered directories
- Confusion

**Workaround**:
```bash
# Clean old backups
find . -name "*.bak" -mtime +7 -delete
```

**Expected Fix**: v1.1.0  
**Priority**: P2

---

### 10. Documentation Gaps

**Severity**: Low  
**Status**: Known  
**Workaround**: Check multiple sources

**Description**:
Some features lack comprehensive documentation or examples.

**Impact**:
- Confusion
- Learning curve
- Support requests

**Workaround**:
- Check all documentation
- Ask in community
- Report gaps

**Expected Fix**: Ongoing  
**Priority**: P2

---

## Resolved Issues

### None

No issues have been resolved in v1.0.0 (initial release).

## Reporting New Issues

To report a new issue:

1. **Check Known Issues**: Verify it's not already listed
2. **Gather Information**:
   - Version number
   - Operating system
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Logs/screenshots
3. **Submit Issue**:
   - GitHub Issues
   - Include all information
   - Label appropriately

## Issue Severity Levels

- **Critical**: Application crashes, data loss, security vulnerability
- **High**: Major feature broken, no workaround
- **Medium**: Feature impaired, workaround available
- **Low**: Minor issue, cosmetic, documentation

## Issue Status

- **Known**: Issue identified, workaround available
- **Investigating**: Issue reported, being investigated
- **Confirmed**: Issue confirmed, fix planned
- **In Progress**: Fix being developed
- **Resolved**: Fix released
- **Closed**: Issue closed

## Workaround Best Practices

### General

1. **Check Documentation**: Read relevant docs
2. **Search Issues**: Check GitHub Issues
3. **Ask Community**: Discord, forums
4. **Contact Support**: Email support

### Technical

1. **Restart Application**: Clear state
2. **Clear Cache**: Remove cached data
3. **Update Dependencies**: Use latest versions
4. **Check Logs**: Review error messages

### Configuration

1. **Reset Settings**: Use defaults
2. **Check Permissions**: Verify access
3. **Validate Paths**: Ensure correct
4. **Test Connectivity**: Verify network

## Tracking

Issues are tracked in:
- GitHub Issues: https://github.com/yourusername/acp-ui/issues
- Internal Tracker: [Link]
- Support Tickets: [Link]

## Metrics

**Issue Metrics**:
- Total known: 10
- Critical: 0
- High: 2
- Medium: 3
- Low: 5

**Resolution Rate**: 0% (initial release)  
**Average Resolution Time**: N/A

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
