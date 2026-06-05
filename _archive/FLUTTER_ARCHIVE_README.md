# Flutter Legacy Archive

> 日期: 2026-06-05
> 决策: 选择 Tauri Mobile 作为移动端方案，暂停 Flutter 开发

## 原因

1. **代码复用**: Tauri Mobile 与桌面端共享 Rust 后端 + Vue 前端
2. **功能对齐**: 自动对齐（共用 src/），无需手动同步
3. **维护成本**: 1人可维护三端，Flutter需额外开发者
4. **当前状态**: Flutter项目功能滞后，Tauri已有Android构建产物

## 迁移步骤

```bash
# 1. 移动Flutter项目到归档目录
mv acp_ui_flutter _archive/flutter_legacy

# 2. 清理Flutter相关配置（如有）
# - 移除 CI 中的 Flutter 构建步骤
# - 更新文档中的移动端说明

# 3. 配置 Tauri Mobile CI
# - 已有 src-tauri/gen/android/ 产物
# - iOS 构建待配置
```

## 恢复方案

如果未来需要恢复 Flutter:
```bash
mv _archive/flutter_legacy acp_ui_flutter
flutter pub get
```

## Tauri Mobile 前进路线

1. 完成 Tauri Android 构建配置
2. 配置 Tauri iOS 构建（需要 Mac）
3. 统一三端 CI 流程