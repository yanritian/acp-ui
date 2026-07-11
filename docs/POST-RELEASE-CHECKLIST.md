# 发布后检查清单

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本检查清单确保 Hermes Game Operator 发布后所有关键功能正常运行。

---

## 立即检查（发布后 1 小时内）

### 服务健康

- [ ] **应用服务正常运行**
  ```bash
  curl https://your-domain.com/health
  # 期望: {"status":"ok","version":"0.1.0-alpha"}
  ```

- [ ] **数据库连接正常**
  ```bash
  psql -h localhost -U hermes -d hermes -c "SELECT 1;"
  # 期望: 1 row
  ```

- [ ] **Redis 连接正常**
  ```bash
  redis-cli ping
  # 期望: PONG
  ```

- [ ] **所有服务启动**
  ```bash
  docker-compose ps
  # 期望: 所有服务状态为 Up
  ```

---

### 功能验证

- [ ] **用户登录**
  1. 访问登录页面
  2. 输入测试账户凭据
  3. 成功登录后跳转到仪表板

- [ ] **创建任务**
  1. 点击 "New Task"
  2. 填写任务信息
  3. 任务创建成功

- [ ] **查看任务列表**
  1. 访问任务列表页面
  2. 任务正确显示
  3. 分页功能正常

- [ ] **任务详情**
  1. 点击任务查看详情
  2. 任务详情正确显示
  3. 事件流正常更新

---

### API 测试

- [ ] **健康检查端点**
  ```bash
  curl https://your-domain.com/health
  ```

- [ ] **认证端点**
  ```bash
  curl -X POST https://your-domain.com/auth/token \
    -H "Content-Type: application/json" \
    -d '{"grant_type":"authorization_code","code":"test"}'
  ```

- [ ] **任务 API**
  ```bash
  curl https://your-domain.com/api/tasks \
    -H "Authorization: Bearer $TOKEN"
  ```

---

## 短期检查（发布后 24 小时内）

### 监控指标

- [ ] **CPU 使用率正常**
  - 检查 Prometheus
  - CPU 使用率 < 70%

- [ ] **内存使用率正常**
  - 检查 Prometheus
  - 内存使用率 < 80%

- [ ] **磁盘使用率正常**
  - 检查 Prometheus
  - 磁盘使用率 < 75%

- [ ] **响应时间正常**
  - 检查 Grafana
  - P95 响应时间 < 500ms

- [ ] **错误率正常**
  - 检查 Grafana
  - 错误率 < 1%

---

### 日志检查

- [ ] **无严重错误**
  ```bash
  grep "ERROR" /var/log/hermes-game-operator/app.log | tail -20
  ```

- [ ] **无异常警告**
  ```bash
  grep "WARNING" /var/log/hermes-game-operator/app.log | tail -20
  ```

- [ ] **日志轮转正常**
  ```bash
  ls -lh /var/log/hermes-game-operator/
  ```

---

### 备份验证

- [ ] **数据库备份成功**
  ```bash
  ls -lh /var/backups/hermes/db/
  # 检查最新备份
  ```

- [ ] **备份文件完整**
  ```bash
  # 验证备份文件
  pg_restore -l /var/backups/hermes/db/latest.backup
  ```

- [ ] **云存储备份**
  ```bash
  aws s3 ls s3://hermes-backups/db/ --recursive | tail -5
  ```

---

### 安全扫描

- [ ] **无安全漏洞**
  ```bash
  npm audit
  # 检查 Node.js 依赖
  ```

- [ ] **SSL 证书有效**
  ```bash
  openssl s_client -connect your-domain.com:443
  # 检查证书有效期
  ```

- [ ] **防火墙规则正确**
  ```bash
  sudo ufw status
  # 检查防火墙规则
  ```

---

## 中期检查（发布后 1 周内）

### 性能测试

- [ ] **负载测试通过**
  ```bash
  # 使用 k6 进行负载测试
  k6 run load-test.js
  ```

- [ ] **压力测试通过**
  ```bash
  # 使用 k6 进行压力测试
  k6 run stress-test.js
  ```

- [ ] **性能基准符合预期**
  - 响应时间 < 500ms
  - 吞吐量 > 1000 req/s
  - 错误率 < 1%

---

### 用户反馈

- [ ] **收集用户反馈**
  ```bash
  # 查看用户反馈渠道
  # - GitHub Issues
  # - Discord
  # - 邮件
  ```

- [ ] **处理用户报告的问题**
  - 优先级: P0 > P1 > P2
  - 及时响应用户

- [ ] **更新文档**
  - 根据用户反馈更新文档
  - 添加常见问题解答

---

### 系统优化

- [ ] **数据库优化**
  ```bash
  # 分析慢查询
  psql -h localhost -U hermes -d hermes -c "SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"
  ```

- [ ] **缓存优化**
  ```bash
  # 检查 Redis 命中率
  redis-cli INFO stats | grep keyspace
  ```

- [ ] **资源使用优化**
  - 调整实例大小
  - 优化配置参数

---

## 长期检查（发布后 1 个月内）

### 灾难恢复演练

- [ ] **备份恢复测试**
  ```bash
  # 测试备份恢复
  ./scripts/test-backup-restore.sh
  ```

- [ ] **故障转移测试**
  ```bash
  # 测试故障转移
  ./scripts/test-failover.sh
  ```

- [ ] **灾难恢复演练**
  - 按照 DRP 执行
  - 记录恢复时间
  - 评估恢复效果

---

### 安全审计

- [ ] **安全漏洞扫描**
  ```bash
  # 使用 Trivy 扫描
  trivy image hermes-game-operator:latest
  ```

- [ ] **渗透测试**
  - 聘请专业团队
  - 测试常见漏洞
  - 修复发现的问题

- [ ] **合规性检查**
  - GDPR 合规
  - SOC 2 合规
  - 其他适用标准

---

### 成本分析

- [ ] **云资源成本**
  ```bash
  # AWS 成本分析
  aws ce get-cost-and-usage \
    --time-period Start=2026-07-01,End=2026-07-31 \
    --granularity MONTHLY \
    --metrics "BlendedCost"
  ```

- [ ] **成本优化建议**
  - 使用预留实例
  - 优化资源使用
  - 清理未使用资源

- [ ] **预算规划**
  - 分析历史成本
  - 预测未来成本
  - 设置预算告警

---

## 检查清单总结

### 关键指标

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 可用性 | > 99.9% | _____% | ✅/❌ |
| 响应时间 | < 500ms | _____ms | ✅/❌ |
| 错误率 | < 1% | _____% | ✅/❌ |
| 备份成功率 | 100% | _____% | ✅/❌ |
| 安全漏洞 | 0 | _____ | ✅/❌ |

### 问题跟踪

| 问题 | 优先级 | 状态 | 负责人 |
|------|--------|------|--------|
| _____ | P0/P1/P2 | 开放/处理中/已解决 | _____ |

### 下一步行动

1. _____
2. _____
3. _____

---

## 签发

**检查人**: _________________
**日期**: _________________
**状态**: ✅ 通过 / ❌ 未通过

**审核人**: _________________
**日期**: _________________

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team