<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'vue-router';

const router = useRouter();

// Types
interface GameInfo {
  isGame: boolean;
  engine: string | null;
  size: string | null;
  scenes: string[];
  projectName: string;
  version: string | null;
}

interface GameStatus {
  gameId: string;
  engine: string;
  processId: number;
  isRunning: boolean;
  startedAt: number;
  elapsedMs: number;
  memoryMb: number | null;
}

interface BuildProgress {
  stage: string;
  progress: number;
  message: string;
}

// State
const cwd = ref('');
const gameInfo = ref<GameInfo | null>(null);
const runningGames = ref<GameStatus[]>([]);
const buildProgress = ref<BuildProgress | null>(null);
const isBuilding = ref(false);
const isLoading = ref(false);
const error = ref<string | null>(null);
const logs = ref<string[]>([]);

// Computed
const hasGame = computed(() => gameInfo.value?.isGame ?? false);
const engineName = computed(() => gameInfo.value?.engine ?? 'Unknown');
const gameSize = computed(() => gameInfo.value?.size ?? 'Unknown');
const isRunning = computed(() => runningGames.value.length > 0);

// Methods
async function detectGame() {
  if (!cwd.value) {
    error.value = 'Please select a project directory';
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    const info = await invoke<GameInfo>('game_detect', { cwd: cwd.value });
    gameInfo.value = info;
    addLog(`Detected ${info.engine} project: ${info.projectName}`);
  } catch (e) {
    error.value = `Failed to detect game: ${e}`;
    addLog(`Error: ${e}`);
  } finally {
    isLoading.value = false;
  }
}

async function exportGame(target: string) {
  if (!gameInfo.value) {
    error.value = 'No game detected';
    return;
  }

  isBuilding.value = true;
  buildProgress.value = {
    stage: 'preparing',
    progress: 0,
    message: 'Preparing build...',
  };
  logs.value = [];

  try {
    addLog(`Starting export for ${target}...`);

    // Simulate progress updates (in production, use WebSocket events)
    updateProgress('detecting', 0.1, 'Detecting game engine...');
    await sleep(500);

    updateProgress('preparing', 0.2, 'Preparing build environment...');
    await sleep(500);

    updateProgress('compiling', 0.3, 'Compiling scripts...');
    await sleep(1000);

    updateProgress('compiling', 0.5, 'Building resources...');
    await sleep(1000);

    updateProgress('linking', 0.7, 'Linking assemblies...');
    await sleep(500);

    updateProgress('finalizing', 0.9, 'Packaging game...');
    await sleep(500);

    const result = await invoke<any>('game_export', {
      request: {
        cwd: cwd.value,
        target,
        development: false,
      },
    });

    if (result.success) {
      updateProgress('complete', 1.0, 'Build completed successfully!');
      addLog(`✓ Build successful: ${result.outputPath}`);
      addLog(`  Build time: ${result.buildTimeMs}ms`);
      if (result.fileSizeMb) {
        addLog(`  File size: ${result.fileSizeMb.toFixed(2)} MB`);
      }
    } else {
      throw new Error(result.error || 'Build failed');
    }
  } catch (e) {
    updateProgress('failed', 0, `Build failed: ${e}`);
    addLog(`✗ Error: ${e}`);
    error.value = `Build failed: ${e}`;
  } finally {
    isBuilding.value = false;
  }
}

async function launchGame() {
  if (!gameInfo.value) {
    error.value = 'No game detected';
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    addLog('Launching game...');

    const result = await invoke<any>('game_launch', {
      request: {
        cwd: cwd.value,
        executable: null,
        args: [],
      },
    });

    if (result.success) {
      addLog(`✓ Game launched (PID: ${result.processId})`);
      addLog(`  Mode: ${result.mode}`);
      await refreshRunningGames();
    } else {
      throw new Error(result.error || 'Launch failed');
    }
  } catch (e) {
    error.value = `Launch failed: ${e}`;
    addLog(`✗ Error: ${e}`);
  } finally {
    isLoading.value = false;
  }
}

async function stopGame(gameId: string) {
  isLoading.value = true;
  error.value = null;

  try {
    addLog(`Stopping game ${gameId}...`);
    await invoke('game_stop', { gameId });
    addLog('✓ Game stopped');
    await refreshRunningGames();
  } catch (e) {
    error.value = `Stop failed: ${e}`;
    addLog(`✗ Error: ${e}`);
  } finally {
    isLoading.value = false;
  }
}

async function refreshRunningGames() {
  try {
    const games = await invoke<GameStatus[]>('game_list_running');
    runningGames.value = games;
  } catch (e) {
    console.error('Failed to refresh running games:', e);
  }
}

function updateProgress(stage: string, progress: number, message: string) {
  buildProgress.value = { stage, progress, message };
  addLog(`[${stage.toUpperCase()}] ${message}`);
}

function addLog(message: string) {
  const timestamp = new Date().toLocaleTimeString();
  logs.value.push(`[${timestamp}] ${message}`);
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// 跳转到开发助手（Executive Session）
function openDevAssistant() {
  if (!cwd.value) {
    error.value = '请先设置项目目录';
    return;
  }
  // 跳转到 ExecutiveSession，带上游戏项目路径和 godot mode
  router.push({
    path: '/executive-session',
    query: {
      workspace: cwd.value,
      mode: 'godot'
    }
  });
}

// Lifecycle
onMounted(() => {
  // Refresh running games every 5 seconds
  const interval = setInterval(refreshRunningGames, 5000);
  onUnmounted(() => clearInterval(interval));
});
</script>

<template>
  <div class="game-manager">
    <!-- Header -->
    <div class="header">
      <h2>🎮 Game Manager</h2>
      <div class="status">
        <button
          v-if="hasGame"
          class="dev-assistant-btn"
          @click="openDevAssistant"
          title="打开 AI 开发助手"
        >
          🤖 开发助手
        </button>
        <span v-if="hasGame" class="badge success">
          {{ engineName }} Project Detected
        </span>
        <span v-else class="badge warning">
          No Game Detected
        </span>
      </div>
    </div>

    <!-- Project Directory -->
    <div class="section">
      <label>Project Directory:</label>
      <div class="input-group">
        <input
          v-model="cwd"
          type="text"
          placeholder="Select or enter project directory..."
          @keyup.enter="detectGame"
        />
        <button @click="detectGame" :disabled="isLoading || !cwd">
          {{ isLoading ? 'Detecting...' : 'Detect' }}
        </button>
      </div>
    </div>

    <!-- Game Info -->
    <div v-if="gameInfo" class="section">
      <h3>Project Information</h3>
      <div class="info-grid">
        <div class="info-item">
          <span class="label">Project:</span>
          <span class="value">{{ gameInfo.projectName }}</span>
        </div>
        <div class="info-item">
          <span class="label">Engine:</span>
          <span class="value">{{ engineName }}</span>
        </div>
        <div class="info-item">
          <span class="label">Version:</span>
          <span class="value">{{ gameInfo.version || 'Unknown' }}</span>
        </div>
        <div class="info-item">
          <span class="label">Size:</span>
          <span class="value">{{ gameSize }}</span>
        </div>
        <div class="info-item">
          <span class="label">Scenes:</span>
          <span class="value">{{ gameInfo.scenes.length }}</span>
        </div>
      </div>
    </div>

    <!-- Build Progress -->
    <div v-if="buildProgress" class="section">
      <h3>Build Progress</h3>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${buildProgress.progress * 100}%` }"
          :class="{
            success: buildProgress.stage === 'complete',
            error: buildProgress.stage === 'failed',
          }"
        ></div>
      </div>
      <div class="progress-info">
        <span class="stage">{{ buildProgress.stage }}</span>
        <span class="percent">{{ Math.round(buildProgress.progress * 100) }}%</span>
      </div>
      <p class="message">{{ buildProgress.message }}</p>
    </div>

    <!-- Actions -->
    <div v-if="hasGame" class="section">
      <h3>Actions</h3>
      <div class="action-buttons">
        <!-- Export Buttons -->
        <div class="button-group">
          <h4>Export</h4>
          <button
            @click="exportGame('windows')"
            :disabled="isBuilding"
            class="btn-primary"
          >
            🪟 Windows
          </button>
          <button
            @click="exportGame('macos')"
            :disabled="isBuilding"
            class="btn-primary"
          >
            🍎 macOS
          </button>
          <button
            @click="exportGame('linux')"
            :disabled="isBuilding"
            class="btn-primary"
          >
            🐧 Linux
          </button>
          <button
            @click="exportGame('web')"
            :disabled="isBuilding"
            class="btn-secondary"
          >
            🌐 Web
          </button>
        </div>

        <!-- Launch Button -->
        <div class="button-group">
          <h4>Run</h4>
          <button
            @click="launchGame"
            :disabled="isLoading || isRunning"
            class="btn-success"
          >
            ▶ Launch Game
          </button>
        </div>
      </div>
    </div>

    <!-- Running Games -->
    <div v-if="runningGames.length > 0" class="section">
      <h3>Running Games</h3>
      <div class="game-list">
        <div v-for="game in runningGames" :key="game.gameId" class="game-item">
          <div class="game-info">
            <span class="engine-badge">{{ game.engine }}</span>
            <span class="pid">PID: {{ game.processId }}</span>
            <span class="status">
              {{ game.isRunning ? '🟢 Running' : '🔴 Stopped' }}
            </span>
            <span class="elapsed">
              {{ Math.round(game.elapsedMs / 1000) }}s
            </span>
            <span v-if="game.memoryMb" class="memory">
              {{ game.memoryMb.toFixed(0) }} MB
            </span>
          </div>
          <button
            @click="stopGame(game.gameId)"
            :disabled="!game.isRunning"
            class="btn-danger"
          >
            ⏹ Stop
          </button>
        </div>
      </div>
    </div>

    <!-- Logs -->
    <div class="section">
      <h3>Logs</h3>
      <div class="log-container">
        <div v-for="(log, index) in logs" :key="index" class="log-line">
          {{ log }}
        </div>
        <div v-if="logs.length === 0" class="log-empty">
          No logs yet
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-banner">
      <span class="error-icon">⚠️</span>
      <span class="error-message">{{ error }}</span>
      <button @click="error = null" class="close-btn">×</button>
    </div>
  </div>
</template>

<style scoped>
.game-manager {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 2px solid #e0e0e0;
}

.header h2 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.badge {
  padding: 6px 12px;
  border-radius: 16px;
  font-size: 14px;
  font-weight: 500;
}

.badge.success {
  background: #d4edda;
  color: #155724;
}

.badge.warning {
  background: #fff3cd;
  color: #856404;
}

.dev-assistant-btn {
  padding: 6px 12px;
  margin-right: 12px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.2s;
}

.dev-assistant-btn:hover {
  background: #0056b3;
}

.section {
  margin-bottom: 24px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
}

.section h3 {
  margin: 0 0 12px 0;
  font-size: 18px;
  color: #333;
}

.section h4 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #666;
}

.input-group {
  display: flex;
  gap: 8px;
}

.input-group input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.input-group input:focus {
  outline: none;
  border-color: #007bff;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: white;
  border-radius: 6px;
}

.info-item .label {
  font-weight: 500;
  color: #666;
}

.info-item .value {
  color: #333;
}

.progress-bar {
  height: 24px;
  background: #e0e0e0;
  border-radius: 12px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-fill {
  height: 100%;
  background: #007bff;
  transition: width 0.3s ease;
}

.progress-fill.success {
  background: #28a745;
}

.progress-fill.error {
  background: #dc3545;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
  color: #666;
  margin-bottom: 4px;
}

.message {
  margin: 0;
  font-size: 14px;
  color: #333;
}

.action-buttons {
  display: flex;
  gap: 24px;
}

.button-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

button {
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #0056b3;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #545b62;
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background: #218838;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.game-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.game-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: white;
  border-radius: 6px;
}

.game-info {
  display: flex;
  gap: 12px;
  align-items: center;
}

.engine-badge {
  padding: 4px 8px;
  background: #e9ecef;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.pid, .status, .elapsed, .memory {
  font-size: 14px;
  color: #666;
}

.log-container {
  max-height: 300px;
  overflow-y: auto;
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 6px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
}

.log-line {
  padding: 2px 0;
}

.log-empty {
  color: #666;
  font-style: italic;
}

.error-banner {
  position: fixed;
  bottom: 20px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-width: 400px;
}

.error-icon {
  font-size: 20px;
}

.error-message {
  flex: 1;
  color: #721c24;
  font-size: 14px;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 20px;
  color: #721c24;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
}

.close-btn:hover {
  color: #491217;
}
</style>
