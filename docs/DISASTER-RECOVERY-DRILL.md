# 灾难恢复演练指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 演练目标

### 恢复时间目标 (RTO)

| 场景 | RTO | 说明 |
|------|-----|------|
| 应用故障 | 15 分钟 | 单个服务重启 |
| 数据库故障 | 1 小时 | 主从切换 |
| 区域故障 | 4 小时 | 跨区域恢复 |
| 灾难 | 24 小时 | 完全恢复 |

### 恢复点目标 (RPO)

| 数据类型 | RPO | 说明 |
|----------|-----|------|
| 任务数据 | 1 小时 | 最大数据丢失 |
| 事件数据 | 1 小时 | 最大数据丢失 |
| 审计日志 | 15 分钟 | 最大数据丢失 |
| 配置数据 | 24 小时 | 最大数据丢失 |

---

## 演练场景

### 场景 1: 应用服务器故障

**触发条件**:
- 服务无响应
- 健康检查失败
- 错误率 > 5%

**演练步骤**:

```bash
#!/bin/bash
# drill-app-failure.sh

echo "=== 应用服务器故障演练 ==="

# 1. 模拟故障
echo "1. 模拟应用服务器故障..."
kubectl scale deployment hermes-game-operator --replicas=0 -n hermes

# 2. 验证服务不可用
echo "2. 验证服务不可用..."
sleep 10
curl -f https://api.your-domain.com/health && echo "FAIL: Service still up" || echo "OK: Service down"

# 3. 自动恢复
echo "3. 触发自动恢复..."
kubectl scale deployment hermes-game-operator --replicas=3 -n hermes

# 4. 等待恢复
echo "4. 等待服务恢复..."
kubectl rollout status deployment hermes-game-operator -n hermes

# 5. 验证恢复
echo "5. 验证服务恢复..."
curl -f https://api.your-domain.com/health && echo "OK: Service recovered" || echo "FAIL: Service not recovered"

# 6. 验证数据完整性
echo "6. 验证数据完整性..."
psql -h localhost -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

# 7. 记录恢复时间
echo "7. 记录恢复时间..."
echo "Recovery time: $(date)"

echo "=== 演练完成 ==="
```

**预期结果**:
- 服务在 15 分钟内恢复
- 数据完整性验证通过
- 无数据丢失

---

### 场景 2: 数据库故障

**触发条件**:
- 主数据库无响应
- 连接池耗尽
- 磁盘空间不足

**演练步骤**:

```bash
#!/bin/bash
# drill-db-failure.sh

echo "=== 数据库故障演练 ==="

# 1. 模拟主库故障
echo "1. 模拟主数据库故障..."
kubectl exec -it postgres-primary-0 -n hermes -- pg_ctl stop -D /var/lib/postgresql/data

# 2. 验证主从切换
echo "2. 验证自动主从切换..."
sleep 30

# 3. 检查新主库
echo "3. 检查新主库状态..."
kubectl exec -it postgres-replica-0 -n hermes -- psql -c "SELECT pg_is_in_recovery();"

# 4. 更新连接字符串
echo "4. 更新应用连接..."
kubectl set env deployment/hermes-game-operator -n hermes \
  DATABASE_URL=postgres://hermes:password@postgres-replica-0:5432/hermes

# 5. 验证应用恢复
echo "5. 验证应用恢复..."
curl -f https://api.your-domain.com/health

# 6. 重建复制
echo "6. 重建主从复制..."
./scripts/rebuild-replication.sh

# 7. 验证数据完整性
echo "7. 验证数据完整性..."
psql -h postgres-replica-0 -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

echo "=== 演练完成 ==="
```

**预期结果**:
- 主从切换在 1 小时内完成
- 数据丢失 < 1 小时
- 应用自动恢复

---

### 场景 3: 区域故障

**触发条件**:
- 整个区域不可用
- 网络中断
- 自然灾害

**演练步骤**:

```bash
#!/bin/bash
# drill-region-failure.sh

echo "=== 区域故障演练 ==="

# 1. 模拟区域故障
echo "1. 模拟区域故障..."
aws route53 change-resource-record-sets \
  --hosted-zone-id $ZONE_ID \
  --change-batch file://drill-dns-change.json

# 2. 激活 DR 站点
echo "2. 激活灾难恢复站点..."
kubectl config use-context dr-cluster

# 3. 恢复数据库
echo "3. 恢复数据库..."
./scripts/restore-db.sh $(aws s3 ls s3://hermes-backups-dr/database/ | sort | tail -n 1 | awk '{print $4}')

# 4. 恢复应用
echo "4. 恢复应用..."
kubectl apply -f k8s/production/

# 5. 验证服务
echo "5. 验证 DR 站点服务..."
curl -f https://dr.api.your-domain.com/health

# 6. 验证数据
echo "6. 验证数据完整性..."
psql -h dr-db -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

# 7. 恢复原区域
echo "7. 恢复原区域..."
aws route53 change-resource-record-sets \
  --hosted-zone-id $ZONE_ID \
  --change-batch file://restore-dns-change.json

echo "=== 演练完成 ==="
```

**预期结果**:
- DR 站点在 4 小时内激活
- 数据丢失 < 1 小时
- 服务在 DR 站点正常运行

---

### 场景 4: 数据丢失

**触发条件**:
- 误删除数据
- 数据损坏
- 恶意攻击

**演练步骤**:

```bash
#!/bin/bash
# drill-data-loss.sh

echo "=== 数据丢失演练 ==="

# 1. 模拟数据丢失
echo "1. 模拟数据丢失..."
psql -h localhost -U hermes -d hermes -c "DELETE FROM tasks WHERE created_at > '2026-07-10';"

# 2. 确定恢复点
echo "2. 确定恢复点..."
aws s3 ls s3://hermes-backups/database/ | sort | tail -n 5

# 3. 恢复到临时环境
echo "3. 恢复到临时环境..."
./scripts/restore-to-temp.sh backup_20260710_020000.backup.gz

# 4. 验证恢复数据
echo "4. 验证恢复数据..."
psql -h temp-db -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

# 5. 导出丢失数据
echo "5. 导出丢失数据..."
psql -h temp-db -U hermes -d hermes -c "\COPY tasks TO '/tmp/lost_tasks.csv' WITH CSV"

# 6. 导入到生产环境
echo "6. 导入到生产环境..."
psql -h localhost -U hermes -d hermes -c "\COPY tasks FROM '/tmp/lost_tasks.csv' WITH CSV"

# 7. 验证数据完整性
echo "7. 验证数据完整性..."
psql -h localhost -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

# 8. 清理
echo "8. 清理临时环境..."
./scripts/cleanup-temp.sh

echo "=== 演练完成 ==="
```

**预期结果**:
- 数据在 2 小时内恢复
- 数据丢失 < 1 小时
- 数据完整性验证通过

---

## 演练计划

### 频率

| 演练类型 | 频率 | 参与人员 |
|----------|------|---------|
| 桌面演练 | 每月 | 核心团队 |
| 功能演练 | 每季度 | 技术团队 |
| 全面演练 | 每年 | 全体团队 |

### 时间表

```
Q1:
  - 1月: 桌面演练（应用故障）
  - 2月: 桌面演练（数据库故障）
  - 3月: 功能演练（应用故障）

Q2:
  - 4月: 桌面演练（区域故障）
  - 5月: 桌面演练（数据丢失）
  - 6月: 功能演练（数据库故障）

Q3:
  - 7月: 桌面演练（应用故障）
  - 8月: 桌面演练（数据库故障）
  - 9月: 全面演练

Q4:
  - 10月: 桌面演练（区域故障）
  - 11月: 桌面演练（数据丢失）
  - 12月: 功能演练（区域故障）
```

---

## 演练检查清单

### 演练前

- [ ] 演练场景已定义
- [ ] 演练步骤已文档化
- [ ] 参与人员已通知
- [ ] 备份已验证
- [ ] 回滚计划已准备
- [ ] 通信渠道已测试

### 演练中

- [ ] 按步骤执行
- [ ] 记录每个步骤时间
- [ ] 记录问题和偏差
- [ ] 监控指标和日志
- [ ] 验证恢复结果

### 演练后

- [ ] 生成演练报告
- [ ] 分析恢复时间
- [ ] 识别改进点
- [ ] 更新演练文档
- [ ] 安排下次演练

---

## 演练报告模板

```markdown
# 灾难恢复演练报告

## 基本信息
- **演练日期**: 2026-07-11
- **演练场景**: 应用服务器故障
- **参与人员**: 张三、李四、王五
- **演练时长**: 45 分钟

## 演练结果

### 恢复时间
- **目标 RTO**: 15 分钟
- **实际 RTO**: 12 分钟
- **状态**: ✅ 达标

### 数据完整性
- **验证项目**: 任务数据
- **验证结果**: ✅ 通过
- **数据丢失**: 无

### 问题记录
1. 监控告警延迟 2 分钟
   - 影响: 轻微
   - 改进: 优化告警配置

2. 文档中缺少一个步骤
   - 影响: 轻微
   - 改进: 更新文档

## 改进措施
1. 优化监控告警配置
2. 更新演练文档
3. 增加自动化测试

## 下次演练
- **日期**: 2026-08-11
- **场景**: 数据库故障
```

---

## 最佳实践

### 1. 定期演练

- 每月至少一次桌面演练
- 每季度至少一次功能演练
- 每年至少一次全面演练

### 2. 自动化

- 自动化恢复流程
- 自动化验证流程
- 自动化报告生成

### 3. 文档化

- 详细的演练步骤
- 清晰的角色职责
- 完整的问题记录

### 4. 持续改进

- 分析每次演练
- 识别改进点
- 更新流程和文档

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DR Team