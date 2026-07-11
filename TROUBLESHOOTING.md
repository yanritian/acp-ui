# 故障排除指南

## 目录

- [常见问题](#常见问题)
- [错误代码](#错误代码)
- [调试技巧](#调试技巧)
- [性能问题](#性能问题)
- [联系支持](#联系支持)

---

## 常见问题

### 无法连接到服务器

**问题**: 客户端无法连接到 Game Operator 服务器

**可能原因**:
1. 服务器未运行
2. 网络连接问题
3. 防火墙阻止连接
4. 服务器地址配置错误

**解决方案**:

```bash
# 1. 检查服务器状态
curl http://localhost:8080/health

# 2. 检查服务器日志
docker-compose logs backend

# 3. 检查防火墙设置
sudo ufw status

# 4. 验证服务器地址
ping localhost
```

---

### 认证失败

**问题**: 无法登录或认证令牌无效

**可能原因**:
1. 令牌过期
2. 令牌格式错误
3. 服务器配置问题
4. 时钟不同步

**解决方案**:

```bash
# 1. 检查令牌有效期
echo $AUTH_TOKEN

# 2. 重新获取令牌
curl -X POST http://localhost:8080/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "user", "password": "pass"}'

# 3. 同步系统时钟
sudo ntpdate pool.ntp.org

# 4. 检查服务器日志
docker-compose logs backend | grep -i auth
```

---

### 任务创建失败

**问题**: 无法创建新任务

**可能原因**:
1. 项目路径无效
2. 权限不足
3. 项目类型不支持
4. 服务器错误

**解决方案**:

```bash
# 1. 验证项目路径
ls -la /path/to/project

# 2. 检查权限
whoami
chmod -R 755 /path/to/project

# 3. 验证项目类型
cat /path/to/project/project.godot

# 4. 查看错误日志
docker-compose logs backend | grep -i task
```

---

### 审批超时

**问题**: 审批请求超时或长时间未处理

**可能原因**:
1. 审批队列过长
2. 审批者未登录
3. 网络延迟
4. 系统负载过高

**解决方案**:

```bash
# 1. 检查审批队列
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/approvals

# 2. 检查系统负载
top
htop

# 3. 重启服务
docker-compose restart backend

# 4. 检查数据库连接
psql -h localhost -U hermes -d hermes -c "SELECT COUNT(*) FROM approvals;"
```

---

### 性能下降

**问题**: 系统响应缓慢

**可能原因**:
1. 数据库查询缓慢
2. 内存不足
3. CPU 使用率高
4. 网络延迟

**解决方案**:

```bash
# 1. 检查数据库性能
psql -h localhost -U hermes -d hermes -c "EXPLAIN ANALYZE SELECT * FROM tasks;"

# 2. 检查内存使用
free -h

# 3. 检查 CPU 使用
top

# 4. 检查网络延迟
ping localhost

# 5. 优化数据库
psql -h localhost -U hermes -d hermes -c "VACUUM ANALYZE;"
```

---

## 错误代码

### 400 Bad Request

**描述**: 请求格式错误

**常见原因**:
- 缺少必填字段
- 字段格式不正确
- 数据类型错误

**解决方案**: 检查请求体格式，确保所有必填字段都存在且格式正确。

---

### 401 Unauthorized

**描述**: 认证失败

**常见原因**:
- 令牌缺失
- 令牌过期
- 令牌无效

**解决方案**: 重新获取认证令牌，确保令牌在有效期内。

---

### 403 Forbidden

**描述**: 权限不足

**常见原因**:
- 用户没有访问权限
- 项目 scope 不匹配
- 角色权限不足

**解决方案**: 检查用户角色和权限，确保有足够的访问权限。

---

### 404 Not Found

**描述**: 资源不存在

**常见原因**:
- 任务 ID 不存在
- 审批 ID 不存在
- API 路径错误

**解决方案**: 验证资源 ID 和 API 路径是否正确。

---

### 409 Conflict

**描述**: 状态冲突

**常见原因**:
- 任务状态不允许操作
- 审批已处理
- 资源已被修改

**解决方案**: 检查资源当前状态，确保操作在当前状态下是允许的。

---

### 429 Too Many Requests

**描述**: 请求过于频繁

**常见原因**:
- 超过速率限制
- 短时间内发送大量请求

**解决方案**: 减少请求频率，实施请求节流。

---

### 500 Internal Server Error

**描述**: 服务器内部错误

**常见原因**:
- 数据库连接失败
- 代码异常
- 配置错误

**解决方案**: 查看服务器日志，联系技术支持。

---

## 调试技巧

### 启用调试日志

```bash
# 前端
export NODE_ENV=development
npm run dev

# 后端
export RUST_LOG=debug
cargo run
```

---

### 查看实时日志

```bash
# Docker 日志
docker-compose logs -f

# 应用日志
tail -f logs/app.log

# 系统日志
journalctl -u hermes-game-operator -f
```

---

### 网络抓包

```bash
# 使用 tcpdump
sudo tcpdump -i any port 8080

# 使用 Wireshark
wireshark
```

---

### 数据库调试

```bash
# 连接数据库
psql -h localhost -U hermes -d hermes

# 查看表结构
\dt

# 查看数据
SELECT * FROM tasks LIMIT 10;

# 查看索引
\d tasks
```

---

## 性能问题

### 内存泄漏

**症状**: 内存使用持续增长

**解决方案**:

```bash
# 1. 检查内存使用
free -h

# 2. 重启服务
docker-compose restart

# 3. 分析内存使用
valgrind --leak-check=full ./target/debug/hermes-operator
```

---

### CPU 使用率高

**症状**: CPU 使用率持续 100%

**解决方案**:

```bash
# 1. 检查 CPU 使用
top

# 2. 识别问题进程
ps aux | grep hermes

# 3. 分析 CPU 使用
perf top
```

---

### 磁盘空间不足

**症状**: 磁盘空间不足

**解决方案**:

```bash
# 1. 检查磁盘使用
df -h

# 2. 清理日志
find logs -name "*.log" -mtime +7 -delete

# 3. 清理备份
find backups -name "*.tar.gz" -mtime +7 -delete

# 4. 清理 Docker
docker system prune -a
```

---

## 联系支持

### 提供以下信息

在联系技术支持时，请提供以下信息：

1. **错误描述**: 详细描述遇到的问题
2. **复现步骤**: 如何复现问题
3. **环境信息**:
   - 操作系统和版本
   - Node.js 版本
   - Rust 版本
   - 浏览器和版本
4. **日志文件**: 相关日志文件
5. **截图**: 错误截图
6. **配置信息**: 相关配置（脱敏）

### 联系方式

- **Email**: support@example.com
- **GitHub Issues**: https://github.com/your-org/hermes-game-operator/issues
- **Discord**: https://discord.gg/your-invite

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Team