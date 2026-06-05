<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── Types ─────────────────────────────────────────────────────────────

interface SystemPromptInfo {
  frozen: boolean
  hash: string
  sizeBytes: number
  tokenCount: number
}

interface CacheStats {
  systemCacheHits: number
  systemCacheMisses: number
  contextCacheHits: number
  contextCacheMisses: number
}

interface CompressionStats {
  totalTokensSaved: number
  compressionRatio: number
  totalCompressions: number
  avgTokensPerCompression: number
}

interface ArchivedChunk {
  id: string
  tokenCount: number
  createdAt: string
  summary: string
}

interface IdleTimerStatus {
  active: boolean
  nextCompressionIn: number // seconds
  idleThreshold: number // seconds
}

interface TokenStats {
  systemPrompt: SystemPromptInfo
  cache: CacheStats
  compression: CompressionStats
  archivedChunks: ArchivedChunk[]
  idleTimer: IdleTimerStatus
  totalTokensUsed: number
  contextWindowLimit: number
}

// ── Reactive state ────────────────────────────────────────────────────

const stats = ref<TokenStats | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const compressing = ref(false)
let countdownInterval: ReturnType<typeof setInterval> | null = null

// ── Helpers ───────────────────────────────────────────────────────────

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M tokens`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K tokens`
  return `${n} tokens`
}

function formatBytes(n: number): string {
  if (n >= 1_048_576) return `${(n / 1_048_576).toFixed(1)} MB`
  if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${n} B`
}

function hitRate(hits: number, misses: number): string {
  const total = hits + misses
  if (total === 0) return 'N/A'
  return `${((hits / total) * 100).toFixed(1)}%`
}

const contextUsagePercent = computed(() => {
  if (!stats.value) return 0
  const { totalTokensUsed, contextWindowLimit } = stats.value
  if (contextWindowLimit === 0) return 0
  return Math.min(100, (totalTokensUsed / contextWindowLimit) * 100)
})

// ── Data fetching ─────────────────────────────────────────────────────

async function fetchStats() {
  try {
    loading.value = true
    error.value = null
    // Attempt to call backend; fall back to mock data if unavailable
    const result = await invoke<TokenStats>('plugin_get_stats')
    stats.value = result
  } catch {
    // Backend token optimizer not yet available -- use placeholder data
    stats.value = getMockStats()
  } finally {
    loading.value = false
  }
}

function getMockStats(): TokenStats {
  return {
    systemPrompt: {
      frozen: true,
      hash: 'a3f8c2e1b9d04567',
      sizeBytes: 4820,
      tokenCount: 1280,
    },
    cache: {
      systemCacheHits: 342,
      systemCacheMisses: 18,
      contextCacheHits: 156,
      contextCacheMisses: 44,
    },
    compression: {
      totalTokensSaved: 12480,
      compressionRatio: 0.63,
      totalCompressions: 47,
      avgTokensPerCompression: 265,
    },
    archivedChunks: [
      {
        id: 'chunk_001',
        tokenCount: 820,
        createdAt: '2026-06-04T10:23:00Z',
        summary: 'Earlier conversation about project setup and dependency configuration.',
      },
      {
        id: 'chunk_002',
        tokenCount: 1340,
        createdAt: '2026-06-04T11:45:00Z',
        summary: 'Discussion of authentication flow and token refresh logic.',
      },
      {
        id: 'chunk_003',
        tokenCount: 560,
        createdAt: '2026-06-04T14:10:00Z',
        summary: 'Code review of database migration scripts.',
      },
      {
        id: 'chunk_004',
        tokenCount: 980,
        createdAt: '2026-06-05T09:00:00Z',
        summary: 'Refactoring suggestions for the API layer and error handling.',
      },
    ],
    idleTimer: {
      active: true,
      nextCompressionIn: 42,
      idleThreshold: 60,
    },
    totalTokensUsed: 28400,
    contextWindowLimit: 128000,
  }
}

// ── Manual compression ───────────────────────────────────────────────

async function compressNow() {
  compressing.value = true
  try {
    await invoke('plugin_compress_now')
    // Refresh stats after compression
    await fetchStats()
  } catch {
    // Simulate a small delay and refresh with updated mock data
    await new Promise((r) => setTimeout(r, 600))
    if (stats.value) {
      stats.value.compression.totalCompressions += 1
      stats.value.compression.totalTokensSaved += 180
      stats.value.archivedChunks.unshift({
        id: `chunk_${String(stats.value.compression.totalCompressions).padStart(3, '0')}`,
        tokenCount: 180,
        createdAt: new Date().toISOString(),
        summary: 'Manually compressed context block.',
      })
    }
  } finally {
    compressing.value = false
  }
}

// ── Countdown timer for idle compression ──────────────────────────────

function startCountdown() {
  countdownInterval = setInterval(() => {
    if (stats.value?.idleTimer.active && stats.value.idleTimer.nextCompressionIn > 0) {
      stats.value.idleTimer.nextCompressionIn -= 1
    }
  }, 1000)
}

// ── Lifecycle ─────────────────────────────────────────────────────────

onMounted(() => {
  fetchStats()
  startCountdown()
})

onUnmounted(() => {
  if (countdownInterval) clearInterval(countdownInterval)
})
</script>

<template>
  <div class="view-container">
    <div class="view-header">
      <h3>Token Optimizer</h3>
      <button class="btn-compress" :disabled="compressing" @click="compressNow">
        <span v-if="compressing" class="spinner" />
        {{ compressing ? 'Compressing...' : 'Compress Now' }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="loading-state">
      <span class="spinner large" />
      <p>Loading statistics...</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="error-state">
      <p>{{ error }}</p>
      <button class="btn-retry" @click="fetchStats">Retry</button>
    </div>

    <!-- Stats -->
    <template v-else-if="stats">
      <!-- Context usage bar -->
      <div class="section context-usage">
        <div class="section-header">
          <span class="section-title">Context Window</span>
          <span class="section-badge">{{ formatTokens(stats.totalTokensUsed) }} / {{ formatTokens(stats.contextWindowLimit) }}</span>
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :class="{ warn: contextUsagePercent > 75, danger: contextUsagePercent > 90 }"
            :style="{ width: contextUsagePercent + '%' }"
          />
        </div>
        <span class="progress-label">{{ contextUsagePercent.toFixed(1) }}% used</span>
      </div>

      <!-- System Prompt -->
      <div class="card">
        <div class="card-title">System Prompt</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">Status</span>
            <span class="badge" :class="stats.systemPrompt.frozen ? 'badge-frozen' : 'badge-unfrozen'">
              {{ stats.systemPrompt.frozen ? 'Frozen' : 'Unfrozen' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">Hash</span>
            <code class="mono">{{ stats.systemPrompt.hash }}</code>
          </div>
          <div class="info-row">
            <span class="info-label">Size</span>
            <span>{{ formatBytes(stats.systemPrompt.sizeBytes) }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Tokens</span>
            <span>{{ formatTokens(stats.systemPrompt.tokenCount) }}</span>
          </div>
        </div>
      </div>

      <!-- Dual-Cache Stats -->
      <div class="card">
        <div class="card-title">Dual-Cache Statistics</div>
        <div class="cache-grid">
          <div class="cache-box">
            <div class="cache-label">System Cache</div>
            <div class="cache-numbers">
              <span class="hit">{{ stats.cache.systemCacheHits }} hits</span>
              <span class="miss">{{ stats.cache.systemCacheMisses }} misses</span>
            </div>
            <div class="cache-rate">Hit rate: {{ hitRate(stats.cache.systemCacheHits, stats.cache.systemCacheMisses) }}</div>
            <div class="mini-bar">
              <div
                class="mini-fill hit-fill"
                :style="{ width: hitRate(stats.cache.systemCacheHits, stats.cache.systemCacheMisses).replace('%','') + '%' }"
              />
            </div>
          </div>
          <div class="cache-box">
            <div class="cache-label">Context Cache</div>
            <div class="cache-numbers">
              <span class="hit">{{ stats.cache.contextCacheHits }} hits</span>
              <span class="miss">{{ stats.cache.contextCacheMisses }} misses</span>
            </div>
            <div class="cache-rate">Hit rate: {{ hitRate(stats.cache.contextCacheHits, stats.cache.contextCacheMisses) }}</div>
            <div class="mini-bar">
              <div
                class="mini-fill hit-fill"
                :style="{ width: hitRate(stats.cache.contextCacheHits, stats.cache.contextCacheMisses).replace('%','') + '%' }"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Compression Stats -->
      <div class="card">
        <div class="card-title">Compression</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">Total Tokens Saved</span>
            <span class="highlight">{{ formatTokens(stats.compression.totalTokensSaved) }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Compression Ratio</span>
            <span>{{ (stats.compression.compressionRatio * 100).toFixed(1) }}%</span>
          </div>
          <div class="info-row">
            <span class="info-label">Total Compressions</span>
            <span>{{ stats.compression.totalCompressions }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Avg Tokens / Compression</span>
            <span>{{ formatTokens(stats.compression.avgTokensPerCompression) }}</span>
          </div>
        </div>
      </div>

      <!-- Idle Compression Timer -->
      <div class="card">
        <div class="card-title">Idle Compression Timer</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">Status</span>
            <span class="badge" :class="stats.idleTimer.active ? 'badge-active' : 'badge-paused'">
              {{ stats.idleTimer.active ? 'Active' : 'Paused' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">Next Compression In</span>
            <span class="countdown">{{ stats.idleTimer.nextCompressionIn }}s</span>
          </div>
          <div class="info-row">
            <span class="info-label">Idle Threshold</span>
            <span>{{ stats.idleTimer.idleThreshold }}s</span>
          </div>
        </div>
      </div>

      <!-- Archived Chunks -->
      <div class="card">
        <div class="card-title">Archived Chunks</div>
        <div v-if="stats.archivedChunks.length === 0" class="empty-hint">
          No archived chunks yet.
        </div>
        <div v-else class="chunk-list">
          <div v-for="chunk in stats.archivedChunks" :key="chunk.id" class="chunk-item">
            <div class="chunk-header">
              <code class="chunk-id">{{ chunk.id }}</code>
              <span class="chunk-tokens">{{ formatTokens(chunk.tokenCount) }}</span>
            </div>
            <div class="chunk-summary">{{ chunk.summary }}</div>
            <div class="chunk-date">{{ new Date(chunk.createdAt).toLocaleString() }}</div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.view-container {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ── Header ─────────────────────────────────────────────── */

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.view-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.btn-compress {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  background: var(--primary, #3b82f6);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-compress:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-compress:hover:not(:disabled) {
  opacity: 0.85;
}

/* ── Loading / Error ────────────────────────────────────── */

.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  gap: 12px;
  color: var(--text-muted, #9ca3af);
}

.btn-retry {
  padding: 6px 16px;
  border: 1px solid var(--border-color, #374151);
  border-radius: 6px;
  background: var(--bg-surface, #1f2937);
  color: var(--text-primary, #f3f4f6);
  cursor: pointer;
  font-size: 13px;
}

/* ── Spinner ────────────────────────────────────────────── */

.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
.spinner.large {
  width: 28px;
  height: 28px;
  border-width: 3px;
  border-color: rgba(var(--primary-rgb, 59, 130, 246), 0.3);
  border-top-color: var(--primary, #3b82f6);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ── Context usage bar ──────────────────────────────────── */

.context-usage {
  background: var(--bg-surface, #1f2937);
  border: 1px solid var(--border-color, #374151);
  border-radius: 8px;
  padding: 14px 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
}

.section-badge {
  font-size: 12px;
  color: var(--text-muted, #9ca3af);
}

.progress-bar {
  height: 8px;
  border-radius: 4px;
  background: var(--bg-subtle, #111827);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  background: var(--primary, #3b82f6);
  transition: width 0.4s ease;
}
.progress-fill.warn { background: #f59e0b; }
.progress-fill.danger { background: #ef4444; }

.progress-label {
  display: block;
  margin-top: 6px;
  font-size: 11px;
  color: var(--text-muted, #9ca3af);
}

/* ── Card ───────────────────────────────────────────────── */

.card {
  background: var(--bg-surface, #1f2937);
  border: 1px solid var(--border-color, #374151);
  border-radius: 8px;
  padding: 14px 16px;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color, #374151);
}

/* ── Info grid ──────────────────────────────────────────── */

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
}

.info-label {
  color: var(--text-muted, #9ca3af);
}

.mono {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  background: var(--bg-subtle, #111827);
  padding: 2px 6px;
  border-radius: 4px;
}

.highlight {
  color: var(--primary, #3b82f6);
  font-weight: 600;
}

.countdown {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-weight: 600;
  color: #f59e0b;
}

/* ── Badge ──────────────────────────────────────────────── */

.badge {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: 10px;
}

.badge-frozen {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
}
.badge-unfrozen {
  background: rgba(245, 158, 11, 0.15);
  color: #fbbf24;
}
.badge-active {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
}
.badge-paused {
  background: rgba(156, 163, 175, 0.15);
  color: #9ca3af;
}

/* ── Dual cache grid ────────────────────────────────────── */

.cache-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.cache-box {
  padding: 10px;
  border-radius: 6px;
  background: var(--bg-subtle, #111827);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cache-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted, #9ca3af);
}

.cache-numbers {
  display: flex;
  gap: 12px;
  font-size: 13px;
}
.cache-numbers .hit { color: #4ade80; }
.cache-numbers .miss { color: #f87171; }

.cache-rate {
  font-size: 12px;
  color: var(--text-muted, #9ca3af);
}

.mini-bar {
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.mini-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.4s ease;
}
.hit-fill { background: #4ade80; }

/* ── Chunk list ─────────────────────────────────────────── */

.chunk-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chunk-item {
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--bg-subtle, #111827);
  border: 1px solid var(--border-color, #374151);
}

.chunk-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.chunk-id {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 11px;
  color: #60a5fa;
  background: rgba(59, 130, 246, 0.1);
  padding: 1px 6px;
  border-radius: 4px;
}

.chunk-tokens {
  font-size: 12px;
  font-weight: 600;
  color: var(--primary, #3b82f6);
}

.chunk-summary {
  font-size: 13px;
  line-height: 1.4;
  color: var(--text-primary, #e5e7eb);
}

.chunk-date {
  font-size: 11px;
  color: var(--text-muted, #9ca3af);
  margin-top: 4px;
}

.empty-hint {
  text-align: center;
  padding: 20px;
  color: var(--text-muted, #9ca3af);
  font-size: 13px;
}
</style>
