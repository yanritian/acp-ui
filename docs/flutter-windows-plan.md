# Flutter 和 Windows App 完成计划

> **创建日期**: 2026-05-25
> **状态**: 待执行

---

## 一、当前状态分析

### 1.1 Tauri Windows App

| 项目 | 状态 | 说明 |
|------|------|------|
| tauri.conf.json | ✅ 配置完整 | productName, version, identifier 已设置 |
| capabilities | ✅ 已定义 | default.json (桌面), mobile.json (移动) |
| 构建验证 | ⚠️ 未测试 | 需运行 `cargo tauri build` 生成 .exe |
| 新模块集成 | ⚠️ 未测试 | Agent Teams 模块在 Windows 端运行验证 |

### 1.2 Flutter 移动端

| 项目 | 状态 | 说明 |
|------|------|------|
| 项目结构 | ✅ 已建立 | 62 个 .dart 文件 |
| 核心模块 | ✅ 已实现 | agent_bridge, websocket, orchestrator |
| UI 组件 | ✅ 已实现 | agent_teams_dashboard, collaboration |
| 测试文件 | ✅ 已编写 | 14 个测试文件 |
| Flutter 构建 | ⚠️ 有问题 | Kotlin/Gradle 编译错误 |
| 新模块集成 | ❌ 未同步 | Agent Registry/DAG/Circuit Breaker 未添加 |

---

## 二、Windows App 计划

### Phase W1: 构建验证

```bash
cd D:/dingsun/acp-ui
cargo tauri build --bundles nsis
```

**预期输出**: `src-tauri/target/release/bundle/nsis/acp-ui_0.1.14_x64-setup.exe`

**可能问题**:
- NSIS 打包器未安装 → 使用 `msi` 替代
- 依赖缺失 → 检查 `cargo build --release`

### Phase W2: 新模块 Windows 验证

测试新 Tauri 命令在 Windows 端运行:

```typescript
// 前端调用测试
await invoke('analyze_task_complexity', { input: '测试任务', inputType: 'text' });
await invoke('get_circuit_breaker_status', { target: 'agent-1' });
await invoke('create_dag_plan', { ... });
```

### Phase W3: Windows 特定修复

| 问题 | 解决方案 |
|------|---------|
| 路径分隔符 | 已使用 `/` 代替 `\` |
| CRLF/LF | git 配置 `core.autocrlf=true` |
| PowerShell PATH | 使用完整路径调用命令 |

---

## 三、Flutter 移动端计划

### Phase F1: 构建环境修复

**问题**: Gradle/Kotlin 编译错误

**解决步骤**:
1. 清理缓存: `flutter clean`
2. 删除 Gradle 缓存: 删除 `build/` 和 `.dart_tool/`
3. 重新构建: `flutter build apk --debug`

**验证命令**:
```bash
cd D:/dingsun/acp-ui/acp_ui_flutter
flutter clean
flutter pub get
flutter build apk --debug
```

### Phase F2: 新模块同步

需要在 Flutter 端添加对应的 Rust 模块功能:

| Rust 模块 | Flutter 对应 | 状态 |
|-----------|--------------|------|
| Agent Registry | `agent_registry.dart` | ❌ 未创建 |
| Smart Router | `smart_router.dart` | ❌ 未创建 |
| Circuit Breaker | `circuit_breaker.dart` | ❌ 未创建 |
| Team DAG | `team_dag.dart` | ❌ 未创建 |
| Self-Healing | 已有 `self_healing.dart` | ✅ |

**创建优先级**:
1. `smart_router.dart` - 任务路由决策
2. `circuit_breaker.dart` - 熔断器状态管理
3. `team_dag.dart` - DAG 执行可视化

### Phase F3: WebSocket 连接验证

Flutter 通过 WebSocket 连接 Tauri 后端:

```dart
// websocket_service.dart 已实现
final wsUrl = 'ws://localhost:3000';
await _channel.connect(wsUrl);
```

**测试连接**:
1. 启动 Tauri: `cargo tauri dev`
2. 启动 Flutter: `flutter run`
3. 验证 WebSocket 消息传递

### Phase F4: UI 适配

| 视图 | 需要修复 |
|------|---------|
| AgentTeamsDashboard | 添加新模块数据展示 |
| CollaborationNetwork | DAG 可视化集成 |
| HermesDashboard | Circuit Breaker 状态显示 |

---

## 四、执行顺序

### 自动执行流程

```
Phase W1 → Phase W2 → Phase F1 → Phase F2 → Phase F3 → Phase F4
```

每个 Phase 完成后自动提交。

---

## 五、预计工作量

| 阶段 | 预计时间 | 说明 |
|------|---------|------|
| W1-W3 | 30分钟 | Windows 构建和验证 |
| F1 | 15分钟 | Flutter 环境修复 |
| F2 | 45分钟 | 新模块 Dart 实现 |
| F3-F4 | 30分钟 | 连接和 UI 验证 |

**总计**: 约 2 小时

---

## 六、风险和对策

| 风险 | 对策 |
|------|------|
| Flutter Gradle 错误持续 | 使用 Android Studio 直接构建 |
| Windows NSIS 缺失 | 使用 MSI 或直接运行 exe |
| WebSocket 跨平台连接失败 | 添加 HTTP fallback |

---

## 七、完成标准

- [ ] Windows exe 构建成功
- [ ] Flutter APK 构建成功
- [ ] 新 Tauri 命令两端都能调用
- [ ] WebSocket 连接稳定
- [ ] UI 显示新模块数据

---

*下一步: 执行 `/acp-ui-flutter-windows` 自动化工作流*