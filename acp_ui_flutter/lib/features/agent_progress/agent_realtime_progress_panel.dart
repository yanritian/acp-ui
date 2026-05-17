import 'package:flutter/material.dart';
import '../../data/models/agent_realtime/agent_realtime_types.dart';
import '../agent_pet/agent_pet_avatar.dart';

/// Agent实时进度面板 (Phase 2)
/// Claude Code风格的实时状态可视化
class AgentRealtimeProgressPanel extends StatefulWidget {
  final String agentId;
  final String agentName;
  final AgentRealtimeStatus? initialStatus;

  const AgentRealtimeProgressPanel({
    super.key,
    required this.agentId,
    required this.agentName,
    this.initialStatus,
  });

  @override
  State<AgentRealtimeProgressPanel> createState() => _AgentRealtimeProgressPanelState();
}

class _AgentRealtimeProgressPanelState extends State<AgentRealtimeProgressPanel> {
  late AgentRealtimeStatus _status;
  bool _isConnected = false;

  @override
  void initState() {
    super.initState();
    _status = widget.initialStatus ?? _createDefaultStatus();
    // 模拟连接状态
    Future.delayed(Duration(milliseconds: 500), () {
      setState(() => _isConnected = true);
    });
  }

  AgentRealtimeStatus _createDefaultStatus() {
    return AgentRealtimeStatus(
      agentId: widget.agentId,
      agentName: widget.agentName,
      currentActivity: CurrentActivity(
        type: AgentActivityType.idle,
        startTime: DateTime.now().millisecondsSinceEpoch,
        duration: 0,
        progress: 0,
      ),
      thinking: ThinkingState(
        content: '',
        startTime: 0,
        depth: 0,
        chunks: [],
        isStreaming: false,
      ),
      output: OutputState(
        content: '',
        chunks: [],
        totalLength: 0,
        currentPosition: 0,
        isStreaming: false,
      ),
      stats: AgentStats(
        tasksCompleted: 0,
        tasksFailed: 0,
        averageDuration: 0,
        successRate: 100,
        totalThinkingTime: 0,
        totalToolCalls: 0,
      ),
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [Color(0xFF1a1a2e), Color(0xFF16213e)],
        ),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Color(0xFF3a3a5a)),
      ),
      child: Column(
        children: [
          _buildHeader(),
          Expanded(child: _buildContent()),
          _buildFooter(),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    final indicator = activityIndicators[_status.currentActivity.type]!;
    final elapsed = _formatDuration(_status.currentActivity.duration);

    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(0xFF1a1a2e).withOpacity(0.8),
        border: Border(bottom: BorderSide(color: Color(0xFF3a3a5a))),
      ),
      child: Row(
        children: [
          // Pet avatar
          AgentPetAvatar(
            agentId: widget.agentId,
            agentName: widget.agentName,
            currentActivity: _status.currentActivity.type,
            emotion: _getEmotionFromActivity(_status.currentActivity.type),
            size: 50,
          ),
          SizedBox(width: 12),
          // Agent info
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                widget.agentName,
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                  color: Colors.white,
                ),
              ),
              Text(
                elapsed,
                style: TextStyle(fontSize: 12, color: Color(0xFF8b8b9b)),
              ),
            ],
          ),
          Spacer(),
          // Activity indicator
          _buildActivityIndicator(indicator),
        ],
      ),
    );
  }

  Widget _buildActivityIndicator(ActivityIndicatorConfig indicator) {
    return Container(
      padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: _parseColor(indicator.color),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Text(indicator.icon, style: TextStyle(fontSize: 18)),
          SizedBox(width: 8),
          Text(
            indicator.description,
            style: TextStyle(fontSize: 13, color: Colors.white),
          ),
        ],
      ),
    );
  }

  Widget _buildContent() {
    final activityType = _status.currentActivity.type;

    return Padding(
      padding: EdgeInsets.all(20),
      child: SingleChildScrollView(
        child: Column(
          children: [
            if (activityType == AgentActivityType.thinking)
              _buildThinkingDisplay(),
            if (activityType == AgentActivityType.executing)
              _buildToolExecution(),
            if (activityType == AgentActivityType.outputting)
              _buildOutputDisplay(),
            if (activityType == AgentActivityType.waiting)
              _buildPermissionWaiting(),
            if (activityType == AgentActivityType.idle)
              _buildIdleState(),
            if (activityType == AgentActivityType.error)
              _buildErrorState(),
          ],
        ),
      ),
    );
  }

  Widget _buildThinkingDisplay() {
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(0xFF8B5CF6).withOpacity(0.1),
        border: Border.all(color: Color(0xFF8B5CF6).withOpacity(0.3)),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text('💭', style: TextStyle(fontSize: 24)),
              SizedBox(width: 8),
              Text('思考过程', style: TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: Color(0xFF8b5cf6),
              )),
              Spacer(),
              Text('深度: ${'🧠' * _status.thinking.depth}',
                style: TextStyle(fontSize: 12)),
            ],
          ),
          SizedBox(height: 12),
          Text(
            _status.thinking.content,
            style: TextStyle(fontSize: 13, color: Colors.white),
          ),
          if (_status.thinking.isStreaming)
            Padding(
              padding: EdgeInsets.only(top: 8),
              child: Row(
                children: [
                  Container(
                    width: 8, height: 8,
                    decoration: BoxDecoration(
                      color: Color(0xFF8b5cf6),
                      shape: BoxShape.circle,
                    ),
                  ),
                  SizedBox(width: 8),
                  Text('实时思考中...', style: TextStyle(
                    fontSize: 11, color: Color(0xFF8b5cf6))),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildToolExecution() {
    if (_status.toolExecution == null) return SizedBox();

    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(0xFFF59E0B).withOpacity(0.1),
        border: Border.all(color: Color(0xFFF59E0B).withOpacity(0.3)),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text('🔧', style: TextStyle(fontSize: 20)),
              SizedBox(width: 12),
              Text(_status.toolExecution!.toolName,
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
              Spacer(),
              Text(_formatDuration(_status.toolExecution!.duration ?? 0),
                style: TextStyle(fontSize: 12, color: Color(0xFF8b8b9b))),
            ],
          ),
          SizedBox(height: 12),
          LinearProgressIndicator(
            value: _status.toolExecution!.progress / 100,
            backgroundColor: Color(0xFF3a3a5a),
            valueColor: AlwaysStoppedAnimation(Color(0xFFF59E0B)),
          ),
          SizedBox(height: 8),
          Text('进度: ${_status.toolExecution!.progress}%',
            style: TextStyle(fontSize: 12, color: Colors.white)),
        ],
      ),
    );
  }

  Widget _buildOutputDisplay() {
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(0xFF10B981).withOpacity(0.1),
        border: Border.all(color: Color(0xFF10B981).withOpacity(0.3)),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text('💬', style: TextStyle(fontSize: 20)),
              SizedBox(width: 8),
              Text('Agent输出', style: TextStyle(
                fontSize: 14, fontWeight: FontWeight.w600, color: Color(0xFF10b981))),
              Spacer(),
              Text('${_status.output.currentPosition} / ${_status.output.totalLength}',
                style: TextStyle(fontSize: 12, color: Color(0xFF8b8b9b))),
            ],
          ),
          SizedBox(height: 12),
          Text(_status.output.content,
            style: TextStyle(fontSize: 13, color: Colors.white)),
        ],
      ),
    );
  }

  Widget _buildPermissionWaiting() {
    if (_status.permissionWaiting == null) return SizedBox();

    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(0xFF6B7280).withOpacity(0.1),
        border: Border.all(color: Color(0xFF6B7280).withOpacity(0.3)),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          Row(
            children: [
              Text('⏸️', style: TextStyle(fontSize: 20)),
              SizedBox(width: 8),
              Text('等待权限', style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600)),
            ],
          ),
          SizedBox(height: 12),
          Text(_status.permissionWaiting!.description,
            style: TextStyle(fontSize: 13, color: Colors.white)),
          SizedBox(height: 16),
          ...(_status.permissionWaiting!.options.map((option) =>
            ElevatedButton(
              onPressed: () {},
              style: ElevatedButton.styleFrom(
                backgroundColor: Color(0xFF3B82F6),
                minimumSize: Size(double.infinity, 40),
              ),
              child: Text(option.label),
            ),
          ).toList()),
        ],
      ),
    );
  }

  Widget _buildIdleState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text('😴', style: TextStyle(fontSize: 48)),
          SizedBox(height: 16),
          Text('Agent空闲，等待新任务...',
            style: TextStyle(fontSize: 14, color: Color(0xFF8b8b9b))),
        ],
      ),
    );
  }

  Widget _buildErrorState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text('❌', style: TextStyle(fontSize: 48)),
          SizedBox(height: 16),
          Text('执行出错，请检查日志',
            style: TextStyle(fontSize: 14, color: Colors.red)),
        ],
      ),
    );
  }

  Widget _buildFooter() {
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Color(0xFF16213e).withOpacity(0.9),
        border: Border(top: BorderSide(color: Color(0xFF3a3a5a))),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildStatItem('✅', '${_status.stats.tasksCompleted}', '完成'),
          _buildStatItem('❌', '${_status.stats.tasksFailed}', '失败'),
          _buildStatItem('⚡', '${_status.stats.totalToolCalls}', '工具'),
          _buildStatItem('🧠', '${(_status.stats.totalThinkingTime / 1000).round()}s', '思考'),
          _buildStatItem('📊', '${_status.stats.successRate.toInt()}%', '成功'),
        ],
      ),
    );
  }

  Widget _buildStatItem(String icon, String value, String label) {
    return Container(
      padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Color(0xFF3a3a5a).withOpacity(0.3),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Row(
        children: [
          Text(icon, style: TextStyle(fontSize: 14)),
          SizedBox(width: 4),
          Text(value, style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: Colors.white)),
          SizedBox(width: 4),
          Text(label, style: TextStyle(fontSize: 11, color: Color(0xFF8b8b9b))),
        ],
      ),
    );
  }

  AgentEmotionType _getEmotionFromActivity(AgentActivityType activity) {
    switch (activity) {
      case AgentActivityType.thinking:
        return AgentEmotionType.focused;
      case AgentActivityType.executing:
        return AgentEmotionType.focused;
      case AgentActivityType.outputting:
        return AgentEmotionType.happy;
      case AgentActivityType.waiting:
        return AgentEmotionType.bored;
      case AgentActivityType.idle:
        return AgentEmotionType.happy;
      case AgentActivityType.error:
        return AgentEmotionType.confused;
    }
  }

  String _formatDuration(int duration) {
    if (duration < 1000) return '$duration ms';
    if (duration < 60000) return '${(duration / 1000).toStringAsFixed(1)}s';
    return '${(duration / 60000).toStringAsFixed(1)}m';
  }

  Color _parseColor(String hex) {
    return Color(int.parse(hex.replaceFirst('#', '0xFF')));
  }
}