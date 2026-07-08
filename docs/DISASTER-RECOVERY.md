# Disaster Recovery Guide

## Overview

This guide covers disaster recovery procedures for Hermes Game Operator.

---

## Disaster Scenarios

### Scenario 1: Data Loss

**Trigger**: Database corruption, accidental deletion  
**Impact**: Loss of task history, events, approvals  
**Recovery Time**: 1-4 hours  

### Scenario 2: Service Outage

**Trigger**: Server crash, network failure  
**Impact**: Application unavailable  
**Recovery Time**: 15-30 minutes  

### Scenario 3: Security Breach

**Trigger**: Unauthorized access, data leak  
**Impact**: Compromised data, reputation damage  
**Recovery Time**: 24-48 hours  

### Scenario 4: Performance Degradation

**Trigger**: Resource exhaustion, traffic spike  
**Impact**: Slow response, timeouts  
**Recovery Time**: 30-60 minutes  

---

## Recovery Procedures

### Procedure 1: Data Recovery

#### Step 1: Assess Damage

```bash
# Check database status
sqlite3 ~/.config/hermes-operator/data.db "PRAGMA integrity_check;"

# Check backup availability
ls -lh ~/.config/hermes-operator/backups/
```

#### Step 2: Stop Services

```bash
# Stop application
pkill -f hermes-operator

# Stop background processes
pkill -f operator-worker
```

#### Step 3: Restore from Backup

```bash
# Identify latest backup
LATEST_BACKUP=$(ls -t ~/.config/hermes-operator/backups/*.db | head -1)

# Restore database
cp "$LATEST_BACKUP" ~/.config/hermes-operator/data.db

# Verify restoration
sqlite3 ~/.config/hermes-operator/data.db "SELECT COUNT(*) FROM tasks;"
```

#### Step 4: Restart Services

```bash
# Start application
npm run tauri dev

# Verify functionality
curl http://localhost:1420/health
```

#### Step 5: Verify Data

```bash
# Check task count
sqlite3 ~/.config/hermes-operator/data.db "SELECT COUNT(*) FROM tasks;"

# Check event count
sqlite3 ~/.config/hermes-operator/data.db "SELECT COUNT(*) FROM events;"

# Check recent tasks
sqlite3 ~/.config/hermes-operator/data.db "SELECT * FROM tasks ORDER BY created_at DESC LIMIT 5;"
```

---

### Procedure 2: Service Recovery

#### Step 1: Identify Issue

```bash
# Check process status
ps aux | grep hermes-operator

# Check logs
tail -100 ~/.config/hermes-operator/logs/app.log

# Check port usage
netstat -tlnp | grep 1420
```

#### Step 2: Kill Stuck Processes

```bash
# Find process ID
PID=$(ps aux | grep hermes-operator | grep -v grep | awk '{print $2}')

# Kill process
kill -9 $PID
```

#### Step 3: Clear Cache

```bash
# Clear application cache
rm -rf ~/.config/hermes-operator/cache/*

# Clear temp files
rm -rf /tmp/hermes-operator-*
```

#### Step 4: Restart Service

```bash
# Start application
npm run tauri dev

# Monitor startup
tail -f ~/.config/hermes-operator/logs/app.log
```

#### Step 5: Verify Service

```bash
# Health check
curl http://localhost:1420/health

# API test
curl http://localhost:1420/api/tasks
```

---

### Procedure 3: Security Incident Response

#### Step 1: Contain Breach

```bash
# Stop all services
pkill -f hermes-operator

# Revoke API keys
sqlite3 ~/.config/hermes-operator/data.db "UPDATE api_keys SET revoked = 1;"

# Block suspicious IPs
echo "192.168.1.100" >> ~/.config/hermes-operator/blocked_ips.txt
```

#### Step 2: Assess Damage

```bash
# Check for unauthorized access
sqlite3 ~/.config/hermes-operator/data.db "SELECT * FROM access_logs WHERE timestamp > datetime('now', '-1 day');"

# Check for data exfiltration
sqlite3 ~/.config/hermes-operator/data.db "SELECT * FROM export_logs WHERE timestamp > datetime('now', '-1 day');"

# Check for privilege escalation
sqlite3 ~/.config/hermes-operator/data.db "SELECT * FROM permission_changes WHERE timestamp > datetime('now', '-1 day');"
```

#### Step 3: Rotate Credentials

```bash
# Generate new API key
NEW_KEY=$(openssl rand -hex 32)

# Update configuration
sed -i "s/api_key=.*/api_key=$NEW_KEY/" ~/.config/hermes-operator/config.json

# Restart services
npm run tauri dev
```

#### Step 4: Notify Stakeholders

```bash
# Send notification
echo "Security incident detected. Services restored." | mail -s "Security Alert" security@example.com

# Update status page
curl -X POST https://status.example.com/incidents \
  -H "Authorization: Bearer $STATUS_TOKEN" \
  -d '{"title":"Security Incident","status":"resolved"}'
```

#### Step 5: Document Incident

```bash
# Create incident report
cat > /tmp/incident-$(date +%Y%m%d).md << EOF
# Security Incident Report

**Date**: $(date)
**Severity**: High
**Duration**: 2 hours
**Impact**: Unauthorized access detected

## Timeline
- $(date): Incident detected
- $(date): Services stopped
- $(date): Breach contained
- $(date): Credentials rotated
- $(date): Services restored

## Root Cause
[To be determined]

## Remediation
- Revoked compromised credentials
- Blocked suspicious IPs
- Enhanced monitoring

## Lessons Learned
[To be documented]
EOF
```

---

### Procedure 4: Performance Recovery

#### Step 1: Identify Bottleneck

```bash
# Check CPU usage
top -b -n 1 | head -20

# Check memory usage
free -h

# Check disk I/O
iostat -x 1 5

# Check network
netstat -s
```

#### Step 2: Optimize Resources

```bash
# Clear cache
rm -rf ~/.config/hermes-operator/cache/*

# Compact database
sqlite3 ~/.config/hermes-operator/data.db "VACUUM;"

# Restart services
pkill -f hermes-operator
npm run tauri dev
```

#### Step 3: Scale Resources

```bash
# Increase memory limit
echo "memory_limit=2G" >> ~/.config/hermes-operator/config.json

# Increase connection pool
echo "max_connections=100" >> ~/.config/hermes-operator/config.json

# Restart with new config
npm run tauri dev
```

#### Step 4: Monitor Recovery

```bash
# Monitor performance
watch -n 5 'curl http://localhost:1420/health'

# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:1420/api/tasks
```

---

## Backup Strategy

### Backup Schedule

| Frequency | Type | Retention | Location |
|-----------|------|-----------|----------|
| Hourly | Incremental | 24 hours | Local |
| Daily | Full | 7 days | Local + Remote |
| Weekly | Full | 30 days | Remote |
| Monthly | Full | 1 year | Archive |

### Backup Commands

#### Manual Backup

```bash
# Backup database
sqlite3 ~/.config/hermes-operator/data.db ".backup '/tmp/hermes-backup-$(date +%Y%m%d-%H%M%S).db'"

# Backup configuration
cp -r ~/.config/hermes-operator /tmp/hermes-config-backup-$(date +%Y%m%d)

# Compress backup
tar -czf hermes-backup-$(date +%Y%m%d).tar.gz /tmp/hermes-backup-* /tmp/hermes-config-backup-*
```

#### Automated Backup

**backup.sh**:
```bash
#!/bin/bash

BACKUP_DIR="/backups/hermes-operator"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup database
sqlite3 ~/.config/hermes-operator/data.db ".backup '$BACKUP_DIR/database-$TIMESTAMP.db'"

# Backup configuration
cp -r ~/.config/hermes-operator "$BACKUP_DIR/config-$TIMESTAMP"

# Compress
tar -czf "$BACKUP_DIR/backup-$TIMESTAMP.tar.gz" "$BACKUP_DIR/database-$TIMESTAMP.db" "$BACKUP_DIR/config-$TIMESTAMP"

# Cleanup
rm -rf "$BACKUP_DIR/database-$TIMESTAMP.db" "$BACKUP_DIR/config-$TIMESTAMP"

# Upload to remote
scp "$BACKUP_DIR/backup-$TIMESTAMP.tar.gz" remote:/backups/

# Delete old backups
find "$BACKUP_DIR" -name "backup-*.tar.gz" -mtime +7 -delete

echo "Backup completed: backup-$TIMESTAMP.tar.gz"
```

**Cron Job**:
```bash
# Run daily at 2 AM
0 2 * * * /path/to/backup.sh
```

---

## Recovery Testing

### Test Schedule

- **Monthly**: Backup restoration test
- **Quarterly**: Full disaster recovery drill
- **Annually**: Complete failover test

### Test Procedures

#### Monthly Test

```bash
# Restore backup to test environment
./restore-test.sh

# Verify data integrity
sqlite3 test.db "PRAGMA integrity_check;"

# Verify application functionality
curl http://localhost:1420/health
```

#### Quarterly Drill

```bash
# Simulate disaster
./simulate-disaster.sh

# Execute recovery procedures
./disaster-recovery.sh

# Verify recovery
./verify-recovery.sh

# Document results
./document-results.sh
```

---

## Communication Plan

### Internal Communication

| Role | Contact | Method | Timing |
|------|---------|--------|--------|
| Engineering Lead | lead@example.com | Email | Immediate |
| Operations Lead | ops@example.com | Email | Immediate |
| Security Lead | security@example.com | Email | Immediate |
| Management | mgmt@example.com | Email | < 1 hour |

### External Communication

| Audience | Channel | Template | Timing |
|----------|---------|----------|--------|
| Users | Status Page | Status Update | < 15 min |
| Customers | Email | Incident Notice | < 1 hour |
| Partners | Email | Partner Notice | < 2 hours |
| Public | Social Media | Public Statement | < 4 hours |

### Communication Templates

#### Initial Notice

```
Subject: Service Disruption - [Date]

We are experiencing a service disruption affecting Hermes Game Operator.

Impact: [Description]
Estimated Recovery: [Time]

We are working to resolve this issue and will provide updates every 30 minutes.

Status Page: https://status.example.com

Thank you for your patience.
```

#### Resolution Notice

```
Subject: Service Restored - [Date]

The service disruption has been resolved.

Impact: [Description]
Duration: [Time]
Root Cause: [Brief description]

We apologize for any inconvenience. A detailed post-mortem will be published within 48 hours.

Status Page: https://status.example.com

Thank you for your patience.
```

---

## Post-Incident Review

### Review Checklist

- [ ] Incident timeline documented
- [ ] Root cause identified
- [ ] Contributing factors analyzed
- [ ] Impact assessed
- [ ] Response effectiveness evaluated
- [ ] Communication effectiveness evaluated
- [ ] Lessons learned documented
- [ ] Action items created
- [ ] Improvements planned

### Post-Mortem Template

```markdown
# Post-Mortem: [Incident Name]

**Date**: [Date]
**Severity**: [P0/P1/P2/P3]
**Duration**: [Time]
**Impact**: [Description]

## Summary
[Brief summary of incident]

## Timeline
- HH:MM - [Event]
- HH:MM - [Event]

## Root Cause
[Detailed root cause analysis]

## Contributing Factors
- Factor 1
- Factor 2

## Impact
- Users affected: [Number]
- Data loss: [Description]
- Revenue impact: [Amount]

## Response
- Detection: [How detected]
- Response time: [Time]
- Resolution time: [Time]

## What Went Well
- [Positive aspect]

## What Went Wrong
- [Negative aspect]

## Lessons Learned
- [Lesson]

## Action Items
- [ ] Action 1 (Owner, Due date)
- [ ] Action 2 (Owner, Due date)

## Preventive Measures
- [Measure 1]
- [Measure 2]
```

---

## Resources

- [Incident Response Guide](INCIDENT-RESPONSE.md)
- [Maintenance Guide](MAINTENANCE-GUIDE.md)
- [Security Policy](SECURITY.md)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
