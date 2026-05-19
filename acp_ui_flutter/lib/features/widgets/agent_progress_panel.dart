/// Agent Progress Panel
/// 实时进度显示面板

import 'package:flutter/material.dart';
import '../../data/models/agent_realtime/agent_realtime_types.dart';

class AgentProgressPanel extends StatelessWidget {
  final AgentRealtimeStatus? agent;

  const AgentProgressPanel({
    super.key,
    this.agent,
  });

  @override
  Widget build(BuildContext context) {
    if (agent == null) {
      return _buildEmptyState();
    }

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
          // 标题
          Row(
            children: [
              Icon(Icons.timeline, size: 20, color: Colors.blue),
              const SizedBox(width: 8),
              Text(
                '${agent!.agentName} 实时进度',
                style: const TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.bold,
                ),
              ),
              const Spacer(),
              _buildActivityBadge(agent!.currentActivity.type),
            ],
          ),
          const SizedBox(height: 16),
          // 当前活动
          _buildCurrentActivity(),
          const SizedBox(height: 16),
          // 思考过程
          if (agent!.thinking.isStreaming) _buildThinkingSection(),
          const SizedBox(height: 16),
          // 输出区域
          if (agent!.output.content.isNotEmpty) _buildOutputSection(),
          const SizedBox(height: 16),
          // 统计数据
          _buildStatsSection(),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        borderRadius: BorderRadius.circular(12),
      ),
      child: const Center(
        child: Column(
          children: [
            Icon(Icons.hourglass_empty, size: 48, color: Colors.grey),
            SizedBox(height: 16),
            Text('选择一个Agent查看进度', style: TextStyle(color: Colors.grey)),
          ],
        ),
      ),
    );
  }

  Widget _buildActivityBadge(AgentActivityType type) {
    Color color;
    String text;
    IconData icon;

    switch (type) {
      case AgentActivityType.thinking:
        color = Colors.blue;
        text = '思考';
        icon = Icons.psychology;
        break;
      case AgentActivityType.executing:
        color = Colors.orange;
        text = '执行';
        icon = Icons.bolt;
        break;
      case AgentActivityType.outputting:
        color = Colors.green;
        text = '输出';
        icon = Icons.output;
        break;
      case AgentActivityType.idle:
        color = Colors.grey;
        text = '空闲';
        icon = Icons.bedtime;
        break;
      case AgentActivityType.waiting:
        color = Colors.purple;
        text = '等待';
        icon = Icons.hourglass_empty;
        break;
      case AgentActivityType.error:
        color = Colors.red;
        text = '错误';
        icon = Icons.error;
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.2),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 4),
          Text(text, style: TextStyle(color: color, fontSize: 12)),
        ],
      ),
    );
  }

  Widget _buildCurrentActivity() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.blue.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.play_circle, size: 16),
              const SizedBox(width: 8),
              const Text('当前活动', style: TextStyle(fontWeight: FontWeight.bold)),
              const Spacer(),
              Text(
                '${agent!.currentActivity.progress}%',
                style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
              ),
            ],
          ),
          const SizedBox(height: 8),
          LinearProgressIndicator(
            value: agent!.currentActivity.progress / 100,
            backgroundColor: Colors.grey.withOpacity(0.2),
            valueColor: const AlwaysStoppedAnimation(Colors.blue),
          ),
          if (agent!.currentActivity.description != null)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                agent!.currentActivity.description!,
                style: const TextStyle(fontSize: 12, color: Colors.grey),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildThinkingSection() {
    final thinking = agent!.thinking;

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.purple.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.psychology, size: 16, color: Colors.purple),
              const SizedBox(width: 8),
              const Text('思考过程', style: TextStyle(fontWeight: FontWeight.bold)),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.purple.withOpacity(0.3),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.stream, size: 12, color: Colors.white),
                    const SizedBox(width: 4),
                    const Text('实时', style: TextStyle(fontSize: 10, color: Colors.white)),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          // 思考内容
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.5),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              thinking.content,
              style: const TextStyle(fontSize: 13),
            ),
          ),
          const SizedBox(height: 12),
          // 思考块时间线
          _buildThinkingChunks(thinking.chunks),
        ],
      ),
    );
  }

  Widget _buildThinkingChunks(List<ThinkingChunk> chunks) {
    if (chunks.isEmpty) return const SizedBox.shrink();

    return Row(
      children: chunks.take(5).map((chunk) {
        final durationColor = chunk.duration > 200 ? Colors.orange : Colors.green;
        return Container(
          margin: const EdgeInsets.only(right: 4),
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: Colors.grey.withOpacity(0.2),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 8,
                height: 8,
                decoration: BoxDecoration(
                  color: durationColor,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                '${chunk.duration}ms',
                style: const TextStyle(fontSize: 10, color: Colors.grey),
              ),
            ],
          ),
        );
      }).toList(),
    );
  }

  Widget _buildOutputSection() {
    final output = agent!.output;

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.green.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.output, size: 16, color: Colors.green),
              const SizedBox(width: 8),
              const Text('输出结果', style: TextStyle(fontWeight: FontWeight.bold)),
              const Spacer(),
              if (output.isStreaming)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.green.withOpacity(0.3),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(Icons.edit, size: 12, color: Colors.white),
                      const SizedBox(width: 4),
                      const Text('生成中', style: TextStyle(fontSize: 10, color: Colors.white)),
                    ],
                  ),
                ),
            ],
          ),
          const SizedBox(height: 12),
          // 输出内容（打字机效果）
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.5),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              output.content.substring(0, output.currentPosition),
              style: const TextStyle(fontSize: 13),
            ),
          ),
          const SizedBox(height: 8),
          // 输出进度
          LinearProgressIndicator(
            value: output.currentPosition / output.totalLength,
            backgroundColor: Colors.grey.withOpacity(0.2),
            valueColor: const AlwaysStoppedAnimation(Colors.green),
          ),
        ],
      ),
    );
  }

  Widget _buildStatsSection() {
    final stats = agent!.stats;

    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceAround,
      children: [
        _buildStatItem('完成任务', '${stats.tasksCompleted}', Icons.check_circle, Colors.green),
        _buildStatItem('失败任务', '${stats.tasksFailed}', Icons.error, Colors.red),
        _buildStatItem('成功率', '${(stats.successRate * 100).toStringAsFixed(0)}%', Icons.thumb_up, Colors.blue),
        _buildStatItem('平均耗时', '${(stats.averageDuration / 1000).toStringAsFixed(1)}s', Icons.timer, Colors.orange),
      ],
    );
  }

  Widget _buildStatItem(String label, String value, IconData icon, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 20, color: color),
          const SizedBox(height: 4),
          Text(value, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold)),
          Text(label, style: const TextStyle(fontSize: 10, color: Colors.grey)),
        ],
      ),
    );
  }
}