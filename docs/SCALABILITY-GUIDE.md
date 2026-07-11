# 扩展性指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的水平扩展和垂直扩展策略。

---

## 扩展策略

### 垂直扩展 vs 水平扩展

| 策略 | 描述 | 优点 | 缺点 |
|------|------|------|------|
| **垂直扩展** | 增加单个实例的资源 | 简单，无需修改代码 | 有上限，需要停机 |
| **水平扩展** | 增加实例数量 | 无限扩展，高可用 | 需要负载均衡 |

### 推荐方案

```
初期: 垂直扩展
- 单实例，增加 CPU/内存
- 适合: < 1000 用户

中期: 水平扩展
- 多实例 + 负载均衡
- 适合: 1000 - 10000 用户

后期: 混合扩展
- 微服务架构 + 自动扩展
- 适合: > 10000 用户
```

---

## 水平扩展

### 应用层扩展

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    image: hermes-game-operator:latest
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1'
          memory: 1G
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgres://db:5432/hermes
      - REDIS_URL=redis://redis:6379
```

### 负载均衡

```nginx
# nginx.conf
upstream hermes_backend {
    least_conn;
    server app1:8080 max_fails=3 fail_timeout=30s;
    server app2:8080 max_fails=3 fail_timeout=30s;
    server app3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://hermes_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 数据库扩展

### 主从复制

```bash
# 主库配置 (postgresql.conf)
wal_level = replica
max_wal_senders = 10
wal_keep_segments = 64

# 从库配置
hot_standby = on
primary_conninfo = 'host=primary port=5432 user=replication'
```

### 读写分离

```javascript
// 数据库连接池
const readPool = new Pool({
  host: 'replica1,replica2,replica3',
  user: 'readonly',
  database: 'hermes'
});

const writePool = new Pool({
  host: 'primary',
  user: 'readwrite',
  database: 'hermes'
});

// 读操作使用从库
async function getTasks() {
  return readPool.query('SELECT * FROM tasks');
}

// 写操作使用主库
async function createTask(task) {
  return writePool.query('INSERT INTO tasks ...');
}
```

---

## 缓存扩展

### Redis Cluster

```bash
# 创建 Redis Cluster
redis-cli --cluster create \
  10.0.0.1:6379 \
  10.0.0.2:6379 \
  10.0.0.3:6379 \
  --cluster-replicas 1
```

### 缓存策略

```javascript
// 多级缓存
const cache = new MultiLevelCache([
  new MemoryCache({ maxSize: 1000 }),
  new RedisCache({ url: 'redis://localhost:6379' })
]);

// 缓存任务
async function getTask(taskId) {
  // 尝试从缓存获取
  let task = await cache.get(`task:${taskId}`);
  
  if (!task) {
    // 从数据库获取
    task = await database.getTask(taskId);
    
    // 存入缓存
    await cache.set(`task:${taskId}`, task, { ttl: 3600 });
  }
  
  return task;
}
```

---

## 微服务架构

### 服务拆分

```
hermes-game-operator/
├── api-gateway/          # API 网关
├── task-service/         # 任务服务
├── event-service/        # 事件服务
├── approval-service/     # 审批服务
├── auth-service/         # 认证服务
└── notification-service/ # 通知服务
```

### 服务通信

```javascript
// 使用消息队列
const amqp = require('amqplib');

// 发布事件
async function publishEvent(event) {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();
  
  await channel.assertQueue('hermes-events');
  channel.sendToQueue('hermes-events', Buffer.from(JSON.stringify(event)));
}

// 订阅事件
async function subscribeToEvents() {
  const connection = await amqp.connect('amqp://localhost');
  const channel = await connection.createChannel();
  
  await channel.assertQueue('hermes-events');
  channel.consume('hermes-events', (msg) => {
    const event = JSON.parse(msg.content.toString());
    handleEvent(event);
    channel.ack(msg);
  });
}
```

---

## 自动扩展

### Kubernetes HPA

```yaml
# horizontal-pod-autoscaler.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: hermes-game-operator-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hermes-game-operator
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### AWS Auto Scaling

```json
{
  "AutoScalingGroupName": "hermes-game-operator-asg",
  "MinSize": 2,
  "MaxSize": 10,
  "DesiredCapacity": 3,
  "TargetTrackingConfigurations": [
    {
      "PredefinedMetricSpecification": {
        "PredefinedMetricType": "ASGAverageCPUUtilization"
      },
      "TargetValue": 70.0
    }
  ]
}
```

---

## 性能优化

### 数据库优化

```sql
-- 添加索引
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);

-- 分区表
CREATE TABLE tasks (
  task_id UUID,
  created_at TIMESTAMPTZ
) PARTITION BY RANGE (created_at);

-- 创建分区
CREATE TABLE tasks_2026_07 PARTITION OF tasks
  FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');
```

### 应用优化

```javascript
// 批处理
async function processTasks(tasks) {
  const batchSize = 100;
  const results = [];
  
  for (let i = 0; i < tasks.length; i += batchSize) {
    const batch = tasks.slice(i, i + batchSize);
    const batchResults = await Promise.all(
      batch.map(task => processTask(task))
    );
    results.push(...batchResults);
  }
  
  return results;
}

// 流式处理
async function streamLargeData(query) {
  const cursor = pool.query(query).stream();
  
  for await (const row of cursor) {
    yield row;
  }
}
```

---

## 监控扩展

### 分布式追踪

```javascript
// 使用 OpenTelemetry
const { trace } = require('@opentelemetry/api');

const tracer = trace.getTracer('hermes-game-operator');

async function handleRequest(req, res) {
  const span = tracer.startSpan('handle-request');
  
  try {
    // 处理请求
    const result = await processRequest(req);
    span.setStatus({ code: SpanStatusCode.OK });
    return result;
  } catch (error) {
    span.setStatus({ code: SpanStatusCode.ERROR });
    throw error;
  } finally {
    span.end();
  }
}
```

### 集中式日志

```javascript
// 结构化日志
const logger = require('pino')({
  level: 'info',
  transport: {
    target: 'pino-elasticsearch',
    options: {
      node: 'http://localhost:9200'
    }
  }
});

// 日志上下文
logger.info({
  userId: req.user.id,
  tenantId: req.tenant.id,
  requestId: req.id
}, 'Request processed');
```

---

## 扩展测试

### 负载测试

```javascript
// 使用 k6
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 100 },
    { duration: '3m', target: 500 },
    { duration: '1m', target: 0 }
  ]
};

export default function () {
  const res = http.get('https://your-domain.com/api/tasks');
  
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500
  });
  
  sleep(1);
}
```

### 压力测试

```bash
# 使用 Apache JMeter
jmeter -n -t test-plan.jmx -l results.jtl -e -o report/
```

---

## 扩展清单

### 扩展前检查

- [ ] 数据库已优化（索引、分区）
- [ ] 缓存已配置（Redis/Memcached）
- [ ] 负载均衡已配置
- [ ] 自动扩展已配置
- [ ] 监控已配置
- [ ] 日志已集中
- [ ] 备份策略已更新
- [ ] 性能测试已通过

### 扩展后验证

- [ ] 服务正常运行
- [ ] 负载均衡工作正常
- [ ] 数据库同步正常
- [ ] 缓存命中率正常
- [ ] 性能指标达标
- [ ] 错误率在可接受范围

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Scalability Team