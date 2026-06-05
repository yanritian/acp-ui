/**
 * Session Lifecycle Store
 *
 * Manages session creation, resumption, disconnection, and auto-reconnection.
 * This is the core store that coordinates with other session stores.
 */
import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { loadKvStore, type KVStore } from '../lib/host/storage';
import { getAppVersion } from '../lib/host';
import { trackEvent, trackError } from '../lib/telemetry';
import type { SavedSession, AgentConfig } from '../lib/types';
import { getTransportKind } from '../lib/types';
import { AcpClientBridge, createAcpClient } from '../lib/acp-bridge';
import { onAgentStderr, spawnAgent, killAgent } from '../lib/host';
import { isDesktop } from '../lib/platform';
import { useConfigStore } from './config';
import type { SessionNotification, AuthMethod, NewSessionResponse } from '@agentclientprotocol/sdk';

const STORE_PATH = 'sessions.json';
const PROTOCOL_VERSION = 1;

// Auto reconnect configuration
const AUTO_RECONNECT_CONFIG = {
  enabled: true,
  maxRetries: 5,
  baseDelayMs: 1000,    // 1s, 2s, 4s, 8s, 16s
  maxDelayMs: 30000,
};

// App version (loaded once at startup)
let appVersion = '0.1.0';

// Startup phase detection patterns
function detectPhase(line: string): string | null {
  const lower = line.toLowerCase();
  if (lower.includes('download') || lower.includes('fetch') || lower.includes('get ')) {
    return 'downloading';
  }
  if (lower.includes('install') || lower.includes('added') || lower.includes('packages')) {
    return 'installing';
  }
  if (lower.includes('build') || lower.includes('compil')) {
    return 'building';
  }
  if (lower.includes('start') || lower.includes('spawn')) {
    return 'starting';
  }
  return null;
}

// Types for cross-store communication
export interface SessionLifecycleState {
  savedSessions: SavedSession[];
  currentSession: SavedSession | null;
  isConnected: boolean;
  isLoading: boolean;
  isConnecting: boolean;
  isReconnecting: boolean;
  error: string | null;
  startupPhase: string;
  startupLogs: string[];
  startupElapsed: number;
  acpClient: AcpClientBridge | null;
}

export const useSessionLifecycleStore = defineStore('sessionLifecycle', () => {
  // State
  const savedSessions = ref<SavedSession[]>([]);
  const currentSession = ref<SavedSession | null>(null);
  const isConnected = ref(false);
  const isLoading = ref(false);
  const isConnecting = ref(false);
  const isReconnecting = ref(false);
  const error = ref<string | null>(null);

  // Startup progress tracking
  const startupPhase = ref<string>('starting');
  const startupLogs = ref<string[]>([]);
  const startupElapsed = ref<number>(0);

  // Internal state
  let startupTimer: ReturnType<typeof setInterval> | null = null;
  let stderrUnlisten: (() => void) | null = null;
  let reconnectAttemptCount = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let connectionAborted = false;

  // Current ACP client
  let acpClient: AcpClientBridge | null = null;
  let store: KVStore | null = null;

  // External store references (will be set by session.ts)
  let sessionUpdateHandler: ((notification: SessionNotification) => void) | null = null;
  let permissionWatchSetup: ((client: AcpClientBridge) => (() => void) | null) | null = null;
  let messageWatchSetup: (() => void) | null = null;
  let messageSaveHandler: (() => void) | null = null;
  let authMethodHandler: ((methods: AuthMethod[], agentName: string) => Promise<string | null>) | null = null;

  // Computed
  const hasActiveSession = computed(() => currentSession.value !== null);
  const resumableSessions = computed(() =>
    savedSessions.value.filter(s => s.supportsLoadSession === true)
  );

  // Getters for external access
  const getAcpClient = () => acpClient;

  // Set external handlers (called by session.ts to wire up dependencies)
  function setSessionUpdateHandler(handler: (notification: SessionNotification) => void) {
    sessionUpdateHandler = handler;
  }

  function setPermissionWatchSetup(setup: (client: AcpClientBridge) => (() => void) | null) {
    permissionWatchSetup = setup;
  }

  function setMessageWatchSetup(setup: () => void) {
    messageWatchSetup = setup;
  }

  function setMessageSaveHandler(handler: () => void) {
    messageSaveHandler = handler;
  }

  function setAuthMethodHandler(handler: (methods: AuthMethod[], agentName: string) => Promise<string | null>) {
    authMethodHandler = handler;
  }

  // Initialize store
  async function initStore() {
    store = await loadKvStore(STORE_PATH);
    const saved = await store.get<SavedSession[]>('sessions');
    if (saved) {
      savedSessions.value = saved;
    }

    // Load app version (Tauri API on desktop/mobile, build-time inject on web)
    try {
      appVersion = await getAppVersion();
    } catch (e) {
      console.warn('Failed to get app version:', e);
    }

    // Set up beforeunload handler for emergency message save
    if (typeof window !== 'undefined') {
      window.addEventListener('beforeunload', () => {
        if (messageSaveHandler) {
          messageSaveHandler();
        }
      });

      // Auto reconnect triggers
      window.addEventListener('online', () => {
        if (!isConnected.value && currentSession.value?.supportsLoadSession) {
          cancelAutoReconnect();
          reconnectAttemptCount = 0;
          tryReconnect().catch(e => console.warn('Online reconnect failed:', e));
        }
      });

      window.addEventListener('focus', () => {
        if (!isConnected.value && !isReconnecting.value && currentSession.value?.supportsLoadSession) {
          cancelAutoReconnect();
          reconnectAttemptCount = 0;
          tryReconnect().catch(e => console.warn('Focus reconnect failed:', e));
        }
      });
    }
  }

  async function saveSessionsToStore() {
    if (store) {
      await store.set('sessions', savedSessions.value);
      await store.save();
    }
  }

  // Handle an unexpected transport close
  function handleUnexpectedClose(reason?: string): void {
    if (!acpClient) return;
    acpClient = null;
    isConnected.value = false;
    isLoading.value = false;
    error.value = `Connection lost: ${reason ?? 'transport closed'}`;

    if (AUTO_RECONNECT_CONFIG.enabled && currentSession.value?.supportsLoadSession) {
      scheduleAutoReconnect();
    }
  }

  // Schedule auto reconnect with exponential backoff
  function scheduleAutoReconnect(): void {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }

    if (reconnectAttemptCount >= AUTO_RECONNECT_CONFIG.maxRetries) {
      error.value = `Connection lost. Auto reconnect failed after ${AUTO_RECONNECT_CONFIG.maxRetries} attempts. Click reconnect to try manually.`;
      reconnectAttemptCount = 0;
      return;
    }

    const delay = Math.min(
      AUTO_RECONNECT_CONFIG.baseDelayMs * Math.pow(2, reconnectAttemptCount),
      AUTO_RECONNECT_CONFIG.maxDelayMs
    );
    reconnectAttemptCount++;

    error.value = `Connection lost. Reconnecting in ${Math.round(delay / 1000)}s (attempt ${reconnectAttemptCount}/${AUTO_RECONNECT_CONFIG.maxRetries})...`;

    reconnectTimer = setTimeout(async () => {
      reconnectTimer = null;
      const success = await tryReconnect();
      if (!success) {
        if (!isConnected.value && currentSession.value?.supportsLoadSession) {
          scheduleAutoReconnect();
        }
      } else {
        if (isConnected.value) {
          reconnectAttemptCount = 0;
        } else {
          scheduleAutoReconnect();
        }
      }
    }, delay);
  }

  // Cancel any pending auto reconnect
  function cancelAutoReconnect(): void {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    reconnectAttemptCount = 0;
  }

  // Create new session
  async function createSession(agentName: string, cwd: string): Promise<NewSessionResponse | undefined> {
    isLoading.value = true;
    isConnecting.value = true;
    connectionAborted = false;
    error.value = null;

    const configStore = useConfigStore();
    const agentConfig: AgentConfig | undefined = configStore.getAgent(agentName);
    const transportKind = agentConfig
      ? getTransportKind(agentConfig)
      : 'stdio';
    const isRemote = transportKind !== 'stdio';

    // Reset and start progress tracking
    startupPhase.value = 'starting';
    startupLogs.value = [];
    startupElapsed.value = 0;
    startupTimer = setInterval(() => {
      startupElapsed.value++;
    }, 1000);

    let spawnedInstance: { id: string } | null = null;

    try {
      if (!agentConfig) {
        throw new Error(`Agent '${agentName}' not found in config`);
      }

      if (!isRemote) {
        startupPhase.value = 'starting';
        const agentInstance = await spawnAgent(agentName);
        spawnedInstance = agentInstance;

        stderrUnlisten = await onAgentStderr((stderr) => {
          if (stderr.agent_id !== agentInstance.id) return;
          startupLogs.value.push(stderr.line);
          const detectedPhase = detectPhase(stderr.line);
          if (detectedPhase) {
            startupPhase.value = detectedPhase;
          }
        });

        if (connectionAborted) {
          await killAgent(agentInstance.id).catch((err) =>
            console.warn('killAgent during abort failed:', err)
          );
          spawnedInstance = null;
          throw new Error('Connection cancelled');
        }

        startupPhase.value = 'initializing';
        acpClient = await createAcpClient(agentInstance);
        spawnedInstance = null;
      } else {
        startupPhase.value = 'connecting';

        if (connectionAborted) {
          throw new Error('Connection cancelled');
        }

        acpClient = await createAcpClient({ name: agentName, config: agentConfig });
      }

      // Wire up session update handler
      if (sessionUpdateHandler) {
        acpClient.onSessionUpdate = sessionUpdateHandler;
      }
      acpClient.onTransportClose = (reason) => {
        handleUnexpectedClose(reason);
      };

      // Wire up permission watch
      if (permissionWatchSetup) {
        permissionWatchSetup(acpClient);
      }

      if (connectionAborted) {
        await acpClient.disconnect();
        throw new Error('Connection cancelled');
      }

      startupPhase.value = 'connecting';

      const canAccessFs = isDesktop();

      const initResponse = await acpClient.initialize({
        protocolVersion: PROTOCOL_VERSION,
        clientCapabilities: {
          fs: {
            readTextFile: canAccessFs,
            writeTextFile: canAccessFs,
          },
        },
        clientInfo: {
          name: 'acp-ui',
          title: 'ACP UI',
          version: appVersion,
        },
      });

      console.log('Agent initialized:', initResponse);

      const supportsLoadSession = initResponse.agentCapabilities?.loadSession ?? false;

      if (connectionAborted) {
        await acpClient.disconnect();
        throw new Error('Connection cancelled');
      }

      const availableAuthMethods = initResponse.authMethods || [];

      if (connectionAborted) {
        await acpClient.disconnect();
        throw new Error('Connection cancelled');
      }

      // Try to create session - may fail with auth_required
      let sessionResponse;
      try {
        sessionResponse = await acpClient.newSession({
          cwd,
          mcpServers: [],
        });
      } catch (sessionError: unknown) {
        const errorMessage = sessionError instanceof Error ? sessionError.message : String(sessionError);
        const isAuthRequired = errorMessage.toLowerCase().includes('authentication required') ||
                               errorMessage.includes('-32000');

        if (isAuthRequired && availableAuthMethods.length > 0) {
          console.log('Authentication required, available methods:', availableAuthMethods);

          let selectedMethodId: string | null = null;
          if (authMethodHandler) {
            selectedMethodId = await authMethodHandler(availableAuthMethods, agentName);
          }

          if (!selectedMethodId || connectionAborted) {
            await acpClient.disconnect();
            throw new Error('Authentication cancelled by user');
          }

          console.log('Authenticating with method:', selectedMethodId);
          const authResponse = await acpClient.authenticate({
            methodId: selectedMethodId,
          });
          console.log('Authentication successful:', authResponse);

          if (connectionAborted) {
            await acpClient.disconnect();
            throw new Error('Connection cancelled');
          }

          sessionResponse = await acpClient.newSession({
            cwd,
            mcpServers: [],
          });
        } else {
          throw sessionError;
        }
      }

      // Save session
      const session: SavedSession = {
        id: crypto.randomUUID(),
        agentName,
        sessionId: sessionResponse.sessionId,
        title: `Session ${new Date().toLocaleString()}`,
        lastUpdated: Date.now(),
        cwd,
        supportsLoadSession,
      };

      currentSession.value = session;
      savedSessions.value.push(session);
      await saveSessionsToStore();

      isConnected.value = true;

      // Wire up message persistence watch
      if (messageWatchSetup) {
        messageWatchSetup();
      }

      trackEvent('SessionCreated', { agentName, success: 'true' });

      // Return session response for capabilities store to use
      return sessionResponse;

    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      if (acpClient) {
        try {
          await acpClient.disconnect();
        } catch (cleanupErr) {
          console.warn('disconnect during createSession cleanup failed:', cleanupErr);
        }
      } else if (spawnedInstance) {
        try {
          await killAgent(spawnedInstance.id);
        } catch (cleanupErr) {
          console.warn('killAgent during createSession cleanup failed:', cleanupErr);
        }
      }
      acpClient = null;
      trackEvent('SessionCreated', { agentName, success: 'false' });
      trackError(e instanceof Error ? e : new Error(String(e)));
      throw e;
    } finally {
      isLoading.value = false;
      isConnecting.value = false;
      if (startupTimer) {
        clearInterval(startupTimer);
        startupTimer = null;
      }
      if (stderrUnlisten) {
        stderrUnlisten();
        stderrUnlisten = null;
      }
    }
  }

  // Resume existing session
  async function resumeSession(savedSession: SavedSession): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const configStore = useConfigStore();
      const agentConfig: AgentConfig | undefined = configStore.getAgent(savedSession.agentName);
      if (!agentConfig) {
        throw new Error(`Agent '${savedSession.agentName}' not found in config`);
      }

      acpClient = await createAcpClient({
        name: savedSession.agentName,
        config: agentConfig,
      });

      if (sessionUpdateHandler) {
        acpClient.onSessionUpdate = sessionUpdateHandler;
      }
      acpClient.onTransportClose = (reason) => {
        handleUnexpectedClose(reason);
      };

      if (permissionWatchSetup) {
        permissionWatchSetup(acpClient);
      }

      const canAccessFs = isDesktop();

      const initResponse = await acpClient.initialize({
        protocolVersion: PROTOCOL_VERSION,
        clientCapabilities: {
          fs: {
            readTextFile: canAccessFs,
            writeTextFile: canAccessFs,
          },
        },
        clientInfo: {
          name: 'acp-ui',
          title: 'ACP UI',
          version: appVersion,
        },
      });

      const availableAuthMethods = initResponse.authMethods || [];

      // Try to load existing session - may fail with auth_required
      try {
        await acpClient.loadSession({
          sessionId: savedSession.sessionId,
          cwd: savedSession.cwd,
          mcpServers: [],
        });
      } catch (sessionError: unknown) {
        const errorMessage = sessionError instanceof Error ? sessionError.message : String(sessionError);
        const isAuthRequired = errorMessage.toLowerCase().includes('authentication required') ||
                               errorMessage.includes('-32000');

        const isSessionNotFound = errorMessage.toLowerCase().includes('session not found') ||
                                  errorMessage.includes('not found') ||
                                  errorMessage.includes('-32001');

        if (isSessionNotFound) {
          await acpClient.disconnect();
          acpClient = null;
          savedSessions.value = savedSessions.value.filter(s => s.id !== savedSession.id);
          await saveSessionsToStore();
          throw new Error(`Session "${savedSession.title}" has expired. The agent no longer has this session.`);
        }

        if (isAuthRequired && availableAuthMethods.length > 0) {
          console.log('Authentication required, available methods:', availableAuthMethods);

          let selectedMethodId: string | null = null;
          if (authMethodHandler) {
            selectedMethodId = await authMethodHandler(availableAuthMethods, savedSession.agentName);
          }

          if (!selectedMethodId) {
            await acpClient.disconnect();
            throw new Error('Authentication cancelled by user');
          }

          console.log('Authenticating with method:', selectedMethodId);
          const authResponse = await acpClient.authenticate({
            methodId: selectedMethodId,
          });
          console.log('Authentication successful:', authResponse);

          await acpClient.loadSession({
            sessionId: savedSession.sessionId,
            cwd: savedSession.cwd,
            mcpServers: [],
          });
        } else {
          throw sessionError;
        }
      }

      currentSession.value = savedSession;
      isConnected.value = true;

      if (messageWatchSetup) {
        messageWatchSetup();
      }

      trackEvent('SessionResumed', { agentName: savedSession.agentName, success: 'true' });

      savedSession.lastUpdated = Date.now();
      await saveSessionsToStore();

    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      if (acpClient) {
        try {
          await acpClient.disconnect();
        } catch (cleanupErr) {
          console.warn('disconnect during resumeSession cleanup failed:', cleanupErr);
        }
        acpClient = null;
      }
      trackEvent('SessionResumed', { agentName: savedSession.agentName, success: 'false' });
      trackError(e instanceof Error ? e : new Error(String(e)));
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  // Cancel ongoing connection attempt
  async function cancelConnection(): Promise<void> {
    connectionAborted = true;

    if (acpClient) {
      try {
        await acpClient.disconnect();
      } catch (e) {
        console.error('Error disconnecting:', e);
      }
      acpClient = null;
    }

    isLoading.value = false;
    isConnecting.value = false;
    error.value = null;
  }

  // Disconnect current session
  async function disconnect(): Promise<void> {
    cancelAutoReconnect();

    const agentName = currentSession.value?.agentName || 'unknown';
    const sessionStart = currentSession.value?.lastUpdated || Date.now();
    const sessionDuration = Math.round((Date.now() - sessionStart) / 1000);

    if (acpClient) {
      await acpClient.disconnect();
      acpClient = null;
    }

    trackEvent('SessionDisconnected', {
      agentName,
      sessionDurationSeconds: String(sessionDuration),
    });

    currentSession.value = null;
    isConnected.value = false;
    error.value = null;
  }

  // Delete saved session - returns sessionId for message cleanup
  async function deleteSession(sessionStoreId: string): Promise<string | null> {
    const session = savedSessions.value.find(s => s.id === sessionStoreId);
    savedSessions.value = savedSessions.value.filter(s => s.id !== sessionStoreId);
    await saveSessionsToStore();

    return session ? session.sessionId : null;
  }

  // Try to reconnect
  async function tryReconnect(): Promise<boolean> {
    if (isConnected.value || isConnecting.value || isLoading.value) {
      return false;
    }
    const session = currentSession.value;
    if (!session) {
      return false;
    }
    if (acpClient) {
      return false;
    }
    if (!session.supportsLoadSession) {
      return false;
    }

    error.value = null;
    isReconnecting.value = true;
    try {
      await resumeSession(session);
      reconnectAttemptCount = 0;
      return true;
    } catch (e) {
      console.warn('Foreground reconnect failed:', e);
      return true;
    } finally {
      isReconnecting.value = false;
    }
  }

  function clearError() {
    error.value = null;
  }

  // Internal setter for acpClient (used by session.ts)
  function _setAcpClient(client: AcpClientBridge | null) {
    acpClient = client;
  }

  return {
    // State
    savedSessions,
    currentSession,
    isConnected,
    isLoading,
    isConnecting,
    isReconnecting,
    error,
    startupPhase,
    startupLogs,
    startupElapsed,

    // Computed
    hasActiveSession,
    resumableSessions,

    // Actions
    initStore,
    createSession,
    resumeSession,
    disconnect,
    deleteSession,
    cancelConnection,
    tryReconnect,
    clearError,
    saveSessionsToStore,
    cancelAutoReconnect,

    // Internal setters for cross-store wiring
    setSessionUpdateHandler,
    setPermissionWatchSetup,
    setMessageWatchSetup,
    setMessageSaveHandler,
    setAuthMethodHandler,
    _setAcpClient,

    // Getters
    get acpClient() { return acpClient; },
  };
});