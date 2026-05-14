# ACP-UI Flutter

多Agent协作平台的Flutter实现。

## 技术栈

- **Flutter**: 3.22.0
- **Dart**: 3.4.0
- **状态管理**: Riverpod 2.4+
- **导航**: go_router 13+
- **网络**: web_socket_channel, http
- **存储**: sqflite, shared_preferences

## 项目结构

```
lib/
├── main.dart           # 应用入口
├── app.dart            # 路由配置和主布局
├── core/               # 核心业务逻辑
│   ├── agent/          # Agent管理
│   │   ├── agent_bridge.dart    # 单Agent通信
│   │   └── agent_pool.dart      # 多Agent连接池
│   ├── transport/      # 通信层
│   │   ├── acp_transport.dart   # 传输接口
│   │   └── websocket_transport.dart
│   ├── session/        # 会话管理
│   ├── permission/     # 权限检查 (Claw Code 5级)
│   ├── orchestrator/   # 多Agent编排
│   ├── self_improvement/  # 自修复/进化
│   └── gateway/        # IM Gateway (飞书/Telegram)
├── features/           # UI功能模块
│   ├── chat/           # 聊天界面
│   ├── multi_agent/    # 多Agent面板
│   ├── history/        # 历史记录
│   ├── settings/       # 设置
│   ├── evolution/      # 进化监控
│   └── hermes/         # Hermes Dashboard
├── shared/             # 共享组件
│   ├── widgets/        # Sidebar等
│   └── theme/          # AppTheme
└── data/               # 数据模型
    ├── models/         # Agent, Session
    └── repositories/   # 数据仓库
```

## 快速开始

### 环境要求

- Flutter SDK 3.22+ (已安装于 `../flutter/flutter/`)
- Visual Studio Build Tools 2019+ (Windows桌面)
- Chrome/Edge (Web)

### 运行

```bash
# 设置Flutter路径
export PATH="../flutter/flutter/bin:$PATH"

# 获取依赖
flutter pub get

# 运行Web
flutter run -d chrome

# 运行Windows桌面
flutter run -d windows

# 运行测试
flutter test
```

### 构建

```bash
# Web发布版
flutter build web --release
# 输出: build/web/ (23MB)

# Windows发布版
flutter build windows --release
# 输出: build/windows/x64/runner/Release/
```

## 功能模块

### ChatView
- Agent选择器
- 消息气泡显示
- 输入发送

### MultiAgentView
- Agent状态面板
- 任务进度追踪
- 输出流显示

### EvolutionDashboard
- 自修复日志
- 进化模式视图
- 统计数据

### HermesDashboard
- 任务图视图
- 实时日志流

## 权限系统 (Claw Code 5级)

| 模式 | 说明 |
|------|------|
| ReadOnly | 仅允许Read操作 |
| WorkspaceWrite | 工作目录内读写 |
| Allow | 自动允许+规则检查 |
| Prompt | 每次需用户确认 |
| DangerFullAccess | 无限制 (危险) |

## IM Gateway

- **FeishuGateway**: 飞书/Lark API集成
- **TelegramGateway**: Telegram Bot API

## 迁移进度

| Phase | 状态 | 内容 |
|-------|------|------|
| Phase 1 | ✅ | UI骨架 (86文件) |
| Phase 2 | ✅ | 核心通信层 (6文件) |
| Phase 3 | ✅ | 业务逻辑 (2文件) |
| Phase 4 | ✅ | IM Gateway (1文件) |
| Phase 5 | ✅ | 测试+Web构建 |

## 许可

MIT