<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import type { OutputChunk } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  content: string
  chunks: OutputChunk[]
  isStreaming: boolean
  currentPosition: number
}>()

// State
const displayedContent = ref('')
const isTyping = ref(false)

// Methods - typewriter effect
let typewriterInterval: ReturnType<typeof setInterval> | null = null

function startTypewriterEffect() {
  if (typewriterInterval) {
    clearInterval(typewriterInterval)
  }

  isTyping.value = true
  const charsToAdd = props.content.slice(displayedContent.value.length)
  let charIndex = 0

  typewriterInterval = setInterval(() => {
    if (charIndex < charsToAdd.length) {
      displayedContent.value += charsToAdd[charIndex]
      charIndex++
    } else {
      clearInterval(typewriterInterval!)
      typewriterInterval = null
      isTyping.value = false
    }
  }, 20) // 20ms per character - faster than thinking
}

function stopTypewriterEffect() {
  if (typewriterInterval) {
    clearInterval(typewriterInterval)
    typewriterInterval = null
  }
  isTyping.value = false
}

// Watch for content changes
watch(() => props.content, (newContent) => {
  if (props.isStreaming && newContent.length > displayedContent.value.length) {
    startTypewriterEffect()
  } else if (!props.isStreaming) {
    displayedContent.value = newContent
    stopTypewriterEffect()
  }
})

// Lifecycle
onMounted(() => {
  if (props.isStreaming && props.content) {
    startTypewriterEffect()
  } else {
    displayedContent.value = props.content
  }
})

onUnmounted(() => {
  stopTypewriterEffect()
})
</script>

<template>
  <div class="output-typewriter">
    <!-- Header -->
    <div class="output-header">
      <div class="output-icon">💬</div>
      <div class="output-label">{{ t('agentProgress.agentOutput') }}</div>
      <div class="output-progress">
        <span class="current-position">{{ currentPosition }}</span>
        <span class="total-length"> / {{ content.length }}</span>
        <span v-if="isStreaming" class="streaming-badge">{{ t('agentProgress.outputRealtime') }}</span>
      </div>
    </div>

    <!-- Content -->
    <div class="output-content">
      <div class="output-text">
        {{ displayedContent }}
        <span v-if="isTyping" class="cursor">|</span>
      </div>
    </div>

    <!-- Chunks info -->
    <div v-if="chunks.length > 0" class="chunks-info">
      <div class="chunks-label">{{ t('agentProgress.outputChunks') }}: {{ chunks.length }}</div>
      <div class="chunks-types">
        <span
          v-for="chunk in chunks.slice(-5)"
          :key="chunk.id"
          :class="['chunk-type-badge', chunk.type]"
        >
          {{ chunk.type }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.output-typewriter {
  padding: 16px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 12px;
}

.output-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(16, 185, 129, 0.2);
}

.output-icon {
  font-size: 20px;
  margin-right: 8px;
}

.output-label {
  font-size: 14px;
  font-weight: 600;
  color: #10b981;
}

.output-progress {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: 'Monaco', monospace;
}

.current-position {
  font-size: 12px;
  color: #10b981;
  font-weight: 600;
}

.total-length {
  font-size: 12px;
  color: #8b8b9b;
}

.streaming-badge {
  padding: 2px 8px;
  background: rgba(16, 185, 129, 0.2);
  border-radius: 6px;
  font-size: 10px;
  color: #10b981;
  margin-left: 8px;
}

.output-content {
  margin-bottom: 12px;
}

.output-text {
  font-size: 13px;
  color: #e0e0e0;
  line-height: 1.6;
  word-break: break-word;
  white-space: pre-wrap;
}

.cursor {
  color: #10b981;
  font-weight: bold;
  animation: blink 0.5s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.chunks-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 8px;
  border-top: 1px solid rgba(16, 185, 129, 0.2);
}

.chunks-label {
  font-size: 11px;
  color: #8b8b9b;
}

.chunks-types {
  display: flex;
  gap: 4px;
}

.chunk-type-badge {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 10px;
  color: white;
}

.chunk-type-badge.text {
  background: #3b82f6;
}

.chunk-type-badge.code {
  background: #8b5cf6;
}

.chunk-type-badge.file {
  background: #f59e0b;
}

.chunk-type-badge.command {
  background: #ef4444;
}

.chunk-type-badge.result {
  background: #10b981;
}
</style>