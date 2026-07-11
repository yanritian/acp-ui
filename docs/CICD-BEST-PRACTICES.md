# CI/CD 最佳实践

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## CI/CD 流水线架构

```
┌─────────────────────────────────────┐
│          CI/CD Pipeline             │
├─────────────────────────────────────┤
│  Source: GitHub / GitLab            │
├─────────────────────────────────────┤
│  Build: Docker / Kubernetes         │
├─────────────────────────────────────┤
│  Test: Unit / Integration / E2E     │
├─────────────────────────────────────┤
│  Deploy: Dev / Staging / Prod       │
├─────────────────────────────────────┤
│  Monitor: Metrics / Logs / Alerts   │
└─────────────────────────────────────┘
```

---

## GitHub Actions

### 基本流水线

```yaml
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'

    - name: Install dependencies
      run: npm ci

    - name: Run linter
      run: npm run lint

    - name: Run unit tests
      run: npm run test:unit

    - name: Run integration tests
      run: npm run test:integration

    - name: Upload coverage
      uses: codecov/codecov-action@v3

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    
    permissions:
      contents: read
      packages: write

    steps:
    - uses: actions/checkout@v3

    - name: Log in to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}

    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=sha

    - name: Build and push
      uses: docker/build-push-action@v4
      with:
        context: .
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy-dev:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/develop'
    environment: development

    steps:
    - uses: actions/checkout@v3

    - name: Deploy to development
      run: |
        kubectl set image deployment/hermes-game-operator \
          hermes=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
          -n hermes-dev

  deploy-staging:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: staging

    steps:
    - uses: actions/checkout@v3

    - name: Deploy to staging
      run: |
        kubectl set image deployment/hermes-game-operator \
          hermes=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
          -n hermes-staging

    - name: Run smoke tests
      run: npm run test:smoke

  deploy-production:
    needs: deploy-staging
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production

    steps:
    - uses: actions/checkout@v3

    - name: Deploy to production
      run: |
        kubectl set image deployment/hermes-game-operator \
          hermes=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
          -n hermes-production

    - name: Verify deployment
      run: |
        kubectl rollout status deployment/hermes-game-operator -n hermes-production

    - name: Notify Slack
      if: success()
      run: |
        curl -X POST https://slack.com/api/chat.postMessage \
          -H "Authorization: Bearer ${{ secrets.SLACK_TOKEN }}" \
          -d '{"channel":"#deployments","text":"✅ Production deployment successful"}'
```

---

## GitLab CI/CD

### 基本流水线

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy

variables:
  DOCKER_REGISTRY: registry.gitlab.com
  IMAGE_NAME: $CI_PROJECT_PATH

test:
  stage: test
  image: node:18
  script:
    - npm ci
    - npm run lint
    - npm run test:unit
    - npm run test:integration
  coverage: '/All files[^|]*\|[^|]*\s+([\d\.]+)/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage/cobertura-coverage.xml

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build -t $DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_SHA .
    - docker push $DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_SHA
  only:
    - main
    - develop

deploy-dev:
  stage: deploy
  image: bitnami/kubectl:latest
  script:
    - kubectl set image deployment/hermes-game-operator hermes=$DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_SHA -n hermes-dev
  environment:
    name: development
  only:
    - develop

deploy-staging:
  stage: deploy
  image: bitnami/kubectl:latest
  script:
    - kubectl set image deployment/hermes-game-operator hermes=$DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_SHA -n hermes-staging
  environment:
    name: staging
  only:
    - main

deploy-production:
  stage: deploy
  image: bitnami/kubectl:latest
  script:
    - kubectl set image deployment/hermes-game-operator hermes=$DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_SHA -n hermes-production
    - kubectl rollout status deployment/hermes-game-operator -n hermes-production
  environment:
    name: production
  when: manual
  only:
    - main
```

---

## Docker 最佳实践

### Dockerfile

```dockerfile
# Multi-stage build
FROM node:18-alpine AS builder

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Copy source
COPY . .

# Build
RUN npm run build

# Production stage
FROM node:18-alpine AS production

WORKDIR /app

# Create non-root user
RUN addgroup -g 1001 -S hermes && \
    adduser -S hermes -u 1001

# Copy built files
COPY --from=builder --chown=hermes:hermes /app/dist ./dist
COPY --from=builder --chown=hermes:hermes /app/node_modules ./node_modules
COPY --from=builder --chown=hermes:hermes /app/package.json ./

# Set environment
ENV NODE_ENV=production
ENV PORT=8080

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:8080/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Switch to non-root user
USER hermes

# Start application
CMD ["node", "dist/server.js"]
```

### .dockerignore

```
node_modules
npm-debug.log
dist
.git
.gitignore
README.md
.env
.env.*
*.md
.vscode
.idea
coverage
.nyc_output
test
tests
*.test.js
*.spec.js
```

---

## Kubernetes 部署

### 蓝绿部署

```yaml
# blue-green-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-game-operator-blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hermes-game-operator
      color: blue
  template:
    metadata:
      labels:
        app: hermes-game-operator
        color: blue
    spec:
      containers:
      - name: hermes
        image: hermes-game-operator:blue

---
apiVersion: v1
kind: Service
metadata:
  name: hermes-game-operator
spec:
  selector:
    app: hermes-game-operator
    color: blue  # 切换到 green
  ports:
  - port: 80
    targetPort: 8080
```

### 金丝雀部署

```yaml
# canary-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hermes-game-operator-canary
spec:
  replicas: 1
  selector:
    matchLabels:
      app: hermes-game-operator
      track: canary
  template:
    metadata:
      labels:
        app: hermes-game-operator
        track: canary
    spec:
      containers:
      - name: hermes
        image: hermes-game-operator:canary

---
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: hermes-game-operator
spec:
  hosts:
  - hermes-game-operator
  http:
  - route:
    - destination:
        host: hermes-game-operator
        subset: stable
      weight: 95
    - destination:
        host: hermes-game-operator
        subset: canary
      weight: 5
```

---

## 测试策略

### 测试金字塔

```
        ┌─────────┐
        │   E2E   │  10%
        ├─────────┤
        │Integration│  20%
        ├─────────┤
        │   Unit  │  70%
        └─────────┘
```

### 测试覆盖率目标

| 类型 | 目标 |
|------|------|
| 单元测试 | > 80% |
| 集成测试 | > 70% |
| E2E 测试 | > 60% |
| 总体覆盖率 | > 75% |

---

## 安全实践

### 密钥管理

```yaml
# 使用 Sealed Secrets
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: hermes-secrets
spec:
  encryptedData:
    database-url: AgBy3i4OJSWK+PiTySYZZA9rO...
```

### 镜像扫描

```yaml
# 使用 Trivy
- name: Run Trivy vulnerability scanner
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
    format: 'sarif'
    output: 'trivy-results.sarif'
```

---

## 监控和回滚

### 部署监控

```yaml
# 监控部署指标
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: hermes-game-operator
spec:
  selector:
    matchLabels:
      app: hermes-game-operator
  endpoints:
  - port: metrics
    interval: 15s
```

### 自动回滚

```bash
# 监控部署状态
kubectl rollout status deployment/hermes-game-operator

# 回滚
kubectl rollout undo deployment/hermes-game-operator

# 查看历史
kubectl rollout history deployment/hermes-game-operator
```

---

## 最佳实践清单

### 代码质量

- [ ] 代码审查
- [ ] 静态分析
- [ ] 依赖扫描
- [ ] 测试覆盖率

### 安全

- [ ] 密钥加密
- [ ] 镜像扫描
- [ ] RBAC 配置
- [ ] 网络策略

### 部署

- [ ] 蓝绿/金丝雀部署
- [ ] 自动回滚
- [ ] 健康检查
- [ ] 监控告警

### 文档

- [ ] 变更日志
- [ ] 部署文档
- [ ] 回滚文档
- [ ] 运维手册

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator DevOps Team