// Worker SDK TypeScript Unit Tests
//
// Tests for AcpWorker SDK functionality

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import {
  AcpWorker,
  type WorkerCapabilities,
  type TaskDescription,
  type TaskResult,
  type WorkerStatus,
} from '../../../sdk/typescript/src/index'

describe('AcpWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('constructor', () => {
    it('creates worker without throwing', () => {
      const worker = new AcpWorker({
        workerId: 'worker-001',
        workerType: 'claude_code',
        capabilities: ['code-generation', 'test-writing'],
      })

      // Worker created successfully (no exception)
      expect(worker).toBeDefined()
    })

    it('creates worker with minimal config', () => {
      const worker = new AcpWorker({
        workerId: 'minimal-worker',
        workerType: 'codex',
        capabilities: [],
      })

      expect(worker).toBeDefined()
    })
  })

  describe('register', () => {
    it('calls swarm_register_worker Tauri command', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce({
        workerId: 'test-worker',
        workerType: 'claude_code',
        capabilities: ['test'],
        maxComplexity: 5,
        maxConcurrent: 3,
        supportsStreaming: true,
        supportsCancel: true,
        defaultTimeoutMs: 60000,
      })

      const worker = new AcpWorker({
        workerId: 'test-worker',
        workerType: 'claude_code',
        capabilities: ['test'],
      })

      await worker.register()

      expect(mockInvoke).toHaveBeenCalledWith('swarm_register_worker', {
        workerType: 'claude_code',
        workerId: 'test-worker',
      })
    })

    it('handles registration failure', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockRejectedValueOnce(new Error('Registration failed'))

      const worker = new AcpWorker({
        workerId: 'fail-worker',
        workerType: 'claude_code',
        capabilities: [],
      })

      await expect(worker.register()).rejects.toThrow('Registration failed')
    })
  })

  describe('shutdown', () => {
    it('calls swarm_shutdown_worker Tauri command', async () => {
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValueOnce(undefined)

      const worker = new AcpWorker({
        workerId: 'shutdown-test',
        workerType: 'claude_code',
        capabilities: [],
      })

      await worker.shutdown()

      expect(mockInvoke).toHaveBeenCalledWith('swarm_shutdown_worker', {
        workerId: 'shutdown-test',
      })
    })
  })
})

describe('WorkerCapabilities types', () => {
  it('validates WorkerCapabilities structure', () => {
    const caps: WorkerCapabilities = {
      workerId: 'worker-001',
      workerType: 'claude_code',
      capabilities: ['code-gen', 'test-write'],
      maxComplexity: 5,
      maxConcurrent: 3,
      supportsStreaming: true,
      supportsCancel: true,
      defaultTimeoutMs: 60000,
    }

    expect(caps.workerId).toBe('worker-001')
    expect(caps.workerType).toBe('claude_code')
    expect(caps.capabilities).toHaveLength(2)
    expect(caps.maxComplexity).toBe(5)
    expect(caps.supportsStreaming).toBe(true)
  })

  it('accepts empty capabilities array', () => {
    const caps: WorkerCapabilities = {
      workerId: 'minimal',
      workerType: 'generic',
      capabilities: [],
      maxComplexity: 1,
      maxConcurrent: 1,
      supportsStreaming: false,
      supportsCancel: false,
      defaultTimeoutMs: 30000,
    }

    expect(caps.capabilities).toEqual([])
  })
})

describe('TaskDescription types', () => {
  it('validates TaskDescription structure', () => {
    const task: TaskDescription = {
      taskId: 'task-001',
      prompt: 'Write a test for this function',
      workingDir: '/tmp/project',
      context: { file: 'src/lib.ts' },
      timeoutMs: 60000,
      priority: 1,
      expectedFormat: 'code',
    }

    expect(task.taskId).toBe('task-001')
    expect(task.prompt).toContain('test')
    expect(task.timeoutMs).toBe(60000)
    expect(task.expectedFormat).toBe('code')
  })

  it('accepts all expectedFormat values', () => {
    const formats = ['structured', 'markdown', 'diff', 'text', 'code']

    formats.forEach((format) => {
      const task: TaskDescription = {
        taskId: `task-${format}`,
        prompt: 'Test',
        context: {},
        timeoutMs: 30000,
        priority: 1,
        expectedFormat: format as any,
      }

      expect(task.expectedFormat).toBe(format)
    })
  })
})

describe('TaskResult types', () => {
  it('validates successful TaskResult', () => {
    const result: TaskResult = {
      taskId: 'task-001',
      output: 'Task completed successfully',
      success: true,
      tokensUsed: 500,
    }

    expect(result.success).toBe(true)
    expect(result.output).toContain('successfully')
    expect(result.error).toBeUndefined()
  })

  it('validates failed TaskResult', () => {
    const result: TaskResult = {
      taskId: 'task-002',
      output: '',
      success: false,
      error: 'Execution failed: timeout',
    }

    expect(result.success).toBe(false)
    expect(result.error).toContain('timeout')
  })
})

describe('WorkerStatus types', () => {
  it('validates healthy WorkerStatus', () => {
    const status: WorkerStatus = {
      workerId: 'worker-001',
      health: 'healthy',
      currentTask: undefined,
      tasksCompleted: 10,
      tasksFailed: 0,
    }

    expect(status.health).toBe('healthy')
    expect(status.tasksCompleted).toBe(10)
    expect(status.currentTask).toBeUndefined()
  })

  it('validates busy WorkerStatus', () => {
    const status: WorkerStatus = {
      workerId: 'worker-002',
      health: 'busy',
      currentTask: 'task-123',
      tasksCompleted: 5,
      tasksFailed: 1,
    }

    expect(status.health).toBe('busy')
    expect(status.currentTask).toBe('task-123')
  })

  it('validates offline WorkerStatus', () => {
    const status: WorkerStatus = {
      workerId: 'worker-003',
      health: 'offline',
      currentTask: undefined,
      tasksCompleted: 0,
      tasksFailed: 0,
    }

    expect(status.health).toBe('offline')
  })
})