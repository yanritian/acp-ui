/**
 * Session Store - Aggregation Entry Point
 *
 * This file re-exports all session stores and provides a unified interface
 * for components that need session functionality.
 *
 * The session functionality is split into 4 stores:
 * - sessionLifecycle: Session creation, resumption, disconnection, auto-reconnect
 * - sessionMessages: Message CRUD, persistence, session update handling
 * - sessionPermissions: Permission requests and authentication
 * - sessionCapabilities: Modes, commands, and models management
 */
import { defineStore } from 'pinia';
import { computed, watch } from 'vue';
import { useSessionLifecycleStore } from './session-lifecycle';
import { useSessionMessagesStore } from './session-messages';
import { useSessionPermissionsStore } from './session-permissions';
import { useSessionCapabilitiesStore } from './session-capabilities';
import type { SavedSession, ChatMessage, ToolCallInfo, PermissionRequest, SessionMode, SlashCommand, ModelInfo } from '../lib/types';
import type { SessionNotification, AuthMethod, AvailableCommand, ContentBlock, PromptResponse } from '@agentclientprotocol/sdk';
import type { AcpClientBridge } from '../lib/acp-bridge';

// Re-export types and individual stores
export { useSessionLifecycleStore } from './session-lifecycle';
export { useSessionMessagesStore } from './session-messages';
export { useSessionPermissionsStore } from './session-permissions';
export { useSessionCapabilitiesStore } from './session-capabilities';

/**
 * Unified Session Store
 *
 * Provides the same interface as the original session.ts for backward compatibility.
 * Delegates to individual stores while coordinating cross-store interactions.
 */
export const useSessionStore = defineStore('session', () => {
  // Get individual stores - these are safe to call inside setup
  const lifecycleStore = useSessionLifecycleStore();
  const messagesStore = useSessionMessagesStore();
  const permissionsStore = useSessionPermissionsStore();
  const capabilitiesStore = useSessionCapabilitiesStore();

  // Direct computed wrappers for store properties (avoiding storeToRefs in setup)
  // Lifecycle store properties - unwrap the ref by accessing .value in computed
  const savedSessions = computed(() => lifecycleStore.savedSessions);
  const currentSession = computed({
    get: () => lifecycleStore.currentSession,
    set: (val) => { lifecycleStore.currentSession = val; }
  });
  const isConnected = computed({
    get: () => lifecycleStore.isConnected,
    set: (val) => { lifecycleStore.isConnected = val; }
  });
  const isLoading = computed({
    get: () => lifecycleStore.isLoading,
    set: (val) => { lifecycleStore.isLoading = val; }
  });
  const isConnecting = computed({
    get: () => lifecycleStore.isConnecting,
    set: (val) => { lifecycleStore.isConnecting = val; }
  });
  const isReconnecting = computed({
    get: () => lifecycleStore.isReconnecting,
    set: (val) => { lifecycleStore.isReconnecting = val; }
  });
  const error = computed({
    get: () => lifecycleStore.error,
    set: (val) => { lifecycleStore.error = val; }
  });
  const startupPhase = computed(() => lifecycleStore.startupPhase);
  const startupLogs = computed(() => lifecycleStore.startupLogs);
  const startupElapsed = computed(() => lifecycleStore.startupElapsed);

  // Messages store properties - unwrap refs
  const messages = computed({
    get: () => messagesStore.messages,
    set: (val) => { messagesStore.messages = val; }
  });
  const toolCalls = computed(() => messagesStore.toolCalls);

  // Permissions store properties
  // pendingPermission is a read-only computed from pendingPermissions[0]
  const pendingPermission = computed(() => permissionsStore.pendingPermission);
  const pendingPermissions = computed({
    get: () => permissionsStore.pendingPermissions,
    set: (val) => { permissionsStore.pendingPermissions = val; }
  });
  const pendingAuthMethods = computed({
    get: () => permissionsStore.pendingAuthMethods,
    set: (val) => { permissionsStore.pendingAuthMethods = val; }
  });
  const pendingAuthAgentName = computed({
    get: () => permissionsStore.pendingAuthAgentName,
    set: (val) => { permissionsStore.pendingAuthAgentName = val; }
  });

  // Capabilities store properties
  const availableModes = computed(() => capabilitiesStore.availableModes);
  const currentModeId = computed({
    get: () => capabilitiesStore.currentModeId,
    set: (val) => { capabilitiesStore.currentModeId = val; }
  });
  const availableCommands = computed(() => capabilitiesStore.availableCommands);
  const availableModels = computed(() => capabilitiesStore.availableModels);
  const currentModelId = computed({
    get: () => capabilitiesStore.currentModelId,
    set: (val) => { capabilitiesStore.currentModelId = val; }
  });

  // Computed - use direct store access for consistency
  const hasActiveSession = computed(() => lifecycleStore.currentSession !== null);
  const messageList = computed(() => messagesStore.messages);
  const toolCallList = computed(() => Array.from(messagesStore.toolCalls.values()));
  const resumableSessions = computed(() =>
    lifecycleStore.savedSessions.filter(s => s.supportsLoadSession === true)
  );

  // Permission watch cleanup
  let permissionWatchStop: (() => void) | null = null;

  // Wire up cross-store dependencies
  function wireUpDependencies() {
    // Set up session update handler that delegates to messages and capabilities
    lifecycleStore.setSessionUpdateHandler((notification: SessionNotification) => {
      const update = notification.update;

      // Handle message updates
      messagesStore.handleSessionUpdate(notification);

      // Handle capability updates
      if (update.sessionUpdate === 'current_mode_update') {
        if ('modeId' in update && update.modeId) {
          capabilitiesStore.updateCurrentMode(update.modeId as string);
        }
      } else if (update.sessionUpdate === 'available_commands_update') {
        if ('availableCommands' in update && Array.isArray(update.availableCommands)) {
          // Convert AvailableCommand[] to the expected format
          const commands = (update.availableCommands as AvailableCommand[]).map((cmd) => ({
            name: cmd.name,
            description: cmd.description,
            input: cmd.input ? { hint: cmd.input.hint ?? undefined } : undefined,
          }));
          capabilitiesStore.setCommands(commands);
        }
      }
    });

    // Set up permission watch factory
    lifecycleStore.setPermissionWatchSetup((client) => {
      if (permissionWatchStop) permissionWatchStop();
      permissionWatchStop = watch(
        () => client?.pendingPermissionRequest.value,
        (newValue) => {
          permissionsStore.setPendingPermission(newValue ?? null);
        },
        { immediate: true }
      );
      return permissionWatchStop;
    });

    // Set up message watch factory
    lifecycleStore.setMessageWatchSetup(() => {
      messagesStore.setupMessageWatch();
    });

    // Set up emergency save handler
    lifecycleStore.setMessageSaveHandler(() => {
      messagesStore.emergencySave();
    });

    // Set up auth method handler
    lifecycleStore.setAuthMethodHandler(async (methods: AuthMethod[], agentName: string) => {
      return permissionsStore.promptForAuthMethod(methods, agentName);
    });
  }

  // Initialize all stores
  async function initStore() {
    await lifecycleStore.initStore();
    await messagesStore.initPersistence();
    wireUpDependencies();
  }

  // Create session with capability setup
  async function createSession(agentName: string, cwd: string): Promise<void> {
    const sessionResponse = await lifecycleStore.createSession(agentName, cwd);

    // Set capabilities from session response
    if (sessionResponse) {
      capabilitiesStore.setModes(sessionResponse.modes ?? undefined);
      capabilitiesStore.setModels(sessionResponse.models ?? undefined);
    }

    // Update messages store session ID
    if (lifecycleStore.currentSession) {
      messagesStore.setCurrentSessionId(lifecycleStore.currentSession.sessionId);
    }

    // Clear previous state
    messagesStore.clearMessages();
    capabilitiesStore.clearCapabilities();
  }

  // Resume session with message loading
  async function resumeSession(savedSession: SavedSession): Promise<void> {
    await lifecycleStore.resumeSession(savedSession);

    // Update messages store session ID
    if (lifecycleStore.currentSession) {
      messagesStore.setCurrentSessionId(lifecycleStore.currentSession.sessionId);
    }

    // If no messages were replayed by the agent, try to load saved messages
    if (messagesStore.messages.length === 0 && lifecycleStore.currentSession) {
      const savedMessages = await messagesStore.loadMessages(lifecycleStore.currentSession.sessionId);
      if (savedMessages.length > 0) {
        messagesStore.messages = savedMessages;
      }
    }
  }

  // Send prompt using direct client call
  async function sendPrompt(text: string): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = lifecycleStore.currentSession?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    // Add user message
    messagesStore.messages.push({
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: Date.now(),
    });

    lifecycleStore.isLoading = true;
    try {
      const response = await client.prompt({
        sessionId,
        prompt: [
          {
            type: 'text' as const,
            text,
          },
        ],
      });

      console.log('Prompt completed:', response.stopReason);

      // Update session title on first message (take first 30 characters)
      if (messagesStore.messages.length === 1 && lifecycleStore.currentSession) {
        const newTitle = text.slice(0, 30) + (text.length > 30 ? '...' : '');
        lifecycleStore.currentSession.title = newTitle;
        lifecycleStore.currentSession.lastUpdated = Date.now();
        await lifecycleStore.saveSessionsToStore();
      }
    } finally {
      lifecycleStore.isLoading = false;
    }
  }

  // Cancel operation
  async function cancelOperation(): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = currentSession.value?.sessionId;

    if (!client || !sessionId) return;

    await client.cancel({
      sessionId,
    });
  }

  // Cancel connection
  async function cancelConnection(): Promise<void> {
    // Cancel pending auth
    permissionsStore.cancelPendingAuth();
    await lifecycleStore.cancelConnection();
  }

  // Resolve permission (delegates to permissionsStore for batch support)
  function resolvePermission(sessionIdOrOptionId: string, optionId?: string): void {
    if (optionId !== undefined) {
      // New batch mode: sessionId + optionId
      permissionsStore.resolvePermission(sessionIdOrOptionId, optionId);
    } else {
      // Legacy single mode: just optionId
      const client = lifecycleStore.acpClient;
      if (client) {
        client.resolvePermission(sessionIdOrOptionId);
      }
    }
  }

  // Resolve batch permissions
  function resolveBatchPermissions(items: { sessionId: string; optionId: string }[]): void {
    permissionsStore.resolveBatchPermissions(items);
  }

  // Cancel all permissions
  function cancelAllPermissions(): void {
    permissionsStore.cancelAllPermissions();
  }

  // Cancel permission
  function cancelPermission(sessionId?: string): void {
    if (sessionId !== undefined) {
      permissionsStore.cancelPermission(sessionId);
    } else {
      // Legacy single mode
      const client = lifecycleStore.acpClient;
      if (client) {
        client.cancelPermission();
      }
    }
  }

  // Select auth method
  function selectAuthMethod(methodId: string): void {
    permissionsStore.selectAuthMethod(methodId);
  }

  // Cancel auth selection
  function cancelAuthSelection(): void {
    permissionsStore.cancelAuthSelection();
  }

  // Disconnect with cleanup
  async function disconnect(): Promise<void> {
    // Cancel auto reconnect
    lifecycleStore.cancelAutoReconnect();

    // Stop watches
    if (permissionWatchStop) {
      permissionWatchStop();
      permissionWatchStop = null;
    }
    messagesStore.stopMessageWatch();

    // Save messages before disconnecting
    if (lifecycleStore.currentSession && messagesStore.messages.length > 0) {
      try {
        await messagesStore.saveMessages();
      } catch (e) {
        console.warn('Failed to save messages on disconnect:', e);
      }
    }

    // Disconnect
    await lifecycleStore.disconnect();

    // Clear state
    messagesStore.clearMessages();
    capabilitiesStore.clearCapabilities();
    permissionsStore.clearPendingState();
    messagesStore.setCurrentSessionId(null);
  }

  // Delete session
  async function deleteSession(sessionStoreId: string): Promise<void> {
    const sessionId = await lifecycleStore.deleteSession(sessionStoreId);
    if (sessionId) {
      await messagesStore.deleteMessages(sessionId);
    }
  }

  // Set mode
  async function setMode(modeId: string): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = lifecycleStore.currentSession?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.setMode({
      sessionId,
      modeId,
    });

    // Optimistically update the current mode
    capabilitiesStore.updateCurrentMode(modeId);
  }

  // Set model
  async function setModel(modelId: string): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = lifecycleStore.currentSession?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.unstable_setSessionModel({
      sessionId,
      modelId,
    });

    // Optimistically update the current model - need to find method in capabilitiesStore
    // capabilitiesStore doesn't have updateCurrentModel method, use setModels with current
    const currentModels = capabilitiesStore.availableModels;
    capabilitiesStore.setModels({
      availableModels: currentModels,
      currentModelId: modelId,
    });
  }

  // Clear error
  function clearError() {
    lifecycleStore.clearError();
  }

  // Try reconnect
  async function tryReconnect(): Promise<boolean> {
    return lifecycleStore.tryReconnect();
  }

  return {
    // State (from lifecycle)
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

    // State (from messages)
    messages,
    toolCalls,

    // State (from permissions)
    pendingPermission,
    pendingPermissions,
    pendingAuthMethods,
    pendingAuthAgentName,

    // State (from capabilities)
    availableModes,
    currentModeId,
    availableCommands,
    availableModels,
    currentModelId,

    // Computed
    hasActiveSession,
    messageList,
    toolCallList,
    resumableSessions,

    // Actions (lifecycle)
    initStore,
    createSession,
    resumeSession,
    disconnect,
    cancelConnection,
    tryReconnect,
    clearError,
    deleteSession,

    // Actions (messages)
    sendPrompt,
    cancelOperation,

    // Actions (permissions)
    resolvePermission,
    resolveBatchPermissions,
    cancelPermission,
    cancelAllPermissions,
    selectAuthMethod,
    cancelAuthSelection,

    // Actions (capabilities)
    setMode,
    setModel,

    // Expose client for permission handling
    get acpClient() { return lifecycleStore.acpClient; },
  };
});