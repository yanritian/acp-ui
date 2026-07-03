<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Types
interface RenPyProject {
  path: string;
  name: string;
  version: string;
  isProject: boolean;
}

interface Character {
  id: string;
  name: string;
  color: string;
  expressions: Expression[];
}

interface Expression {
  name: string;
  imageFilename: string;
}

interface Scene {
  label: string;
  background: string | null;
  dialogues: Dialogue[];
  menu: Menu | null;
  nextLabel: string | null;
}

interface Dialogue {
  speaker: string | null;
  text: string;
  characterShows: CharacterShow[];
}

interface CharacterShow {
  characterId: string;
  expression: string;
  position: string;
}

interface Menu {
  prompt: string | null;
  choices: Choice[];
}

interface Choice {
  text: string;
  targetLabel: string;
}

interface Background {
  id: string;
  imageFilename: string;
}

interface StorySpec {
  title: string;
  characters: Character[];
  scenes: Scene[];
  backgrounds: Background[];
}

// State
const renpyVersion = ref<string>('');
const projectPath = ref('');
const projectName = ref('');
const projectBasePath = ref('D:/games');
const isCreatingProject = ref(false);
const isGeneratingScript = ref(false);
const isRunningGame = ref(false);
const isCompiling = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

// Story Editor State
const storyTitle = ref('');
const characters = ref<Character[]>([]);
const scenes = ref<Scene[]>([]);
const backgrounds = ref<Background[]>([]);

// Computed
const hasProject = computed<boolean>(() => projectPath.value !== '' && projectPath.value.length > 0);
const hasCharacters = computed(() => characters.value.length > 0);
const hasScenes = computed(() => scenes.value.length > 0);
const storySpec = computed<StorySpec>(() => ({
  title: storyTitle.value,
  characters: characters.value,
  scenes: scenes.value,
  backgrounds: backgrounds.value,
}));

// Methods
async function getRenPyVersion() {
  try {
    const result = await invoke<{ success: boolean; version: string | null; error: string | null }>(
      'renpy_get_version',
      { request: {} }
    );

    if (result.success && result.version) {
      renpyVersion.value = result.version;
      showSuccess(`Ren'Py version: ${result.version}`);
    } else {
      showError(result.error || 'Failed to get Ren\'Py version');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  }
}

async function createProject() {
  if (!projectName.value) {
    showError('Please enter a project name');
    return;
  }

  isCreatingProject.value = true;
  error.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      projectPath: string | null;
      error: string | null;
    }>('renpy_create_project', {
      request: {
        name: projectName.value,
        path: projectBasePath.value,
      },
    });

    if (result.success && result.projectPath) {
      projectPath.value = result.projectPath;
      showSuccess(`Project created at: ${result.projectPath}`);
    } else {
      showError(result.error || 'Failed to create project');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  } finally {
    isCreatingProject.value = false;
  }
}

async function detectProject() {
  if (!projectPath.value) {
    showError('Please enter a project path');
    return;
  }

  try {
    const result = await invoke<{
      success: boolean;
      isProject: boolean;
      projectName: string | null;
      error: string | null;
    }>('renpy_detect_project', {
      request: {
        path: projectPath.value,
      },
    });

    if (result.success && result.isProject) {
      showSuccess(`Detected Ren'Py project: ${result.projectName || 'Unknown'}`);
    } else {
      showError(result.error || 'Not a Ren\'Py project');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  }
}

function addCharacter() {
  const id = `char_${characters.value.length + 1}`;
  characters.value.push({
    id,
    name: `Character ${characters.value.length + 1}`,
    color: '#c8ffc8',
    expressions: [
      { name: 'happy', imageFilename: `${id}_happy.png` },
      { name: 'sad', imageFilename: `${id}_sad.png` },
    ],
  });
}

function removeCharacter(index: number) {
  characters.value.splice(index, 1);
}

function addScene() {
  const label = scenes.value.length === 0 ? 'start' : `scene_${scenes.value.length}`;
  scenes.value.push({
    label,
    background: null,
    dialogues: [],
    menu: null,
    nextLabel: null,
  });
}

function removeScene(index: number) {
  scenes.value.splice(index, 1);
}

function addDialogue(sceneIndex: number) {
  scenes.value[sceneIndex].dialogues.push({
    speaker: null,
    text: 'Dialogue text...',
    characterShows: [],
  });
}

function removeDialogue(sceneIndex: number, dialogueIndex: number) {
  scenes.value[sceneIndex].dialogues.splice(dialogueIndex, 1);
}

function addMenu(sceneIndex: number) {
  scenes.value[sceneIndex].menu = {
    prompt: 'What do you want to do?',
    choices: [
      { text: 'Option 1', targetLabel: 'scene_1' },
      { text: 'Option 2', targetLabel: 'scene_2' },
    ],
  };
}

function removeMenu(sceneIndex: number) {
  scenes.value[sceneIndex].menu = null;
}

function addBackground() {
  const id = `bg_${backgrounds.value.length + 1}`;
  backgrounds.value.push({
    id,
    imageFilename: `${id}.png`,
  });
}

function removeBackground(index: number) {
  backgrounds.value.splice(index, 1);
}

async function generateScript() {
  if (!projectPath.value) {
    showError('Please create or select a project first');
    return;
  }

  if (!hasCharacters.value) {
    showError('Please add at least one character');
    return;
  }

  if (!hasScenes.value) {
    showError('Please add at least one scene');
    return;
  }

  // Check if start label exists
  const hasStartLabel = scenes.value.some(s => s.label === 'start');
  if (!hasStartLabel) {
    showError('Story must have a scene with label "start"');
    return;
  }

  isGeneratingScript.value = true;
  error.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      scriptPath: string | null;
      error: string | null;
    }>('renpy_generate_script', {
      request: {
        projectPath: projectPath.value,
        storySpec: storySpec.value,
      },
    });

    if (result.success && result.scriptPath) {
      showSuccess(`Script generated at: ${result.scriptPath}`);
    } else {
      showError(result.error || 'Failed to generate script');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  } finally {
    isGeneratingScript.value = false;
  }
}

async function runGame() {
  if (!projectPath.value) {
    showError('Please create or select a project first');
    return;
  }

  isRunningGame.value = true;
  error.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      processId: number | null;
      error: string | null;
    }>('renpy_run_game', {
      request: {
        projectPath: projectPath.value,
      },
    });

    if (result.success && result.processId) {
      showSuccess(`Game launched with PID: ${result.processId}`);
    } else {
      showError(result.error || 'Failed to launch game');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  } finally {
    isRunningGame.value = false;
  }
}

async function compileGame() {
  if (!projectPath.value) {
    showError('Please create or select a project first');
    return;
  }

  isCompiling.value = true;
  error.value = null;

  try {
    const result = await invoke<{
      success: boolean;
      output: string | null;
      error: string | null;
    }>('renpy_compile_game', {
      request: {
        projectPath: projectPath.value,
      },
    });

    if (result.success) {
      showSuccess('Compilation successful!');
      if (result.output) {
        console.log('Compilation output:', result.output);
      }
    } else {
      showError(result.error || 'Compilation failed');
    }
  } catch (e) {
    showError(`Error: ${e}`);
  } finally {
    isCompiling.value = false;
  }
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

// Lifecycle
onMounted(() => {
  getRenPyVersion();
});
</script>

<template>
  <div class="renpy-manager">
    <div class="header">
      <h1>🎭 Ren'Py Game Manager</h1>
      <div class="version" v-if="renpyVersion">
        Ren'Py {{ renpyVersion }}
      </div>
    </div>

    <!-- Status Messages -->
    <div v-if="success" class="alert alert-success">
      {{ success }}
    </div>
    <div v-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <!-- Project Section -->
    <section class="section">
      <h2>📁 Project</h2>

      <div class="form-group">
        <label>Project Name</label>
        <input
          v-model="projectName"
          type="text"
          placeholder="MyVisualNovel"
          :disabled="hasProject"
        />
      </div>

      <div class="form-group">
        <label>Base Path</label>
        <input
          v-model="projectBasePath"
          type="text"
          placeholder="D:/games"
          :disabled="hasProject"
        />
      </div>

      <div class="form-group" v-if="hasProject">
        <label>Project Path</label>
        <input
          v-model="projectPath"
          type="text"
          readonly
        />
      </div>

      <div class="form-group" v-else>
        <label>Or Select Existing Project</label>
        <input
          v-model="projectPath"
          type="text"
          placeholder="D:/games/ExistingProject"
        />
      </div>

      <div class="button-group">
        <button
          @click="createProject"
          :disabled="isCreatingProject || hasProject || undefined"
          class="btn btn-primary"
        >
          {{ isCreatingProject ? 'Creating...' : 'Create New Project' }}
        </button>

        <button
          @click="detectProject"
          :disabled="!projectPath || undefined"
          class="btn btn-secondary"
        >
          Detect Project
        </button>
      </div>
    </section>

    <!-- Story Editor -->
    <section class="section" v-if="hasProject">
      <h2>📖 Story Editor</h2>

      <div class="form-group">
        <label>Story Title</label>
        <input
          v-model="storyTitle"
          type="text"
          placeholder="My Amazing Story"
        />
      </div>

      <!-- Characters -->
      <div class="subsection">
        <h3>👥 Characters</h3>
        <div v-for="(char, index) in characters" :key="char.id" class="character-card">
          <div class="character-header">
            <input
              v-model="char.name"
              type="text"
              placeholder="Character name"
              class="character-name"
            />
            <input
              v-model="char.color"
              type="color"
              class="character-color"
            />
            <button @click="removeCharacter(index)" class="btn btn-danger btn-sm">
              Remove
            </button>
          </div>
          <div class="character-expressions">
            <div v-for="(expr, exprIndex) in char.expressions" :key="expr.name" class="expression-item">
              <input
                v-model="expr.name"
                type="text"
                placeholder="Expression name"
                class="expression-name"
              />
              <input
                v-model="expr.imageFilename"
                type="text"
                placeholder="Image filename"
                class="expression-filename"
              />
            </div>
          </div>
        </div>
        <button @click="addCharacter" class="btn btn-secondary">
          + Add Character
        </button>
      </div>

      <!-- Backgrounds -->
      <div class="subsection">
        <h3>🖼️ Backgrounds</h3>
        <div v-for="(bg, index) in backgrounds" :key="bg.id" class="background-item">
          <input
            v-model="bg.id"
            type="text"
            placeholder="Background ID (e.g., bg classroom)"
            class="background-id"
          />
          <input
            v-model="bg.imageFilename"
            type="text"
            placeholder="Image filename"
            class="background-filename"
          />
          <button @click="removeBackground(index)" class="btn btn-danger btn-sm">
            Remove
          </button>
        </div>
        <button @click="addBackground" class="btn btn-secondary">
          + Add Background
        </button>
      </div>

      <!-- Scenes -->
      <div class="subsection">
        <h3>🎬 Scenes</h3>
        <div v-for="(scene, sceneIndex) in scenes" :key="scene.label" class="scene-card">
          <div class="scene-header">
            <input
              v-model="scene.label"
              type="text"
              placeholder="Label (e.g., start, chapter1)"
              class="scene-label"
            />
            <select v-model="scene.background" class="scene-background">
              <option :value="null">No background</option>
              <option v-for="bg in backgrounds" :key="bg.id" :value="bg.id">
                {{ bg.id }}
              </option>
            </select>
            <button @click="removeScene(sceneIndex)" class="btn btn-danger btn-sm">
              Remove
            </button>
          </div>

          <!-- Dialogues -->
          <div class="dialogues">
            <div v-for="(dialogue, dialogueIndex) in scene.dialogues" :key="dialogueIndex" class="dialogue-item">
              <select v-model="dialogue.speaker" class="dialogue-speaker">
                <option :value="null">Narrator</option>
                <option v-for="char in characters" :key="char.id" :value="char.id">
                  {{ char.name }}
                </option>
              </select>
              <input
                v-model="dialogue.text"
                type="text"
                placeholder="Dialogue text"
                class="dialogue-text"
              />
              <button @click="removeDialogue(sceneIndex, dialogueIndex)" class="btn btn-danger btn-sm">
                Remove
              </button>
            </div>
            <button @click="addDialogue(sceneIndex)" class="btn btn-secondary btn-sm">
              + Add Dialogue
            </button>
          </div>

          <!-- Menu -->
          <div class="menu-section">
            <div v-if="scene.menu" class="menu-card">
              <input
                v-model="scene.menu.prompt"
                type="text"
                placeholder="Menu prompt (optional)"
                class="menu-prompt"
              />
              <div v-for="(choice, choiceIndex) in scene.menu.choices" :key="choiceIndex" class="choice-item">
                <input
                  v-model="choice.text"
                  type="text"
                  placeholder="Choice text"
                  class="choice-text"
                />
                <input
                  v-model="choice.targetLabel"
                  type="text"
                  placeholder="Target label"
                  class="choice-target"
                />
              </div>
              <button @click="removeMenu(sceneIndex)" class="btn btn-danger btn-sm">
                Remove Menu
              </button>
            </div>
            <button v-else @click="addMenu(sceneIndex)" class="btn btn-secondary btn-sm">
              + Add Menu
            </button>
          </div>

          <!-- Next Label -->
          <div class="next-label">
            <label>Jump to:</label>
            <select v-model="scene.nextLabel">
              <option :value="null">None</option>
              <option v-for="s in scenes.filter(s => s.label !== scene.label)" :key="s.label" :value="s.label">
                {{ s.label }}
              </option>
            </select>
          </div>
        </div>
        <button @click="addScene" class="btn btn-secondary">
          + Add Scene
        </button>
      </div>
    </section>

    <!-- Actions -->
    <section class="section" v-if="hasProject">
      <h2>🚀 Actions</h2>

      <div class="button-group">
        <button
          @click="generateScript"
          :disabled="isGeneratingScript || undefined"
          class="btn btn-primary"
        >
          {{ isGeneratingScript ? 'Generating...' : 'Generate Script' }}
        </button>

        <button
          @click="compileGame"
          :disabled="isCompiling || undefined"
          class="btn btn-secondary"
        >
          {{ isCompiling ? 'Compiling...' : 'Compile Game' }}
        </button>

        <button
          @click="runGame"
          :disabled="isRunningGame || undefined"
          class="btn btn-success"
        >
          {{ isRunningGame ? 'Running...' : 'Run Game' }}
        </button>
      </div>
    </section>

    <!-- Preview -->
    <section class="section" v-if="hasProject">
      <h2>👁️ Preview</h2>
      <pre class="preview">{{ JSON.stringify(storySpec, null, 2) }}</pre>
    </section>
  </div>
</template>

<style scoped>
.renpy-manager {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 2px solid #e0e0e0;
}

.header h1 {
  margin: 0;
  font-size: 28px;
  color: #333;
}

.version {
  font-size: 14px;
  color: #666;
  background: #f0f0f0;
  padding: 5px 10px;
  border-radius: 4px;
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
  padding: 20px;
  margin-bottom: 20px;
}

.section h2 {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 20px;
  color: #333;
}

.subsection {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #e0e0e0;
}

.subsection h3 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 16px;
  color: #555;
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

.form-group input[type="text"] {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input[type="text"]:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.1);
}

.form-group input[readonly] {
  background: #f8f9fa;
  cursor: not-allowed;
}

.button-group {
  display: flex;
  gap: 10px;
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

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

.character-card {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 12px;
}

.character-header {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}

.character-name {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.character-color {
  width: 50px;
  height: 36px;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
}

.character-expressions {
  margin-left: 20px;
}

.expression-item {
  display: flex;
  gap: 10px;
  margin-bottom: 8px;
}

.expression-name,
.expression-filename {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.background-item {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
  align-items: center;
}

.background-id,
.background-filename {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.scene-card {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 6px;
  padding: 16px;
  margin-bottom: 16px;
}

.scene-header {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 12px;
}

.scene-label {
  flex: 2;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.scene-background {
  flex: 2;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.dialogues {
  margin-left: 20px;
  margin-bottom: 12px;
}

.dialogue-item {
  display: flex;
  gap: 10px;
  margin-bottom: 8px;
  align-items: center;
}

.dialogue-speaker {
  width: 150px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.dialogue-text {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.menu-section {
  margin-left: 20px;
  margin-bottom: 12px;
}

.menu-card {
  background: #fff;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 12px;
}

.menu-prompt {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
  margin-bottom: 10px;
}

.choice-item {
  display: flex;
  gap: 10px;
  margin-bottom: 8px;
}

.choice-text,
.choice-target {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.next-label {
  margin-left: 20px;
  display: flex;
  gap: 10px;
  align-items: center;
}

.next-label label {
  font-size: 13px;
  color: #666;
}

.next-label select {
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 13px;
}

.preview {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 4px;
  padding: 16px;
  font-size: 12px;
  line-height: 1.5;
  overflow-x: auto;
  max-height: 400px;
  overflow-y: auto;
}
</style>
