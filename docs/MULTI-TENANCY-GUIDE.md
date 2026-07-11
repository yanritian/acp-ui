# 多租户指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的多租户架构和实施方法。

---

## 租户隔离模型

### 数据隔离级别

| 级别 | 描述 | 优点 | 缺点 |
|------|------|------|------|
| **共享数据库** | 所有租户共享一个数据库 | 成本低，易维护 | 隔离性差 |
| **共享模式** | 共享数据库，独立模式 | 平衡隔离和成本 | 需要模式管理 |
| **独立数据库** | 每个租户独立数据库 | 完全隔离 | 成本高 |

### 推荐方案

```
┌─────────────────────────────────────┐
│      Application Layer              │
│  (Tenant-aware middleware)          │
└─────────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│Tenant │ │Tenant │ │Tenant │
│  A    │ │  B    │ │  C    │
└───────┘ └───────┘ └───────┘
    │         │         │
    └─────────┼─────────┘
              │
    ┌─────────┼─────────┐
    │                   │
┌───▼───┐          ┌───▼───┐
│Shared │          │Shared │
│  DB   │          │Cache  │
└───────┘          └───────┘
```

---

## 租户标识

### 租户 ID 生成

```javascript
// 生成租户 ID
function generateTenantId() {
  return 'tenant_' + crypto.randomUUID();
}

// 租户模型
const Tenant = {
  tenant_id: 'tenant_abc123',
  name: 'Acme Corp',
  plan: 'enterprise',
  created_at: '2026-07-11T00:00:00Z',
  settings: {
    max_users: 1000,
    max_tasks: 10000,
    features: ['advanced_analytics', 'custom_branding']
  }
};
```

### 租户上下文

```javascript
// 租户上下文中间件
function tenantMiddleware(req, res, next) {
  const tenantId = req.headers['x-tenant-id'] || 
                   req.subdomain ||
                   req.query.tenant_id;
  
  if (!tenantId) {
    return res.status(400).json({
      error: 'Tenant ID required'
    });
  }
  
  // 加载租户信息
  const tenant = await loadTenant(tenantId);
  
  if (!tenant) {
    return res.status(404).json({
      error: 'Tenant not found'
    });
  }
  
  // 设置租户上下文
  req.tenant = tenant;
  
  next();
}
```

---

## 数据隔离

### 数据库模式隔离

```sql
-- 为每个租户创建模式
CREATE SCHEMA tenant_abc123;

-- 在租户模式中创建表
CREATE TABLE tenant_abc123.tasks (
  task_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  goal TEXT NOT NULL,
  status VARCHAR(50) NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_tasks_tenant ON tenant_abc123.tasks(tenant_id);
```

### 查询过滤

```javascript
// 租户感知的查询
async function getTasks(tenantId, filters = {}) {
  const query = `
    SELECT * FROM tasks 
    WHERE tenant_id = $1
    ${filters.status ? 'AND status = $2' : ''}
    ORDER BY created_at DESC
    LIMIT $${filters.status ? 3 : 2}
  `;
  
  const params = [tenantId];
  if (filters.status) {
    params.push(filters.status);
  }
  params.push(filters.limit || 50);
  
  const { rows } = await pool.query(query, params);
  return rows;
}
```

---

## 租户管理

### 租户创建

```javascript
async function createTenant(tenantData) {
  const client = await pool.connect();
  
  try {
    await client.query('BEGIN');
    
    // 创建租户记录
    const { rows: [tenant] } = await client.query(`
      INSERT INTO tenants (name, plan, settings)
      VALUES ($1, $2, $3)
      RETURNING *
    `, [tenantData.name, tenantData.plan, tenantData.settings]);
    
    // 创建租户模式
    await client.query(`
      CREATE SCHEMA tenant_${tenant.tenant_id}
    `);
    
    // 创建租户表
    await client.query(`
      CREATE TABLE tenant_${tenant.tenant_id}.tasks (
        LIKE public.tasks INCLUDING ALL
      )
    `);
    
    await client.query('COMMIT');
    
    return tenant;
  } catch (error) {
    await client.query('ROLLBACK');
    throw error;
  } finally {
    client.release();
  }
}
```

### 租户删除

```javascript
async function deleteTenant(tenantId) {
  const client = await pool.connect();
  
  try {
    await client.query('BEGIN');
    
    // 删除租户模式
    await client.query(`
      DROP SCHEMA tenant_${tenantId} CASCADE
    `);
    
    // 删除租户记录
    await client.query(`
      DELETE FROM tenants WHERE tenant_id = $1
    `, [tenantId]);
    
    await client.query('COMMIT');
  } catch (error) {
    await client.query('ROLLBACK');
    throw error;
  } finally {
    client.release();
  }
}
```

---

## 租户配置

### 功能开关

```javascript
// 租户功能配置
const tenantFeatures = {
  'tenant_abc123': {
    advanced_analytics: true,
    custom_branding: true,
    api_access: true,
    webhooks: true
  },
  'tenant_def456': {
    advanced_analytics: false,
    custom_branding: false,
    api_access: true,
    webhooks: false
  }
};

// 检查功能
function hasFeature(tenantId, feature) {
  const features = tenantFeatures[tenantId] || {};
  return features[feature] === true;
}
```

### 配额管理

```javascript
// 租户配额
const tenantQuotas = {
  'tenant_abc123': {
    max_users: 1000,
    max_tasks: 10000,
    max_storage_gb: 100,
    api_calls_per_day: 100000
  }
};

// 检查配额
async function checkQuota(tenantId, resource) {
  const quota = tenantQuotas[tenantId];
  const usage = await getUsage(tenantId, resource);
  
  return usage < quota[`max_${resource}`];
}
```

---

## 租户计费

### 计费模型

```javascript
// 计费计划
const pricingPlans = {
  starter: {
    monthly_price: 29,
    included_users: 5,
    included_tasks: 100,
    price_per_additional_user: 5,
    price_per_additional_task: 0.1
  },
  professional: {
    monthly_price: 99,
    included_users: 25,
    included_tasks: 1000,
    price_per_additional_user: 3,
    price_per_additional_task: 0.05
  },
  enterprise: {
    monthly_price: 499,
    included_users: 100,
    included_tasks: 10000,
    price_per_additional_user: 2,
    price_per_additional_task: 0.02
  }
};

// 计算账单
async function calculateBill(tenantId) {
  const tenant = await getTenant(tenantId);
  const plan = pricingPlans[tenant.plan];
  const usage = await getUsage(tenantId);
  
  let total = plan.monthly_price;
  
  // 计算额外用户费用
  if (usage.users > plan.included_users) {
    total += (usage.users - plan.included_users) * plan.price_per_additional_user;
  }
  
  // 计算额外任务费用
  if (usage.tasks > plan.included_tasks) {
    total += (usage.tasks - plan.included_tasks) * plan.price_per_additional_task;
  }
  
  return total;
}
```

---

## 租户监控

### 租户指标

```javascript
// 收集租户指标
async function collectTenantMetrics(tenantId) {
  const metrics = {
    tenant_id: tenantId,
    timestamp: new Date().toISOString(),
    active_users: await getActiveUsers(tenantId),
    total_tasks: await getTotalTasks(tenantId),
    api_calls: await getApiCalls(tenantId),
    storage_used_gb: await getStorageUsed(tenantId),
    response_time_ms: await getAvgResponseTime(tenantId)
  };
  
  // 存储指标
  await storeMetrics(metrics);
  
  return metrics;
}
```

### 租户告警

```javascript
// 租户告警规则
const tenantAlerts = [
  {
    name: 'High API Usage',
    condition: (metrics) => metrics.api_calls > 90000,
    severity: 'warning',
    message: 'API usage approaching limit'
  },
  {
    name: 'Storage Full',
    condition: (metrics) => metrics.storage_used_gb > 90,
    severity: 'critical',
    message: 'Storage almost full'
  }
];

// 检查告警
async function checkTenantAlerts(tenantId) {
  const metrics = await collectTenantMetrics(tenantId);
  
  for (const alert of tenantAlerts) {
    if (alert.condition(metrics)) {
      await sendAlert(tenantId, alert);
    }
  }
}
```

---

## 租户 API

### 租户管理 API

```javascript
// 创建租户
app.post('/api/tenants', async (req, res) => {
  try {
    const tenant = await createTenant(req.body);
    res.status(201).json(tenant);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 获取租户
app.get('/api/tenants/:tenantId', async (req, res) => {
  try {
    const tenant = await getTenant(req.params.tenantId);
    res.json(tenant);
  } catch (error) {
    res.status(404).json({ error: 'Tenant not found' });
  }
});

// 更新租户
app.put('/api/tenants/:tenantId', async (req, res) => {
  try {
    const tenant = await updateTenant(req.params.tenantId, req.body);
    res.json(tenant);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// 删除租户
app.delete('/api/tenants/:tenantId', async (req, res) => {
  try {
    await deleteTenant(req.params.tenantId);
    res.status(204).send();
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});
```

---

## 租户安全

### 租户隔离验证

```javascript
// 确保租户只能访问自己的数据
function validateTenantAccess(tenantId, resourceId) {
  return async (req, res, next) => {
    const resource = await getResource(resourceId);
    
    if (resource.tenant_id !== tenantId) {
      return res.status(403).json({
        error: 'Access denied'
      });
    }
    
    next();
  };
}

// 使用
app.get('/api/tasks/:taskId', 
  tenantMiddleware,
  validateTenantAccess(req.tenant.tenant_id, req.params.taskId),
  async (req, res) => {
    // 处理请求
  }
);
```

### 租户认证

```javascript
// 租户特定的认证
async function authenticateTenant(req, res, next) {
  const token = req.headers.authorization?.replace('Bearer ', '');
  
  if (!token) {
    return res.status(401).json({ error: 'Token required' });
  }
  
  try {
    const decoded = jwt.verify(token, process.env.JWT_SECRET);
    
    // 验证租户
    if (decoded.tenant_id !== req.tenant.tenant_id) {
      return res.status(403).json({ error: 'Invalid tenant' });
    }
    
    req.user = decoded;
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid token' });
  }
}
```

---

## 租户迁移

### 租户数据迁移

```javascript
// 迁移租户数据到新数据库
async function migrateTenant(tenantId, targetDatabase) {
  const sourceClient = await pool.connect();
  const targetClient = await targetDatabase.connect();
  
  try {
    await sourceClient.query('BEGIN');
    await targetClient.query('BEGIN');
    
    // 导出租户数据
    const { rows: tasks } = await sourceClient.query(`
      SELECT * FROM tenant_${tenantId}.tasks
    `);
    
    // 导入到新数据库
    for (const task of tasks) {
      await targetClient.query(`
        INSERT INTO tasks (tenant_id, goal, status, created_at)
        VALUES ($1, $2, $3, $4)
      `, [tenantId, task.goal, task.status, task.created_at]);
    }
    
    await sourceClient.query('COMMIT');
    await targetClient.query('COMMIT');
    
  } catch (error) {
    await sourceClient.query('ROLLBACK');
    await targetClient.query('ROLLBACK');
    throw error;
  } finally {
    sourceClient.release();
    targetClient.release();
  }
}
```

---

## 最佳实践

### 1. 始终过滤租户

```javascript
// ❌ 不推荐：忘记过滤租户
const tasks = await pool.query('SELECT * FROM tasks');

// ✅ 推荐：始终过滤租户
const tasks = await pool.query(
  'SELECT * FROM tasks WHERE tenant_id = $1',
  [tenantId]
);
```

### 2. 使用租户上下文

```javascript
// 使用请求范围的租户上下文
const tenantContext = new Map();

function getTenantContext(req) {
  if (!tenantContext.has(req)) {
    tenantContext.set(req, {
      tenantId: req.tenant.tenant_id,
      startTime: Date.now()
    });
  }
  
  return tenantContext.get(req);
}
```

### 3. 定期清理租户数据

```javascript
// 清理不活跃租户的数据
async function cleanupInactiveTenants() {
  const inactiveTenants = await getInactiveTenants(90); // 90天不活跃
  
  for (const tenant of inactiveTenants) {
    await archiveTenantData(tenant.tenant_id);
    await notifyTenant(tenant, 'Your account has been archived');
  }
}
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Multi-Tenancy Team