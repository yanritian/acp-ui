<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue';
import { useI18n } from '@/locales';

// Tauri API imports (conditionally available)
let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null;
let listen: ((event: string, handler: (event: unknown) => void) => Promise<() => void>) | null = null;

// 检测是否在Tauri环境中
const isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// 初始化Tauri API
async function initTauriAPI(): Promise<boolean> {
  if (!isTauriEnv) {
    console.log('[ExecutiveSessionView] Not in Tauri environment');
    return false;
  }

  try {
    const core = await import('@tauri-apps/api/core');
    const event = await import('@tauri-apps/api/event');
    invoke = core.invoke;
    listen = event.listen;
    console.log('[ExecutiveSessionView] Tauri API initialized successfully');
    return true;
  } catch (e) {
    console.log('[ExecutiveSessionView] Tauri API import failed:', e);
    return false;
  }
}

const { t } = useI18n();

// Executive Agent 会话记录类型
interface ExecutionLog {
  timestamp: string;
  agentType: string;
  action: string;
  file?: string;
  contentPreview?: string;
  error?: string;
}

interface GeneratedFile {
  path: string;
  relativePath: string;
  content: string;
  lines: number;
  createdAt: string;
}

interface ExecutiveSession {
  taskId: string;
  request: string;
  workspace: string;
  status: 'running' | 'completed' | 'error';
  files: GeneratedFile[];
  logs: ExecutionLog[];
  summary?: string;
  startTime: string;
  endTime?: string;
}

// 状态
const sessions = ref<ExecutiveSession[]>([]);
const currentSession = ref<ExecutiveSession | null>(null);
const selectedFile = ref<GeneratedFile | null>(null);
const isLoading = ref(false);
const newRequest = ref('');

// 初始化工作目录
const WORKSPACE_PATH = 'D:/dingsun/acp-ui/workspace';

// 计算属性
const totalFiles = computed(() =>
  sessions.value.reduce((sum, s) => sum + s.files.length, 0)
);

const totalLines = computed(() =>
  sessions.value.reduce((sum, s) =>
    sum + s.files.reduce((fSum, f) => fSum + f.lines, 0), 0
  )
);

// Tauri 事件监听器引用
let unlisteners: (() => void)[] = [];

// 加载持久化的会话记录
async function loadPersistedSessions() {
  // Try Tauri invoke first (if in Tauri environment)
  if (invoke) {
    try {
      const records = await invoke('load_executive_sessions', { limit: 50 });

      for (const record of records as any[]) {
        const session: ExecutiveSession = {
          taskId: record.id,
          request: record.request,
          workspace: record.workspace,
          status: record.status as 'running' | 'completed' | 'error',
          files: record.files_json ? JSON.parse(record.files_json) : [],
          logs: record.logs_json ? JSON.parse(record.logs_json) : [],
          summary: record.summary,
          startTime: record.created_at,
          endTime: record.completed_at,
        };
        // Only add if not already in list
        if (!sessions.value.find(s => s.taskId === session.taskId)) {
          sessions.value.push(session);
        }
      }

      // Sort by start time (newest first)
      sessions.value.sort((a, b) =>
        new Date(b.startTime).getTime() - new Date(a.startTime).getTime()
      );

      console.log(`[ExecutiveSessionView] Loaded ${sessions.value.length} persisted sessions from Tauri`);
      return;
    } catch (e) {
      console.error('[ExecutiveSessionView] Failed to load from Tauri:', e);
    }
  }

  // Fallback: Use WebSocket to query database (works in web mode)
  console.log('[ExecutiveSessionView] Using WebSocket fallback to load sessions');
  try {
    const ws = new WebSocket('ws://localhost:1421');

    ws.onopen = () => {
      console.log('[ExecutiveSessionView] WebSocket connected');
      // Send list_executive_sessions command
      ws.send(JSON.stringify({
        id: 'load-sessions-' + Date.now(),
        type: 'command',  // Use 'type' not 'request_type' (server expects 'type')
        command: 'list_executive_sessions',
        token: null,  // No auth token needed when server has no token set
        payload: { limit: 50 }
      }));
    };

    ws.onmessage = (event) => {
      try {
        const response = JSON.parse(event.data);
        console.log('[ExecutiveSessionView] WebSocket response:', response);

        if (response.ok && response.data?.sessions) {
          for (const record of response.data.sessions as any[]) {
            const session: ExecutiveSession = {
              taskId: record.id,
              request: record.request,
              workspace: record.workspace,
              status: record.status as 'running' | 'completed' | 'error',
              files: record.files_json ? JSON.parse(record.files_json) : [],
              logs: record.logs_json ? JSON.parse(record.logs_json) : [],
              summary: record.summary,
              startTime: record.created_at,
              endTime: record.completed_at,
            };
            if (!sessions.value.find(s => s.taskId === session.taskId)) {
              sessions.value.push(session);
            }
          }

          sessions.value.sort((a, b) =>
            new Date(b.startTime).getTime() - new Date(a.startTime).getTime()
          );

          console.log(`[ExecutiveSessionView] Loaded ${sessions.value.length} sessions from WebSocket`);
        }
        ws.close();
      } catch (e) {
        console.error('[ExecutiveSessionView] Failed to parse WebSocket response:', e);
      }
    };

    ws.onerror = (error) => {
      console.error('[ExecutiveSessionView] WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('[ExecutiveSessionView] WebSocket closed');
    };
  } catch (e) {
    console.error('[ExecutiveSessionView] Failed to connect WebSocket:', e);
  }
}

// 保存会话到数据库
async function saveSessionToDatabase(session: ExecutiveSession) {
  if (!invoke) {
    console.log('[ExecutiveSessionView] invoke not available, skipping save');
    return;
  }

  try {

    const record = {
      id: session.taskId,
      request: session.request,
      workspace: session.workspace,
      status: session.status,
      summary: session.summary,
      files_json: JSON.stringify(session.files),
      logs_json: JSON.stringify(session.logs),
      created_at: session.startTime,
      completed_at: session.endTime,
    };

    await invoke('save_executive_session_record', { session: record });
    console.log(`[ExecutiveSessionView] Saved session ${session.taskId} to database`);
  } catch (e) {
    console.error('[ExecutiveSessionView] Failed to save session to database:', e);
  }
}

// 事件监听 - 初始化Tauri API并监听事件
onMounted(async () => {
  // 先初始化Tauri API
  const tauriReady = await initTauriAPI();

  // 初始化 Executive Agent（设置工作目录）
  if (tauriReady && invoke) {
    try {
      await invoke('init_executive_agent', { workspace: WORKSPACE_PATH });
      console.log('[ExecutiveSessionView] Executive Agent initialized with workspace:', WORKSPACE_PATH);
    } catch (e) {
      console.error('[ExecutiveSessionView] Failed to initialize Executive Agent:', e);
    }
  }

  // 加载持久化的会话
  await loadPersistedSessions();

  // 监听事件
  if (listen) {
    try {
      // 监听任务开始
      const unlisten1 = await listen('task-started', (event) => {
        const data = (event as any).payload as any;
        const session: ExecutiveSession = {
          taskId: data.taskId,
          request: data.request,
          workspace: data.workspace,
          status: 'running',
          files: [],
          logs: [],
          startTime: new Date().toISOString(),
        };
        sessions.value.unshift(session);
        currentSession.value = session;
        isLoading.value = true;
      });
      unlisteners.push(unlisten1);

    // 监听智能体消息
      const unlisten2 = await listen('agent-message', (event) => {
        const data = (event as any).payload as any;
        if (currentSession.value && data.taskId === currentSession.value.taskId) {
          currentSession.value.logs.push({
            timestamp: data.timestamp || new Date().toISOString(),
            agentType: data.agentType,
            action: 'message',
            contentPreview: data.content?.substring(0, 200),
          });
        }
      });
      unlisteners.push(unlisten2);

      // 监听文件创建
      const unlisten3 = await listen('file-created', (event) => {
        const data = (event as any).payload as any;
        if (currentSession.value) {
          currentSession.value.logs.push({
            timestamp: new Date().toISOString(),
            agentType: 'Coder',
            action: 'file_created',
            file: data.path,
            contentPreview: `${data.lines} 行`,
          });
        }
      });
      unlisteners.push(unlisten3);

      // 监听批量文件创建
      const unlisten4 = await listen('files-created', (event) => {
        const data = (event as any).payload as any;
        if (currentSession.value) {
          for (const file of data.files || []) {
            currentSession.value.logs.push({
              timestamp: new Date().toISOString(),
              agentType: data.agentType,
              action: 'file_created',
              file: file,
            });
          }
        }
      });
      unlisteners.push(unlisten4);

      // 监听任务完成
      const unlisten5 = await listen('task-completed', (event) => {
        const data = (event as any).payload as any;
        const session = sessions.value.find(s => s.taskId === data.taskId);
        if (session) {
          session.status = 'completed';
          session.endTime = new Date().toISOString();
          session.summary = data.summary;
          session.files = data.files?.map((f: any) => ({
            path: f.path,
            relativePath: f.relativePath || f,
            content: '',
            lines: 0,
            createdAt: new Date().toISOString(),
          })) || [];

          // 保存到数据库
          saveSessionToDatabase(session);
        }
        isLoading.value = false;
      });
      unlisteners.push(unlisten5);

      // 监听错误
      const unlisten6 = await listen('agent-error', (event) => {
        const data = (event as any).payload as any;
        if (currentSession.value) {
          currentSession.value.logs.push({
            timestamp: new Date().toISOString(),
            agentType: data.agentType,
            action: 'error',
            error: data.error,
          });
          currentSession.value.status = 'error';

          // 保存到数据库
          saveSessionToDatabase(currentSession.value);
        }
        isLoading.value = false;
      });
      unlisteners.push(unlisten6);

      // 监听来自Flutter WebSocket的remote-command事件
      const unlisten7 = await listen('remote-command', async (event) => {
        const data = (event as any).payload as any;
        console.log('[ExecutiveSessionView] Received remote-command:', data);

        if (data.type === 'execute_development_task' && data.request) {
          if (!invoke) return;
          // 调用真正的Tauri command执行任务
          try {
            isLoading.value = true;
            const result = await invoke('execute_development_task', { request: data.request });
            console.log('[ExecutiveSessionView] Task executed successfully:', result);
          } catch (e) {
            console.error('[ExecutiveSessionView] Task execution failed:', e);
            if (currentSession.value) {
              currentSession.value.logs.push({
                timestamp: new Date().toISOString(),
                agentType: 'System',
                action: 'error',
                error: String(e),
              });
              currentSession.value.status = 'error';
            }
            isLoading.value = false;
          }
        }
      });
      unlisteners.push(unlisten7);

    } catch (e) {
      console.log('[ExecutiveSessionView] Tauri event listeners not available:', e);
    }
  } else {
    console.log('[ExecutiveSessionView] listen not available, running in web mode');
  }
});

// 清理监听器
onUnmounted(() => {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
});

// 方法
function selectSession(session: ExecutiveSession) {
  currentSession.value = session;
  selectedFile.value = null;
}

function selectFile(file: GeneratedFile) {
  selectedFile.value = file;
}

function formatTime(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString();
}

function formatDate(timestamp: string): string {
  return new Date(timestamp).toLocaleString();
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'running': return '#ff9800';
    case 'completed': return '#4caf50';
    case 'error': return '#f44336';
    default: return '#9e9e9e';
  }
}

function getAgentIcon(agentType: string): string {
  switch (agentType.toLowerCase()) {
    case 'planner': return '📋';
    case 'architect': return '🏗️';
    case 'coder': return '💻';
    case 'coderewriter': return '👁️';
    case 'tester': return '🧪';
    case 'securityreviewer': return '🔒';
    default: return '🤖';
  }
}

function getActionIcon(action: string): string {
  switch (action) {
    case 'file_created': return '📄';
    case 'message': return '💬';
    case 'error': return '❌';
    default: return '📌';
  }
}

// 模拟测试数据
function addTestSession() {
  const testSession: ExecutiveSession = {
    taskId: 'test-' + Date.now(),
    request: '做一个ERP系统',
    workspace: 'D:/dingsun/acp-ui/erp_system',
    status: 'completed',
    files: [
      { path: 'D:/dingsun/acp-ui/erp_system/src/models/product.rs', relativePath: 'src/models/product.rs', content: 'pub struct Product { ... }', lines: 45, createdAt: new Date().toISOString() },
      { path: 'D:/dingsun/acp-ui/erp_system/src/api/handlers.rs', relativePath: 'src/api/handlers.rs', content: 'pub async fn get_products() { ... }', lines: 120, createdAt: new Date().toISOString() },
      { path: 'D:/dingsun/acp-ui/erp_system/tests/test_product.rs', relativePath: 'tests/test_product.rs', content: '#[test] fn test_product_creation() { ... }', lines: 35, createdAt: new Date().toISOString() },
    ],
    logs: [
      { timestamp: new Date(Date.now() - 60000).toISOString(), agentType: 'Planner', action: 'message', contentPreview: '分析需求：ERP系统核心模块...' },
      { timestamp: new Date(Date.now() - 50000).toISOString(), agentType: 'Architect', action: 'message', contentPreview: '设计架构：模块化结构...' },
      { timestamp: new Date(Date.now() - 40000).toISOString(), agentType: 'Coder', action: 'file_created', file: 'src/models/product.rs' },
      { timestamp: new Date(Date.now() - 30000).toISOString(), agentType: 'Coder', action: 'file_created', file: 'src/api/handlers.rs' },
      { timestamp: new Date(Date.now() - 20000).toISOString(), agentType: 'Tester', action: 'file_created', file: 'tests/test_product.rs' },
      { timestamp: new Date(Date.now() - 10000).toISOString(), agentType: 'CodeReviewer', action: 'message', contentPreview: '代码审查完成，无重大问题...' },
    ],
    summary: '✅ ERP系统核心模块已完成，包含商品管理API和单元测试',
    startTime: new Date(Date.now() - 60000).toISOString(),
    endTime: new Date().toISOString(),
  };
  sessions.value.unshift(testSession);
  currentSession.value = testSession;

  // 保存到数据库
  saveSessionToDatabase(testSession);
}

// 执行新任务
async function executeTask() {
  if (!newRequest.value.trim() || !invoke) {
    console.log('[ExecutiveSessionView] No request or invoke not available');
    return;
  }

  const request = newRequest.value.trim();
  newRequest.value = '';
  isLoading.value = true;

  try {
    console.log('[ExecutiveSessionView] Executing task:', request);
    const result = await invoke('execute_development_task', { request });
    console.log('[ExecutiveSessionView] Task result:', result);
  } catch (e) {
    console.error('[ExecutiveSessionView] Task execution failed:', e);
    if (currentSession.value) {
      currentSession.value.logs.push({
        timestamp: new Date().toISOString(),
        agentType: 'System',
        action: 'error',
        error: String(e),
      });
      currentSession.value.status = 'error';
    }
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="executive-session-view">
    <!-- 标题栏 -->
    <div class="header">
      <h2>🚀 Executive Agent 会话记录</h2>
      <div class="stats">
        <span class="stat-item">
          <span class="stat-icon">📝</span>
          {{ sessions.length }} 个会话
        </span>
        <span class="stat-item">
          <span class="stat-icon">📄</span>
          {{ totalFiles }} 个文件
        </span>
        <span class="stat-item">
          <span class="stat-icon">📊</span>
          {{ totalLines }} 行代码
        </span>
      </div>
      <button class="test-btn" @click="addTestSession">+ 添加测试会话</button>
    </div>

    <!-- 任务输入区 -->
    <div class="task-input-section">
      <div class="input-row">
        <input
          v-model="newRequest"
          type="text"
          placeholder="输入开发需求，例如：做一个简单的计算器..."
          class="task-input"
          :disabled="isLoading"
          @keyup.enter="executeTask"
        />
        <button
          class="execute-btn"
          :disabled="isLoading || !newRequest.trim()"
          @click="executeTask"
        >
          {{ isLoading ? '执行中...' : '执行' }}
        </button>
      </div>
      <p class="input-hint">Agent 将使用阿里百炼云 qwen3.6-plus 模型执行任务</p>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 左侧：会话列表 -->
      <div class="session-list-panel">
        <h3>会话列表</h3>
        <div class="session-list">
          <div v-if="sessions.length === 0" class="empty-state">
            <p>暂无会话记录</p>
            <p class="hint">发送需求后，Agent 执行记录将在这里显示</p>
          </div>
          <div
            v-for="session in sessions"
            :key="session.taskId"
            class="session-card"
            :class="{ active: currentSession?.taskId === session.taskId }"
            @click="selectSession(session)"
          >
            <div class="session-header">
              <span class="session-status" :style="{ color: getStatusColor(session.status) }">
                {{ session.status === 'running' ? '⏳' : session.status === 'completed' ? '✅' : '❌' }}
              </span>
              <span class="session-request">{{ session.request }}</span>
            </div>
            <div class="session-meta">
              <span class="meta-item">📄 {{ session.files.length }} 个文件</span>
              <span class="meta-item">📝 {{ session.logs.length }} 条日志</span>
            </div>
            <div class="session-time">
              {{ formatDate(session.startTime) }}
            </div>
          </div>
        </div>
      </div>

      <!-- 中间：执行详情 -->
      <div class="session-detail-panel">
        <div v-if="!currentSession" class="empty-detail">
          <p>选择一个会话查看详情</p>
        </div>
        <div v-else class="session-detail">
          <!-- 需求和摘要 -->
          <div class="request-section">
            <h4>📋 原始需求</h4>
            <p class="request-text">{{ currentSession.request }}</p>
          </div>

          <div v-if="currentSession.summary" class="summary-section">
            <h4>✅ 执行摘要</h4>
            <p class="summary-text">{{ currentSession.summary }}</p>
          </div>

          <!-- 执行日志时间线 -->
          <div class="logs-section">
            <h4>⏱️ 执行时间线</h4>
            <div class="timeline">
              <div
                v-for="(log, index) in currentSession.logs"
                :key="index"
                class="timeline-item"
              >
                <div class="timeline-marker">
                  {{ getAgentIcon(log.agentType) }}
                </div>
                <div class="timeline-content">
                  <div class="timeline-header">
                    <span class="agent-type">{{ log.agentType }}</span>
                    <span class="timeline-time">{{ formatTime(log.timestamp) }}</span>
                  </div>
                  <div class="timeline-body">
                    <span class="action-icon">{{ getActionIcon(log.action) }}</span>
                    <span v-if="log.file" class="file-name">{{ log.file }}</span>
                    <span v-if="log.contentPreview" class="content-preview">{{ log.contentPreview }}</span>
                    <span v-if="log.error" class="error-text">{{ log.error }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧：文件列表 -->
      <div class="files-panel">
        <div v-if="!currentSession" class="empty-files">
          <p>选择会话查看生成的文件</p>
        </div>
        <div v-else class="files-content">
          <h4>📁 生成的文件 ({{ currentSession.files.length }})</h4>
          <div class="file-list">
            <div
              v-for="file in currentSession.files"
              :key="file.path"
              class="file-item"
              :class="{ active: selectedFile?.path === file.path }"
              @click="selectFile(file)"
            >
              <span class="file-icon">📄</span>
              <div class="file-info">
                <span class="file-name">{{ file.relativePath }}</span>
                <span class="file-meta">{{ file.lines }} 行</span>
              </div>
            </div>
          </div>

          <!-- 文件预览 -->
          <div v-if="selectedFile" class="file-preview">
            <h5>{{ selectedFile.relativePath }}</h5>
            <pre class="code-preview"><code>{{ selectedFile.content || '// 点击文件查看代码内容' }}</code></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 加载指示器 -->
    <div v-if="isLoading" class="loading-overlay">
      <div class="loading-spinner"></div>
      <p>Agent 正在执行...</p>
    </div>
  </div>
</template>

<style scoped>
.executive-session-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary, #fff);
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.header h2 {
  font-size: 1.25rem;
  margin: 0;
}

.stats {
  display: flex;
  gap: 1rem;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  color: var(--text-secondary, #666);
}

.stat-icon {
  font-size: 1rem;
}

.test-btn {
  padding: 0.5rem 1rem;
  border: 1px solid var(--primary-color, #0066cc);
  border-radius: 6px;
  background: transparent;
  color: var(--primary-color, #0066cc);
  cursor: pointer;
}

.test-btn:hover {
  background: var(--primary-color, #0066cc);
  color: white;
}

.task-input-section {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
  background: var(--bg-secondary, #f5f5f5);
}

.input-row {
  display: flex;
  gap: 0.5rem;
}

.task-input {
  flex: 1;
  padding: 0.75rem 1rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  font-size: 0.875rem;
  outline: none;
}

.task-input:focus {
  border-color: var(--primary-color, #0066cc);
}

.task-input:disabled {
  background: var(--bg-disabled, #e0e0e0);
}

.execute-btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 8px;
  background: var(--primary-color, #0066cc);
  color: white;
  cursor: pointer;
  font-weight: 500;
}

.execute-btn:hover:not(:disabled) {
  background: var(--primary-dark, #0052a3);
}

.execute-btn:disabled {
  background: var(--bg-disabled, #e0e0e0);
  color: var(--text-muted, #999);
  cursor: not-allowed;
}

.input-hint {
  margin: 0.5rem 0 0 0;
  font-size: 0.75rem;
  color: var(--text-muted, #999);
}

.main-content {
  display: grid;
  grid-template-columns: 300px 1fr 300px;
  flex: 1;
  overflow: hidden;
}

.session-list-panel,
.session-detail-panel,
.files-panel {
  padding: 1rem;
  border-right: 1px solid var(--border-color, #e0e0e0);
  overflow-y: auto;
}

.files-panel {
  border-right: none;
}

.session-list-panel h3,
.session-detail-panel h4,
.files-panel h4 {
  font-size: 0.875rem;
  color: var(--text-secondary, #666);
  margin: 0 0 1rem 0;
}

.empty-state,
.empty-detail,
.empty-files {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted, #999);
}

.hint {
  font-size: 0.75rem;
  margin-top: 0.5rem;
}

.session-card {
  padding: 0.75rem;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  margin-bottom: 0.5rem;
  cursor: pointer;
  transition: all 0.15s;
}

.session-card:hover {
  background: var(--bg-hover, #f5f5f5);
}

.session-card.active {
  border-color: var(--primary-color, #0066cc);
  background: var(--primary-light, #e6f2ff);
}

.session-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-status {
  font-size: 1rem;
}

.session-request {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-meta {
  display: flex;
  gap: 0.75rem;
  margin-top: 0.25rem;
  font-size: 0.75rem;
  color: var(--text-secondary, #666);
}

.session-time {
  font-size: 0.75rem;
  color: var(--text-muted, #999);
  margin-top: 0.25rem;
}

.session-detail {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.request-section,
.summary-section {
  padding: 1rem;
  border-radius: 8px;
  background: var(--bg-secondary, #f5f5f5);
}

.request-text {
  margin: 0.5rem 0 0 0;
  font-size: 1rem;
}

.summary-text {
  margin: 0.5rem 0 0 0;
  font-size: 0.875rem;
  white-space: pre-wrap;
}

.logs-section {
  flex: 1;
}

.timeline {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.timeline-item {
  display: flex;
  gap: 0.75rem;
}

.timeline-marker {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--bg-secondary, #f5f5f5);
  font-size: 1rem;
}

.timeline-content {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  background: var(--bg-secondary, #f5f5f5);
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.agent-type {
  font-weight: 500;
  font-size: 0.875rem;
}

.timeline-time {
  font-size: 0.75rem;
  color: var(--text-muted, #999);
}

.timeline-body {
  margin-top: 0.25rem;
  font-size: 0.875rem;
}

.action-icon {
  margin-right: 0.25rem;
}

.file-name {
  color: var(--primary-color, #0066cc);
  cursor: pointer;
}

.content-preview {
  color: var(--text-secondary, #666);
  display: block;
  margin-top: 0.25rem;
  font-size: 0.75rem;
  white-space: pre-wrap;
}

.error-text {
  color: var(--error-color, #f44336);
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.file-item:hover {
  background: var(--bg-hover, #f5f5f5);
}

.file-item.active {
  background: var(--primary-light, #e6f2ff);
}

.file-icon {
  font-size: 1rem;
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.file-name {
  font-size: 0.875rem;
}

.file-meta {
  font-size: 0.75rem;
  color: var(--text-muted, #999);
}

.file-preview {
  margin-top: 1rem;
  padding: 1rem;
  border-radius: 8px;
  background: var(--bg-code, #1e1e1e);
}

.file-preview h5 {
  margin: 0 0 0.5rem 0;
  font-size: 0.75rem;
  color: var(--text-secondary, #666);
}

.code-preview {
  margin: 0;
  padding: 0.5rem;
  font-size: 0.75rem;
  color: var(--code-text, #d4d4d4);
  white-space: pre-wrap;
  overflow-x: auto;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--primary-color, #0066cc);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>