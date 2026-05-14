# Flutter Migration Plan for ACP-UI

## 背景

用户决定将ACP-UI从Tauri迁移到Flutter，以获得：
- 更一致的跨平台UI（Skia渲染）
- 更成熟的移动端生态
- 更流畅的动画/手势体验

## 当前状态

| 模块 | Tauri实现 | 迁移复杂度 |
|------|----------|------------|
| **前端UI** | Vue 3 + TypeScript | 🔴 高 - 需完全重写为Dart |
| **后端Rust** | Agent管理/WebSocket/DB | 🟡 中 - 可通过FFI或重新实现 |
| **ACP协议** | TypeScript acp-bridge | 🔴 高 - 需移植为Dart |
| **自修复系统** | TypeScript self-improvement | 🟡 中 - 算法可移植 |
| **多Agent编排** | TypeScript orchestrator | 🟡 中 - 架构相似 |

## 迁移策略

### Phase 1: Flutter基础架构 (1-2周)

```
目标: Flutter项目骨架 + 核心UI结构
├── flutter create acp_ui_flutter
├── 状态管理: Riverpod 或 Provider
├── 导航: go_router
├── UI结构复刻:
│   ├── Sidebar (Drawer on mobile)
│   ├── AgentSelector
│   ├── ChatView
│   ├── MultiAgentChat
│   └── 功能导航面板
└── 响应式布局适配
```

### Phase 2: 后端通信层 (1周)

```
目标: Dart与Agent进程通信
├── WebSocket客户端 (web_socket_channel)
├── JSON-RPC实现 (json_rpc_2)
├── 本地Agent进程管理:
│   ├── desktop: Process.run (dart:io)
│   ├── mobile: 不支持本地Agent (仅远程)
├── ACP协议移植:
│   ├── acp_bridge.dart
│   ├── transport接口
│   └── message handler
└── SQLite存储 (sqflite)
```

### Phase 3: 核心功能移植 (2周)

```
目标: 核心业务逻辑
├── Agent池管理 (agent_pool.dart)
├── Session管理 (session_manager.dart)
├── 多Agent编排 (orchestrator.dart)
├── 权限检查 (permission_checker.dart)
├── 自修复系统 (self_healing.dart)
├── 自进化引擎 (evolution_engine.dart)
└── 远程连接 (remote_connection.dart)
```

### Phase 4: IM Gateway (1周)

```
目标: 飞书/Telegram/Discord集成
├── 飞书SDK (官方Dart SDK或HTTP API)
├── Telegram (telegram_bot.dart via HTTP)
├── Discord (discord.dart via HTTP)
├── 消息卡片生成
└── 会话跨渠道同步
```

### Phase 5: 测试与优化 (1周)

```
目标: 验证所有功能
├── 单元测试 (flutter_test)
├── Widget测试
├── Integration测试
├── 性能优化
└── 打包发布:
    ├── Android (APK/AAB)
    ├── iOS (IPA)
    ├── Web (Flutter Web)
    └── Desktop (可选)
```

## 文件结构规划

```
acp_ui_flutter/
├── lib/
│   ├── main.dart
│   ├── app.dart
│   │
│   ├── core/
│   │   ├── agent/
│   │   │   ├── agent_pool.dart
│   │   │   ├── agent_bridge.dart
│   │   │   ├── agent_config.dart
│   │   │   └── process_manager.dart
│   │   ├── session/
│   │   │   ├── session_manager.dart
│   │   │   ├── session_store.dart
│   │   │   └ message.dart
│   │   ├── permission/
│   │   │   ├── permission_checker.dart
│   │   │   ├── permission_config.dart
│   │   │   └── permission_mode.dart
│   │   ├── self_improvement/
│   │   │   ├── self_healing.dart
│   │   │   ├── evolution_engine.dart
│   │   │   ├── adaptive_strategy.dart
│   │   │   └── healing_report.dart
│   │   ├── orchestrator/
│   │   │   ├── orchestrator.dart
│   │   │   ├── task_queue.dart
│   │   │   ├── agent_matcher.dart
│   │   │   └── multi_agent_chat.dart
│   │   ├── transport/
│   │   │   ├── acp_transport.dart
│   │   │   ├── websocket_transport.dart
│   │   │   └ stdio_transport.dart (desktop only)
│   │   │   └── http_transport.dart
│   │   └── gateway/
│   │       ├── im_gateway.dart
│   │       ├── feishu_gateway.dart
│   │       ├── telegram_gateway.dart
│   │       └── discord_gateway.dart
│   │
│   ├── features/
│   │   ├── chat/
│   │   │   ├── chat_view.dart
│   │   │   ├── chat_controller.dart
│   │   │   ├── message_widget.dart
│   │   │   └── chat_input.dart
│   │   ├── multi_agent/
│   │   │   ├── multi_agent_view.dart
│   │   │   ├── agent_status_panel.dart
│   │   │   └── task_progress.dart
│   │   ├── history/
│   │   │   ├── history_view.dart
│   │   │   ├── history_store.dart
│   │   │   └── task_detail.dart
│   │   ├── settings/
│   │   │   ├── settings_view.dart
│   │   │   ├── agent_selector.dart
│   │   │   └ gateway_config.dart
│   │   ├── evolution/
│   │   │   ├── evolution_dashboard.dart
│   │   │   ├── pattern_view.dart
│   │   │   └ healing_log.dart
│   │   └── hermes/
│   │       ├── hermes_dashboard.dart
│   │       ├── log_stream_view.dart
│   │       └ task_graph_view.dart
│   │
│   ├── shared/
│   │   ├── widgets/
│   │   │   ├── sidebar.dart
│   │   │   ├── nav_button.dart
│   │   │   ├── permission_dialog.dart
│   │   │   ├── error_banner.dart
│   │   │   └ startup_progress.dart
│   │   ├── theme/
│   │   │   ├── app_theme.dart
│   │   │   ├── colors.dart
│   │   │   └── text_styles.dart
│   │   └ utils/
│   │       ├── platform.dart
│   │       ├── json_rpc.dart
│   │       └ date_utils.dart
│   │       └ offline_cache.dart
│   │
│   └── data/
│       ├── models/
│       │   ├── agent.dart
│       │   ├── session.dart
│       │   ├── task.dart
│       │   ├── event.dart
│       │   └── log_entry.dart
│       ├── repositories/
│       │   ├── agent_repository.dart
│       │   ├── session_repository.dart
│       │   ├── history_repository.dart
│       │   └── config_repository.dart
│       ├── services/
│       │   ├── database_service.dart
│       │   ├── config_service.dart
│       │   ├── websocket_service.dart
│       │   └── notification_service.dart
│       └ stores/
│           ├── agent_store.dart
│           ├── session_store.dart
│           ├── team_runtime_store.dart
│           └ evolution_store.dart
│
├── pubspec.yaml
├── analysis_options.yaml
│
├── test/
│   ├── core/
│   │   ├── agent_pool_test.dart
│   │   ├── session_manager_test.dart
│   │   ├── permission_checker_test.dart
│   │   └ self_healing_test.dart
│   ├── features/
│   │   ├── chat_test.dart
│   │   ├── multi_agent_test.dart
│   │   └ history_test.dart
│   └ widget_test.dart
│
├── android/
├── ios/
├── web/
├── windows/ (optional)
├── macos/ (optional)
├── linux/ (optional)
│
└── docs/
    ├── MIGRATION_GUIDE.md
    ├── ARCHITECTURE.md
    └── TESTING.md
```

## 关键依赖

```yaml
# pubspec.yaml
dependencies:
  flutter:
    sdk: flutter
  
  # State Management
  flutter_riverpod: ^2.4.0
  
  # Navigation
  go_router: ^13.0.0
  
  # Network
  web_socket_channel: ^2.4.0
  http: ^1.2.0
  
  # Storage
  sqflite: ^2.3.0
  shared_preferences: ^2.2.0
  
  # UI Components
  flutter_markdown: ^0.6.0
  cupertino_icons: ^1.0.0
  
  # Utilities
  uuid: ^4.0.0
  intl: ^0.18.0
  json_annotation: ^4.8.0
  
  # Platform
  path_provider: ^2.1.0
  process_run: ^0.13.0  # desktop only

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^3.0.0
  build_runner: ^2.4.0
  json_serializable: ^6.7.0
```

## 保留的Rust后端 (可选)

如果需要保留Rust后端用于：
- 高性能Agent进程管理
- 复杂的数据库操作
- 长期运行的后台任务

可以通过 **flutter_rust_bridge** 实现FFI调用：

```dart
// 通过FFI调用Rust
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';

// Rust侧保持:
// - agent.rs (进程管理)
// - database.rs (SQLite)
// - hooks_executor.rs (Hook执行)
// - permission_checker.rs (权限检查)
```

## 时间估算

| Phase | 时间 | 依赖 |
|------|------|------|
| Phase 1 | 1-2周 | 无 |
| Phase 2 | 1周 | Phase 1完成 |
| Phase 3 | 2周 | Phase 2完成 |
| Phase 4 | 1周 | Phase 3完成 |
| Phase 5 | 1周 | Phase 4完成 |
| **总计** | **5-7周** | |

## 风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| Dart不支持本地进程管理 | 移动端无本地Agent | 仅支持远程Agent (WebSocket/HTTP) |
| Flutter Web性能较弱 | Web版体验下降 | 优先移动端，Web为备选 |
| 学习曲线 (Dart/Flutter) | 开发效率降低 | 先做Phase 1骨架，逐步迁移 |
| 原有功能遗漏 | 功能缺失 | 对照Tauri版本逐功能验证 |

## 下一步行动

1. 创建Flutter项目骨架
2. 复刻UI结构 (Sidebar + ChatView + 导航)
3. 实现ACP Bridge Dart版本
4. 逐模块迁移业务逻辑

---

**版本**: v1.0
**日期**: 2026-05-14
**状态**: 规划完成，待执行