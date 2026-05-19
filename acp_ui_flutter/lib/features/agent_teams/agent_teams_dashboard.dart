/// Agent Teams Dashboard
/// 多Agent协作平台主界面

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/stores/agent_realtime/agent_realtime_store.dart';
import '../../data/stores/agent_realtime/agent_pet_store.dart';
import '../../data/models/agent_realtime/agent_realtime_types.dart';
import '../../data/models/agent_pet/agent_pet_types.dart';
import '../widgets/agent_progress_panel.dart';
import '../widgets/agent_pet_avatar.dart';
import '../widgets/collaboration_network.dart';

/// 视图模式枚举
enum ViewMode {
  all,
  progress,
  collaboration,
  pet,
}

/// Agent Teams Dashboard 主页面
class AgentTeamsDashboard extends ConsumerStatefulWidget {
  const AgentTeamsDashboard({super.key});

  @override
  ConsumerState<AgentTeamsDashboard> createState() => _AgentTeamsDashboardState();
}

class _AgentTeamsDashboardState extends ConsumerState<AgentTeamsDashboard> {
  ViewMode _currentView = ViewMode.all;
  String? _selectedAgentId;

  @override
  void initState() {
    super.initState();
    _selectedAgentId = 'planner-001';
  }

  @override
  Widget build(BuildContext context) {
    final realtimeState = ref.watch(agentRealtimeProvider);
    final petState = ref.watch(agentPetProvider);

    return Scaffold(
      appBar: _buildAppBar(realtimeState),
      body: Row(
        children: [
          // 左侧：Agent团队列表
          _buildAgentTeamPanel(realtimeState, petState),
          // 右侧：主内容区域
          Expanded(
            child: _buildMainContent(realtimeState, petState),
          ),
        ],
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(AgentRealtimeState realtimeState) {
    return AppBar(
      title: const Row(
        children: [
          Icon(Icons.rocket_launch, size: 28),
          SizedBox(width: 8),
          Text('Agent Teams Platform'),
        ],
      ),
      actions: [
        // 同步状态指示
        _buildSyncIndicator(realtimeState.isConnected),
        // 语言切换按钮
        IconButton(
          icon: const Icon(Icons.language),
          onPressed: () => _showLanguageDialog(),
          tooltip: '切换语言',
        ),
        // 帮助按钮
        IconButton(
          icon: const Icon(Icons.help_outline),
          onPressed: () => _showHelpDialog(),
          tooltip: '帮助',
        ),
      ],
      bottom: PreferredSize(
        preferredSize: const Size.fromHeight(60),
        child: _buildStatsBar(realtimeState),
      ),
    );
  }

  Widget _buildSyncIndicator(bool isConnected) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 8),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: isConnected ? Colors.green.withOpacity(0.2) : Colors.red.withOpacity(0.2),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            isConnected ? Icons.cloud_done : Icons.cloud_off,
            size: 16,
            color: isConnected ? Colors.green : Colors.red,
          ),
          const SizedBox(width: 4),
          Text(
            isConnected ? '已连接' : '断开',
            style: TextStyle(
              color: isConnected ? Colors.green : Colors.red,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatsBar(AgentRealtimeState state) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          _buildStatCard('代理', '${state.agents.length}', Icons.smart_toy),
          const SizedBox(width: 16),
          _buildStatCard('思考', '${state.thinkingCount}', Icons.psychology),
          const SizedBox(width: 16),
          _buildStatCard('执行', '${state.executingCount}', Icons.bolt),
          const SizedBox(width: 16),
          _buildStatCard('成功率', '${(state.globalSuccessRate * 100).toStringAsFixed(0)}%', Icons.check_circle),
          const SizedBox(width: 24),
          // 视图切换按钮
          _buildViewToggleButtons(),
        ],
      ),
    );
  }

  Widget _buildStatCard(String label, String value, IconData icon) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.blue.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 18, color: Colors.blue),
          const SizedBox(width: 8),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(value, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
              Text(label, style: const TextStyle(fontSize: 12, color: Colors.grey)),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildViewToggleButtons() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildViewButton('全部', ViewMode.all, Icons.dashboard),
        _buildViewButton('进度', ViewMode.progress, Icons.timeline),
        _buildViewButton('协作', ViewMode.collaboration, Icons.device_hub),
        _buildViewButton('宠物', ViewMode.pet, Icons.pets),
      ],
    );
  }

  Widget _buildViewButton(String label, ViewMode mode, IconData icon) {
    final isSelected = _currentView == mode;
    return InkWell(
      onTap: () => setState(() => _currentView = mode),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: isSelected ? Colors.blue : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isSelected ? Colors.blue : Colors.grey.withOpacity(0.3),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 16, color: isSelected ? Colors.white : Colors.grey),
            const SizedBox(width: 4),
            Text(
              label,
              style: TextStyle(
                color: isSelected ? Colors.white : Colors.grey,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAgentTeamPanel(
    AgentRealtimeState realtimeState,
    AgentPetState petState,
  ) {
    return Container(
      width: 280,
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        border: Border(
          right: BorderSide(color: Colors.grey.withOpacity(0.2)),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 面板标题
          Container(
            padding: const EdgeInsets.all(16),
            child: const Row(
              children: [
                Icon(Icons.group, size: 20),
                SizedBox(width: 8),
                Text('Agent团队', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
              ],
            ),
          ),
          // Agent列表
          Expanded(
            child: ListView.builder(
              itemCount: realtimeState.agents.length,
              itemBuilder: (context, index) {
                final agent = realtimeState.agents.values.elementAt(index);
                final pet = petState.pets.values.firstWhere(
                  (p) => p.agentId == agent.agentId,
                  orElse: () => _createDefaultPet(agent.agentId),
                );
                return _buildAgentCard(agent, pet);
              },
            ),
          ),
        ],
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

  Widget _buildAgentCard(AgentRealtimeStatus agent, AgentPet pet) {
    final isSelected = _selectedAgentId == agent.agentId;
    final activityIcon = _getActivityIcon(agent.currentActivity.type);
    final activityColor = _getActivityColor(agent.currentActivity.type);

    return InkWell(
      onTap: () => setState(() => _selectedAgentId = agent.agentId),
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: isSelected ? Colors.blue.withOpacity(0.1) : Colors.transparent,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSelected ? Colors.blue : Colors.grey.withOpacity(0.2),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Agent头像和名称
            Row(
              children: [
                AgentPetAvatar(
                  pet: pet,
                  size: 48,
                  showAnimation: true,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        agent.agentName,
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Row(
                        children: [
                          Icon(activityIcon, size: 14, color: activityColor),
                          const SizedBox(width: 4),
                          Text(
                            _getActivityText(agent.currentActivity.type),
                            style: TextStyle(
                              fontSize: 12,
                              color: activityColor,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
                // Lv徽章
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.purple.withOpacity(0.2),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    'Lv.${pet.level}',
                    style: const TextStyle(
                      fontSize: 12,
                      color: Colors.purple,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            // 进度条
            if (agent.currentActivity.type != AgentActivityType.idle)
              LinearProgressIndicator(
                value: agent.currentActivity.progress / 100,
                backgroundColor: Colors.grey.withOpacity(0.2),
                valueColor: AlwaysStoppedAnimation(activityColor),
              ),
            const SizedBox(height: 8),
            // 统计信息
            Row(
              children: [
                _buildMiniStat('任务', '${agent.stats.tasksCompleted}'),
                const SizedBox(width: 8),
                _buildMiniStat('成功率', '${(agent.stats.successRate * 100).toStringAsFixed(0)}%'),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMiniStat(String label, String value) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(label, style: const TextStyle(fontSize: 10, color: Colors.grey)),
        const SizedBox(width: 4),
        Text(value, style: const TextStyle(fontSize: 10, fontWeight: FontWeight.bold)),
      ],
    );
  }

  IconData _getActivityIcon(AgentActivityType type) {
    switch (type) {
      case AgentActivityType.thinking:
        return Icons.psychology;
      case AgentActivityType.executing:
        return Icons.bolt;
      case AgentActivityType.outputting:
        return Icons.output;
      case AgentActivityType.idle:
        return Icons.bedtime;
      case AgentActivityType.waiting:
        return Icons.hourglass_empty;
      case AgentActivityType.error:
        return Icons.error;
    }
  }

  Color _getActivityColor(AgentActivityType type) {
    switch (type) {
      case AgentActivityType.thinking:
        return Colors.blue;
      case AgentActivityType.executing:
        return Colors.orange;
      case AgentActivityType.outputting:
        return Colors.green;
      case AgentActivityType.idle:
        return Colors.grey;
      case AgentActivityType.waiting:
        return Colors.purple;
      case AgentActivityType.error:
        return Colors.red;
    }
  }

  String _getActivityText(AgentActivityType type) {
    switch (type) {
      case AgentActivityType.thinking:
        return '正在思考';
      case AgentActivityType.executing:
        return '正在执行';
      case AgentActivityType.outputting:
        return '正在输出';
      case AgentActivityType.idle:
        return '空闲';
      case AgentActivityType.waiting:
        return '等待';
      case AgentActivityType.error:
        return '错误';
    }
  }

  Widget _buildMainContent(
    AgentRealtimeState realtimeState,
    AgentPetState petState,
  ) {
    final selectedAgent = realtimeState.agents[_selectedAgentId ?? ''];
    final selectedPet = petState.pets.values.firstWhere(
      (p) => p.agentId == _selectedAgentId,
      orElse: () => _createDefaultPet(_selectedAgentId ?? ''),
    );

    switch (_currentView) {
      case ViewMode.all:
        return _buildAllView(realtimeState, petState, selectedAgent, selectedPet);
      case ViewMode.progress:
        return AgentProgressPanel(agent: selectedAgent);
      case ViewMode.collaboration:
        return const CollaborationNetwork();
      case ViewMode.pet:
        return _buildPetDetailView(selectedPet);
    }
  }

  Widget _buildAllView(
    AgentRealtimeState realtimeState,
    AgentPetState petState,
    AgentRealtimeStatus? agent,
    AgentPet pet,
  ) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 实时进度面板
          if (agent != null)
            AgentProgressPanel(agent: agent),
          const SizedBox(height: 16),
          // 协作网络迷你版
          const CollaborationNetwork(miniMode: true),
          const SizedBox(height: 16),
          // 宠物互动面板
          _buildPetInteractionPanel(pet),
        ],
      ),
    );
  }

  Widget _buildPetDetailView(AgentPet pet) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 大头像
          Center(
            child: AgentPetAvatar(
              pet: pet,
              size: 120,
              showAnimation: true,
            ),
          ),
          const SizedBox(height: 24),
          // 状态信息
          _buildPetStatusCard(pet),
          const SizedBox(height: 16),
          // 互动按钮
          _buildInteractionButtons(pet),
          const SizedBox(height: 16),
          // 成就展示
          _buildAchievementsSection(pet),
        ],
      ),
    );
  }

  Widget _buildPetInteractionPanel(AgentPet pet) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.grey.withOpacity(0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(Icons.pets, size: 20),
              SizedBox(width: 8),
              Text('宠物状态', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              _buildEmotionIndicator(pet),
              _buildHappinessBar(pet),
              _buildIntimacyBar(pet),
            ],
          ),
          const SizedBox(height: 12),
          _buildMiniInteractionButtons(pet),
        ],
      ),
    );
  }

  Widget _buildEmotionIndicator(AgentPet pet) {
    return Column(
      children: [
        Text(
          _getEmotionEmoji(pet.emotion),
          style: const TextStyle(fontSize: 32),
        ),
        const SizedBox(height: 4),
        Text(pet.statusText, style: const TextStyle(fontSize: 12)),
      ],
    );
  }

  String _getEmotionEmoji(PetEmotion emotion) {
    switch (emotion) {
      case PetEmotion.happy:
        return '😊';
      case PetEmotion.thinking:
        return '🤔';
      case PetEmotion.sleepy:
        return '😴';
      case PetEmotion.confused:
        return '😕';
      case PetEmotion.excited:
        return '🎉';
      case PetEmotion.bored:
        return '😐';
      case PetEmotion.calm:
        return '🙂';
    }
  }

  Widget _buildHappinessBar(AgentPet pet) {
    return Column(
      children: [
        const Text('快乐度', style: TextStyle(fontSize: 12)),
        const SizedBox(height: 4),
        SizedBox(
          width: 80,
          child: LinearProgressIndicator(
            value: pet.happiness / 100,
            backgroundColor: Colors.grey.withOpacity(0.2),
            valueColor: AlwaysStoppedAnimation(Colors.green),
          ),
        ),
        Text('${pet.happiness.toStringAsFixed(0)}%', style: const TextStyle(fontSize: 10)),
      ],
    );
  }

  Widget _buildIntimacyBar(AgentPet pet) {
    return Column(
      children: [
        const Text('亲密度', style: TextStyle(fontSize: 12)),
        const SizedBox(height: 4),
        SizedBox(
          width: 80,
          child: LinearProgressIndicator(
            value: pet.intimacy / 100,
            backgroundColor: Colors.grey.withOpacity(0.2),
            valueColor: AlwaysStoppedAnimation(Colors.pink),
          ),
        ),
        Text('${pet.intimacy.toStringAsFixed(0)}%', style: const TextStyle(fontSize: 10)),
      ],
    );
  }

  Widget _buildMiniInteractionButtons(AgentPet pet) {
    final petStore = ref.read(agentPetProvider.notifier);

    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        _buildMiniButton('👋', '抚摸', () => petStore.interactWithPet(pet.petId, 'pet')),
        _buildMiniButton('👉', '戳', () => petStore.interactWithPet(pet.petId, 'poke')),
        _buildMiniButton('🍖', '喂食', () => petStore.interactWithPet(pet.petId, 'feed')),
        _buildMiniButton('🎾', '玩', () => petStore.interactWithPet(pet.petId, 'play')),
      ],
    );
  }

  Widget _buildMiniButton(String emoji, String label, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(
          color: Colors.blue.withOpacity(0.1),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          children: [
            Text(emoji, style: const TextStyle(fontSize: 20)),
            Text(label, style: const TextStyle(fontSize: 10)),
          ],
        ),
      ),
    );
  }

  Widget _buildPetStatusCard(AgentPet pet) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          Row(
            children: [
              Expanded(child: _buildStatusItem('等级', 'Lv.${pet.level}')),
              Expanded(child: _buildStatusItem('经验', '${pet.experience}')),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(child: _buildStatusItem('快乐度', '${pet.happiness.toStringAsFixed(0)}%')),
              Expanded(child: _buildStatusItem('亲密度', '${pet.intimacy.toStringAsFixed(0)}%')),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(child: _buildStatusItem('总任务', '${pet.totalTasks}')),
              Expanded(child: _buildStatusItem('成功率', '${((pet.successfulTasks / pet.totalTasks) * 100).toStringAsFixed(0)}%')),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildStatusItem(String label, String value) {
    return Column(
      children: [
        Text(label, style: const TextStyle(fontSize: 12, color: Colors.grey)),
        const SizedBox(height: 4),
        Text(value, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
      ],
    );
  }

  Widget _buildInteractionButtons(AgentPet pet) {
    final petStore = ref.read(agentPetProvider.notifier);

    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        ElevatedButton.icon(
          onPressed: () => petStore.interactWithPet(pet.petId, 'pet'),
          icon: const Text('👋', style: TextStyle(fontSize: 24)),
          label: const Text('抚摸 (+5快乐)'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.green,
            foregroundColor: Colors.white,
          ),
        ),
        ElevatedButton.icon(
          onPressed: () => petStore.interactWithPet(pet.petId, 'feed'),
          icon: const Text('🍖', style: TextStyle(fontSize: 24)),
          label: const Text('喂食 (+10快乐+5亲密)'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.orange,
            foregroundColor: Colors.white,
          ),
        ),
        ElevatedButton.icon(
          onPressed: () => petStore.interactWithPet(pet.petId, 'play'),
          icon: const Text('🎾', style: TextStyle(fontSize: 24)),
          label: const Text('玩耍 (+15快乐+10亲密)'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.purple,
            foregroundColor: Colors.white,
          ),
        ),
      ],
    );
  }

  Widget _buildAchievementsSection(AgentPet pet) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(Icons.emoji_events, size: 20, color: Colors.amber),
              SizedBox(width: 8),
              Text('成就', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            ],
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: pet.achievements.map((a) => _buildAchievementBadge(a)).toList(),
          ),
        ],
      ),
    );
  }

  Widget _buildAchievementBadge(String achievementId) {
    final achievementInfo = _getAchievementInfo(achievementId);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.amber.withOpacity(0.2),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.amber),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(achievementInfo['emoji']!, style: const TextStyle(fontSize: 16)),
          const SizedBox(width: 4),
          Text(achievementInfo['name']!, style: const TextStyle(fontSize: 12)),
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

  void _showLanguageDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('切换语言'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(title: const Text('🇨🇳 中文'), onTap: () {}),
            ListTile(title: const Text('🇺🇸 English'), onTap: () {}),
            ListTile(title: const Text('🇯🇵 日本語'), onTap: () {}),
            ListTile(title: const Text('🇰🇷 한국어'), onTap: () {}),
          ],
        ),
      ),
    );
  }

  void _showHelpDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.help),
            SizedBox(width: 8),
            Text('使用帮助'),
          ],
        ),
        content: const SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('快捷键:', style: TextStyle(fontWeight: FontWeight.bold)),
              SizedBox(height: 8),
              Text('Ctrl+1 → 全部视图'),
              Text('Ctrl+2 → 进度视图'),
              Text('Ctrl+3 → 协作视图'),
              Text('Ctrl+4 → 宠物视图'),
              SizedBox(height: 16),
              Text('宠物互动:', style: TextStyle(fontWeight: FontWeight.bold)),
              SizedBox(height: 8),
              Text('👋 抚摸: +5 快乐度'),
              Text('👉戳一下: -2 快乐度'),
              Text('🍖 喂食: +10 快乐度 +5 亲密度'),
              Text('🎾 玩耍: +15 快乐度 +10 亲密度'),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }
}