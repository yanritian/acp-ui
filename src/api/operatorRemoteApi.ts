// Remote Operator API
// Shared REST client for web, VSCode, IDEA, mobile, and other remote consoles.

import type {
  ApprovalRequest,
  ApproveRequest,
  OperatorEvent,
  OperatorTask,
  RemoteAuditRecord,
  RemoteCommandResponse,
  RemotePlatformCapability,
  RemoteRedirectRequest,
  StartTaskRequest,
  StartTaskResponse,
  TaskSummary,
} from '@/types/operator'

export const DEFAULT_REMOTE_OPERATOR_BASE_URL = 'http://127.0.0.1:1422'

export interface OperatorRemoteApiOptions {
  baseUrl?: string
  token?: string
  clientId?: string
  headers?: HeadersInit
  fetcher?: typeof fetch
}

export interface OperatorRemoteApiClient {
  readonly baseUrl: string
  readonly eventsWebSocketUrl: string
  getPlatforms(): Promise<RemotePlatformCapability[]>
  startTask(request: StartTaskRequest): Promise<StartTaskResponse>
  listTasks(): Promise<OperatorTask[]>
  getTask(taskId: string): Promise<OperatorTask>
  pauseTask(taskId: string, expectedRevision?: number): Promise<RemoteCommandResponse>
  resumeTask(taskId: string, expectedRevision?: number): Promise<RemoteCommandResponse>
  stopTask(taskId: string, expectedRevision?: number): Promise<RemoteCommandResponse>
  redirectTask(taskId: string, request: RemoteRedirectRequest): Promise<RemoteCommandResponse>
  approve(request: ApproveRequest): Promise<RemoteCommandResponse>
  getPendingApprovals(taskId: string): Promise<ApprovalRequest[]>
  listEvents(taskId: string, limit?: number, afterSequence?: number): Promise<OperatorEvent[]>
  getTaskSummary(taskId: string): Promise<TaskSummary>
  getAudit(limit?: number): Promise<RemoteAuditRecord[]>
}

interface ApiErrorBody {
  error?: string
  code?: number
}

export class OperatorRemoteApiError extends Error {
  readonly status: number
  readonly code?: number

  constructor(message: string, status: number, code?: number) {
    super(message)
    this.name = 'OperatorRemoteApiError'
    this.status = status
    this.code = code
  }
}

export function buildOperatorEventsWebSocketUrl(
  baseUrl: string = DEFAULT_REMOTE_OPERATOR_BASE_URL
): string {
  const normalized = normalizeBaseUrl(baseUrl)
  const wsBase = normalized
    .replace(/^https:/i, 'wss:')
    .replace(/^http:/i, 'ws:')
  return `${wsBase}/ws/events`
}

export function createOperatorRemoteApi(
  options: OperatorRemoteApiOptions = {}
): OperatorRemoteApiClient {
  const baseUrl = normalizeBaseUrl(options.baseUrl ?? DEFAULT_REMOTE_OPERATOR_BASE_URL)
  const fetcher = options.fetcher ?? fetch
  const securityHeaders: Record<string, string> = {
    ...(options.clientId ? { 'X-ACP-Operator-Client': options.clientId } : {}),
    ...(options.token ? { Authorization: `Bearer ${options.token}` } : {}),
  }

  async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const response = await fetcher(`${baseUrl}${path}`, {
      ...init,
      headers: buildHeaders(
        { ...headersToRecord(options.headers), ...securityHeaders },
        init.headers,
        init.body !== undefined
      ),
    })

    if (!response.ok) {
      throw await toApiError(response)
    }

    if (response.status === 204) {
      return undefined as T
    }

    return await response.json() as T
  }

  function postJson<T>(path: string, body?: unknown): Promise<T> {
    return request<T>(path, {
      method: 'POST',
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  }

  return {
    baseUrl,
    eventsWebSocketUrl: buildOperatorEventsWebSocketUrl(baseUrl),

    getPlatforms() {
      return request<RemotePlatformCapability[]>('/api/operator/platforms')
    },

    startTask(startRequest) {
      return postJson<StartTaskResponse>('/api/operator/tasks', startRequest)
    },

    listTasks() {
      return request<OperatorTask[]>('/api/operator/tasks')
    },

    getTask(taskId) {
      return request<OperatorTask>(`/api/operator/tasks/${encodePath(taskId)}`)
    },

    pauseTask(taskId, expectedRevision) {
      return postJson<RemoteCommandResponse>(
        `/api/operator/tasks/${encodePath(taskId)}/pause`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision }
      )
    },

    resumeTask(taskId, expectedRevision) {
      return postJson<RemoteCommandResponse>(
        `/api/operator/tasks/${encodePath(taskId)}/resume`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision }
      )
    },

    stopTask(taskId, expectedRevision) {
      return postJson<RemoteCommandResponse>(
        `/api/operator/tasks/${encodePath(taskId)}/stop`,
        expectedRevision === undefined ? undefined : { expected_revision: expectedRevision }
      )
    },

    redirectTask(taskId, redirectRequest) {
      return postJson<RemoteCommandResponse>(
        `/api/operator/tasks/${encodePath(taskId)}/redirect`,
        redirectRequest
      )
    },

    approve(approvalRequest) {
      return postJson<RemoteCommandResponse>('/api/operator/approvals/decision', approvalRequest)
    },

    getPendingApprovals(taskId) {
      return request<ApprovalRequest[]>(`/api/operator/tasks/${encodePath(taskId)}/approvals`)
    },

    listEvents(taskId, limit, afterSequence) {
      const params = new URLSearchParams()
      if (limit !== undefined) params.set('limit', String(limit))
      if (afterSequence !== undefined) params.set('after_sequence', String(afterSequence))
      const query = params.toString() ? `?${params.toString()}` : ''
      return request<OperatorEvent[]>(`/api/operator/tasks/${encodePath(taskId)}/events${query}`)
    },

    getTaskSummary(taskId) {
      return request<TaskSummary>(`/api/operator/tasks/${encodePath(taskId)}/summary`)
    },

    getAudit(limit) {
      const query = limit === undefined ? '' : `?limit=${encodeURIComponent(String(limit))}`
      return request<RemoteAuditRecord[]>(`/api/operator/audit${query}`)
    },
  }
}

export const OperatorRemoteApi = createOperatorRemoteApi()

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, '')
}

function encodePath(value: string): string {
  return encodeURIComponent(value)
}

function buildHeaders(
  defaultHeaders: HeadersInit | undefined,
  requestHeaders: HeadersInit | undefined,
  hasBody: boolean
): HeadersInit {
  return {
    Accept: 'application/json',
    ...(hasBody ? { 'Content-Type': 'application/json' } : {}),
    ...headersToRecord(defaultHeaders),
    ...headersToRecord(requestHeaders),
  }
}

function headersToRecord(headers: HeadersInit | undefined): Record<string, string> {
  if (!headers) {
    return {}
  }

  if (headers instanceof Headers) {
    return Object.fromEntries(headers.entries())
  }

  if (Array.isArray(headers)) {
    return Object.fromEntries(headers)
  }

  return headers
}

async function toApiError(response: Response): Promise<OperatorRemoteApiError> {
  let body: ApiErrorBody | undefined

  try {
    body = await response.json() as ApiErrorBody
  } catch {
    body = undefined
  }

  return new OperatorRemoteApiError(
    body?.error ?? `Remote Operator request failed with HTTP ${response.status}`,
    response.status,
    body?.code
  )
}
