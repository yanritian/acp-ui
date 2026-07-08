# Log Analysis Guide

## Overview

This guide covers log analysis and troubleshooting for Hermes Game Operator.

---

## Log Locations

### Application Logs

| Log Type | Location | Format |
|----------|----------|--------|
| Application | `~/.config/hermes-operator/logs/app.log` | JSON |
| Error | `~/.config/hermes-operator/logs/error.log` | JSON |
| Access | `~/.config/hermes-operator/logs/access.log` | JSON |
| Audit | `~/.config/hermes-operator/logs/audit.log` | JSON |

### Database Logs

| Log Type | Location | Format |
|----------|----------|--------|
| Query | SQLite query log | Text |
| Migration | `~/.config/hermes-operator/logs/migration.log` | Text |

---

## Log Format

### Application Log

```json
{
  "timestamp": "2026-07-08T10:30:00Z",
  "level": "info",
  "message": "Task started",
  "context": {
    "task_id": "task_123",
    "domain": "game.godot",
    "goal": "Add double jump"
  },
  "metadata": {
    "version": "1.0.0",
    "environment": "production"
  }
}
```

### Error Log

```json
{
  "timestamp": "2026-07-08T10:35:00Z",
  "level": "error",
  "message": "Failed to analyze project",
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "project.godot not found",
    "stack": "Error: project.godot not found\n    at analyzeProject..."
  },
  "context": {
    "task_id": "task_123",
    "project_path": "/path/to/project"
  }
}
```

---

## Log Levels

| Level | Description | Usage |
|-------|-------------|-------|
| **DEBUG** | Detailed debugging | Development only |
| **INFO** | General information | Normal operations |
| **WARN** | Warning messages | Potential issues |
| **ERROR** | Error messages | Failures |
| **CRITICAL** | Critical errors | System failures |

---

## Common Log Patterns

### Task Lifecycle

```bash
# Start task
grep "task_started" app.log

# Planning phase
grep "task_planning" app.log

# Execution phase
grep "task_running" app.log

# Completion
grep "task_completed" app.log
```

### Error Patterns

```bash
# Find all errors
grep '"level":"error"' error.log

# Find specific error
grep "PROJECT_NOT_FOUND" error.log

# Find errors by task
grep "task_123" error.log
```

### Performance Issues

```bash
# Find slow operations (> 5s)
grep '"duration_ms":[5-9][0-9][0-9][0-9]' app.log

# Find timeouts
grep "TIMEOUT" error.log
```

---

## Log Analysis Tools

### Command Line

#### View Recent Logs

```bash
# Last 100 lines
tail -100 ~/.config/hermes-operator/logs/app.log

# Follow logs
tail -f ~/.config/hermes-operator/logs/app.log

# Filter by level
grep '"level":"error"' ~/.config/hermes-operator/logs/error.log
```

#### Search Logs

```bash
# Search for task
grep "task_123" ~/.config/hermes-operator/logs/app.log

# Search by time range
grep "2026-07-08T10:" ~/.config/hermes-operator/logs/app.log

# Search for errors
grep -i "error\|fail" ~/.config/hermes-operator/logs/app.log
```

#### Count Events

```bash
# Count errors
grep -c '"level":"error"' ~/.config/hermes-operator/logs/error.log

# Count by task
grep -o '"task_id":"[^"]*"' ~/.config/hermes-operator/logs/app.log | sort | uniq -c

# Count by level
grep -o '"level":"[^"]*"' ~/.config/hermes-operator/logs/app.log | sort | uniq -c
```

### Advanced Tools

#### jq (JSON Processor)

```bash
# Parse JSON logs
cat app.log | jq .

# Filter by level
cat app.log | jq 'select(.level == "error")'

# Extract specific fields
cat app.log | jq '{timestamp, level, message}'

# Count errors
cat app.log | jq 'select(.level == "error")' | wc -l

# Find slow operations
cat app.log | jq 'select(.duration_ms > 5000)'
```

#### Logstash / Elasticsearch

```bash
# Send logs to Elasticsearch
cat app.log | logstash -f logstash.conf

# Query Elasticsearch
curl -X GET "localhost:9200/hermes-logs/_search" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "match": {
        "level": "error"
      }
    }
  }'
```

---

## Log Rotation

### Configuration

**logrotate.conf**:
```
~/.config/hermes-operator/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    missingok
    create 0644 hermes-operator hermes-operator
}
```

### Manual Rotation

```bash
# Rotate logs
logrotate -f /etc/logrotate.d/hermes-operator

# Compress old logs
gzip ~/.config/hermes-operator/logs/app.log.1

# Delete old logs
find ~/.config/hermes-operator/logs/ -name "*.log.*" -mtime +30 -delete
```

---

## Monitoring

### Real-time Monitoring

```bash
# Watch logs
watch -n 5 'tail -50 ~/.config/hermes-operator/logs/app.log'

# Monitor errors
watch -n 5 'grep -c "error" ~/.config/hermes-operator/logs/error.log'

# Monitor task count
watch -n 5 'grep -c "task_started" ~/.config/hermes-operator/logs/app.log'
```

### Alerting

```bash
# Alert on errors
tail -f error.log | while read line; do
  if echo "$line" | grep -q '"level":"error"'; then
    echo "ALERT: Error detected" | mail -s "Hermes Operator Alert" admin@example.com
  fi
done
```

---

## Log Analysis Queries

### Task Analysis

```sql
-- SQLite queries
-- Task count by status
SELECT status, COUNT(*) as count
FROM tasks
GROUP BY status;

-- Average task duration
SELECT AVG(duration_seconds) as avg_duration
FROM tasks
WHERE status = 'completed';

-- Tasks by date
SELECT DATE(created_at) as date, COUNT(*) as count
FROM tasks
GROUP BY DATE(created_at)
ORDER BY date DESC;
```

### Error Analysis

```sql
-- Error count by type
SELECT error_type, COUNT(*) as count
FROM errors
GROUP BY error_type
ORDER BY count DESC;

-- Recent errors
SELECT * FROM errors
ORDER BY timestamp DESC
LIMIT 10;

-- Errors by task
SELECT task_id, COUNT(*) as error_count
FROM errors
GROUP BY task_id
ORDER BY error_count DESC;
```

### Performance Analysis

```sql
-- Slow tasks
SELECT task_id, duration_seconds
FROM tasks
WHERE duration_seconds > 60
ORDER BY duration_seconds DESC;

-- Average response time by endpoint
SELECT endpoint, AVG(response_time_ms) as avg_response
FROM api_logs
GROUP BY endpoint
ORDER BY avg_response DESC;
```

---

## Debugging Techniques

### Enable Debug Logging

```bash
# Set log level
export HERMES_LOG_LEVEL=debug

# Start application
npm run tauri dev

# View debug logs
tail -f ~/.config/hermes-operator/logs/app.log
```

### Trace Request

```bash
# Find request ID
grep "request_id: req_123" app.log

# Trace entire request
grep "req_123" app.log | jq .
```

### Inspect Database

```bash
# Open database
sqlite3 ~/.config/hermes-operator/data.db

# Check tasks
SELECT * FROM tasks WHERE task_id = 'task_123';

# Check events
SELECT * FROM events WHERE task_id = 'task_123' ORDER BY timestamp;

# Check errors
SELECT * FROM errors WHERE task_id = 'task_123';
```

---

## Best Practices

### 1. Structured Logging

```typescript
// Good
logger.info({
  task_id: taskId,
  action: 'start',
  timestamp: new Date().toISOString()
}, 'Task started')

// Bad
logger.info(`Task ${taskId} started`)
```

### 2. Include Context

```typescript
// Good
logger.error({
  task_id: taskId,
  project_path: projectPath,
  error: error.message,
  stack: error.stack
}, 'Failed to analyze project')

// Bad
logger.error('Failed to analyze project')
```

### 3. Log Levels Appropriately

```typescript
// Good
logger.debug('Entering function analyzeProject')
logger.info('Task started', { task_id: taskId })
logger.warn('Project is large', { file_count: 1000 })
logger.error('Analysis failed', { error: error.message })

// Bad
logger.info('Entering function analyzeProject') // Too verbose
logger.error('Task started') // Wrong level
```

### 4. Avoid Sensitive Data

```typescript
// Good
logger.info('Task started', { task_id: taskId })

// Bad
logger.info('Task started', { task_id: taskId, api_key: 'sk-...' })
```

---

## Resources

- [Winston Logger](https://github.com/winstonjs/winston)
- [Pino Logger](https://github.com/pinojs/pino)
- [Logstash](https://www.elastic.co/logstash)
- [ELK Stack](https://www.elastic.co/elastic-stack)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
