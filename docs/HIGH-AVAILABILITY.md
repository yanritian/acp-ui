# 高可用性配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的高可用性（HA）配置建议，确保服务在组件故障时仍然可用。

---

## 可用性目标

### 可用性级别

| 级别 | 可用性 | 每月停机时间 | 适用场景 |
|------|--------|-------------|---------|
| 99% | 99% | 7.3 小时 | 开发/测试 |
| 99.9% | 99.9% | 43.8 分钟 | 小型生产 |
| 99.99% | 99.99% | 4.38 分钟 | 中型生产 |
| 99.999% | 99.999% | 26.3 秒 | 关键生产 |

---

## 架构设计

### 单区域 HA

```
┌─────────────────────────────────────┐
│         Load Balancer (Active)      │
└─────────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│App-1  │ │App-2 │ │App-3  │
└───────┘ └──────┘ └───────┘
    │         │         │
    └─────────┼─────────┘
              │
    ┌─────────┼─────────┐
    │                   │
┌───▼───┐          ┌───▼───┐
│DB-    │          │DB-    │
│Primary│◄────────►│Replica│
└───────┘          └───────┘
    │
    └──────► Redis Cluster
```

### 多区域 HA

```
┌─────────────────────────────────────┐
│      Global Load Balancer           │
└─────────────────────────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
┌───▼───┐          ┌───▼───┐
│Region │          │Region │
│  A    │          │  B    │
└───────┘          └───────┘
    │                   │
┌───▼───┐          ┌───▼───┐
│LB-A   │          │LB-B   │
└───────┘          └───────┘
    │                   │
┌───▼───┐          ┌───▼───┐
│App-A  │          │App-B  │
└───────┘          └───────┘
```

---

## 应用层高可用

### 多实例部署

```bash
# Docker Compose 配置
version: '3.8'

services:
  app:
    image: hermes-game-operator:latest
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        max_attempts: 3
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-game-operator
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: hermes-game-operator
  template:
    metadata:
      labels:
        app: hermes-game-operator
    spec:
      containers:
      - name: app
        image: hermes-game-operator:latest
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

---

## 数据库层高可用

### PostgreSQL 主从复制

```bash
# 主库配置 (postgresql.conf)
wal_level = replica
max_wal_senders = 10
wal_keep_segments = 64

# 从库配置
hot_standby = on
```

### 自动故障转移

```bash
# 使用 Patroni
patroni:
  name: hermes-db-1
  scope: hermes-db
  restapi:
    listen: 0.0.0.0:8008
    connect_address: 10.0.0.1:8008
  postgresql:
    listen: 0.0.0.0:5432
    connect_address: 10.0.0.1:5432
    data_dir: /var/lib/postgresql/15/main
```

---

## 缓存层高可用

### Redis Sentinel

```bash
# Sentinel 配置
sentinel monitor hermes-redis 10.0.0.1 6379 2
sentinel down-after-milliseconds hermes-redis 5000
sentinel failover-timeout hermes-redis 60000
sentinel parallel-syncs hermes-redis 1
```

### Redis Cluster

```bash
# 创建集群
redis-cli --cluster create \
  10.0.0.1:6379 \
  10.0.0.2:6379 \
  10.0.0.3:6379 \
  --cluster-replicas 1
```

---

## 负载均衡

### Nginx 配置

```nginx
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
        proxy_next_upstream error timeout invalid_header http_500;
        proxy_connect_timeout 5s;
        proxy_read_timeout 30s;
    }

    location /health {
        proxy_pass http://hermes_backend/health;
        proxy_next_upstream error timeout invalid_header http_500;
    }
}
```

### AWS ALB 配置

```yaml
# CloudFormation 模板
Type: AWS::ElasticLoadBalancingV2::LoadBalancer
Properties:
  Name: hermes-alb
  Scheme: internet-facing
  Type: application
  Subnets:
    - !Ref PublicSubnet1
    - !Ref PublicSubnet2
  HealthCheckPath: /health
  HealthCheckIntervalSeconds: 30
  HealthyThresholdCount: 2
  UnhealthyThresholdCount: 3
```

---

## 监控和告警

### 健康检查

```bash
# 应用健康检查
curl -f https://your-domain.com/health || exit 1

# 数据库健康检查
psql -h localhost -U hermes -d hermes -c "SELECT 1;" || exit 1

# Redis 健康检查
redis-cli ping || exit 1
```

### 监控指标

```yaml
# Prometheus 告警规则
groups:
  - name: hermes-ha
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"

      - alert: InstanceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Instance {{ $labels.instance }} down"
```

---

## 故障转移测试

### 定期测试

```bash
# 每月执行故障转移测试
#!/bin/bash

# 1. 停止主实例
sudo systemctl stop hermes-game-operator

# 2. 验证自动故障转移
sleep 10
curl https://your-domain.com/health

# 3. 恢复主实例
sudo systemctl start hermes-game-operator

# 4. 验证服务恢复
curl https://your-domain.com/health
```

### 混沌工程

```bash
# 使用 Chaos Monkey
# 随机终止实例以测试弹性
chaos-monkey --assaults-kill-count=1 --assaults-kill-frequency=3600
```

---

## 数据复制

### 跨区域复制

```bash
# PostgreSQL 逻辑复制
CREATE SUBSCRIPTION hermes_sub
CONNECTION 'host=remote-db port=5432 dbname=hermes user=replication'
PUBLICATION hermes_pub;
```

### 文件同步

```bash
# 使用 rsync 同步文件
rsync -avz /var/uploads/ remote:/var/uploads/

# 使用 S3 跨区域复制
aws s3api put-bucket-replication \
  --bucket hermes-data \
  --replication-configuration file://replication.json
```

---

## 备份和恢复

### 自动备份

```bash
# 每小时备份
0 * * * * /usr/local/bin/hermes-backup.sh

# 备份脚本
#!/bin/bash
BACKUP_DIR="/var/backups/hermes"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 数据库备份
pg_dump -h localhost -U hermes hermes | gzip > \
  "$BACKUP_DIR/db_$TIMESTAMP.sql.gz"

# 上传到云存储
aws s3 cp "$BACKUP_DIR/db_$TIMESTAMP.sql.gz" \
  s3://hermes-backups/db/
```

---

## 配置清单

### 高可用性检查清单

- [ ] 应用层：至少 2 个实例
- [ ] 数据库：主从复制配置
- [ ] 缓存：Redis Sentinel 或 Cluster
- [ ] 负载均衡：健康检查配置
- [ ] 监控：所有组件监控
- [ ] 告警：关键指标告警
- [ ] 备份：自动备份配置
- [ ] 测试：定期故障转移测试
- [ ] 文档：HA 配置文档
- [ ] 演练：定期演练

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team