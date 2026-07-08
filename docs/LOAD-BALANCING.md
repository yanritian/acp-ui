# Load Balancing Configuration Guide

## Overview

This guide covers load balancing setup for Hermes Game Operator in production environments.

---

## Load Balancing Strategies

### 1. Round Robin

**Description**: Distribute requests evenly across all servers

**Pros**:
- Simple to implement
- Even distribution

**Cons**:
- Doesn't consider server load
- Doesn't consider server health

**Configuration**:
```nginx
upstream hermes_backend {
    server 192.168.1.10:1420;
    server 192.168.1.11:1420;
    server 192.168.1.12:1420;
}
```

---

### 2. Least Connections

**Description**: Send requests to server with fewest active connections

**Pros**:
- Considers server load
- Better for long-running tasks

**Cons**:
- More complex
- Requires connection tracking

**Configuration**:
```nginx
upstream hermes_backend {
    least_conn;
    server 192.168.1.10:1420;
    server 192.168.1.11:1420;
    server 192.168.1.12:1420;
}
```

---

### 3. IP Hash

**Description**: Route same client to same server

**Pros**:
- Session persistence
- Cache efficiency

**Cons**:
- Uneven distribution
- Server failure affects clients

**Configuration**:
```nginx
upstream hermes_backend {
    ip_hash;
    server 192.168.1.10:1420;
    server 192.168.1.11:1420;
    server 192.168.1.12:1420;
}
```

---

### 4. Weighted Round Robin

**Description**: Distribute based on server capacity

**Pros**:
- Considers server capacity
- Flexible distribution

**Cons**:
- Requires capacity knowledge
- Static weights

**Configuration**:
```nginx
upstream hermes_backend {
    server 192.168.1.10:1420 weight=3;  # High capacity
    server 192.168.1.11:1420 weight=2;  # Medium capacity
    server 192.168.1.12:1420 weight=1;  # Low capacity
}
```

---

## Nginx Configuration

### Basic Setup

**/etc/nginx/nginx.conf**:
```nginx
worker_processes auto;
error_log /var/log/nginx/error.log;
pid /run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';
    
    access_log /var/log/nginx/access.log main;
    
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/json
               application/json application/javascript
               application/xml+rss application/atom+xml
               image/svg+xml;
    
    # Upstream backend
    upstream hermes_backend {
        least_conn;
        server 192.168.1.10:1420;
        server 192.168.1.11:1420;
        server 192.168.1.12:1420;
        keepalive 32;
    }
    
    # Server block
    server {
        listen 80;
        server_name hermes.example.com;
        
        # Redirect to HTTPS
        return 301 https://$server_name$request_uri;
    }
    
    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name hermes.example.com;
        
        # SSL configuration
        ssl_certificate /etc/ssl/certs/hermes.crt;
        ssl_certificate_key /etc/ssl/private/hermes.key;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;
        
        # Security headers
        add_header X-Frame-Options DENY;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        
        # Proxy to backend
        location / {
            proxy_pass http://hermes_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_cache_bypass $http_upgrade;
            
            # Timeouts
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
        
        # WebSocket support
        location /ws {
            proxy_pass http://hermes_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "Upgrade";
            proxy_set_header Host $host;
        }
        
        # Static files
        location /static {
            alias /var/www/hermes/static;
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
```

---

## HAProxy Configuration

### Basic Setup

**/etc/haproxy/haproxy.cfg**:
```
global
    log /dev/log local0
    log /dev/log local1 notice
    chroot /var/lib/haproxy
    stats socket /run/haproxy/admin.sock mode 660 level admin
    stats timeout 30s
    user haproxy
    group haproxy
    daemon
    
    # SSL configuration
    ssl-default-bind-ciphers HIGH:!aNULL:!MD5
    ssl-default-bind-options no-sslv3

defaults
    log global
    mode http
    option httplog
    option dontlognull
    timeout connect 5000
    timeout client 50000
    timeout server 50000
    errorfile 400 /etc/haproxy/errors/400.http
    errorfile 403 /etc/haproxy/errors/403.http
    errorfile 408 /etc/haproxy/errors/408.http
    errorfile 500 /etc/haproxy/errors/500.http
    errorfile 502 /etc/haproxy/errors/502.http
    errorfile 503 /etc/haproxy/errors/503.http
    errorfile 504 /etc/haproxy/errors/504.http

# Frontend configuration
frontend hermes_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/hermes.pem
    mode http
    
    # Redirect HTTP to HTTPS
    http-request redirect scheme https unless ssl_fc
    
    # Security headers
    http-response set-header X-Frame-Options DENY
    http-response set-header X-Content-Type-Options nosniff
    http-response set-header X-XSS-Protection "1; mode=block"
    
    # ACLs
    acl is_api path_beg /api
    acl is_ws path_beg /ws
    acl is_static path_beg /static
    
    # Routing
    use_backend hermes_api if is_api
    use_backend hermes_ws if is_ws
    use_backend hermes_static if is_static
    default_backend hermes_backend

# Backend configuration
backend hermes_backend
    mode http
    balance leastconn
    option httpchk GET /health
    
    # Health check
    http-check expect status 200
    
    # Servers
    server hermes1 192.168.1.10:1420 check inter 5s fall 3 rise 2
    server hermes2 192.168.1.11:1420 check inter 5s fall 3 rise 2
    server hermes3 192.168.1.12:1420 check inter 5s fall 3 rise 2

# WebSocket backend
backend hermes_ws
    mode http
    balance roundrobin
    option httpchk GET /health
    
    server hermes1 192.168.1.10:1420 check
    server hermes2 192.168.1.11:1420 check
    server hermes3 192.168.1.12:1420 check

# Static files backend
backend hermes_static
    mode http
    balance roundrobin
    
    server static1 192.168.1.10:80 check
    server static2 192.168.1.11:80 check

# Statistics
listen stats
    bind *:8080
    mode http
    stats enable
    stats uri /stats
    stats realm HAProxy\ Statistics
    stats auth admin:password
    stats refresh 10s
```

---

## Health Checks

### Application Health Endpoint

```typescript
// src/health.ts
import express from 'express'

const router = express.Router()

router.get('/health', async (req, res) => {
  try {
    // Check database
    const dbHealth = await checkDatabase()
    
    // Check services
    const servicesHealth = await checkServices()
    
    // Overall health
    const healthy = dbHealth.healthy && servicesHealth.healthy
    
    res.status(healthy ? 200 : 503).json({
      status: healthy ? 'healthy' : 'unhealthy',
      timestamp: new Date().toISOString(),
      version: process.env.APP_VERSION,
      checks: {
        database: dbHealth,
        services: servicesHealth
      }
    })
  } catch (error) {
    res.status(503).json({
      status: 'unhealthy',
      error: error.message,
      timestamp: new Date().toISOString()
    })
  }
})

async function checkDatabase() {
  try {
    await db.query('SELECT 1')
    return { healthy: true, latency_ms: 10 }
  } catch (error) {
    return { healthy: false, error: error.message }
  }
}

async function checkServices() {
  // Check external services
  return { healthy: true }
}

export default router
```

---

## Monitoring

### Load Balancer Metrics

```bash
# Nginx metrics
nginx -V

# HAProxy metrics
echo "show stat" | socat stdio /run/haproxy/admin.sock

# Custom metrics
curl http://localhost:8080/stats;csv
```

### Backend Metrics

```typescript
// Track request distribution
const metrics = {
  requests: new Map<string, number>(),
  responseTimes: new Map<string, number[]>()
}

function trackRequest(server: string, responseTime: number) {
  metrics.requests.set(server, (metrics.requests.get(server) || 0) + 1)
  
  if (!metrics.responseTimes.has(server)) {
    metrics.responseTimes.set(server, [])
  }
  metrics.responseTimes.get(server)!.push(responseTime)
}

function getMetrics() {
  return {
    requests: Object.fromEntries(metrics.requests),
    avgResponseTime: Object.fromEntries(
      Array.from(metrics.responseTimes.entries()).map(([server, times]) => [
        server,
        times.reduce((a, b) => a + b, 0) / times.length
      ])
    )
  }
}
```

---

## Scaling

### Auto-scaling Configuration

**AWS Auto Scaling Group**:
```json
{
  "AutoScalingGroupName": "hermes-asg",
  "MinSize": 2,
  "MaxSize": 10,
  "DesiredCapacity": 3,
  "HealthCheckType": "ELB",
  "HealthCheckGracePeriod": 300,
  "LaunchTemplate": {
    "LaunchTemplateId": "lt-1234567890",
    "Version": "$Latest"
  },
  "TargetGroupARNs": [
    "arn:aws:elasticloadbalancing:us-west-2:123456789012:targetgroup/hermes-tg/1234567890123456"
  ]
}
```

**Scaling Policies**:
```json
{
  "PolicyName": "ScaleUp",
  "PolicyType": "TargetTrackingScaling",
  "TargetTrackingConfiguration": {
    "PredefinedMetricSpecification": {
      "PredefinedMetricType": "ASGAverageCPUUtilization"
    },
    "TargetValue": 70.0
  }
}
```

---

## Best Practices

### 1. Use Health Checks

```nginx
upstream backend {
    server 192.168.1.10:1420 max_fails=3 fail_timeout=30s;
    server 192.168.1.11:1420 max_fails=3 fail_timeout=30s;
}
```

### 2. Enable Keep-Alive

```nginx
upstream backend {
    keepalive 32;
}

location / {
    proxy_http_version 1.1;
    proxy_set_header Connection "";
}
```

### 3. Configure Timeouts

```nginx
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;
```

### 4. Monitor Performance

```bash
# Check load balancer stats
watch -n 5 'curl http://localhost:8080/stats;csv'

# Monitor backend health
watch -n 5 'curl http://backend:1420/health'
```

---

## Resources

- [Nginx Load Balancing](https://docs.nginx.com/nginx/admin-guide/load-balancer/http-load-balancer/)
- [HAProxy Configuration](https://www.haproxy.org/download/2.4/doc/configuration.txt)
- [AWS ELB Documentation](https://docs.aws.amazon.com/elasticloadbalancing/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
