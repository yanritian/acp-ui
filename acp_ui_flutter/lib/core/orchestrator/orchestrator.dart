import 'dart:async';

import '../agent/agent_pool.dart';
import '../agent/agent_bridge.dart';
import '../session/session_manager.dart';

/// Task status enum
enum TaskStatus {
  pending,
  running,
  completed,
  failed,
  paused,
}

/// Task model
class Task {
  final String id;
  final String description;
  final TaskStatus status;
  final List<String> assignedAgents;
  final DateTime createdAt;
  final DateTime? completedAt;
  final String? result;

  Task({
    required this.id,
    required this.description,
    this.status = TaskStatus.pending,
    this.assignedAgents = const [],
    required this.createdAt,
    this.completedAt,
    this.result,
  });
}

/// Orchestrator - coordinates multi-agent task execution
class Orchestrator {
  final AgentPool _agentPool;
  final SessionManager _sessionManager;
  final Map<String, Task> _tasks = {};
  final StreamController<Task> _taskController = StreamController<Task>.broadcast();

  Orchestrator({
    required AgentPool agentPool,
    required SessionManager sessionManager,
  })  : _agentPool = agentPool,
        _sessionManager = sessionManager;

  /// Create a new task
  Task createTask(String description) {
    final task = Task(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      description: description,
      createdAt: DateTime.now(),
    );

    _tasks[task.id] = task;
    return task;
  }

  /// Assign agents to a task based on capabilities
  Task assignAgents(String taskId, List<String> requiredCapabilities) {
    final task = _tasks[taskId];
    if (task == null) {
      throw Exception('Task $taskId not found');
    }

    // Find agents matching capabilities
    final matchingAgents = _agentPool.getActiveAgents()
        .where((bridge) {
          // Check if agent has required capabilities
          // TODO: Implement capability matching
          return true;
        })
        .toList();

    final updatedTask = Task(
      id: task.id,
      description: task.description,
      status: TaskStatus.running,
      assignedAgents: matchingAgents.map((b) => b.agent.id).toList(),
      createdAt: task.createdAt,
    );

    _tasks[taskId] = updatedTask;
    _taskController.add(updatedTask);
    return updatedTask;
  }

  /// Execute a task with assigned agents
  Future<String> executeTask(String taskId) async {
    final task = _tasks[taskId];
    if (task == null) {
      throw Exception('Task $taskId not found');
    }

    if (task.assignedAgents.isEmpty) {
      throw Exception('No agents assigned to task $taskId');
    }

    // Create session for task execution
    final session = _sessionManager.createSession(task.assignedAgents.first);

    // Send task to primary agent
    final primaryBridge = _agentPool.getAgent(task.assignedAgents.first);
    if (primaryBridge == null) {
      throw Exception('Agent ${task.assignedAgents.first} not found');
    }

    await primaryBridge.sendMessage(task.description);

    // Collect results from all agents
    final results = await collectResults(task);

    // Mark task as completed
    final completedTask = Task(
      id: task.id,
      description: task.description,
      status: TaskStatus.completed,
      assignedAgents: task.assignedAgents,
      createdAt: task.createdAt,
      completedAt: DateTime.now(),
      result: results,
    );

    _tasks[taskId] = completedTask;
    _taskController.add(completedTask);

    return results;
  }

  /// Collect results from assigned agents
  Future<String> collectResults(Task task) async {
    final buffer = StringBuffer();

    for (final agentId in task.assignedAgents) {
      final bridge = _agentPool.getAgent(agentId);
      if (bridge == null) continue;

      // Wait for agent response
      final response = await bridge.receiveMessages().first.timeout(
        Duration(minutes: 5),
        onTimeout: () => AgentMessage(
          id: 'timeout',
          role: MessageRole.tool,
          content: 'Agent $agentId timed out',
          timestamp: DateTime.now(),
        ),
      );

      buffer.writeln('Agent $agentId: ${response.content}');
    }

    return buffer.toString();
  }

  /// Cancel a running task
  void cancelTask(String taskId) {
    final task = _tasks[taskId];
    if (task == null) return;

    _tasks[taskId] = Task(
      id: task.id,
      description: task.description,
      status: TaskStatus.failed,
      assignedAgents: task.assignedAgents,
      createdAt: task.createdAt,
      completedAt: DateTime.now(),
      result: 'Task cancelled',
    );
  }

  /// Get task by ID
  Task? getTask(String taskId) => _tasks[taskId];

  /// Get all tasks
  List<Task> listTasks() => _tasks.values.toList();

  /// Get task stream
  Stream<Task> get taskStream => _taskController.stream;
}