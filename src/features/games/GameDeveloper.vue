<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Types
interface CodeFile {
  path: string;
  content: string;
  language: string;
}

interface GeneratedCode {
  files: CodeFile[];
  total_lines: number;
  language: string;
  engine: string;
}

interface EngineInfo {
  name: string;
  display_name: string;
  language: string;
  description: string;
  best_for: string[];
}

// State
const gddJson = ref('');
const outputFormat = ref('MultipleFiles');
const isGenerating = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);
const generatedCode = ref<GeneratedCode | null>(null);
const selectedFile = ref<string | null>(null);
const supportedEngines = ref<EngineInfo[]>([]);

// Computed
const selectedFileContent = computed(() => {
  if (!generatedCode.value || !selectedFile.value) return '';
  const file = generatedCode.value.files.find(f => f.path === selectedFile.value);
  return file ? file.content : '';
});

const selectedFileLanguage = computed(() => {
  if (!generatedCode.value || !selectedFile.value) return '';
  const file = generatedCode.value.files.find(f => f.path === selectedFile.value);
  return file ? file.language : '';
});

// Methods
async function loadSupportedEngines() {
  try {
    const result = await invoke<{
      success: boolean;
      engines: EngineInfo[] | null;
      error: string | null;
    }>('game_developer_get_supported_engines', {
      request: {},
    });

    if (result.success && result.engines) {
      supportedEngines.value = result.engines;
    } else {
      showError(result.error || '加载引擎列表失败');
    }
  } catch (e) {
    showError(`错误: ${e}`);
  }
}

async function generateCode() {
  if (!gddJson.value.trim()) {
    showError('请输入 GDD JSON');
    return;
  }

  isGenerating.value = true;
  error.value = null;
  success.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      code_json: string | null;
      error: string | null;
    }>('game_developer_generate_code', {
      request: {
        gdd_json: gddJson.value,
        output_format: outputFormat.value,
        target_files: null,
      },
    });

    if (result.success && result.code_json) {
      generatedCode.value = JSON.parse(result.code_json);
      if (generatedCode.value!.files.length > 0) {
        selectedFile.value = generatedCode.value!.files[0].path;
      }
      showSuccess(`代码生成成功！共 ${generatedCode.value!.total_lines} 行`);
    } else {
      showError(result.error || '生成失败');
    }
  } catch (e) {
    showError(`错误: ${e}`);
  } finally {
    isGenerating.value = false;
  }
}

async function generateSpecificFile(fileType: string) {
  if (!gddJson.value.trim()) {
    showError('请输入 GDD JSON');
    return;
  }

  isGenerating.value = true;
  error.value = null;
  success.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      file_content: string | null;
      file_path: string | null;
      error: string | null;
    }>('game_developer_generate_file', {
      request: {
        gdd_json: gddJson.value,
        file_type: fileType,
      },
    });

    if (result.success && result.file_content) {
      showSuccess(`文件 ${result.file_path} 生成成功`);
      // Display the file content
      if (!generatedCode.value) {
        generatedCode.value = {
          files: [{
            path: result.file_path || 'unknown',
            content: result.file_content,
            language: fileType,
          }],
          total_lines: result.file_content.split('\n').length,
          language: fileType,
          engine: 'Unknown',
        };
      }
      selectedFile.value = result.file_path;
    } else {
      showError(result.error || '生成失败');
    }
  } catch (e) {
    showError(`错误: ${e}`);
  } finally {
    isGenerating.value = false;
  }
}

function copyFileContent() {
  if (!selectedFileContent.value) return;

  navigator.clipboard.writeText(selectedFileContent.value).then(() => {
    showSuccess('代码已复制到剪贴板');
  }).catch(() => {
    showError('复制失败');
  });
}

function copyAllCode() {
  if (!generatedCode.value) return;

  const allCode = generatedCode.value.files.map(f =>
    `// File: ${f.path}\n// Language: ${f.language}\n\n${f.content}`
  ).join('\n\n' + '='.repeat(80) + '\n\n');

  navigator.clipboard.writeText(allCode).then(() => {
    showSuccess('所有代码已复制到剪贴板');
  }).catch(() => {
    showError('复制失败');
  });
}

function loadExampleGDD() {
  const exampleGDD = {
    concept: {
      title: "示例视觉小说",
      genre: "Visual Novel",
      core_idea: "一个校园恋爱故事",
      target_audience: "General",
      estimated_scope: "Medium"
    },
    core_mechanics: [
      {
        name: "对话系统",
        description: "角色对话与分支选择",
        implementation_priority: "Critical",
        code_hints: "使用 menu: 实现选项"
      }
    ],
    systems: [
      {
        name: "对话系统",
        components: ["DialogueManager", "CharacterDisplay"],
        interactions: ["显示对话文本", "处理玩家选择"],
        code_structure: "状态机管理对话流程"
      }
    ],
    asset_requirements: [
      {
        name: "角色立绘",
        asset_type: "Sprite2D",
        description: "角色头像和表情",
        style: "Anime",
        quantity: 5,
        specifications: "PNG with transparency"
      }
    ],
    technical_architecture: {
      engine: "RenPy",
      programming_language: "Ren'Py Script",
      project_type: "Visual Novel",
      scenes: ["MainMenu", "Chapter1", "Chapter2", "Ending"],
      networking: "SinglePlayer"
    },
    development_phases: [
      {
        phase_name: "原型",
        tasks: ["设置项目", "核心机制", "测试"],
        estimated_days: 7
      }
    ]
  };

  gddJson.value = JSON.stringify(exampleGDD, null, 2);
  showSuccess('已加载示例 GDD');
}

function reset() {
  gddJson.value = '';
  generatedCode.value = null;
  selectedFile.value = null;
  error.value = null;
  success.value = null;
}

function showSuccess(message: string) {
  success.value = message;
  error.value = null;
  setTimeout(() => {
    success.value = null;
  }, 5000);
}

function showError(message: string) {
  error.value = message;
  success.value = null;
  setTimeout(() => {
    error.value = null;
  }, 5000);
}

function getLanguageIcon(language: string): string {
  switch (language.toLowerCase()) {
    case 'renpy': return '🎭';
    case 'gdscript': return '🎮';
    case 'csharp': return '🎯';
    case 'c++': return '🚀';
    case 'cpp': return '🚀';
    default: return '📝';
  }
}

// Lifecycle
loadSupportedEngines();
</script>

<template>
  <div class="game-developer">
    <div class="header">
      <h1>💻 AI 游戏代码生成器</h1>
      <p class="subtitle">输入游戏设计文档（GDD），AI 自动生成完整的游戏代码</p>
    </div>

    <!-- Status Messages -->
    <div v-if="success" class="alert alert-success">
      {{ success }}
    </div>
    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <!-- Input Section -->
    <section class="section input-section">
      <h2>📋 游戏设计文档 (GDD)</h2>

      <div class="form-group">
        <label>GDD JSON *</label>
        <textarea
          v-model="gddJson"
          placeholder="粘贴游戏设计文档的 JSON..."
          rows="10"
          class="code-input"
        ></textarea>
      </div>

      <div class="form-group">
        <label>输出格式</label>
        <select v-model="outputFormat">
          <option value="SingleFile">单文件</option>
          <option value="MultipleFiles">多文件</option>
          <option value="ProjectStructure">项目结构</option>
        </select>
      </div>

      <div class="button-group">
        <button
          @click="generateCode"
          :disabled="isGenerating"
          class="btn btn-primary"
        >
          {{ isGenerating ? '生成中...' : '🚀 生成完整代码' }}
        </button>

        <button @click="loadExampleGDD" class="btn btn-secondary">
          📄 加载示例 GDD
        </button>

        <button @click="reset" class="btn btn-danger">
          🔄 重置
        </button>
      </div>

      <!-- Quick Generate Buttons -->
      <div class="quick-generate">
        <h3>⚡ 快速生成</h3>
        <div class="quick-buttons">
          <button @click="generateSpecificFile('script')" class="btn btn-sm">
            🎭 生成脚本
          </button>
          <button @click="generateSpecificFile('main')" class="btn btn-sm">
            🎮 生成主场景
          </button>
          <button @click="generateSpecificFile('player')" class="btn btn-sm">
            👤 生成玩家控制器
          </button>
        </div>
      </div>
    </section>

    <!-- Supported Engines -->
    <section class="section" v-if="supportedEngines.length > 0">
      <h2>🎯 支持的引擎</h2>
      <div class="engines-grid">
        <div v-for="engine in supportedEngines" :key="engine.name" class="engine-card">
          <div class="engine-header">
            <strong>{{ engine.display_name }}</strong>
            <span class="engine-language">{{ engine.language }}</span>
          </div>
          <p class="engine-description">{{ engine.description }}</p>
          <div class="engine-best-for">
            <strong>适用于:</strong>
            <div class="tag-list">
              <span v-for="tag in engine.best_for" :key="tag" class="tag">{{ tag }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Generated Code Display -->
    <section v-if="generatedCode" class="section code-section">
      <div class="code-header">
        <h2>📦 生成的代码</h2>
        <div class="code-stats">
          <span class="stat">引擎: {{ generatedCode.engine }}</span>
          <span class="stat">语言: {{ generatedCode.language }}</span>
          <span class="stat">总行数: {{ generatedCode.total_lines }}</span>
          <span class="stat">文件数: {{ generatedCode.files.length }}</span>
        </div>
        <button @click="copyAllCode" class="btn btn-secondary btn-sm">
          📋 复制所有代码
        </button>
      </div>

      <div class="code-layout">
        <!-- File List -->
        <div class="file-list">
          <h3>📁 文件列表</h3>
          <div
            v-for="file in generatedCode.files"
            :key="file.path"
            class="file-item"
            :class="{ active: selectedFile === file.path }"
            @click="selectedFile = file.path"
          >
            <span class="file-icon">{{ getLanguageIcon(file.language) }}</span>
            <div class="file-info">
              <div class="file-path">{{ file.path }}</div>
              <div class="file-language">{{ file.language }}</div>
            </div>
          </div>
        </div>

        <!-- Code Display -->
        <div class="code-display">
          <div class="code-display-header">
            <h3>{{ selectedFile || '选择一个文件' }}</h3>
            <button
              v-if="selectedFileContent"
              @click="copyFileContent"
              class="btn btn-secondary btn-sm"
            >
              📋 复制
            </button>
          </div>
          <pre v-if="selectedFileContent" class="code-content"><code>{{ selectedFileContent }}</code></pre>
          <div v-else class="no-file-selected">
            选择一个文件查看代码
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.game-developer {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.header {
  text-align: center;
  margin-bottom: 40px;
}

.header h1 {
  font-size: 32px;
  margin-bottom: 10px;
  color: #333;
}

.subtitle {
  font-size: 16px;
  color: #666;
}

.alert {
  padding: 12px 16px;
  border-radius: 6px;
  margin-bottom: 20px;
  font-size: 14px;
}

.alert-success {
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.alert-error {
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.section {
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 24px;
}

.section h2 {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 22px;
  color: #333;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #555;
}

.form-group textarea,
.form-group select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.form-group textarea {
  resize: vertical;
  min-height: 200px;
}

.form-group textarea:focus,
.form-group select:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
}

.code-input {
  background: #f8f9fa;
}

.button-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 24px;
}

.quick-generate {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid #e0e0e0;
}

.quick-generate h3 {
  margin-top: 0;
  margin-bottom: 12px;
  font-size: 16px;
  color: #555;
}

.quick-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.6;
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

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #c82333;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 13px;
}

.engines-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.engine-card {
  padding: 16px;
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.engine-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.engine-language {
  padding: 2px 8px;
  background: #e9ecef;
  border-radius: 4px;
  font-size: 12px;
  color: #666;
}

.engine-description {
  margin: 8px 0;
  font-size: 14px;
  color: #555;
  line-height: 1.5;
}

.engine-best-for {
  margin-top: 12px;
  font-size: 13px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.tag {
  padding: 3px 8px;
  background: #007bff;
  color: white;
  border-radius: 12px;
  font-size: 12px;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 12px;
}

.code-stats {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.stat {
  padding: 4px 10px;
  background: #e9ecef;
  border-radius: 4px;
  font-size: 13px;
  color: #555;
}

.code-layout {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 20px;
  min-height: 500px;
}

.file-list {
  border: 1px solid #dee2e6;
  border-radius: 6px;
  padding: 16px;
  background: #f8f9fa;
  overflow-y: auto;
  max-height: 600px;
}

.file-list h3 {
  margin-top: 0;
  margin-bottom: 12px;
  font-size: 16px;
  color: #333;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.file-item:hover {
  background: #e9ecef;
}

.file-item.active {
  background: #007bff;
  color: white;
}

.file-icon {
  font-size: 20px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-path {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-language {
  font-size: 11px;
  opacity: 0.7;
  margin-top: 2px;
}

.code-display {
  border: 1px solid #dee2e6;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.code-display-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #dee2e6;
}

.code-display-header h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.code-content {
  margin: 0;
  padding: 16px;
  background: #282c34;
  color: #abb2bf;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  overflow: auto;
  flex: 1;
  max-height: 600px;
}

.no-file-selected {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: #999;
  font-size: 16px;
  background: #f8f9fa;
}

@media (max-width: 1024px) {
  .code-layout {
    grid-template-columns: 1fr;
  }

  .file-list {
    max-height: 300px;
  }
}
</style>
