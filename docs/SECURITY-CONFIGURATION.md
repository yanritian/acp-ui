# 安全配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 安全架构

```
┌─────────────────────────────────────┐
│          Security Layers            │
├─────────────────────────────────────┤
│  Network Security (VPC, SG, WAF)    │
├─────────────────────────────────────┤
│  Application Security (Auth, CORS)  │
├─────────────────────────────────────┤
│  Data Security (Encryption, KMS)    │
├─────────────────────────────────────┤
│  Identity Security (IAM, MFA)       │
└─────────────────────────────────────┘
```

---

## 身份和访问管理 (IAM)

### IAM 角色

```yaml
# iam-role.yaml
Resources:
  HermesRole:
    Type: AWS::IAM::Role
    Properties:
      RoleName: hermes-game-operator-role
      AssumeRolePolicyDocument:
        Version: "2012-10-17"
        Statement:
          - Effect: Allow
            Principal:
              Service: ec2.amazonaws.com
            Action: sts:AssumeRole
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore
      Policies:
        - PolicyName: HermesPolicy
          PolicyDocument:
            Version: "2012-10-17"
            Statement:
              - Effect: Allow
                Action:
                  - s3:GetObject
                  - s3:PutObject
                Resource: arn:aws:s3:::hermes-data/*
              - Effect: Allow
                Action:
                  - logs:CreateLogGroup
                  - logs:CreateLogStream
                  - logs:PutLogEvents
                Resource: arn:aws:logs:*:*:*
```

### IAM 用户

```yaml
Resources:
  HermesUser:
    Type: AWS::IAM::User
    Properties:
      UserName: hermes-service-user
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/IAMUserChangePassword
      LoginProfile:
        Password: !Ref UserPassword
        PasswordResetRequired: true

  HermesAccessKey:
    Type: AWS::IAM::AccessKey
    Properties:
      UserName: !Ref HermesUser
```

### 最佳实践

```yaml
# 强制 MFA
Resources:
  MFAPolicy:
    Type: AWS::IAM::Policy
    Properties:
      PolicyName: RequireMFA
      PolicyDocument:
        Version: "2012-10-17"
        Statement:
          - Effect: Deny
            Action: "*"
            Resource: "*"
            Condition:
              BoolIfExists:
                aws:MultiFactorAuthPresent: false
```

---

## 网络安全

### VPC 安全

```yaml
# vpc-security.yaml
Resources:
  HermesVPC:
    Type: AWS::EC2::VPC
    Properties:
      CidrBlock: 10.0.0.0/16
      EnableDnsHostnames: true
      EnableDnsSupport: true
      InstanceTenancy: dedicated

  FlowLog:
    Type: AWS::EC2::FlowLog
    Properties:
      ResourceId: !Ref HermesVPC
      ResourceType: VPC
      TrafficType: ALL
      LogDestinationType: cloud-watch-logs
      LogGroupName: /aws/vpc/flow-logs
```

### 安全组

```yaml
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
      SecurityGroupEgress:
        - IpProtocol: tcp
          FromPort: 443
          ToPort: 443
          CidrIp: 0.0.0.0/0
```

### WAF 规则

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
        - Name: SQLInjectionRule
          Priority: 1
          Statement:
            SqlInjectionMatchStatement:
              FieldToMatch:
                Body: {}
              TextTransformations:
                - Priority: 1
                  Type: URL_DECODE
          Action:
            Block: {}
          VisibilityConfig:
            SampledRequestsEnabled: true
            CloudWatchMetricsEnabled: true
            MetricName: SQLInjectionRule
```

---

## 应用安全

### CORS 配置

```javascript
// cors.js
const cors = require('cors');

app.use(cors({
  origin: [
    'https://your-domain.com',
    'https://www.your-domain.com'
  ],
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization'],
  credentials: true,
  maxAge: 86400 // 24 小时
}));
```

### 安全头

```javascript
// security-headers.js
const helmet = require('helmet');

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'"],
      fontSrc: ["'self'"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"]
    }
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true
  },
  referrerPolicy: { policy: 'strict-origin-when-cross-origin' }
}));
```

---

## 数据安全

### 加密配置

```yaml
# encryption.yaml
Resources:
  HermesKMSKey:
    Type: AWS::KMS::Key
    Properties:
      Description: KMS key for Hermes data encryption
      KeyPolicy:
        Version: "2012-10-17"
        Statement:
          - Sid: Enable IAM User Permissions
            Effect: Allow
            Principal:
              AWS: !Sub arn:aws:iam::${AWS::AccountId}:root
            Action: kms:*
            Resource: "*"
          - Sid: Allow use of the key
            Effect: Allow
            Principal:
              AWS: !GetAtt HermesRole.Arn
            Action:
              - kms:Encrypt
              - kms:Decrypt
              - kms:ReEncrypt*
              - kms:GenerateDataKey*
              - kms:DescribeKey
            Resource: "*"

  HermesBucket:
    Type: AWS::S3::Bucket
    Properties:
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: aws:kms
              KMSMasterKeyID: !Ref HermesKMSKey
```

### 数据库加密

```sql
-- 启用 SSL
ALTER SYSTEM SET ssl = on;
ALTER SYSTEM SET ssl_cert_file = '/etc/ssl/certs/server.crt';
ALTER SYSTEM SET ssl_key_file = '/etc/ssl/private/server.key';

-- 强制 SSL 连接
-- pg_hba.conf
hostssl all all 0.0.0.0/0 cert
```

---

## 审计和合规

### CloudTrail

```yaml
Resources:
  HermesTrail:
    Type: AWS::CloudTrail::Trail
    Properties:
      TrailName: hermes-trail
      S3BucketName: !Ref TrailBucket
      IsLogging: true
      EnableLogFileValidation: true
      IncludeGlobalServiceEvents: true
      IsMultiRegionTrail: true
      CloudWatchLogsLogGroupArn: !GetAtt TrailLogGroup.Arn
      CloudWatchLogsRoleArn: !GetAtt TrailRole.Arn
```

### Config 规则

```yaml
Resources:
  EncryptedVolumeRule:
    Type: AWS::Config::ConfigRule
    Properties:
      ConfigRuleName: encrypted-volumes
      Source:
        Owner: AWS
        SourceIdentifier: ENCRYPTED_VOLUMES
      Scope:
        ComplianceResourceTypes:
          - AWS::EC2::Volume
```

---

## 安全监控

### GuardDuty

```yaml
Resources:
  HermesDetector:
    Type: AWS::GuardDuty::Detector
    Properties:
      Enable: true
      FindingPublishingFrequency: FIFTEEN_MINUTES
```

### Security Hub

```yaml
Resources:
  HermesHub:
    Type: AWS::SecurityHub::Hub
    
  SecurityStandard:
    Type: AWS::SecurityHub::StandardsSubscription
    Properties:
      StandardsArn: arn:aws:securityhub:::standards/aws-foundational-security-best-practices/v/1.0.0
```

---

## 安全最佳实践

### 1. 最小权限原则

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::hermes-data/specific-path/*"
    }
  ]
}
```

### 2. 定期轮换密钥

```bash
# 每 90 天轮换
aws iam create-access-key --user-name hermes-service-user
aws iam update-access-key --user-name hermes-service-user --access-key-id OLD_KEY --status Inactive
```

### 3. 启用 MFA

```bash
# 为所有用户启用 MFA
aws iam create-virtual-mfa-device --virtual-mfa-device-name hermes-mfa
```

### 4. 定期安全扫描

```bash
# 使用 Inspector
aws inspector start-assessment-run --assessment-template-arn arn:aws:inspector:us-east-1:123456789:target/0-ABCD/template/0-EFGH
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Security Team