# 存储配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 存储架构

```
┌─────────────────────────────────────┐
│        Application Layer            │
└─────────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│Block  │ │File  │ │Object │
│Storage│ │System│ │Storage│
│(EBS)  │ │(EFS) │ │(S3)   │
└───────┘ └──────┘ └───────┘
```

---

## 块存储 (EBS)

### 卷类型选择

| 类型 | IOPS | 吞吐量 | 适用场景 |
|------|------|--------|---------|
| **gp3** | 3000 | 125 MB/s | 通用工作负载 |
| **io2** | 64000 | 1000 MB/s | 关键应用 |
| **st1** | 500 | 500 MB/s | 大数据 |
| **sc1** | 250 | 250 MB/s | 冷数据 |

### EBS 配置

```yaml
# ebs-volume.yaml
Resources:
  HermesVolume:
    Type: AWS::EC2::Volume
    Properties:
      AvailabilityZone: us-east-1a
      Size: 100
      VolumeType: gp3
      Iops: 3000
      Throughput: 125
      Encrypted: true
      Tags:
        - Key: Name
          Value: hermes-data
```

### 挂载卷

```bash
# 附加卷到实例
aws ec2 attach-volume \
  --volume-id vol-12345678 \
  --instance-id i-12345678 \
  --device /dev/xvdf

# 挂载文件系统
sudo mount /dev/xvdf /mnt/data

# 添加到 fstab
echo '/dev/xvdf /mnt/data ext4 defaults,nofail 0 2' | sudo tee -a /etc/fstab
```

---

## 文件系统 (EFS)

### EFS 配置

```yaml
# efs.yaml
Resources:
  HermesFileSystem:
    Type: AWS::EFS::FileSystem
    Properties:
      Encrypted: true
      PerformanceMode: generalPurpose
      ThroughputMode: bursting
      FileSystemPolicy:
        Version: "2012-10-17"
        Statement:
          - Effect: "Allow"
            Action:
              - "elasticfilesystem:ClientMount"
              - "elasticfilesystem:ClientWrite"
            Principal:
              AWS: "*"

  HermesMountTarget:
    Type: AWS::EFS::MountTarget
    Properties:
      FileSystemId: !Ref HermesFileSystem
      SubnetId: !Ref PrivateSubnet1
      SecurityGroups:
        - !Ref EFSSecurityGroup
```

### 挂载 EFS

```bash
# 安装 EFS 工具
sudo yum install -y amazon-efs-utils

# 挂载 EFS
sudo mount -t efs fs-12345678:/ /mnt/efs

# 添加到 fstab
echo 'fs-12345678:/ /mnt/efs efs defaults,_netdev 0 0' | sudo tee -a /etc/fstab
```

---

## 对象存储 (S3)

### S3 存储桶

```yaml
# s3-bucket.yaml
Resources:
  HermesBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketName: hermes-data-12345678
      VersioningConfiguration:
        Status: Enabled
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        BlockPublicPolicy: true
        IgnorePublicAcls: true
        RestrictPublicBuckets: true
      LifecycleConfiguration:
        Rules:
          - Id: ArchiveOldFiles
            Status: Enabled
            Transitions:
              - TransitionInDays: 90
                StorageClass: GLACIER
```

### S3 访问策略

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject"
      ],
      "Resource": "arn:aws:s3:::hermes-data-12345678/*"
    },
    {
      "Effect": "Allow",
      "Action": [
        "s3:ListBucket"
      ],
      "Resource": "arn:aws:s3:::hermes-data-12345678"
    }
  ]
}
```

---

## 数据库存储

### RDS 存储

```yaml
# rds.yaml
Resources:
  HermesDB:
    Type: AWS::RDS::DBInstance
    Properties:
      DBInstanceClass: db.r5.large
      Engine: postgres
      EngineVersion: "15.4"
      AllocatedStorage: 100
      MaxAllocatedStorage: 500
      StorageType: gp3
      StorageEncrypted: true
      MultiAZ: true
      BackupRetentionPeriod: 7
      PreferredBackupWindow: "03:00-04:00"
      PreferredMaintenanceWindow: "sun:04:00-sun:05:00"
```

### 存储优化

```sql
-- 启用表分区
CREATE TABLE tasks (
  task_id UUID,
  created_at TIMESTAMPTZ
) PARTITION BY RANGE (created_at);

-- 创建分区
CREATE TABLE tasks_2026_07 PARTITION OF tasks
  FOR VALUES FROM ('2026-07-01') TO ('2026-08-01');

-- 启用压缩
ALTER TABLE tasks SET (
  toast_tuple_target = 128,
  parallel_workers = 4
);
```

---

## 备份和恢复

### 自动备份

```yaml
# backup-lambda.yaml
Resources:
  BackupFunction:
    Type: AWS::Lambda::Function
    Properties:
      FunctionName: hermes-backup
      Runtime: python3.9
      Handler: index.handler
      Environment:
        Variables:
          SOURCE_BUCKET: hermes-data-12345678
          DEST_BUCKET: hermes-backup-12345678
      Timeout: 300

  BackupSchedule:
    Type: AWS::Events::Rule
    Properties:
      ScheduleExpression: "rate(1 day)"
      Targets:
        - Arn: !GetAtt BackupFunction.Arn
          Id: BackupTarget
```

### 备份脚本

```bash
#!/bin/bash
# backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SOURCE_BUCKET="hermes-data-12345678"
DEST_BUCKET="hermes-backup-12345678"

# 备份数据库
pg_dump -h localhost -U hermes hermes | gzip > /tmp/db_$TIMESTAMP.sql.gz

# 上传到 S3
aws s3 cp /tmp/db_$TIMESTAMP.sql.gz s3://$DEST_BUCKET/db/

# 备份应用数据
aws s3 sync s3://$SOURCE_BUCKET s3://$DEST_BUCKET/data_$TIMESTAMP/

# 清理旧备份
aws s3 ls s3://$DEST_BUCKET/ | grep db_ | awk '{print $4}' | sort -r | tail -n +8 | xargs -I {} aws s3 rm s3://$DEST_BUCKET/{}

# 清理临时文件
rm /tmp/db_$TIMESTAMP.sql.gz
```

---

## 监控存储

### CloudWatch 指标

```yaml
# storage-alarm.yaml
Resources:
  StorageAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: hermes-storage-high
      MetricName: VolumeUsagePercent
      Namespace: AWS/EBS
      Statistic: Maximum
      Period: 300
      EvaluationPeriods: 2
      Threshold: 80
      ComparisonOperator: GreaterThanThreshold
      Dimensions:
        - Name: VolumeId
          Value: vol-12345678
```

### 存储监控

```bash
#!/bin/bash
# monitor-storage.sh

# 检查磁盘使用
df -h /mnt/data | awk 'NR==2 {print $5}' | sed 's/%//' | {
  read usage
  if [ $usage -gt 80 ]; then
    echo "WARNING: Disk usage is $usage%"
    # 发送告警
  fi
}

# 检查 IOPS
iostat -x 1 1 | grep -E 'sd[a-z]' | awk '{print $1, $8, $9}'
```

---

## 最佳实践

### 1. 始终加密

```yaml
StorageEncrypted: true
KmsKeyId: !Ref HermesKMSKey
```

### 2. 启用版本控制

```yaml
VersioningConfiguration:
  Status: Enabled
```

### 3. 设置生命周期策略

```yaml
LifecycleConfiguration:
  Rules:
    - Id: ArchiveOldFiles
      Status: Enabled
      Transitions:
        - TransitionInDays: 90
          StorageClass: GLACIER
```

### 4. 监控存储使用

```bash
# 定期检查
aws cloudwatch get-metric-statistics \
  --namespace AWS/EBS \
  --metric-name VolumeUsagePercent \
  --dimensions Name=VolumeId,Value=vol-12345678 \
  --start-time $(date -d '1 hour ago') \
  --end-time $(date) \
  --period 300 \
  --statistics Maximum
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Storage Team