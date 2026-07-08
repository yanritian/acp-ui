# Hermes Game Operator

> Godot 游戏开发 Agent 控制平面
> 版本: 0.1.0-alpha

---

## 概述

Hermes Game Operator 是一个基于 Tauri + Vue 3 + Rust 的 Godot 游戏开发 Agent 控制平面。它提供了一个安全、可控的方式来管理 AI Agent 对 Godot 项目的修改。

## 特性

### 任务管理
- ✅ 任务生命周期管理（创建、暂停、继续、停止）
- ✅ 任务重定向
- ✅ 事件流跟踪
- ✅ 任务总结

### 安全机制
- ✅ PathGuard - 路径边界验证
- ✅ CommandGuard - 命令白名单
- ✅ 注入防护
- ✅ API Key 验证

### Godot 集成
- ✅ 项目检测
- ✅ 项目分析
- ✅ 场景解析
- ✅ 玩家控制器识别

### 审批流程
- ✅ 审批队列
- ✅ 多级审批（silent/notify/approve/forbidden）
- ✅ 审批历史

---

## 快速开始

### 环境要求

- Node.js >= 18.x
- Rust >= 1.70
- Windows 10 SDK (Windows)
- Hermes CLI (可选)

### 安装

```bash
# 安装依赖
npm install

# 运行测试
npm run test

# 构建前端
npm run build

# 启动开发服务器
npm run tauri dev
```

---

## API 概览

### 任务操作

```typescript
// 启动任务
await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: 'D:/projects/my-game',
  goal: '添加二段跳功能'
})

// 暂停任务
await OperatorApi.pauseTask('task_123')

// 继续任务
await OperatorApi.resumeTask('task_123')
```

### 文件操作

```typescript
// 读取文件
const content = await OperatorApi.fileRead('task_123', 'scripts/Player.gd')

// 修改文件
await OperatorApi.filePatch('task_123', 'scripts/Player.gd', 'new content')
```

---

## 项目结构

```
src/features/game-operator/     # Vue 前端组件
src/api/operatorApi.ts          # API 层
src/types/operator.ts           # TypeScript 类型

src-tauri/src/operator/         # Rust 后端
├── types.rs                    # 协议类型
├── state_machine.rs            # 状态机
├── commands.rs                 # Tauri 命令
├── security.rs                 # 安全守卫
├── file_tools.rs               # 文件工具
├── approval_queue.rs           # 审批队列
└── hermes_cli_bridge.rs        # Hermes CLI 桥接

src-tauri/src/domains/games/godot/  # Godot Domain Pack
├── project_analyzer.rs
└── scene_parser.rs
```

---

## 文档

- [API 参考](docs/api/operator-api-reference.md)
- [配置指南](docs/setup/configuration-guide.md)
- [架构文档](docs/architecture/system-overview.md)
- [快速参考](docs/QUICK-REFERENCE.md)
- [故障排除](docs/TROUBLESHOOTING.md)

---

## 测试

```bash
# 单元测试
npm run test

# E2E 测试
npm run test:e2e

# 测试数量: 413
```

---

## 许可证

MIT License

---

## 贡献

欢迎提交 Pull Request！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。