// One-Shot Screen - Main Agent Platform interface

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../../data/services/agent_platform_api.dart';
import '../../features/agent_platform/cubit/agent_platform_cubit.dart';

class OneShotScreen extends StatelessWidget {
  const OneShotScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => AgentPlatformCubit()..fetchAgents(),
      child: const OneShotView(),
    );
  }
}

class OneShotView extends StatefulWidget {
  const OneShotView({super.key});

  @override
  State<OneShotView> createState() => _OneShotViewState();
}

class _OneShotViewState extends State<OneShotView> {
  final TextEditingController _inputController = TextEditingController();
  String? _preferredAgent;
  double? _maxCost;

  final Map<String, String> _roleLabels = {
    'developer': '开发者',
    'marketer': '营销人员',
    'designer': '设计师',
    'finance': '财务人员',
    'gamer': '游戏开发者',
    'writer': '内容创作者',
    'analyst': '数据分析师',
    'general': '普通用户',
  };

  final Map<String, String> _sceneLabels = {
    'code_generation': '代码生成',
    'code_review': '代码审查',
    'bug_fix': 'Bug修复',
    'image_generation': '图片生成',
    'video_generation': '视频生成',
    'copywriting': '文案创作',
    'translation': '翻译',
    'document_generation': '文档生成',
    'excel_analysis': 'Excel分析',
    'game_build': '游戏构建',
    'general_task': '通用任务',
  };

  void _execute() {
    final cubit = context.read<AgentPlatformCubit>();
    final input = _inputController.text.trim();
    if (input.isEmpty) return;

    cubit.executeOneShot(
      input: input,
      preferredAgent: _preferredAgent,
      maxCost: _maxCost,
    );
  }

  void _clear() {
    _inputController.clear();
    _preferredAgent = null;
    _maxCost = null;
    context.read<AgentPlatformCubit>().fetchAgents();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Agent Platform'),
        centerTitle: true,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Header
            const Text(
              '一键智能任务执行',
              style: TextStyle(fontSize: 16, color: Colors.grey),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),

            // Main Input
            TextField(
              controller: _inputController,
              maxLines: 5,
              decoration: InputDecoration(
                hintText: '输入你的需求，例如：帮我写代码、生成图片...',
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
                filled: true,
                fillColor: Colors.grey[100],
              ),
            ),
            const SizedBox(height: 16),

            // Options Row
            Row(
              children: [
                Expanded(
                  flex: 2,
                  child: BlocBuilder<AgentPlatformCubit, AgentPlatformState>(
                    builder: (context, state) {
                      final agents = state is AgentsLoaded ? state.agents : [];
                      return DropdownButtonFormField<String>(
                        initialValue: _preferredAgent,
                        decoration: InputDecoration(
                          labelText: 'Agent',
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(8),
                          ),
                        ),
                        items: [
                          const DropdownMenuItem(value: null, child: Text('自动选择')),
                          ...agents.map((a) => DropdownMenuItem(
                            value: a.id,
                            child: Text(a.name),
                          )),
                        ],
                        onChanged: (v) => setState(() => _preferredAgent = v),
                      );
                    },
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    keyboardType: TextInputType.number,
                    decoration: InputDecoration(
                      labelText: '最大成本',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                    onChanged: (v) => _maxCost = double.tryParse(v),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Action Buttons
            Row(
              children: [
                Expanded(
                  child: ElevatedButton(
                    onPressed: _inputController.text.isNotEmpty ? _execute : null,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.blue,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(vertical: 16),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                    child: const Text('执行'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: OutlinedButton(
                    onPressed: _clear,
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 16),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                    child: const Text('清空'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 24),

            // Result Section
            BlocBuilder<AgentPlatformCubit, AgentPlatformState>(
              builder: (context, state) {
                if (state is OneShotExecuting) {
                  return const Center(
                    child: Column(
                      children: [
                        CircularProgressIndicator(),
                        SizedBox(height: 12),
                        Text('执行中...'),
                      ],
                    ),
                  );
                }

                if (state is OneShotCompleted) {
                  return _buildResultCard(state.result, state.agents);
                }

                if (state is AgentPlatformError) {
                  return Card(
                    color: Colors.red[50],
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Row(
                        children: [
                          const Icon(Icons.error, color: Colors.red),
                          const SizedBox(width: 12),
                          Expanded(child: Text(state.message)),
                          IconButton(
                            icon: const Icon(Icons.close),
                            onPressed: () => context.read<AgentPlatformCubit>().clearError(),
                          ),
                        ],
                      ),
                    ),
                  );
                }

                return const SizedBox.shrink();
              },
            ),
            const SizedBox(height: 24),

            // Agents Grid
            BlocBuilder<AgentPlatformCubit, AgentPlatformState>(
              builder: (context, state) {
                final agents = state is AgentsLoaded ? state.agents :
                    state is OneShotCompleted ? state.agents : [];
                if (agents.isEmpty) return const SizedBox.shrink();

                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('可用 Agent (${agents.length})',
                      style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
                    const SizedBox(height: 12),
                    GridView.builder(
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: 2,
                        mainAxisSpacing: 12,
                        crossAxisSpacing: 12,
                        childAspectRatio: 1.5,
                      ),
                      itemCount: agents.length,
                      itemBuilder: (_, i) => _buildAgentCard(agents[i]),
                    ),
                  ],
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildResultCard(OneShotResponse result, List<AgentSummary> agents) {
    final roleLabel = _roleLabels[result.detectedRole] ?? result.detectedRole;
    final sceneLabel = _sceneLabels[result.detectedScene] ?? result.detectedScene;
    final agent = agents.firstWhere(
      (a) => a.id == result.selectedAgent,
      orElse: () => agents.first,
    );

    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Detected Role & Scene
            Row(
              children: [
                Text('检测角色: ', style: TextStyle(color: Colors.grey[600])),
                Text(roleLabel, style: const TextStyle(color: Colors.blue)),
                const SizedBox(width: 16),
                Text('检测场景: ', style: TextStyle(color: Colors.grey[600])),
                Text(sceneLabel, style: const TextStyle(color: Colors.blue)),
              ],
            ),
            const SizedBox(height: 16),

            // Selected Agent
            Row(
              children: [
                const Icon(Icons.smart_toy, size: 32),
                const SizedBox(width: 12),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(agent.name, style: const TextStyle(fontWeight: FontWeight.bold)),
                    Text(result.selectionReason, style: TextStyle(color: Colors.grey[600], fontSize: 12)),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Result Preview
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey[100],
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(result.resultPreview),
            ),
            const SizedBox(height: 12),

            // Cost Info
            DefaultTextStyle(
              style: TextStyle(color: Colors.grey[600], fontSize: 12),
              child: Row(
                children: [
                  Text('预估成本: ¥${result.estimatedCost.toStringAsFixed(4)}'),
                  const SizedBox(width: 16),
                  Text('预计耗时: ${(result.transparency.estimatedDurationMs / 1000).toStringAsFixed(0)}s'),
                  const SizedBox(width: 16),
                  Text('隐私级别: ${result.transparency.privacyLevel}'),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAgentCard(AgentSummary agent) {
    return Card(
      elevation: 1,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(agent.name, style: const TextStyle(fontWeight: FontWeight.w500)),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: agent.status == 'available' ? Colors.green[100] : Colors.red[100],
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    agent.status,
                    style: TextStyle(
                      fontSize: 10,
                      color: agent.status == 'available' ? Colors.green[700] : Colors.red[700],
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: LinearProgressIndicator(
                    value: agent.healthScore / 100,
                    backgroundColor: Colors.grey[200],
                    color: Colors.green,
                  ),
                ),
                const SizedBox(width: 8),
                Text('${agent.healthScore.toStringAsFixed(0)}%',
                  style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 4,
              children: agent.capabilities.take(3).map((c) =>
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.grey[200],
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(c, style: const TextStyle(fontSize: 10)),
                ),
              ).toList(),
            ),
          ],
        ),
      ),
    );
  }
}