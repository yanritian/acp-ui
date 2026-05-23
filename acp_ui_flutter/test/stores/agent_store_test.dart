import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/data/stores/agent_store.dart';
import 'package:acp_ui_flutter/data/models/agent.dart';

void main() {
  group('Agent Store Providers', () {
    test('agentListProvider returns correct mock data', () {
      final container = ProviderContainer();
      final agents = container.read(agentListProvider);

      expect(agents.length, 5);

      // Verify agent types
      expect(agents.any((a) => a.type == AgentType.planner), true);
      expect(agents.any((a) => a.type == AgentType.architect), true);
      expect(agents.any((a) => a.type == AgentType.codeReviewer), true);
      expect(agents.any((a) => a.type == AgentType.tddGuide), true);
      expect(agents.any((a) => a.type == AgentType.securityReviewer), true);

      container.dispose();
    });

    test('selectedAgentProvider starts as null', () {
      final container = ProviderContainer();
      final selectedAgent = container.read(selectedAgentProvider);

      expect(selectedAgent, isNull);

      container.dispose();
    });

    test('selectedAgentProvider can be updated', () {
      final container = ProviderContainer();
      final agent = Agent(
        id: 'test-agent',
        name: 'Test Agent',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );

      container.read(selectedAgentProvider.notifier).state = agent;

      expect(container.read(selectedAgentProvider), agent);

      container.dispose();
    });

    test('agentStatusProvider starts with empty map', () {
      final container = ProviderContainer();
      final statusMap = container.read(agentStatusProvider);

      expect(statusMap, isEmpty);

      container.dispose();
    });

    test('AgentStatusNotifier can update status', () {
      final container = ProviderContainer();
      final notifier = container.read(agentStatusProvider.notifier);

      notifier.updateStatus('agent-1', AgentStatus.busy);

      expect(container.read(agentStatusProvider)['agent-1'], AgentStatus.busy);

      container.dispose();
    });

    test('AgentStatusNotifier convenience methods work', () {
      final container = ProviderContainer();
      final notifier = container.read(agentStatusProvider.notifier);

      notifier.setIdle('agent-1');
      expect(container.read(agentStatusProvider)['agent-1'], AgentStatus.idle);

      notifier.setBusy('agent-2');
      expect(container.read(agentStatusProvider)['agent-2'], AgentStatus.busy);

      notifier.setError('agent-3');
      expect(container.read(agentStatusProvider)['agent-3'], AgentStatus.error);

      notifier.setOffline('agent-4');
      expect(container.read(agentStatusProvider)['agent-4'], AgentStatus.offline);

      container.dispose();
    });
  });
}