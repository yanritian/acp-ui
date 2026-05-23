/// Agent Teams Dashboard - Modern Mobile UI
/// Material Design 3 风格的多Agent协作平台主界面

import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/stores/agent_realtime/agent_realtime_store.dart';
import '../../data/stores/agent_realtime/agent_pet_store.dart';
import '../../data/models/agent_realtime/agent_realtime_types.dart';
import '../../data/models/agent_pet/agent_pet_types.dart';
import '../../data/services/websocket_service.dart';

/// 视图模式枚举
enum ViewMode {
  home,
  progress,
  collaboration,
  pet,
}

/// 主色调 - Material Design 3
const Color kPrimaryColor = Color(0xFF6750A4);
const Color kSecondaryColor = Color(0xFF7D5260);
const Color kTertiaryColor = Color(0xFF7D5700);
const Color kSurfaceColor = Color(0xFFFFFBFE);
const Color kBackgroundColor = Color(0xFFF7F2FA);
/// Material Design 3 灰色调色板
const Color kGrey100 = Color(0xFFF5F5F5);
const Color kGrey400 = Color(0xFFBDBDBD);
const Color kGrey500 = Color(0xFF9E9E9E);
const Color kGrey600 = Color(0xFF757575);
const Color kGrey700 = Color(0xFF616161);
const Color kGrey800 = Color(0xFF424242);
const Color kGrey900 = Color(0xFF212121);
const Color kTeal700 = Color(0xFF00796B);
const Color kAmber700 = Color(0xFFFFA000);

const Color kCardColor = Colors.white;

/// Agent Teams Dashboard 主页面
class AgentTeamsDashboard extends ConsumerStatefulWidget {
  const AgentTeamsDashboard({super.key});

  @override
  ConsumerState<AgentTeamsDashboard> createState() => _AgentTeamsDashboardState();
}

class _AgentTeamsDashboardState extends ConsumerState<AgentTeamsDashboard> {
  ViewMode _currentView = ViewMode.home;
  final _requestController = TextEditingController();
  final _scrollController = ScrollController();
  List<String> _executionLogs = [];
  bool _isExecuting = false;

  // 配置状态
  String _workspacePath = 'D:/dingsun/acp-ui/erp_system';
  String _wsUrl = 'ws://10.0.2.2:1421';
  bool _isExecutiveAgentInitialized = false;
  StreamSubscription<String>? _messageSubscription;

  // Agent 选择
  String _selectedAgentMode = 'hermes_native';
  String _selectedAgentName = 'Claude Code';
  String? _selectedAgentId = 'planner-001';

  // 可用Agent列表
  final List<Map<String, dynamic>> _availableAgents = [
    {'name': 'Claude Code', 'icon': '🤖', 'color': Colors.indigo},
    {'name': 'GitHub Copilot', 'icon': '✈️', 'color': Colors.blue},
    {'name': 'Gemini CLI', 'icon': '💎', 'color': Colors.teal},
    {'name': 'Qwen Code', 'icon': '🌟', 'color': Colors.amber},
    {'name': 'Hermes Agent', 'icon': '⚡', 'color': Colors.deepPurple},
  ];

  // 任务历史
  List<Map<String, dynamic>> _taskHistory = [];
  bool _isLoadingHistory = false;

  @override
  void initState() {
    super.initState();
    _connectWebSocket();
  }

  @override
  void dispose() {
    _requestController.dispose();
    _scrollController.dispose();
    _messageSubscription?.cancel();
    super.dispose();
  }

  Future<void> _connectWebSocket() async {
    final wsService = ref.read(webSocketServiceProvider);
    wsService.configureUrl(_wsUrl);
    try {
      await wsService.connect();
      ref.read(connectionStatusProvider.notifier).state = true;
      _messageSubscription = wsService.messageStream.listen(_handleWebSocketMessage);
      await _initExecutiveAgent();
      await _loadTaskHistory();
    } catch (e) {
      print('[AgentTeams] WebSocket连接失败: $e');
      ref.read(connectionStatusProvider.notifier).state = false;
    }
  }

  void _handleWebSocketMessage(String data) {
    try {
      final msg = jsonDecode(data) as Map<String, dynamic>;
      final msgType = msg['type'] as String?;
      final msgData = msg['data'] as Map<String, dynamic>?;

      switch (msgType) {
        case 'task-started':
          setState(() {
            _isExecuting = true;
            _executionLogs.add('🚀 任务开始');
          });
          break;
        case 'agent-started':
          setState(() {
            _isExecuting = true;
            _executionLogs.add('🤖 Agent启动: ${msgData?['agentName'] ?? ''}');
          });
          break;
        case 'agent-message':
          final content = msgData?['message'] as String?;
          if (content != null) {
            setState(() {
              final preview = content.length > 100 ? '${content.substring(0, 100)}...' : content;
              _executionLogs.add(preview);
            });
          }
          break;
        case 'task-completed':
          setState(() {
            _isExecuting = false;
            _executionLogs.add('✅ 任务完成');
          });
          break;
        case 'agent-error':
          setState(() {
            _isExecuting = false;
            _executionLogs.add('❌ 错误: ${msgData?['error'] ?? ''}');
          });
          break;
        default:
          if (msg['ok'] == true && msg['data'] != null) {
            final data = msg['data'] as Map<String, dynamic>;
            if (data['tasks'] != null) {
              setState(() {
                _taskHistory = (data['tasks'] as List).map((t) => t as Map<String, dynamic>).toList();
                _isLoadingHistory = false;
              });
            }
          }
      }
      _scrollToBottom();
    } catch (e) {
      print('[AgentTeams] 消息解析错误: $e');
    }
  }

  void _scrollToBottom() {
    if (_scrollController.hasClients) {
      _scrollController.animateTo(
        _scrollController.position.maxScrollExtent,
        duration: const Duration(milliseconds: 100),
        curve: Curves.easeOut,
      );
    }
  }

  Future<void> _initExecutiveAgent() async {
    final wsService = ref.read(webSocketServiceProvider);
    if (!wsService.isConnected()) return;
    await wsService.initExecutiveAgent(_workspacePath);
    _isExecutiveAgentInitialized = true;
  }

  Future<void> _executeTask() async {
    final request = _requestController.text.trim();
    if (request.isEmpty) return;

    final wsService = ref.read(webSocketServiceProvider);
    if (!wsService.isConnected()) {
      setState(() => _executionLogs.add('❌ WebSocket未连接'));
      return;
    }

    setState(() {
      _isExecuting = true;
      _executionLogs.add('📤 发送: $request');
      _requestController.clear();
    });

    if (_selectedAgentMode == 'hermes_native') {
      await wsService.executeDevelopmentTask(request);
    } else {
      await wsService.spawnAndExecuteAgent(_selectedAgentName, request, _workspacePath);
    }
  }

  Future<void> _loadTaskHistory() async {
    final wsService = ref.read(webSocketServiceProvider);
    if (!wsService.isConnected()) return;
    setState(() => _isLoadingHistory = true);
    await wsService.getTaskHistory(10, null, null);
  }

  @override
  Widget build(BuildContext context) {
    final realtimeState = ref.watch(agentRealtimeProvider);
    final petState = ref.watch(agentPetProvider);
    final isConnected = ref.watch(connectionStatusProvider);

    return Scaffold(
      backgroundColor: kBackgroundColor,
      appBar: _buildAppBar(isConnected),
      bottomNavigationBar: _buildBottomNav(),
      floatingActionButton: _buildFAB(),
      floatingActionButtonLocation: FloatingActionButtonLocation.centerDocked,
      body: _buildBody(realtimeState, petState),
    );
  }

  PreferredSizeWidget _buildAppBar(bool isConnected) {
    return AppBar(
      backgroundColor: kSurfaceColor,
      elevation: 0,
      scrolledUnderElevation: 1,
      title: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(6),
            decoration: BoxDecoration(
              color: kPrimaryColor.withOpacity(0.1),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(Icons.rocket_launch_rounded, color: kPrimaryColor, size: 20),
          ),
          const SizedBox(width: 10),
          const Text('Agent Teams', style: TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
      actions: [
        // 连接状态
        Container(
          margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
          decoration: BoxDecoration(
            color: isConnected ? Colors.green.withOpacity(0.15) : Colors.red.withOpacity(0.15),
            borderRadius: BorderRadius.circular(16),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(isConnected ? Icons.cloud_done_rounded : Icons.cloud_off_rounded,
                size: 16, color: isConnected ? Colors.green : Colors.red),
              const SizedBox(width: 4),
              Text(isConnected ? '在线' : '离线',
                style: TextStyle(fontSize: 12, color: isConnected ? Colors.green : Colors.red)),
            ],
          ),
        ),
        IconButton(
          icon: Icon(Icons.settings_rounded, color: kGrey700),
          onPressed: () => _showConfigDialog(),
        ),
      ],
    );
  }

  Widget _buildBottomNav() {
    return Container(
      decoration: BoxDecoration(
        color: kSurfaceColor,
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.08), blurRadius: 8, offset: const Offset(0, -2)),
        ],
      ),
      child: BottomNavigationBar(
        currentIndex: _currentView.index,
        onTap: (index) => setState(() => _currentView = ViewMode.values[index]),
        type: BottomNavigationBarType.fixed,
        backgroundColor: Colors.transparent,
        elevation: 0,
        selectedItemColor: kPrimaryColor,
        unselectedItemColor: kGrey500,
        selectedLabelStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500),
        unselectedLabelStyle: const TextStyle(fontSize: 12),
        items: [
          BottomNavigationBarItem(
            icon: const Icon(Icons.home_outlined),
            activeIcon: const Icon(Icons.home),
            label: '首页',
          ),
          BottomNavigationBarItem(
            icon: const Icon(Icons.trending_up_outlined),
            activeIcon: const Icon(Icons.trending_up),
            label: '进度',
          ),
          BottomNavigationBarItem(
            icon: const Icon(Icons.people_outline),
            activeIcon: const Icon(Icons.people),
            label: '协作',
          ),
          BottomNavigationBarItem(
            icon: const Icon(Icons.pets_outlined),
            activeIcon: const Icon(Icons.pets),
            label: '宠物',
          ),
        ],
      ),
    );
  }

  Widget _buildFAB() {
    return FloatingActionButton(
      onPressed: () => _showInputDialog(),
      backgroundColor: kPrimaryColor,
      elevation: 4,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: const Icon(Icons.add_rounded, color: Colors.white, size: 28),
    );
  }

  Widget _buildBody(AgentRealtimeState realtimeState, AgentPetState petState) {
    switch (_currentView) {
      case ViewMode.home:
        return _buildHomeView(realtimeState, petState);
      case ViewMode.progress:
        return _buildProgressView(realtimeState);
      case ViewMode.collaboration:
        return _buildCollaborationView(realtimeState);
      case ViewMode.pet:
        return _buildPetView(petState);
    }
  }

  /// 首页 - 任务列表 + 统计
  Widget _buildHomeView(AgentRealtimeState realtimeState, AgentPetState petState) {
    return RefreshIndicator(
      onRefresh: _loadTaskHistory,
      color: kPrimaryColor,
      child: CustomScrollView(
        physics: const AlwaysScrollableScrollPhysics(),
        slivers: [
          // 统计概览
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
              child: _buildStatsOverview(realtimeState),
            ),
          ),
          // 快捷操作区
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
              child: _buildQuickActions(),
            ),
          ),
          // Agent 列表
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
              child: _buildAgentListSection(realtimeState, petState),
            ),
          ),
          // 任务历史
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 100),
              child: _buildTaskHistorySection(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatsOverview(AgentRealtimeState state) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [kPrimaryColor, kPrimaryColor.withOpacity(0.7)],
        ),
        borderRadius: BorderRadius.circular(20),
        boxShadow: [
          BoxShadow(color: kPrimaryColor.withOpacity(0.3), blurRadius: 12, offset: const Offset(0, 4)),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Text('智能协作平台', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, color: Colors.white)),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.white.withOpacity(0.2),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text('v1.0', style: const TextStyle(fontSize: 12, color: Colors.white)),
              ),
            ],
          ),
          const SizedBox(height: 20),
          Row(
            children: [
              _buildStatItem('${state.agents.length}', '活跃Agent', Icons.smart_toy_rounded),
              const SizedBox(width: 24),
              _buildStatItem('${state.thinkingCount}', '思考中', Icons.psychology_rounded),
              const SizedBox(width: 24),
              _buildStatItem('${state.executingCount}', '执行中', Icons.bolt_rounded),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildStatItem(String value, String label, IconData icon) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, color: Colors.white.withOpacity(0.8), size: 16),
            const SizedBox(width: 6),
            Text(value, style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: Colors.white)),
          ],
        ),
        Text(label, style: TextStyle(fontSize: 12, color: Colors.white.withOpacity(0.8))),
      ],
    );
  }

  Widget _buildQuickActions() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('快捷操作', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
        const SizedBox(height: 12),
        Row(
          children: [
            _buildQuickActionCard(Icons.code_rounded, '新建任务', kPrimaryColor, () => _showInputDialog()),
            const SizedBox(width: 12),
            _buildQuickActionCard(Icons.folder_rounded, '项目目录', Colors.teal, () => _showConfigDialog()),
            const SizedBox(width: 12),
            _buildQuickActionCard(Icons.history_rounded, '历史记录', Colors.indigo, () => _loadTaskHistory()),
          ],
        ),
      ],
    );
  }

  Widget _buildQuickActionCard(IconData icon, String label, Color color, VoidCallback onTap) {
    return Expanded(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 16),
          decoration: BoxDecoration(
            color: color.withOpacity(0.1),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: color.withOpacity(0.3)),
          ),
          child: Column(
            children: [
              Icon(icon, color: color, size: 24),
              const SizedBox(height: 8),
              Text(label, style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: color)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildAgentListSection(AgentRealtimeState realtimeState, AgentPetState petState) {
    final agents = realtimeState.agents.values.toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Text('Agent 团队', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
            const Spacer(),
            TextButton.icon(
              onPressed: () {},
              icon: Icon(Icons.add_rounded, size: 18),
              label: const Text('添加'),
              style: TextButton.styleFrom(foregroundColor: kPrimaryColor),
            ),
          ],
        ),
        const SizedBox(height: 12),
        if (agents.isEmpty)
          _buildEmptyAgentsPlaceholder()
        else
          ...agents.map((agent) => _buildAgentCard(agent, petState)),
      ],
    );
  }

  Widget _buildEmptyAgentsPlaceholder() {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: BoxDecoration(
        color: kGrey100,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Center(
        child: Column(
          children: [
            Icon(Icons.smart_toy_outlined, size: 48, color: kGrey400),
            const SizedBox(height: 12),
            Text('暂无活跃Agent', style: TextStyle(color: kGrey500)),
            const SizedBox(height: 8),
            Text('点击下方按钮创建任务', style: TextStyle(fontSize: 12, color: kGrey400)),
          ],
        ),
      ),
    );
  }

  Widget _buildAgentCard(AgentRealtimeStatus agent, AgentPetState petState) {
    final isSelected = _selectedAgentId == agent.agentId;
    final pet = petState.pets.values.firstWhere(
      (p) => p.agentId == agent.agentId,
      orElse: () => _createDefaultPet(agent.agentId),
    );

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: isSelected ? kPrimaryColor : Colors.grey.withOpacity(0.15)),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.04), blurRadius: 8, offset: const Offset(0, 2)),
        ],
      ),
      child: InkWell(
        onTap: () => setState(() => _selectedAgentId = agent.agentId),
        borderRadius: BorderRadius.circular(16),
        child: Row(
          children: [
            // 头像
            Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: kPrimaryColor.withOpacity(0.1),
                shape: BoxShape.circle,
              ),
              child: Center(
                child: Text(_getAgentEmoji(agent.agentName), style: const TextStyle(fontSize: 24)),
              ),
            ),
            const SizedBox(width: 12),
            // 信息
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Text(agent.agentName, style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600)),
                      const SizedBox(width: 8),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: _getActivityColor(agent.currentActivity.type).withOpacity(0.15),
                          borderRadius: BorderRadius.circular(6),
                        ),
                        child: Text(
                          _getActivityText(agent.currentActivity.type),
                          style: TextStyle(fontSize: 11, color: _getActivityColor(agent.currentActivity.type)),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      Icon(Icons.star_rounded, size: 14, color: Colors.amber),
                      const SizedBox(width: 4),
                      Text('Lv.${pet.level}', style: const TextStyle(fontSize: 12, color: kGrey600)),
                      const SizedBox(width: 12),
                      Icon(Icons.task_alt_rounded, size: 14, color: Colors.green),
                      const SizedBox(width: 4),
                      Text('${agent.stats.tasksCompleted} 任务', style: const TextStyle(fontSize: 12, color: kGrey600)),
                    ],
                  ),
                ],
              ),
            ),
            // 进度指示
            if (agent.currentActivity.type != AgentActivityType.idle)
              SizedBox(
                width: 40,
                height: 40,
                child: CircularProgressIndicator(
                  value: agent.currentActivity.progress / 100,
                  strokeWidth: 3,
                  backgroundColor: Colors.grey.withOpacity(0.2),
                  valueColor: AlwaysStoppedAnimation(_getActivityColor(agent.currentActivity.type)),
                ),
              ),
          ],
        ),
      ),
    );
  }

  AgentPet _createDefaultPet(String agentId) {
    return AgentPet(
      petId: '${agentId}-pet',
      petName: 'Pet',
      agentId: agentId,
      emotion: PetEmotion.calm,
      happiness: 50.0,
      intimacy: 30.0,
      level: 1,
      experience: 0,
      totalTasks: 0,
      successfulTasks: 0,
      averageDuration: 0,
      currentAnimation: PetAnimation.idle,
      lastInteractionTime: DateTime.now().millisecondsSinceEpoch,
      achievements: [],
      statusText: '准备就绪',
    );
  }

  String _getAgentEmoji(String agentName) {
    final name = agentName.toLowerCase();
    if (name.contains('planner')) return '';
    if (name.contains('architect')) return '🏗️';
    if (name.contains('developer') || name.contains('coder')) return '💻';
    if (name.contains('reviewer')) return '🔍';
    if (name.contains('tdd') || name.contains('test')) return '🧪';
    if (name.contains('security')) return '🔒';
    if (name.contains('doc')) return '';
    return '🤖';
  }

  Color _getActivityColor(AgentActivityType type) {
    switch (type) {
      case AgentActivityType.thinking: return Colors.blue;
      case AgentActivityType.executing: return Colors.orange;
      case AgentActivityType.outputting: return Colors.green;
      case AgentActivityType.idle: return Colors.grey;
      case AgentActivityType.waiting: return Colors.purple;
      case AgentActivityType.error: return Colors.red;
    }
  }

  String _getActivityText(AgentActivityType type) {
    switch (type) {
      case AgentActivityType.thinking: return '思考';
      case AgentActivityType.executing: return '执行';
      case AgentActivityType.outputting: return '输出';
      case AgentActivityType.idle: return '空闲';
      case AgentActivityType.waiting: return '等待';
      case AgentActivityType.error: return '错误';
    }
  }

  Widget _buildTaskHistorySection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Text('任务历史', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
            const Spacer(),
            if (_isLoadingHistory)
              SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: kPrimaryColor))
            else
              TextButton.icon(
                onPressed: _loadTaskHistory,
                icon: Icon(Icons.refresh_rounded, size: 18),
                label: const Text('刷新'),
                style: TextButton.styleFrom(foregroundColor: kPrimaryColor),
              ),
          ],
        ),
        const SizedBox(height: 12),
        if (_taskHistory.isEmpty)
          _buildEmptyHistoryPlaceholder()
        else
          ..._taskHistory.map((task) => _buildTaskCard(task)),
      ],
    );
  }

  Widget _buildEmptyHistoryPlaceholder() {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: kGrey100,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Center(
        child: Column(
          children: [
            Icon(Icons.history_outlined, size: 40, color: kGrey400),
            const SizedBox(height: 12),
            Text('暂无任务历史', style: TextStyle(color: kGrey500)),
          ],
        ),
      ),
    );
  }

  Widget _buildTaskCard(Map<String, dynamic> task) {
    final status = task['status'] as String?;
    final name = task['name'] as String? ?? '未知任务';
    final statusColor = status == 'success' ? Colors.green : status == 'failed' ? Colors.red : Colors.orange;
    final statusIcon = status == 'success' ? Icons.check_circle_rounded : status == 'failed' ? Icons.error_rounded : Icons.pending_rounded;

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.03), blurRadius: 4, offset: const Offset(0, 1)),
        ],
      ),
      child: Row(
        children: [
          Icon(statusIcon, color: statusColor, size: 20),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(name, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500)),
                const SizedBox(height: 4),
                Text('状态: ${status ?? "进行中"}', style: TextStyle(fontSize: 12, color: kGrey500)),
              ],
            ),
          ),
          Icon(Icons.chevron_right_rounded, color: kGrey400),
        ],
      ),
    );
  }

  /// 进度视图
  Widget _buildProgressView(AgentRealtimeState realtimeState) {
    final selectedAgent = realtimeState.agents[_selectedAgentId ?? ''];

    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 执行状态
                if (_isExecuting) _buildExecutingIndicator(),
                // 执行日志
                if (_executionLogs.isNotEmpty) _buildExecutionLogs(),
                const SizedBox(height: 16),
                // Agent进度
                if (selectedAgent != null) _buildAgentProgressDetail(selectedAgent),
                // 空状态
                if (selectedAgent == null && !_isExecuting) _buildProgressEmptyState(),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildExecutingIndicator() {
    return Container(
      padding: const EdgeInsets.all(16),
      margin: const EdgeInsets.only(bottom: 16),
      decoration: BoxDecoration(
        gradient: LinearGradient(colors: [Colors.indigo.withOpacity(0.1), Colors.blue.withOpacity(0.1)]),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: Colors.indigo.withOpacity(0.3)),
      ),
      child: Row(
        children: [
          SizedBox(
            width: 24,
            height: 24,
            child: CircularProgressIndicator(strokeWidth: 2, color: kPrimaryColor),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('正在执行', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                Text('${_selectedAgentMode == 'hermes_native' ? "Hermes Native" : _selectedAgentName}',
                  style: TextStyle(fontSize: 12, color: kGrey600)),
              ],
            ),
          ),
          IconButton(
            onPressed: () => setState(() => _isExecuting = false),
            icon: Icon(Icons.stop_circle_rounded, color: Colors.red),
          ),
        ],
      ),
    );
  }

  Widget _buildExecutionLogs() {
    return Container(
      height: 200,
      margin: const EdgeInsets.only(bottom: 16),
      decoration: BoxDecoration(
        color: kGrey900,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: kGrey800,
              borderRadius: const BorderRadius.vertical(top: Radius.circular(12)),
            ),
            child: Row(
              children: [
                Icon(Icons.terminal_rounded, color: Colors.greenAccent, size: 16),
                const SizedBox(width: 8),
                const Text('执行日志', style: TextStyle(fontSize: 12, color: Colors.white)),
                const Spacer(),
                IconButton(
                  onPressed: () => setState(() => _executionLogs.clear()),
                  icon: Icon(Icons.clear_all_rounded, color: kGrey500, size: 18),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                ),
              ],
            ),
          ),
          Expanded(
            child: ListView.builder(
              controller: _scrollController,
              padding: const EdgeInsets.all(8),
              itemCount: _executionLogs.length,
              itemBuilder: (context, index) => Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Text(_executionLogs[index], style: const TextStyle(fontSize: 12, color: Colors.white)),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAgentProgressDetail(AgentRealtimeStatus agent) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(20),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.06), blurRadius: 10, offset: const Offset(0, 3)),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 标题
          Row(
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: kPrimaryColor.withOpacity(0.1),
                  shape: BoxShape.circle,
                ),
                child: Center(child: Text(_getAgentEmoji(agent.agentName), style: const TextStyle(fontSize: 20))),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(agent.agentName, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
                    Text(_getActivityText(agent.currentActivity.type),
                      style: TextStyle(fontSize: 12, color: _getActivityColor(agent.currentActivity.type))),
                  ],
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                decoration: BoxDecoration(
                  color: _getActivityColor(agent.currentActivity.type).withOpacity(0.15),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text('${agent.currentActivity.progress}%',
                  style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600,
                    color: _getActivityColor(agent.currentActivity.type))),
              ),
            ],
          ),
          const SizedBox(height: 20),
          // 进度条
          LinearProgressIndicator(
            value: agent.currentActivity.progress / 100,
            backgroundColor: Colors.grey.withOpacity(0.15),
            valueColor: AlwaysStoppedAnimation(_getActivityColor(agent.currentActivity.type)),
            minHeight: 8,
            borderRadius: BorderRadius.circular(4),
          ),
          const SizedBox(height: 20),
          // 统计
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              _buildProgressStat(Icons.task_alt_rounded, '${agent.stats.tasksCompleted}', '完成', Colors.green),
              _buildProgressStat(Icons.error_outline_rounded, '${agent.stats.tasksFailed}', '失败', Colors.red),
              _buildProgressStat(Icons.timer_outlined, '${(agent.stats.averageDuration / 1000).toStringAsFixed(1)}s', '耗时', Colors.orange),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildProgressStat(IconData icon, String value, String label, Color color) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: color.withOpacity(0.1),
            shape: BoxShape.circle,
          ),
          child: Icon(icon, color: color, size: 20),
        ),
        const SizedBox(height: 8),
        Text(value, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
        Text(label, style: TextStyle(fontSize: 12, color: kGrey600)),
      ],
    );
  }

  Widget _buildProgressEmptyState() {
    return Container(
      padding: const EdgeInsets.all(40),
      decoration: BoxDecoration(
        color: kGrey100,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Center(
        child: Column(
          children: [
            Icon(Icons.hourglass_empty_rounded, size: 64, color: kGrey400),
            const SizedBox(height: 16),
            const Text('暂无执行进度', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500)),
            const SizedBox(height: 8),
            Text('选择一个Agent或创建新任务', style: TextStyle(color: kGrey500)),
          ],
        ),
      ),
    );
  }

  /// 协作视图
  Widget _buildCollaborationView(AgentRealtimeState realtimeState) {
    final agents = realtimeState.agents.values.toList();

    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 协作网络概览
                _buildCollaborationOverview(agents),
                const SizedBox(height: 16),
                // Agent选择器
                _buildAgentSelector(agents),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildCollaborationOverview(List<AgentRealtimeStatus> agents) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [Colors.teal.withOpacity(0.15), Colors.cyan.withOpacity(0.1)],
        ),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.teal.withOpacity(0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.people_rounded, color: kTeal700, size: 24),
              const SizedBox(width: 12),
              const Text('协作网络', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
            ],
          ),
          const SizedBox(height: 20),
          // 可视化网格
          Container(
            height: 160,
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.5),
              borderRadius: BorderRadius.circular(12),
            ),
            child: _buildCollaborationGraph(agents),
          ),
        ],
      ),
    );
  }

  Widget _buildCollaborationGraph(List<AgentRealtimeStatus> agents) {
    if (agents.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.device_hub_rounded, size: 40, color: kGrey400),
            const SizedBox(height: 8),
            Text('暂无Agent连接', style: TextStyle(color: kGrey500)),
          ],
        ),
      );
    }

    // 显示Agent节点网格
    return GridView.builder(
      padding: const EdgeInsets.all(12),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        mainAxisSpacing: 12,
        crossAxisSpacing: 12,
      ),
      itemCount: agents.length,
      itemBuilder: (context, index) {
        final agent = agents[index];
        final isSelected = _selectedAgentId == agent.agentId;
        return GestureDetector(
          onTap: () => setState(() => _selectedAgentId = agent.agentId),
          child: Container(
            decoration: BoxDecoration(
              color: isSelected ? kPrimaryColor.withOpacity(0.2) : Colors.grey.withOpacity(0.1),
              shape: BoxShape.circle,
              border: Border.all(color: isSelected ? kPrimaryColor : Colors.grey.withOpacity(0.3), width: 2),
            ),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(_getAgentEmoji(agent.agentName), style: const TextStyle(fontSize: 24)),
                const SizedBox(height: 4),
                Text(agent.agentName.split(' ').first,
                  style: TextStyle(fontSize: 10, fontWeight: FontWeight.w500),
                  textAlign: TextAlign.center,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildAgentSelector(List<AgentRealtimeStatus> agents) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('选择协作Agent', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            ...agents.map((agent) => InkWell(
              onTap: () => setState(() => _selectedAgentId = agent.agentId),
              borderRadius: BorderRadius.circular(12),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: _selectedAgentId == agent.agentId ? kPrimaryColor.withOpacity(0.15) : Colors.grey.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: _selectedAgentId == agent.agentId ? kPrimaryColor : Colors.grey.withOpacity(0.2)),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(_getAgentEmoji(agent.agentName), style: const TextStyle(fontSize: 16)),
                    const SizedBox(width: 6),
                    Text(agent.agentName.split(' ').first,
                      style: TextStyle(fontSize: 12, fontWeight: FontWeight.w500)),
                  ],
                ),
              ),
            )),
            InkWell(
              onTap: () => _showInputDialog(),
              borderRadius: BorderRadius.circular(12),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: Colors.grey.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.add_rounded, size: 16, color: kGrey600),
                    const SizedBox(width: 6),
                    Text('添加', style: TextStyle(fontSize: 12, color: kGrey600)),
                  ],
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }

  /// 宠物视图
  Widget _buildPetView(AgentPetState petState) {
    final selectedPet = petState.pets.values.firstWhere(
      (p) => p.agentId == _selectedAgentId,
      orElse: () => _createDefaultPet(_selectedAgentId ?? ''),
    );

    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                // 宠物头像
                _buildPetAvatar(selectedPet),
                const SizedBox(height: 24),
                // 状态卡片
                _buildPetStatusCards(selectedPet),
                const SizedBox(height: 24),
                // 互动按钮
                _buildInteractionPanel(selectedPet),
                const SizedBox(height: 24),
                // 成就
                _buildAchievementsPanel(selectedPet),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildPetAvatar(AgentPet pet) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [_getPetGradientColor(pet.emotion).withOpacity(0.2), _getPetGradientColor(pet.emotion).withOpacity(0.1)],
        ),
        shape: BoxShape.circle,
        boxShadow: [
          BoxShadow(color: _getPetGradientColor(pet.emotion).withOpacity(0.3), blurRadius: 20, offset: const Offset(0, 8)),
        ],
      ),
      child: Column(
        children: [
          Text(_getEmotionEmoji(pet.emotion), style: const TextStyle(fontSize: 64)),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: BoxDecoration(
              color: Colors.purple.withOpacity(0.15),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text('Lv.${pet.level}', style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: Colors.purple)),
          ),
        ],
      ),
    );
  }

  Color _getPetGradientColor(PetEmotion emotion) {
    switch (emotion) {
      case PetEmotion.happy: return Colors.green;
      case PetEmotion.thinking: return Colors.blue;
      case PetEmotion.sleepy: return Colors.grey;
      case PetEmotion.confused: return Colors.orange;
      case PetEmotion.excited: return Colors.purple;
      case PetEmotion.bored: return Colors.amber;
      case PetEmotion.calm: return Colors.teal;
    }
  }

  String _getEmotionEmoji(PetEmotion emotion) {
    switch (emotion) {
      case PetEmotion.happy: return '😊';
      case PetEmotion.thinking: return '🤔';
      case PetEmotion.sleepy: return '😴';
      case PetEmotion.confused: return '😕';
      case PetEmotion.excited: return '🎉';
      case PetEmotion.bored: return '😐';
      case PetEmotion.calm: return '🙂';
    }
  }

  Widget _buildPetStatusCards(AgentPet pet) {
    return Row(
      children: [
        Expanded(child: _buildPetStatusCard(Icons.favorite_rounded, '快乐度', '${pet.happiness.toStringAsFixed(0)}%', Colors.green)),
        const SizedBox(width: 12),
        Expanded(child: _buildPetStatusCard(Icons.favorite_border_rounded, '亲密度', '${pet.intimacy.toStringAsFixed(0)}%', Colors.pink)),
        const SizedBox(width: 12),
        Expanded(child: _buildPetStatusCard(Icons.task_alt_rounded, '任务数', '${pet.totalTasks}', Colors.indigo)),
      ],
    );
  }

  Widget _buildPetStatusCard(IconData icon, String label, String value, Color color) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(16),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.05), blurRadius: 8, offset: const Offset(0, 2)),
        ],
      ),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: color.withOpacity(0.1),
              shape: BoxShape.circle,
            ),
            child: Icon(icon, color: color, size: 20),
          ),
          const SizedBox(height: 12),
          Text(value, style: const TextStyle(fontSize: 20, fontWeight: FontWeight.bold)),
          Text(label, style: TextStyle(fontSize: 12, color: kGrey600)),
        ],
      ),
    );
  }

  Widget _buildInteractionPanel(AgentPet pet) {
    final petStore = ref.read(agentPetProvider.notifier);

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(20),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.05), blurRadius: 10, offset: const Offset(0, 3)),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('互动', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
          const SizedBox(height: 16),
          Row(
            children: [
              _buildInteractionButton('👋', '抚摸', '+5快乐', Colors.green, () => petStore.interactWithPet(pet.petId, 'pet')),
              const SizedBox(width: 8),
              _buildInteractionButton('👉', '戳戳', '-2快乐', Colors.orange, () => petStore.interactWithPet(pet.petId, 'poke')),
              const SizedBox(width: 8),
              _buildInteractionButton('🍖', '喂食', '+10快乐', Colors.brown, () => petStore.interactWithPet(pet.petId, 'feed')),
              const SizedBox(width: 8),
              _buildInteractionButton('🎾', '玩耍', '+15快乐', Colors.purple, () => petStore.interactWithPet(pet.petId, 'play')),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildInteractionButton(String emoji, String label, String effect, Color color, VoidCallback onTap) {
    return Expanded(
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            color: color.withOpacity(0.1),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: color.withOpacity(0.2)),
          ),
          child: Column(
            children: [
              Text(emoji, style: const TextStyle(fontSize: 24)),
              const SizedBox(height: 4),
              Text(label, style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500)),
              Text(effect, style: TextStyle(fontSize: 10, color: kGrey500)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildAchievementsPanel(AgentPet pet) {
    final achievements = pet.achievements;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: kCardColor,
        borderRadius: BorderRadius.circular(20),
        boxShadow: [
          BoxShadow(color: Colors.black.withOpacity(0.05), blurRadius: 10, offset: const Offset(0, 3)),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.emoji_events_rounded, color: kAmber700, size: 20),
              const SizedBox(width: 8),
              const Text('成就', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
              const Spacer(),
              Text('${achievements.length} 个', style: TextStyle(fontSize: 12, color: kGrey600)),
            ],
          ),
          const SizedBox(height: 16),
          if (achievements.isEmpty)
            Center(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  children: [
                    Icon(Icons.star_outline_rounded, size: 32, color: kGrey400),
                    const SizedBox(height: 8),
                    Text('暂无成就', style: TextStyle(color: kGrey500)),
                    Text('完成任务解锁成就', style: TextStyle(fontSize: 12, color: kGrey400)),
                  ],
                ),
              ),
            )
          else
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: achievements.map((id) => _buildAchievementBadge(id)).toList(),
            ),
        ],
      ),
    );
  }

  Widget _buildAchievementBadge(String id) {
    final info = _getAchievementInfo(id);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        gradient: LinearGradient(colors: [Colors.amber.withOpacity(0.2), Colors.orange.withOpacity(0.15)]),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.amber.withOpacity(0.4)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(info['emoji']!, style: const TextStyle(fontSize: 14)),
          const SizedBox(width: 6),
          Text(info['name']!, style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }

  Map<String, String> _getAchievementInfo(String id) {
    const achievements = {
      'first_task': {'emoji': '🏆', 'name': '初出茅庐'},
      'ten_tasks': {'emoji': '📈', 'name': '小有成就'},
      'fifty_tasks': {'emoji': '⭐', 'name': '经验丰富'},
      'hundred_tasks': {'emoji': '🎖️', 'name': '大师级'},
      'speed_demon': {'emoji': '⚡', 'name': '闪电执行'},
      'perfect_run': {'emoji': '✨', 'name': '完美执行'},
    };
    return achievements[id] ?? {'emoji': '🏅', 'name': id};
  }

  void _showInputDialog() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => Container(
        padding: EdgeInsets.fromLTRB(20, 20, 20, MediaQuery.of(context).viewInsets.bottom + 20),
        decoration: const BoxDecoration(
          color: kSurfaceColor,
          borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 标题
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: kPrimaryColor.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Icon(Icons.add_task_rounded, color: kPrimaryColor, size: 20),
                ),
                const SizedBox(width: 12),
                const Text('创建新任务', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
              ],
            ),
            const SizedBox(height: 20),
            // Agent选择
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey.withOpacity(0.08),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Row(
                children: [
                  Icon(Icons.smart_toy_rounded, color: kPrimaryColor, size: 18),
                  const SizedBox(width: 8),
                  Text('执行模式:', style: const TextStyle(fontSize: 12)),
                  const Spacer(),
                  Text(_selectedAgentMode == 'hermes_native' ? 'Hermes Native' : _selectedAgentName,
                    style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500)),
                  Icon(Icons.chevron_right_rounded, color: kGrey400),
                ],
              ),
            ),
            const SizedBox(height: 16),
            // 输入框
            TextField(
              controller: _requestController,
              autofocus: true,
              maxLines: 4,
              decoration: InputDecoration(
                hintText: '描述你的需求...',
                hintStyle: TextStyle(color: kGrey400),
                filled: true,
                fillColor: Colors.grey.withOpacity(0.06),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(16),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.all(16),
              ),
            ),
            const SizedBox(height: 16),
            // 发送按钮
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                onPressed: () {
                  Navigator.pop(context);
                  _executeTask();
                },
                style: ElevatedButton.styleFrom(
                  backgroundColor: kPrimaryColor,
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(vertical: 14),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(Icons.send_rounded, size: 18),
                    const SizedBox(width: 8),
                    const Text('发送任务', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _showConfigDialog() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => DraggableScrollableSheet(
        initialChildSize: 0.6,
        maxChildSize: 0.9,
        minChildSize: 0.4,
        builder: (context, scrollController) => Container(
          decoration: const BoxDecoration(
            color: kSurfaceColor,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          child: Column(
            children: [
              // 头部
              Container(
                padding: const EdgeInsets.fromLTRB(20, 20, 20, 16),
                child: Row(
                  children: [
                    Container(
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: Colors.indigo.withOpacity(0.1),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Icon(Icons.settings_rounded, color: Colors.indigo, size: 20),
                    ),
                    const SizedBox(width: 12),
                    const Text('配置设置', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
                    const Spacer(),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(Icons.close_rounded, color: kGrey600),
                    ),
                  ],
                ),
              ),
              // 内容
              Expanded(
                child: ListView(
                  controller: scrollController,
                  padding: const EdgeInsets.all(20),
                  children: [
                    // 执行模式
                    const Text('执行模式', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                    const SizedBox(height: 12),
                    _buildModeOption('Hermes Native', '阿里云 Coding Plan', Icons.rocket_launch_rounded, Colors.indigo,
                      _selectedAgentMode == 'hermes_native', () {
                        setState(() => _selectedAgentMode = 'hermes_native');
                        Navigator.pop(context);
                      }),
                    const SizedBox(height: 8),
                    ..._availableAgents.map((agent) => Padding(
                      padding: const EdgeInsets.only(bottom: 8),
                      child: _buildModeOption(
                        agent['name'] as String,
                        '外部 Agent',
                        Icons.smart_toy_rounded,
                        agent['color'] as Color,
                        _selectedAgentName == agent['name'] && _selectedAgentMode == 'external_agent',
                        () {
                          setState(() {
                            _selectedAgentMode = 'external_agent';
                            _selectedAgentName = agent['name'] as String;
                          });
                          Navigator.pop(context);
                        },
                      ),
                    )),
                    const SizedBox(height: 24),
                    // WebSocket URL
                    const Text('WebSocket服务器', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                    const SizedBox(height: 8),
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.grey.withOpacity(0.08),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(_wsUrl, style: const TextStyle(fontSize: 13)),
                    ),
                    const SizedBox(height: 24),
                    // 工作目录
                    const Text('工作目录', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
                    const SizedBox(height: 8),
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.grey.withOpacity(0.08),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(_workspacePath, style: const TextStyle(fontSize: 13)),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildModeOption(String title, String subtitle, IconData icon, Color color, bool isSelected, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: isSelected ? color.withOpacity(0.12) : Colors.grey.withOpacity(0.06),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: isSelected ? color : Colors.grey.withOpacity(0.15), width: isSelected ? 2 : 1),
        ),
        child: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: color.withOpacity(0.15),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(icon, color: color, size: 20),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: TextStyle(fontSize: 14, fontWeight: FontWeight.w500, color: isSelected ? color : Colors.black)),
                  Text(subtitle, style: const TextStyle(fontSize: 11, color: Colors.grey)),
                ],
              ),
            ),
            if (isSelected) Icon(Icons.check_circle_rounded, color: color, size: 20),
          ],
        ),
      ),
    );
  }
}