# 容器编排指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## Kubernetes 架构

```
┌─────────────────────────────────────┐
│          Kubernetes Cluster         │
├─────────────────────────────────────┤
│  Control Plane (Master Nodes)       │
│  - API Server                       │
│  - etcd                             │
│  - Scheduler                        │
│  - Controller Manager               │
├─────────────────────────────────────┤
│  Worker Nodes                       │
│  - kubelet                          │
│  - kube-proxy                       │
│  - Container Runtime                │
├─────────────────────────────────────┤
│  Workloads                          │
│  - Deployments                      │
│  - Services                         │
│  - ConfigMaps                       │
│  - Secrets                          │
└─────────────────────────────────────┘
```

---

## Deployment 配置

### 基本部署

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-game-operator
  namespace: hermes
  labels:
    app: hermes-game-operator
    version: v0.1.0-alpha
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hermes-game-operator
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: hermes-game-operator
        version: v0.1.0-alpha
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: hermes-service-account
      containers:
      - name: hermes
        image: hermes-game-operator:v0.1.0-alpha
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: NODE_ENV
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: hermes-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: hermes-secrets
              key: redis-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: logs
          mountPath: /app/logs
      volumes:
      - name: config
        configMap:
          name: hermes-config
      - name: logs
        emptyDir: {}
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - hermes-game-operator
              topologyKey: kubernetes.io/hostname
```

---

## Service 配置

### ClusterIP Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: hermes-game-operator
  namespace: hermes
  labels:
    app: hermes-game-operator
spec:
  type: ClusterIP
  selector:
    app: hermes-game-operator
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
```

### LoadBalancer Service

```yaml
# service-lb.yaml
apiVersion: v1
kind: Service
metadata:
  name: hermes-game-operator-lb
  namespace: hermes
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
    service.beta.kubernetes.io/aws-load-balancer-scheme: "internet-facing"
spec:
  type: LoadBalancer
  selector:
    app: hermes-game-operator
  ports:
  - name: https
    port: 443
    targetPort: 8080
    protocol: TCP
```

---

## Ingress 配置

### Nginx Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hermes-game-operator
  namespace: hermes
  annotations:
    kubernetes.io/ingress.class: "nginx"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - api.your-domain.com
    secretName: hermes-tls
  rules:
  - host: api.your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: hermes-game-operator
            port:
              number: 80
```

---

## ConfigMap 和 Secret

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hermes-config
  namespace: hermes
data:
  config.json: |
    {
      "server": {
        "host": "0.0.0.0",
        "port": 8080
      },
      "database": {
        "pool": {
          "min": 5,
          "max": 20
        }
      },
      "cache": {
        "ttl": 3600
      },
      "logging": {
        "level": "info",
        "format": "json"
      }
    }
```

### Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: hermes-secrets
  namespace: hermes
type: Opaque
data:
  database-url: cG9zdGdyZXM6Ly9oZXJtZXM6cGFzc3dvcmRAZGI6NTQzMi9oZXJtZXM=
  redis-url: cmVkaXM6Ly9yZWRpczo2Mzc5
  jwt-secret: eW91ci1qd3Qtc2VjcmV0LWtleQ==
```

---

## Horizontal Pod Autoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: hermes-game-operator-hpa
  namespace: hermes
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hermes-game-operator
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

---

## Pod Disruption Budget

```yaml
# pdb.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: hermes-game-operator-pdb
  namespace: hermes
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: hermes-game-operator
```

---

## Namespace 和资源配额

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: hermes
  labels:
    name: hermes

# resource-quota.yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: hermes-quota
  namespace: hermes
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "20"
    limits.memory: 40Gi
    pods: "50"
    services: "20"
    secrets: "50"
    configmaps: "50"
```

---

## RBAC 配置

```yaml
# service-account.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: hermes-service-account
  namespace: hermes

# role.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: hermes-role
  namespace: hermes
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list"]

# role-binding.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: hermes-role-binding
  namespace: hermes
subjects:
- kind: ServiceAccount
  name: hermes-service-account
  namespace: hermes
roleRef:
  kind: Role
  name: hermes-role
  apiGroup: rbac.authorization.k8s.io
```

---

## 部署最佳实践

### 1. 使用标签

```yaml
labels:
  app: hermes-game-operator
  version: v0.1.0-alpha
  environment: production
  team: backend
```

### 2. 资源限制

```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### 3. 健康检查

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
```

### 4. 安全上下文

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Kubernetes Team