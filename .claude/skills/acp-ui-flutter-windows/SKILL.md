---
name: acp-ui-flutter-windows
description: Flutter 和 Windows App 自动化完成工作流
version: 1.0.0
source: local-custom
---

# Flutter 和 Windows App 自动化工作流

全程自动执行，不停止询问。

## Phase W1: Windows 构建验证

```bash
cd D:/dingsun/acp-ui
cargo build --release
# 检查输出
ls src-tauri/target/release/acp-ui.exe
```

如果失败，分析错误并自动修复。

提交:
```bash
git commit -m "build(windows): verify release build"
```

## Phase W2: Tauri Bundle 打包

```bash
cargo tauri build --bundles msi
```

检查输出: `src-tauri/target/release/bundle/msi/`

## Phase F1: Flutter 环境修复

```bash
cd D:/dingsun/acp-ui/acp_ui_flutter
flutter clean
flutter pub get
flutter build apk --debug
```

如果 Gradle 错误:
1. 删除 `.gradle` 缓存
2. 重新运行

## Phase F2: 新模块 Dart 实现

创建以下文件:

### smart_router.dart

```dart
// lib/core/router/smart_router.dart
enum TaskComplexity { simple, medium, complex, veryComplex }
enum TaskType { frontend, backend, fullstack, testing }
enum RouteTarget { claudeHaiku, claudeSonnet, codexHaiku, team }

class SmartRouter {
  RouteDecision analyze(String input, String inputType) {
    // 调用 Tauri 命令
    return await invoke('analyze_task_complexity', {...});
  }
}
```

### circuit_breaker.dart

```dart
// lib/core/circuit/circuit_breaker.dart
enum CircuitState { closed, open, halfOpen }

class CircuitBreakerManager {
  Future<bool> isAllowed(String target) async {
    return await invoke('is_circuit_breaker_allowed', { target });
  }
  
  Future<void> reset(String target) async {
    await invoke('reset_circuit_breaker', { target });
  }
}
```

### team_dag.dart

```dart
// lib/core/dag/team_dag.dart
enum ExecutionStrategy { parallel, sequential, hybrid }

class DAGEngine {
  Future<ExecutionPlan> createPlan(...) async {
    return await invoke('create_dag_plan', {...});
  }
  
  Future<double> getProgress(String planId) async {
    return await invoke('get_dag_plan_progress', { planId });
  }
}
```

## Phase F3: WebSocket 连接验证

修改 `websocket_service.dart` 确保连接到 Tauri 后端:

```dart
class WebSocketService {
  // 连接到 Tauri WebSocket 服务
  Future<void> connect() async {
    final url = 'ws://localhost:3000'; // Tauri 默认端口
    _channel = IOWebSocketChannel.connect(url);
  }
}
```

## Phase F4: UI 数据集成

更新 `agent_teams_dashboard.dart`:

```dart
class AgentTeamsDashboard extends StatefulWidget {
  @override
  void initState() {
    super.initState();
    _loadCircuitBreakerStatus();
    _loadDAGProgress();
  }
  
  Future<void> _loadCircuitBreakerStatus() async {
    final breakers = await invoke('get_all_circuit_breakers');
    setState(() => _breakers = breakers);
  }
}
```

## Phase F5: 最终验证

```bash
# Windows
cargo test --lib
cargo tauri build --bundles msi

# Flutter
flutter test
flutter build apk --debug
```

## 最终汇报

完成后输出:
```
✅ Flutter 和 Windows App 完成

Windows:
- exe: ✅ 已生成
- msi: ✅ 已打包

Flutter:
- APK: ✅ 已构建
- 测试: ✅ 通过
- 新模块: ✅ 已添加

Git 提交: {数量}
```

## 禁止事项

- ❌ 不停止询问
- ❌ 不等待确认
- ❌ 全程自动执行