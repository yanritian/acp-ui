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
import { defineStore, storeToRefs } from 'pinia';
import { ref, computed, watch } from 'vue';
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
  // Get individual stores
  const lifecycleStore = useSessionLifecycleStore();
  const messagesStore = useSessionMessagesStore();
  const permissionsStore = useSessionPermissionsStore();
  const capabilitiesStore = useSessionCapabilitiesStore();

  // Reactive refs from stores
  const {
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
  } = storeToRefs(lifecycleStore);

  const {
    messages,
    toolCalls,
  } = storeToRefs(messagesStore);

  const {
    pendingPermission,
    pendingAuthMethods,
    pendingAuthAgentName,
  } = storeToRefs(permissionsStore);

  const {
    availableModes,
    currentModeId,
    availableCommands,
    availableModels,
    currentModelId,
  } = storeToRefs(capabilitiesStore);

  // Computed
  const hasActiveSession = computed(() => currentSession.value !== null);
  const messageList = computed(() => messages.value);
  const toolCallList = computed(() => Array.from(toolCalls.value.values()));
  const resumableSessions = computed(() =>
    savedSessions.value.filter(s => s.supportsLoadSession === true)
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
    if (currentSession.value) {
      messagesStore.setCurrentSessionId(currentSession.value.sessionId);
    }

    // Clear previous state
    messagesStore.clearMessages();
    capabilitiesStore.clearCapabilities();
  }

  // Resume session with message loading
  async function resumeSession(savedSession: SavedSession): Promise<void> {
    await lifecycleStore.resumeSession(savedSession);

    // Update messages store session ID
    if (currentSession.value) {
      messagesStore.setCurrentSessionId(currentSession.value.sessionId);
    }

    // If no messages were replayed by the agent, try to load saved messages
    if (messages.value.length === 0 && currentSession.value) {
      const savedMessages = await messagesStore.loadMessages(currentSession.value.sessionId);
      if (savedMessages.length > 0) {
        messages.value = savedMessages;
      }
    }
  }

  // Send prompt using direct client call
  async function sendPrompt(text: string): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = currentSession.value?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    // Add user message
    messages.value.push({
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: Date.now(),
    });

    isLoading.value = true;
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
      if (messages.value.length === 1 && currentSession.value) {
        const newTitle = text.slice(0, 30) + (text.length > 30 ? '...' : '');
        currentSession.value.title = newTitle;
        currentSession.value.lastUpdated = Date.now();
        await lifecycleStore.saveSessionsToStore();
      }
    } finally {
      isLoading.value = false;
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

  // Resolve permission
  function resolvePermission(optionId: string): void {
    const client = lifecycleStore.acpClient;
    if (client) {
      client.resolvePermission(optionId);
    }
  }

  // Cancel permission
  function cancelPermission(): void {
    const client = lifecycleStore.acpClient;
    if (client) {
      client.cancelPermission();
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
    if (currentSession.value && messages.value.length > 0) {
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
    const sessionId = currentSession.value?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.setMode({
      sessionId,
      modeId,
    });

    // Optimistically update the current mode
    currentModeId.value = modeId;
  }

  // Set model
  async function setModel(modelId: string): Promise<void> {
    const client = lifecycleStore.acpClient;
    const sessionId = currentSession.value?.sessionId;

    if (!client || !sessionId) {
      throw new Error('No active session');
    }

    await client.unstable_setSessionModel({
      sessionId,
      modelId,
    });

    // Optimistically update the current model
    currentModelId.value = modelId;
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
    cancelPermission,
    selectAuthMethod,
    cancelAuthSelection,

    // Actions (capabilities)
    setMode,
    setModel,

    // Expose client for permission handling
    get acpClient() { return lifecycleStore.acpClient; },
  };
});