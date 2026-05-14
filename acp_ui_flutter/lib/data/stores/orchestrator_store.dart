import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/orchestrator/orchestrator.dart';
import '../../core/agent/agent_pool.dart';
import '../../core/session/session_manager.dart';
import 'agent_store.dart';
import 'session_store.dart';

/// Orchestrator Provider
final orchestratorProvider = Provider<Orchestrator>((ref) {
  final pool = ref.watch(agentPoolProvider);
  final sessionManager = ref.watch(sessionManagerProvider);
  return Orchestrator(
    agentPool: pool,
    sessionManager: sessionManager,
  );
});

/// Tasks Provider
final tasksProvider = Provider<List<Task>>((ref) {
  final orchestrator = ref.watch(orchestratorProvider);
  return orchestrator.listTasks();
});

/// Active Tasks Provider
final activeTasksProvider = Provider<List<Task>>((ref) {
  final tasks = ref.watch(tasksProvider);
  return tasks.where((t) => t.status == TaskStatus.running).toList();
});

/// Task Stream Provider
final taskStreamProvider = StreamProvider<Task>((ref) {
  final orchestrator = ref.watch(orchestratorProvider);
  return orchestrator.taskStream;
});

/// Task Execution Provider (async)
final taskExecutionProvider = FutureProvider.family<String, String>((ref, taskId) async {
  final orchestrator = ref.watch(orchestratorProvider);
  return orchestrator.executeTask(taskId);
});