# 系统管理员指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南为系统管理员提供 Hermes Game Operator 的安装、配置、运维和故障排除指南。

---

## 系统要求

### 最低要求

- **操作系统**: Linux (推荐 Ubuntu 22.04 LTS), Windows Server 2019+, macOS 12+
- **CPU**: 4 核心
- **内存**: 8 GB RAM
- **磁盘空间**: 50 GB SSD
- **网络**: 1 Gbps

### 推荐配置

- **CPU**: 8+ 核心
- **内存**: 16+ GB RAM
- **磁盘空间**: 100+ GB SSD
- **网络**: 10 Gbps

### 软件依赖

- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Node.js**: 18.x+
- **PostgreSQL**: 15+
- **Redis**: 7+

---

## 安装

### Docker 安装（推荐）

#### 步骤 1: 安装 Docker

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io docker-compose-plugin

# 启动 Docker
sudo systemctl start docker
sudo systemctl enable docker
```

#### 步骤 2: 克隆仓库

```bash
git clone https://gitee.com/yan_fan_tian/acp-ui.git
cd acp-ui
```

#### 步骤 3: 配置环境变量

```bash
cp .env.example .env
nano .env
```

编辑 `.env` 文件：

```bash
# 服务器配置
SERVER_URL=https://your-domain.com
PORT=8080

# 数据库配置
POSTGRES_USER=hermes
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=hermes

# Redis 配置
REDIS_PASSWORD=your-secure-redis-password

# 认证配置
JWT_SECRET=your-jwt-secret-key
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret

# 日志配置
LOG_LEVEL=info
```

#### 步骤 4: 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

#### 步骤 5: 验证安装

```bash
# 检查服务状态
docker-compose ps

# 健康检查
curl https://your-domain.com/health
```

---

### 裸机安装

#### 步骤 1: 安装依赖

```bash
# 安装 Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 安装 PostgreSQL
sudo apt-get install postgresql-15

# 安装 Redis
sudo apt-get install redis-server

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 步骤 2: 配置数据库

```bash
# 创建数据库
sudo -u postgres psql

CREATE USER hermes WITH PASSWORD 'your-secure-password';
CREATE DATABASE hermes OWNER hermes;
\q
```

#### 步骤 3: 克隆和构建

```bash
git clone https://gitee.com/yan_fan_tian/acp-ui.git
cd acp-ui

# 安装依赖
npm install

# 构建
npm run build
```

#### 步骤 4: 配置 systemd 服务

创建 `/etc/systemd/system/hermes-game-operator.service`:

```ini
[Unit]
Description=Hermes Game Operator
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=hermes
WorkingDirectory=/opt/hermes-game-operator
ExecStart=/usr/bin/node dist/server.js
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl start hermes-game-operator
sudo systemctl enable hermes-game-operator
```

---

## 配置

### 主配置文件

编辑 `config.json`:

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "cors": {
      "origin": ["https://your-domain.com"],
      "methods": ["GET", "POST", "PUT", "DELETE"]
    }
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "user": "hermes",
    "password": "your-secure-password",
    "database": "hermes",
    "pool": {
      "min": 5,
      "max": 20
    }
  },
  "redis": {
    "host": "localhost",
    "port": 6379,
    "password": "your-secure-redis-password"
  },
  "auth": {
    "jwt": {
      "secret": "your-jwt-secret-key",
      "expiresIn": "1h"
    },
    "oidc": {
      "issuer": "https://auth.your-domain.com",
      "clientId": "your-client-id",
      "clientSecret": "your-client-secret"
    }
  },
  "logging": {
    "level": "info",
    "file": "/var/log/hermes-game-operator/app.log"
  }
}
```

### 安全配置

#### SSL/TLS

```bash
# 生成 SSL 证书
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/hermes.key \
  -out /etc/ssl/certs/hermes.crt
```

更新配置：

```json
{
  "server": {
    "ssl": {
      "key": "/etc/ssl/private/hermes.key",
      "cert": "/etc/ssl/certs/hermes.crt"
    }
  }
}
```

#### 防火墙配置

```bash
# UFW (Ubuntu)
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# firewalld (CentOS/RHEL)
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

---

## 运维

### 监控

#### Prometheus 配置

创建 `/etc/prometheus/hermes-game-operator.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'hermes-game-operator'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

#### Grafana 仪表板

导入预配置的仪表板：

1. 打开 Grafana
2. 导入仪表板 ID: 12345
3. 配置数据源为 Prometheus

### 日志管理

#### 日志轮转

创建 `/etc/logrotate.d/hermes-game-operator`:

```
/var/log/hermes-game-operator/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0640 hermes hermes
    sharedscripts
    postrotate
        systemctl reload hermes-game-operator
    endscript
}
```

#### 日志分析

```bash
# 查看错误日志
grep "ERROR" /var/log/hermes-game-operator/app.log

# 统计请求
awk '{print $1}' /var/log/hermes-game-operator/access.log | sort | uniq -c

# 查看慢请求
awk '$11 > 1000' /var/log/hermes-game-operator/access.log
```

### 备份

#### 自动备份

创建备份脚本 `/usr/local/bin/hermes-backup.sh`:

```bash
#!/bin/bash

BACKUP_DIR="/var/backups/hermes-game-operator"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 备份数据库
pg_dump -h localhost -U hermes hermes | gzip > \
  "$BACKUP_DIR/db_$TIMESTAMP.sql.gz"

# 备份配置
tar -czf "$BACKUP_DIR/config_$TIMESTAMP.tar.gz" \
  /opt/hermes-game-operator/config

# 清理旧备份（保留 7 天）
find "$BACKUP_DIR" -type f -mtime +7 -delete
```

设置 cron 任务：

```bash
# 每天凌晨 2 点备份
0 2 * * * /usr/local/bin/hermes-backup.sh
```

### 更新

#### Docker 更新

```bash
# 拉取最新代码
git pull origin main

# 重新构建
docker-compose build

# 滚动更新
docker-compose up -d --no-deps --build
```

#### 裸机更新

```bash
# 停止服务
sudo systemctl stop hermes-game-operator

# 拉取最新代码
git pull origin main

# 安装依赖
npm install

# 构建
npm run build

# 启动服务
sudo systemctl start hermes-game-operator
```

---

## 故障排除

### 常见问题

#### 服务无法启动

```bash
# 查看日志
journalctl -u hermes-game-operator -n 50

# 检查端口占用
sudo netstat -tulpn | grep :8080

# 检查权限
sudo -u hermes ls -la /opt/hermes-game-operator
```

#### 数据库连接失败

```bash
# 检查 PostgreSQL 状态
sudo systemctl status postgresql

# 测试连接
psql -h localhost -U hermes -d hermes -c "SELECT 1;"

# 检查 pg_hba.conf
sudo nano /etc/postgresql/15/main/pg_hba.conf
```

#### Redis 连接失败

```bash
# 检查 Redis 状态
sudo systemctl status redis

# 测试连接
redis-cli ping

# 检查配置
sudo nano /etc/redis/redis.conf
```

### 性能问题

#### 高 CPU 使用

```bash
# 查看进程
top -p $(pgrep -d ',' -f hermes-game-operator)

# 分析 CPU
perf top -p $(pgrep -f hermes-game-operator)
```

#### 高内存使用

```bash
# 查看内存
free -h

# 分析内存
valgrind --tool=massif node dist/server.js
```

#### 慢查询

```bash
# 启用慢查询日志
psql -h localhost -U hermes -d hermes -c "ALTER SYSTEM SET log_min_duration_statement = 1000;"

# 查看慢查询
cat /var/log/postgresql/postgresql-15-main-slow.log
```

---

## 安全最佳实践

### 认证和授权

- 使用强密码（12+ 字符）
- 启用多因素认证（MFA）
- 定期轮换密钥和令牌
- 实施最小权限原则

### 网络安全

- 使用 HTTPS（TLS 1.3）
- 配置防火墙规则
- 实施入侵检测系统
- 定期进行安全扫描

### 数据安全

- 加密敏感数据
- 实施数据备份策略
- 定期审计访问日志
- 遵循合规要求

---

## 支持

### 文档

- [用户指南](USER-GUIDE.md)
- [开发者指南](DEVELOPER.md)
- [故障排除](TROUBLESHOOTING.md)
- [安全策略](SECURITY.md)

### 社区

- **GitHub Issues**: 报告 bug
- **Discord**: 加入社区
- **邮件列表**: 订阅更新

### 专业支持

- **邮箱**: support@example.com
- **电话**: +1-234-567-8900
- **Slack**: 加入支持频道

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team