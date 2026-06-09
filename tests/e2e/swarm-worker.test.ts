// E2E Test: Swarm Worker Registration and Task Execution
// Tests the Day 1-4 swarm implementation

import { test, expect, describe } from 'vitest'
import { swarmRegisterWorker, swarmListWorkers, swarmHealthCheck, swarmSendTask, swarmGetWorkerStatus, swarmShutdownWorker } from '../src/lib/swarm-api'

// Note: These tests require Tauri runtime to be active
// Run with: npm run tauri dev first, then npm run test:e2e

describe('Swarm Worker API', () => {
  const testWorkerId = 'claude-code-test-1'
  const testWorkerType = 'claude_code'

  test.skip('should register Claude Code worker', async () => {
    const capabilities = await swarmRegisterWorker(testWorkerType, testWorkerId)

    expect(capabilities.workerId).toBe(testWorkerId)
    expect(capabilities.workerType).toBe(testWorkerType)
    expect(capabilities.capabilities.length).toBeGreaterThan(0)
    expect(capabilities.maxComplexity).toBeGreaterThanOrEqual(1)
    expect(capabilities.supportsStreaming).toBe(true)
  })

  test.skip('should list registered workers', async () => {
    const workers = await swarmListWorkers()

    expect(workers.length).toBeGreaterThanOrEqual(1)
    expect(workers.some(w => w.workerId === testWorkerId)).toBe(true)
  })

  test.skip('should health check worker', async () => {
    const healthy = await swarmHealthCheck(testWorkerId)

    expect(typeof healthy).toBe('boolean')
  })

  test.skip('should get worker status', async () => {
    const status = await swarmGetWorkerStatus(testWorkerId)

    expect(status.workerId).toBe(testWorkerId)
    expect(status.workerType).toBe(testWorkerType)
    expect(['healthy', 'busy', 'starting', 'offline']).toContain(status.health)
  })

  test.skip('should send task to worker', async () => {
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

  test.skip('should shutdown worker', async () => {
    await swarmShutdownWorker(testWorkerId)

    // After shutdown, health check should fail
    const workers = await swarmListWorkers()
    expect(workers.some(w => w.workerId === testWorkerId)).toBe(false)
  })
})

describe('Swarm Orchestrator Client', () => {
  test.skip('should initialize orchestrator', async () => {
    const { SwarmOrchestratorClient } = await import('../src/lib/swarm-api')
    const client = new SwarmOrchestratorClient()

    await client.initialize()
    const workers = client.listWorkers()

    expect(Array.isArray(workers)).toBe(true)
  })
})