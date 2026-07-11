# ACP-UI - Hermes Game Operator

<div align="center">

![Version](https://img.shields.io/badge/version-0.1.0--alpha-blue.svg)
![Tests](https://img.shields.io/badge/tests-1278%20passed-green.svg)
![Languages](https://img.shields.io/badge/languages-13%20supported-orange.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

**多平台 Agent 系统 - 游戏开发轨道**

[English](#english) | [中文](#中文)

</div>

---

## 🌟 English

### Overview

ACP-UI is a multi-platform agent system focused on game development. It provides a unified interface for managing AI agents across Desktop, VSCode, IntelliJ IDEA, Web, and Mobile platforms.

### ✨ Key Features

- 🎮 **Hermes Game Operator** - Complete game development workflow
- 🌍 **13 Languages** - zh-CN, zh-TW, en-US, pt-BR, de-DE, es-ES, ru-RU, ja-JP, ko-KR, vi-VN, th-TH, ms-MY, fr-FR
- 🖥️ **5 Clients** - Desktop, VSCode, IDEA, Web, Mobile
- ✅ **1278 Tests** - Comprehensive test coverage
- 🔒 **Security Guards** - PathGuard + CommandGuard
- 📋 **Approval System** - 1024x720 accessibility compliant
- 🛠️ **File Tools** - read/patch/patch_preview/list

### 🚀 Quick Start

```bash
# Clone repository
git clone https://github.com/yanritian/acp-ui.git
cd acp-ui

# Install dependencies
npm install

# Start development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

### 📊 Project Status

- **Phase A-E**: ✅ 100% Complete
- **Priority Defects**: ✅ 4/4 Fixed
- **Test Coverage**: ✅ 100% Passing
- **Internationalization**: ✅ 13 Languages

### 🏗️ Architecture

```
ACP-UI/
├── src/                    # Vue 3 Frontend
│   ├── features/          # Feature modules
│   │   └── game-operator/ # Game Operator
│   ├── locales/           # i18n translations
│   ├── stores/            # Pinia stores
│   └── components/        # Shared components
├── src-tauri/             # Tauri Backend (Rust)
│   └── src/               # Rust source
├── acp_ui_flutter/        # Flutter Mobile
└── docs/                  # Documentation
```

### 📦 Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Tauri (Rust)
- **Mobile**: Flutter
- **Testing**: Vitest (1278 tests)
- **State**: Pinia
- **i18n**: vue-i18n

### 📄 Documentation

- [System Architecture](docs/system-architecture.md)
- [Implementation Plan](docs/implementation-plan-phased.md)
- [API Documentation](docs/adapter-api-documentation.md)
- [Completion Report](docs/COMPLETION-STATUS-2026-07-11.md)

---

## 🌟 中文

### 概述

ACP-UI 是一个多平台 Agent 系统，专注于游戏开发。它提供统一的界面来管理跨 Desktop、VSCode、IntelliJ IDEA、Web 和 Mobile 平台的 AI 代理。

### ✨ 核心特性

- 🎮 **Hermes Game Operator** - 完整的游戏开发工作流
- 🌍 **13种语言** - 简体中文、繁体中文、英语、葡萄牙语(巴西)、德语、西班牙语、俄语、日语、韩语、越南语、泰语、马来语、法语
- 🖥️ **5个客户端** - 桌面版、VSCode、IDEA、Web、移动端
- ✅ **1278个测试** - 全面的测试覆盖
- 🔒 **安全守卫** - PathGuard + CommandGuard
- 📋 **审批系统** - 符合1024x720可达性标准
- 🛠️ **文件工具** - 读取/补丁/预览/列表

### 🚀 快速开始

```bash
# 克隆仓库
git clone https://github.com/yanritian/acp-ui.git
cd acp-ui

# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 运行测试
npm test

# 生产构建
npm run build
```

### 📊 项目状态

- **Phase A-E**: ✅ 100% 完成
- **优先级缺陷**: ✅ 4/4 已修复
- **测试覆盖**: ✅ 100% 通过
- **国际化**: ✅ 13种语言

### 🏗️ 系统架构

```
ACP-UI/
├── src/                    # Vue 3 前端
│   ├── features/          # 功能模块
│   │   └── game-operator/ # 游戏操作员
│   ├── locales/           # 国际化翻译
│   ├── stores/            # Pinia 状态管理
│   └── components/        # 共享组件
├── src-tauri/             # Tauri 后端 (Rust)
│   └── src/               # Rust 源码
├── acp_ui_flutter/        # Flutter 移动端
└── docs/                  # 文档
```

### 📦 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Tauri (Rust)
- **移动端**: Flutter
- **测试**: Vitest (1278个测试)
- **状态管理**: Pinia
- **国际化**: vue-i18n

### 📄 文档

- [系统架构](docs/system-architecture.md)
- [实施计划](docs/implementation-plan-phased.md)
- [API文档](docs/adapter-api-documentation.md)
- [完成报告](docs/COMPLETION-STATUS-2026-07-11.md)

---

## 📝 Recent Changes (v0.1.0-alpha)

### Fixed
- ✅ DSK-001: 1024x720 approval button accessibility
- ✅ I18N-001: Game Operator internationalization
- ✅ BACKEND-001: Backend event/error standardization
- ✅ PLAT-001: VSCode/IDEA clients

### Added
- ✅ pt-BR (Portuguese - Brazil) locale
- ✅ zh-TW (Traditional Chinese) locale
- ✅ 13 languages complete support

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License.

---

<div align="center">

**Made with ❤️ by ACP-UI Team**

[GitHub](https://github.com/yanritian/acp-ui) | [Issues](https://github.com/yanritian/acp-ui/issues) | [Releases](https://github.com/yanritian/acp-ui/releases)

</div>
