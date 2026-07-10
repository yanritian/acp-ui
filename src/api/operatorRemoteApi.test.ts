import { describe, expect, it, vi } from 'vitest'

import { createOperatorRemoteApi } from './operatorRemoteApi'

describe('Operator Remote API security', () => {
  it('injects bearer and client headers and reads the audit endpoint', async () => {
    const fetcher = vi.fn(async (
      _input: RequestInfo | URL,
      _init?: RequestInit,
    ): Promise<Response> => new Response(JSON.stringify([]), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }))
    const client = createOperatorRemoteApi({
      baseUrl: 'http://192.168.1.20:1422/',
      token: 'secret-canary',
      clientId: 'vscode-main',
      fetcher,
    })

    await client.getAudit(25)

    expect(fetcher).toHaveBeenCalledOnce()
    const [url, init] = fetcher.mock.calls[0]
    const headers = new Headers(init?.headers)
    expect(url).toBe('http://192.168.1.20:1422/api/operator/audit?limit=25')
    expect(headers.get('Authorization')).toBe('Bearer secret-canary')
    expect(headers.get('X-ACP-Operator-Client')).toBe('vscode-main')
    expect(client.eventsWebSocketUrl).toBe('ws://192.168.1.20:1422/ws/events')
    expect(client.eventsWebSocketUrl).not.toContain('secret-canary')
  })

  it('keeps the local default client credential-free', async () => {
    const fetcher = vi.fn(async (
      _input: RequestInfo | URL,
      _init?: RequestInit,
    ): Promise<Response> => new Response(JSON.stringify([]), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }))
    const client = createOperatorRemoteApi({ fetcher })

    await client.getPlatforms()

    const [, init] = fetcher.mock.calls[0]
    const headers = new Headers(init?.headers)
    expect(headers.has('Authorization')).toBe(false)
    expect(headers.has('X-ACP-Operator-Client')).toBe(false)
  })
})
