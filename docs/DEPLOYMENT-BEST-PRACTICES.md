# 部署最佳实践

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 部署策略

### 蓝绿部署

```
┌─────────────────┐
│  Load Balancer  │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼───┐
│ Blue  │ │Green │
│(Active)│ │(Idle)│
└───────┘ └──────┘
```

**优点**:
- 零停机时间
- 快速回滚
- 完整测试环境

**实施**:
```bash
#!/bin/bash
# 部署到绿色环境
kubectl apply -f deployment-green.yaml

# 等待健康检查
kubectl wait --for=condition=ready pod -l app=hermes-green

# 切换流量
kubectl patch service hermes-service -p '{"spec":{"selector":{"app":"hermes-green"}}}'

# 验证
curl https://your-domain.com/health
```

---

### 金丝雀部署

```
┌─────────────────┐
│  Load Balancer  │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼───┐
│ 95%   │ │ 5%   │
│Stable │ │Canary│
└───────┘ └──────┘
```

**优点**:
- 渐进式发布
- 风险最小化
- 实时监控

**实施**:
```yaml
# deployment-canary.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermes
      track: canary
  template:
    metadata:
      labels:
        app: hermes
        track: canary
    spec:
      containers:
      - name: hermes
        image: hermes-game-operator:v2.0.0
```

```bash
# 逐步增加流量
kubectl set weights deployment hermes-stable=95 hermes-canary=5
kubectl set weights deployment hermes-stable=90 hermes-canary=10
kubectl set weights deployment hermes-stable=50 hermes-canary=50
kubectl set weights deployment hermes-stable=0 hermes-canary=100
```

---

### 滚动更新

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-game-operator
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  replicas: 3
  template:
    spec:
      containers:
      - name: hermes
        image: hermes-game-operator:v2.0.0
```

**优点**:
- 简单
- 无需额外基础设施
- 自动处理

---

## 环境管理

### 多环境配置

```
environments/
├── dev/
│   ├── values.yaml
│   └── secrets.yaml
├── staging/
│   ├── values.yaml
│   └── secrets.yaml
└── production/
    ├── values.yaml
    └── secrets.yaml
```

### 配置管理

```yaml
# values.yaml
environment: production

app:
  name: hermes-game-operator
  version: 0.1.0-alpha
  replicas: 3

database:
  host: db.example.com
  port: 5432
  name: hermes
  
cache:
  host: redis.example.com
  port: 6379

logging:
  level: info
  format: json
```

---

## 回滚策略

### 自动回滚

```yaml
# deployment.yaml
spec:
  strategy:
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: hermes
        image: hermes-game-operator:v2.0.0
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          failureThreshold: 3
```

### 手动回滚

```bash
# 查看部署历史
kubectl rollout history deployment hermes-game-operator

# 回滚到上一个版本
kubectl rollout undo deployment hermes-game-operator

# 回滚到特定版本
kubectl rollout undo deployment hermes-game-operator --to-revision=3

# 验证回滚
kubectl rollout status deployment hermes-game-operator
```

---

## 监控和告警

### 健康检查

```javascript
// health.js
app.get('/health', async (req, res) => {
  try {
    // 检查数据库
    await pool.query('SELECT 1');
    
    // 检查 Redis
    await redis.ping();
    
    // 检查外部服务
    await checkExternalServices();
    
    res.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: process.env.APP_VERSION
    });
  } catch (error) {
    res.status(503).json({
      status: 'unhealthy',
      error: error.message
    });
  }
});
```

### 就绪检查

```javascript
// ready.js
app.get('/ready', async (req, res) => {
  try {
    // 检查数据库连接池
    if (pool.waitingCount > 0) {
      throw new Error('Connection pool exhausted');
    }
    
    // 检查内存使用
    const memoryUsage = process.memoryUsage();
    if (memoryUsage.heapUsed > memoryUsage.heapTotal * 0.9) {
      throw new Error('Memory usage too high');
    }
    
    res.json({
      status: 'ready',
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    res.status(503).json({
      status: 'not ready',
      error: error.message
    });
  }
});
```

---

## 安全实践

### 密钥管理

```bash
# 使用 Kubernetes Secrets
kubectl create secret generic hermes-secrets \
  --from-literal=DATABASE_URL=postgres://... \
  --from-literal=REDIS_URL=redis://... \
  --from-literal=JWT_SECRET=...

# 使用外部密钥管理
kubectl create secret generic hermes-secrets \
  --from-literal=VAULT_TOKEN=...
```

### 网络策略

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: hermes-network-policy
spec:
  podSelector:
    matchLabels:
      app: hermes
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432
```

---

## 性能优化

### 资源限制

```yaml
# deployment.yaml
spec:
  template:
    spec:
      containers:
      - name: hermes
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

### 自动扩展

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: hermes-hpa
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
```

---

## 部署清单

### 部署前检查

- [ ] 代码已审查
- [ ] 测试已通过
- [ ] 文档已更新
- [ ] 配置已验证
- [ ] 密钥已配置
- [ ] 监控已配置
- [ ] 回滚计划已准备
- [ ] 通知已发送

### 部署中检查

- [ ] 部署已启动
- [ ] 健康检查通过
- [ ] 就绪检查通过
- [ ] 监控指标正常
- [ ] 日志无错误
- [ ] 性能指标正常

### 部署后检查

- [ ] 服务正常运行
- [ ] 功能测试通过
- [ ] 性能测试通过
- [ ] 监控正常
- [ ] 日志正常
- [ ] 用户反馈正常

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team