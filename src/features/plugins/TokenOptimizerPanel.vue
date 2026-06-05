<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useTokenOptimizerStore } from '@/stores/token-optimizer'

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

interface ArchivedChunkDisplay {
  id: string
  tokenCount: number
  createdAt: string
  summary: string
}

interface IdleTimerStatus {
  active: boolean
  nextCompressionIn: number
  idleThreshold: number
}

interface TokenStats {
  systemPrompt: SystemPromptInfo
  cache: CacheStats
  compression: CompressionStats
  archivedChunks: ArchivedChunkDisplay[]
  idleTimer: IdleTimerStatus
  totalTokensUsed: number
  contextWindowLimit: number
}

// ── Store ────────────────────────────────────────────────────────────

const store = useTokenOptimizerStore()

// ── Local state ──────────────────────────────────────────────────────

const budgetInputValue = ref(store.tokenBudgetLimit)
const showBudgetInput = ref(false)
const stats = ref<TokenStats | null>(null)

// ── Computed from store ──────────────────────────────────────────────

const loading = computed(() => store.loading)
const error = computed(() => store.error)
const compressing = computed(() => store.compressing)
const clearingCache = computed(() => store.clearingCache)
const contextUsagePercent = computed(() => store.contextUsagePercent)
const trendData = computed(() => store.trendData)
const trendTimeRange = computed(() => store.trendTimeRange)

// Sync store state to local stats ref
watch(
  () => store.currentState,
  (newState) => {
    if (newState) {
      stats.value = {
        systemPrompt: newState.systemPrompt,
        cache: newState.cache,
        compression: newState.compression,
        archivedChunks: newState.archivedChunks.map(chunk => ({
          id: chunk.id,
          tokenCount: estimateTokens(chunk.compressedSummary),
          createdAt: chunk.timestamp,
          summary: chunk.compressedSummary,
        })),
        idleTimer: newState.idleTimer,
        totalTokensUsed: newState.totalTokensUsed,
        contextWindowLimit: newState.contextWindowLimit,
      }
    }
  },
  { immediate: true }
)

// ── Helpers ───────────────────────────────────────────────────────────

import { estimateTokens } from '@/lib/agent-runtime/token-optimizer'

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return `${n}`
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

// Chart helpers
function getTrendLabels(): string[] {
  return trendData.value.map(d => {
    if ('hour' in d) return d.hour
    if ('date' in d) return d.date.slice(5)
    return ''
  })
}

function getTrendValues(): number[] {
  return trendData.value.map(d => d.totalTokens)
}

function getMaxTrendValue(): number {
  return Math.max(...getTrendValues(), 1)
}

// ── Actions ───────────────────────────────────────────────────────────

async function compressNow() {
  await store.compressNow()
}

async function clearCache() {
  await store.clearCache()
}

function toggleTrendRange() {
  store.toggleTrendTimeRange()
}

function showBudgetEdit() {
  showBudgetInput.value = true
  budgetInputValue.value = store.tokenBudgetLimit
}

async function saveBudget() {
  const newLimit = parseInt(budgetInputValue.value.toString(), 10)
  if (newLimit > 0 && newLimit <= 500_000) {
    await store.updateBudgetLimit(newLimit)
  }
  showBudgetInput.value = false
}

function cancelBudgetEdit() {
  showBudgetInput.value = false
  budgetInputValue.value = store.tokenBudgetLimit
}

function retryFetch() {
  store.fetchStats()
}

// ── Lifecycle ─────────────────────────────────────────────────────────

onMounted(() => {
  store.startCountdown()
})

onUnmounted(() => {
  store.stopCountdown()
})
</script>

<template>
  <div class="view-container">
    <div class="view-header">
      <h3>Token Optimizer</h3>
      <div class="header-actions">
        <button class="btn-compress" :disabled="compressing" @click="compressNow">
          <span v-if="compressing" class="spinner" />
          {{ compressing ? '压缩中...' : '压缩当前上下文' }}
        </button>
        <button class="btn-clear" :disabled="clearingCache" @click="clearCache">
          <span v-if="clearingCache" class="spinner" />
          {{ clearingCache ? '清除中...' : '清除缓存' }}
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="loading-state">
      <span class="spinner large" />
      <p>加载统计数据...</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="error-state">
      <p>{{ error }}</p>
      <button class="btn-retry" @click="retryFetch">重试</button>
    </div>

    <!-- Stats -->
    <template v-else-if="stats">
      <!-- Context usage bar with budget setting -->
      <div class="section context-usage">
        <div class="section-header">
          <span class="section-title">上下文窗口</span>
          <div class="budget-control">
            <span v-if="!showBudgetInput" class="section-badge" @click="showBudgetEdit">
              {{ formatTokens(stats.totalTokensUsed) }} / {{ formatTokens(stats.contextWindowLimit) }}
              <span class="edit-icon">✎</span>
            </span>
            <div v-else class="budget-input-group">
              <input
                v-model.number="budgetInputValue"
                type="number"
                class="budget-input"
                min="1000"
                max="500000"
                step="10000"
              />
              <button class="btn-save-budget" @click="saveBudget">保存</button>
              <button class="btn-cancel-budget" @click="cancelBudgetEdit">取消</button>
            </div>
          </div>
        </div>
        <div class="progress-bar">
          <div
            class="progress-fill"
            :class="{ warn: contextUsagePercent > 75, danger: contextUsagePercent > 90 }"
            :style="{ width: contextUsagePercent + '%' }"
          />
        </div>
        <span class="progress-label">{{ contextUsagePercent.toFixed(1) }}% 已使用</span>
      </div>

      <!-- Token Usage Trend Chart -->
      <div class="card trend-card">
        <div class="card-title-row">
          <span class="card-title">Token 用量趋势</span>
          <button class="btn-toggle-range" @click="toggleTrendRange">
            {{ trendTimeRange === 'hour' ? '按小时' : '按天' }}
          </button>
        </div>
        <div class="trend-chart">
          <div class="chart-container">
            <div class="chart-y-axis">
              <span>{{ formatTokens(getMaxTrendValue()) }}</span>
              <span>0</span>
            </div>
            <div class="chart-bars">
              <div
                v-for="(value, idx) in getTrendValues()"
                :key="idx"
                class="bar"
                :style="{ height: (value / getMaxTrendValue() * 100) + '%' }"
              >
                <span class="bar-tooltip">{{ formatTokens(value) }}</span>
              </div>
            </div>
          </div>
          <div class="chart-x-axis">
            <span v-for="(label, idx) in getTrendLabels()" :key="idx">{{ label }}</span>
          </div>
        </div>
        <div class="trend-stats">
          <div class="trend-stat-item">
            <span class="trend-label">输入</span>
            <span class="trend-value input">{{ formatTokens(trendData.reduce((sum, d) => sum + d.inputTokens, 0)) }}</span>
          </div>
          <div class="trend-stat-item">
            <span class="trend-label">输出</span>
            <span class="trend-value output">{{ formatTokens(trendData.reduce((sum, d) => sum + d.outputTokens, 0)) }}</span>
          </div>
          <div class="trend-stat-item">
            <span class="trend-label">缓存</span>
            <span class="trend-value cached">{{ formatTokens(trendData.reduce((sum, d) => sum + d.cachedTokens, 0)) }}</span>
          </div>
        </div>
      </div>

      <!-- System Prompt -->
      <div class="card">
        <div class="card-title">系统提示</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">状态</span>
            <span class="badge" :class="stats.systemPrompt.frozen ? 'badge-frozen' : 'badge-unfrozen'">
              {{ stats.systemPrompt.frozen ? '已冻结' : '未冻结' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">Hash</span>
            <code class="mono">{{ stats.systemPrompt.hash }}</code>
          </div>
          <div class="info-row">
            <span class="info-label">大小</span>
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
        <div class="card-title">双缓存统计</div>
        <div class="cache-grid">
          <div class="cache-box">
            <div class="cache-label">系统缓存</div>
            <div class="cache-numbers">
              <span class="hit">{{ stats.cache.systemCacheHits }} 命中</span>
              <span class="miss">{{ stats.cache.systemCacheMisses }} 未命中</span>
            </div>
            <div class="cache-rate">命中率: {{ hitRate(stats.cache.systemCacheHits, stats.cache.systemCacheMisses) }}</div>
            <div class="mini-bar">
              <div
                class="mini-fill hit-fill"
                :style="{ width: hitRate(stats.cache.systemCacheHits, stats.cache.systemCacheMisses).replace('%','').replace('N/A','0') + '%' }"
              />
            </div>
          </div>
          <div class="cache-box">
            <div class="cache-label">上下文缓存</div>
            <div class="cache-numbers">
              <span class="hit">{{ stats.cache.contextCacheHits }} 命中</span>
              <span class="miss">{{ stats.cache.contextCacheMisses }} 未命中</span>
            </div>
            <div class="cache-rate">命中率: {{ hitRate(stats.cache.contextCacheHits, stats.cache.contextCacheMisses) }}</div>
            <div class="mini-bar">
              <div
                class="mini-fill hit-fill"
                :style="{ width: hitRate(stats.cache.contextCacheHits, stats.cache.contextCacheMisses).replace('%','').replace('N/A','0') + '%' }"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Compression Stats -->
      <div class="card">
        <div class="card-title">压缩统计</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">总共节省</span>
            <span class="highlight">{{ formatTokens(stats.compression.totalTokensSaved) }} tokens</span>
          </div>
          <div class="info-row">
            <span class="info-label">压缩率</span>
            <span>{{ (stats.compression.compressionRatio * 100).toFixed(1) }}%</span>
          </div>
          <div class="info-row">
            <span class="info-label">压缩次数</span>
            <span>{{ stats.compression.totalCompressions }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">平均节省</span>
            <span>{{ formatTokens(stats.compression.avgTokensPerCompression) }}/次</span>
          </div>
        </div>
      </div>

      <!-- Idle Compression Timer -->
      <div class="card">
        <div class="card-title">空闲压缩计时器</div>
        <div class="info-grid">
          <div class="info-row">
            <span class="info-label">状态</span>
            <span class="badge" :class="stats.idleTimer.active ? 'badge-active' : 'badge-paused'">
              {{ stats.idleTimer.active ? '活动' : '暂停' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">下次压缩</span>
            <span class="countdown">{{ stats.idleTimer.nextCompressionIn }}s</span>
          </div>
          <div class="info-row">
            <span class="info-label">空闲阈值</span>
            <span>{{ stats.idleTimer.idleThreshold }}s</span>
          </div>
        </div>
      </div>

      <!-- Archived Chunks -->
      <div class="card">
        <div class="card-title">已归档块</div>
        <div v-if="stats.archivedChunks.length === 0" class="empty-hint">
          暂无已归档的上下文块。
        </div>
        <div v-else class="chunk-list">
          <div v-for="chunk in stats.archivedChunks" :key="chunk.id" class="chunk-item">
            <div class="chunk-header">
              <code class="chunk-id">{{ chunk.id }}</code>
              <span class="chunk-tokens">{{ formatTokens(chunk.tokenCount) }} tokens</span>
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
  flex-wrap: wrap;
  gap: 12px;
}

.view-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-compress {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
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

.btn-clear {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid var(--border-color, #374151);
  border-radius: 6px;
  background: var(--bg-surface, #1f2937);
  color: var(--text-primary, #f3f4f6);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-clear:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-clear:hover:not(:disabled) {
  background: var(--bg-subtle, #111827);
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
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.edit-icon {
  font-size: 10px;
  opacity: 0.6;
}

.budget-control {
  display: flex;
  align-items: center;
}

.budget-input-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.budget-input {
  width: 100px;
  padding: 4px 8px;
  border: 1px solid var(--border-color, #374151);
  border-radius: 4px;
  background: var(--bg-subtle, #111827);
  color: var(--text-primary, #f3f4f6);
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
}

.btn-save-budget,
.btn-cancel-budget {
  padding: 4px 10px;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.btn-save-budget {
  background: var(--primary, #3b82f6);
  color: #fff;
}

.btn-cancel-budget {
  background: var(--bg-subtle, #111827);
  color: var(--text-muted, #9ca3af);
  border: 1px solid var(--border-color, #374151);
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

/* ── Trend Chart ────────────────────────────────────────── */

.trend-card {
  padding: 14px 16px;
}

.card-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color, #374151);
}

.btn-toggle-range {
  padding: 4px 12px;
  border: 1px solid var(--border-color, #374151);
  border-radius: 4px;
  background: var(--bg-subtle, #111827);
  color: var(--text-muted, #9ca3af);
  font-size: 12px;
  cursor: pointer;
}

.trend-chart {
  margin-bottom: 12px;
}

.chart-container {
  display: flex;
  height: 80px;
  gap: 8px;
}

.chart-y-axis {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  font-size: 10px;
  color: var(--text-muted, #9ca3af);
  width: 50px;
}

.chart-bars {
  flex: 1;
  display: flex;
  align-items: flex-end;
  gap: 4px;
  background: var(--bg-subtle, #111827);
  border-radius: 4px;
  padding: 4px;
}

.bar {
  flex: 1;
  min-width: 8px;
  background: var(--primary, #3b82f6);
  border-radius: 2px 2px 0 0;
  transition: height 0.3s ease;
  position: relative;
}

.bar:hover {
  background: #60a5fa;
}

.bar-tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-surface, #1f2937);
  border: 1px solid var(--border-color, #374151);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 10px;
  color: var(--text-primary, #f3f4f6);
  opacity: 0;
  transition: opacity 0.2s;
  white-space: nowrap;
}

.bar:hover .bar-tooltip {
  opacity: 1;
}

.chart-x-axis {
  display: flex;
  justify-content: space-around;
  margin-top: 4px;
  font-size: 9px;
  color: var(--text-muted, #9ca3af);
}

.trend-stats {
  display: flex;
  gap: 16px;
  justify-content: center;
}

.trend-stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.trend-label {
  font-size: 11px;
  color: var(--text-muted, #9ca3af);
}

.trend-value {
  font-size: 13px;
  font-weight: 600;
}

.trend-value.input { color: #60a5fa; }
.trend-value.output { color: #4ade80; }
.trend-value.cached { color: #fbbf24; }

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