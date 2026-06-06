// Development Flow Skill Chain - 完整开发流程自动化
// 需求分析 → 计划撰写 → 写代码 → 代码调整 → 功能测试 → 反馈用户

import type { SkillExecutionResult, SkillContext } from './types'
import { invokeSkill } from './skill-invoker'

// ---------------------------------------------------------------------------
// Development Flow Skills Definition
// ---------------------------------------------------------------------------

export const DEVELOPMENT_FLOW_SKILLS: readonly string[] = [
  'planning',        // 需求分析 + 计划撰写
  'code-execution',  // 写代码
  'code-review',     // 代码审查
  'file-operations', // 代码调整
  'testing',         // 功能测试
  'documentation',   // 文档生成
  'communication',   // 反馈用户
]

// ---------------------------------------------------------------------------
// Flow Stage Types
// ---------------------------------------------------------------------------

export interface FlowStage {
  name: string
  skill: string
  inputs: string[]
  outputs: string[]
  condition?: (context: FlowContext) => boolean
  onFailure: 'stop' | 'retry' | 'skip' | 'fallback'
  maxRetries: number
}

export interface FlowContext {
  taskId: string
  request: string
  workspace: string
  cwd: string
  agentName: string
  sessionId?: string

  // Accumulated results from previous stages
  analysis?: AnalysisResult
  plan?: PlanResult
  code?: CodeResult
  review?: ReviewResult
  adjustments?: AdjustmentsResult
  testResults?: TestResult
  docs?: DocumentationResult

  // Error tracking
  errors: FlowError[]
  currentStage: string
  stageHistory: StageExecution[]
}

export interface FlowError {
  stage: string
  error: string
  timestamp: number
  retryCount: number
}

export interface StageExecution {
  stage: string
  startedAt: number
  completedAt?: number
  success: boolean
  output?: string
}

// ---------------------------------------------------------------------------
// Stage Result Types
// ---------------------------------------------------------------------------

export interface AnalysisResult {
  requirements: string[]
  constraints: string[]
  dependencies: string[]
  complexity: 'low' | 'medium' | 'high'
  suggestedApproach: string
}

export interface PlanResult {
  steps: PlanStep[]
  estimatedTime: number
  filesToCreate: string[]
  filesToModify: string[]
  risks: string[]
}

export interface PlanStep {
  order: number
  action: string
  description: string
  skill?: string
  estimatedMs: number
}

export interface CodeResult {
  filesCreated: string[]
  filesModified: string[]
  linesAdded: number
  linesRemoved: number
  summary: string
}

export interface ReviewResult {
  issues: ReviewIssue[]
  score: number
  passed: boolean
  suggestions: string[]
}

export interface ReviewIssue {
  file: string
  line?: number
  severity: 'critical' | 'high' | 'medium' | 'low'
  message: string
  fixSuggestion?: string
}

export interface AdjustmentsResult {
  filesModified: string[]
  changesApplied: number
  issuesFixed: number
  remainingIssues: ReviewIssue[]
}

export interface TestResult {
  totalTests: number
  passed: number
  failed: number
  coverage?: number
  failures: TestFailure[]
}

export interface TestFailure {
  testName: string
  error: string
  file: string
}

export interface DocumentationResult {
  filesGenerated: string[]
  readmeUpdated: boolean
  apiDocsGenerated: boolean
}

// ---------------------------------------------------------------------------
// Development Flow Definition
// ---------------------------------------------------------------------------

export const DEVELOPMENT_FLOW_STAGES: FlowStage[] = [
  {
    name: 'analysis',
    skill: 'planning',
    inputs: ['request', 'workspace'],
    outputs: ['analysis'],
    onFailure: 'stop',
    maxRetries: 2,
  },
  {
    name: 'planning',
    skill: 'planning',
    inputs: ['analysis'],
    outputs: ['plan'],
    onFailure: 'retry',
    maxRetries: 3,
  },
  {
    name: 'code-execution',
    skill: 'code-execution',
    inputs: ['plan', 'workspace'],
    outputs: ['code'],
    onFailure: 'retry',
    maxRetries: 3,
  },
  {
    name: 'code-review',
    skill: 'code-review',
    inputs: ['code'],
    outputs: ['review'],
    condition: (ctx) => (ctx.code?.filesCreated?.length ?? 0) > 0 || (ctx.code?.filesModified?.length ?? 0) > 0,
    onFailure: 'skip',
    maxRetries: 1,
  },
  {
    name: 'adjustments',
    skill: 'file-operations',
    inputs: ['review', 'code'],
    outputs: ['adjustments'],
    condition: (ctx) => ctx.review?.passed === false && (ctx.review?.issues?.length ?? 0) > 0,
    onFailure: 'retry',
    maxRetries: 2,
  },
  {
    name: 'testing',
    skill: 'testing',
    inputs: ['code', 'plan'],
    outputs: ['testResults'],
    condition: (ctx) => ctx.plan?.steps?.some(s => s.action === 'write-tests') ?? false,
    onFailure: 'retry',
    maxRetries: 3,
  },
  {
    name: 'documentation',
    skill: 'documentation',
    inputs: ['code', 'plan'],
    outputs: ['docs'],
    condition: (ctx) => ctx.plan?.steps?.some(s => s.action === 'generate-docs') ?? false,
    onFailure: 'skip',
    maxRetries: 1,
  },
  {
    name: 'communication',
    skill: 'communication',
    inputs: ['testResults', 'docs', 'code'],
    outputs: [],
    onFailure: 'skip',
    maxRetries: 1,
  },
]

// ---------------------------------------------------------------------------
// Flow Orchestrator
// ---------------------------------------------------------------------------

export class DevelopmentFlowOrchestrator {
  private context: FlowContext

  constructor(initialContext: Partial<FlowContext>) {
    this.context = {
      taskId: initialContext.taskId || generateTaskId(),
      request: initialContext.request || '',
      workspace: initialContext.workspace || process.cwd(),
      cwd: initialContext.cwd || process.cwd(),
      agentName: initialContext.agentName || 'claude-code',
      sessionId: initialContext.sessionId,
      errors: [],
      currentStage: '',
      stageHistory: [],
    }
  }

  /**
   * Execute the complete development flow
   */
  async execute(): Promise<FlowContext> {
    console.log(`[DevFlow] Starting flow for task: ${this.context.taskId}`)

    for (const stage of DEVELOPMENT_FLOW_STAGES) {
      // Check condition
      if (stage.condition && !stage.condition(this.context)) {
        console.log(`[DevFlow] Skipping stage ${stage.name} (condition not met)`)
        continue
      }

      this.context.currentStage = stage.name

      const execution: StageExecution = {
        stage: stage.name,
        startedAt: Date.now(),
        success: false,
      }

      try {
        const result = await this.executeStage(stage)
        execution.success = true
        execution.completedAt = Date.now()
        execution.output = result.output

        // Store result in context
        this.storeStageResult(stage.name, result)

        console.log(`[DevFlow] Stage ${stage.name} completed successfully`)
      } catch (error) {
        execution.completedAt = Date.now()
        execution.output = error instanceof Error ? error.message : String(error)

        const handled = await this.handleFailure(stage, error)

        if (!handled && stage.onFailure === 'stop') {
          this.context.errors.push({
            stage: stage.name,
            error: execution.output,
            timestamp: Date.now(),
            retryCount: 0,
          })
          this.context.stageHistory.push(execution)
          console.log(`[DevFlow] Flow stopped at stage ${stage.name}`)
          return this.context
        }
      }

      this.context.stageHistory.push(execution)
    }

    console.log(`[DevFlow] Flow completed successfully`)
    return this.context
  }

  /**
   * Execute a single stage
   */
  private async executeStage(stage: FlowStage): Promise<SkillExecutionResult> {
    const inputs = this.gatherInputs(stage)

    const context: SkillContext = {
      sessionId: this.context.sessionId,
      agentName: this.context.agentName,
      taskId: this.context.taskId,
      cwd: this.context.cwd,
    }

    return invokeSkill(stage.skill, inputs, context)
  }

  /**
   * Gather inputs for a stage from context
   */
  private gatherInputs(stage: FlowStage): Record<string, unknown> {
    const inputs: Record<string, unknown> = {
      request: this.context.request,
      workspace: this.context.workspace,
    }

    for (const inputKey of stage.inputs) {
      if (inputKey in this.context) {
        inputs[inputKey] = this.context[inputKey as keyof FlowContext]
      }
    }

    return inputs
  }

  /**
   * Store stage result in context
   */
  private storeStageResult(stageName: string, result: SkillExecutionResult): void {
    // Parse output and store in appropriate context field
    switch (stageName) {
      case 'analysis':
        this.context.analysis = this.parseAnalysisResult(result.output)
        break
      case 'planning':
        this.context.plan = this.parsePlanResult(result.output)
        break
      case 'code-execution':
        this.context.code = this.parseCodeResult(result.output)
        break
      case 'code-review':
        this.context.review = this.parseReviewResult(result.output)
        break
      case 'adjustments':
        this.context.adjustments = this.parseAdjustmentsResult(result.output)
        break
      case 'testing':
        this.context.testResults = this.parseTestResult(result.output)
        break
      case 'documentation':
        this.context.docs = this.parseDocumentationResult(result.output)
        break
    }
  }

  /**
   * Handle stage failure
   */
  private async handleFailure(stage: FlowStage, error: unknown): Promise<boolean> {
    const errorMsg = error instanceof Error ? error.message : String(error)

    if (stage.onFailure === 'skip') {
      console.log(`[DevFlow] Skipping failed stage ${stage.name}`)
      return true
    }

    if (stage.onFailure === 'retry' && stage.maxRetries > 0) {
      for (let attempt = 1; attempt <= stage.maxRetries; attempt++) {
        console.log(`[DevFlow] Retrying stage ${stage.name} (attempt ${attempt}/${stage.maxRetries})`)

        try {
          await new Promise(r => setTimeout(r, 1000 * attempt)) // Exponential backoff
          const result = await this.executeStage(stage)
          this.storeStageResult(stage.name, result)
          return true
        } catch (retryError) {
          this.context.errors.push({
            stage: stage.name,
            error: retryError instanceof Error ? retryError.message : String(retryError),
            timestamp: Date.now(),
            retryCount: attempt,
          })
        }
      }
    }

    if (stage.onFailure === 'fallback') {
      // Try fallback approach (simpler/safer method)
      console.log(`[DevFlow] Attempting fallback for stage ${stage.name}`)
      // TODO: Implement fallback logic
    }

    return false
  }

  // ---------------------------------------------------------------------------
  // Result Parsing Methods
  // ---------------------------------------------------------------------------

  private parseAnalysisResult(output: string): AnalysisResult {
    // Simple parsing - in production would use structured output from Agent
    const lines = output.split('\n')
    return {
      requirements: lines.filter(l => l.startsWith('REQ:')).map(l => l.replace('REQ:', '').trim()),
      constraints: lines.filter(l => l.startsWith('CONST:')).map(l => l.replace('CONST:', '').trim()),
      dependencies: lines.filter(l => l.startsWith('DEP:')).map(l => l.replace('DEP:', '').trim()),
      complexity: output.includes('high complexity') ? 'high' : output.includes('medium complexity') ? 'medium' : 'low',
      suggestedApproach: lines.find(l => l.startsWith('APPROACH:'))?.replace('APPROACH:', '').trim() || '',
    }
  }

  private parsePlanResult(output: string): PlanResult {
    const lines = output.split('\n')
    const steps: PlanStep[] = []
    let currentOrder = 1

    for (const line of lines) {
      if (line.match(/^\d+\./)) {
        steps.push({
          order: currentOrder++,
          action: line.replace(/^\d+\.\s*/, '').split(':')[0].trim(),
          description: line,
          estimatedMs: 5000,
        })
      }
    }

    return {
      steps,
      estimatedTime: steps.length * 5000,
      filesToCreate: lines.filter(l => l.includes('create')).map(l => l.match(/`([^`]+)`/)?.[1] || ''),
      filesToModify: lines.filter(l => l.includes('modify')).map(l => l.match(/`([^`]+)`/)?.[1] || ''),
      risks: lines.filter(l => l.startsWith('RISK:')).map(l => l.replace('RISK:', '').trim()),
    }
  }

  private parseCodeResult(output: string): CodeResult {
    const filesCreatedMatch = output.match(/Files created: (\d+)/)
    const filesModifiedMatch = output.match(/Files modified: (\d+)/)

    return {
      filesCreated: output.match(/Created: ([^\n]+)/)?.[1]?.split(',').map(f => f.trim()) || [],
      filesModified: output.match(/Modified: ([^\n]+)/)?.[1]?.split(',').map(f => f.trim()) || [],
      linesAdded: parseInt(output.match(/Lines added: (\d+)/)?.[1] || '0'),
      linesRemoved: parseInt(output.match(/Lines removed: (\d+)/)?.[1] || '0'),
      summary: output.split('\n').find(l => l.startsWith('Summary:'))?.replace('Summary:', '').trim() || '',
    }
  }

  private parseReviewResult(output: string): ReviewResult {
    const issues: ReviewIssue[] = []
    const lines = output.split('\n')

    for (const line of lines) {
      const match = line.match(/(\w+):\s*(.+) at (.+)(?: line (\d+))?/)
      if (match) {
        issues.push({
          severity: match[1].toLowerCase() as ReviewIssue['severity'],
          message: match[2],
          file: match[3],
          line: match[4] ? parseInt(match[4]) : undefined,
        })
      }
    }

    const scoreMatch = output.match(/Score: (\d+)\/100/)
    const score = scoreMatch ? parseInt(scoreMatch[1]) : 100

    return {
      issues,
      score,
      passed: issues.filter(i => i.severity === 'critical' || i.severity === 'high').length === 0,
      suggestions: lines.filter(l => l.startsWith('Suggestion:')).map(l => l.replace('Suggestion:', '').trim()),
    }
  }

  private parseAdjustmentsResult(output: string): AdjustmentsResult {
    return {
      filesModified: output.match(/Modified: ([^\n]+)/)?.[1]?.split(',').map(f => f.trim()) || [],
      changesApplied: parseInt(output.match(/Changes applied: (\d+)/)?.[1] || '0'),
      issuesFixed: parseInt(output.match(/Issues fixed: (\d+)/)?.[1] || '0'),
      remainingIssues: [],
    }
  }

  private parseTestResult(output: string): TestResult {
    const passedMatch = output.match(/Passed: (\d+)/)
    const failedMatch = output.match(/Failed: (\d+)/)
    const coverageMatch = output.match(/Coverage: (\d+)%/)

    return {
      totalTests: parseInt(passedMatch?.[1] || '0') + parseInt(failedMatch?.[1] || '0'),
      passed: parseInt(passedMatch?.[1] || '0'),
      failed: parseInt(failedMatch?.[1] || '0'),
      coverage: coverageMatch ? parseInt(coverageMatch[1]) : undefined,
      failures: [],
    }
  }

  private parseDocumentationResult(output: string): DocumentationResult {
    return {
      filesGenerated: output.match(/Generated: ([^\n]+)/)?.[1]?.split(',').map(f => f.trim()) || [],
      readmeUpdated: output.includes('README updated'),
      apiDocsGenerated: output.includes('API docs generated'),
    }
  }

  /**
   * Get current context
   */
  getContext(): FlowContext {
    return this.context
  }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

function generateTaskId(): string {
  return `task-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

/**
 * Create and execute a development flow
 */
export async function executeDevelopmentFlow(
  request: string,
  workspace: string,
  options?: Partial<FlowContext>
): Promise<FlowContext> {
  const orchestrator = new DevelopmentFlowOrchestrator({
    request,
    workspace,
    ...options,
  })

  return orchestrator.execute()
}

/**
 * Get development flow status for display
 */
export function getFlowStatus(context: FlowContext): {
  stage: string
  progress: number
  errors: number
  completed: boolean
} {
  const totalStages = DEVELOPMENT_FLOW_STAGES.length
  const completedStages = context.stageHistory.filter(e => e.success).length

  return {
    stage: context.currentStage,
    progress: Math.round((completedStages / totalStages) * 100),
    errors: context.errors.length,
    completed: context.currentStage === '' && context.stageHistory.length > 0,
  }
}