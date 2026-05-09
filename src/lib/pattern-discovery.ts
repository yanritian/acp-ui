/**
 * Pattern Discovery Engine
 *
 * Analyzes historical data to discover useful patterns
 */

import type { EvolutionRecord, PatternRecord } from '../stores/error'
import type { MemoryRecord } from '../stores/memory'

export interface DiscoveredPattern {
  name: string
  description: string
  category: string
  evidence: string[]
  confidence: number
}

/**
 * Extract patterns from memory records
 */
export function extractPatternsFromMemories(memories: MemoryRecord[]): DiscoveredPattern[] {
  const patterns: DiscoveredPattern[] = []

  // Group memories by type
  const factMemories = memories.filter(m => m.memoryType === 'fact' || m.memoryType === 'pattern')

  // Analyze content for recurring themes
  const contentMap = new Map<string, number>()

  for (const memory of factMemories) {
    // Extract key phrases
    const phrases = extractKeyPhrases(memory.content)
    for (const phrase of phrases) {
      const count = contentMap.get(phrase) ?? 0
      contentMap.set(phrase, count + 1)
    }
  }

  // Find recurring phrases (appear more than 2 times)
  for (const [phrase, count] of contentMap.entries()) {
    if (count >= 2) {
      const category = detectPatternCategory(phrase)
      patterns.push({
        name: phrase,
        description: `在 ${count} 次记录中出现的模式`,
        category,
        evidence: factMemories
          .filter(m => m.content.includes(phrase))
          .map(m => m.content.slice(0, 100)),
        confidence: count / factMemories.length,
      })
    }
  }

  return patterns.slice(0, 10)
}

function extractKeyPhrases(content: string): string[] {
  const phrases: string[] = []

  // Extract noun phrases (simplified)
  const words = content.split(/[\s,，。.!！?？]+/)
  const meaningfulWords = words.filter(w => w.length >= 2)

  // Create phrases from consecutive meaningful words
  for (let i = 0; i < meaningfulWords.length - 1; i++) {
    const phrase = meaningfulWords[i] + ' ' + meaningfulWords[i + 1]
    phrases.push(phrase)
  }

  return phrases
}

function detectPatternCategory(content: string): string {
  const lowerContent = content.toLowerCase()

  if (lowerContent.includes('代码') || lowerContent.includes('函数') || lowerContent.includes('类')) {
    return 'coding'
  }
  if (lowerContent.includes('架构') || lowerContent.includes('模块') || lowerContent.includes('设计')) {
    return 'architecture'
  }
  if (lowerContent.includes('测试') || lowerContent.includes('验证') || lowerContent.includes('覆盖')) {
    return 'testing'
  }
  if (lowerContent.includes('流程') || lowerContent.includes('步骤') || lowerContent.includes('顺序')) {
    return 'workflow'
  }

  return 'coding'
}

/**
 * Analyze evolution records to find improvement patterns
 */
export function analyzeEvolutionPatterns(evolutions: EvolutionRecord[]): DiscoveredPattern[] {
  const patterns: DiscoveredPattern[] = []

  // Group by domain
  const domainGroups = new Map<string, EvolutionRecord[]>()

  for (const evo of evolutions) {
    const records = domainGroups.get(evo.domain) ?? []
    records.push(evo)
    domainGroups.set(evo.domain, records)
  }

  // Analyze each domain
  for (const [domain, records] of domainGroups.entries()) {
    const improvements = records.filter(r => r.evolutionType === 'improvement')

    if (improvements.length >= 2) {
      // Find common improvement patterns in this domain
      const commonPatterns = findCommonChanges(improvements)

      for (const pattern of commonPatterns) {
        patterns.push({
          name: `${domain}领域改进模式`,
          description: pattern,
          category: detectPatternCategory(domain),
          evidence: improvements.map(i => i.reason).slice(0, 3),
          confidence: improvements.length / records.length,
        })
      }
    }
  }

  return patterns
}

function findCommonChanges(evolutions: EvolutionRecord[]): string[] {
  const changes: string[] = []

  // Extract common keywords from reasons
  const allReasons = evolutions.map(e => e.reason).join(' ')
  const keywords = extractKeyPhrases(allReasons)

  // Find recurring themes
  const keywordCounts = new Map<string, number>()
  for (const kw of keywords) {
    const count = keywordCounts.get(kw) ?? 0
    keywordCounts.set(kw, count + 1)
  }

  for (const [kw, count] of keywordCounts.entries()) {
    if (count >= 2) {
      changes.push(`常见改进: ${kw}`)
    }
  }

  return changes
}

/**
 * Discover patterns from all available data
 */
export function discoverPatterns(
  memories: MemoryRecord[],
  evolutions: EvolutionRecord[],
  existingPatterns: PatternRecord[]
): DiscoveredPattern[] {
  // Extract from memories
  const memoryPatterns = extractPatternsFromMemories(memories)

  // Extract from evolutions
  const evolutionPatterns = analyzeEvolutionPatterns(evolutions)

  // Combine and deduplicate
  const allPatterns = [...memoryPatterns, ...evolutionPatterns]

  // Filter out patterns that already exist
  const existingNames = new Set(existingPatterns.map(p => p.name.toLowerCase()))
  const newPatterns = allPatterns.filter(
    p => !existingNames.has(p.name.toLowerCase())
  )

  // Sort by confidence
  return newPatterns.sort((a, b) => b.confidence - a.confidence).slice(0, 10)
}

/**
 * Score pattern quality
 */
export function scorePattern(pattern: PatternRecord): number {
  // High usage count = useful pattern
  const usageScore = Math.min(0.5, pattern.usageCount * 0.05)

  // High success rate = reliable pattern
  const successScore = (pattern.successRate ?? 0) * 0.3

  // Has examples = well-documented pattern
  const exampleScore = pattern.examples ? 0.2 : 0

  return usageScore + successScore + exampleScore
}