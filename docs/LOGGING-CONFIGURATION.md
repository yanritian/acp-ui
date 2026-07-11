# 日志管理指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 日志架构

```
┌─────────────────────────────────────┐
│          Logging Stack              │
├─────────────────────────────────────┤
│  Collection: Fluentd, Filebeat      │
├─────────────────────────────────────┤
│  Aggregation: Logstash              │
├─────────────────────────────────────┤
│  Storage: Elasticsearch             │
├─────────────────────────────────────┤
│  Visualization: Kibana              │
└─────────────────────────────────────┘
```

---

## 结构化日志

### 日志格式

```javascript
// logger.js
const pino = require('pino');

const logger = pino({
  level: process.env.LOG_LEVEL || 'info',
  formatters: {
    level: (label) => {
      return { level: label };
    },
    bindings: (bindings) => {
      return { 
        pid: bindings.pid,
        hostname: bindings.hostname,
        service: 'hermes-game-operator'
      };
    }
  },
  timestamp: pino.stdTimeFunctions.isoTime,
  base: {
    env: process.env.NODE_ENV,
    version: process.env.APP_VERSION
  }
});

// 使用
logger.info({ 
  userId: user.id, 
  taskId: task.id,
  duration: 150 
}, 'Task completed successfully');

logger.error({ 
  error: err.message, 
  stack: err.stack,
  userId: user.id 
}, 'Task failed');
```

### 日志级别

| 级别 | 用途 | 示例 |
|------|------|------|
| **fatal** | 系统无法继续运行 | 数据库连接失败 |
| **error** | 操作失败 | API 调用失败 |
| **warn** | 潜在问题 | 高内存使用 |
| **info** | 重要事件 | 用户登录 |
| **debug** | 调试信息 | 请求详情 |
| **trace** | 详细跟踪 | 函数调用 |

---

## Fluentd 配置

### 基本配置

```xml
# fluent.conf
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
  type_name _doc
  logstash_format true
  logstash_prefix hermes
  <buffer>
    @type file
    path /var/log/fluentd/buffer
    flush_interval 10s
    retry_max_interval 30s
    retry_forever true
  </buffer>
</match>
```

### 过滤器

```xml
<filter hermes.**>
  @type record_transformer
  <record>
    hostname ${hostname}
    service hermes-game-operator
    environment ${env}
  </record>
</filter>

<filter hermes.**>
  @type parser
  key_name message
  reserve_data true
  <parse>
    @type json
  </parse>
</filter>
```

---

## Elasticsearch 配置

### 索引模板

```json
{
  "index_patterns": ["hermes-*"],
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1,
    "index.lifecycle.name": "hermes-ilm-policy",
    "index.lifecycle.rollover_alias": "hermes-write"
  },
  "mappings": {
    "properties": {
      "@timestamp": {
        "type": "date"
      },
      "level": {
        "type": "keyword"
      },
      "message": {
        "type": "text"
      },
      "userId": {
        "type": "keyword"
      },
      "taskId": {
        "type": "keyword"
      },
      "duration": {
        "type": "long"
      },
      "error": {
        "type": "text"
      }
    }
  }
}
```

### 生命周期策略

```json
{
  "policy": {
    "phases": {
      "hot": {
        "min_age": "0ms",
        "actions": {
          "rollover": {
            "max_size": "50gb",
            "max_age": "7d"
          }
        }
      },
      "warm": {
        "min_age": "7d",
        "actions": {
          "shrink": {
            "number_of_shards": 1
          }
        }
      },
      "cold": {
        "min_age": "30d",
        "actions": {
          "freeze": {}
        }
      },
      "delete": {
        "min_age": "90d",
        "actions": {
          "delete": {}
        }
      }
    }
  }
}
```

---

## Kibana 配置

### 索引模式

```json
{
  "title": "hermes-*",
  "timeFieldName": "@timestamp"
}
```

### 仪表板

```json
{
  "title": "Hermes Game Operator Logs",
  "panelsJSON": [
    {
      "type": "visualization",
      "title": "Log Level Distribution",
      "gridData": {
        "x": 0,
        "y": 0,
        "w": 24,
        "h": 15
      }
    },
    {
      "type": "visualization",
      "title": "Error Rate Over Time",
      "gridData": {
        "x": 24,
        "y": 0,
        "w": 24,
        "h": 15
      }
    }
  ]
}
```

---

## 日志最佳实践

### 1. 日志结构

```json
{
  "@timestamp": "2026-07-11T10:30:00.000Z",
  "level": "info",
  "message": "Task created",
  "service": "hermes-game-operator",
  "environment": "production",
  "userId": "user_123",
  "taskId": "task_456",
  "duration": 150,
  "metadata": {
    "version": "0.1.0-alpha",
    "hostname": "app-1"
  }
}
```

### 2. 日志聚合

- **集中式日志**: 所有日志集中存储
- **结构化格式**: 使用 JSON 格式
- **统一时间戳**: 使用 ISO 8601 格式
- **包含上下文**: 包含请求 ID、用户 ID 等

### 3. 日志保留

| 日志类型 | 保留时间 | 存储位置 |
|----------|---------|---------|
| 应用日志 | 30 天 | Elasticsearch |
| 访问日志 | 90 天 | S3 |
| 审计日志 | 7 年 | S3 Glacier |
| 错误日志 | 90 天 | Elasticsearch |

---

## 日志查询示例

### Kibana 查询

```
level:error AND service:hermes-game-operator

level:error AND duration:>1000

userId:user_123 AND taskId:task_456

@timestamp:[now-1h TO now] AND level:error

message:"connection timeout"
```

### API 查询

```bash
# 搜索错误日志
curl -X GET "localhost:9200/hermes-*/_search" -H 'Content-Type: application/json' -d'
{
  "query": {
    "bool": {
      "must": [
        { "term": { "level": "error" } },
        { "range": { "@timestamp": { "gte": "now-1h" } } }
      ]
    }
  }
}'
```

---

## 日志监控

### 日志告警

```yaml
# log-alerts.yml
groups:
  - name: log-alerts
    rules:
      - alert: HighErrorRate
        expr: sum(rate(log_messages_total{level="error"}[5m])) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate in logs"
          description: "{{ $value }} errors per second"
```

### 日志指标

```javascript
// metrics.js
const prometheus = require('prom-client');

const logMessagesTotal = new prometheus.Counter({
  name: 'log_messages_total',
  help: 'Total number of log messages',
  labelNames: ['level', 'service']
});

// 记录日志指标
logMessagesTotal.inc({ level: 'info', service: 'hermes-game-operator' });
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Logging Team