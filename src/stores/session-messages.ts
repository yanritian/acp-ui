/**
 * Session Messages Store
 *
 * Manages message CRUD, persistence, and session update handling.
 */
import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { loadKvStore, type KVStore } from '../lib/host/storage';
import type { ChatMessage, ToolCallInfo } from '../lib/types';
import type { SessionNotification } from '@agentclientprotocol/sdk';

const MESSAGES_STORE_PATH = 'messages.json';

// Debounce timer for message persistence
let messageSaveTimer: ReturnType<typeof setTimeout> | null = null;
const MESSAGE_SAVE_DEBOUNCE_MS = 500;

// Current session reference (set by session.ts)
let currentSessionId: string | null = null;

export const useSessionMessagesStore = defineStore('sessionMessages', () => {
  // State
  const messages = ref<ChatMessage[]>([]);
  const toolCalls = ref<Map<string, ToolCallInfo>>(new Map());

  // Persistence
  let store: KVStore | null = null;

  // Computed
  const messageList = computed(() => messages.value);
  const toolCallList = computed(() => Array.from(toolCalls.value.values()));

  // Initialize persistence
  async function initPersistence() {
    store = await loadKvStore(MESSAGES_STORE_PATH);
  }

  // Set current session ID (called by session.ts)
  function setCurrentSessionId(sessionId: string | null) {
    currentSessionId = sessionId;
  }

  // Save messages for current session (debounced)
  async function saveMessages(): Promise<void> {
    if (!currentSessionId || !store) return;

    await store.set(currentSessionId, messages.value);
    await store.save();
  }

  // Debounced message save - prevents excessive writes during rapid updates
  function debouncedSaveMessages(): void {
    if (messageSaveTimer) {
      clearTimeout(messageSaveTimer);
    }
    messageSaveTimer = setTimeout(() => {
      saveMessages().catch(e => console.warn('Message save failed:', e));
    }, MESSAGE_SAVE_DEBOUNCE_MS);
  }

  // Emergency synchronous save for beforeunload
  function emergencySave(): void {
    if (currentSessionId && messages.value.length > 0) {
      try {
        const key = `acp-ui:messages:${currentSessionId}`;
        localStorage.setItem(key, JSON.stringify(messages.value));
      } catch (e) {
        console.warn('Emergency message save failed:', e);
      }
    }
  }

  // Load messages for a session
  async function loadMessages(sessionId: string): Promise<ChatMessage[]> {
    if (!store) {
      await initPersistence();
    }
    const saved = await store!.get<ChatMessage[]>(sessionId);
    return saved || [];
  }

  // Delete messages for a session (when session is deleted)
  async function deleteMessages(sessionId: string): Promise<void> {
    if (!store) {
      await initPersistence();
    }
    await store!.set(sessionId, null);
    await store!.save();
  }

  // Set up message persistence watch
  function setupMessageWatch(): void {
    watch(
      () => messages.value,
      () => {
        debouncedSaveMessages();
      },
      { deep: true }
    );
  }

  // Stop message persistence watch
  function stopMessageWatch(): void {
    if (messageSaveTimer) {
      clearTimeout(messageSaveTimer);
      messageSaveTimer = null;
    }
  }

  // Clear messages
  function clearMessages(): void {
    messages.value = [];
    toolCalls.value.clear();
  }

  // Add a message
  function addMessage(message: ChatMessage): void {
    messages.value.push(message);
  }

  // Handle session update notifications
  function handleSessionUpdate(notification: SessionNotification): void {
    const update = notification.update;

    switch (update.sessionUpdate) {
      case 'user_message_chunk':
        // Append to last user message or create new (for replay)
        const lastUserMsg = messages.value[messages.value.length - 1];
        if (lastUserMsg && lastUserMsg.role === 'user') {
          if (update.content.type === 'text') {
            lastUserMsg.content += update.content.text;
          }
        } else {
          messages.value.push({
            id: crypto.randomUUID(),
            role: 'user',
            content: update.content.type === 'text' ? update.content.text : '',
            timestamp: Date.now(),
          });
        }
        break;

      case 'agent_message_chunk':
        // Append to last assistant message or create new
        const lastMsg = messages.value[messages.value.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          if (update.content.type === 'text') {
            lastMsg.content += update.content.text;
          }
        } else {
          messages.value.push({
            id: crypto.randomUUID(),
            role: 'assistant',
            content: update.content.type === 'text' ? update.content.text : '',
            timestamp: Date.now(),
            toolCalls: [],
          });
        }
        break;

      case 'agent_thought_chunk':
        // Append to last assistant message's thought field or create new
        const lastAssistantMsg = messages.value[messages.value.length - 1];
        if (lastAssistantMsg && lastAssistantMsg.role === 'assistant') {
          if (update.content.type === 'text') {
            lastAssistantMsg.thought = (lastAssistantMsg.thought || '') + update.content.text;
          }
        } else {
          messages.value.push({
            id: crypto.randomUUID(),
            role: 'assistant',
            content: '',
            thought: update.content.type === 'text' ? update.content.text : '',
            timestamp: Date.now(),
            toolCalls: [],
          });
        }
        break;

      case 'tool_call':
        // Add tool call to the current assistant message
        const currentAssistantMsg = messages.value[messages.value.length - 1];
        if (currentAssistantMsg && currentAssistantMsg.role === 'assistant') {
          if (!currentAssistantMsg.toolCalls) {
            currentAssistantMsg.toolCalls = [];
          }
          currentAssistantMsg.toolCalls.push({
            toolCallId: update.toolCallId,
            title: update.title,
            kind: update.kind || 'other',
            status: update.status || 'pending',
            locations: update.locations,
          });
        }
        // Also keep in global map for updates
        toolCalls.value.set(update.toolCallId, {
          toolCallId: update.toolCallId,
          title: update.title,
          kind: update.kind || 'other',
          status: update.status || 'pending',
          locations: update.locations,
        });
        break;

      case 'tool_call_update':
        const existing = toolCalls.value.get(update.toolCallId);
        if (existing) {
          if (update.status) existing.status = update.status;
          if (update.title) existing.title = update.title;
          // Also update in the message's toolCalls array
          for (const msg of messages.value) {
            if (msg.toolCalls) {
              const tc = msg.toolCalls.find(t => t.toolCallId === update.toolCallId);
              if (tc) {
                if (update.status) tc.status = update.status;
                if (update.title) tc.title = update.title;
              }
            }
          }
        }
        break;

      default:
        console.log('Unhandled session update:', update);
    }
  }

  return {
    // State
    messages,
    toolCalls,

    // Computed
    messageList,
    toolCallList,

    // Actions
    initPersistence,
    setCurrentSessionId,
    saveMessages,
    debouncedSaveMessages,
    emergencySave,
    loadMessages,
    deleteMessages,
    setupMessageWatch,
    stopMessageWatch,
    clearMessages,
    addMessage,
    handleSessionUpdate,
  };
});