// QA Agent Enhanced - Auto-fix suggestions + Test validation + Performance verification
// Phase 2 enhancement for complete development cycle QA

import { invokeSkill } from './skill-system/skill-invoker'
import { trackEvent, trackError } from './self-improvement/telemetry'
import type { ReviewResult, ReviewError, ReviewWarning } from './code-review-agent'

// ---------------------------------------------------------------------------
// QA Enhanced Types
// ---------------------------------------------------------------------------

export interface QAAutoFixSuggestion {
  errorId: string
  file: string
  line: number
  originalCode: string
  suggestedFix: string
  fixType: 'type' | 'syntax' | 'security' | 'performance' | 'style'
  confidence: number // 0-1
  autoApplicable: boolean // Can be safely auto-applied
  explanation: string
}

export interface QATestValidation {
  testCoverage: number
  unitTestsPassed: number
  unitTestsFailed: number
  integrationTestsPassed: number
  integrationTestsFailed: number
  criticalPathsCovered: boolean
  missingTests: string[]
}

export interface QAPerformanceValidation {
  buildTimeMs: number
  bundleSizeKB: number
  loadTimeMs: number
  memoryUsageMB: number
  performanceScore: number
  bottlenecks: string[]
}

export interface QAFullReport {
  reviewResult: ReviewResult
  autoFixSuggestions: QAAutoFixSuggestion[]
  testValidation: QATestValidation
  performanceValidation: QAPerformanceValidation
  overallScore: number
  passStatus: 'pass' | 'fail' | 'needs-review'
  timestamp: number
}

export interface QAConfig {
  autoFixEnabled: boolean
  autoFixConfidenceThreshold: number // Minimum confidence for auto-apply
  testCoverageThreshold: number
  performanceThreshold: number
  criticalErrorsBlock: boolean
}

export const DEFAULT_QA_CONFIG: QAConfig = {
  autoFixEnabled: true,
  autoFixConfidenceThreshold: 0.8,
  testCoverageThreshold: 80,
  performanceThreshold: 70,
  criticalErrorsBlock: true,
}

// ---------------------------------------------------------------------------
// QA Agent Enhanced Class
// ---------------------------------------------------------------------------

export class QAAgentEnhanced {
  private config: QAConfig

  constructor(config?: Partial<QAConfig>) {
    this.config = { ...DEFAULT_QA_CONFIG, ...config }
  }

  /**
   * Execute full QA validation
   */
  async fullQAValidation(projectPath: string): Promise<QAFullReport> {
    const startTime = Date.now()

    // 1. Code review
    const reviewResult = await this.runCodeReview(projectPath)

    // 2. Generate auto-fix suggestions
    const autoFixSuggestions = await this.generateAutoFixSuggestions(reviewResult.errors)

    // 3. Test validation
    const testValidation = await this.validateTests(projectPath)

    // 4. Performance validation
    const performanceValidation = await this.validatePerformance(projectPath)

    // 5. Calculate overall score
    const overallScore = this.calculateOverallScore(
      reviewResult.score,
      testValidation.testCoverage,
      performanceValidation.performanceScore
    )

    // 6. Determine pass status
    const passStatus = this.determinePassStatus(
      reviewResult,
      testValidation,
      performanceValidation,
      overallScore
    )

    const report: QAFullReport = {
      reviewResult,
      autoFixSuggestions,
      testValidation,
      performanceValidation,
      overallScore,
      passStatus,
      timestamp: Date.now(),
    }

    trackEvent({
      type: 'behavior',
      name: 'qa-validation-complete',
      data: {
        overallScore,
        passStatus,
        autoFixCount: autoFixSuggestions.length,
        testCoverage: testValidation.testCoverage,
        durationMs: Date.now() - startTime,
      },
    })

    return report
  }

  /**
   * Run code review using existing agent
   */
  private async runCodeReview(projectPath: string): Promise<ReviewResult> {
    try {
      const result = await invokeSkill('code-review', {
        action: 'full-review',
        projectPath,
      })

      if (!result.success) {
        return {
          success: false,
          errors: [],
          warnings: [],
          score: 0,
          report: result.error || 'Code review failed',
        }
      }

      // Parse review result from output
      return this.parseReviewResult(result.output)
    } catch {
      return {
        success: false,
        errors: [],
        warnings: [],
        score: 0,
        report: 'Code review failed',
      }
    }
  }

  /**
   * Generate auto-fix suggestions for errors
   */
  async generateAutoFixSuggestions(errors: ReviewError[]): Promise<QAAutoFixSuggestion[]> {
    if (!this.config.autoFixEnabled || errors.length === 0) {
      return []
    }

    const suggestions: QAAutoFixSuggestion[] = []

    for (const error of errors) {
      // Only suggest fixes for non-critical errors (unless confidence is very high)
      if (error.severity === 'critical') continue

      try {
        const result = await invokeSkill('code-execution', {
          action: 'suggest-fix',
          file: error.file,
          line: error.line,
          errorMessage: error.message,
          rule: error.rule,
        })

        if (result.success) {
          const suggestion = this.parseAutoFixSuggestion(result.output, error)
          if (suggestion && suggestion.confidence >= this.config.autoFixConfidenceThreshold) {
            suggestions.push(suggestion)
          }
        }
      } catch (err: unknown) {
        // Track error via telemetry per project coding standards
        trackError(err instanceof Error ? err : new Error(String(err)), {
          context: 'qa-auto-fix-suggestion',
        })
      }
    }

    return suggestions
  }

  /**
   * Validate test coverage and results
   */
  private async validateTests(projectPath: string): Promise<QATestValidation> {
    try {
      const result = await invokeSkill('testing', {
        action: 'validate-coverage',
        projectPath,
      })

      if (!result.success) {
        return {
          testCoverage: 0,
          unitTestsPassed: 0,
          unitTestsFailed: 0,
          integrationTestsPassed: 0,
          integrationTestsFailed: 0,
          criticalPathsCovered: false,
          missingTests: [],
        }
      }

      return this.parseTestValidation(result.output)
    } catch {
      return {
        testCoverage: 0,
        unitTestsPassed: 0,
        unitTestsFailed: 0,
        integrationTestsPassed: 0,
        integrationTestsFailed: 0,
        criticalPathsCovered: false,
        missingTests: [],
      }
    }
  }

  /**
   * Validate performance metrics
   */
  private async validatePerformance(projectPath: string): Promise<QAPerformanceValidation> {
    try {
      const result = await invokeSkill('testing', {
        action: 'performance-analysis',
        projectPath,
      })

      if (!result.success) {
        return {
          buildTimeMs: 0,
          bundleSizeKB: 0,
          loadTimeMs: 0,
          memoryUsageMB: 0,
          performanceScore: 0,
          bottlenecks: [],
        }
      }

      return this.parsePerformanceValidation(result.output)
    } catch {
      return {
        buildTimeMs: 0,
        bundleSizeKB: 0,
        loadTimeMs: 0,
        memoryUsageMB: 0,
        performanceScore: 0,
        bottlenecks: [],
      }
    }
  }

  /**
   * Apply auto-fix suggestions
   */
  async applyAutoFixes(suggestions: QAAutoFixSuggestion[]): Promise<{
    applied: number
    failed: number
    results: Array<{ suggestion: QAAutoFixSuggestion; success: boolean; error?: string }>
  }> {
    const results: Array<{ suggestion: QAAutoFixSuggestion; success: boolean; error?: string }> = []
    let applied = 0
    let failed = 0

    for (const suggestion of suggestions) {
      if (!suggestion.autoApplicable) {
        results.push({ suggestion, success: false, error: 'Not auto-applicable' })
        failed++
        continue
      }

      try {
        const result = await invokeSkill('file-operations', {
          action: 'apply-fix',
          file: suggestion.file,
          line: suggestion.line,
          fix: suggestion.suggestedFix,
          original: suggestion.originalCode,
        })

        if (result.success) {
          results.push({ suggestion, success: true })
          applied++
          trackEvent({
            type: 'behavior',
            name: 'qa-auto-fix-applied',
            data: { file: suggestion.file, fixType: suggestion.fixType },
          })
        } else {
          results.push({ suggestion, success: false, error: result.error })
          failed++
        }
      } catch (error) {
        results.push({ suggestion, success: false, error: String(error) })
        failed++
      }
    }

    return { applied, failed, results }
  }

  // ---------------------------------------------------------------------------
// Parsing Methods
// ---------------------------------------------------------------------------

  private parseReviewResult(output: string): ReviewResult {
    const lines = output.split('\n')
    const errors: ReviewError[] = []
    const warnings: ReviewWarning[] = []

    // Parse score
    const scoreMatch = output.match(/Score:\s*(\d+)\/100/)
    const score = scoreMatch ? parseInt(scoreMatch[1]) : 100

    // Parse errors
    for (const line of lines) {
      const errorMatch = line.match(/\[(\w+)\]\s*(.+):(\d+)\s*-\s*(.+)\s*\((\w+)\)/)
      if (errorMatch) {
        errors.push({
          file: errorMatch[2],
          line: parseInt(errorMatch[3]),
          column: 0,
          message: errorMatch[4],
          rule: errorMatch[5],
          severity: errorMatch[1].toLowerCase() as ReviewError['severity'],
        })
      }

      const warningMatch = line.match(/Warning:\s*(.+):(\d+)\s*-\s*(.+)/)
      if (warningMatch) {
        warnings.push({
          file: warningMatch[1],
          line: parseInt(warningMatch[2]),
          message: warningMatch[3],
          suggestion: '',
        })
      }
    }

    return {
      success: errors.filter(e => e.severity === 'critical').length === 0,
      errors,
      warnings,
      score,
      report: output,
    }
  }

  private parseAutoFixSuggestion(output: string, error: ReviewError): QAAutoFixSuggestion | null {
    const lines = output.split('\n')

    const originalMatch = output.match(/Original:\s*`([^`]+)`/)
    const fixMatch = output.match(/Fix:\s*`([^`]+)`/)
    const confidenceMatch = output.match(/Confidence:\s*(\d+)%/)
    const typeMatch = output.match(/Type:\s*(\w+)/)
    const explanationMatch = output.match(/Explanation:\s*(.+)/)

    if (!originalMatch || !fixMatch) return null

    const confidence = confidenceMatch ? parseInt(confidenceMatch[1]) / 100 : 0.5
    const fixType = (typeMatch?.[1] || 'syntax') as QAAutoFixSuggestion['fixType']

    return {
      errorId: `${error.file}:${error.line}`,
      file: error.file,
      line: error.line,
      originalCode: originalMatch[1],
      suggestedFix: fixMatch[1],
      fixType,
      confidence,
      autoApplicable: confidence >= this.config.autoFixConfidenceThreshold && fixType !== 'security',
      explanation: explanationMatch?.[1] || '',
    }
  }

  private parseTestValidation(output: string): QATestValidation {
    const coverageMatch = output.match(/Coverage:\s*(\d+)%/)
    const unitPassedMatch = output.match(/Unit tests passed:\s*(\d+)/)
    const unitFailedMatch = output.match(/Unit tests failed:\s*(\d+)/)
    const integrationPassedMatch = output.match(/Integration tests passed:\s*(\d+)/)
    const integrationFailedMatch = output.match(/Integration tests failed:\s*(\d+)/)
    const criticalMatch = output.match(/Critical paths covered:\s*(true|false)/)

    const missingTestsMatch = output.match(/Missing tests:\s*\n((?:\s*-\s*.+\n?)+)/)
    const missingTests: string[] = []
    if (missingTestsMatch) {
      const testLines = missingTestsMatch[1].split('\n')
      for (const line of testLines) {
        const test = line.replace(/^\s*-\s*/, '').trim()
        if (test) missingTests.push(test)
      }
    }

    return {
      testCoverage: coverageMatch ? parseInt(coverageMatch[1]) : 0,
      unitTestsPassed: unitPassedMatch ? parseInt(unitPassedMatch[1]) : 0,
      unitTestsFailed: unitFailedMatch ? parseInt(unitFailedMatch[1]) : 0,
      integrationTestsPassed: integrationPassedMatch ? parseInt(integrationPassedMatch[1]) : 0,
      integrationTestsFailed: integrationFailedMatch ? parseInt(integrationFailedMatch[1]) : 0,
      criticalPathsCovered: criticalMatch?.[1] === 'true',
      missingTests,
    }
  }

  private parsePerformanceValidation(output: string): QAPerformanceValidation {
    const buildMatch = output.match(/Build time:\s*(\d+)ms/)
    const bundleMatch = output.match(/Bundle size:\s*(\d+)KB/)
    const loadMatch = output.match(/Load time:\s*(\d+)ms/)
    const memoryMatch = output.match(/Memory usage:\s*(\d+)MB/)
    const scoreMatch = output.match(/Performance score:\s*(\d+)\/100/)

    const bottlenecksMatch = output.match(/Bottlenecks:\s*\n((?:\s*-\s*.+\n?)+)/)
    const bottlenecks: string[] = []
    if (bottlenecksMatch) {
      const bnLines = bottlenecksMatch[1].split('\n')
      for (const line of bnLines) {
        const bn = line.replace(/^\s*-\s*/, '').trim()
        if (bn) bottlenecks.push(bn)
      }
    }

    return {
      buildTimeMs: buildMatch ? parseInt(buildMatch[1]) : 0,
      bundleSizeKB: bundleMatch ? parseInt(bundleMatch[1]) : 0,
      loadTimeMs: loadMatch ? parseInt(loadMatch[1]) : 0,
      memoryUsageMB: memoryMatch ? parseInt(memoryMatch[1]) : 0,
      performanceScore: scoreMatch ? parseInt(scoreMatch[1]) : 0,
      bottlenecks,
    }
  }

  // ---------------------------------------------------------------------------
// Scoring Methods
// ---------------------------------------------------------------------------

  private calculateOverallScore(
    reviewScore: number,
    testCoverage: number,
    performanceScore: number
  ): number {
    // Weighted average: Review 40%, Tests 40%, Performance 20%
    return Math.round(reviewScore * 0.4 + testCoverage * 0.4 + performanceScore * 0.2)
  }

  private determinePassStatus(
    review: ReviewResult,
    tests: QATestValidation,
    performance: QAPerformanceValidation,
    overallScore: number
  ): 'pass' | 'fail' | 'needs-review' {
    // Critical errors block
    if (this.config.criticalErrorsBlock && review.errors.some((e: ReviewError) => e.severity === 'critical')) {
      return 'fail'
    }

    // Test coverage threshold
    if (tests.testCoverage < this.config.testCoverageThreshold) {
      return 'needs-review'
    }

    // Performance threshold
    if (performance.performanceScore < this.config.performanceThreshold) {
      return 'needs-review'
    }

    // Overall score threshold
    if (overallScore >= 80) {
      return 'pass'
    }

    if (overallScore >= 60) {
      return 'needs-review'
    }

    return 'fail'
  }
}

// ---------------------------------------------------------------------------
// Singleton Instance
// ---------------------------------------------------------------------------

export const qaAgentEnhanced = new QAAgentEnhanced()

// ---------------------------------------------------------------------------
// Convenience Functions
// ---------------------------------------------------------------------------

/**
 * Run full QA validation
 */
export async function runFullQA(projectPath: string): Promise<QAFullReport> {
  return qaAgentEnhanced.fullQAValidation(projectPath)
}

/**
 * Get auto-fix suggestions from errors
 */
export async function getAutoFixSuggestions(errors: ReviewError[]): Promise<QAAutoFixSuggestion[]> {
  return qaAgentEnhanced.generateAutoFixSuggestions(errors)
}

/**
 * Apply approved auto-fixes
 */
export async function applyAutoFixes(suggestions: QAAutoFixSuggestion[]): Promise<{
  applied: number
  failed: number
  results: Array<{ suggestion: QAAutoFixSuggestion; success: boolean; error?: string }>
}> {
  return qaAgentEnhanced.applyAutoFixes(suggestions)
}