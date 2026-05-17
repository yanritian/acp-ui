/**
 * Error-Solution Matching Engine
 *
 * Matches new errors with historical solutions for auto-fix suggestions
 */

import type { ErrorRecord, SolutionRecord } from '../stores/error'

export interface MatchedSolution {
  solution: SolutionRecord
  similarity: number
  matchReason: string
}

// Error category patterns for matching
const CATEGORY_PATTERNS: Record<string, RegExp[]> = {
  build_error: [
    /cannot find module/i,
    /module not found/i,
    /failed to compile/i,
    /syntax error/i,
    /type error/i,
    /undefined variable/i,
    /import error/i,
  ],
  runtime_error: [
    /null pointer/i,
    /undefined is not/i,
    /cannot read property/i,
    /maximum call stack/i,
    /out of memory/i,
    /timeout/i,
    /connection refused/i,
  ],
  logic_error: [
    /incorrect result/i,
    /unexpected behavior/i,
    /wrong output/i,
    /not working as expected/i,
    /逻辑错误/i,
    /结果不正确/i,
  ],
  dependency_error: [
    /dependency not found/i,
    /version conflict/i,
    /package not installed/i,
    /npm install/i,
    /peer dependency/i,
    /版本冲突/i,
  ],
  config_error: [
    /invalid config/i,
    /missing config/i,
    /config file not found/i,
    /parse error in config/i,
    /environment variable/i,
    /配置错误/i,
  ],
}

// Common solution templates by category (returns translation keys)
const SOLUTION_TEMPLATES: Record<string, string[]> = {
  build_error: [
    'errorSuggestions.checkImportPath',
    'errorSuggestions.ensureModuleInstalled',
    'errorSuggestions.checkSyntaxTypes',
    'errorSuggestions.clearCacheRebuild',
    'errorSuggestions.checkTsconfig',
  ],
  runtime_error: [
    'errorSuggestions.addNullCheck',
    'errorSuggestions.checkObjectExists',
    'errorSuggestions.optimizeRecursion',
    'errorSuggestions.increaseTimeout',
    'errorSuggestions.checkConnection',
  ],
  dependency_error: [
    'errorSuggestions.runNpmInstall',
    'errorSuggestions.checkPackageVersion',
    'errorSuggestions.deleteNodeModules',
    'errorSuggestions.useLegacyPeerDeps',
    'errorSuggestions.lockDependencyVersion',
  ],
  config_error: [
    'errorSuggestions.checkConfigPath',
    'errorSuggestions.validateConfigFormat',
    'errorSuggestions.setEnvVariables',
    'errorSuggestions.checkConfigSyntax',
    'errorSuggestions.restoreDefaultConfig',
  ],
}

/**
 * Calculate similarity between two error messages
 */
export function calculateSimilarity(error1: string, error2: string): number {
  // Normalize messages
  const norm1 = normalizeMessage(error1)
  const norm2 = normalizeMessage(error2)

  // Tokenize
  const tokens1 = tokenize(norm1)
  const tokens2 = tokenize(norm2)

  // Jaccard similarity
  const set1 = new Set(tokens1)
  const set2 = new Set(tokens2)
  const intersection = new Set([...set1].filter(x => set2.has(x)))
  const union = new Set([...set1, ...set2])

  return intersection.size / union.size
}

function normalizeMessage(msg: string): string {
  return msg
    .toLowerCase()
    .replace(/[^\w\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

function tokenize(msg: string): string[] {
  return msg.split(' ').filter(t => t.length > 2)
}

/**
 * Match error with historical solutions
 */
export function matchSolution(
  error: ErrorRecord,
  historicalSolutions: SolutionRecord[]
): MatchedSolution[] {
  const matches: MatchedSolution[] = []

  // Get all successful solutions for same category
  const relevantSolutions = historicalSolutions.filter(
    s => s.success === true
  )

  for (const solution of relevantSolutions) {
    // Calculate text similarity
    const similarity = calculateSimilarity(error.message, solution.approach)

    if (similarity > 0.3) {
      matches.push({
        solution,
        similarity,
        matchReason: `errorSuggestions.similarity: ${Math.round(similarity * 100)}%`,
      })
    }
  }

  // Sort by similarity
  return matches.sort((a, b) => b.similarity - a.similarity)
}

/**
 * Detect error category from message
 */
export function detectCategory(message: string): string {
  for (const [category, patterns] of Object.entries(CATEGORY_PATTERNS)) {
    for (const pattern of patterns) {
      if (pattern.test(message)) {
        return category
      }
    }
  }
  return 'runtime_error' // Default category
}

/**
 * Get suggested solutions for an error
 */
export function getSuggestedFixes(error: ErrorRecord): string[] {
  const category = error.category

  // Get template suggestions for the category
  const templates = SOLUTION_TEMPLATES[category] ?? []

  // Add context-specific suggestions based on message content
  const contextSuggestions = extractContextSuggestions(error.message)

  return [...templates, ...contextSuggestions].slice(0, 5)
}

function extractContextSuggestions(message: string): string[] {
  const suggestions: string[] = []

  // File path suggestions
  if (message.includes('.ts') || message.includes('.tsx')) {
    suggestions.push('errorSuggestions.checkTsTypes')
  }
  if (message.includes('.vue')) {
    suggestions.push('errorSuggestions.checkVueProps')
  }
  if (message.includes('.js') || message.includes('.jsx')) {
    suggestions.push('errorSuggestions.checkJsExports')
  }

  // Import suggestions
  if (message.includes('import') || message.includes('require')) {
    suggestions.push('errorSuggestions.verifyImportPath')
  }

  // Function suggestions
  if (message.includes('function') || message.includes('method')) {
    suggestions.push('errorSuggestions.checkFunctionSignature')
  }

  return suggestions
}

/**
 * Auto-classify and suggest fix for an error
 */
export function autoAnalyzeError(message: string, stackTrace?: string): {
  category: string
  suggestedFixes: string[]
  priority: number
} {
  const category = detectCategory(message)
  const suggestedFixes = getSuggestedFixes({ id: '', category, message, context: null, stackTrace: stackTrace ?? null, agentId: null, taskId: null, status: 'open', solutionId: null, createdAt: '', resolvedAt: null })

  // Calculate priority based on severity indicators
  let priority = 1
  if (message.includes('critical') || message.includes('fatal') || message.includes('崩溃')) {
    priority = 3
  } else if (message.includes('error') || message.includes('失败')) {
    priority = 2
  }

  return {
    category,
    suggestedFixes,
    priority,
  }
}