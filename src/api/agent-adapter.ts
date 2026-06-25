// Agent Adapter API - TypeScript bindings for Rust backend
//
// Phase 1 Week 4: Frontend API integration

import { invoke } from '@tauri-apps/api/core';

// === Types ===

export interface AgentConfig {
  cwd?: string;
  timeout_ms?: number;
  max_retries?: number;
}

export interface AgentTask {
  id: string;
  description: string;
  cwd?: string;
  scene_type?: SceneType;
  platform?: Platform;
}

export interface AgentResult {
  task_id: string;
  agent_id: string;
  status: ResultStatus;
  output: TaskOutput;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  cost: ActualCost;
  duration_ms: number;
  timestamp: number;
}

export type ResultStatus = 'Success' | 'Failed' | 'Cancelled' | 'Timeout' | 'BudgetExceeded';

export interface TaskOutput {
  type: 'Text' | 'File' | 'Url' | 'Data';
  content: string;
}

export interface ActualCost {
  amount: number;
  currency: string;
  token_usage: TokenUsage;
}

export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export type SceneType =
  | 'WebDevelopment'
  | 'MiniProgramDevelopment'
  | 'DesktopDevelopment'
  | 'GameDevelopment'
  | 'GameArtGeneration'
  | 'GameCrossPlatform'
  | 'GamePerformanceOptimization';

export type Platform = 'Web' | 'MiniProgram' | 'Desktop' | 'Mobile' | 'Game';

export interface HealthMetrics {
  health_score: number;
  success_rate: number;
  avg_latency_ms: number;
  error_count: number;
  circuit_breaker_state: CircuitBreakerState;
}

export type CircuitBreakerState = 'Closed' | { Open: { opened_at: number } } | { HalfOpen: { test_requests: number } };

export interface CostEstimate {
  min_cost: number;
  max_cost: number;
  currency: string;
  breakdown: Record<string, number>;
  token_estimate: TokenUsage;
}

// === Self-Optimizing Router Types ===

export interface ProficiencyScore {
  agent_id: string;
  scene: SceneType;
  platform: Platform;
  proficiency: number;
  success_count: number;
  failure_count: number;
  avg_latency_ms: number;
  avg_cost: number;
}

export interface SelfOptimizingRouteResult {
  selected_agent: string;
  optimization_applied: boolean;
  proficiency_scores: ProficiencyScore[];
  reason: string;
}

export interface HistorySummary {
  total_executions: number;
  success_count: number;
  failure_count: number;
  success_rate: number;
  avg_latency_ms: number;
  total_cost: number;
}

// === Privacy Orchestrator Types ===

export type PrivacyLevel = 'StrictLocal' | 'Sanitized' | 'Standard' | 'External';

export interface PrivacyZone {
  id: string;
  name: string;
  privacy_level: PrivacyLevel;
  enabled: boolean;
}

export interface PrivacyDecision {
  task_id: string;
  privacy_level: PrivacyLevel;
  requires_local_execution: boolean;
  sensitive_data_detected: boolean;
  excluded_files: string[];
  sanitized_content?: string;
  reason: string;
}

// === Agent Adapter API ===

export const AgentAdapterApi = {
  /**
   * Initialize Claude Code adapter
   */
  async initClaude(cwd?: string): Promise<string> {
    return invoke('agent_adapter_init_claude', { cwd });
  },

  /**
   * Initialize Codex adapter
   */
  async initCodex(cwd?: string): Promise<string> {
    return invoke('agent_adapter_init_codex', { cwd });
  },

  /**
   * Execute task with Claude Code
   */
  async executeClaude(task: AgentTask): Promise<AgentResult> {
    return invoke('agent_adapter_execute_claude', {
      task_id: task.id,
      description: task.description,
      cwd: task.cwd,
      scene_type: task.scene_type,
      platform: task.platform,
    });
  },

  /**
   * Execute task with Codex
   */
  async executeCodex(task: AgentTask): Promise<AgentResult> {
    return invoke('agent_adapter_execute_codex', {
      task_id: task.id,
      description: task.description,
      cwd: task.cwd,
      scene_type: task.scene_type,
      platform: task.platform,
    });
  },

  /**
   * Get adapter health metrics
   */
  async getHealth(adapterId: string): Promise<HealthMetrics> {
    return invoke('agent_adapter_get_health', { adapter_id: adapterId });
  },

  /**
   * Estimate cost for task
   */
  async estimateCost(adapterId: string, description: string): Promise<CostEstimate> {
    return invoke('agent_adapter_estimate_cost', {
      adapter_id: adapterId,
      description,
    });
  },
};

// === Self-Optimizing Router API ===

export const SelfOptimizingRouterApi = {
  /**
   * Initialize router with custom config
   */
  async init(config?: {
    min_history_size?: number;
    ewma_alpha?: number;
    cost_weight?: number;
    performance_weight?: number;
    privacy_weight?: number;
  }): Promise<void> {
    return invoke('self_optimizing_init', config || {});
  },

  /**
   * Record execution for learning
   */
  async recordExecution(record: {
    task_id: string;
    scene_type: SceneType;
    platform: Platform;
    agent_id: string;
    success: boolean;
    duration_ms: number;
    cost: number;
  }): Promise<void> {
    return invoke('self_optimizing_record', record);
  },

  /**
   * Get optimal agent recommendation
   */
  async route(input: string, options?: {
    scene_type?: SceneType;
    platform?: Platform;
  }): Promise<SelfOptimizingRouteResult> {
    return invoke('self_optimizing_route', {
      input,
      input_type: 'Text',
      ...options,
    });
  },

  /**
   * Get proficiency scores
   */
  async getProficiencies(agentId?: string): Promise<ProficiencyScore[]> {
    return invoke('self_optimizing_get_proficiencies', { agent_id: agentId });
  },

  /**
   * Get history summary
   */
  async getHistory(): Promise<HistorySummary> {
    return invoke('self_optimizing_get_history');
  },

  /**
   * Reset proficiency scores
   */
  async reset(): Promise<void> {
    return invoke('self_optimizing_reset');
  },
};

// === Privacy Orchestrator API ===

export const PrivacyOrchestratorApi = {
  /**
   * Analyze task for privacy requirements
   */
  async analyzeTask(params: {
    task_id: string;
    content: string;
    file_paths: string[];
    command?: string;
  }): Promise<PrivacyDecision> {
    return invoke('privacy_analyze_task', params);
  },

  /**
   * Get active privacy zone
   */
  async getActiveZone(): Promise<PrivacyZone | null> {
    return invoke('privacy_get_active_zone');
  },

  /**
   * Set active privacy zone
   */
  async setZone(zoneId: string): Promise<void> {
    return invoke('privacy_set_zone', { zone_id: zoneId });
  },

  /**
   * Check if path is safe for external processing
   */
  async checkPath(path: string): Promise<boolean> {
    return invoke('privacy_check_path', { path });
  },

  /**
   * List all privacy zones
   */
  async listZones(): Promise<PrivacyZone[]> {
    return invoke('privacy_list_zones');
  },

  /**
   * Create custom privacy zone
   */
  async createZone(params: {
    id: string;
    name: string;
    level: PrivacyLevel;
  }): Promise<PrivacyZone> {
    return invoke('privacy_create_zone', {
      id: params.id,
      name: params.name,
      level: params.level,
    });
  },
};

// === Project Context API ===

export const ProjectContextApi = {
  /**
   * Create new project context
   */
  async create(params: {
    id: string;
    name: string;
    root_path: string;
  }): Promise<void> {
    return invoke('project_context_create', params);
  },

  /**
   * Update project context with file info
   */
  async update(id: string, files: FileInfoInput[]): Promise<ContextSummary> {
    return invoke('project_context_update', { id, files });
  },

  /**
   * Get context summary
   */
  async getSummary(id: string): Promise<ContextSummary> {
    return invoke('project_context_get_summary', { id });
  },
};

export interface FileInfoInput {
  path: string;
  line_count: number;
  is_source: boolean;
  is_test: boolean;
  is_config: boolean;
  is_doc: boolean;
}

export interface ContextSummary {
  language: ProjectLanguage;
  framework?: ProjectFramework;
  scene: Scene;
  platform: ProjectPlatform;
  file_count: number;
  recent_change_count: number;
  pattern_count: number;
}

export type ProjectLanguage =
  | 'TypeScript'
  | 'JavaScript'
  | 'Vue'
  | 'Rust'
  | 'Go'
  | 'Python'
  | 'Java'
  | 'Kotlin'
  | 'Swift'
  | 'CSharp'
  | 'Cpp'
  | 'C'
  | 'Unknown';

export type ProjectFramework =
  | 'Vue'
  | 'React'
  | 'Angular'
  | 'Tauri'
  | 'Electron'
  | 'Flutter'
  | 'RustNative'
  | 'GoNative'
  | 'PythonNative'
  | 'Unity'
  | 'Godot';

export type Scene = 'Web' | 'MiniProgram' | 'Desktop' | 'Game' | 'Unknown';

export type ProjectPlatform =
  | 'Web'
  | 'MiniProgramWechat'
  | 'MiniProgramAlipay'
  | 'DesktopWindows'
  | 'DesktopMacos'
  | 'DesktopLinux'
  | 'DesktopCross'
  | 'GameUnity'
  | 'GameGodot'
  | 'Unknown';

// === Desktop Development Types (Phase 2) ===

export type DesktopFramework = 'tauri' | 'electron' | 'hybrid' | 'unknown';

export interface DesktopDetectionResult {
  framework: DesktopFramework;
  platform: string;
  recommended_bundles: string[];
  cwd: string;
}

export type DesktopBundle = 'msi' | 'dmg' | 'appimage' | 'deb' | 'rpm' | 'nsis' | 'none';

export type ElectronPlatform = 'win' | 'mac' | 'linux' | 'all';
export type ElectronArch = 'x64' | 'arm64' | 'universal';

// === Desktop Development API ===

export const DesktopApi = {
  /**
   * Detect desktop framework (Tauri or Electron)
   */
  async detectProject(cwd: string): Promise<DesktopDetectionResult> {
    return invoke('desktop_detect_project', { cwd });
  },

  /**
   * Build Tauri app in dev mode
   */
  async tauriDev(cwd: string): Promise<string> {
    return invoke('desktop_tauri_dev', { cwd });
  },

  /**
   * Build Tauri app in release mode
   */
  async tauriBuild(cwd: string, bundle?: DesktopBundle): Promise<string> {
    return invoke('desktop_tauri_build', { cwd, bundle });
  },

  /**
   * Start Electron in dev mode
   */
  async electronDev(cwd: string): Promise<string> {
    return invoke('desktop_electron_dev', { cwd });
  },

  /**
   * Build Electron app
   */
  async electronBuild(cwd: string, platform?: ElectronPlatform, arch?: ElectronArch): Promise<string> {
    return invoke('desktop_electron_build', { cwd, platform, arch });
  },

  /**
   * Get current desktop platform
   */
  async getPlatform(): Promise<string> {
    return invoke('desktop_get_platform');
  },

  /**
   * Get available build targets for platform
   */
  async getTargets(platform: string): Promise<string[]> {
    return invoke('desktop_get_targets', { platform });
  },
};

// === Convenience functions ===

/**
 * Smart execute: automatically route task to best agent
 */
export async function smartExecute(task: AgentTask): Promise<AgentResult> {
  // 1. Check privacy
  const privacyDecision = await PrivacyOrchestratorApi.analyzeTask({
    task_id: task.id,
    content: task.description,
    file_paths: task.cwd ? [task.cwd] : [],
  });

  // 2. If requires local execution, use Claude Code
  if (privacyDecision.requires_local_execution) {
    return AgentAdapterApi.executeClaude(task);
  }

  // 3. Route to best agent based on history
  const routeResult = await SelfOptimizingRouterApi.route(task.description, {
    scene_type: task.scene_type,
    platform: task.platform,
  });

  // 4. Execute with selected agent
  if (routeResult.selected_agent.includes('codex')) {
    return AgentAdapterApi.executeCodex(task);
  } else {
    return AgentAdapterApi.executeClaude(task);
  }
}

/**
 * Get cost estimate before execution
 */
export async function estimateTaskCost(task: AgentTask): Promise<{
  claude: CostEstimate;
  codex: CostEstimate;
}> {
  const claudeEstimate = await AgentAdapterApi.estimateCost('claude-code', task.description);
  const codexEstimate = await AgentAdapterApi.estimateCost('codex', task.description);

  return { claude: claudeEstimate, codex: codexEstimate };
}

// === Game Development Types (Phase 3) ===

export type GameFramework = 'unity' | 'godot' | 'hybrid' | 'unknown';

export type UnityPlatform = 'Windows' | 'MacOS' | 'Linux' | 'Android' | 'iOS' | 'WebGL';

export type GodotPlatform = 'Windows' | 'MacOS' | 'Linux' | 'Android' | 'iOS' | 'Web';

export interface GameDetectionResult {
  framework: GameFramework;
  scenes: string[];
  cwd: string;
}

// === Game Development API ===

export const GameApi = {
  /**
   * Detect game framework (Unity or Godot)
   */
  async detectFramework(cwd: string): Promise<GameDetectionResult> {
    return invoke('game_detect_framework', { cwd });
  },

  /**
   * Build Unity project (development mode)
   */
  async unityBuildDev(cwd: string, platform?: UnityPlatform): Promise<string> {
    return invoke('unity_build_dev', { cwd, platform });
  },

  /**
   * Build Unity project (release mode)
   */
  async unityBuildRelease(cwd: string, platform?: UnityPlatform): Promise<string> {
    return invoke('unity_build_release', { cwd, platform });
  },

  /**
   * Get Unity scenes in project
   */
  async unityGetScenes(cwd: string): Promise<string[]> {
    return invoke('unity_get_scenes', { cwd });
  },

  /**
   * Export Godot project (development mode)
   */
  async godotBuildDev(cwd: string, platform?: GodotPlatform): Promise<string> {
    return invoke('godot_build_dev', { cwd, platform });
  },

  /**
   * Export Godot project (release mode)
   */
  async godotBuildRelease(cwd: string, platform?: GodotPlatform): Promise<string> {
    return invoke('godot_build_release', { cwd, platform });
  },

  /**
   * Get Godot scenes in project
   */
  async godotGetScenes(cwd: string): Promise<string[]> {
    return invoke('godot_get_scenes', { cwd });
  },

  /**
   * Get available game platforms
   */
  async getPlatforms(): Promise<string[]> {
    return invoke('game_get_platforms');
  },

  // === Game Asset Commands (Phase 3 Week 4) ===

  /**
   * Detect game assets in project
   */
  async detectAssets(cwd: string, framework: string): Promise<GameAsset[]> {
    return invoke('game_detect_assets', { cwd, framework });
  },

  /**
   * Get game asset statistics
   */
  async getAssetStats(cwd: string, framework: string): Promise<AssetStats> {
    return invoke('game_get_asset_stats', { cwd, framework });
  },

  /**
   * Get optimization suggestions for asset
   */
  async getOptimizationSuggestions(
    assetType: string,
    sizeBytes: number,
    format: string,
  ): Promise<AssetOptimization[]> {
    return invoke('game_get_optimization_suggestions', {
      asset_type: assetType,
      size_bytes: sizeBytes,
      format,
    });
  },

  /**
   * Get all supported asset types
   */
  async getAssetTypes(): Promise<string[]> {
    return invoke('game_get_asset_types');
  },
};

// === Game Asset Types (Phase 3 Week 4) ===

export type GameAssetType = 'sprite2d' | 'texture' | 'model3d' | 'animation' | 'audio' | 'shader' | 'level' | 'font' | 'video' | 'script' | 'unknown';

export interface GameAsset {
  name: string;
  asset_type: GameAssetType;
  path: string;
  size_bytes: number;
  format: string;
  optimization_suggestions: AssetOptimization[];
}

export interface AssetOptimization {
  suggestion_type: OptimizationType;
  priority: number;
  description: string;
  estimated_savings?: EstimatedSaving;
}

export type OptimizationType = 'Compress' | 'Resize' | 'ConvertFormat' | 'OptimizeMesh' | 'OptimizeAudio' | 'Split' | 'RemoveUnused';

export type EstimatedSaving = { Bytes: number } | { Percentage: number };

export interface AssetStats {
  total_assets: number;
  total_size_bytes: number;
  type_counts: Record<string, number>;
  type_sizes: Record<string, number>;
  optimization_count: number;
}

// === Marketing API (Phase 4) ===

export const MarketingApi = {
  /**
   * Initialize Kimi adapter
   */
  async initKimi(apiKey: string, model?: string): Promise<string> {
    return invoke('marketing_init_kimi', { api_key: apiKey, model });
  },

  /**
   * Generate text with Kimi
   */
  async kimiGenerateText(
    apiKey: string,
    prompt: string,
    systemPrompt?: string,
    model?: string,
  ): Promise<KimiGenerateResult> {
    return invoke('kimi_generate_text', {
      api_key: apiKey,
      prompt,
      system_prompt: systemPrompt,
      model,
    });
  },

  /**
   * Translate text with Kimi
   */
  async kimiTranslate(apiKey: string, text: string, targetLanguage: string): Promise<string> {
    return invoke('kimi_translate', {
      api_key: apiKey,
      text,
      target_language: targetLanguage,
    });
  },

  /**
   * Initialize Jimeng adapter
   */
  async initJimeng(apiKey: string, defaultStyle?: string): Promise<string> {
    return invoke('marketing_init_jimeng', { api_key: apiKey, default_style: defaultStyle });
  },

  /**
   * Generate image with Jimeng
   */
  async jimengGenerateImage(
    apiKey: string,
    prompt: string,
    style?: JimengStyle,
    size?: JimengImageSize,
  ): Promise<JimengGenerateResult> {
    return invoke('jimeng_generate_image', {
      api_key: apiKey,
      prompt,
      style,
      size,
    });
  },

  /**
   * Initialize Kling adapter
   */
  async initKling(apiKey: string, defaultDuration?: number): Promise<string> {
    return invoke('marketing_init_kling', {
      api_key: apiKey,
      default_duration: defaultDuration,
    });
  },

  /**
   * Generate video with Kling (async)
   */
  async klingGenerateVideo(
    apiKey: string,
    prompt: string,
    mode?: KlingMode,
  ): Promise<KlingGenerateResult> {
    return invoke('kling_generate_video', {
      api_key: apiKey,
      prompt,
      mode,
    });
  },

  /**
   * Get Kling video job status
   */
  async klingGetJobStatus(apiKey: string, taskId: string): Promise<KlingJobStatusResult> {
    return invoke('kling_get_job_status', {
      api_key: apiKey,
      task_id: taskId,
    });
  },
};

// === Marketing Types (Phase 4) ===

export type JimengStyle = 'realistic' | 'anime' | 'abstract' | 'cartoon' | 'oil_painting' | 'watercolor' | 'sketch' | 'cyberpunk' | 'fantasy' | 'minimalist';

export type JimengImageSize = '512' | '1024' | 'portrait' | 'landscape' | 'wide';

export type KlingMode = 'text_to_video' | 'image_to_video' | 'avatar' | 'extend';

export type KlingTaskStatus = 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';

export interface KimiGenerateResult {
  content: string;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export interface JimengGenerateResult {
  image_urls: string[];
  cost: number;
  style: string;
}

export interface KlingGenerateResult {
  task_id: string;
  estimated_duration: number;
  cost: number;
  mode: string;
}

export interface KlingJobStatusResult {
  task_id: string;
  status: KlingTaskStatus;
  video_url?: string;
  error?: string;
}

// === Office API (Phase 4 Week 3) ===

export const OfficeApi = {
  /**
   * Initialize WPS adapter
   */
  async initWps(apiKey: string, defaultFormat?: DocumentFormat): Promise<string> {
    return invoke('office_init_wps', { api_key: apiKey, default_format: defaultFormat });
  },

  /**
   * Generate document with WPS
   */
  async generateDoc(
    apiKey: string,
    prompt: string,
    documentType?: WpsDocumentType,
    format?: DocumentFormat,
  ): Promise<WPSGenerateResult> {
    return invoke('wps_generate_doc', {
      api_key: apiKey,
      prompt,
      document_type: documentType,
      format,
    });
  },

  /**
   * Analyze Excel file
   */
  async analyzeExcel(apiKey: string, fileUrl: string): Promise<ExcelAnalysisResult> {
    return invoke('wps_analyze_excel', {
      api_key: apiKey,
      file_url: fileUrl,
    });
  },
};

// === Budget API (Phase 4 Week 4) ===

export const BudgetApi = {
  /**
   * Set budget limits
   */
  async setLimit(
    dailyLimit: number,
    monthlyLimit: number,
    hardStop?: boolean,
  ): Promise<void> {
    return invoke('budget_set_limit', {
      daily_limit: dailyLimit,
      monthly_limit: monthlyLimit,
      hard_stop: hardStop,
    });
  },

  /**
   * Get current usage
   */
  async getUsage(): Promise<CostUsage> {
    return invoke('budget_get_usage');
  },

  /**
   * Get cost forecast
   */
  async getForecast(): Promise<CostForecast> {
    return invoke('budget_get_forecast');
  },

  /**
   * Check if budget allows execution
   */
  async isAllowed(): Promise<boolean> {
    return invoke('budget_is_allowed');
  },
};

// === Office Types (Phase 4 Week 3) ===

export type DocumentFormat = 'docx' | 'pdf' | 'pptx' | 'xlsx' | 'txt';

export type WpsDocumentType = 'report' | 'contract' | 'proposal' | 'resume' | 'summary' | 'plan' | 'article';

export interface WPSGenerateResult {
  document_url: string;
  cost: number;
  document_type: string;
  format: string;
}

export interface ExcelAnalysisResult {
  summary: string;
  charts: string[];
  insights: string[];
  cost: number;
}

// === Budget Types (Phase 4 Week 4) ===

export interface CostUsage {
  daily_total: number;
  monthly_total: number;
  total_all_time: number;
  adapter_totals: Record<string, number>;
  scene_totals: Record<string, number>;
  token_totals: TokenTotals;
}

export interface TokenTotals {
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export interface CostForecast {
  predicted_daily: number;
  predicted_monthly: number;
  days_remaining: number;
  budget_remaining_percent: number;
  will_exceed: boolean;
}

// === One-Shot API (Phase 5) ===

export const OneShotApi = {
  /**
   * One-shot unified entry point
   */
  async execute(
    input: string,
    options?: {
      context_hint?: string;
      preferred_agent?: string;
      max_cost?: number;
      timeout_ms?: number;
    },
  ): Promise<OneShotResponse> {
    return invoke('one_shot_execute', {
      input,
      ...options,
    });
  },

  /**
   * Detect user role from input
   */
  async detectRole(input: string): Promise<UserRoleResult> {
    return invoke('one_shot_detect_role', { input });
  },

  /**
   * Detect scene from input
   */
  async detectScene(input: string): Promise<string> {
    return invoke('one_shot_detect_scene', { input });
  },

  /**
   * Get recommended agent
   */
  async getRecommendation(role: string, scene: string): Promise<AgentRecommendation> {
    return invoke('one_shot_get_recommendation', { role, scene });
  },
};

// === One-Shot Types (Phase 5) ===

export type UserRoleType = 'developer' | 'marketer' | 'designer' | 'finance' | 'gamer' | 'writer' | 'analyst' | 'general';

export interface OneShotResponse {
  detected_role: UserRoleType;
  detected_scene: string;
  selected_agent: string;
  selection_reason: string;
  result?: OneShotResult;
  cost: number;
  duration_ms: number;
  transparency: TransparencyInfo;
}

export interface OneShotResult {
  output_type: 'Text' | 'File' | 'Url' | 'Image' | 'Video' | 'Document' | 'Data';
  content: string;
  metadata: Record<string, string>;
}

export interface TransparencyInfo {
  dag_visualization: string;
  estimated_cost: number;
  estimated_duration_ms: number;
  proficiency_scores: Record<string, number>;
  privacy_level: string;
  budget_status: string;
}

export interface UserRoleResult {
  role: UserRoleType;
  confidence: number;
}

export interface AgentRecommendation {
  agent: string;
  reason: string;
  alternatives: string[];
}