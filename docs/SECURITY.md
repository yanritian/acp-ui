# Security Guide

This guide covers security best practices for Hermes Game Operator.

## Security Architecture

### Defense in Depth

Hermes Game Operator uses multiple layers of security:

1. **Path Validation** - PathGuard prevents unauthorized file access
2. **Command Filtering** - CommandGuard blocks dangerous commands
3. **Approval System** - User approval for dangerous operations
4. **Event Tracking** - Complete audit trail
5. **File Backup** - Automatic backups before modifications

### Threat Model

**Protected Against**:
- ✅ Path traversal attacks
- ✅ Command injection
- ✅ Unauthorized file access
- ✅ Malicious command execution
- ✅ Data exfiltration
- ✅ Privilege escalation

**Not Protected Against**:
- ❌ Compromised operator process
- ❌ Malicious project files (requires manual review)
- ❌ API key theft (user responsibility)
- ❌ Physical access attacks

## PathGuard

### How It Works

PathGuard validates all file operations:

```rust
// Validate path
let canonical = path.canonicalize()?;

// Check against allowed roots
for root in &self.allowed_roots {
    if canonical.starts_with(root) {
        return Ok(());
    }
}

Err(PathGuardError::PathOutsideBoundary)
```

### Configuration

**Settings** → **Security** → **PathGuard**:

- **Allowed Roots**: Project directories
- **Forbidden Directories**: .git, node_modules, target
- **Follow Symlinks**: Disabled by default
- **Canonicalize Paths**: Always enabled

### Best Practices

1. **Use Project-Relative Paths**
   ```typescript
   // Good
   const path = "./scripts/Player.gd";
   
   // Bad
   const path = "/etc/passwd";
   ```

2. **Validate Before Use**
   ```typescript
   const result = await OperatorApi.fileRead(path, allowedRoots);
   ```

3. **Check for Traversal**
   ```typescript
   if (path.includes("..")) {
     throw new Error("Path traversal detected");
   }
   ```

### Security Checks

PathGuard prevents:
- Access to system files
- Directory traversal attacks
- Symlink attacks
- Access outside project

## CommandGuard

### How It Works

CommandGuard filters shell commands:

```rust
// Check if command is allowed
if !self.allowed_commands.contains(&base_command) {
    return Err(CommandGuardError::UnknownCommand);
}

// Check if command is forbidden
if self.forbidden_commands.contains(&base_command) {
    return Err(CommandGuardError::ForbiddenCommand);
}
```

### Allowed Commands

**Default Allowlist**:
- `godot` - Godot engine commands
- `ls` / `dir` - List files
- `cat` / `type` - Read files
- `find` / `grep` - Search files

### Forbidden Commands

**Always Blocked**:
- `rm` / `del` - Delete files
- `rmdir` - Remove directories
- `format` - Format drives
- `mkfs` - Create filesystems
- `dd` - Low-level disk operations
- `sudo` / `su` - Privilege escalation
- `powershell` / `cmd` - Shell execution

### Best Practices

1. **Review Command Details**
   ```typescript
   const validated = await CommandGuard.validateCommand(command);
   console.log(validated);
   ```

2. **Use Whitelist**
   ```typescript
   // Only allow specific commands
   const allowed = ["godot", "ls", "cat"];
   ```

3. **Log All Commands**
   ```typescript
   console.log(`Executing: ${command}`);
   ```

### Security Checks

CommandGuard prevents:
- Dangerous command execution
- Privilege escalation
- System modification
- Data destruction

## Approval System

### Approval Levels

**Level 0 (Silent)**:
- Safe read operations
- No user notification needed
- Examples: file.read, project.analyze

**Level 1 (Notify)**:
- Can execute automatically
- Show in progress stream
- Examples: file.list, search

**Level 2 (Approve)**:
- Requires user approval
- Show diff/preview
- Examples: file.patch, shell.command

**Level 3 (Forbidden)**:
- Always blocked
- Requires policy change
- Examples: file.delete, system.access

### Approval Workflow

1. **Request Generated**
   ```typescript
   const request = {
     approval_id: "appr_001",
     level: "approve",
     action: "file.patch",
     preview: { files: ["Player.gd"] }
   };
   ```

2. **User Reviews**
   - Check action details
   - Review file diffs
   - Assess risk level

3. **User Decides**
   ```typescript
   await OperatorApi.approve({
     task_id: "task_123",
     approval_id: "appr_001",
     decision: "approve"
   });
   ```

### Best Practices

1. **Always Review Diffs**
   ```typescript
   const diff = await OperatorApi.filePatchPreview(path, newContent);
   console.log(diff);
   ```

2. **Check Risk Level**
   - Level 0-1: Generally safe
   - Level 2: Review carefully
   - Level 3: Avoid unless necessary

3. **Respond Promptly**
   - Don't let approvals timeout
   - Ask for more time if needed

### Security Checks

Approval system prevents:
- Unauthorized modifications
- Dangerous operations
- Data loss
- Security violations

## File Backup

### Automatic Backups

Before any file modification:
1. Original file copied to `.bak`
2. Backup stored in same directory
3. Backup retained until task completion

### Backup Process

```rust
// Create backup
let backup_path = path.with_extension("bak");
fs::copy(&path, &backup_path)?;

// Modify file
fs::write(&path, new_content)?;
```

### Restore Process

```bash
# Restore from backup
cp file.gd.bak file.gd
```

### Best Practices

1. **Keep Backups**
   - Don't delete .bak files manually
   - Let operator manage lifecycle

2. **Test Restores**
   ```bash
   # Verify backup is valid
   diff file.gd file.gd.bak
   ```

3. **Use Version Control**
   ```bash
   git add .
   git commit -m "Backup before operator task"
   ```

### Security Checks

File backup prevents:
- Data loss
- Accidental overwrites
- Unrecoverable changes

## Event Tracking

### Audit Trail

Every operation is logged:

```json
{
  "event_id": "evt_001",
  "task_id": "task_123",
  "timestamp": "2026-07-08T10:30:00Z",
  "type": "file_modified",
  "payload": {
    "path": "scripts/Player.gd",
    "lines_changed": 5
  }
}
```

### Event Types

**Security-Relevant Events**:
- `file_read` - File accessed
- `file_modified` - File changed
- `file_backed_up` - Backup created
- `command_executed` - Command run
- `approval_requested` - Approval needed
- `approval_granted` - Approval given
- `approval_rejected` - Approval denied

### Best Practices

1. **Review Event Stream**
   - Check for suspicious activity
   - Verify all operations expected
   - Monitor approval requests

2. **Export Logs**
   ```typescript
   const events = await OperatorApi.listEvents(taskId);
   fs.writeFileSync("audit.log", JSON.stringify(events));
   ```

3. **Set Alerts**
   ```typescript
   events.forEach(event => {
     if (event.type === "file_modified") {
       alert(`File modified: ${event.payload.path}`);
     }
   });
   ```

### Security Checks

Event tracking provides:
- Complete audit trail
- Incident investigation
- Compliance reporting
- Security monitoring

## API Security

### API Key Management

**Storage**:
- Use system keychain
- Never log API keys
- Rotate keys regularly

**Configuration**:
```bash
# Environment variable
export ANTHROPIC_API_KEY="your-key-here"

# Or in settings UI
Settings → API Keys → Add New
```

### Best Practices

1. **Never Commit Keys**
   ```bash
   # Add to .gitignore
   echo ".env" >> .gitignore
   echo "*.key" >> .gitignore
   ```

2. **Rotate Regularly**
   - Monthly rotation recommended
   - Immediately if compromised

3. **Use Least Privilege**
   - Only grant necessary permissions
   - Use read-only keys where possible

### Security Checks

API security prevents:
- Unauthorized API access
- Key theft
- Usage abuse
- Cost overruns

## Network Security

### HTTPS Only

All API calls use HTTPS:
```typescript
// Good
const endpoint = "https://api.anthropic.com";

// Bad (will fail)
const endpoint = "http://api.anthropic.com";
```

### WebSocket Security

WebSocket connections use WSS:
```typescript
// Good
const ws = new WebSocket("wss://your-endpoint.com");

// Bad (will fail)
const ws = new WebSocket("ws://your-endpoint.com");
```

### Best Practices

1. **Verify Certificates**
   - Don't disable certificate validation
   - Check certificate validity

2. **Use Secure Protocols**
   - TLS 1.2 or higher
   - Strong cipher suites

3. **Monitor Connections**
   ```typescript
   ws.onclose = (event) => {
     console.log(`Connection closed: ${event.code}`);
   };
   ```

### Security Checks

Network security prevents:
- Man-in-the-middle attacks
- Eavesdropping
- Data tampering
- Connection hijacking

## Access Control

### User Authentication

**Local Application**:
- No authentication required
- Relies on OS user permissions

**Remote Access**:
- Requires API key
- Optional: OAuth2 tokens

### Best Practices

1. **Limit Access**
   - Only authorized users
   - Use role-based access

2. **Monitor Access**
   ```typescript
   console.log(`User ${userId} accessed ${resource}`);
   ```

3. **Revoke Access**
   ```typescript
   await OperatorApi.revokeAccess(userId);
   ```

### Security Checks

Access control prevents:
- Unauthorized access
- Privilege escalation
- Data breaches

## Incident Response

### Security Incident

If you suspect a security incident:

1. **Stop Operations**
   ```typescript
   await OperatorApi.stopAllTasks();
   ```

2. **Collect Evidence**
   ```typescript
   const events = await OperatorApi.listEvents(taskId);
   fs.writeFileSync("incident.log", JSON.stringify(events));
   ```

3. **Assess Impact**
   - What data was accessed?
   - What files were modified?
   - What commands were run?

4. **Contain Incident**
   - Revoke API keys
   - Change passwords
   - Isolate affected systems

5. **Report Incident**
   - Email: security@example.com
   - Include timeline
   - Include evidence

### Best Practices

1. **Have Response Plan**
   - Define roles
   - Define procedures
   - Test regularly

2. **Document Incidents**
   - Timeline
   - Impact
   - Response actions
   - Lessons learned

3. **Improve Security**
   - Fix vulnerabilities
   - Update procedures
   - Train users

### Security Checks

Incident response provides:
- Quick containment
- Damage mitigation
- Recovery planning
- Prevention improvement

## Compliance

### Data Protection

**GDPR Compliance**:
- No personal data stored
- API keys encrypted
- User consent required

**CCPA Compliance**:
- Data access rights
- Data deletion rights
- Opt-out options

### Best Practices

1. **Privacy by Design**
   - Minimize data collection
   - Encrypt sensitive data
   - Limit retention

2. **Transparency**
   - Clear privacy policy
   - User consent
   - Data access logs

3. **User Rights**
   ```typescript
   // User can request data
   const data = await OperatorApi.exportUserData(userId);
   
   // User can delete data
   await OperatorApi.deleteUserData(userId);
   ```

### Security Checks

Compliance ensures:
- Legal requirements met
- User privacy protected
- Data properly handled

## Security Checklist

### Before Deployment

- [ ] PathGuard enabled
- [ ] CommandGuard configured
- [ ] Approval system active
- [ ] File backup enabled
- [ ] Event tracking enabled
- [ ] API keys secured
- [ ] HTTPS enforced
- [ ] Access controls set
- [ ] Audit logging enabled
- [ ] Incident plan ready

### During Operation

- [ ] Monitor event stream
- [ ] Review approvals
- [ ] Check backups
- [ ] Verify logs
- [ ] Test restores
- [ ] Rotate keys
- [ ] Update policies

### After Incident

- [ ] Contain incident
- [ ] Collect evidence
- [ ] Assess impact
- [ ] Notify stakeholders
- [ ] Fix vulnerabilities
- [ ] Update procedures
- [ ] Document lessons

## Resources

- [User Manual](USER-MANUAL.md)
- [API Reference](api.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Best Practices](BEST-PRACTICES.md)
- [FAQ](FAQ.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
