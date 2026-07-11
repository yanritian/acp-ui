# 灾难恢复计划 (DRP)

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本灾难恢复计划（Disaster Recovery Plan，DRP）定义了 Hermes Game Operator 在发生重大故障时的恢复策略和程序。

---

## 恢复目标

### 恢复时间目标 (RTO)

| 场景 | RTO | 说明 |
|------|-----|------|
| 应用服务器故障 | 1 小时 | 恢复应用服务 |
| 数据库故障 | 2 小时 | 恢复数据库服务 |
| 数据中心故障 | 4 小时 | 切换到备用数据中心 |
| 网络故障 | 30 分钟 | 恢复网络连接 |

### 恢复点目标 (RPO)

| 数据类型 | RPO | 说明 |
|----------|-----|------|
| 任务数据 | 1 小时 | 最多丢失 1 小时数据 |
| 事件数据 | 1 小时 | 最多丢失 1 小时数据 |
| 审计日志 | 15 分钟 | 最多丢失 15 分钟数据 |
| 配置数据 | 24 小时 | 最多丢失 24 小时数据 |

---

## 风险评估

### 高风险场景

1. **数据中心故障**
   - 可能性: 低
   - 影响: 高
   - 恢复策略: 多区域部署

2. **数据库损坏**
   - 可能性: 中
   - 影响: 高
   - 恢复策略: 主从复制 + 备份

3. **网络中断**
   - 可能性: 中
   - 影响: 中
   - 恢复策略: 多线路 + VPN

### 中风险场景

1. **应用服务器故障**
   - 可能性: 中
   - 影响: 中
   - 恢复策略: 负载均衡 + 自动扩展

2. **缓存服务故障**
   - 可能性: 中
   - 影响: 低
   - 恢复策略: Redis Sentinel

3. **证书过期**
   - 可能性: 低
   - 影响: 中
   - 恢复策略: 自动续期

---

## 备份策略

### 数据库备份

#### 完整备份

```bash
# 每天凌晨 2:00 执行完整备份
0 2 * * * /usr/local/bin/hermes-db-backup.sh full

# 备份脚本
#!/bin/bash
BACKUP_DIR="/var/backups/hermes/db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 完整备份
pg_dump -h localhost -U hermes -F c -f \
  "$BACKUP_DIR/full_$TIMESTAMP.backup" hermes

# 压缩
gzip "$BACKUP_DIR/full_$TIMESTAMP.backup"

# 上传到云存储
aws s3 cp "$BACKUP_DIR/full_$TIMESTAMP.backup.gz" \
  s3://hermes-backups/db/

# 清理本地备份（保留 7 天）
find "$BACKUP_DIR" -name "full_*.backup.gz" -mtime +7 -delete
```

#### 增量备份

```bash
# 每 6 小时执行增量备份
0 */6 * * * /usr/local/bin/hermes-db-backup.sh incremental

# 备份脚本
#!/bin/bash
BACKUP_DIR="/var/backups/hermes/db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 增量备份（基于 WAL）
pg_basebackup -h localhost -U hermes -D \
  "$BACKUP_DIR/incr_$TIMESTAMP" --wal-method=stream

# 压缩
tar -czf "$BACKUP_DIR/incr_$TIMESTAMP.tar.gz" \
  "$BACKUP_DIR/incr_$TIMESTAMP"

# 上传到云存储
aws s3 cp "$BACKUP_DIR/incr_$TIMESTAMP.tar.gz" \
  s3://hermes-backups/db/

# 清理本地备份（保留 3 天）
find "$BACKUP_DIR" -name "incr_*.tar.gz" -mtime +3 -delete
```

### 配置文件备份

```bash
# 每次配置更改后自动备份
#!/bin/bash
BACKUP_DIR="/var/backups/hermes/config"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 备份配置
tar -czf "$BACKUP_DIR/config_$TIMESTAMP.tar.gz" \
  /opt/hermes-game-operator/config \
  /opt/hermes-game-operator/.env

# 上传到云存储
aws s3 cp "$BACKUP_DIR/config_$TIMESTAMP.tar.gz" \
  s3://hermes-backups/config/
```

### 日志备份

```bash
# 每天凌晨 3:00 备份日志
0 3 * * * /usr/local/bin/hermes-log-backup.sh

# 备份脚本
#!/bin/bash
BACKUP_DIR="/var/backups/hermes/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 备份日志
tar -czf "$BACKUP_DIR/logs_$TIMESTAMP.tar.gz" \
  /var/log/hermes-game-operator/

# 上传到云存储
aws s3 cp "$BACKUP_DIR/logs_$TIMESTAMP.tar.gz" \
  s3://hermes-backups/logs/

# 清理本地备份（保留 30 天）
find "$BACKUP_DIR" -name "logs_*.tar.gz" -mtime +30 -delete
```

---

## 恢复程序

### 场景 1: 应用服务器故障

**检测**:
```bash
# 健康检查失败
curl -f https://your-domain.com/health || echo "Service down"
```

**恢复步骤**:

1. **通知团队**
   ```bash
   # 发送告警
   curl -X POST https://slack.com/api/chat.postMessage \
     -H "Authorization: Bearer $SLACK_TOKEN" \
     -d '{"channel":"#alerts","text":"Application server down!"}'
   ```

2. **启动备用实例**
   ```bash
   # 使用 Terraform 启动新实例
   cd /opt/terraform
   terraform apply -auto-approve -var="instance_count=1"
   ```

3. **恢复服务**
   ```bash
   # 部署应用
   ansible-playbook -i inventory deploy.yml
   
   # 验证服务
   curl https://your-domain.com/health
   ```

4. **更新负载均衡**
   ```bash
   # 添加新实例到负载均衡
   aws elbv2 register-targets \
     --target-group-arn $TARGET_GROUP_ARN \
     --targets Id=$NEW_INSTANCE_ID
   ```

**预计恢复时间**: 1 小时

---

### 场景 2: 数据库故障

**检测**:
```bash
# 数据库连接失败
psql -h localhost -U hermes -d hermes -c "SELECT 1;" || echo "DB down"
```

**恢复步骤**:

1. **切换到从库**
   ```bash
   # 提升从库为主库
   sudo -u postgres pg_ctlcluster 15 main promote
   ```

2. **更新连接字符串**
   ```bash
   # 更新配置
   sed -i 's/primary-db/replica-db/g' /opt/hermes-game-operator/config/database.json
   
   # 重启服务
   sudo systemctl restart hermes-game-operator
   ```

3. **验证数据一致性**
   ```bash
   # 检查数据
   psql -h localhost -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"
   ```

4. **重建主从复制**
   ```bash
   # 在新主库上创建备份
   pg_basebackup -h new-primary -U replication -D /var/lib/postgresql/15/main
   
   # 配置复制
   sudo systemctl restart postgresql
   ```

**预计恢复时间**: 2 小时

---

### 场景 3: 数据中心故障

**检测**:
```bash
# 多个服务不可用
curl -f https://your-domain.com/health || echo "Datacenter down"
```

**恢复步骤**:

1. **激活灾难恢复站点**
   ```bash
   # 切换到 DR 站点
   aws route53 change-resource-record-sets \
     --hosted-zone-id $ZONE_ID \
     --change-batch file://dr-dns-change.json
   ```

2. **恢复数据库**
   ```bash
   # 从备份恢复
   aws s3 cp s3://hermes-backups/db/latest.backup.gz /tmp/
   gunzip /tmp/latest.backup.gz
   pg_restore -h dr-db -U hermes -d hermes /tmp/latest.backup
   ```

3. **恢复应用**
   ```bash
   # 部署到 DR 站点
   ansible-playbook -i dr-inventory deploy.yml
   ```

4. **验证服务**
   ```bash
   # 健康检查
   curl https://dr.your-domain.com/health
   ```

5. **通知用户**
   ```bash
   # 发送状态更新
   curl -X POST https://statuspage.io/api/v1/incidents \
     -H "Authorization: Token $STATUSPAGE_TOKEN" \
     -d '{"incident":{"name":"DR Site Active"}}'
   ```

**预计恢复时间**: 4 小时

---

## 测试计划

### 定期测试

| 测试类型 | 频率 | 负责人 | 说明 |
|----------|------|--------|------|
| 备份恢复测试 | 每月 | 运维团队 | 验证备份可恢复 |
| 故障转移测试 | 每季度 | 运维团队 | 测试故障转移 |
| 完整 DR 测试 | 每年 | 全体团队 | 完整灾难恢复演练 |

### 备份恢复测试

```bash
# 每月执行
#!/bin/bash

# 1. 下载最新备份
aws s3 cp s3://hermes-backups/db/latest.backup.gz /tmp/

# 2. 解压
gunzip /tmp/latest.backup.gz

# 3. 恢复到测试数据库
pg_restore -h test-db -U hermes -d test_hermes /tmp/latest.backup

# 4. 验证数据
psql -h test-db -U hermes -d test_hermes -c "SELECT COUNT(*) FROM tasks;"

# 5. 清理
rm /tmp/latest.backup
```

### 故障转移测试

```bash
# 每季度执行
#!/bin/bash

# 1. 停止主服务
sudo systemctl stop hermes-game-operator

# 2. 切换到从库
sudo -u postgres pg_ctlcluster 15 main promote

# 3. 验证服务
curl https://your-domain.com/health

# 4. 恢复主服务
sudo systemctl start hermes-game-operator
```

---

## 通信计划

### 内部通信

| 角色 | 联系方式 | 职责 |
|------|---------|------|
| 事件经理 | event-manager@example.com | 协调恢复工作 |
| 技术负责人 | tech-lead@example.com | 技术指导 |
| 运维团队 | ops@example.com | 执行恢复 |
| 开发团队 | dev@example.com | 技术支持 |

### 外部通信

| 对象 | 联系方式 | 内容 |
|------|---------|------|
| 用户 | status@your-domain.com | 服务状态更新 |
| 客户成功 | csm@example.com | 客户沟通 |
| 管理层 | management@example.com | 管理层汇报 |

### 状态页面

```bash
# 更新状态页面
curl -X POST https://statuspage.io/api/v1/incidents \
  -H "Authorization: Token $STATUSPAGE_TOKEN" \
  -d '{
    "incident": {
      "name": "Service Disruption",
      "status": "investigating",
      "body": "We are investigating a service disruption..."
    }
  }'
```

---

## 文档维护

### 更新频率

- **每月**: 审查备份策略
- **每季度**: 更新恢复程序
- **每年**: 完整 DRP 审查

### 审查清单

- [ ] 备份策略是否有效？
- [ ] 恢复时间目标是否合理？
- [ ] 联系人信息是否最新？
- [ ] 恢复程序是否测试过？
- [ ] 文档是否清晰易懂？

---

## 附录

### 联系人列表

| 姓名 | 角色 | 电话 | 邮箱 |
|------|------|------|------|
| John Doe | 事件经理 | +1-234-567-8900 | john@example.com |
| Jane Smith | 技术负责人 | +1-234-567-8901 | jane@example.com |

### 关键系统

| 系统 | URL | 说明 |
|------|-----|------|
| 生产环境 | https://your-domain.com | 生产服务 |
| DR 站点 | https://dr.your-domain.com | 灾难恢复站点 |
| 监控 | https://grafana.your-domain.com | 监控系统 |
| 日志 | https://kibana.your-domain.com | 日志系统 |

### 恢复脚本

- `/usr/local/bin/hermes-db-backup.sh` - 数据库备份
- `/usr/local/bin/hermes-log-backup.sh` - 日志备份
- `/usr/local/bin/hermes-restore.sh` - 完整恢复

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team
**版本**: 0.1.0-alpha