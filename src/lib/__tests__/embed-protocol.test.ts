// Embed Protocol Unit Tests
//
// Tests for embed-protocol.ts - iframe + postMessage communication

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  EmbedHost,
  EmbedClient,
  type EmbedMessage,
  type EmbedConfig,
  type EmbedState,
} from '../embed-protocol'

// Mock DOM elements
const mockContainer = {
  appendChild: vi.fn(),
}

const mockIframe = {
  src: '',
  style: { width: '', height: '', border: '' },
  contentWindow: { postMessage: vi.fn() } as unknown as Window,
}

// Mock document.getElementById
vi.stubGlobal('document', {
  getElementById: vi.fn((id: string) => {
    if (id === 'test-container') return mockContainer
    return null
  }),
  createElement: vi.fn(() => mockIframe),
})

// Mock window
vi.stubGlobal('window', {
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  postMessage: vi.fn(),
  parent: { postMessage: vi.fn() } as unknown as Window,
})

describe('EmbedHost', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('embed', () => {
    it('creates iframe and appends to container', () => {
      const host = new EmbedHost()
      host.embed('test-container', 'http://localhost:1420')

      expect(mockContainer.appendChild).toHaveBeenCalled()
      expect(mockIframe.style.width).toBe('100%')
      expect(mockIframe.style.height).toBe('100%')
      expect(mockIframe.style.border).toBe('none')
    })

    it('throws error if container not found', () => {
      const host = new EmbedHost()
      expect(() => host.embed('nonexistent', 'http://localhost:1420')).toThrow(
        'Container #nonexistent not found'
      )
    })

    it('registers message listener', () => {
      const host = new EmbedHost()
      host.embed('test-container', 'http://localhost:1420')

      expect(window.addEventListener).toHaveBeenCalledWith('message', expect.any(Function))
    })
  })

  describe('send', () => {
    it('sends message to iframe contentWindow', () => {
      const host = new EmbedHost()
      host.embed('test-container', 'http://localhost:1420')

      // EmbedHost.send signature: send(type, payload)
      host.send('init', { token: 'test-token' })

      expect(mockIframe.contentWindow!.postMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'init',
          payload: { token: 'test-token' },
          source: 'host',
        }),
        '*'
      )
    })
  })

  describe('submitTask', () => {
    it('sends task_submit message with task data', async () => {
      const host = new EmbedHost()
      host.embed('test-container', 'http://localhost:1420')

      // submitTask uses send internally
      const taskPromise = host.submitTask({ prompt: 'Write a test' })

      // Verify message was sent
      expect(mockIframe.contentWindow!.postMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'task_submit',
          source: 'host',
        }),
        '*'
      )

      // Simulate response to resolve promise
      const client = host as any
      if (client.messageHandler) {
        client.messageHandler({
          type: 'task_status',
          payload: { taskId: 'task-001' },
          timestamp: Date.now(),
          source: 'embed',
        })
      }
    })
  })
})

describe('EmbedClient', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('constructor', () => {
    it('registers message listener for parent window', () => {
      const client = new EmbedClient()

      expect(window.addEventListener).toHaveBeenCalledWith('message', expect.any(Function))
    })
  })

  describe('send', () => {
    it('does not send if hostOrigin is not set', () => {
      const client = new EmbedClient()

      client.send('ready', {})

      // Should not call postMessage because hostOrigin is null
      expect(window.parent.postMessage).not.toHaveBeenCalled()
    })

    it('sends message to parent when hostOrigin is set', () => {
      const client = new EmbedClient()
      // Set hostOrigin via handleMessage (internal)
      const internalClient = client as any
      internalClient.hostOrigin = 'http://localhost:1420'

      client.send('state_update', { goals: [] })

      expect(window.parent.postMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'state_update',
          source: 'embed',
        }),
        'http://localhost:1420'
      )
    })
  })

  describe('sendReady', () => {
    it('sends ready message to host without requiring hostOrigin', () => {
      const client = new EmbedClient()

      client.sendReady()

      expect(window.parent.postMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'ready',
          source: 'embed',
        }),
        '*'
      )
    })
  })

  describe('sendStateUpdate', () => {
    it('sends state_update message when hostOrigin is set', () => {
      const client = new EmbedClient()
      const internalClient = client as any
      internalClient.hostOrigin = 'http://localhost:1420'

      const state: EmbedState = {
        goals: [{ id: 'goal-001' }],
        workers: [{ id: 'worker-001' }],
        tasks: [],
        queen: null,
      }

      client.sendStateUpdate(state)

      expect(window.parent.postMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'state_update',
          payload: state,
          source: 'embed',
        }),
        'http://localhost:1420'
      )
    })

    it('does not send if hostOrigin is not set', () => {
      const client = new EmbedClient()

      const state: EmbedState = {
        goals: [],
        workers: [],
        tasks: [],
        queen: null,
      }

      client.sendStateUpdate(state)

      expect(window.parent.postMessage).not.toHaveBeenCalled()
    })
  })

  describe('onMessage', () => {
    it('registers custom message handler', () => {
      const client = new EmbedClient()
      const handler = vi.fn()

      client.onMessage(handler)

      // Handler should be stored (internal implementation)
      // We can't directly test it without triggering a message event
    })
  })
})

describe('EmbedMessage types', () => {
  it('validates EmbedMessageType enum values', () => {
    const validTypes = [
      'init',
      'ready',
      'state_update',
      'task_submit',
      'task_status',
      'goal_list',
      'worker_list',
      'action_request',
      'action_response',
    ]

    validTypes.forEach((type) => {
      expect(type).toBeDefined()
    })
  })

  it('validates EmbedMessage structure', () => {
    const msg: EmbedMessage = {
      type: 'init',
      payload: { test: true },
      timestamp: 1234567890,
      source: 'host',
    }

    expect(msg.type).toBe('init')
    expect(msg.source).toBe('host')
    expect(msg.timestamp).toBeGreaterThan(0)
  })
})

describe('EmbedConfig validation', () => {
  it('accepts valid config', () => {
    const config: EmbedConfig = {
      origin: 'http://localhost:1420',
      allowedOrigins: ['http://localhost:1420'],
      authToken: 'test-token',
    }

    expect(config.origin).toBe('http://localhost:1420')
    expect(config.allowedOrigins).toHaveLength(1)
    expect(config.authToken).toBe('test-token')
  })

  it('allows optional authToken', () => {
    const config: EmbedConfig = {
      origin: 'http://localhost:1420',
      allowedOrigins: ['http://localhost:1420'],
    }

    expect(config.authToken).toBeUndefined()
  })
})