# 集成指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 与第三方系统和工具的集成说明。

---

## CI/CD 集成

### GitHub Actions

```yaml
# .github/workflows/hermes-operator.yml
name: Hermes Game Operator Integration

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run tests
      run: npm test
    
    - name: Build
      run: npm run build

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3
    
    - name: Deploy to production
      run: |
        curl -X POST https://your-domain.com/api/deploy \
          -H "Authorization: Bearer ${{ secrets.DEPLOY_TOKEN }}"
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy

test:
  stage: test
  image: node:18
  script:
    - npm ci
    - npm test

build:
  stage: build
  image: node:18
  script:
    - npm run build
  artifacts:
    paths:
      - dist/

deploy:
  stage: deploy
  image: alpine:latest
  script:
    - apk add curl
    - curl -X POST https://your-domain.com/api/deploy
  only:
    - main
```

### Jenkins

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    stages {
        stage('Test') {
            steps {
                sh 'npm ci'
                sh 'npm test'
            }
        }
        
        stage('Build') {
            steps {
                sh 'npm run build'
            }
        }
        
        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                sh '''
                    curl -X POST https://your-domain.com/api/deploy \
                      -H "Authorization: Bearer ${DEPLOY_TOKEN}"
                '''
            }
        }
    }
}
```

---

## 监控集成

### Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'hermes-game-operator'
    metrics_path: '/metrics'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:8080']
    
    # 自定义指标
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'hermes_.*'
        action: keep
```

### Grafana

```json
{
  "dashboard": {
    "title": "Hermes Game Operator",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{path}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "{{status}}"
          }
        ]
      }
    ]
  }
}
```

### Datadog

```yaml
# datadog.yaml
init_config:

instances:
  - url: https://your-domain.com
    timeout: 10
    tags:
      - env:production
      - service:hermes-game-operator

logs:
  - type: file
    path: /var/log/hermes-game-operator/app.log
    service: hermes-game-operator
    source: nodejs
```

---

## 日志集成

### ELK Stack

```ruby
# logstash.conf
input {
  file {
    path => "/var/log/hermes-game-operator/app.log"
    start_position => "beginning"
    codec => "json"
  }
}

filter {
  json {
    source => "message"
  }
  
  date {
    match => [ "timestamp", "ISO8601" ]
  }
  
  mutate {
    add_field => { "service" => "hermes-game-operator" }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "hermes-game-operator-%{+YYYY.MM.dd}"
  }
}
```

### Splunk

```ini
# inputs.conf
[monitor:///var/log/hermes-game-operator/app.log]
disabled = false
index = hermes
sourcetype = hermes-game-operator

# props.conf
[hermes-game-operator]
TIME_FORMAT = %Y-%m-%dT%H:%M:%S.%3NZ
MAX_TIMESTAMP_LOOKAHEAD = 30
SHOULD_LINEMERGE = false
```

---

## 通知集成

### Slack

```bash
# 发送通知到 Slack
curl -X POST https://slack.com/api/chat.postMessage \
  -H "Authorization: Bearer $SLACK_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "channel": "#deployments",
    "text": "🚀 Hermes Game Operator deployed successfully",
    "attachments": [
      {
        "color": "good",
        "fields": [
          {
            "title": "Version",
            "value": "0.1.0-alpha",
            "short": true
          },
          {
            "title": "Environment",
            "value": "Production",
            "short": true
          }
        ]
      }
    ]
  }'
```

### Microsoft Teams

```bash
# 发送通知到 Teams
curl -X POST "https://outlook.office.com/webhook/YOUR-WEBHOOK-URL" \
  -H "Content-Type: application/json" \
  -d '{
    "@type": "MessageCard",
    "@context": "http://schema.org/extensions",
    "themeColor": "0076D7",
    "summary": "Hermes Game Operator Deployment",
    "sections": [{
      "activityTitle": "Deployment Successful",
      "activitySubtitle": "Hermes Game Operator v0.1.0-alpha",
      "facts": [{
        "name": "Environment",
        "value": "Production"
      }],
      "markdown": true
    }]
  }'
```

### PagerDuty

```bash
# 创建 PagerDuty 事件
curl -X POST https://events.pagerduty.com/v2/enqueue \
  -H "Content-Type: application/json" \
  -d '{
    "routing_key": "YOUR-ROUTING-KEY",
    "event_action": "trigger",
    "payload": {
      "summary": "Hermes Game Operator service degraded",
      "severity": "warning",
      "source": "hermes-game-operator"
    }
  }'
```

---

## 存储集成

### AWS S3

```javascript
// 使用 S3 存储文件
const AWS = require('aws-sdk');

const s3 = new AWS.S3({
  accessKeyId: process.env.AWS_ACCESS_KEY_ID,
  secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  region: 'us-east-1'
});

// 上传文件
await s3.putObject({
  Bucket: 'hermes-data',
  Key: 'backups/db-backup.sql',
  Body: backupData
}).promise();
```

### Azure Blob Storage

```javascript
// 使用 Azure Blob Storage
const { BlobServiceClient } = require('@azure/storage-blob');

const blobServiceClient = BlobServiceClient.fromConnectionString(
  process.env.AZURE_STORAGE_CONNECTION_STRING
);

const containerClient = blobServiceClient.getContainerClient('hermes-data');
const blockBlobClient = containerClient.getBlockBlobClient('backups/db-backup.sql');

await blockBlobClient.upload(backupData, backupData.length);
```

---

## 数据库集成

### PostgreSQL

```javascript
// 使用 PostgreSQL
const { Pool } = require('pg');

const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  ssl: { rejectUnauthorized: false }
});

// 查询任务
const tasks = await pool.query(`
  SELECT * FROM tasks 
  WHERE status = $1 
  ORDER BY created_at DESC 
  LIMIT $2
`, ['running', 50]);
```

### MongoDB

```javascript
// 使用 MongoDB
const { MongoClient } = require('mongodb');

const client = new MongoClient(process.env.MONGODB_URL);
const db = client.db('hermes');
const tasks = db.collection('tasks');

// 查询任务
const tasks = await tasks.find({ status: 'running' })
  .sort({ created_at: -1 })
  .limit(50)
  .toArray();
```

---

## 缓存集成

### Redis

```javascript
// 使用 Redis
const Redis = require('ioredis');

const redis = new Redis(process.env.REDIS_URL);

// 缓存任务
await redis.setex('task:task_001', 3600, JSON.stringify(task));

// 获取缓存
const cachedTask = await redis.get('task:task_001');
```

### Memcached

```javascript
// 使用 Memcached
const Memcached = require('memcached');

const memcached = new Memcached('localhost:11211');

// 缓存任务
memcached.set('task:task_001', JSON.stringify(task), 3600, (err) => {
  if (err) console.error(err);
});

// 获取缓存
memcached.get('task:task_001', (err, data) => {
  if (err) console.error(err);
  console.log(JSON.parse(data));
});
```

---

## 消息队列集成

### RabbitMQ

```javascript
// 使用 RabbitMQ
const amqp = require('amqplib');

const connection = await amqp.connect(process.env.RABBITMQ_URL);
const channel = await connection.createChannel();

// 发布消息
await channel.assertQueue('hermes-tasks');
channel.sendToQueue('hermes-tasks', Buffer.from(JSON.stringify(task)));

// 消费消息
await channel.consume('hermes-tasks', (msg) => {
  const task = JSON.parse(msg.content.toString());
  processTask(task);
  channel.ack(msg);
});
```

### Apache Kafka

```javascript
// 使用 Kafka
const { Kafka } = require('kafkajs');

const kafka = new Kafka({
  clientId: 'hermes-game-operator',
  brokers: ['localhost:9092']
});

const producer = kafka.producer();
const consumer = kafka.consumer({ groupId: 'hermes-group' });

// 发布消息
await producer.connect();
await producer.send({
  topic: 'hermes-tasks',
  messages: [{ value: JSON.stringify(task) }]
});

// 消费消息
await consumer.connect();
await consumer.subscribe({ topic: 'hermes-tasks' });
await consumer.run({
  eachMessage: async ({ message }) => {
    const task = JSON.parse(message.value.toString());
    processTask(task);
  }
});
```

---

## 认证集成

### OAuth2

```javascript
// 使用 OAuth2
const passport = require('passport');
const OAuth2Strategy = require('passport-oauth2').Strategy;

passport.use(new OAuth2Strategy({
  authorizationURL: 'https://auth.example.com/authorize',
  tokenURL: 'https://auth.example.com/token',
  clientID: process.env.CLIENT_ID,
  clientSecret: process.env.CLIENT_SECRET,
  callbackURL: 'https://your-domain.com/auth/callback'
}, (accessToken, refreshToken, profile, done) => {
  return done(null, profile);
}));
```

### SAML

```javascript
// 使用 SAML
const SamlStrategy = require('passport-saml').Strategy;

passport.use(new SamlStrategy({
  path: '/auth/callback',
  entryPoint: 'https://idp.example.com/saml/sso',
  issuer: 'hermes-game-operator',
  cert: process.env.SAML_CERT
}, (profile, done) => {
  return done(null, profile);
}));
```

---

## 示例应用

### 完整 CI/CD 流水线

```yaml
# 完整的 CI/CD 配置
name: Complete Pipeline

on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - run: npm ci
    - run: npm test
    
  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - run: npm run build
    
  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
    - name: Deploy
      run: ./scripts/deploy.sh
      
  notify:
    needs: deploy
    runs-on: ubuntu-latest
    steps:
    - name: Notify Slack
      run: ./scripts/notify-slack.sh
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Integration Team