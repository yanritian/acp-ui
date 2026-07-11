#!/bin/bash

# Hermes Game Operator 备份脚本
# 用法: ./scripts/backup.sh [目标目录]

set -e

BACKUP_DIR=${1:-"./backups"}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
PROJECT_NAME="hermes-game-operator"

echo "💾 开始备份 Hermes Game Operator"
echo "备份目录: $BACKUP_DIR"
echo "时间戳: $TIMESTAMP"

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 备份数据库
backup_database() {
    echo "🗄️  备份数据库..."

    if command -v pg_dump &> /dev/null; then
        pg_dump -h localhost -U hermes -d hermes \
            > "$BACKUP_DIR/${PROJECT_NAME}_db_$TIMESTAMP.sql"
        echo "✅ 数据库备份完成"
    else
        echo "⚠️  pg_dump 未找到，跳过数据库备份"
    fi
}

# 备份 Redis
backup_redis() {
    echo "🔴 备份 Redis..."

    if command -v redis-cli &> /dev/null; then
        redis-cli BGSAVE
        sleep 2
        cp /var/lib/redis/dump.rdb "$BACKUP_DIR/${PROJECT_NAME}_redis_$TIMESTAMP.rdb" 2>/dev/null || true
        echo "✅ Redis 备份完成"
    else
        echo "⚠️  redis-cli 未找到，跳过 Redis 备份"
    fi
}

# 备份配置文件
backup_config() {
    echo "⚙️  备份配置文件..."

    tar -czf "$BACKUP_DIR/${PROJECT_NAME}_config_$TIMESTAMP.tar.gz" \
        -C . \
        .env \
        config.json \
        src-tauri/tauri.conf.json \
        2>/dev/null || true

    echo "✅ 配置文件备份完成"
}

# 备份日志
backup_logs() {
    echo "📝 备份日志文件..."

    if [ -d "logs" ]; then
        tar -czf "$BACKUP_DIR/${PROJECT_NAME}_logs_$TIMESTAMP.tar.gz" \
            -C . \
            logs/
        echo "✅ 日志文件备份完成"
    else
        echo "⚠️  日志目录不存在，跳过日志备份"
    fi
}

# 备份用户上传的文件
backup_uploads() {
    echo "📁 备份用户上传文件..."

    if [ -d "uploads" ]; then
        tar -czf "$BACKUP_DIR/${PROJECT_NAME}_uploads_$TIMESTAMP.tar.gz" \
            -C . \
            uploads/
        echo "✅ 用户上传文件备份完成"
    else
        echo "⚠️  上传目录不存在，跳过上传文件备份"
    fi
}

# 创建备份清单
create_manifest() {
    echo "📋 创建备份清单..."

    cat > "$BACKUP_DIR/${PROJECT_NAME}_manifest_$TIMESTAMP.txt" << EOF
Hermes Game Operator Backup Manifest
=====================================

Timestamp: $TIMESTAMP
Project: $PROJECT_NAME
Backup Directory: $BACKUP_DIR

Files Included:
- Database backup: ${PROJECT_NAME}_db_$TIMESTAMP.sql
- Redis backup: ${PROJECT_NAME}_redis_$TIMESTAMP.rdb
- Configuration: ${PROJECT_NAME}_config_$TIMESTAMP.tar.gz
- Logs: ${PROJECT_NAME}_logs_$TIMESTAMP.tar.gz
- Uploads: ${PROJECT_NAME}_uploads_$TIMESTAMP.tar.gz

Backup Size: $(du -sh "$BACKUP_DIR/${PROJECT_NAME}_manifest_$TIMESTAMP.txt" | cut -f1)

EOF

    echo "✅ 备份清单创建完成"
}

# 清理旧备份
cleanup_old_backups() {
    echo "🧹 清理旧备份..."

    # 保留最近 7 天的备份
    find "$BACKUP_DIR" -name "${PROJECT_NAME}_*" -type f -mtime +7 -delete

    echo "✅ 旧备份清理完成"
}

# 压缩备份
compress_backup() {
    echo "📦 压缩备份..."

    tar -czf "$BACKUP_DIR/${PROJECT_NAME}_full_$TIMESTAMP.tar.gz" \
        -C "$BACKUP_DIR" \
        ${PROJECT_NAME}_db_$TIMESTAMP.sql \
        ${PROJECT_NAME}_redis_$TIMESTAMP.rdb \
        ${PROJECT_NAME}_config_$TIMESTAMP.tar.gz \
        ${PROJECT_NAME}_logs_$TIMESTAMP.tar.gz \
        ${PROJECT_NAME}_uploads_$TIMESTAMP.tar.gz \
        ${PROJECT_NAME}_manifest_$TIMESTAMP.txt \
        2>/dev/null || true

    echo "✅ 备份压缩完成"
}

# 主函数
main() {
    backup_database
    backup_redis
    backup_config
    backup_logs
    backup_uploads
    create_manifest
    compress_backup
    cleanup_old_backups

    echo "🎉 备份完成！"
    echo "备份文件: $BACKUP_DIR/${PROJECT_NAME}_full_$TIMESTAMP.tar.gz"
}

main