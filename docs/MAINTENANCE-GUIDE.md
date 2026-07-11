# 维护指南

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [日常维护](#日常维护)
2. [依赖更新](#依赖更新)
3. [性能监控](#性能监控)
4. [错误处理](#错误处理)
5. [备份与恢复](#备份与恢复)
6. [版本发布](#版本发布)

---

## 日常维护

### 日志检查

#### 桌面应用

日志位置：
- **Windows**: `%APPDATA%\acp-ui\logs\`
- **macOS**: `~/Library/Application Support/acp-ui/logs/`
- **Linux**: `~/.config/acp-ui/logs/`

检查最近日志：
```bash
# Windows
type %APPDATA%\acp-ui\logs\app.log | findstr /I "error"

# macOS/Linux
tail -f ~/.config/acp-ui/logs/app.log | grep -i error
```

#### Web 应用

使用浏览器开发者工具：
1. 打开 DevTools (F12)
2. 切换到 Console 标签
3. 过滤错误信息

### 清理缓存

```bash
# 清理 npm 缓存
npm cache clean --force

# 清理构建产物
rm -rf dist node_modules/.vite

# 重新安装依赖
rm -rf node_modules package-lock.json
npm install
```

### 数据库维护

```sql
-- 检查数据库大小
SELECT pg_size_pretty(pg_database_size('acp_ui'));

-- 清理过期数据
DELETE FROM events WHERE created_at < NOW() - INTERVAL '30 days';

-- 重建索引
REINDEX DATABASE acp_ui;

-- 分析表
ANALYZE;
```

---

## 依赖更新

### 检查更新

```bash
# 检查过期依赖
npm outdated

# 检查安全漏洞
npm audit

# 检查所有依赖
npm list --depth=0
```

### 更新依赖

```bash
# 更新次要版本
npm update

# 更新主要版本（需要手动）
npm install package@latest

# 更新所有依赖（谨慎使用）
npx npm-check-updates -u
npm install
```

### 更新后检查

```bash
# 运行测试
npm test

# 类型检查
npm run typecheck

# 构建检查
npm run build

# E2E 测试
npm run test:e2e
```

---

## 性能监控

### 前端性能

#### 使用 Chrome DevTools

1. 打开 Performance 标签
2. 点击 Record
3. 执行操作
4. 停止录制
5. 分析结果

#### 关键指标

- **FCP** (First Contentful Paint): < 1s
- **LCP** (Largest Contentful Paint): < 2.5s
- **FID** (First Input Delay): < 100ms
- **CLS** (Cumulative Layout Shift): < 0.1

#### 内存监控

```javascript
// 检查内存使用
console.log(`Used: ${performance.memory.usedJSHeapSize / 1048576} MB`);
console.log(`Total: ${performance.memory.totalJSHeapSize / 1048576} MB`);
```

### 后端性能

#### Rust 性能分析

```bash
# 使用 cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bin acp-ui

# 使用 perf
perf record --call-graph dwarf ./target/release/acp-ui
perf report
```

#### 数据库性能

```sql
-- 慢查询日志
SET GLOBAL slow_query_log = 'ON';
SET GLOBAL long_query_time = 1;

-- 查看慢查询
SHOW VARIABLES LIKE 'slow_query%';

-- 优化查询
EXPLAIN ANALYZE SELECT * FROM events WHERE operator_id = 'op_001';
```

---

## 错误处理

### 常见错误

#### 构建失败

**错误**: `Module not found`

**解决方案**:
```bash
# 清理缓存
rm -rf node_modules dist
npm install
npm run build
```

#### 测试失败

**错误**: `Test suite failed to run`

**解决方案**:
```bash
# 检查测试环境
npm test -- --verbose

# 运行单个测试
npm test -- path/to/test.test.ts

# 清理测试缓存
rm -rf node_modules/.vitest
```

#### 运行时错误

**错误**: `Cannot read property 'x' of undefined`

**解决方案**:
1. 检查错误堆栈
2. 定位问题代码
3. 添加空值检查
4. 使用可选链操作符 `?.`

### 错误日志

```typescript
// 使用结构化日志
import { logger } from './logger'

try {
  await riskyOperation()
} catch (error) {
  logger.error('Operation failed', {
    error: error.message,
    stack: error.stack,
    context: { userId: '123', action: 'create' }
  })
}
```

---

## 备份与恢复

### 数据库备份

```bash
# PostgreSQL
pg_dump acp_ui > backup_$(date +%Y%m%d).sql

# SQLite
sqlite3 acp-ui.db ".backup 'backup_$(date +%Y%m%d).db'"
```

### 配置备份

```bash
# 备份配置文件
tar -czf config_backup_$(date +%Y%m%d).tar.gz \
  config/ \
  .env \
  package.json
```

### 恢复数据

```bash
# 恢复 PostgreSQL
psql acp_ui < backup_20260712.sql

# 恢复 SQLite
sqlite3 acp-ui.db ".restore 'backup_20260712.db'"
```

### 自动化备份

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d)
BACKUP_DIR="/backups"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份数据库
pg_dump acp_ui > $BACKUP_DIR/db_$DATE.sql

# 备份配置
tar -czf $BACKUP_DIR/config_$DATE.tar.gz config/ .env

# 清理旧备份（保留30天）
find $BACKUP_DIR -type f -mtime +30 -delete

# 上传到云存储
aws s3 sync $BACKUP_DIR s3://my-backups/acp-ui/
```

---

## 版本发布

### 发布流程

1. **更新版本号**
   ```bash
   npm version patch  # 0.1.0 -> 0.1.1
   npm version minor  # 0.1.1 -> 0.2.0
   npm version major  # 0.2.0 -> 1.0.0
   ```

2. **更新 CHANGELOG**
   ```markdown
   ## [0.1.1] - 2026-07-12
   
   ### Fixed
   - Bug fix description
   
   ### Added
   - New feature description
   ```

3. **运行测试**
   ```bash
   npm test
   npm run typecheck
   npm run build
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "chore: release v0.1.1"
   git tag v0.1.1
   ```

5. **推送**
   ```bash
   git push origin main
   git push origin v0.1.1
   ```

6. **创建 Release**
   ```bash
   gh release create v0.1.1 \
     --title "v0.1.1" \
     --notes "Release notes..."
   ```

### 发布检查清单

- [ ] 所有测试通过
- [ ] 类型检查通过
- [ ] 构建成功
- [ ] CHANGELOG 更新
- [ ] 版本号更新
- [ ] 文档更新
- [ ] 代码审查完成
- [ ] 安全审计通过

---

## 安全维护

### 依赖安全检查

```bash
# 检查已知漏洞
npm audit

# 自动修复
npm audit fix

# 详细报告
npm audit --json > audit_report.json
```

### 代码安全扫描

```bash
# 使用 ESLint 安全插件
npm install eslint-plugin-security --save-dev

# 运行扫描
npx eslint src --ext .ts,.vue
```

### 定期安全审查

每月进行一次安全审查：
- [ ] 检查依赖漏洞
- [ ] 审查权限设置
- [ ] 检查日志敏感信息
- [ ] 验证认证机制
- [ ] 测试安全防护

---

## 监控告警

### 设置告警

```typescript
// 错误率告警
if (errorRate > 0.05) { // 5%
  sendAlert('High error rate detected')
}

// 响应时间告警
if (avgResponseTime > 1000) { // 1s
  sendAlert('Slow response time')
}

// 内存使用告警
if (memoryUsage > 0.9) { // 90%
  sendAlert('High memory usage')
}
```

### 监控工具

- **应用监控**: Sentry, Bugsnag, Rollbar
- **性能监控**: New Relic, Datadog
- **日志管理**: ELK Stack, Splunk
- **指标监控**: Prometheus + Grafana

---

## 维护计划

### 日常 (每天)

- [ ] 检查错误日志
- [ ] 监控系统指标
- [ ] 处理用户反馈

### 每周

- [ ] 检查依赖更新
- [ ] 清理过期数据
- [ ] 备份数据库
- [ ] 审查安全日志

### 每月

- [ ] 全面安全检查
- [ ] 性能优化
- [ ] 文档更新
- [ ] 发布维护版本

### 每季度

- [ ] 依赖大版本更新
- [ ] 架构审查
- [ ] 技术债务清理
- [ ] 发布主要版本

---

## 更多信息

- [部署指南](DEPLOYMENT-GUIDE.md)
- [最佳实践](BEST-PRACTICES.md)
- [贡献指南](../CONTRIBUTING.md)
- [GitHub 仓库](https://github.com/yanritian/acp-ui)

---

<div align="center">

**保持系统健康运行！**

[查看部署指南 →](DEPLOYMENT-GUIDE.md)

</div>
