# Mobile Platform Architecture Design

## Overview

Mobile platform provides iOS/Android access to Agent Platform using Flutter with unified cross-platform UI and communication with Tauri backend via REST API.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Mobile App (Flutter)                         │
├─────────────────────────────────────────────────────────────────┤
│  Presentation Layer                                              │
│  ├── Screens                                                     │
│  │   ├── HomeScreen              (One-Shot Interface)           │
│  │   ├── AgentListScreen         (Available agents)             │
│  │   ├── CostTrackingScreen      (Cost dashboard)               │
│  │   ├── GameAssetScreen         (Game asset tools)             │
│  │   ├── MarketingScreen         (Jimeng/Kling/WPS)             │
│  │   └── SettingsScreen          (Privacy & config)             │
│  ├── Widgets                                                     │
│  │   ├── AgentCard               (Agent status card)            │
│  │   ├── CostMeter               (Budget progress)              │
│  │   ├── RoleDetectorBadge       (Detected role display)        │
│  │   └── OneShotInputField       (Main input field)             │
│  └── Navigation                                                  │
│      ├── BottomNavigationBar                                    │
│      └── DrawerMenu                                              │
│                                                                  │
│  Business Logic Layer                                           │
│  ├── Cubits (State Management)                                  │
│  │   ├── AgentCubit              (Agent operations)             │
│  │   ├── CostCubit               (Cost tracking)                │
│  │   ├── PrivacyCubit            (Privacy settings)             │
│  │   ├── OneShotCubit            (One-shot execution)           │
│  │   └── UserCubit               (User role detection)          │
│  ├── Services                                                    │
│  │   ├── AgentService            (API calls)                    │
│  │   ├── CostService             (Cost API)                     │
│  │   ├── WebSocketService        (Real-time updates)            │
│  │   ├── StorageService          (Local storage)                │
│  │   └── NotificationService     (Push notifications)           │
│  └── Models                                                      │
│  │   ├── Agent                                                   │
│  │   ├── AgentTask                                               │
│  │   ├── CostSummary                                             │
│  │   ├── UserRole                                                │
│  │   └── OneShotRequest                                          │
│                                                                  │
│  Data Layer                                                      │
│  ├── Repository                                                  │
│  │   ├── AgentRepository                                         │
│  │   ├── CostRepository                                          │
│  │   └── SettingsRepository                                      │
│  ├── Data Sources                                                │
│  │   ├── RemoteDataSource        (API calls)                    │
│  │   └── LocalDataSource         (SharedPreferences/SQLite)     │
│  └── API Client                                                  │
│  │   ├── RestClient              (HTTP requests)                │
│  │   └── WebSocketClient         (Real-time connection)         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTPS REST API / WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri Backend Server                          │
│  ├── HTTP API (same as Web Platform)                            │
│  ├── WebSocket Server                                           │
│  ├── Push Notification Service                                  │
│  └── Agent Adapter System                                       │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
acp_ui_flutter/
├── lib/
│   ├── main.dart
│   ├── app.dart
│   ├── core/
│   │   ├── constants/
│   │   │   ├── api_constants.dart
│   │   │   ├── app_colors.dart
│   │   │   └── app_strings.dart
│   │   ├── theme/
│   │   │   ├── app_theme.dart
│   │   │   └── light_theme.dart
│   │   │   └── dark_theme.dart
│   │   ├── utils/
│   │   │   ├── date_utils.dart
│   │   │   ├── format_utils.dart
│   │   │   └── validators.dart
│   │   └── router/
│   │   │   ├── app_router.dart
│   │   │   └── routes.dart
│   │
│   ├── data/
│   │   ├── models/
│   │   │   ├── agent_model.dart
│   │   │   ├── agent_task_model.dart
│   │   │   ├── cost_summary_model.dart
│   │   │   ├── user_role_model.dart
│   │   │   ├── one_shot_request_model.dart
│   │   │   ├── one_shot_response_model.dart
│   │   │   └── health_metrics_model.dart
│   │   ├── repositories/
│   │   │   ├── agent_repository.dart
│   │   │   ├── cost_repository.dart
│   │   │   └── settings_repository.dart
│   │   ├── datasources/
│   │   │   ├── remote/
│   │   │   │   ├── agent_api.dart
│   │   │   │   ├── cost_api.dart
│   │   │   │   └── oneshot_api.dart
│   │   │   └── local/
│   │   │   ├── storage_service.dart
│   │   │   └── cache_service.dart
│   │   └── services/
│   │   │   ├── api_client.dart
│   │   │   ├── websocket_client.dart
│   │   │   └── notification_service.dart
│   │
│   ├── domain/
│   │   ├── entities/
│   │   │   ├── agent.dart
│   │   │   ├── task.dart
│   │   │   ├── cost.dart
│   │   │   └── user_role.dart
│   │   ├── usecases/
│   │   │   ├── execute_task_usecase.dart
│   │   │   ├── get_agents_usecase.dart
│   │   │   ├── track_cost_usecase.dart
│   │   │   └── detect_role_usecase.dart
│   │
│   ├── presentation/
│   │   ├── cubits/
│   │   │   ├── agent/
│   │   │   │   ├── agent_cubit.dart
│   │   │   │   ├── agent_state.dart
│   │   │   │   ├── cost_cubit.dart
│   │   │   │   ├── cost_state.dart
│   │   │   │   ├── oneshot_cubit.dart
│   │   │   │   ├── oneshot_state.dart
│   │   │   │   ├── privacy_cubit.dart
│   │   │   │   ├── privacy_state.dart
│   │   │   │   ├── user_cubit.dart
│   │   │   │   └── user_state.dart
│   │   ├── screens/
│   │   │   ├── home/
│   │   │   │   ├── home_screen.dart
│   │   │   │   ├── home_view.dart
│   │   │   │   ├── widgets/
│   │   │   │   │   ├── oneshot_input.dart
│   │   │   │   │   ├── role_badge.dart
│   │   │   │   │   ├── agent_selector.dart
│   │   │   │   │   ├── cost_indicator.dart
│   │   │   │   │   └── result_display.dart
│   │   │   ├── agents/
│   │   │   │   ├── agents_screen.dart
│   │   │   │   ├── agent_detail_screen.dart
│   │   │   │   └── widgets/
│   │   │   │   │   ├── agent_card.dart
│   │   │   │   │   ├── health_indicator.dart
│   │   │   │   │   └── capability_chip.dart
│   │   │   ├── costs/
│   │   │   │   ├── costs_screen.dart
│   │   │   │   ├── widgets/
│   │   │   │   │   ├── cost_summary_card.dart
│   │   │   │   │   ├── budget_progress.dart
│   │   │   │   │   ├── cost_history_list.dart
│   │   │   │   │   └── alert_card.dart
│   │   │   ├── game/
│   │   │   │   ├── game_screen.dart
│   │   │   │   ├── asset_list_screen.dart
│   │   │   │   ├── widgets/
│   │   │   │   │   ├── asset_card.dart
│   │   │   │   │   ├── optimization_suggestion.dart
│   │   │   │   │   └── build_status.dart
│   │   │   ├── marketing/
│   │   │   │   ├── marketing_screen.dart
│   │   │   │   ├── image_generation_screen.dart
│   │   │   │   ├── video_generation_screen.dart
│   │   │   │   ├── widgets/
│   │   │   │   │   ├── generation_preview.dart
│   │   │   │   │   ├── style_selector.dart
│   │   │   │   │   └── cost_estimate.dart
│   │   │   └── settings/
│   │   │   │   ├── settings_screen.dart
│   │   │   │   ├── privacy_settings_screen.dart
│   │   │   │   ├── budget_settings_screen.dart
│   │   │   │   └── widgets/
│   │   │   │   │   ├── zone_card.dart
│   │   │   │   │   ├── budget_slider.dart
│   │   │   │   │   └── api_key_input.dart
│   │   ├── widgets/
│   │   │   ├── common/
│   │   │   │   ├── loading_indicator.dart
│   │   │   │   ├── error_display.dart
│   │   │   │   ├── empty_state.dart
│   │   │   │   ├── app_button.dart
│   │   │   │   └── app_card.dart
│   │   │   ├── navigation/
│   │   │   │   ├── bottom_nav.dart
│   │   │   │   └── app_drawer.dart
│   │
│   ├── di/
│   │   ├── injection.dart           (Dependency injection setup)
│   │
│   └── l10n/
│   │   ├── app_en.arb
│   │   ├── app_zh.arb
│   │
├── assets/
│   ├── icons/
│   ├── images/
│   ├── fonts/
│
├── test/
│   ├── unit/
│   ├── integration/
│   ├── widget/
│
└── pubspec.yaml
```

## Key Widgets Implementation

### OneShotInput Widget
```dart
class OneShotInputWidget extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Main input field
        TextField(
          maxLines: 5,
          decoration: InputDecoration(
            hintText: '输入你的需求...',
            border: OutlineInputBorder(),
          ),
          onChanged: (value) => context.read<OneShotCubit>().updateInput(value),
        ),
        
        // Options row
        Row(
          children: [
            // Agent selector
            Expanded(
              child: DropdownButtonFormField<String>(
                items: [
                  DropdownMenuItem(value: '', child: Text('自动选择')),
                  DropdownMenuItem(value: 'claude-code', child: Text('Claude Code')),
                  DropdownMenuItem(value: 'kimi', child: Text('Kimi')),
                  DropdownMenuItem(value: 'jimeng', child: Text('即梦')),
                ],
                onChanged: (value) => context.read<OneShotCubit>().selectAgent(value),
              ),
            ),
            
            // Budget input
            Expanded(
              child: TextField(
                decoration: InputDecoration(hintText: '最大成本'),
                keyboardType: TextInputType.number,
              ),
            ),
          ],
        ),
        
        // Execute button
        ElevatedButton(
          onPressed: () => context.read<OneShotCubit>().execute(),
          child: Text('执行'),
        ),
        
        // Result display
        BlocBuilder<OneShotCubit, OneShotState>(
          builder: (context, state) {
            if (state.isLoading) return CircularProgressIndicator();
            if (state.hasResult) return ResultDisplayWidget(result: state.result);
            return SizedBox.shrink();
          },
        ),
      ],
    );
  }
}
```

### AgentCard Widget
```dart
class AgentCard extends StatelessWidget {
  final Agent agent;
  
  @override
  Widget build(BuildContext context) {
    return Card(
      child: Column(
        children: [
          // Header with name and status
          ListTile(
            title: Text(agent.name),
            trailing: StatusBadge(status: agent.status),
          ),
          
          // Health metrics
          Padding(
            padding: EdgeInsets.all(8),
            child: Row(
              children: [
                HealthIndicator(score: agent.healthScore),
                SizedBox(width: 8),
                Text('成功率: ${agent.successRate.toStringAsFixed(2)}'),
              ],
            ),
          ),
          
          // Capabilities
          Wrap(
            children: agent.capabilities
              .map((cap) => CapabilityChip(capability: cap))
              .toList(),
          ),
        ],
      ),
    );
  }
}
```

### CostDashboard Widget
```dart
class CostDashboardScreen extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => CostCubit()..loadSummary(),
      child: Scaffold(
        appBar: AppBar(title: Text('成本追踪')),
        body: BlocBuilder<CostCubit, CostState>(
          builder: (context, state) {
            return Column(
              children: [
                // Monthly total
                CostSummaryCard(
                  title: '本月成本',
                  amount: state.summary.monthlyTotal,
                  budget: state.budget.monthlyLimit,
                ),
                
                // Budget progress
                BudgetProgress(
                  current: state.summary.monthlyTotal,
                  limit: state.budget.monthlyLimit,
                  thresholds: [80, 95, 100],
                ),
                
                // Agent breakdown
                Expanded(
                  child: PieChart(
                    data: state.breakdown,
                    labels: state.agentNames,
                  ),
                ),
                
                // Alerts
                if (state.alerts.isNotEmpty)
                  AlertList(alerts: state.alerts),
              ],
            );
          },
        ),
      ),
    );
  }
}
```

## State Management (Cubit Pattern)

### OneShotCubit
```dart
class OneShotCubit extends Cubit<OneShotState> {
  final OneShotRepository repository;
  final UserCubit userCubit;
  
  OneShotCubit(this.repository, this.userCubit) : super(OneShotState.initial());
  
  void updateInput(String input) {
    emit(state.copyWith(input: input));
    
    // Auto-detect user role
    final role = UserRoleDetector.detect(input);
    userCubit.updateRole(role);
  }
  
  void selectAgent(String? agentId) {
    emit(state.copyWith(preferredAgent: agentId));
  }
  
  Future<void> execute() async {
    emit(state.copyWith(isLoading: true));
    
    try {
      final request = OneShotRequest(
        input: state.input,
        preferredAgent: state.preferredAgent,
        maxCost: state.maxCost,
      );
      
      final response = await repository.execute(request);
      emit(state.copyWith(
        isLoading: false,
        result: response,
        hasResult: true,
      ));
    } catch (e) {
      emit(state.copyWith(
        isLoading: false,
        error: e.toString(),
      ));
    }
  }
}
```

### CostCubit
```dart
class CostCubit extends Cubit<CostState> {
  final CostRepository repository;
  late WebSocketClient wsClient;
  
  CostCubit(this.repository) : super(CostState.initial()) {
    _initWebSocket();
  }
  
  void _initWebSocket() {
    wsClient = WebSocketClient(Uri.parse('ws://localhost:3000/ws/events'));
    wsClient.listen((event) {
      if (event.type == 'cost_update') {
        emit(state.copyWith(summary: event.payload));
      }
      if (event.type == 'budget_alert') {
        emit(state.copyWith(alerts: [...state.alerts, event.payload]));
      }
    });
  }
  
  Future<void> loadSummary() async {
    final summary = await repository.getSummary();
    emit(state.copyWith(summary: summary));
  }
  
  Future<void> setBudget(CostBudget budget) async {
    await repository.setBudget(budget);
    emit(state.copyWith(budget: budget));
  }
}
```

## API Client

### RestClient
```dart
class RestClient {
  final String baseUrl;
  final Dio dio;
  
  RestClient(this.baseUrl) {
    dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: 5000,
      receiveTimeout: 30000,
    ));
    
    dio.interceptors.add(AuthInterceptor());
    dio.interceptors.add(LogInterceptor());
  }
  
  // Agent API
  Future<List<Agent>> getAgents() => dio.get('/api/agents');
  Future<Agent> getAgent(String id) => dio.get('/api/agents/$id');
  
  // One-Shot API
  Future<OneShotResponse> executeOneShot(OneShotRequest request) =>
    dio.post('/api/oneshot', data: request.toJson());
  
  // Cost API
  Future<CostSummary> getCostSummary() => dio.get('/api/costs/summary');
  Future<void> setBudget(CostBudget budget) =>
    dio.post('/api/costs/budget', data: budget.toJson());
}
```

## Push Notifications

### NotificationService
```dart
class NotificationService {
  final FirebaseMessaging _fcm = FirebaseMessaging.instance;
  
  Future<void> initialize() async {
    await _fcm.requestPermission();
    
    // Handle foreground messages
    FirebaseMessaging.onMessage.listen((message) {
      _showLocalNotification(message);
    });
    
    // Handle background messages
    FirebaseMessaging.onBackgroundMessage(_backgroundHandler);
  }
  
  void _showLocalNotification(RemoteMessage message) {
    if (message.data['type'] == 'budget_alert') {
      FlutterLocalNotificationsPlugin().show(
        0,
        '预算提醒',
        message.data['message'],
        NotificationDetails(
          android: AndroidNotificationDetails('alerts', '预算提醒'),
        ),
      );
    }
  }
}

// Background handler (must be top-level)
Future<void> _backgroundHandler(RemoteMessage message) async {
  // Handle background notification
}
```

## Platform-Specific Features

### iOS
- Push notifications via APNs
- Siri shortcuts for quick agent tasks
- Widget for cost monitoring
- Background app refresh for real-time updates

### Android
- Push notifications via FCM
- Quick settings tile for one-shot execution
- Widget for cost monitoring
- Background service for WebSocket connection

## Responsive Design

```dart
class ResponsiveBuilder extends StatelessWidget {
  final Widget mobile;
  final Widget tablet;
  final Widget desktop;
  
  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        if (constraints.maxWidth < 600) return mobile;
        if (constraints.maxWidth < 900) return tablet;
        return desktop;
      },
    );
  }
}

// Usage
ResponsiveBuilder(
  mobile: MobileHomeScreen(),
  tablet: TabletHomeScreen(),
  desktop: DesktopHomeScreen(),
)
```

## Testing Strategy

### Unit Tests
```dart
// test/unit/user_role_detector_test.dart
void main() {
  group('UserRoleDetector', () {
    test('detects Developer role from code keywords', () {
      final role = UserRoleDetector.detect('帮我写一个 Rust 函数');
      expect(role, UserRole.Developer);
    });
    
    test('detects Marketer role from marketing keywords', () {
      final role = UserRoleDetector.detect('写一个抖音营销文案');
      expect(role, UserRole.Marketer);
    });
    
    test('defaults to General for unknown input', () {
      final role = UserRoleDetector.detect('随便说点什么');
      expect(role, UserRole.General);
    });
  });
}
```

### Integration Tests
```dart
// test/integration/oneshot_test.dart
void main() {
  group('OneShot Flow', () {
    testWidgets('executes task and shows result', (tester) async {
      await tester.pumpWidget(MyApp());
      
      // Enter input
      await tester.enterText(find.byType(TextField), '帮我写代码');
      
      // Tap execute
      await tester.tap(find.text('执行'));
      await tester.pumpAndSettle();
      
      // Verify result
      expect(find.byType(ResultDisplayWidget), findsOneWidget);
      expect(find.text('Claude Code'), findsOneWidget);
    });
  });
}
```

## Build & Deployment

### Development
```bash
# Start Tauri backend server
cd src-tauri && cargo run --features http-server

# Run Flutter app
cd acp_ui_flutter && flutter run
```

### Production Build
```bash
# Android
flutter build apk --release
flutter build appbundle --release

# iOS
flutter build ios --release
```

### CI/CD Pipeline
```yaml
# .github/workflows/mobile-build.yml
name: Mobile Build
on: [push]

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: subosito/flutter-action@v2
      - run: flutter pub get
      - run: flutter test
      - run: flutter build apk
      - run: flutter build ios --no-codesign
```

---

**Status: Design Complete**
**Next: Implement Flutter project structure + API integration**