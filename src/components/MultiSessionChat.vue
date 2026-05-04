<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMultiSessionStore } from '../stores/multi-session'
import SessionTabs from './SessionTabs.vue'

const multiSession = useMultiSessionStore()
const inputText = ref('')

const activeSession = computed(() => multiSession.activeSession)
const messages = computed(() => multiSession.messages)
const isConnected = computed(() => multiSession.isConnected)
const isLoading = computed(() => multiSession.isLoading)
const error = computed(() => multiSession.error)

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text) return
  inputText.value = ''
  await multiSession.sendPrompt(text)
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString()
}
</script>

<template>
  <div class="multi-session-chat">
    <SessionTabs />
    <div class="chat-area">
    <div v-if="!activeSession" class="empty-state">
      <h3>没有活跃会话</h3>
      <p>请点击标签栏中的 + 创建新会话</p>
    </div>

    <template v-else>
      <!-- Messages -->
      <div class="messages-container">
        <div v-if="messages.length === 0" class="empty-chat">
          <h3>{{ activeSession.agentName }}</h3>
          <p>开始对话吧</p>
        </div>

        <div
          v-for="msg in messages"
          :key="msg.id"
          class="message"
          :class="msg.role"
        >
          <div class="message-header">
            <span class="message-role">{{ msg.role === 'user' ? '你' : activeSession.agentName }}</span>
            <span class="message-time">{{ formatTime(msg.timestamp) }}</span>
          </div>
          <div class="message-content">
            <div v-if="msg.thought" class="message-thought">
              <em>{{ msg.thought }}</em>
            </div>
            <div class="message-text">{{ msg.content }}</div>
          </div>
        </div>

        <div v-if="isLoading" class="loading-indicator">
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>
      </div>

      <!-- Error banner -->
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button @click="multiSession.clearError">✕</button>
      </div>

      <!-- Input -->
      <div class="input-area">
        <textarea
          v-model="inputText"
          class="chat-input"
          placeholder="输入消息..."
          :disabled="!isConnected || isLoading"
          @keydown.enter.exact.prevent="sendMessage"
          rows="2"
        ></textarea>
        <button
          class="send-btn"
          :disabled="!inputText.trim() || !isConnected || isLoading"
          @click="sendMessage"
        >
          发送
        </button>
      </div>
    </template>
    </div>
  </div>
</template>

<style scoped>
.multi-session-chat {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-state,
.empty-chat {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.empty-state h3,
.empty-chat h3 {
  color: var(--text-primary);
  margin-bottom: 8px;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  max-width: 80%;
  padding: 10px 14px;
  border-radius: 12px;
}

.message.user {
  align-self: flex-end;
  background: var(--bg-primary);
  color: white;
}

.message.assistant {
  align-self: flex-start;
  background: var(--bg-assistant);
  color: var(--text-primary);
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
  font-size: 12px;
  opacity: 0.7;
}

.message-thought {
  font-size: 12px;
  opacity: 0.6;
  margin-bottom: 4px;
  padding-left: 8px;
  border-left: 2px solid currentColor;
}

.message-text {
  white-space: pre-wrap;
  word-break: break-word;
}

.loading-indicator {
  display: flex;
  gap: 4px;
  padding: 8px;
  align-self: flex-start;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
  animation: bounce 1.4s infinite ease-in-out;
}

.dot:nth-child(1) { animation-delay: -0.32s; }
.dot:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
  40% { transform: scale(1); opacity: 1; }
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #fee;
  color: #c00;
  font-size: 13px;
}

.error-banner button {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}

.input-area {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-surface);
}

.chat-input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-main);
  color: var(--text-primary);
  font-size: 14px;
  resize: none;
  font-family: inherit;
}

.chat-input:focus {
  border-color: var(--primary);
  outline: none;
}

.chat-input:disabled {
  opacity: 0.5;
}

.send-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background: var(--bg-primary);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
