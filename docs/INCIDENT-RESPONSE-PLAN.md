# 事件响应计划 (IRP)

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本事件响应计划（Incident Response Plan，IRP）定义了 Hermes Game Operator 在发生安全事件或系统故障时的响应流程。

---

## 事件分类

### 严重性级别

| 级别 | 描述 | 响应时间 | 示例 |
|------|------|---------|------|
| **P0 - 严重** | 服务完全不可用 | 15 分钟 | 数据中心故障、数据库损坏 |
| **P1 - 高** | 主要功能受影响 | 1 小时 | 应用崩溃、API 故障 |
| **P2 - 中** | 部分功能受影响 | 4 小时 | 性能下降、非关键服务故障 |
| **P3 - 低** | 轻微问题 | 24 小时 | UI 问题、文档错误 |

### 事件类型

1. **安全事件**
   - 数据泄露
   - 未授权访问
   - DDoS 攻击
   - 恶意软件

2. **系统事件**
   - 服务宕机
   - 数据库故障
   - 网络中断
   - 硬件故障

3. **应用事件**
   - 应用崩溃
   - API 错误
   - 性能问题
   - 功能缺陷

---

## 响应团队

### 角色和职责

| 角色 | 职责 | 联系方式 |
|------|------|---------|
| **事件经理** | 协调整个响应过程 | event-manager@example.com |
| **技术负责人** | 提供技术指导 | tech-lead@example.com |
| **安全专家** | 处理安全事件 | security@example.com |
| **运维工程师** | 执行恢复操作 | ops@example.com |
| **开发负责人** | 提供开发支持 | dev-lead@example.com |
| **通信协调员** | 管理内外部沟通 | comms@example.com |

### 升级流程

```
P3 (低) → 运维工程师
P2 (中) → 运维工程师 + 技术负责人
P1 (高) → 事件经理 + 技术负责人 + 运维工程师
P0 (严重) → 事件经理 + 技术负责人 + 安全专家 + 运维工程师 + 管理层
```

---

## 响应流程

### 阶段 1: 检测和报告

#### 检测

**监控告警**:
```bash
# Prometheus 告警
curl http://prometheus:9090/api/v1/alerts
```

**日志分析**:
```bash
# 查看错误日志
tail -f /var/log/hermes-game-operator/app.log | grep "ERROR"
```

**用户报告**:
- GitHub Issues
- Discord
- 邮件
- 电话

#### 报告

**创建事件记录**:
```bash
# 使用事件管理工具
curl -X POST https://pagerduty.com/api/v2/incidents \
  -H "Authorization: Token token=$PAGERDUTY_TOKEN" \
  -d '{
    "incident": {
      "type": "incident",
      "title": "Service Outage",
      "urgency": "high"
    }
  }'
```

**通知团队**:
```bash
# Slack 通知
curl -X POST https://slack.com/api/chat.postMessage \
  -H "Authorization: Bearer $SLACK_TOKEN" \
  -d '{"channel":"#incidents","text":"🚨 P1 Incident: Service degraded"}'
```

---

### 阶段 2: 评估和分类

#### 评估影响

**检查清单**:
- [ ] 受影响的服务
- [ ] 受影响的用户数量
- [ ] 数据丢失风险
- [ ] 安全影响
- [ ] 业务影响

**确定严重性**:
```
P0: 服务完全不可用，大量用户受影响
P1: 主要功能受影响，部分用户受影响
P2: 部分功能受影响，少量用户受影响
P3: 轻微问题，影响最小
```

#### 分类事件

**事件类型**:
- 安全事件
- 系统事件
- 应用事件

**事件优先级**:
- 立即处理
- 高优先级
- 中优先级
- 低优先级

---

### 阶段 3: 遏制和缓解

#### 立即行动

**P0 - 严重事件**:

1. **启用备用系统**
   ```bash
   # 切换到 DR 站点
   aws route53 change-resource-record-sets \
     --hosted-zone-id $ZONE_ID \
     --change-batch file://dr-dns-change.json
   ```

2. **隔离受影响系统**
   ```bash
   # 停止受影响的服务
   sudo systemctl stop hermes-game-operator
   ```

3. **启动应急团队**
   ```bash
   # 召开紧急会议
   # 所有相关人员立即上线
   ```

**P1 - 高优先级事件**:

1. **重启服务**
   ```bash
   sudo systemctl restart hermes-game-operator
   ```

2. **扩展资源**
   ```bash
   # 增加实例
   aws autoscaling set-desired-capacity \
     --auto-scaling-group-name hermes-asg \
     --desired-capacity 5
   ```

3. **通知用户**
   ```bash
   # 更新状态页面
   curl -X POST https://statuspage.io/api/v1/incidents \
     -H "Authorization: Token $TOKEN" \
     -d '{"incident":{"name":"Service Degraded"}}'
   ```

---

### 阶段 4: 调查和修复

#### 根本原因分析

**收集证据**:
```bash
# 查看系统日志
journalctl -u hermes-game-operator --since "1 hour ago"

# 查看应用日志
cat /var/log/hermes-game-operator/app.log

# 查看数据库日志
cat /var/log/postgresql/postgresql-15-main.log
```

**分析日志**:
```bash
# 查找错误模式
grep "ERROR" /var/log/hermes-game-operator/app.log | awk '{print $5}' | sort | uniq -c
```

**创建时间线**:
```
14:30 - 服务开始响应缓慢
14:35 - 用户开始报告问题
14:40 - 监控告警触发
14:45 - 事件团队启动
14:50 - 识别问题：数据库连接池耗尽
14:55 - 修复：增加连接池大小
15:00 - 服务恢复正常
```

#### 实施修复

**热修复**:
```bash
# 应用修复
git cherry-pick $COMMIT_HASH

# 重新部署
ansible-playbook -i production deploy.yml
```

**配置更改**:
```bash
# 更新配置
nano /opt/hermes-game-operator/config.json

# 重启服务
sudo systemctl restart hermes-game-operator
```

---

### 阶段 5: 恢复和验证

#### 恢复服务

**验证修复**:
```bash
# 健康检查
curl https://your-domain.com/health

# 功能测试
curl https://your-domain.com/api/tasks \
  -H "Authorization: Bearer $TOKEN"
```

**监控指标**:
```bash
# 检查 Prometheus
curl http://prometheus:9090/api/v1/query?query=up

# 检查 Grafana
# 查看仪表板
```

**用户确认**:
- [ ] 用户报告问题已解决
- [ ] 监控指标正常
- [ ] 日志无错误

---

### 阶段 6: 事后分析

#### 事后审查会议

**议程**:
1. 事件时间线回顾
2. 响应过程评估
3. 根本原因分析
4. 改进建议
5. 行动计划

**参与者**:
- 事件经理
- 技术负责人
- 相关工程师
- 管理层（P0/P1 事件）

#### 事后报告

**报告模板**:

```markdown
# 事件报告

## 事件摘要
- **事件 ID**: INC-2026-001
- **严重性**: P1
- **持续时间**: 30 分钟
- **影响用户**: 1000+

## 时间线
- 14:30 - 服务开始响应缓慢
- 14:35 - 用户开始报告问题
- 14:40 - 监控告警触发
- 14:45 - 事件团队启动
- 14:50 - 识别问题
- 14:55 - 应用修复
- 15:00 - 服务恢复

## 根本原因
数据库连接池配置不足，导致高并发时连接耗尽。

## 影响
- 服务响应时间增加 10 倍
- 部分 API 请求失败
- 用户无法创建新任务

## 改进措施
1. 增加数据库连接池大小
2. 实施连接池监控
3. 添加自动扩展机制
4. 优化慢查询

## 经验教训
- 监控告警及时触发
- 团队响应迅速
- 沟通渠道畅通
- 需要更好的负载测试
```

---

## 通信计划

### 内部通信

| 事件级别 | 通知渠道 | 频率 |
|----------|---------|------|
| P0 | Slack + 电话 + 邮件 | 每 15 分钟 |
| P1 | Slack + 邮件 | 每 30 分钟 |
| P2 | Slack | 每 1 小时 |
| P3 | Slack | 每日 |

### 外部通信

| 对象 | 渠道 | 内容 |
|------|------|------|
| 用户 | 状态页面 | 服务状态更新 |
| 客户 | 邮件 | 详细影响说明 |
| 管理层 | 邮件 | 事件摘要 |
| 媒体 | 新闻稿 | 公开声明（如需要） |

### 状态页面更新

```bash
# 更新状态
curl -X POST https://statuspage.io/api/v1/incidents \
  -H "Authorization: Token $TOKEN" \
  -d '{
    "incident": {
      "name": "Service Degradation",
      "status": "investigating",
      "body": "We are investigating reports of service degradation..."
    }
  }'
```

---

## 工具和资源

### 监控工具

- **Prometheus**: 指标收集
- **Grafana**: 可视化
- **Alertmanager**: 告警管理

### 日志工具

- **ELK Stack**: 日志收集和分析
- **Graylog**: 日志管理

### 事件管理

- **PagerDuty**: 告警和事件管理
- **OpsGenie**: 事件响应

### 通信工具

- **Slack**: 团队沟通
- **Zoom**: 视频会议
- **Email**: 正式通信

---

## 培训和演练

### 培训计划

| 培训类型 | 频率 | 对象 |
|----------|------|------|
| 事件响应培训 | 每季度 | 全体团队 |
| 安全培训 | 每半年 | 全体团队 |
| 工具培训 | 按需 | 新成员 |

### 演练计划

| 演练类型 | 频率 | 说明 |
|----------|------|------|
| 桌面演练 | 每季度 | 讨论场景 |
| 功能演练 | 每半年 | 测试特定功能 |
| 全面演练 | 每年 | 完整响应流程 |

---

## 文档维护

### 更新频率

- **每月**: 审查联系人信息
- **每季度**: 更新流程和工具
- **每年**: 完整 IRP 审查

### 审查清单

- [ ] 联系人信息是否最新？
- [ ] 流程是否清晰？
- [ ] 工具是否可用？
- [ ] 团队是否培训？
- [ ] 演练是否执行？

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
| 监控 | https://grafana.your-domain.com | 监控系统 |
| 日志 | https://kibana.your-domain.com | 日志系统 |
| 状态页面 | https://status.your-domain.com | 状态页面 |

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Security Team
**版本**: 0.1.0-alpha