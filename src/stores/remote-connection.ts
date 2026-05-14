import { ref, computed } from 'vue';
import { defineStore } from 'pinia';

interface RemoteMessage {
  id: string;
  type: string;
  command?: string;
  ok?: boolean;
  data?: unknown;
  error?: string;
}

interface LogEntry {
  id: string;
  agentId: string;
  logType: string;
  content: string;
  timestamp: string;
  source: string;
  metadata?: string;
}

export const useRemoteConnectionStore = defineStore('remote-connection', () => {
  // Connection state
  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const isAuthenticated = ref(false);
  const connectionUrl = ref('');
  const reconnectAttempts = ref(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 3000;

  // Message handling
  const requestId = ref(1);
  const pendingRequests = ref<Map<string, { resolve: (value: unknown) => void; reject: (error: string) => void }>>(new Map());

  // Log stream state
  const subscribedToLogs = ref(false);
  const logBuffer = ref<LogEntry[]>([]);
  const logsPaused = ref(false);
  const maxLogBufferSize = 500;

  // Agent state from remote
  const remoteAgents = ref<Array<{ name: string; transport: { type: string } }>>([]);
  const remoteStatus = ref<{
    agentsConfigured: number;
    runningTasks: number;
    totalTasks: number;
    wsServerRunning: boolean;
    connectedClients: number;
  } | null>(null);

  // Error handling
  const lastError = ref<string | null>(null);

  // Computed
  const hasConnection = computed(() => isConnected.value && ws.value !== null);
  const canSubscribeLogs = computed(() => hasConnection.value && isAuthenticated.value);
  const logCount = computed(() => logBuffer.value.length);

  // Connect to remote WebSocket server
  function connect(url: string): Promise<void> {
    return new Promise((resolve, reject) => {
      if (ws.value) {
        disconnect();
      }

      connectionUrl.value = url;
      lastError.value = null;

      try {
        ws.value = new WebSocket(url);

        ws.value.onopen = () => {
          isConnected.value = true;
          reconnectAttempts.value = 0;
          console.log('Remote WebSocket connected');
          resolve();
        };

        ws.value.onmessage = (event) => {
          handleMessage(event.data);
        };

        ws.value.onerror = (error) => {
          lastError.value = 'WebSocket error';
          console.error('WebSocket error:', error);
          reject('WebSocket connection error');
        };

        ws.value.onclose = (event) => {
          isConnected.value = false;
          isAuthenticated.value = false;
          subscribedToLogs.value = false;
          console.log('WebSocket closed:', event.code, event.reason);

          // Attempt reconnect if not intentional close
          if (event.code !== 1000 && reconnectAttempts.value < maxReconnectAttempts) {
            scheduleReconnect();
          }
        };
      } catch (e) {
        lastError.value = String(e);
        reject(String(e));
      }
    });
  }

  // Schedule reconnect attempt
  function scheduleReconnect() {
    reconnectAttempts.value++;
    console.log(`Scheduling reconnect attempt ${reconnectAttempts.value}/${maxReconnectAttempts}`);
    setTimeout(() => {
      if (connectionUrl.value && !isConnected.value) {
        connect(connectionUrl.value).catch(console.error);
      }
    }, reconnectDelay);
  }

  // Disconnect from server
  function disconnect() {
    if (ws.value) {
      ws.value.close(1000, 'User disconnect');
      ws.value = null;
    }
    isConnected.value = false;
    isAuthenticated.value = false;
    subscribedToLogs.value = false;
    reconnectAttempts.value = maxReconnectAttempts; // Prevent auto-reconnect
  }

  // Handle incoming message
  function handleMessage(data: string) {
    try {
      const message: RemoteMessage = JSON.parse(data);

      // Handle log batch
      if (message.type === 'log_batch') {
        handleLogBatch((message.data as { logs: LogEntry[] }).logs);
        return;
      }

      // Handle auth response
      if (message.ok && message.data && (message.data as { client_id?: string }).client_id) {
        isAuthenticated.value = true;
        console.log('Authenticated with client ID:', (message.data as { client_id: string }).client_id);
      }

      // Handle response to pending request
      if (message.id && pendingRequests.value.has(message.id)) {
        const handler = pendingRequests.value.get(message.id)!;
        if (message.ok) {
          handler.resolve(message.data);
        } else {
          handler.reject(message.error || 'Unknown error');
        }
        pendingRequests.value.delete(message.id);
      }
    } catch (e) {
      console.error('Failed to parse message:', e);
    }
  }

  // Send request to server
  function sendRequest(command: string, payload?: unknown): Promise<unknown> {
    return new Promise((resolve, reject) => {
      if (!ws.value || !isConnected.value) {
        reject('Not connected');
        return;
      }

      const id = `req-${requestId.value++}`;
      const request = {
        id,
        type: 'request',
        command,
        payload: payload || {}
      };

      ws.value.send(JSON.stringify(request));
      pendingRequests.value.set(id, { resolve, reject });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (pendingRequests.value.has(id)) {
          pendingRequests.value.delete(id);
          reject('Request timeout');
        }
      }, 30000);
    });
  }

  // Get status from remote
  async function getStatus() {
    try {
      const result = await sendRequest('get_status');
      remoteStatus.value = result as typeof remoteStatus.value;
      return result;
    } catch (e) {
      lastError.value = String(e);
      throw e;
    }
  }

  // List agents from remote
  async function listAgents() {
    try {
      const result = await sendRequest('list_agents') as { agents: typeof remoteAgents.value };
      remoteAgents.value = result.agents || [];
      return remoteAgents.value;
    } catch (e) {
      lastError.value = String(e);
      throw e;
    }
  }

  // Subscribe to log stream
  async function subscribeLogs(agentId?: string) {
    try {
      await sendRequest('subscribe_logs', { agent_id: agentId || null });
      subscribedToLogs.value = true;
    } catch (e) {
      lastError.value = String(e);
      throw e;
    }
  }

  // Unsubscribe from log stream
  async function unsubscribeLogs(agentId?: string) {
    try {
      await sendRequest('unsubscribe_logs', { agent_id: agentId || null });
      subscribedToLogs.value = false;
    } catch (e) {
      lastError.value = String(e);
      throw e;
    }
  }

  // Handle incoming log batch
  function handleLogBatch(logs: LogEntry[]) {
    if (!logsPaused.value) {
      logBuffer.value.push(...logs);
      // Keep buffer limited
      if (logBuffer.value.length > maxLogBufferSize) {
        logBuffer.value = logBuffer.value.slice(-maxLogBufferSize);
      }
    }
  }

  // Pause/resume log stream
  function togglePause() {
    logsPaused.value = !logsPaused.value;
  }

  // Clear log buffer
  function clearLogs() {
    logBuffer.value = [];
  }

  // Filter logs by type
  function getLogsByType(type: string): LogEntry[] {
    return logBuffer.value.filter(log => log.logType === type);
  }

  // Filter logs by agent
  function getLogsByAgent(agentId: string): LogEntry[] {
    return logBuffer.value.filter(log => log.agentId === agentId);
  }

  // Search logs
  function searchLogs(keyword: string): LogEntry[] {
    const lower = keyword.toLowerCase();
    return logBuffer.value.filter(log => log.content.toLowerCase().includes(lower));
  }

  // Pause/Resume agent
  async function pauseAgent(agentId: string) {
    return sendRequest('pause_agent', { agent_id: agentId });
  }

  async function resumeAgent(agentId: string) {
    return sendRequest('resume_agent', { agent_id: agentId });
  }

  // Inject message to agent
  async function injectMessage(agentId: string, message: string) {
    return sendRequest('inject_message', { agent_id: agentId, message });
  }

  return {
    // State
    isConnected,
    isAuthenticated,
    connectionUrl,
    reconnectAttempts,
    subscribedToLogs,
    logBuffer,
    logsPaused,
    remoteAgents,
    remoteStatus,
    lastError,

    // Computed
    hasConnection,
    canSubscribeLogs,
    logCount,

    // Actions
    connect,
    disconnect,
    sendRequest,
    getStatus,
    listAgents,
    subscribeLogs,
    unsubscribeLogs,
    togglePause,
    clearLogs,
    getLogsByType,
    getLogsByAgent,
    searchLogs,
    pauseAgent,
    resumeAgent,
    injectMessage
  };
});