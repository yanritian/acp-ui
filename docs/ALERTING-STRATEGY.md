# Alerting Strategy Guide

## Overview

This guide covers alerting strategies for Hermes Game Operator.

---

## Alert Severity Levels

### Critical (P0)

**Definition**: System down, data loss, security breach  
**Response Time**: Immediate (< 15 minutes)  
**Notification**: Page + Email + SMS  
**Escalation**: On-call → Engineering Lead → CTO

**Examples**:
- Database corruption
- Service completely down
- Security breach detected
- Data loss confirmed

---

### High (P1)

**Definition**: Major functionality broken, significant degradation  
**Response Time**: < 1 hour  
**Notification**: Page + Email  
**Escalation**: On-call → Engineering Lead

**Examples**:
- Task execution failing > 50%
- API response time > 10s
- Memory usage > 95%
- Disk space < 5%

---

### Medium (P2)

**Definition**: Minor functionality broken, performance degradation  
**Response Time**: < 4 hours  
**Notification**: Email + Slack  
**Escalation**: On-call

**Examples**:
- Task execution failing 10-50%
- API response time 5-10s
- Memory usage 80-95%
- Disk space 5-10%

---

### Low (P3)

**Definition**: Cosmetic issues, minor degradation  
**Response Time**: < 24 hours  
**Notification**: Slack  
**Escalation**: None

**Examples**:
- Task execution failing < 10%
- API response time 2-5s
- Memory usage 70-80%
- Non-critical errors

---

## Alert Rules

### Rule 1: Service Health

```yaml
# alerts/service.yml
groups:
  - name: hermes-service
    rules:
      - alert: ServiceDown
        expr: up{job="hermes-operator"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Hermes Operator is down"
          description: "Service has been down for {{ $value }} minutes"
          runbook: "https://wiki.example.com/runbooks/service-down"
```

---

### Rule 2: Task Failure Rate

```yaml
- alert: HighTaskFailureRate
  expr: |
    (
      rate(hermes_tasks_failed[5m])
      /
      rate(hermes_tasks_total[5m])
    ) > 0.5
  for: 5m
  labels:
    severity: high
  annotations:
    summary: "High task failure rate"
    description: "Task failure rate is {{ $value | humanizePercentage }}"
    runbook: "https://wiki.example.com/runbooks/task-failures"
```

---

### Rule 3: API Response Time

```yaml
- alert: SlowAPIResponse
  expr: |
    histogram_quantile(0.99, 
      rate(hermes_api_request_duration_seconds_bucket[5m])
    ) > 5
  for: 5m
  labels:
    severity: medium
  annotations:
    summary: "Slow API response time"
    description: "99th percentile response time is {{ $value }}s"
    runbook: "https://wiki.example.com/runbooks/slow-api"
```

---

### Rule 4: Error Rate

```yaml
- alert: HighErrorRate
  expr: rate(hermes_errors_total[5m]) > 0.1
  for: 5m
  labels:
    severity: medium
  annotations:
    summary: "High error rate"
    description: "Error rate is {{ $value }} errors per second"
    runbook: "https://wiki.example.com/runbooks/high-errors"
```

---

### Rule 5: Memory Usage

```yaml
- alert: HighMemoryUsage
  expr: |
    (node_memory_total_bytes - node_memory_available_bytes)
    /
    node_memory_total_bytes
    * 100 > 90
  for: 5m
  labels:
    severity: high
  annotations:
    summary: "High memory usage"
    description: "Memory usage is {{ $value }}%"
    runbook: "https://wiki.example.com/runbooks/high-memory"
```

---

### Rule 6: Disk Space

```yaml
- alert: LowDiskSpace
  expr: |
    (node_filesystem_avail_bytes / node_filesystem_size_bytes)
    * 100 < 10
  for: 5m
  labels:
    severity: high
  annotations:
    summary: "Low disk space"
    description: "Only {{ $value }}% disk space remaining"
    runbook: "https://wiki.example.com/runbooks/low-disk"
```

---

### Rule 7: CPU Usage

```yaml
- alert: HighCPUUsage
  expr: |
    100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 90
  for: 5m
  labels:
    severity: medium
  annotations:
    summary: "High CPU usage"
    description: "CPU usage is {{ $value }}%"
    runbook: "https://wiki.example.com/runbooks/high-cpu"
```

---

### Rule 8: Database Connection Pool

```yaml
- alert: DatabaseConnectionPoolExhausted
  expr: |
    hermes_db_connections_active / hermes_db_connections_max > 0.9
  for: 5m
  labels:
    severity: high
  annotations:
    summary: "Database connection pool exhausted"
    description: "{{ $value | humanizePercentage }} of connections in use"
    runbook: "https://wiki.example.com/runbooks/db-connections"
```

---

### Rule 9: Task Queue Depth

```yaml
- alert: HighTaskQueueDepth
  expr: hermes_task_queue_depth > 1000
  for: 5m
  labels:
    severity: medium
  annotations:
    summary: "High task queue depth"
    description: "{{ $value }} tasks in queue"
    runbook: "https://wiki.example.com/runbooks/task-queue"
```

---

### Rule 10: Security Incident

```yaml
- alert: SecurityIncident
  expr: |
    rate(hermes_security_violations_total[5m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Security incident detected"
    description: "{{ $value }} security violations detected"
    runbook: "https://wiki.example.com/runbooks/security-incident"
```

---

## Alert Routing

### Routing Tree

```yaml
# alertmanager.yml
route:
  receiver: 'default'
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  
  routes:
    - match:
        severity: critical
      receiver: 'critical'
      continue: true
    
    - match:
        severity: high
      receiver: 'high'
      continue: true
    
    - match:
        severity: medium
      receiver: 'medium'
    
    - match:
        severity: low
      receiver: 'low'
```

---

### Receivers

```yaml
receivers:
  - name: 'default'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts'
  
  - name: 'critical'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
    email_configs:
      - to: 'oncall@example.com'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-critical'
  
  - name: 'high'
    email_configs:
      - to: 'oncall@example.com'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-high'
  
  - name: 'medium'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-medium'
  
  - name: 'low'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/XXX/YYY/ZZZ'
        channel: '#alerts-low'
```

---

## Alert Inhibition

```yaml
inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'high'
    equal: ['alertname', 'instance']
  
  - source_match:
      severity: 'high'
    target_match:
      severity: 'medium'
    equal: ['alertname', 'instance']
```

---

## Alert Silencing

### Silence Alert

```bash
# Silence for 2 hours
curl -X POST http://localhost:9093/api/v2/silences \
  -H "Content-Type: application/json" \
  -d '{
    "matchers": [
      {
        "name": "alertname",
        "value": "HighMemoryUsage",
        "isRegex": false
      }
    ],
    "startsAt": "2026-07-08T10:00:00Z",
    "endsAt": "2026-07-08T12:00:00Z",
    "createdBy": "admin",
    "comment": "Maintenance window"
  }'
```

---

### List Silences

```bash
curl http://localhost:9093/api/v2/silences
```

---

### Delete Silence

```bash
curl -X DELETE http://localhost:9093/api/v2/silence/SILENCE_ID
```

---

## Runbook Templates

### Runbook: Service Down

```markdown
# Service Down Runbook

## Symptoms
- Service is not responding
- Health check failing
- Up metric is 0

## Impact
- All functionality unavailable
- Users cannot access the application

## Diagnosis
1. Check service logs
   ```bash
   journalctl -u hermes-operator -f
   ```

2. Check service status
   ```bash
   systemctl status hermes-operator
   ```

3. Check port availability
   ```bash
   netstat -tlnp | grep 1420
   ```

## Resolution
1. Restart service
   ```bash
   systemctl restart hermes-operator
   ```

2. If restart fails, check logs for errors
   ```bash
   journalctl -u hermes-operator -n 100
   ```

3. If issue persists, restore from backup
   ```bash
   ./restore.sh
   ```

## Escalation
If not resolved in 15 minutes, escalate to Engineering Lead.
```

---

### Runbook: High Task Failure Rate

```markdown
# High Task Failure Rate Runbook

## Symptoms
- Task failure rate > 50%
- Multiple task_failed events
- Error logs increasing

## Impact
- Users cannot complete tasks
- Poor user experience

## Diagnosis
1. Check task logs
   ```bash
   grep "task_failed" ~/.config/hermes-operator/logs/app.log
   ```

2. Check error patterns
   ```bash
   grep "error" ~/.config/hermes-operator/logs/error.log | tail -20
   ```

3. Check external services
   ```bash
   curl https://api.anthropic.com/v1/messages -H "x-api-key: $API_KEY"
   ```

## Resolution
1. If external service issue, wait for recovery
2. If code issue, rollback to previous version
   ```bash
   ./rollback.sh
   ```

3. If configuration issue, restore configuration
   ```bash
   cp ~/.config/hermes-operator.backup/config.json ~/.config/hermes-operator/config.json
   ```

## Escalation
If not resolved in 1 hour, escalate to Engineering Lead.
```

---

## Best Practices

### 1. Alert on Symptoms, Not Causes

```yaml
# Good: Alert on symptom
- alert: HighErrorRate
  expr: rate(hermes_errors_total[5m]) > 0.1

# Bad: Alert on cause
- alert: DatabaseSlow
  expr: hermes_db_query_time > 100
```

### 2. Include Runbook Links

```yaml
annotations:
  runbook: "https://wiki.example.com/runbooks/high-errors"
```

### 3. Avoid Alert Fatigue

```yaml
# Good: Meaningful threshold
- alert: HighErrorRate
  expr: rate(hermes_errors_total[5m]) > 0.1

# Bad: Too sensitive
- alert: AnyError
  expr: hermes_errors_total > 0
```

### 4. Test Alerts Regularly

```bash
# Test alert expression
curl 'http://localhost:9090/api/v1/query?query=rate(hermes_errors_total[5m])%20%3E%200.1'
```

---

## Resources

- [Alertmanager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)
- [Prometheus Alerting Rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
- [PagerDuty Integration](https://www.pagerduty.com/docs/guides/prometheus-integration-guide/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
