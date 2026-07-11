# 服务网格指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## Istio 架构

```
┌─────────────────────────────────────┐
│          Service Mesh               │
├─────────────────────────────────────┤
│  Control Plane (Istiod)             │
│  - Pilot (Service Discovery)        │
│  - Citadel (Security)               │
│  - Galley (Configuration)           │
├─────────────────────────────────────┤
│  Data Plane (Envoy Proxies)         │
│  - Sidecar Proxies                  │
│  - Traffic Management               │
│  - Security (mTLS)                  │
│  - Observability                    │
└─────────────────────────────────────┘
```

---

## Istio 安装

### 基本安装

```bash
# 下载 Istio
curl -L https://istio.io/downloadIstio | sh -
cd istio-1.20.0

# 安装 Istio
istioctl install --set profile=demo

# 验证安装
istioctl verify-install
```

### 启用自动注入

```bash
# 为命名空间启用自动注入
kubectl label namespace hermes istio-injection=enabled
```

---

## 流量管理

### VirtualService

```yaml
# virtualservice.yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: hermes-game-operator
  namespace: hermes
spec:
  hosts:
  - hermes-game-operator
  http:
  - match:
    - headers:
        x-canary:
          exact: "true"
    route:
    - destination:
        host: hermes-game-operator
        subset: canary
      weight: 100
  - route:
    - destination:
        host: hermes-game-operator
        subset: stable
      weight: 95
    - destination:
        host: hermes-game-operator
        subset: canary
      weight: 5
    timeout: 30s
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: 5xx,reset,connect-failure
```

### DestinationRule

```yaml
# destinationrule.yaml
apiVersion: networking.istio.io/v1alpha3
kind: DestinationRule
metadata:
  name: hermes-game-operator
  namespace: hermes
spec:
  host: hermes-game-operator
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        h2UpgradePolicy: DEFAULT
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
    loadBalancer:
      simple: ROUND_ROBIN
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
  subsets:
  - name: stable
    labels:
      version: stable
  - name: canary
    labels:
      version: canary
```

---

## 安全

### PeerAuthentication

```yaml
# peerauthentication.yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: hermes-game-operator
  namespace: hermes
spec:
  mtls:
    mode: STRICT
```

### AuthorizationPolicy

```yaml
# authorizationpolicy.yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: hermes-game-operator
  namespace: hermes
spec:
  selector:
    matchLabels:
      app: hermes-game-operator
  rules:
  - from:
    - source:
        principals: ["cluster.local/ns/hermes/sa/hermes-service-account"]
    to:
    - operation:
        methods: ["GET", "POST", "PUT", "DELETE"]
        paths: ["/api/*"]
  - from:
    - source:
        namespaces: ["istio-system"]
    to:
    - operation:
        paths: ["/metrics", "/health", "/ready"]
```

---

## 可观测性

### Telemetry

```yaml
# telemetry.yaml
apiVersion: telemetry.istio.io/v1alpha1
kind: Telemetry
metadata:
  name: hermes-game-operator
  namespace: hermes
spec:
  metrics:
  - providers:
    - name: prometheus
    overrides:
    - match:
        metric: REQUEST_COUNT
      tagOverrides:
        destination_service:
          operation: REMOVE
  accessLogging:
  - providers:
    - name: envoy
    filter:
      expression: "response.code >= 400"
  tracing:
  - providers:
    - name: zipkin
    randomSamplingPercentage: 10.0
```

### Kiali 仪表板

```bash
# 安装 Kiali
istioctl install --set profile=demo \
  --set values.kiali.enabled=true

# 访问 Kiali
istioctl dashboard kiali
```

---

## 金丝雀发布

### 渐进式流量切换

```yaml
# canary-release.yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: hermes-game-operator-canary
spec:
  hosts:
  - hermes-game-operator
  http:
  - route:
    - destination:
        host: hermes-game-operator
        subset: stable
      weight: 90
    - destination:
        host: hermes-game-operator
        subset: canary
      weight: 10
```

### 自动金丝雀

```yaml
# istio-canary.yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: hermes-canary
spec:
  metrics:
  - name: success-rate
    interval: 1m
    successCondition: result[0] >= 0.95
    provider:
      prometheus:
        address: http://prometheus.istio-system:9090
        query: |
          sum(irate(
            istio_requests_total{
              reporter="source",
              destination_service_name="hermes-game-operator",
              response_code!="500"
            }[5m]
          )) / 
          sum(irate(
            istio_requests_total{
              reporter="source",
              destination_service_name="hermes-game-operator"
            }[5m]
          ))
```

---

## 故障注入

### 延迟注入

```yaml
# fault-injection-delay.yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: hermes-game-operator-fault
spec:
  hosts:
  - hermes-game-operator
  http:
  - fault:
      delay:
        percentage:
          value: 10
        fixedDelay: 5s
    route:
    - destination:
        host: hermes-game-operator
```

### 错误注入

```yaml
# fault-injection-abort.yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: hermes-game-operator-fault
spec:
  hosts:
  - hermes-game-operator
  http:
  - fault:
      abort:
        percentage:
          value: 10
        httpStatus: 503
    route:
    - destination:
        host: hermes-game-operator
```

---

## 监控

### Prometheus 集成

```yaml
# prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus
  namespace: istio-system
data:
  prometheus.yml: |
    scrape_configs:
    - job_name: 'istio-mesh'
      kubernetes_sd_configs:
      - role: endpoints
        namespaces:
          names:
          - istio-system
    - job_name: 'envoy-stats'
      metrics_path: /stats/prometheus
      kubernetes_sd_configs:
      - role: pod
```

### Grafana 仪表板

```bash
# 导入 Istio 仪表板
kubectl apply -f https://raw.githubusercontent.com/istio/istio/release-1.20/samples/addons/grafana.yaml

# 访问 Grafana
istioctl dashboard grafana
```

---

## 最佳实践

### 1. 渐进式发布

```yaml
# 逐步增加流量
weights: [5, 10, 25, 50, 100]
```

### 2. 健康检查

```yaml
# 配置健康检查
healthChecks:
  - path: /health
    interval: 10s
    timeout: 5s
    unhealthyThreshold: 3
```

### 3. 熔断器

```yaml
# 配置熔断器
circuitBreaker:
  maxConnections: 100
  maxPendingRequests: 100
  maxRequests: 1000
```

### 4. 重试策略

```yaml
# 配置重试
retries:
  attempts: 3
  perTryTimeout: 2s
  retryOn: 5xx,reset,connect-failure
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Service Mesh Team