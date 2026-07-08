# Deployment Guide

This guide covers deployment options for ACP UI and the Hermes Game Operator.

## 🚀 Quick Deployment

### Desktop Application (Tauri)

```bash
# Build for production
npm run tauri build

# Output location:
# - Windows: src-tauri/target/release/bundle/msi/
# - macOS: src-tauri/target/release/bundle/dmg/
# - Linux: src-tauri/target/release/bundle/deb/
```

### Web Application

```bash
# Build web version
npm run build:web

# Output location: dist-web/

# Preview locally
npm run preview:web
```

### Mobile Applications

#### Android

```bash
# Initialize Android project (first time only)
npm run tauri android init

# Build APK
npm run tauri android build -- --debug --apk

# Output: src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

#### iOS

```bash
# Initialize iOS project (first time only)
npm run tauri ios init

# Build via Xcode
npm run tauri ios dev
```

## 🌐 Web Deployment

### GitHub Pages

The web version is automatically deployed to GitHub Pages via GitHub Actions.

**Workflow**: `.github/workflows/deploy-web.yml`

**Deployment URL**: https://acp-ui.github.io/

**Manual Deployment**:

```bash
# Build
npm run build:web

# Deploy to gh-pages branch
npm run deploy:web
```

### Custom Domain

To use a custom domain:

1. Create `CNAME` file in `dist-web/` with your domain
2. Update DNS settings to point to GitHub Pages
3. Configure HTTPS in repository settings

### Netlify

```bash
# Build
npm run build:web

# Deploy
netlify deploy --prod --dir=dist-web
```

### Vercel

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel --prod
```

## 🖥️ Desktop Deployment

### Windows

**Installer Types**:
- MSI (recommended for enterprise)
- NSIS (lightweight)

**Build**:
```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```

**Distribution**:
- Upload to GitHub Releases
- Microsoft Store (requires certification)
- Chocolatey package

### macOS

**Build for Apple Silicon**:
```bash
npm run tauri build -- --target aarch64-apple-darwin
```

**Build for Intel**:
```bash
npm run tauri build -- --target x86_64-apple-darwin
```

**Universal Binary**:
```bash
npm run tauri build -- --target universal-apple-darwin
```

**Distribution**:
- Upload to GitHub Releases
- Mac App Store (requires Apple Developer account)
- Homebrew Cask

### Linux

**Debian/Ubuntu**:
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

**Output**: `.deb` package

**Fedora/RHEL**:
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

**Output**: `.rpm` package

**AppImage**:
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

**Output**: `.AppImage` (portable)

**Distribution**:
- GitHub Releases
- Flathub (Flatpak)
- Snap Store
- AUR (Arch Linux)

## 📱 Mobile Deployment

### Android

**Prerequisites**:
- JDK 17 (Temurin recommended)
- Android SDK platform 34
- NDK 26
- Rust Android targets

**Build**:
```bash
# Debug APK
npm run tauri android build -- --debug --apk

# Release APK
npm run tauri android build -- --release --apk

# App Bundle (for Play Store)
npm run tauri android build -- --release --aab
```

**Distribution**:
- Google Play Store
- APK sideloading
- F-Droid (requires open-source license)

### iOS

**Prerequisites**:
- macOS with Xcode 15+
- Apple Developer account
- Provisioning profiles

**Build**:
```bash
# Development build
npm run tauri ios dev

# Release build
npm run tauri ios build -- --release
```

**Distribution**:
- App Store (requires Apple Developer Program)
- TestFlight (beta testing)
- Ad-hoc distribution

## 🔄 CI/CD

### GitHub Actions

**Workflow**: `.github/workflows/build.yml`

**Triggers**:
- Push to `main`
- Pull requests
- Manual dispatch

**Jobs**:
1. **Build**: Compile for all platforms
2. **Test**: Run unit and E2E tests
3. **Release**: Create GitHub release with assets
4. **Deploy**: Deploy web version to GitHub Pages

### GitLab CI

**Configuration**: `.gitlab-ci.yml`

```yaml
stages:
  - build
  - test
  - release
  - deploy

build:
  stage: build
  script:
    - npm install
    - npm run tauri build
  artifacts:
    paths:
      - src-tauri/target/release/bundle/

test:
  stage: test
  script:
    - npm run test

release:
  stage: release
  script:
    - npm run release
  only:
    - tags

deploy-web:
  stage: deploy
  script:
    - npm run build:web
    - npm run deploy:web
  only:
    - main
```

### Azure DevOps

**Pipeline**: `azure-pipelines.yml`

```yaml
trigger:
  - main

pool:
  vmImage: 'ubuntu-latest'

steps:
  - task: NodeTool@0
    inputs:
      versionSpec: '18.x'
  
  - script: npm install
    displayName: 'Install dependencies'
  
  - script: npm run build
    displayName: 'Build'
  
  - script: npm run test
    displayName: 'Test'
  
  - task: PublishBuildArtifacts@1
    inputs:
      pathtoPublish: 'dist'
      artifactName: 'drop'
```

## 🐳 Docker

### Dockerfile

```dockerfile
FROM node:18 AS builder
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build:web

FROM nginx:alpine
COPY --from=builder /app/dist-web /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  acp-ui:
    build: .
    ports:
      - "80:80"
    environment:
      - NODE_ENV=production
```

### Build and Run

```bash
# Build image
docker build -t acp-ui .

# Run container
docker run -p 80:80 acp-ui

# Docker Compose
docker-compose up -d
```

## 📊 Monitoring

### Application Monitoring

**Tools**:
- Sentry (error tracking)
- LogRocket (session replay)
- Mixpanel (analytics)

**Setup**:

```typescript
// src/main.ts
import * as Sentry from "@sentry/vue";

Sentry.init({
  app,
  dsn: "YOUR_SENTRY_DSN",
  integrations: [
    new Sentry.BrowserTracing({
      tracingOrigins: ["localhost", "your-domain.com"],
    }),
  ],
  tracesSampleRate: 1.0,
});
```

### Server Monitoring

**Tools**:
- Prometheus (metrics)
- Grafana (dashboards)
- ELK Stack (logging)

**Metrics to Track**:
- Active sessions
- API request latency
- Error rates
- Resource usage

## 🔒 Security

### HTTPS

**Required for**:
- Web deployment
- WebSocket connections
- API endpoints

**Certificate Options**:
- Let's Encrypt (free)
- Cloudflare (free tier)
- Commercial certificates

### Content Security Policy

```html
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self' 'unsafe-inline';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  connect-src 'self' ws: wss:;
">
```

### Secure Headers

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "no-referrer-when-downgrade" always;
```

## 🚨 Troubleshooting

### Build Failures

**Issue**: Rust compilation errors

**Solution**:
```bash
# Update Rust
rustup update

# Clean build
cd src-tauri && cargo clean
```

**Issue**: TypeScript errors

**Solution**:
```bash
# Check types
npm run typecheck

# Fix type errors
npx vue-tsc --noEmit
```

### Runtime Issues

**Issue**: WebSocket connection fails

**Solution**:
- Check firewall settings
- Verify agent is running
- Check CORS configuration

**Issue**: UI not loading

**Solution**:
- Clear browser cache
- Check console for errors
- Verify build output

## 📈 Performance Optimization

### Web

- Enable gzip compression
- Use CDN for static assets
- Implement lazy loading
- Optimize images

### Desktop

- Use native modules where possible
- Minimize bundle size
- Implement code splitting
- Optimize startup time

### Mobile

- Use native APIs
- Optimize for low-end devices
- Implement offline support
- Minimize battery usage

## 📚 Additional Resources

- [Tauri Documentation](https://tauri.app/)
- [Vue 3 Documentation](https://vuejs.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Docker Documentation](https://docs.docker.com/)

---

**Last Updated**: 2026-07-08
