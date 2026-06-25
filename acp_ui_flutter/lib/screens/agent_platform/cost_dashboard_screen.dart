// Cost Dashboard Screen - Cost tracking and budget management

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import '../../data/services/agent_platform_api.dart';
import '../../features/agent_platform/cubit/agent_platform_cubit.dart';

class CostDashboardScreen extends StatelessWidget {
  const CostDashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => AgentPlatformCubit()..fetchCostSummary(),
      child: const CostDashboardView(),
    );
  }
}

class CostDashboardView extends StatefulWidget {
  const CostDashboardView({super.key});

  @override
  State<CostDashboardView> createState() => _CostDashboardViewState();
}

class _CostDashboardViewState extends State<CostDashboardView> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('成本追踪'),
        centerTitle: true,
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => context.read<AgentPlatformCubit>().fetchCostSummary(),
          ),
        ],
      ),
      body: BlocBuilder<AgentPlatformCubit, AgentPlatformState>(
        builder: (context, state) {
          if (state is AgentPlatformLoading) {
            return const Center(child: CircularProgressIndicator());
          }

          if (state is AgentPlatformError) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.error_outline, size: 48, color: Colors.red),
                  const SizedBox(height: 16),
                  Text(state.message),
                  const SizedBox(height: 16),
                  ElevatedButton(
                    onPressed: () => context.read<AgentPlatformCubit>().fetchCostSummary(),
                    child: const Text('重试'),
                  ),
                ],
              ),
            );
          }

          if (state is CostLoaded) {
            return _buildDashboard(state.summary);
          }

          return const Center(child: Text('加载中...'));
        },
      ),
    );
  }

  Widget _buildDashboard(CostSummary summary) {
    final budgetPercent = summary.budgetRemainingPercent;
    final progressColor = budgetPercent > 50 ? Colors.green :
        budgetPercent > 20 ? Colors.orange : Colors.red;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Monthly Total Card
          _buildCostCard(
            title: '本月成本',
            amount: summary.monthlyTotal,
            currency: summary.currency,
            showProgress: true,
            progressValue: 100 - budgetPercent,
            progressColor: progressColor,
            subtitle: '预算剩余: ${budgetPercent.toStringAsFixed(1)}%',
          ),
          const SizedBox(height: 16),

          // Daily Total Card
          _buildCostCard(
            title: '今日成本',
            amount: summary.dailyTotal,
            currency: summary.currency,
          ),
          const SizedBox(height: 16),

          // Currency Card
          _buildCostCard(
            title: '货币',
            amount: 0,
            currency: summary.currency,
            showAmount: false,
            customContent: Text(
              summary.currency,
              style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 24),

          // Thresholds Legend
          const Text(
            '预算状态',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              _buildThresholdItem(
                color: Colors.green,
                label: '> 50%',
                desc: '预算充足',
              ),
              const SizedBox(width: 12),
              _buildThresholdItem(
                color: Colors.orange,
                label: '20-50%',
                desc: '注意监控',
              ),
              const SizedBox(width: 12),
              _buildThresholdItem(
                color: Colors.red,
                label: '< 20%',
                desc: '即将耗尽',
              ),
            ],
          ),
          const SizedBox(height: 24),

          // Budget Warning (if low)
          if (budgetPercent < 20)
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.red[50],
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: Colors.red[200]!),
              ),
              child: Row(
                children: [
                  const Icon(Icons.warning, color: Colors.red),
                  const SizedBox(width: 12),
                  const Expanded(
                    child: Text(
                      '预算即将耗尽！请考虑调整预算或减少使用频率。',
                      style: TextStyle(color: Colors.red),
                    ),
                  ),
                ],
              ),
            ),
          const SizedBox(height: 24),

          // Refresh Button
          OutlinedButton.icon(
            onPressed: () => context.read<AgentPlatformCubit>().fetchCostSummary(),
            icon: const Icon(Icons.refresh),
            label: const Text('刷新数据'),
            style: OutlinedButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 16),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCostCard({
    required String title,
    required double amount,
    required String currency,
    bool showProgress = false,
    double progressValue = 0,
    Color? progressColor,
    String? subtitle,
    bool showAmount = true,
    Widget? customContent,
  }) {
    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text(
              title,
              style: TextStyle(fontSize: 14, color: Colors.grey[600]),
            ),
            const SizedBox(height: 12),
            if (showAmount)
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    currency,
                    style: const TextStyle(fontSize: 16, color: Colors.grey),
                  ),
                  const SizedBox(width: 4),
                  Text(
                    amount.toStringAsFixed(2),
                    style: const TextStyle(fontSize: 32, fontWeight: FontWeight.bold),
                  ),
                ],
              ),
            if (customContent != null) customContent,
            if (subtitle != null)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  subtitle,
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ),
            if (showProgress)
              Padding(
                padding: const EdgeInsets.only(top: 12),
                child: LinearProgressIndicator(
                  value: progressValue / 100,
                  backgroundColor: Colors.grey[200],
                  color: progressColor,
                  minHeight: 8,
                  borderRadius: BorderRadius.circular(4),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildThresholdItem({
    required Color color,
    required String label,
    required String desc,
  }) {
    return Expanded(
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: color.withOpacity(0.1),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          children: [
            Text(
              label,
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              desc,
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
          ],
        ),
      ),
    );
  }
}