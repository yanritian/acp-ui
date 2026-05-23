import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/core/orchestrator/orchestrator.dart';
import 'package:acp_ui_flutter/core/agent/agent_pool.dart';
import 'package:acp_ui_flutter/core/session/session_manager.dart';
import 'package:acp_ui_flutter/data/models/agent.dart';

void main() {
  group('Orchestrator', () {
    late Orchestrator orchestrator;
    late AgentPool pool;
    late SessionManager sessionManager;

    setUp(() {
      pool = AgentPool(baseUrl: 'ws://localhost:8080', maxAgents: 10);
      sessionManager = SessionManager(maxMessages: 100);
      orchestrator = Orchestrator(agentPool: pool, sessionManager: sessionManager);
    });

    test('should create orchestrator with dependencies', () {
      expect(orchestrator.listTasks(), isEmpty);
    });

    test('should create a new task', () {
      final task = orchestrator.createTask('Test task description');

      expect(task.id, isNotEmpty);
      expect(task.description, 'Test task description');
      expect(task.status, TaskStatus.pending);
      expect(task.assignedAgents, isEmpty);
    });

    test('should get task by ID', () {
      final task = orchestrator.createTask('Test task');
      final retrieved = orchestrator.getTask(task.id);

      expect(retrieved, isNotNull);
      expect(retrieved!.id, task.id);
    });

    test('should return null for non-existent task', () {
      expect(orchestrator.getTask('non-existent'), isNull);
    });

    test('should list all tasks', () {
      orchestrator.createTask('Task 1');
      orchestrator.createTask('Task 2');

      expect(orchestrator.listTasks().length, 2);
    });

    test('should throw when assigning agents to non-existent task', () {
      expect(
        () => orchestrator.assignAgents('non-existent', ['capability']),
        throwsException,
      );
    });

    test('should throw when executing non-existent task', () {
      expect(
        () => orchestrator.executeTask('non-existent'),
        throwsException,
      );
    });

    test('should cancel task gracefully', () {
      final task = orchestrator.createTask('Test task');
      orchestrator.cancelTask(task.id);

      final cancelled = orchestrator.getTask(task.id);
      expect(cancelled!.status, TaskStatus.failed);
      expect(cancelled.result, 'Task cancelled');
    });

    test('should cancel non-existent task gracefully', () {
      orchestrator.cancelTask('non-existent'); // Should not throw
    });
  });

  group('Task Model', () {
    test('should create task with defaults', () {
      final task = Task(
        id: 'task-1',
        description: 'Test',
        createdAt: DateTime.now(),
      );

      expect(task.status, TaskStatus.pending);
      expect(task.assignedAgents, isEmpty);
      expect(task.completedAt, isNull);
      expect(task.result, isNull);
    });

    test('should create task with all fields', () {
      final task = Task(
        id: 'task-1',
        description: 'Test',
        status: TaskStatus.completed,
        assignedAgents: ['agent-1', 'agent-2'],
        createdAt: DateTime(2024, 1, 1),
        completedAt: DateTime(2024, 1, 2),
        result: 'Success',
      );

      expect(task.status, TaskStatus.completed);
      expect(task.assignedAgents.length, 2);
      expect(task.result, 'Success');
    });
  });

  group('TaskStatus Enum', () {
    test('should have all expected values', () {
      expect(TaskStatus.values.length, 5);
      expect(TaskStatus.values.contains(TaskStatus.pending), true);
      expect(TaskStatus.values.contains(TaskStatus.running), true);
      expect(TaskStatus.values.contains(TaskStatus.completed), true);
      expect(TaskStatus.values.contains(TaskStatus.failed), true);
      expect(TaskStatus.values.contains(TaskStatus.paused), true);
    });
  });
}