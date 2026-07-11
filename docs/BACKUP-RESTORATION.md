# 备份和恢复指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 备份策略

### 备份类型

| 类型 | 频率 | 保留时间 | 用途 |
|------|------|---------|------|
| **完整备份** | 每天 | 30 天 | 完整数据恢复 |
| **增量备份** | 每小时 | 7 天 | 快速恢复 |
| **日志备份** | 实时 | 7 天 | 时间点恢复 |
| **配置备份** | 每天 | 90 天 | 配置恢复 |

### 备份存储

```
备份位置:
├── 本地存储 (快速恢复)
│   └── /backup/local/
├── 对象存储 (长期保留)
│   └── s3://hermes-backups/
└── 离线存储 (灾难恢复)
    └── 磁带库 / Glacier
```

---

## 数据库备份

### PostgreSQL 备份

```bash
#!/bin/bash
# db-backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/database"
BUCKET="s3://hermes-backups/database"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 完整备份
pg_dump -h localhost -U hermes -F c -f $BACKUP_DIR/full_$TIMESTAMP.backup hermes

# 压缩
gzip $BACKUP_DIR/full_$TIMESTAMP.backup

# 上传到 S3
aws s3 cp $BACKUP_DIR/full_$TIMESTAMP.backup.gz $BUCKET/

# 创建 WAL 归档备份
pg_basebackup -h localhost -U hermes -D $BACKUP_DIR/base_$TIMESTAMP -Ft -z

# 上传到 S3
aws s3 cp $BACKUP_DIR/base_$TIMESTAMP.tar.gz $BUCKET/

# 清理旧备份（保留 7 天）
find $BACKUP_DIR -name "*.backup.gz" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

# 记录备份元数据
cat > $BACKUP_DIR/metadata_$TIMESTAMP.json << EOF
{
  "timestamp": "$TIMESTAMP",
  "database": "hermes",
  "size": $(du -sh $BACKUP_DIR/full_$TIMESTAMP.backup.gz | cut -f1),
  "type": "full"
}
EOF

echo "Backup completed: $TIMESTAMP"
```

### 自动备份调度

```yaml
# backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: hermes-db-backup
spec:
  schedule: "0 2 * * *"  # 每天凌晨 2 点
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15
            command:
            - /bin/sh
            - -c
            - /scripts/db-backup.sh
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: hermes-db-secret
                  key: password
            volumeMounts:
            - name: backup-script
              mountPath: /scripts
          volumes:
          - name: backup-script
            configMap:
              name: backup-script
          restartPolicy: OnFailure
```

---

## 应用数据备份

### S3 数据备份

```bash
#!/bin/bash
# s3-backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SOURCE_BUCKET="hermes-data"
BACKUP_BUCKET="hermes-backups/data"

# 同步数据
aws s3 sync s3://$SOURCE_BUCKET s3://$BACKUP_BUCKET/$TIMESTAMP/

# 记录元数据
cat > /tmp/metadata_$TIMESTAMP.json << EOF
{
  "timestamp": "$TIMESTAMP",
  "source_bucket": "$SOURCE_BUCKET",
  "backup_bucket": "$BACKUP_BUCKET",
  "path": "$TIMESTAMP"
}
EOF

aws s3 cp /tmp/metadata_$TIMESTAMP.json s3://$BACKUP_BUCKET/

# 清理旧备份（保留 30 天）
aws s3 ls s3://$BACKUP_BUCKET/ | grep -E '^[0-9]{8}_[0-9]{6}/$' | awk '{print $4}' | sort -r | tail -n +31 | xargs -I {} aws s3 rm s3://$BACKUP_BUCKET/{} --recursive

echo "S3 backup completed: $TIMESTAMP"
```

### 配置文件备份

```bash
#!/bin/bash
# config-backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/config"
BUCKET="s3://hermes-backups/config"

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份 Kubernetes 配置
kubectl get all -n hermes -o yaml > $BACKUP_DIR/k8s_$TIMESTAMP.yaml

# 备份环境变量
kubectl get secret -n hermes -o yaml > $BACKUP_DIR/secrets_$TIMESTAMP.yaml

# 备份 ConfigMap
kubectl get configmap -n hermes -o yaml > $BACKUP_DIR/configmaps_$TIMESTAMP.yaml

# 压缩
tar -czf $BACKUP_DIR/config_$TIMESTAMP.tar.gz -C $BACKUP_DIR .

# 上传到 S3
aws s3 cp $BACKUP_DIR/config_$TIMESTAMP.tar.gz $BUCKET/

# 清理
rm -rf $BACKUP_DIR/k8s_$TIMESTAMP.yaml
rm -rf $BACKUP_DIR/secrets_$TIMESTAMP.yaml
rm -rf $BACKUP_DIR/configmaps_$TIMESTAMP.yaml

echo "Config backup completed: $TIMESTAMP"
```

---

## 恢复流程

### 数据库恢复

```bash
#!/bin/bash
# db-restore.sh

BACKUP_FILE=$1
RESTORE_DIR="/restore/database"

# 创建恢复目录
mkdir -p $RESTORE_DIR

# 下载备份
aws s3 cp s3://hermes-backups/database/$BACKUP_FILE $RESTORE_DIR/

# 解压
gunzip $RESTORE_DIR/$BACKUP_FILE

# 恢复数据库
pg_restore -h localhost -U hermes -d hermes -c $RESTORE_DIR/${BACKUP_FILE%.gz}

# 验证恢复
psql -h localhost -U hermes -d hermes -c "SELECT COUNT(*) FROM tasks;"

# 清理
rm -rf $RESTORE_DIR

echo "Database restore completed"
```

### 应用数据恢复

```bash
#!/bin/bash
# s3-restore.sh

TIMESTAMP=$1
SOURCE_BUCKET="hermes-backups/data"
TARGET_BUCKET="hermes-data"

# 同步数据
aws s3 sync s3://$SOURCE_BUCKET/$TIMESTAMP/ s3://$TARGET_BUCKET/

# 验证
aws s3 ls s3://$TARGET_BUCKET/ | wc -l

echo "S3 restore completed: $TIMESTAMP"
```

---

## 测试恢复

### 定期测试

```bash
#!/bin/bash
# test-restore.sh

TEST_ENV="test-restore-$(date +%Y%m%d)"
BACKUP_FILE=$(aws s3 ls s3://hermes-backups/database/ | sort | tail -n 1 | awk '{print $4}')

# 创建测试环境
aws rds create-db-instance \
  --db-instance-identifier $TEST_ENV \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username hermes \
  --master-user-password test-password

# 恢复备份
./db-restore.sh $BACKUP_FILE

# 运行验证测试
npm run test:restore

# 清理测试环境
aws rds delete-db-instance \
  --db-instance-identifier $TEST_ENV \
  --skip-final-snapshot

echo "Restore test completed"
```

---

## 备份监控

### 备份状态检查

```bash
#!/bin/bash
# check-backup.sh

# 检查最新备份时间
LATEST_BACKUP=$(aws s3 ls s3://hermes-backups/database/ | sort | tail -n 1 | awk '{print $4}')
BACKUP_TIME=$(echo $LATEST_BACKUP | sed 's/.*_\(.*\).backup.gz/\1/')

# 检查备份年龄（小时）
BACKUP_AGE=$(( ($(date +%s) - $(date -d "$BACKUP_TIME" +%s)) / 3600 ))

if [ $BACKUP_AGE -gt 25 ]; then
  echo "WARNING: Latest backup is $BACKUP_AGE hours old"
  # 发送告警
  curl -X POST https://slack.com/api/chat.postMessage \
    -H "Authorization: Bearer $SLACK_TOKEN" \
    -d "{\"channel\":\"#alerts\",\"text\":\"Backup age: $BACKUP_AGE hours\"}"
fi
```

### 备份指标

```yaml
# backup-metrics.yaml
groups:
  - name: backup-metrics
    rules:
      - alert: BackupFailed
        expr: backup_last_success_timestamp < time() - 86400
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: "Backup failed or not completed in 24 hours"
```

---

## 灾难恢复

### RTO 和 RPO

| 场景 | RTO | RPO | 策略 |
|------|-----|-----|------|
| 应用故障 | 15 分钟 | 0 | 自动重启 |
| 数据库故障 | 1 小时 | 1 小时 | 主从切换 |
| 区域故障 | 4 小时 | 1 小时 | 跨区域恢复 |
| 灾难 | 24 小时 | 24 小时 | 离线恢复 |

### 灾难恢复演练

```bash
#!/bin/bash
# dr-drill.sh

# 模拟区域故障
echo "Starting DR drill..."

# 切换到 DR 区域
aws route53 change-resource-record-sets \
  --hosted-zone-id $ZONE_ID \
  --change-batch file://dr-dns-change.json

# 恢复数据库
./db-restore.sh $(aws s3 ls s3://hermes-backups-dr/database/ | sort | tail -n 1 | awk '{print $4}')

# 验证服务
curl -f https://dr.your-domain.com/health

# 恢复原区域
aws route53 change-resource-record-sets \
  --hosted-zone-id $ZONE_ID \
  --change-batch file://restore-dns-change.json

echo "DR drill completed"
```

---

## 备份最佳实践

### 3-2-1 规则

- **3** 份数据副本
- **2** 种存储介质
- **1** 份离线备份

### 加密

```bash
# 加密备份
gpg --symmetric --cipher-algo AES256 backup.tar.gz

# 解密备份
gpg --decrypt backup.tar.gz.gpg > backup.tar.gz
```

### 压缩

```bash
# 高压缩比
tar -czf backup.tar.gz --lzma /data

# 快速压缩
tar -czf backup.tar.gz /data
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Backup Team