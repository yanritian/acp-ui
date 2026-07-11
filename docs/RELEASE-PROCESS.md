# 发布流程文档

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 📋 目录

1. [发布类型](#发布类型)
2. [发布前准备](#发布前准备)
3. [发布流程](#发布流程)
4. [发布后检查](#发布后检查)
5. [紧急修复](#紧急修复)
6. [自动化发布](#自动化发布)

---

## 发布类型

### 语义化版本

```
主版本.次版本.修订号-预发布标识
MAJOR.MINOR.PATCH-prerelease

示例:
- 0.1.0-alpha     (预发布)
- 0.1.0           (正式发布)
- 0.1.1           (修订版本)
- 0.2.0           (功能版本)
- 1.0.0           (主版本)
```

### 版本号规则

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向下兼容的功能新增
- **PATCH**: 向下兼容的问题修正
- **prerelease**: alpha, beta, rc 等

---

## 发布前准备

### 1. 代码冻结

```bash
# 创建发布分支
git checkout -b release/v0.1.1

# 只允许修复 bug，不添加新功能
```

### 2. 运行完整测试

```bash
# 单元测试
npm test

# E2E 测试
npm run test:e2e

# 类型检查
npm run typecheck

# 构建检查
npm run build
```

### 3. 更新文档

- [ ] CHANGELOG.md
- [ ] README.md (如需要)
- [ ] API 文档
- [ ] 迁移指南 (如需要)

### 4. 更新版本号

```bash
# 使用 npm 自动更新
npm version patch  # 0.1.0 -> 0.1.1
npm version minor  # 0.1.1 -> 0.2.0
npm version major  # 0.2.0 -> 1.0.0

# 或手动更新
# 修改 package.json 中的 version
# 修改 src-tauri/Cargo.toml 中的 version
```

### 5. 生成 CHANGELOG

```bash
# 使用 conventional-changelog
npx conventional-changelog -p angular -i CHANGELOG.md -s

# 或手动编辑 CHANGELOG.md
```

---

## 发布流程

### 1. 提交更改

```bash
git add .
git commit -m "chore: release v0.1.1"
```

### 2. 创建标签

```bash
# 创建带注释的标签
git tag -a v0.1.1 -m "Release v0.1.1

## Changes
- Fix approval button accessibility
- Add pt-BR locale support
- Update documentation

## Known Issues
- None

## Breaking Changes
- None
"
```

### 3. 推送代码和标签

```bash
git push origin release/v0.1.1
git push origin v0.1.1
```

### 4. 创建 Pull Request

```bash
# 使用 GitHub CLI
gh pr create \
  --base main \
  --head release/v0.1.1 \
  --title "Release v0.1.1" \
  --body "## Changes
- Fix approval button accessibility
- Add pt-BR locale support
- Update documentation

## Checklist
- [x] All tests passing
- [x] Documentation updated
- [x] CHANGELOG updated
- [x] Version bumped
"
```

### 5. 合并到主分支

```bash
# 等待审查和检查通过
# 合并 PR (使用 Squash 合并)
gh pr merge --squash --delete-branch
```

### 6. 创建 GitHub Release

```bash
gh release create v0.1.1 \
  --title "v0.1.1" \
  --notes "## What's Changed

### Bug Fixes
- Fix approval button accessibility at 1024x720
- Fix memory leak in event stream

### Features
- Add pt-BR (Portuguese - Brazil) locale support

### Documentation
- Update API documentation
- Add maintenance guide

**Full Changelog**: https://github.com/yanritian/acp-ui/compare/v0.1.0...v0.1.1"
```

### 7. 构建发布产物

```bash
# 桌面应用
cd src-tauri
cargo tauri build

# Web 应用
cd ..
npm run build:web

# 移动应用
cd acp_ui_flutter
flutter build apk
flutter build ios
```

### 8. 上传产物到 Release

```bash
# 使用 GitHub CLI
gh release upload v0.1.1 \
  src-tauri/target/release/bundle/msi/*.msi \
  src-tauri/target/release/bundle/nsis/*.exe \
  src-tauri/target/release/bundle/dmg/*.dmg \
  src-tauri/target/release/bundle/deb/*.deb \
  src-tauri/target/release/bundle/appimage/*.AppImage \
  dist/*.tar.gz
```

---

## 发布后检查

### 1. 验证 Release

- [ ] GitHub Release 创建成功
- [ ] 所有产物上传成功
- [ ] Release notes 正确显示
- [ ] 下载链接有效

### 2. 测试安装

```bash
# Windows
下载 .msi → 安装 → 启动 → 检查功能

# macOS
下载 .dmg → 安装 → 启动 → 检查功能

# Linux
下载 .deb → 安装 → 启动 → 检查功能

# Web
访问 https://acp-ui.github.io/ → 检查功能
```

### 3. 通知用户

- [ ] 更新网站
- [ ] 发送邮件通知
- [ ] 发布社交媒体
- [ ] 更新文档站点

### 4. 监控

```bash
# 监控错误率
# 检查是否有新的错误报告

# 监控性能
# 检查是否有性能下降

# 监控用户反馈
# 检查是否有用户报告问题
```

---

## 紧急修复

### Hotfix 流程

```bash
# 1. 从主版本标签创建分支
git checkout -b hotfix/v0.1.2 v0.1.1

# 2. 修复问题
# ... 修复代码 ...

# 3. 测试
npm test
npm run build

# 4. 提交
git add .
git commit -m "fix: critical bug in approval system"

# 5. 更新版本号
npm version patch

# 6. 创建标签
git tag -a v0.1.2 -m "Hotfix v0.1.2"

# 7. 推送
git push origin hotfix/v0.1.2
git push origin v0.1.2

# 8. 创建 PR 到 main
gh pr create --base main --head hotfix/v0.1.2

# 9. 创建 Release
gh release create v0.1.2 --title "Hotfix v0.1.2" --notes "..."
```

---

## 自动化发布

### GitHub Actions 工作流

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
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
          npm run build
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*.tar.gz
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 自动化工具

- **semantic-release**: 自动化版本管理
- **release-it**: 自动化发布流程
- **standard-version**: 标准化版本管理

---

## 发布检查清单

### 发布前

- [ ] 所有测试通过
- [ ] 类型检查通过
- [ ] 构建成功
- [ ] 文档更新
- [ ] CHANGELOG 更新
- [ ] 版本号更新
- [ ] 代码审查完成
- [ ] 安全审计通过

### 发布中

- [ ] 创建发布分支
- [ ] 提交更改
- [ ] 创建标签
- [ ] 推送代码
- [ ] 创建 PR
- [ ] 合并 PR
- [ ] 创建 Release
- [ ] 上传产物

### 发布后

- [ ] 验证 Release
- [ ] 测试安装
- [ ] 通知用户
- [ ] 监控错误
- [ ] 监控性能
- [ ] 收集反馈

---

## 更多信息

- [维护指南](MAINTENANCE-GUIDE.md)
- [最佳实践](BEST-PRACTICES.md)
- [贡献指南](../CONTRIBUTING.md)
- [GitHub 仓库](https://github.com/yanritian/acp-ui)

---

<div align="center">

**规范化发布流程，保证质量！**

[查看维护指南 →](MAINTENANCE-GUIDE.md)

</div>
