// E2E Test: Swarm Worker Registration and Task Execution
// Tests the Day 1-4 swarm implementation
//
// In Tauri environment: tests run against real Rust backend.
// In non-Tauri environment: tests run with mocked invokeOrProxy responses
//   that mirror the real Rust adapter return shapes.

import { test, expect, describe, vi, beforeEach, afterEach } from 'vitest'
import { isTauriHost } from '../../src/lib/platform'

// Mock invokeOrProxy for non-Tauri environments
const isTauri = isTauriHost()

if (!isTauri) {
  // Mock the host module to simulate Rust backend responses
  vi.mock('../../src/lib/host', async () => {
    const actual = await vi.importActual('../../src/lib/host')
    return {
      ...actual,
      invokeOrProxy: vi.fn(async (command: string, params?: Record<string, unknown>) => {
        switch (command) {
          case 'swarm_register_worker':
            return {
              workerId: params?.workerId,
              workerType: params?.workerType,
              capabilities: ['reasoning', 'code_generation', 'code_review', 'debugging'],
              maxComplexity: 5,
              maxConcurrent: 1,
              supportsStreaming: true,
              supportsCancel: true,
              defaultTimeoutMs: 600000,
            }
          case 'swarm_list_workers':
            return [{
              workerId: 'claude-code-test-1',
              workerType: 'claude_code',
              capabilities: ['reasoning', 'code_generation'],
              maxComplexity: 5,
              maxConcurrent: 1,
              supportsStreaming: true,
              supportsCancel: true,
              defaultTimeoutMs: 600000,
            }]
          case 'swarm_health_check':
            return true
          case 'swarm_get_worker_status':
            return {
              workerId: params?.workerId || 'claude-code-test-1',
              workerType: 'claude_code',
              health: 'healthy',
              pid: 12345,
              memoryBytes: 52428800,
              cpuPercent: 2.5,
              tasksCompleted: 3,
              tasksFailed: 0,
              currentTask: null,
              startedAt: { timestampMs: Date.now() - 60000 },
              lastHeartbeat: { timestampMs: Date.now() },
            }
          case 'swarm_send_task':
            return {
              taskId: params?.taskId,
              workerId: params?.workerId,
              status: 'running',
              startedAt: { timestampMs: Date.now() },
              output: '',
              error: null,
              pid: 12345,
            }
          case 'swarm_shutdown_worker':
            return undefined
          default:
            throw new Error(`Unknown command: ${command}`)
        }
      }),
    }
  })
}

// Import after mock setup
import {
  swarmRegisterWorker, swarmListWorkers, swarmHealthCheck,
  swarmSendTask, swarmGetWorkerStatus, swarmShutdownWorker,
} from '../../src/lib/swarm-api'

describe('Swarm Worker API', () => {
  const testWorkerId = 'claude-code-test-1'
  const testWorkerType = 'claude_code'

  test('should register Claude Code worker', async () => {
    const capabilities = await swarmRegisterWorker(testWorkerType, testWorkerId)

    expect(capabilities.workerId).toBe(testWorkerId)
    expect(capabilities.workerType).toBe(testWorkerType)
    expect(capabilities.capabilities.length).toBeGreaterThan(0)
    expect(capabilities.maxComplexity).toBeGreaterThanOrEqual(1)
    expect(capabilities.supportsStreaming).toBe(true)
  })

  test('should list registered workers', async () => {
    const workers = await swarmListWorkers()

    expect(workers.length).toBeGreaterThanOrEqual(1)
    expect(workers.some(w => w.workerId === testWorkerId)).toBe(true)
  })

  test('should health check worker', async () => {
    const healthy = await swarmHealthCheck(testWorkerId)

    expect(typeof healthy).toBe('boolean')
  })

  test('should get worker status', async () => {
    const status = await swarmGetWorkerStatus(testWorkerId)

    expect(status.workerId).toBe(testWorkerId)
    expect(status.workerType).toBe(testWorkerType)
    expect(['healthy', 'busy', 'starting', 'offline']).toContain(status.health)
  })

  test('should send task to worker', async () => {
    const taskId = `test-task-${Date.now()}`
    const handle = await swarmSendTask(
      testWorkerId,
      taskId,
      'Test prompt: respond with "OK"',
      undefined,
      30000
    )

    expect(handle.taskId).toBe(taskId)
    expect(handle.workerId).toBe(testWorkerId)
    expect(['pending', 'running']).toContain(handle.status)
  })

  test('should shutdown worker', async () => {
    await swarmShutdownWorker(testWorkerId)

    // After shutdown, list should still work (worker removed on Rust side)
    const workers = await swarmListWorkers()
    expect(Array.isArray(workers)).toBe(true)
  })
})

describe('Swarm Orchestrator Client', () => {
  test('should initialize orchestrator', async () => {
    const { SwarmOrchestratorClient } = await import('../../src/lib/swarm-api')
    const client = new SwarmOrchestratorClient()

    await client.initialize()
    const workers = client.listWorkers()

    expect(Array.isArray(workers)).toBe(true)
  })
})
