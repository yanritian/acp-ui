<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { ThinkingChunk } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  content: string
  chunks: ThinkingChunk[]
  depth: number
  isStreaming: boolean
}>()

// State
const displayedContent = ref('')
const currentChunkIndex = ref(0)
const isTyping = ref(false)

// Computed
const depthIndicator = computed(() => {
  const depth = Math.min(props.depth, 5)
  return '🧠'.repeat(depth)
})

const thinkingDuration = computed(() => {
  if (props.chunks.length === 0) return '0s'
  const totalDuration = props.chunks.reduce((sum, chunk) => sum + chunk.duration, 0)
  if (totalDuration < 1000) return `${totalDuration}ms`
  return `${(totalDuration / 1000).toFixed(1)}s`
})

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
  }, 30) // 30ms per character
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
    // When streaming stops, show full content
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
  <div class="thinking-display">
    <!-- Header -->
    <div class="thinking-header">
      <div class="thinking-icon">💭</div>
      <div class="thinking-info">
        <span class="thinking-label">{{ t('agentProgress.thinkingProcess') }}</span>
        <span class="depth-indicator">{{ depthIndicator }}</span>
      </div>
      <div class="thinking-duration">{{ thinkingDuration }}</div>
    </div>

    <!-- Content -->
    <div class="thinking-content">
      <div class="thinking-text">
        {{ displayedContent }}
        <span v-if="isTyping" class="cursor">|</span>
      </div>

      <!-- Streaming indicator -->
      <div v-if="isStreaming" class="streaming-indicator">
        <div class="streaming-dot"></div>
        <span>{{ t('agentProgress.thinkingRealtime') }}</span>
      </div>
    </div>

    <!-- Chunks timeline (optional visualization) -->
    <div v-if="chunks.length > 0" class="chunks-timeline">
      <div class="timeline-label">{{ t('agentProgress.thinkingChunksTimeline') }}</div>
      <div class="timeline-container">
        <div
          v-for="(chunk, index) in chunks"
          :key="chunk.id"
          :class="['chunk-node', { active: index === currentChunkIndex }]"
          :title="`Chunk ${index + 1}: ${chunk.duration}ms`"
        >
          <div class="chunk-index">{{ index + 1 }}</div>
          <div class="chunk-duration">{{ chunk.duration }}ms</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.thinking-display {
  padding: 16px;
  background: rgba(139, 92, 246, 0.1);
  border: 1px solid rgba(139, 92, 246, 0.3);
  border-radius: 12px;
}

.thinking-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(139, 92, 246, 0.2);
}

.thinking-icon {
  font-size: 24px;
  margin-right: 8px;
}

.thinking-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.thinking-label {
  font-size: 14px;
  font-weight: 600;
  color: #8b5cf6;
}

.depth-indicator {
  font-size: 12px;
  color: #a78bfa;
}

.thinking-duration {
  font-size: 12px;
  color: #8b8b9b;
  font-family: 'Monaco', monospace;
}

.thinking-content {
  margin-bottom: 12px;
}

.thinking-text {
  font-size: 13px;
  color: #e0e0e0;
  line-height: 1.6;
  word-break: break-word;
  white-space: pre-wrap;
}

.cursor {
  color: #8b5cf6;
  font-weight: bold;
  animation: blink 0.6s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.streaming-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding: 8px;
  background: rgba(139, 92, 246, 0.1);
  border-radius: 6px;
}

.streaming-dot {
  width: 8px;
  height: 8px;
  background: #8b5cf6;
  border-radius: 50%;
  animation: pulse-dot 1s infinite;
}

@keyframes pulse-dot {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}

.streaming-indicator span {
  font-size: 11px;
  color: #8b5cf6;
}

.chunks-timeline {
  padding-top: 12px;
  border-top: 1px solid rgba(139, 92, 246, 0.2);
}

.timeline-label {
  font-size: 11px;
  color: #8b8b9b;
  margin-bottom: 8px;
}

.timeline-container {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding: 4px 0;
}

.chunk-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 6px 12px;
  background: rgba(139, 92, 246, 0.1);
  border: 1px solid rgba(139, 92, 246, 0.2);
  border-radius: 6px;
  transition: all 0.2s ease;
}

.chunk-node.active {
  background: rgba(139, 92, 246, 0.2);
  border-color: #8b5cf6;
}

.chunk-index {
  font-size: 12px;
  font-weight: 600;
  color: #8b5cf6;
}

.chunk-duration {
  font-size: 10px;
  color: #8b8b9b;
  font-family: 'Monaco', monospace;
}
</style>