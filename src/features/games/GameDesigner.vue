<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Types
interface GameConcept {
  title: string;
  genre: string;
  core_idea: string;
  target_audience: string;
  estimated_scope: string;
}

interface MechanicDesign {
  name: string;
  description: string;
  implementation_priority: string;
  code_hints: string;
}

interface SystemDesign {
  name: string;
  components: string[];
  interactions: string[];
  code_structure: string;
}

interface AssetRequirement {
  name: string;
  asset_type: string;
  description: string;
  style: string;
  quantity: number;
  specifications: string;
}

interface TechnicalArchitecture {
  engine: string;
  programming_language: string;
  project_type: string;
  scenes: string[];
  networking: string;
}

interface DevelopmentPhase {
  phase_name: string;
  tasks: string[];
  estimated_days: number;
}

interface GameDesignDoc {
  concept: GameConcept;
  core_mechanics: MechanicDesign[];
  systems: SystemDesign[];
  asset_requirements: AssetRequirement[];
  technical_architecture: TechnicalArchitecture;
  development_phases: DevelopmentPhase[];
}

// State
const userInput = ref('');
const preferredEngine = ref<string | null>(null);
const targetPlatform = ref<string | null>(null);
const maxDevelopmentTime = ref<number | null>(null);
const teamSize = ref<number | null>(null);
const showConstraints = ref(false);

const isGenerating = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);
const gdd = ref<GameDesignDoc | null>(null);
const showPreview = ref(false);

// Computed
const hasConstraints = computed(() => {
  return preferredEngine.value || targetPlatform.value || maxDevelopmentTime.value || teamSize.value;
});

// Methods
async function generateGDD() {
  if (!userInput.value.trim()) {
    showError('请输入游戏创意描述');
    return;
  }

  isGenerating.value = true;
  error.value = null;
  success.value = null;

  try {
    const constraints = hasConstraints.value ? {
      preferred_engine: preferredEngine.value,
      target_platform: targetPlatform.value,
      max_development_time: maxDevelopmentTime.value,
      team_size: teamSize.value,
    } : null;

    const result = await invoke<{
      success: boolean;
      gdd_json: string | null;
      error: string | null;
    }>('game_designer_generate_gdd', {
      request: {
        user_input: userInput.value,
        constraints,
      },
    });

    if (result.success && result.gdd_json) {
      gdd.value = JSON.parse(result.gdd_json);
      showSuccess('游戏设计文档生成成功！');
    } else {
      showError(result.error || '生成失败');
    }
  } catch (e) {
    showError(`错误: ${e}`);
  } finally {
    isGenerating.value = false;
  }
}

async function recommendEngine() {
  if (!userInput.value.trim()) {
    showError('请输入游戏描述');
    return;
  }

  // Extract genre from user input (simple heuristic)
  const genre = extractGenre(userInput.value);
  const scope = 'medium';  // Default

  try {
    const result = await invoke<{
      success: boolean;
      engine: string | null;
      reason: string | null;
      error: string | null;
    }>('game_designer_recommend_engine', {
      request: {
        genre,
        scope,
      },
    });

    if (result.success && result.engine) {
      preferredEngine.value = result.engine;
      showSuccess(`推荐引擎: ${result.engine}\n${result.reason}`);
    } else {
      showError(result.error || '推荐失败');
    }
  } catch (e) {
    showError(`错误: ${e}`);
  }
}

function extractGenre(input: string): string {
  const inputLower = input.toLowerCase();

  if (inputLower.includes('galgame') || inputLower.includes('视觉小说') || inputLower.includes('visual novel')) {
    return 'Visual Novel';
  } else if (inputLower.includes('fps') || inputLower.includes('射击') || inputLower.includes('first person')) {
    return 'FPS';
  } else if (inputLower.includes('moba')) {
    return 'MOBA';
  } else if (inputLower.includes('rpg')) {
    return 'RPG';
  } else {
    return 'General';
  }
}

function copyGDD() {
  if (!gdd.value) return;

  const json = JSON.stringify(gdd.value, null, 2);
  navigator.clipboard.writeText(json).then(() => {
    showSuccess('GDD JSON 已复制到剪贴板');
  }).catch(() => {
    showError('复制失败');
  });
}

function reset() {
  userInput.value = '';
  preferredEngine.value = null;
  targetPlatform.value = null;
  maxDevelopmentTime.value = null;
  teamSize.value = null;
  gdd.value = null;
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

function getPriorityColor(priority: string): string {
  switch (priority.toLowerCase()) {
    case 'critical': return '#dc3545';
    case 'high': return '#fd7e14';
    case 'medium': return '#ffc107';
    case 'low': return '#28a745';
    default: return '#6c757d';
  }
}

function getEngineIcon(engine: string): string {
  switch (engine) {
    case 'RenPy': return '🎭';
    case 'Godot': return '🎮';
    case 'Unity': return '🎯';
    case 'Unreal': return '🚀';
    default: return '🎲';
  }
}
</script>

<template>
  <div class="game-designer">
    <div class="header">
      <h1>🎨 AI 游戏设计师</h1>
      <p class="subtitle">输入你的游戏创意，AI 会为你生成完整的游戏设计文档</p>
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
      <h2>💡 游戏创意</h2>

      <div class="form-group">
        <label>描述你的游戏想法 *</label>
        <textarea
          v-model="userInput"
          placeholder="例如：我想做一个校园恋爱galgame，有3个可攻略的女主角，每个角色都有独立的故事线和多个结局..."
          rows="6"
        ></textarea>
      </div>

      <!-- Optional Constraints -->
      <div class="constraints-toggle">
        <button
          @click="showConstraints = !showConstraints"
          class="btn btn-secondary btn-sm"
        >
          {{ showConstraints ? '隐藏' : '显示' }} 高级选项
        </button>
      </div>

      <div v-if="showConstraints" class="constraints">
        <div class="form-group">
          <label>首选引擎</label>
          <select v-model="preferredEngine">
            <option :value="null">自动推荐</option>
            <option value="RenPy">Ren'Py (视觉小说)</option>
            <option value="Godot">Godot (2D/3D)</option>
            <option value="Unity">Unity (通用)</option>
            <option value="Unreal">Unreal (大型3D)</option>
          </select>
        </div>

        <div class="form-group">
          <label>目标平台</label>
          <input
            v-model="targetPlatform"
            type="text"
            placeholder="例如：Windows, Web, Mobile"
          />
        </div>

        <div class="form-row">
          <div class="form-group">
            <label>最大开发时间（天）</label>
            <input
              v-model.number="maxDevelopmentTime"
              type="number"
              placeholder="例如：30"
              min="1"
            />
          </div>

          <div class="form-group">
            <label>团队规模</label>
            <input
              v-model.number="teamSize"
              type="number"
              placeholder="例如：3"
              min="1"
            />
          </div>
        </div>
      </div>

      <div class="button-group">
        <button
          @click="generateGDD"
          :disabled="isGenerating"
          class="btn btn-primary"
        >
          {{ isGenerating ? '生成中...' : '🚀 生成游戏设计文档' }}
        </button>

        <button
          @click="recommendEngine"
          class="btn btn-secondary"
        >
          🎯 推荐引擎
        </button>

        <button
          @click="reset"
          class="btn btn-danger"
        >
          🔄 重置
        </button>
      </div>
    </section>

    <!-- GDD Display -->
    <section v-if="gdd" class="section gdd-section">
      <div class="gdd-header">
        <h2>📋 游戏设计文档</h2>
        <button @click="copyGDD" class="btn btn-secondary btn-sm">
          📋 复制 JSON
        </button>
      </div>

      <!-- Concept -->
      <div class="gdd-card">
        <h3>🎯 游戏概念</h3>
        <div class="concept-grid">
          <div class="concept-item">
            <strong>标题:</strong> {{ gdd.concept.title }}
          </div>
          <div class="concept-item">
            <strong>类型:</strong> {{ gdd.concept.genre }}
          </div>
          <div class="concept-item">
            <strong>目标受众:</strong> {{ gdd.concept.target_audience }}
          </div>
          <div class="concept-item">
            <strong>规模:</strong> {{ gdd.concept.estimated_scope }}
          </div>
        </div>
        <div class="concept-idea">
          <strong>核心创意:</strong>
          <p>{{ gdd.concept.core_idea }}</p>
        </div>
      </div>

      <!-- Technical Architecture -->
      <div class="gdd-card">
        <h3>{{ getEngineIcon(gdd.technical_architecture.engine) }} 技术架构</h3>
        <div class="tech-grid">
          <div class="tech-item">
            <strong>引擎:</strong> {{ gdd.technical_architecture.engine }}
          </div>
          <div class="tech-item">
            <strong>编程语言:</strong> {{ gdd.technical_architecture.programming_language }}
          </div>
          <div class="tech-item">
            <strong>项目类型:</strong> {{ gdd.technical_architecture.project_type }}
          </div>
          <div class="tech-item">
            <strong>网络类型:</strong> {{ gdd.technical_architecture.networking }}
          </div>
        </div>
        <div class="scenes-list">
          <strong>场景列表:</strong>
          <div class="scene-tags">
            <span v-for="scene in gdd.technical_architecture.scenes" :key="scene" class="scene-tag">
              {{ scene }}
            </span>
          </div>
        </div>
      </div>

      <!-- Core Mechanics -->
      <div class="gdd-card">
        <h3>⚙️ 核心机制</h3>
        <div v-for="(mechanic, index) in gdd.core_mechanics" :key="index" class="mechanic-item">
          <div class="mechanic-header">
            <strong>{{ mechanic.name }}</strong>
            <span
              class="priority-badge"
              :style="{ backgroundColor: getPriorityColor(mechanic.implementation_priority) }"
            >
              {{ mechanic.implementation_priority }}
            </span>
          </div>
          <p>{{ mechanic.description }}</p>
          <div class="code-hints">
            <strong>实现提示:</strong> {{ mechanic.code_hints }}
          </div>
        </div>
      </div>

      <!-- Systems -->
      <div class="gdd-card">
        <h3>🏗️ 系统设计</h3>
        <div v-for="(system, index) in gdd.systems" :key="index" class="system-item">
          <h4>{{ system.name }}</h4>
          <div class="system-details">
            <div>
              <strong>组件:</strong>
              <ul>
                <li v-for="comp in system.components" :key="comp">{{ comp }}</li>
              </ul>
            </div>
            <div>
              <strong>交互:</strong>
              <ul>
                <li v-for="interaction in system.interactions" :key="interaction">{{ interaction }}</li>
              </ul>
            </div>
          </div>
          <div class="code-structure">
            <strong>代码结构:</strong> {{ system.code_structure }}
          </div>
        </div>
      </div>

      <!-- Asset Requirements -->
      <div class="gdd-card">
        <h3>🎨 资源需求</h3>
        <div class="assets-grid">
          <div v-for="(asset, index) in gdd.asset_requirements" :key="index" class="asset-item">
            <div class="asset-header">
              <strong>{{ asset.name }}</strong>
              <span class="asset-type">{{ asset.asset_type }}</span>
            </div>
            <p>{{ asset.description }}</p>
            <div class="asset-details">
              <div><strong>风格:</strong> {{ asset.style }}</div>
              <div><strong>数量:</strong> {{ asset.quantity }}</div>
            </div>
            <div class="asset-specs">
              <strong>规格:</strong> {{ asset.specifications }}
            </div>
          </div>
        </div>
      </div>

      <!-- Development Phases -->
      <div class="gdd-card">
        <h3>📅 开发计划</h3>
        <div class="phases-timeline">
          <div v-for="(phase, index) in gdd.development_phases" :key="index" class="phase-item">
            <div class="phase-header">
              <strong>{{ phase.phase_name }}</strong>
              <span class="phase-duration">{{ phase.estimated_days }} 天</span>
            </div>
            <ul>
              <li v-for="task in phase.tasks" :key="task">{{ task }}</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Raw JSON Preview -->
      <div v-if="showPreview" class="gdd-card">
        <h3>📄 JSON 预览</h3>
        <pre class="json-preview">{{ JSON.stringify(gdd, null, 2) }}</pre>
      </div>

      <button @click="showPreview = !showPreview" class="btn btn-secondary">
        {{ showPreview ? '隐藏' : '显示' }} JSON
      </button>
    </section>
  </div>
</template>

<style scoped>
.game-designer {
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
  white-space: pre-line;
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
.form-group input,
.form-group select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
}

.form-group textarea {
  resize: vertical;
  min-height: 120px;
}

.form-group textarea:focus,
.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.constraints-toggle {
  margin-bottom: 16px;
}

.constraints {
  margin-bottom: 20px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

.button-group {
  display: flex;
  gap: 12px;
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

.gdd-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.gdd-card {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 6px;
  padding: 20px;
  margin-bottom: 20px;
}

.gdd-card h3 {
  margin-top: 0;
  margin-bottom: 16px;
  font-size: 18px;
  color: #333;
}

.concept-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.concept-item {
  font-size: 14px;
}

.concept-idea {
  margin-top: 12px;
  padding: 12px;
  background: white;
  border-radius: 4px;
  font-size: 14px;
  line-height: 1.6;
}

.tech-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.tech-item {
  font-size: 14px;
}

.scenes-list {
  margin-top: 12px;
}

.scene-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.scene-tag {
  padding: 4px 12px;
  background: #e9ecef;
  border-radius: 12px;
  font-size: 13px;
}

.mechanic-item {
  padding: 12px;
  background: white;
  border-radius: 4px;
  margin-bottom: 12px;
}

.mechanic-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.priority-badge {
  padding: 4px 10px;
  border-radius: 12px;
  color: white;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
}

.mechanic-item p {
  margin: 8px 0;
  font-size: 14px;
  color: #555;
}

.code-hints {
  margin-top: 8px;
  padding: 8px;
  background: #f1f3f5;
  border-radius: 4px;
  font-size: 13px;
  color: #666;
}

.system-item {
  padding: 16px;
  background: white;
  border-radius: 4px;
  margin-bottom: 12px;
}

.system-item h4 {
  margin-top: 0;
  margin-bottom: 12px;
  font-size: 16px;
  color: #333;
}

.system-details {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 12px;
}

.system-details ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
  font-size: 14px;
}

.system-details li {
  margin-bottom: 4px;
}

.code-structure {
  padding: 8px;
  background: #f1f3f5;
  border-radius: 4px;
  font-size: 13px;
  color: #666;
}

.assets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.asset-item {
  padding: 12px;
  background: white;
  border-radius: 4px;
}

.asset-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.asset-type {
  padding: 2px 8px;
  background: #e9ecef;
  border-radius: 4px;
  font-size: 12px;
  color: #666;
}

.asset-item p {
  margin: 8px 0;
  font-size: 14px;
  color: #555;
}

.asset-details {
  display: flex;
  gap: 16px;
  margin: 8px 0;
  font-size: 13px;
}

.asset-specs {
  padding: 8px;
  background: #f1f3f5;
  border-radius: 4px;
  font-size: 13px;
  color: #666;
}

.phases-timeline {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.phase-item {
  padding: 16px;
  background: white;
  border-radius: 4px;
  border-left: 4px solid #007bff;
}

.phase-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.phase-duration {
  padding: 4px 10px;
  background: #007bff;
  color: white;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
}

.phase-item ul {
  margin: 0;
  padding-left: 20px;
  font-size: 14px;
}

.phase-item li {
  margin-bottom: 4px;
}

.json-preview {
  background: #282c34;
  color: #abb2bf;
  padding: 16px;
  border-radius: 6px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  max-height: 400px;
  overflow-y: auto;
}
</style>
