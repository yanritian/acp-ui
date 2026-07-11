# 容量规划指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的容量规划建议，帮助您根据使用场景选择合适的资源配置。

---

## 部署规模

### 小型部署

**适用场景**:
- 开发/测试环境
- 小型团队（< 10 用户）
- 低并发（< 100 请求/秒）

**推荐配置**:

| 组件 | 规格 | 数量 |
|------|------|------|
| 应用服务器 | 2 CPU, 4GB RAM, 50GB SSD | 1 |
| 数据库 | 2 CPU, 4GB RAM, 100GB SSD | 1 |
| Redis | 1 CPU, 2GB RAM | 1 |
| 负载均衡 | - | - |

**预计成本**: ~$100/月

---

### 中型部署

**适用场景**:
- 生产环境
- 中型团队（10-100 用户）
- 中等并发（100-1000 请求/秒）

**推荐配置**:

| 组件 | 规格 | 数量 |
|------|------|------|
| 应用服务器 | 4 CPU, 8GB RAM, 100GB SSD | 2 |
| 数据库 | 4 CPU, 8GB RAM, 200GB SSD | 2 (主从) |
| Redis | 2 CPU, 4GB RAM | 3 (Sentinel) |
| 负载均衡 | - | 1 |

**预计成本**: ~$500/月

---

### 大型部署

**适用场景**:
- 企业级生产环境
- 大型团队（100-1000 用户）
- 高并发（1000-10000 请求/秒）

**推荐配置**:

| 组件 | 规格 | 数量 |
|------|------|------|
| 应用服务器 | 8 CPU, 16GB RAM, 200GB SSD | 5+ |
| 数据库 | 8 CPU, 16GB RAM, 500GB SSD | 3 (主从) |
| Redis | 4 CPU, 8GB RAM | 6 (Cluster) |
| 负载均衡 | - | 2 (主备) |
| 缓存层 | - | 2 |

**预计成本**: ~$2000/月

---

### 超大型部署

**适用场景**:
- 全球部署
- 超大型团队（1000+ 用户）
- 超高并发（10000+ 请求/秒）

**推荐配置**:

| 组件 | 规格 | 数量 |
|------|------|------|
| 应用服务器 | 16 CPU, 32GB RAM, 500GB SSD | 10+ |
| 数据库 | 16 CPU, 32GB RAM, 1TB SSD | 5+ (分片) |
| Redis | 8 CPU, 16GB RAM | 12+ (Cluster) |
| 负载均衡 | - | 3+ (多区域) |
| CDN | - | 全球 |
| 对象存储 | - | 多区域 |

**预计成本**: ~$10000+/月

---

## 资源估算

### CPU 需求

| 并发用户 | CPU 核心数 | 说明 |
|----------|-----------|------|
| 1-10 | 2 | 小型部署 |
| 10-50 | 4 | 中型部署 |
| 50-200 | 8 | 大型部署 |
| 200-1000 | 16 | 超大型部署 |
| 1000+ | 32+ | 企业级 |

### 内存需求

| 数据类型 | 内存需求 | 说明 |
|----------|---------|------|
| 应用服务器 | 2GB/100 用户 | Node.js 运行时 |
| 数据库 | 4GB/100GB 数据 | PostgreSQL |
| Redis | 1GB/10GB 缓存 | Redis |
| 缓存 | 2GB/100GB 对象 | CDN 缓存 |

### 存储需求

| 数据类型 | 存储需求 | 说明 |
|----------|---------|------|
| 数据库 | 10GB + 1GB/1000 任务 | PostgreSQL |
| 日志 | 1GB/天 | 应用日志 |
| 备份 | 数据库大小 × 2 | 完整备份 |
| 对象存储 | 按使用量 | 文件上传 |

---

## 扩展策略

### 垂直扩展

**适用场景**:
- 单实例性能瓶颈
- 简单扩展需求
- 预算有限

**扩展步骤**:

1. **增加 CPU**
   ```bash
   # 云服务商控制台
   # 增加实例 CPU 核心数
   ```

2. **增加内存**
   ```bash
   # 云服务商控制台
   # 增加实例内存
   ```

3. **增加存储**
   ```bash
   # 扩展磁盘空间
   sudo resize2fs /dev/sda1
   ```

**优点**:
- ✅ 简单快速
- ✅ 无需修改架构

**缺点**:
- ❌ 有上限
- ❌ 需要停机

---

### 水平扩展

**适用场景**:
- 高并发需求
- 高可用性需求
- 大规模部署

**扩展步骤**:

1. **添加应用服务器**
   ```bash
   # 使用 Terraform 添加实例
   terraform apply -var="instance_count=3"
   ```

2. **配置负载均衡**
   ```bash
   # 添加新实例到负载均衡
   aws elbv2 register-targets \
     --target-group-arn $TARGET_GROUP_ARN \
     --targets Id=$NEW_INSTANCE_ID
   ```

3. **扩展数据库**
   ```bash
   # 添加只读副本
   aws rds create-db-instance-read-replica \
     --db-instance-identifier hermes-replica \
     --source-db-instance-identifier hermes-primary
   ```

4. **扩展 Redis**
   ```bash
   # 添加 Redis 节点
   aws elasticache create-cache-cluster \
     --cache-cluster-id hermes-redis-003 \
     --replication-group-id hermes-redis
   ```

**优点**:
- ✅ 无上限
- ✅ 高可用
- ✅ 在线扩展

**缺点**:
- ❌ 复杂
- ❌ 需要负载均衡

---

## 监控指标

### 关键指标

| 指标 | 阈值 | 说明 |
|------|------|------|
| CPU 使用率 | > 70% | 需要扩展 |
| 内存使用率 | > 80% | 需要扩展 |
| 磁盘使用率 | > 75% | 需要扩展 |
| 响应时间 | > 500ms | 需要优化 |
| 错误率 | > 1% | 需要修复 |

### 监控工具

```yaml
# Prometheus 配置
scrape_configs:
  - job_name: 'hermes-game-operator'
    metrics_path: '/metrics'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:8080']

# 告警规则
groups:
  - name: hermes-alerts
    rules:
      - alert: HighCPU
        expr: node_cpu_seconds_total{mode="idle"} < 0.3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
```

---

## 成本优化

### 预留实例

**节省**: 30-60%

```bash
# AWS 预留实例
aws ec2 purchase-reserved-instances-offering \
  --reserved-instances-offering-id $OFFERING_ID \
  --instance-count 3
```

### 自动扩展

**节省**: 20-40%

```yaml
# AWS Auto Scaling
AutoScalingGroup:
  MinSize: 2
  MaxSize: 10
  DesiredCapacity: 3
  TargetTrackingConfiguration:
    TargetValue: 70
    PredefinedMetricSpecification:
      PredefinedMetricType: ASGAverageCPUUtilization
```

### 存储优化

**节省**: 30-50%

```bash
# 使用 S3 生命周期策略
aws s3api put-bucket-lifecycle-configuration \
  --bucket hermes-data \
  --lifecycle-configuration file://lifecycle.json
```

---

## 容量规划清单

### 当前使用情况

- [ ] 用户数量: _____
- [ ] 并发用户: _____
- [ ] 每日请求数: _____
- [ ] 数据量: _____
- [ ] 存储使用: _____

### 增长预测

- [ ] 月度增长率: _____%
- [ ] 年度增长预测: _____
- [ ] 峰值场景: _____

### 资源需求

- [ ] CPU 核心数: _____
- [ ] 内存 (GB): _____
- [ ] 存储 (GB): _____
- [ ] 网络带宽 (Mbps): _____

### 扩展计划

- [ ] 扩展触发点: _____
- [ ] 扩展方式: 垂直/水平
- [ ] 扩展时间窗口: _____
- [ ] 预算限制: $_____

---

## 云服务商建议

### AWS

| 组件 | 服务 | 实例类型 |
|------|------|---------|
| 应用 | ECS | t3.medium |
| 数据库 | RDS | db.t3.medium |
| 缓存 | ElastiCache | cache.t3.medium |
| 负载均衡 | ALB | - |
| 存储 | S3 | - |

### GCP

| 组件 | 服务 | 实例类型 |
|------|------|---------|
| 应用 | Cloud Run | - |
| 数据库 | Cloud SQL | db-custom-2-7680 |
| 缓存 | Memorystore | redis-small |
| 负载均衡 | Cloud LB | - |
| 存储 | Cloud Storage | - |

### Azure

| 组件 | 服务 | 实例类型 |
|------|------|---------|
| 应用 | App Service | S1 |
| 数据库 | Azure SQL | S2 |
| 缓存 | Azure Cache | Basic C1 |
| 负载均衡 | Azure LB | - |
| 存储 | Blob Storage | - |

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team