# 微服务部署指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 微服务架构

```
┌─────────────────────────────────────┐
│         API Gateway                 │
└─────────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│ Task  │ │Event │ │ Auth  │
│Service│ │Serv. │ │Service│
└───────┘ └──────┘ └───────┘
    │         │         │
    └─────────┼─────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│  DB   │ │Cache │ │  MQ   │
│Primary│ │Redis │ │Kafka  │
└───────┘ └──────┘ └───────┘
```

---

## 服务拆分

### 服务边界

| 服务 | 职责 | 数据库 | 通信 |
|------|------|--------|------|
| **Task Service** | 任务管理 | PostgreSQL | REST |
| **Event Service** | 事件流 | PostgreSQL | REST |
| **Auth Service** | 认证授权 | PostgreSQL | gRPC |
| **Notification Service** | 通知 | MongoDB | Kafka |
| **Analytics Service** | 分析 | ClickHouse | Kafka |

### 服务通信

```yaml
# 同步通信 (REST/gRPC)
- Task → Auth: 验证用户
- Task → Event: 创建事件
- Event → Notification: 发送通知

# 异步通信 (Kafka)
- Task → Notification: 任务通知
- Event → Analytics: 分析数据
```

---

## Task Service

### 基本配置

```yaml
# task-service-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-service
  namespace: hermes
spec:
  replicas: 3
  selector:
    matchLabels:
      app: task-service
  template:
    metadata:
      labels:
        app: task-service
    spec:
      containers:
      - name: task-service
        image: hermes-task-service:v0.1.0-alpha
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: grpc
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: task-service-secrets
              key: database-url
        - name: KAFKA_BROKERS
          value: kafka-1:9092,kafka-2:9092,kafka-3:9092
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

### API 定义

```yaml
# task-service-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: task-service
  namespace: hermes
spec:
  selector:
    app: task-service
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: grpc
    port: 9090
    targetPort: 9090
```

---

## Event Service

### 基本配置

```yaml
# event-service-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: event-service
  namespace: hermes
spec:
  replicas: 2
  selector:
    matchLabels:
      app: event-service
  template:
    metadata:
      labels:
        app: event-service
    spec:
      containers:
      - name: event-service
        image: hermes-event-service:v0.1.0-alpha
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: event-service-secrets
              key: database-url
        - name: KAFKA_BROKERS
          value: kafka-1:9092,kafka-2:9092,kafka-3:9092
```

---

## Auth Service

### 基本配置

```yaml
# auth-service-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
  namespace: hermes
spec:
  replicas: 2
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: hermes-auth-service:v0.1.0-alpha
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: grpc
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: auth-service-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-service-secrets
              key: jwt-secret
```

---

## API Gateway

### Kong 配置

```yaml
# api-gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: hermes
spec:
  replicas: 2
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: kong
        image: kong:3.4
        ports:
        - containerPort: 8000
          name: proxy
        - containerPort: 8443
          name: proxy-ssl
        - containerPort: 8001
          name: admin
        env:
        - name: KONG_DATABASE
          value: "postgres"
        - name: KONG_PG_HOST
          value: kong-db
        - name: KONG_PG_PASSWORD
          valueFrom:
            secretKeyRef:
              name: kong-secrets
              key: password
```

### 路由配置

```yaml
# kong-route.yaml
apiVersion: configuration.konghq.com/v1
kind: KongIngress
metadata:
  name: task-service-route
  namespace: hermes
route:
  paths:
  - /api/tasks
  strip_path: false
  protocols:
  - https
```

---

## 服务发现

### Kubernetes DNS

```bash
# 服务间通信
http://task-service.hermes.svc.cluster.local:80
http://event-service.hermes.svc.cluster.local:80
http://auth-service.hermes.svc.cluster.local:9090
```

### Consul 集成

```yaml
# consul-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: consul-config
data:
  config.json: |
    {
      "service": {
        "name": "task-service",
        "port": 8080,
        "check": {
          "http": "http://localhost:8080/health",
          "interval": "10s"
        }
      }
    }
```

---

## 配置管理

### ConfigMap

```yaml
# task-service-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: task-service-config
  namespace: hermes
data:
  config.json: |
    {
      "server": {
        "port": 8080
      },
      "database": {
        "pool": {
          "min": 5,
          "max": 20
        }
      },
      "kafka": {
        "brokers": ["kafka-1:9092"],
        "groupId": "task-service"
      }
    }
```

### Secret

```yaml
# task-service-secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: task-service-secrets
  namespace: hermes
type: Opaque
data:
  database-url: cG9zdGdyZXM6Ly90YXNrO...
  kafka-password: a2Fma2EtcGFzc3dvcmQ=
```

---

## 数据库管理

### 独立数据库

```yaml
# task-service-db.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: task-service-db
  namespace: hermes
spec:
  serviceName: task-service-db
  replicas: 1
  selector:
    matchLabels:
      app: task-service-db
  template:
    metadata:
      labels:
        app: task-service-db
    spec:
      containers:
      - name: postgres
        image: postgres:15
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: task_service
        - name: POSTGRES_USER
          value: task
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: task-service-secrets
              key: database-password
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 50Gi
```

---

## 监控

### Prometheus 指标

```yaml
# service-monitor.yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: task-service
  namespace: hermes
spec:
  selector:
    matchLabels:
      app: task-service
  endpoints:
  - port: http
    interval: 15s
    path: /metrics
```

### Jaeger 追踪

```yaml
# jaeger-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: jaeger-config
data:
  config.json: |
    {
      "service": {
        "name": "task-service"
      },
      "sampler": {
        "type": "const",
        "param": 1
      },
      "reporter": {
        "logSpans": true,
        "collectorEndpoint": "http://jaeger:14268/api/traces"
      }
    }
```

---

## 部署策略

### 滚动更新

```yaml
# rolling-update.yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1
    maxUnavailable: 0
```

### 蓝绿部署

```yaml
# blue-green.yaml
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-service-blue
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: task-service-green
```

---

## 最佳实践

### 1. 服务自治

- 每个服务有自己的数据库
- 服务间通过 API 通信
- 避免共享数据库

### 2. 容错设计

- 实现熔断器
- 实现重试机制
- 实现降级策略

### 3. 可观测性

- 集中式日志
- 分布式追踪
- 指标监控

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Microservices Team