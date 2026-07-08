# Monitoring Dashboard Configuration Guide

## Overview

This guide covers setting up monitoring dashboards for Hermes Game Operator.

---

## Dashboard Tools

### 1. Grafana

**Purpose**: Visualization and alerting

**Installation**:
```bash
# Docker
docker run -d -p 3000:3000 grafana/grafana

# Or use package manager
sudo apt-get install grafana
```

**Configuration**:
```yaml
# grafana.ini
[server]
http_port = 3000

[security]
admin_user = admin
admin_password = admin

[users]
allow_sign_up = false
```

---

### 2. Prometheus

**Purpose**: Metrics collection and storage

**Installation**:
```bash
# Docker
docker run -d -p 9090:9090 prom/prometheus

# Or download binary
wget https://github.com/prometheus/prometheus/releases/download/v2.45.0/prometheus-2.45.0.linux-amd64.tar.gz
tar xzf prometheus-2.45.0.linux-amd64.tar.gz
```

**Configuration**:
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'hermes-operator'
    static_configs:
      - targets: ['localhost:1420']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

---

## Metrics to Monitor

### Application Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `hermes_tasks_total` | Counter | Total tasks created |
| `hermes_tasks_active` | Gauge | Currently active tasks |
| `hermes_tasks_completed` | Counter | Completed tasks |
| `hermes_tasks_failed` | Counter | Failed tasks |
| `hermes_task_duration_seconds` | Histogram | Task execution time |
| `hermes_api_requests_total` | Counter | API requests |
| `hermes_api_request_duration_seconds` | Histogram | API response time |
| `hermes_errors_total` | Counter | Total errors |

### System Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `node_cpu_seconds_total` | Counter | CPU usage |
| `node_memory_total_bytes` | Gauge | Total memory |
| `node_memory_available_bytes` | Gauge | Available memory |
| `node_disk_io_time_seconds_total` | Counter | Disk I/O |
| `node_network_receive_bytes_total` | Counter | Network receive |
| `node_network_transmit_bytes_total` | Counter | Network transmit |

---

## Dashboard Panels

### Panel 1: Task Overview

**Type**: Stat

**Query**:
```promql
hermes_tasks_total
```

**Visualization**:
```json
{
  "type": "stat",
  "title": "Total Tasks",
  "targets": [
    {
      "expr": "hermes_tasks_total",
      "legendFormat": "Tasks"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "color": {
        "mode": "thresholds"
      },
      "thresholds": {
        "steps": [
          { "color": "green", "value": null },
          { "color": "yellow", "value": 100 },
          { "color": "red", "value": 1000 }
        ]
      }
    }
  }
}
```

---

### Panel 2: Task Status Distribution

**Type**: Pie Chart

**Query**:
```promql
hermes_tasks_active
hermes_tasks_completed
hermes_tasks_failed
```

**Visualization**:
```json
{
  "type": "piechart",
  "title": "Task Status",
  "targets": [
    {
      "expr": "hermes_tasks_active",
      "legendFormat": "Active"
    },
    {
      "expr": "hermes_tasks_completed",
      "legendFormat": "Completed"
    },
    {
      "expr": "hermes_tasks_failed",
      "legendFormat": "Failed"
    }
  ]
}
```

---

### Panel 3: Task Duration

**Type**: Histogram

**Query**:
```promql
histogram_quantile(0.50, rate(hermes_task_duration_seconds_bucket[5m]))
histogram_quantile(0.90, rate(hermes_task_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(hermes_task_duration_seconds_bucket[5m]))
```

**Visualization**:
```json
{
  "type": "timeseries",
  "title": "Task Duration (p50, p90, p99)",
  "targets": [
    {
      "expr": "histogram_quantile(0.50, rate(hermes_task_duration_seconds_bucket[5m]))",
      "legendFormat": "p50"
    },
    {
      "expr": "histogram_quantile(0.90, rate(hermes_task_duration_seconds_bucket[5m]))",
      "legendFormat": "p90"
    },
    {
      "expr": "histogram_quantile(0.99, rate(hermes_task_duration_seconds_bucket[5m]))",
      "legendFormat": "p99"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "s"
    }
  }
}
```

---

### Panel 4: API Request Rate

**Type**: Time Series

**Query**:
```promql
rate(hermes_api_requests_total[5m])
```

**Visualization**:
```json
{
  "type": "timeseries",
  "title": "API Request Rate",
  "targets": [
    {
      "expr": "rate(hermes_api_requests_total[5m])",
      "legendFormat": "{{method}} {{endpoint}}"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "reqps"
    }
  }
}
```

---

### Panel 5: Error Rate

**Type**: Time Series

**Query**:
```promql
rate(hermes_errors_total[5m])
```

**Visualization**:
```json
{
  "type": "timeseries",
  "title": "Error Rate",
  "targets": [
    {
      "expr": "rate(hermes_errors_total[5m])",
      "legendFormat": "{{error_type}}"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "eps",
      "color": {
        "mode": "palette-classic"
      }
    }
  }
}
```

---

### Panel 6: System Resources

**Type**: Gauge

**Query**:
```promql
100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
(node_memory_total_bytes - node_memory_available_bytes) / node_memory_total_bytes * 100
```

**Visualization**:
```json
{
  "type": "gauge",
  "title": "CPU & Memory Usage",
  "targets": [
    {
      "expr": "100 - (avg by(instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
      "legendFormat": "CPU %"
    },
    {
      "expr": "(node_memory_total_bytes - node_memory_available_bytes) / node_memory_total_bytes * 100",
      "legendFormat": "Memory %"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "thresholds": {
        "steps": [
          { "color": "green", "value": null },
          { "color": "yellow", "value": 70 },
          { "color": "red", "value": 90 }
        ]
      }
    }
  }
}
```

---

## Alert Rules

### Alert 1: High Error Rate

```yaml
# alerting_rules.yml
groups:
  - name: hermes-operator
    rules:
      - alert: HighErrorRate
        expr: rate(hermes_errors_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"
```

---

### Alert 2: Task Failure Rate

```yaml
- alert: HighTaskFailureRate
  expr: rate(hermes_tasks_failed[5m]) / rate(hermes_tasks_total[5m]) > 0.1
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "High task failure rate"
    description: "Task failure rate is {{ $value | humanizePercentage }}"
```

---

### Alert 3: Slow Task Execution

```yaml
- alert: SlowTaskExecution
  expr: histogram_quantile(0.99, rate(hermes_task_duration_seconds_bucket[5m])) > 60
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Slow task execution"
    description: "99th percentile task duration is {{ $value }} seconds"
```

---

### Alert 4: High CPU Usage

```yaml
- alert: HighCPUUsage
  expr: 100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 90
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High CPU usage"
    description: "CPU usage is {{ $value }}%"
```

---

### Alert 5: Low Disk Space

```yaml
- alert: LowDiskSpace
  expr: (node_filesystem_avail_bytes / node_filesystem_size_bytes) * 100 < 10
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Low disk space"
    description: "Only {{ $value }}% disk space remaining"
```

---

## Dashboard Export

### Export Dashboard

```bash
# Export to JSON
curl -H "Authorization: Bearer $GRAFANA_API_KEY" \
  http://localhost:3000/api/dashboards/uid/$DASHBOARD_UID \
  > dashboard.json
```

### Import Dashboard

```bash
# Import from JSON
curl -X POST -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -H "Content-Type: application/json" \
  -d @dashboard.json \
  http://localhost:3000/api/dashboards/import
```

---

## Best Practices

### 1. Use Meaningful Labels

```typescript
// Good
counter.inc({ endpoint: '/api/tasks', method: 'POST' })

// Bad
counter.inc()
```

### 2. Set Appropriate Scrape Intervals

```yaml
# Good: Different intervals for different metrics
scrape_configs:
  - job_name: 'hermes-operator'
    scrape_interval: 5s  # High frequency for critical metrics
  
  - job_name: 'node-exporter'
    scrape_interval: 15s  # Lower frequency for system metrics
```

### 3. Use Recording Rules

```yaml
# recording_rules.yml
groups:
  - name: hermes-operator-recording
    rules:
      - record: hermes:api_request_duration:avg5m
        expr: avg by(endpoint) (rate(hermes_api_request_duration_seconds_sum[5m]) / rate(hermes_api_request_duration_seconds_count[5m]))
```

### 4. Test Alerts

```bash
# Test alert expression
curl 'http://localhost:9090/api/v1/query?query=rate(hermes_errors_total[5m])%20%3E%200.1'
```

---

## Resources

- [Grafana Documentation](https://grafana.com/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
