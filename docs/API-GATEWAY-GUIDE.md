# API 网关配置指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的 API 网关配置和最佳实践。

---

## API 网关架构

```
┌─────────────────────────────────────┐
│         Client Applications         │
└─────────────────────────────────────┘
              │
┌─────────────▼─────────────┐
│      API Gateway          │
│  - Authentication         │
│  - Rate Limiting          │
│  - Routing                │
│  - Load Balancing         │
│  - Caching                │
└─────────────┬─────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│Task   │ │Event │ │Auth   │
│Service│ │Serv. │ │Service│
└───────┘ └──────┘ └───────┘
```

---

## Kong 配置

### 基本配置

```yaml
# kong.yml
_format_version: "2.1"

services:
  - name: task-service
    url: http://task-service:8080
    routes:
      - name: tasks-route
        paths:
          - /api/tasks
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 100
          hour: 1000
          policy: redis
          redis_host: redis
      - name: jwt
        config:
          claims_to_verify:
            - exp
      - name: cors
        config:
          origins:
            - "*"
          methods:
            - GET
            - POST
            - PUT
            - DELETE
```

### 认证配置

```yaml
plugins:
  - name: jwt
    config:
      key_claim_name: iss
      claims_to_verify:
        - exp
      secret_is_base64: false
      run_on_preflight: true

  - name: oauth2
    config:
      scopes:
        - read
        - write
        - admin
      mandatory_scope: true
      enable_authorization_code: true
      enable_client_credentials: true
```

### 速率限制

```yaml
plugins:
  - name: rate-limiting
    config:
      second: 10
      minute: 100
      hour: 1000
      day: 10000
      month: 100000
      limit_by: ip
      policy: redis
      redis_host: redis
      redis_port: 6379
      fault_tolerant: true
      hide_client_headers: false
```

---

## Nginx 配置

### 基本配置

```nginx
# nginx.conf
upstream task_service {
    least_conn;
    server task-service-1:8080;
    server task-service-2:8080;
    server task-service-3:8080;
}

server {
    listen 80;
    server_name api.your-domain.com;

    # 速率限制
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

    # 安全头
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    location /api/ {
        limit_req zone=api burst=20 nodelay;
        
        proxy_pass http://task_service;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 超时设置
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }

    location /health {
        proxy_pass http://task_service/health;
        access_log off;
    }
}
```

### SSL/TLS 配置

```nginx
server {
    listen 443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/ssl/certs/api.crt;
    ssl_certificate_key /etc/ssl/private/api.key;
    
    # SSL 优化
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;
    
    # HSTS
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
}
```

---

## AWS API Gateway

### CloudFormation 配置

```yaml
Resources:
  HermesApi:
    Type: AWS::ApiGatewayV2::Api
    Properties:
      Name: hermes-game-operator-api
      ProtocolType: HTTP
      CorsConfiguration:
        AllowOrigins:
          - "*"
        AllowMethods:
          - GET
          - POST
          - PUT
          - DELETE
        AllowHeaders:
          - Content-Type
          - Authorization

  TasksIntegration:
    Type: AWS::ApiGatewayV2::Integration
    Properties:
      ApiId: !Ref HermesApi
      IntegrationType: HTTP_PROXY
      IntegrationUri: !Ref TaskServiceLoadBalancer
      PayloadFormatVersion: "2.0"

  TasksRoute:
    Type: AWS::ApiGatewayV2::Route
    Properties:
      ApiId: !Ref HermesApi
      RouteKey: "ANY /api/tasks/{proxy+}"
      Target: !Join
        - /
        - - integrations
          - !Ref TasksIntegration
```

### 使用计划

```yaml
Resources:
  ApiUsagePlan:
    Type: AWS::ApiGatewayV2::Api
    Properties:
      ApiName: hermes-game-operator-api
      ProtocolType: HTTP

  UsagePlan:
    Type: AWS::ApiGateway::UsagePlan
    Properties:
      UsagePlanName: hermes-api-plan
      ApiStages:
        - ApiId: !Ref HermesApi
          Stage: prod
      Throttle:
        BurstLimit: 100
        RateLimit: 50
      Quota:
        Limit: 10000
        Period: MONTH
```

---

## 监控和日志

### 访问日志

```nginx
log_format api '$remote_addr - $remote_user [$time_local] '
               '"$request" $status $body_bytes_sent '
               '"$http_referer" "$http_user_agent" '
               '$request_time $upstream_response_time';

access_log /var/log/nginx/api.log api;
```

### 错误日志

```nginx
error_log /var/log/nginx/error.log warn;
```

### Prometheus 指标

```nginx
server {
    listen 9113;
    
    location /metrics {
        stub_status on;
        access_log off;
    }
}
```

---

## 最佳实践

### 1. 始终使用 HTTPS

```nginx
# 重定向 HTTP 到 HTTPS
server {
    listen 80;
    server_name api.your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

### 2. 实施速率限制

```yaml
plugins:
  - name: rate-limiting
    config:
      minute: 100
      policy: redis
```

### 3. 使用缓存

```nginx
proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=api_cache:10m max_size=1g;

location /api/ {
    proxy_cache api_cache;
    proxy_cache_valid 200 10m;
    proxy_cache_use_stale error timeout updating;
}
```

### 4. 启用 CORS

```nginx
location /api/ {
    if ($request_method = 'OPTIONS') {
        add_header 'Access-Control-Allow-Origin' '*';
        add_header 'Access-Control-Allow-Methods' 'GET, POST, PUT, DELETE, OPTIONS';
        add_header 'Access-Control-Allow-Headers' 'Content-Type, Authorization';
        return 204;
    }
}
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator API Gateway Team