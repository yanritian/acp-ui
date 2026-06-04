// Tests for P1/P2 improvements: Tool Registry, Secrets Sanitizer, Context Builder, Telemetry

import { describe, it, expect, beforeEach } from 'vitest'
import {
  ToolRegistry,
  getToolRegistry,
  BUILTIN_TOOL_SCHEMAS,
  RiskClass,
  PermissionPolicy,
  DEFAULT_RETRY_POLICIES,
} from '../tool-registry'
import {
  SecretsSanitizer,
  getSecretsSanitizer,
  SECRET_PATTERNS,
  sanitizeObject,
  sanitizeJsonString,
} from '../secrets-sanitizer'
import {
  ContextBuilder,
  createContextBuilder,
  DEFAULT_SYSTEM_INSTRUCTIONS,
  DEFAULT_HARNESS_POLICY,
  ContextSectionType,
  TrustLevel,
} from '../context-builder'
import {
  TelemetryStore,
  getTelemetryStore,
  calculateCost,
  MODEL_PRICING,
} from '../telemetry'

// ============================================
// Tool Registry Tests
// ============================================

describe('ToolRegistry', () => {
  let registry: ToolRegistry

  beforeEach(() => {
    registry = new ToolRegistry()
  })

  describe('Built-in Tools', () => {
    it('should have built-in tools loaded', () => {
      expect(registry.has('read_file')).toBe(true)
      expect(registry.has('write_file')).toBe(true)
      expect(registry.has('bash_command')).toBe(true)
      expect(registry.has('draft_message')).toBe(true)
    })

    it('should return tool schema', () => {
      const schema = registry.get('read_file')
      expect(schema).toBeDefined()
      expect(schema?.riskClass).toBe('read_workspace_data')
      expect(schema?.permission).toBe('allow_within_cwd')
      expect(schema?.timeoutSeconds).toBe(10)
    })

    it('should have correct risk classes', () => {
      expect(registry.get('draft_message')?.riskClass).toBe('draft_output')
      expect(registry.get('write_file')?.riskClass).toBe('write_workspace')
      expect(registry.get('bash_command')?.riskClass).toBe('shell_execution')
      expect(registry.get('delete_file')?.riskClass).toBe('destructive_action')
      expect(registry.get('send_message')?.riskClass).toBe('external_send')
    })

    it('should have correct permissions', () => {
      expect(registry.get('read_file')?.permission).toBe('allow_within_cwd')
      expect(registry.get('draft_message')?.permission).toBe('allow')
      expect(registry.get('write_file')?.permission).toBe('approval_required')
      expect(registry.get('delete_file')?.permission).toBe('deny_with_recovery')
    })
  })

  describe('Input Validation', () => {
    it('should validate required fields', () => {
      const result = registry.validateInput('read_file', {})
      expect(result.valid).toBe(false)
      expect(result.errors).toContain('Missing required field: path')
    })

    it('should validate valid input', () => {
      const result = registry.validateInput('read_file', { path: '/home/user/file.txt' })
      expect(result.valid).toBe(true)
      expect(result.errors.length).toBe(0)
    })

    it('should validate min/max constraints', () => {
      const result = registry.validateInput('read_file', {
        path: '/file.txt',
        limit: 1000, // exceeds max of 500
      })
      expect(result.valid).toBe(false)
      expect(result.errors.some(e => e.includes('must be <='))).toBe(true)
    })

    it('should validate enum values', () => {
      const result = registry.validateInput('write_file', {
        path: '/file.txt',
        content: 'test',
        mode: 'invalid_mode', // not in enum
      })
      expect(result.valid).toBe(false)
      expect(result.errors.some(e => e.includes('must be one of'))).toBe(true)
    })

    it('should reject unknown tool', () => {
      const result = registry.validateInput('unknown_tool', {})
      expect(result.valid).toBe(false)
      expect(result.errors).toContain('Unknown tool: unknown_tool')
    })
  })

  describe('Custom Tools', () => {
    it('should register custom tool', () => {
      registry.register({
        name: 'custom_tool',
        purpose: 'Custom tool for testing',
        riskClass: 'read_workspace_data',
        sideEffects: 'read_only',
        permission: 'allow',
        inputSchema: { param: { type: 'string', required: true } },
        outputSchema: { result: { type: 'string' } },
        timeoutSeconds: 5,
        maxResultChars: 1000,
        retryPolicy: DEFAULT_RETRY_POLICIES.none,
        auditLevel: 'basic',
        redactSensitive: false,
      })

      expect(registry.has('custom_tool')).toBe(true)
      expect(registry.get('custom_tool')?.name).toBe('custom_tool')
    })
  })

  describe('Tool Filtering', () => {
    it('should get tools by risk class', () => {
      const draftTools = registry.getByRiskClass('draft_output')
      expect(draftTools.length).toBeGreaterThan(0)
      expect(draftTools.every(t => t.riskClass === 'draft_output')).toBe(true)
    })

    it('should get approval required tools', () => {
      const approvalTools = registry.getApprovalRequired()
      expect(approvalTools.length).toBeGreaterThan(0)
      expect(approvalTools.every(t => t.permission.includes('approval'))).toBe(true)
    })

    it('should get visible tools for session', () => {
      const visible = registry.getVisibleTools(['workspace', 'read'])
      expect(visible.length).toBeGreaterThan(0)

      // Draft tools should always be visible
      expect(visible.some(t => t.riskClass === 'draft_output')).toBe(true)
    })
  })

  describe('Result Creation', () => {
    it('should create success result', () => {
      const result = registry.createResult(
        'tc-1',
        'read_file',
        'success',
        { content: 'file content' }
      )

      expect(result.status).toBe('success')
      expect(result.toolCallId).toBe('tc-1')
      expect(result.data?.content).toBe('file content')
      expect(result.metadata.durationMs).toBeDefined()
    })

    it('should create denied result', () => {
      const result = registry.createDeniedResult(
        'tc-1',
        'delete_file',
        'Destructive operation not allowed',
        'deny_with_recovery',
        'builtin-dangerous'
      )

      expect(result.status).toBe('denied')
      expect(result.denialReason).toBe('Destructive operation not allowed')
      expect(result.permissionDecision?.policy).toBe('deny_with_recovery')
    })

    it('should create approval required result', () => {
      const result = registry.createApprovalRequiredResult(
        'tc-1',
        'send_message',
        'approval_required'
      )

      expect(result.status).toBe('approval_required')
      expect(result.permissionDecision?.policy).toBe('approval_required')
    })
  })
})

// ============================================
// Secrets Sanitizer Tests
// ============================================

describe('SecretsSanitizer', () => {
  let sanitizer: SecretsSanitizer

  beforeEach(() => {
    sanitizer = new SecretsSanitizer()
  })

  describe('API Key Detection', () => {
    it('should detect and redact OpenAI API key', () => {
      // Use a key that matches the pattern exactly
      const content = 'api_key: sk-abcdefghij1234567890klmnopqrst'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).toContain('[OPENAI_API_KEY_REDACTED]')
      expect(sanitized).not.toContain('sk-abcdefghij1234567890')
    })

    it('should detect and redact Anthropic API key', () => {
      // Use a key that matches the pattern
      const content = 'key: sk-ant-abcdefghij1234567890klmnopqrst'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).toContain('[ANTHROPIC_API_KEY_REDACTED]')
      expect(sanitized).not.toContain('sk-ant-abcdefghij1234567890')
    })

    it('should detect and redact AWS access key', () => {
      const content = 'aws_key: AKIAIOSFODNN7EXAMPLE'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).not.toContain('AKIAIOSFODNN7EXAMPLE')
      expect(sanitized).toContain('[AWS_ACCESS_KEY_REDACTED]')
    })
  })

  describe('Password Detection', () => {
    it('should detect and redact password assignment', () => {
      const content = 'password = "my_secret_password_123"'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).not.toContain('my_secret_password')
      expect(sanitized).toContain('password="[REDACTED]"')
    })

    it('should detect and redact password in URL', () => {
      const content = 'mysql://user:password123@localhost/db'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).not.toContain('password123')
      expect(sanitized).toContain('[DATABASE_URL_REDACTED]')
    })
  })

  describe('Token Detection', () => {
    it('should detect and redact JWT token', () => {
      const content = 'token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).toContain('[JWT_TOKEN_REDACTED]')
    })

    it('should detect and redact Bearer token', () => {
      const content = 'Authorization: Bearer abc123def456ghi789jkl'
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).not.toContain('abc123def456ghi789jkl')
      expect(sanitized).toContain('[TOKEN_REDACTED]')
    })
  })

  describe('Private Key Detection', () => {
    it('should detect and redact PEM private key', () => {
      const content = `
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC7VJTUt9Us8cKj
-----END PRIVATE KEY-----
`
      const sanitized = sanitizer.sanitize(content)

      expect(sanitized).toContain('[PRIVATE_KEY_REDACTED]')
      expect(sanitized).not.toContain('MIIEvgIBADANBgkqhkiG9w0')
    })
  })

  describe('Secret Detection Check', () => {
    it('should detect secrets in content', () => {
      const content = 'password = "my_secret_password_123"'
      const result = sanitizer.containsSecrets(content)

      expect(result.hasSecrets).toBe(true)
      expect(result.detected).toContain('password_assignment')
    })

    it('should not detect secrets in safe content', () => {
      const content = 'This is a normal text without any secrets'
      const result = sanitizer.containsSecrets(content)

      expect(result.hasSecrets).toBe(false)
      expect(result.detected.length).toBe(0)
    })
  })

  describe('Sensitive File Detection', () => {
    it('should identify sensitive files', () => {
      expect(sanitizer.isSensitiveFile('.env').sensitive).toBe(true)
      expect(sanitizer.isSensitiveFile('credentials.json').sensitive).toBe(true)
      expect(sanitizer.isSensitiveFile('id_rsa').sensitive).toBe(true)
      expect(sanitizer.isSensitiveFile('/etc/shadow').sensitive).toBe(true)
    })

    it('should not identify non-sensitive files', () => {
      expect(sanitizer.isSensitiveFile('README.md').sensitive).toBe(false)
      expect(sanitizer.isSensitiveFile('src/index.ts').sensitive).toBe(false)
    })
  })

  describe('Statistics', () => {
    it('should track redactions', () => {
      sanitizer.sanitize('key: sk-test-12345678901234567890', 'test')
      sanitizer.sanitize('password = "secret123"', 'test')

      const stats = sanitizer.getStats()
      expect(stats.totalRedactions).toBeGreaterThan(0)
    })

    it('should enable/disable patterns', () => {
      sanitizer.disablePattern('email_address')
      const content = 'email: user@example.com'
      const sanitized = sanitizer.sanitize(content)

      // Should not redact email when disabled
      expect(sanitized).toContain('user@example.com')

      sanitizer.enablePattern('email_address')
      const sanitized2 = sanitizer.sanitize(content)
      expect(sanitized2).toContain('[EMAIL_REDACTED]')
    })
  })

  describe('Object Sanitization', () => {
    it('should sanitize nested objects', () => {
      const obj = {
        config: {
          connection: 'mysql://user:secret123@localhost/db',
        },
        safe: 'normal text',
      }

      const sanitized = sanitizeObject(obj, sanitizer) as Record<string, unknown>
      const config = sanitized.config as Record<string, unknown>
      expect(config.connection).toContain('[DATABASE_URL_REDACTED]')
      expect(sanitized.safe).toBe('normal text')
    })
  })
})

// ============================================
// Context Builder Tests
// ============================================

describe('ContextBuilder', () => {
  let builder: ContextBuilder

  beforeEach(() => {
    builder = new ContextBuilder()
  })

  describe('Section Management', () => {
    it('should add sections', () => {
      builder.addSystemInstructions('System instructions')
      builder.addToolDefinitions('Tool definitions')

      expect(builder.getSectionCount()).toBe(2)
    })

    it('should sort sections by deterministic order', () => {
      builder.addSection('user_input', 'User message', 'user')
      builder.addSection('system_instructions', 'System instructions', 'system')
      builder.addSection('tool_definitions', 'Tools', 'registry')

      const context = builder.build()
      // System instructions should come before user input
      expect(context.indexOf('system_instructions')).toBeLessThan(context.indexOf('user_input'))
    })

    it('should estimate tokens', () => {
      builder.addSystemInstructions('This is a test instruction')
      builder.addToolDefinitions('Tool: read_file')

      expect(builder.getTotalTokenEstimate()).toBeGreaterThan(0)
    })
  })

  describe('Trust Level Separation', () => {
    it('should mark untrusted sections', () => {
      builder.addRetrievedContent('Retrieved file content', 'file.txt')

      const stats = builder.getStats()
      expect(stats.untrustedSections).toBe(1)
    })

    it('should wrap untrusted content', () => {
      builder.addRetrievedContent('File content', 'file.txt')
      const context = builder.build()

      expect(context).toContain('[UNTRUSTED]')
      expect(context).toContain('UNTRUSTED CONTENT')
    })

    it('should separate trusted sections', () => {
      builder.addSystemInstructions('System prompt')
      builder.addDeveloperInstructions('Developer rules')
      builder.addRetrievedContent('File content', 'file.txt')

      const trusted = builder.getSectionsByTrustLevel('trusted')
      expect(trusted.length).toBe(2) // system + developer
    })
  })

  describe('Cacheable Sections', () => {
    it('should identify cacheable sections', () => {
      builder.addSystemInstructions('System prompt')
      builder.addToolDefinitions('Tool definitions')
      builder.addMessages([{ id: 'm1', role: 'user', content: 'Hello', timestamp: Date.now(), toolCalls: [] }])

      const stats = builder.getStats()
      expect(stats.cacheableSections).toBe(2) // system + tool
    })

    it('should generate stable prefix hash', () => {
      builder.addSystemInstructions('System prompt')
      builder.addToolDefinitions('Tool definitions')

      builder.build()
      const hash = builder.getStablePrefixHash()
      expect(hash).toBeDefined()
      expect(hash?.length).toBeGreaterThan(0)
    })
  })

  describe('Compaction Check', () => {
    it('should detect need for compaction', () => {
      // Create builder with low token limit
      builder = new ContextBuilder({ maxContextTokens: 1000 })

      // Add lots of content
      for (let i = 0; i < 100; i++) {
        builder.addSection('user_input', `User message ${i} with lots of content to fill the context window`, `msg-${i}`)
      }

      const needsCompaction = builder.needsCompaction(50) // 50% threshold
      expect(needsCompaction).toBe(true)
    })

    it('should not need compaction for small context', () => {
      builder.addSystemInstructions('System prompt')
      builder.addToolDefinitions('Tools')

      const needsCompaction = builder.needsCompaction(80)
      expect(needsCompaction).toBe(false)
    })
  })

  describe('Build Output', () => {
    it('should build valid context', () => {
      builder.addSystemInstructions('System instructions')
      builder.addDeveloperInstructions('Developer rules')
      builder.addMessages([{ id: 'm1', role: 'user', content: 'Hello', timestamp: Date.now(), toolCalls: [] }])

      const context = builder.build()
      expect(context).toContain('[TRUSTED] system_instructions')
      expect(context).toContain('[TRUSTED] developer_instructions')
      expect(context).toContain('[USER] user_input')
    })

    it('should include dynamic separator', () => {
      builder.addSystemInstructions('System')
      builder.addMessages([{ id: 'm1', role: 'user', content: 'Hello', timestamp: Date.now(), toolCalls: [] }])

      const context = builder.build()
      expect(context).toContain('Dynamic Context')
    })
  })

  describe('Clear', () => {
    it('should clear sections', () => {
      builder.addSystemInstructions('System')
      builder.addToolDefinitions('Tools')
      builder.clear()

      expect(builder.getSectionCount()).toBe(0)
      expect(builder.getStablePrefixHash()).toBeNull()
    })
  })
})

// ============================================
// Telemetry Tests
// ============================================

describe('TelemetryStore', () => {
  let telemetry: TelemetryStore

  beforeEach(() => {
    telemetry = new TelemetryStore()
  })

  describe('Token Recording', () => {
    it('should record token usage', () => {
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 200,
        costUsd: 0.01,
        latencyMs: 1000,
        cacheHit: true,
      })

      const records = telemetry.getSessionTokenRecords('session-1')
      expect(records.length).toBe(1)
      expect(records[0].inputTokens).toBe(1000)
    })

    it('should update session stats', () => {
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 0,
        costUsd: 0.01,
        latencyMs: 1000,
        cacheHit: false,
      })

      const session = telemetry.getSessionTelemetry('session-1')
      expect(session?.totalInputTokens).toBe(1000)
      expect(session?.totalOutputTokens).toBe(500)
    })
  })

  describe('Tool Recording', () => {
    it('should record tool calls', () => {
      // First record token usage to initialize session
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 100,
        outputTokens: 50,
        cachedTokens: 0,
        costUsd: 0.001,
        latencyMs: 100,
        cacheHit: false,
      })

      telemetry.recordToolCall('session-1', 'read_file', 'success')
      telemetry.recordToolCall('session-1', 'write_file', 'success')

      const session = telemetry.getSessionTelemetry('session-1')
      expect(session?.toolCalls.get('read_file')).toBe(1)
      expect(session?.toolCalls.get('write_file')).toBe(1)
    })
  })

  describe('Trace Recording', () => {
    it('should record trace events', () => {
      telemetry.recordTrace({
        sessionId: 'session-1',
        eventType: 'task_started',
        data: { taskId: 'task-1' },
      })

      telemetry.recordTrace({
        sessionId: 'session-1',
        eventType: 'tool_call_executed',
        data: { toolName: 'read_file' },
      })

      const traces = telemetry.getSessionTraces('session-1')
      expect(traces.length).toBe(2)
    })

    it('should get recent traces', () => {
      for (let i = 0; i < 20; i++) {
        telemetry.recordTrace({
          sessionId: `session-${i}`,
          eventType: 'task_started',
          data: {},
        })
      }

      const recent = telemetry.getRecentTraces(10)
      expect(recent.length).toBe(10)
    })
  })

  describe('Statistics', () => {
    it('should calculate overall stats', () => {
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 0,
        costUsd: 0.01,
        latencyMs: 1000,
        cacheHit: false,
      })

      telemetry.recordTokenUsage({
        sessionId: 'session-2',
        taskId: 'task-2',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 2000,
        outputTokens: 1000,
        cachedTokens: 500,
        costUsd: 0.02,
        latencyMs: 2000,
        cacheHit: true,
      })

      const stats = telemetry.getStats()
      expect(stats.totalRuns).toBe(2)
      expect(stats.totalInputTokens).toBe(3000)
      expect(stats.totalOutputTokens).toBe(1500)
      expect(stats.totalCachedTokens).toBe(500)
    })
  })

  describe('Export', () => {
    it('should export as JSON', () => {
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 0,
        costUsd: 0.01,
        latencyMs: 1000,
        cacheHit: false,
      })

      const exported = telemetry.export('json')
      expect(exported).toContain('totalRuns')
      expect(exported).toContain('session-1')
    })

    it('should export as CSV', () => {
      telemetry.recordTokenUsage({
        sessionId: 'session-1',
        taskId: 'task-1',
        modelProvider: 'anthropic',
        modelId: 'claude-sonnet-4-6',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 0,
        costUsd: 0.01,
        latencyMs: 1000,
        cacheHit: false,
      })

      const exported = telemetry.export('csv')
      expect(exported).toContain('timestamp')
      expect(exported).toContain('sessionId')
    })
  })
})

describe('Cost Calculation', () => {
  it('should calculate cost correctly', () => {
    const cost = calculateCost('claude-sonnet-4-6', 1000, 500, 0)
    expect(cost).toBeGreaterThan(0)

    // Claude Sonnet: $0.003/1k input, $0.015/1k output
    const expected = (1000 * 0.003 / 1000) + (500 * 0.015 / 1000)
    expect(Math.abs(cost - expected)).toBeLessThan(0.0001)
  })

  it('should account for cached tokens', () => {
    const costNoCache = calculateCost('claude-sonnet-4-6', 1000, 500, 0)
    const costWithCache = calculateCost('claude-sonnet-4-6', 1000, 500, 500)

    expect(costWithCache).toBeLessThan(costNoCache)
  })

  it('should handle unknown model', () => {
    const cost = calculateCost('unknown-model', 1000, 500, 0)
    expect(cost).toBeGreaterThan(0) // Uses default pricing
  })
})