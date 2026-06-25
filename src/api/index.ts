// API Index - Export all API modules

export * from './agent-adapter';

// Re-export types for convenience
export type {
  AgentConfig,
  AgentTask,
  AgentResult,
  ResultStatus,
  TaskOutput,
  ActualCost,
  TokenUsage,
  SceneType,
  Platform,
  HealthMetrics,
  CostEstimate,
  ProficiencyScore,
  SelfOptimizingRouteResult,
  HistorySummary,
  PrivacyLevel,
  PrivacyZone,
  PrivacyDecision,
  FileInfoInput,
  ContextSummary,
  ProjectLanguage,
  ProjectFramework,
  Scene,
  ProjectPlatform,
} from './agent-adapter';

// Re-export APIs
export { AgentAdapterApi, SelfOptimizingRouterApi, PrivacyOrchestratorApi, ProjectContextApi } from './agent-adapter';
export { smartExecute, estimateTaskCost } from './agent-adapter';
export { DesktopApi } from './agent-adapter';
export { GameApi } from './agent-adapter';
export { MarketingApi } from './agent-adapter';
export { OfficeApi } from './agent-adapter';
export { BudgetApi } from './agent-adapter';
export { OneShotApi } from './agent-adapter';

// Re-export Desktop types
export type {
  DesktopFramework,
  DesktopDetectionResult,
  DesktopBundle,
  ElectronPlatform,
  ElectronArch,
} from './agent-adapter';

// Re-export Game types (Phase 3)
export type {
  GameFramework,
  UnityPlatform,
  GodotPlatform,
  GameDetectionResult,
  GameAssetType,
  GameAsset,
  AssetOptimization,
  OptimizationType,
  EstimatedSaving,
  AssetStats,
} from './agent-adapter';

// Re-export Marketing types (Phase 4)
export type {
  JimengStyle,
  JimengImageSize,
  KlingMode,
  KlingTaskStatus,
  KimiGenerateResult,
  JimengGenerateResult,
  KlingGenerateResult,
  KlingJobStatusResult,
} from './agent-adapter';

// Re-export Office types (Phase 4 Week 3)
export type {
  DocumentFormat,
  WpsDocumentType,
  WPSGenerateResult,
  ExcelAnalysisResult,
} from './agent-adapter';

// Re-export Budget types (Phase 4 Week 4)
export type {
  CostUsage,
  TokenTotals,
  CostForecast,
} from './agent-adapter';

// Re-export One-Shot types (Phase 5)
export type {
  UserRoleType,
  OneShotResponse,
  OneShotResult,
  TransparencyInfo,
  UserRoleResult,
  AgentRecommendation,
} from './agent-adapter';