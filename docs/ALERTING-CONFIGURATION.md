# 告警配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 告警架构

```
┌─────────────────────────────────────┐
│          Alert Pipeline             │
├─────────────────────────────────────┤
│  Detection: Prometheus, CloudWatch  │
├─────────────────────────────────────┤
│  Routing: Alertmanager              │
├─────────────────────────────────────┤
│  Notification: PagerDuty, Slack     │
├─────────────────────────────────────┤
│  Escalation: On-call rotation       │
└─────────────────────────────────────┘
```

---

## Alertmanager 配置

### 基本配置

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alerts@example.com'
  smtp_auth_username: 'alerts@example.com'
  smtp_auth_password: 'password'

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'default-receiver'
  routes:
    - match:
        severity: critical
      receiver: 'critical-receiver'
      continue: true
    - match:
        severity: warning
      receiver: 'warning-receiver'

receivers:
  - name: 'default-receiver'
    email_configs:
      - to: 'team@example.com'
        send_resolved: true

  - name: 'critical-receiver'
    pagerduty_configs:
      - service_key: 'your-pagerduty-key'
        severity: 'critical'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#alerts-critical'
        title: '🚨 Critical Alert'
        text: '{{ .CommonAnnotations.description }}'
        send_resolved: true

  - name: 'warning-receiver'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#alerts-warning'
        title: '⚠️ Warning Alert'
        text: '{{ .CommonAnnotations.description }}'
        send_resolved: true
    email_configs:
      - to: 'team@example.com'
        send_resolved: true

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

---

## Prometheus 告警规则

### 应用告警

```yaml
# app-alerts.yml
groups:
  - name: hermes-app-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate on {{ $labels.instance }}"
          description: "Error rate is {{ $value | humanizePercentage }}"
          runbook_url: "https://wiki.example.com/runbooks/high-error-rate"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.instance }}"
          description: "95th percentile latency is {{ $value }}s"

      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.instance }} is down"
          description: "{{ $labels.instance }} has been down for {{ $value }}s"
```

### 基础设施告警

```yaml
# infra-alerts.yml
groups:
  - name: hermes-infra-alerts
    rules:
      - alert: HighCPUUsage
        expr: rate(process_cpu_seconds_total[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value | humanizePercentage }}"

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes / 1024 / 1024 > 1024
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is {{ $value }}MB"

      - alert: HighDiskUsage
        expr: node_filesystem_avail_bytes / node_filesystem_size_bytes < 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High disk usage on {{ $labels.instance }}"
          description: "Disk usage is {{ $value | humanizePercentage }}"
```

### 数据库告警

```yaml
# db-alerts.yml
groups:
  - name: hermes-db-alerts
    rules:
      - alert: DatabaseDown
        expr: pg_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database is down"
          description: "PostgreSQL is not responding"

      - alert: HighConnections
        expr: pg_stat_activity_count > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of connections"
          description: "{{ $value }} active connections"

      - alert: ReplicationLag
        expr: pg_replication_lag > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Replication lag detected"
          description: "Lag is {{ $value }}s"
```

---

## CloudWatch 告警

### 指标告警

```yaml
# cloudwatch-alerts.yaml
Resources:
  HighErrorAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: hermes-high-errors
      MetricName: ErrorCount
      Namespace: HermesGameOperator
      Statistic: Sum
      Period: 300
      EvaluationPeriods: 2
      Threshold: 10
      ComparisonOperator: GreaterThanThreshold
      AlarmActions:
        - !Ref AlertTopic
      OKActions:
        - !Ref AlertTopic
      AlarmDescription: "High error rate detected"
      TreatMissingData: notBreaching

  HighLatencyAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: hermes-high-latency
      MetricName: ResponseTime
      Namespace: HermesGameOperator
      Statistic: Average
      Period: 300
      EvaluationPeriods: 2
      Threshold: 1000
      ComparisonOperator: GreaterThanThreshold
      AlarmActions:
        - !Ref AlertTopic
```

### SNS 主题

```yaml
Resources:
  AlertTopic:
    Type: AWS::SNS::Topic
    Properties:
      TopicName: hermes-alerts
      Subscription:
        - Protocol: email
          Endpoint: team@example.com
        - Protocol: lambda
          Endpoint: !GetAtt AlertHandler.Arn
```

---

## PagerDuty 集成

### 服务配置

```yaml
# pagerduty-service.yaml
Resources:
  HermesService:
    Type: Custom::PagerDutyService
    Properties:
      ServiceToken: !GetAtt PagerDutyFunction.Arn
      Name: "Hermes Game Operator"
      Description: "Production service for Hermes Game Operator"
      EscalationPolicy: !Ref HermesEscalationPolicy
      AlertCreation: "create_alerts"
      IncidentUrgencyRule:
        Type: "constant"
        Urgency: "high"
```

### 升级策略

```yaml
Resources:
  HermesEscalationPolicy:
    Type: Custom::PagerDutyEscalationPolicy
    Properties:
      Name: "Hermes Escalation Policy"
      EscalationRules:
        - EscalationDelayInMinutes: 15
          Targets:
            - Type: "user_reference"
              Id: "P123456" # Primary on-call
        - EscalationDelayInMinutes: 30
          Targets:
            - Type: "user_reference"
              Id: "P789012" # Secondary on-call
```

---

## Slack 集成

### Webhook 配置

```javascript
// slack-alert.js
const axios = require('axios');

async function sendSlackAlert(channel, message, severity) {
  const colors = {
    critical: '#FF0000',
    warning: '#FFA500',
    info: '#00FF00'
  };

  const payload = {
    channel: channel,
    username: 'Alert Bot',
    icon_emoji: severity === 'critical' ? ':rotating_light:' : ':warning:',
    attachments: [
      {
        color: colors[severity],
        title: `${severity.toUpperCase()} Alert`,
        text: message,
        fields: [
          {
            title: 'Service',
            value: 'Hermes Game Operator',
            short: true
          },
          {
            title: 'Time',
            value: new Date().toISOString(),
            short: true
          }
        ],
        footer: 'Hermes Game Operator Alerts'
      }
    ]
  };

  await axios.post(process.env.SLACK_WEBHOOK_URL, payload);
}

// 使用
await sendSlackAlert('#alerts-critical', 'High error rate detected', 'critical');
```

---

## 告警最佳实践

### 1. 告警分类

| 级别 | 响应时间 | 通知方式 | 示例 |
|------|---------|---------|------|
| **P0 - Critical** | 立即 | PagerDuty + Slack | 服务宕机 |
| **P1 - High** | 15 分钟 | PagerDuty | 高错误率 |
| **P2 - Medium** | 1 小时 | Slack + Email | 高延迟 |
| **P3 - Low** | 4 小时 | Email | 磁盘使用率高 |

### 2. 告警规则

- **可操作性**: 每个告警都应该是可操作的
- **明确性**: 告警应该清晰描述问题
- **上下文**: 提供足够的上下文信息
- **链接**: 提供 runbook 链接

### 3. 告警维护

- **定期审查**: 每月审查告警规则
- **消除噪音**: 移除不必要的告警
- **测试告警**: 定期测试告警流程
- **更新 runbook**: 保持 runbook 最新

---

## Runbook 示例

```markdown
# High Error Rate Runbook

## 症状
错误率超过 5%

## 影响
用户体验下降，可能丢失收入

## 诊断步骤
1. 检查日志：`kubectl logs -l app=hermes`
2. 检查指标：Grafana 仪表板
3. 检查最近的部署

## 缓解措施
1. 回滚最近的部署
2. 增加实例数量
3. 检查依赖服务

## 根因分析
- 检查代码变更
- 检查配置变更
- 检查基础设施变更

## 联系人
- 开发团队: dev@example.com
- 运维团队: ops@example.com
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Alerting Team