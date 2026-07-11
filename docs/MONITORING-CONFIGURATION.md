# 监控配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 监控架构

```
┌─────────────────────────────────────┐
│          Monitoring Stack           │
├─────────────────────────────────────┤
│  Collection: Prometheus, CloudWatch │
├─────────────────────────────────────┤
│  Storage: Prometheus, InfluxDB      │
├─────────────────────────────────────┤
│  Visualization: Grafana, Kibana     │
├─────────────────────────────────────┤
│  Alerting: Alertmanager, PagerDuty  │
└─────────────────────────────────────┘
```

---

## Prometheus 配置

### 基本配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  scrape_timeout: 10s

rule_files:
  - "alert_rules.yml"

scrape_configs:
  - job_name: 'hermes-game-operator'
    metrics_path: '/metrics'
    scrape_interval: 5s
    static_configs:
      - targets: 
        - 'app1:8080'
        - 'app2:8080'
        - 'app3:8080'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### 告警规则

```yaml
# alert_rules.yml
groups:
  - name: hermes-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "95th percentile latency is {{ $value }}s"

      - alert: HighCPUUsage
        expr: rate(process_cpu_seconds_total[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}"

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes / 1024 / 1024 > 1024
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}MB"

      - alert: InstanceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Instance {{ $labels.instance }} down"
```

---

## Grafana 配置

### 数据源

```json
{
  "name": "Prometheus",
  "type": "prometheus",
  "url": "http://prometheus:9090",
  "access": "proxy",
  "isDefault": true
}
```

### 仪表板

```json
{
  "dashboard": {
    "title": "Hermes Game Operator",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (method)",
            "legendFormat": "{{method}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m]))",
            "legendFormat": "Errors"
          }
        ]
      },
      {
        "title": "Response Time (P95)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "P95"
          }
        ]
      },
      {
        "title": "CPU Usage",
        "type": "gauge",
        "targets": [
          {
            "expr": "rate(process_cpu_seconds_total[5m])",
            "legendFormat": "CPU"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "min": 0,
            "max": 1,
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 0.6},
                {"color": "red", "value": 0.8}
              ]
            }
          }
        }
      }
    ]
  }
}
```

---

## CloudWatch 配置

### 自定义指标

```javascript
// metrics.js
const AWS = require('aws-sdk');
const cloudwatch = new AWS.CloudWatch();

async function publishMetric(metricName, value, unit) {
  const params = {
    MetricData: [{
      MetricName: metricName,
      Value: value,
      Unit: unit,
      Dimensions: [{
        Name: 'Service',
        Value: 'HermesGameOperator'
      }]
    }],
    Namespace: 'HermesGameOperator'
  };

  await cloudwatch.putMetricData(params).promise();
}

// 发布指标
await publishMetric('RequestCount', requestCount, 'Count');
await publishMetric('ErrorCount', errorCount, 'Count');
await publishMetric('ResponseTime', responseTime, 'Milliseconds');
```

### 告警

```yaml
# cloudwatch-alarm.yaml
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
```

---

## 日志配置

### 结构化日志

```javascript
// logger.js
const pino = require('pino');

const logger = pino({
  level: process.env.LOG_LEVEL || 'info',
  formatters: {
    level: (label) => {
      return { level: label };
    }
  },
  timestamp: pino.stdTimeFunctions.isoTime
});

// 使用
logger.info({ userId: user.id, taskId: task.id }, 'Task created');
logger.error({ error: err.message, stack: err.stack }, 'Error occurred');
```

### 日志聚合

```yaml
# fluentd.conf
<source>
  @type tail
  path /var/log/hermes-game-operator/*.log
  pos_file /var/log/fluentd/hermes-game-operator.pos
  tag hermes.*
  <parse>
    @type json
    time_key time
    time_format %Y-%m-%dT%H:%M:%S.%NZ
  </parse>
</source>

<match hermes.**>
  @type elasticsearch
  host elasticsearch
  port 9200
  index_name hermes-game-operator
  <buffer>
    @type file
    path /var/log/fluentd/buffer
    flush_interval 10s
  </buffer>
</match>
```

---

## 分布式追踪

### OpenTelemetry

```javascript
// tracing.js
const { NodeTracerProvider } = require('@opentelemetry/sdk-trace-node');
const { SimpleSpanProcessor } = require('@opentelemetry/sdk-trace-base');
const { JaegerExporter } = require('@opentelemetry/exporter-jaeger');

const provider = new NodeTracerProvider();

const exporter = new JaegerExporter({
  serviceName: 'hermes-game-operator',
  endpoint: 'http://jaeger:14268/api/traces'
});

provider.addSpanProcessor(new SimpleSpanProcessor(exporter));
provider.register();

// 使用
const tracer = provider.getTracer('hermes-game-operator');
const span = tracer.startSpan('handleRequest');
try {
  // 处理请求
  span.setStatus({ code: SpanStatusCode.OK });
} catch (error) {
  span.setStatus({ code: SpanStatusCode.ERROR });
  throw error;
} finally {
  span.end();
}
```

---

## 监控最佳实践

### 1. 四个黄金信号

- **延迟**: 请求处理时间
- **流量**: 系统负载
- **错误**: 失败请求比例
- **饱和度**: 资源使用程度

### 2. USE 方法

- **Utilization**: 资源使用率
- **Saturation**: 资源排队工作
- **Errors**: 错误数量

### 3. RED 方法

- **Rate**: 每秒请求数
- **Errors**: 每秒错误数
- **Duration**: 每个请求时间

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Monitoring Team