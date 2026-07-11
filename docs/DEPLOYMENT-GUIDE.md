# 部署指南

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [系统要求](#系统要求)
2. [本地开发部署](#本地开发部署)
3. [生产环境部署](#生产环境部署)
4. [Docker 部署](#docker-部署)
5. [云平台部署](#云平台部署)
6. [监控与维护](#监控与维护)

---

## 系统要求

### 开发环境

- **Node.js**: >= 18.0.0
- **npm**: >= 9.0.0
- **Rust**: >= 1.70.0 (用于 Tauri)
- **系统依赖**:
  - Windows: WebView2
  - macOS: Xcode Command Line Tools
  - Linux: webkit2gtk, libappindicator

### 生产环境

- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)
- **内存**: >= 4GB RAM
- **磁盘空间**: >= 500MB
- **网络**: 可选（用于远程Agent连接）

---

## 本地开发部署

### 1. 克隆仓库

```bash
git clone https://github.com/yanritian/acp-ui.git
cd acp-ui
```

### 2. 安装依赖

```bash
npm install
```

### 3. 启动开发服务器

```bash
# 桌面应用开发
npm run dev

# Web 开发
npm run dev:web
```

### 4. 运行测试

```bash
# 运行所有测试
npm test

# 运行 E2E 测试
npm run test:e2e

# 监听模式
npm run test:watch
```

### 5. 构建

```bash
# 类型检查
npm run typecheck

# 生产构建
npm run build

# 仅构建前端
npm run build:only
```

---

## 生产环境部署

### Windows

#### 方式一：MSI 安装包

1. 构建安装包：
```bash
npm run build
cd src-tauri
cargo tauri build --bundles msi
```

2. 安装包位置：`src-tauri/target/release/bundle/msi/`

3. 双击 `.msi` 文件安装

#### 方式二：NSIS 安装包

1. 构建安装包：
```bash
cargo tauri build --bundles nsis
```

2. 安装包位置：`src-tauri/target/release/bundle/nsis/`

3. 运行 `.exe` 安装程序

---

### macOS

#### Apple Silicon (M1/M2)

```bash
cargo tauri build --target aarch64-apple-darwin --bundles dmg
```

安装包位置：`src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/`

#### Intel

```bash
cargo tauri build --target x86_64-apple-darwin --bundles dmg
```

---

### Linux

#### Debian/Ubuntu (.deb)

```bash
cargo tauri build --bundles deb
```

安装：
```bash
sudo dpkg -i target/release/bundle/deb/acp-ui_*.deb
```

#### AppImage

```bash
cargo tauri build --bundles appimage
```

运行：
```bash
chmod +x target/release/bundle/appimage/acp-ui_*.AppImage
./target/release/bundle/appimage/acp-ui_*.AppImage
```

#### RPM (.rpm)

```bash
cargo tauri build --bundles rpm
```

安装：
```bash
sudo rpm -i target/release/bundle/rpm/acp-ui-*.rpm
```

---

## Docker 部署

### 1. 创建 Dockerfile

```dockerfile
FROM node:18-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build:web

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### 2. 创建 nginx.conf

```nginx
server {
    listen 80;
    server_name localhost;

    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://backend:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    location /ws {
        proxy_pass http://backend:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Host $host;
    }
}
```

### 3. 构建镜像

```bash
docker build -t acp-ui:latest .
```

### 4. 运行容器

```bash
docker run -d -p 80:80 --name acp-ui acp-ui:latest
```

### 5. Docker Compose

```yaml
version: '3.8'

services:
  frontend:
    build: .
    ports:
      - "80:80"
    depends_on:
      - backend

  backend:
    image: acp-ui-backend:latest
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    volumes:
      - ./data:/app/data
```

启动：
```bash
docker-compose up -d
```

---

## 云平台部署

### Vercel

1. 安装 Vercel CLI：
```bash
npm i -g vercel
```

2. 部署：
```bash
vercel --prod
```

### Netlify

1. 安装 Netlify CLI：
```bash
npm i -g netlify-cli
```

2. 部署：
```bash
netlify deploy --prod --dir=dist
```

### AWS S3 + CloudFront

1. 构建：
```bash
npm run build:web
```

2. 上传到 S3：
```bash
aws s3 sync dist/ s3://your-bucket-name --delete
```

3. 创建 CloudFront 分发

### GitHub Pages

1. 构建：
```bash
npm run build:web
```

2. 部署到 gh-pages 分支：
```bash
git subtree push --prefix dist origin gh-pages
```

或使用 GitHub Actions：

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Install and Build
        run: |
          npm ci
          npm run build:web
      
      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dist
```

---

## 监控与维护

### 日志

#### 桌面应用

日志位置：
- Windows: `%APPDATA%\acp-ui\logs\`
- macOS: `~/Library/Application Support/acp-ui/logs/`
- Linux: `~/.config/acp-ui/logs/`

#### Web 应用

使用浏览器开发者工具查看控制台日志。

### 性能监控

```typescript
// 添加性能监控
const observer = new PerformanceObserver((list) => {
  const entries = list.getEntries();
  entries.forEach((entry) => {
    console.log(`${entry.name}: ${entry.duration}ms`);
  });
});

observer.observe({ entryTypes: ['measure', 'navigation'] });
```

### 错误追踪

建议使用错误追踪服务：

- Sentry
- Bugsnag
- Rollbar

示例（Sentry）：

```typescript
import * as Sentry from '@sentry/browser';

Sentry.init({
  dsn: 'YOUR_SENTRY_DSN',
  environment: process.env.NODE_ENV,
  release: 'acp-ui@0.1.0-alpha'
});
```

### 更新

#### 桌面应用

使用 Tauri 自动更新：

```typescript
import { check } from '@tauri-apps/plugin-updater';

const update = await check();
if (update?.available) {
  await update.downloadAndInstall();
}
```

#### Web 应用

使用 Service Worker 缓存更新：

```typescript
// 检查更新
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js').then((reg) => {
    reg.update();
  });
}
```

---

## 故障排除

### 构建失败

**问题**: `cargo tauri build` 失败

**解决方案**:
1. 确保安装了所有系统依赖
2. 更新 Rust: `rustup update`
3. 清理缓存: `cargo clean`

### 测试失败

**问题**: 测试用例失败

**解决方案**:
1. 检查环境变量
2. 清理测试数据库
3. 运行单个测试定位问题: `npm test -- --run path/to/test`

### 性能问题

**问题**: 应用启动慢

**解决方案**:
1. 启用代码分割
2. 优化图片资源
3. 使用懒加载
4. 检查第三方库大小

### 内存泄漏

**问题**: 内存使用持续增长

**解决方案**:
1. 检查事件监听器是否正确清理
2. 使用 WeakMap/WeakSet
3. 避免循环引用
4. 使用 Chrome DevTools Memory 面板分析

---

## 安全建议

1. **HTTPS**: 生产环境必须使用 HTTPS
2. **CSP**: 配置内容安全策略
3. **CORS**: 正确配置跨域策略
4. **认证**: 使用强密码和 JWT
5. **输入验证**: 验证所有用户输入
6. **依赖更新**: 定期更新依赖修复漏洞

---

## 更多信息

- [GitHub 仓库](https://github.com/yanritian/acp-ui)
- [API 文档](API-DOCUMENTATION.md)
- [问题反馈](https://github.com/yanritian/acp-ui/issues)
