# 网络配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 网络架构

```
┌─────────────────────────────────────┐
│          Internet                   │
└─────────────────────────────────────┘
              │
┌─────────────▼─────────────┐
│      Load Balancer        │
│   (Public IP: x.x.x.x)   │
└─────────────┬─────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│  App  │ │  App │ │  App  │
│ Node  │ │ Node │ │ Node  │
└───────┘ └──────┘ └───────┘
    │         │         │
    └─────────┼─────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│  DB   │ │Cache │ │  MQ   │
│Primary│ │Redis │ │Kafka  │
└───────┘ └──────┘ └───────┘
```

---

## VPC 配置

### AWS VPC

```yaml
# vpc.yaml
Resources:
  HermesVPC:
    Type: AWS::EC2::VPC
    Properties:
      CidrBlock: 10.0.0.0/16
      EnableDnsHostnames: true
      EnableDnsSupport: true
      Tags:
        - Key: Name
          Value: hermes-vpc

  PublicSubnet1:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref HermesVPC
      CidrBlock: 10.0.1.0/24
      AvailabilityZone: !Select [0, !GetAZs '']
      MapPublicIpOnLaunch: true
      Tags:
        - Key: Name
          Value: hermes-public-1

  PrivateSubnet1:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref HermesVPC
      CidrBlock: 10.0.10.0/24
      AvailabilityZone: !Select [0, !GetAZs '']
      Tags:
        - Key: Name
          Value: hermes-private-1
```

### 子网规划

```
VPC: 10.0.0.0/16

Public Subnets:
  - 10.0.1.0/24 (LB, NAT)
  - 10.0.2.0/24 (LB, NAT)

Private Subnets:
  - 10.0.10.0/24 (App)
  - 10.0.11.0/24 (App)
  - 10.0.12.0/24 (App)

Database Subnets:
  - 10.0.20.0/24 (DB)
  - 10.0.21.0/24 (DB)

Cache Subnets:
  - 10.0.30.0/24 (Redis)
  - 10.0.31.0/24 (Redis)
```

---

## 安全组配置

### 应用安全组

```yaml
# security-group.yaml
Resources:
  AppSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Security group for app nodes
      VpcId: !Ref HermesVPC
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 8080
          ToPort: 8080
          SourceSecurityGroupId: !Ref LBSecurityGroup
        - IpProtocol: tcp
          FromPort: 9090
          ToPort: 9090
          CidrIp: 10.0.0.0/16
```

### 数据库安全组

```yaml
Resources:
  DatabaseSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Security group for database
      VpcId: !Ref HermesVPC
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 5432
          ToPort: 5432
          SourceSecurityGroupId: !Ref AppSecurityGroup
```

---

## 负载均衡配置

### AWS ALB

```yaml
Resources:
  HermesALB:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Properties:
      Name: hermes-alb
      Scheme: internet-facing
      Type: application
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2
      SecurityGroups:
        - !Ref LBSecurityGroup

  TargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Name: hermes-target-group
      Port: 8080
      Protocol: HTTP
      VpcId: !Ref HermesVPC
      HealthCheckPath: /health
      HealthCheckIntervalSeconds: 30
      HealthyThresholdCount: 2
      UnhealthyThresholdCount: 3
```

### Nginx 负载均衡

```nginx
upstream hermes_backend {
    least_conn;
    server app1:8080 max_fails=3 fail_timeout=30s;
    server app2:8080 max_fails=3 fail_timeout=30s;
    server app3:8080 max_fails=3 fail_timeout=30s;
    
    keepalive 32;
}

server {
    listen 80;
    server_name api.your-domain.com;

    location / {
        proxy_pass http://hermes_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }
}
```

---

## DNS 配置

### Route 53

```yaml
Resources:
  HermesHostedZone:
    Type: AWS::Route53::HostedZone
    Properties:
      Name: your-domain.com

  HermesRecord:
    Type: AWS::Route53::RecordSet
    Properties:
      HostedZoneId: !Ref HermesHostedZone
      Name: api.your-domain.com
      Type: A
      AliasTarget:
        HostedZoneId: !GetAtt HermesALB.CanonicalHostedZoneID
        DNSName: !GetAtt HermesALB.DNSName
```

### 记录类型

```
A Record: api.your-domain.com -> ALB
CNAME: www.your-domain.com -> your-domain.com
MX: your-domain.com -> mail.your-domain.com
TXT: your-domain.com -> "v=spf1 include:_spf.google.com ~all"
```

---

## SSL/TLS 配置

### 证书管理

```yaml
Resources:
  HermesCertificate:
    Type: AWS::CertificateManager::Certificate
    Properties:
      DomainName: api.your-domain.com
      SubjectAlternativeNames:
        - "*.your-domain.com"
      ValidationMethod: DNS
```

### Nginx SSL

```nginx
server {
    listen 443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/ssl/certs/api.crt;
    ssl_certificate_key /etc/ssl/private/api.key;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;
    
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
}
```

---

## 防火墙配置

### AWS WAF

```yaml
Resources:
  HermesWAF:
    Type: AWS::WAFv2::WebACL
    Properties:
      Name: hermes-waf
      Scope: REGIONAL
      DefaultAction:
        Allow: {}
      Rules:
        - Name: RateLimitRule
          Priority: 1
          Statement:
            RateBasedStatement:
              Limit: 2000
              AggregateKeyType: IP
          Action:
            Block: {}
          VisibilityConfig:
            SampledRequestsEnabled: true
            CloudWatchMetricsEnabled: true
            MetricName: RateLimitRule
```

### iptables

```bash
#!/bin/bash

# 清除现有规则
iptables -F
iptables -X

# 默认策略
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# 允许回环
iptables -A INPUT -i lo -j ACCEPT

# 允许已建立的连接
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# 允许 SSH
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# 允许 HTTP/HTTPS
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# 允许应用端口
iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/16 -j ACCEPT

# 记录并拒绝其他
iptables -A INPUT -j LOG
iptables -A INPUT -j DROP
```

---

## 监控网络

### CloudWatch

```yaml
Resources:
  HermesNetworkAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: hermes-network-errors
      MetricName: HTTPCode_Target_5XX_Count
      Namespace: AWS/ApplicationELB
      Statistic: Sum
      Period: 60
      EvaluationPeriods: 2
      Threshold: 10
      ComparisonOperator: GreaterThanThreshold
      Dimensions:
        - Name: LoadBalancer
          Value: !Ref HermesALB
```

### Prometheus

```yaml
scrape_configs:
  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx-exporter:9113']
  
  - job_name: 'alb'
    ec2_service_discovery:
      region: us-east-1
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Network Team