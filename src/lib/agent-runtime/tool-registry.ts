// Tool Registry - Structured tool definitions with schema, risk class, timeout
// Implements MVP Blueprint checklist: typed tool registry, risk classes, approval logic

/**
 * Risk class for tool operations
 */
export type RiskClass =
  | 'read_public_data'      // Safe: reading approved public resources
  | 'read_private_data'     // Scope required: reading user/customer data
  | 'read_workspace_data'   // Allowed within cwd: reading workspace files
  | 'draft_output'          // Safe: drafting messages, reports, plans
  | 'write_workspace'       // Approval may be needed: writing workspace files
  | 'write_system'          // Deny by default: writing system files
  | 'external_send'         // Approval required: sending to external services
  | 'financial_action'      // Approval + strong auth: payments, refunds
  | 'destructive_action'    // Deny or approval + recovery: delete, rm -rf
  | 'identity_change'       // Approval + strong auth: permission changes
  | 'shell_execution'       // Sandbox + approval: bash/shell commands
  | 'browser_control'       // Scoped: browser automation
  | 'connector_call'        // Namespaced + scoped: MCP/external connectors

/**
 * Permission policy for a tool
 */
export type PermissionPolicy =
  | 'allow'                  // Always allowed within scope
  | 'allow_within_cwd'       // Allowed only within working directory
  | 'allow_with_scope'       // Allowed with user/session scope
  | 'approval_required'      // Requires user approval before execution
  | 'approval_strong_auth'   // Requires approval + strong authentication
  | 'sandbox_required'       // Must run in sandbox
  | 'deny'                   // Always denied
  | 'deny_with_recovery'     // Denied unless recovery path exists

/**
 * Side effects declaration
 */
export type SideEffects =
  | 'none'                   // No side effects
  | 'read_only'              // Only reads data
  | 'writes_file'            // Writes to files
  | 'writes_database'        // Writes to database
  | 'sends_external'         // Sends to external services
  | 'executes_process'       // Executes processes/shell commands
  | 'modifies_permissions'   // Changes permissions/access
  | 'financial_transaction'  // Performs financial operations
  | 'destructive'            // Deletes/destroys data

/**
 * Tool schema definition
 */
export interface ToolSchema {
  /** Tool name - specific and domain meaningful */
  name: string
  /** Purpose description - when to use and when not to use */
  purpose: string
  /** Risk class classification */
  riskClass: RiskClass
  /** Side effects declaration */
  sideEffects: SideEffects
  /** Permission policy */
  permission: PermissionPolicy
  /** Input schema - strict validation */
  inputSchema: Record<string, ToolInputField>
  /** Output schema - structured result */
  outputSchema: Record<string, ToolOutputField>
  /** Timeout in seconds */
  timeoutSeconds: number
  /** Maximum result size in characters */
  maxResultChars: number
  /** Retry policy */
  retryPolicy: RetryPolicy
  /** Audit level */
  auditLevel: 'none' | 'basic' | 'full' | 'detailed'
  /** Whether to redact sensitive data in results */
  redactSensitive: boolean
}

/**
 * Tool input field definition
 */
export interface ToolInputField {
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  required: boolean
  description?: string
  default?: unknown
  min?: number
  max?: number
  pattern?: string  // Regex pattern for validation
  enum?: string[]   // Allowed values
}

/**
 * Tool output field definition
 */
export interface ToolOutputField {
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  description?: string
  optional?: boolean
}

/**
 * Retry policy
 */
export interface RetryPolicy {
  /** Whether retries are allowed */
  allowed: boolean
  /** Maximum retry attempts */
  maxAttempts: number
  /** Delay between retries in ms */
  delayMs: number
  /** Whether to retry on timeout */
  retryOnTimeout: boolean
  /** Whether to retry on error */
  retryOnError: boolean
  /** Only retry on idempotent operations */
  requireIdempotent: boolean
}

/**
 * Structured tool result
 */
export interface StructuredToolResult {
  /** Tool call ID */
  toolCallId: string
  /** Tool name */
  toolName: string
  /** Status */
  status: 'success' | 'error' | 'denied' | 'timeout' | 'cancelled' | 'approval_required'
  /** Result data (on success) */
  data?: Record<string, unknown>
  /** Error message (on error/denied/timeout) */
  error?: string
  /** Reason for denial (on denied) */
  denialReason?: string
  /** Permission decision details */
  permissionDecision?: {
    policy: PermissionPolicy
    matchedRule?: string
    scope?: string
  }
  /** Execution metadata */
  metadata: {
    startTime: number
    endTime: number
    durationMs: number
    retryCount: number
    redactions: string[]
  }
}

/**
 * Default retry policies
 */
export const DEFAULT_RETRY_POLICIES = {
  none: {
    allowed: false,
    maxAttempts: 0,
    delayMs: 0,
    retryOnTimeout: false,
    retryOnError: false,
    requireIdempotent: true,
  },
  safe: {
    allowed: true,
    maxAttempts: 3,
    delayMs: 1000,
    retryOnTimeout: true,
    retryOnError: true,
    requireIdempotent: true,
  },
  cautious: {
    allowed: true,
    maxAttempts: 2,
    delayMs: 2000,
    retryOnTimeout: false,
    retryOnError: false,
    requireIdempotent: true,
  },
}

/**
 * Built-in tool definitions
 */
export const BUILTIN_TOOL_SCHEMAS: Record<string, ToolSchema> = {
  // Read operations
  read_file: {
    name: 'read_file',
    purpose: 'Read file content within workspace. Use for inspecting source code, config files, documentation.',
    riskClass: 'read_workspace_data',
    sideEffects: 'read_only',
    permission: 'allow_within_cwd',
    inputSchema: {
      path: { type: 'string', required: true, description: 'Absolute file path' },
      line: { type: 'number', required: false, min: 1, description: 'Start line (1-based)' },
      limit: { type: 'number', required: false, min: 1, max: 500, description: 'Max lines to read' },
    },
    outputSchema: {
      content: { type: 'string', description: 'File content' },
      lines_read: { type: 'number', description: 'Number of lines read' },
      truncated: { type: 'boolean', optional: true, description: 'Whether result was truncated' },
    },
    timeoutSeconds: 10,
    maxResultChars: 8000,
    retryPolicy: DEFAULT_RETRY_POLICIES.safe,
    auditLevel: 'basic',
    redactSensitive: true,
  },

  read_directory: {
    name: 'read_directory',
    purpose: 'List directory contents. Use for exploring project structure.',
    riskClass: 'read_workspace_data',
    sideEffects: 'read_only',
    permission: 'allow_within_cwd',
    inputSchema: {
      path: { type: 'string', required: true, description: 'Directory path' },
      recursive: { type: 'boolean', required: false, default: false },
    },
    outputSchema: {
      entries: { type: 'array', description: 'Directory entries' },
      count: { type: 'number', description: 'Number of entries' },
    },
    timeoutSeconds: 5,
    maxResultChars: 4000,
    retryPolicy: DEFAULT_RETRY_POLICIES.safe,
    auditLevel: 'basic',
    redactSensitive: false,
  },

  // Write operations
  write_file: {
    name: 'write_file',
    purpose: 'Write content to a file. Use for creating or updating source files, configs.',
    riskClass: 'write_workspace',
    sideEffects: 'writes_file',
    permission: 'approval_required',
    inputSchema: {
      path: { type: 'string', required: true, description: 'File path to write' },
      content: { type: 'string', required: true, description: 'Content to write' },
      mode: { type: 'string', required: false, enum: ['create', 'overwrite', 'append'], default: 'overwrite' },
    },
    outputSchema: {
      path: { type: 'string', description: 'Written file path' },
      bytes_written: { type: 'number', description: 'Bytes written' },
    },
    timeoutSeconds: 15,
    maxResultChars: 1000,
    retryPolicy: DEFAULT_RETRY_POLICIES.cautious,
    auditLevel: 'full',
    redactSensitive: false,
  },

  edit_file: {
    name: 'edit_file',
    purpose: 'Edit existing file with string replacement. Use for targeted code modifications.',
    riskClass: 'write_workspace',
    sideEffects: 'writes_file',
    permission: 'approval_required',
    inputSchema: {
      path: { type: 'string', required: true },
      old_string: { type: 'string', required: true, description: 'Text to replace' },
      new_string: { type: 'string', required: true, description: 'Replacement text' },
      replace_all: { type: 'boolean', required: false, default: false },
    },
    outputSchema: {
      path: { type: 'string' },
      replacements: { type: 'number', description: 'Number of replacements made' },
    },
    timeoutSeconds: 10,
    maxResultChars: 500,
    retryPolicy: DEFAULT_RETRY_POLICIES.none,
    auditLevel: 'full',
    redactSensitive: false,
  },

  // Shell operations
  bash_command: {
    name: 'bash_command',
    purpose: 'Execute shell command. Use for build, test, git operations. NOT for destructive commands.',
    riskClass: 'shell_execution',
    sideEffects: 'executes_process',
    permission: 'sandbox_required',
    inputSchema: {
      command: { type: 'string', required: true, description: 'Command to execute' },
      timeout: { type: 'number', required: false, min: 1000, max: 300000, default: 60000 },
      cwd: { type: 'string', required: false, description: 'Working directory' },
    },
    outputSchema: {
      stdout: { type: 'string', description: 'Standard output' },
      stderr: { type: 'string', optional: true, description: 'Standard error' },
      exit_code: { type: 'number', description: 'Exit code' },
      duration_ms: { type: 'number', description: 'Execution duration' },
    },
    timeoutSeconds: 60,
    maxResultChars: 10000,
    retryPolicy: DEFAULT_RETRY_POLICIES.cautious,
    auditLevel: 'detailed',
    redactSensitive: true,
  },

  // Search operations
  search_files: {
    name: 'search_files',
    purpose: 'Search for files matching pattern. Use for finding specific files.',
    riskClass: 'read_workspace_data',
    sideEffects: 'read_only',
    permission: 'allow_within_cwd',
    inputSchema: {
      pattern: { type: 'string', required: true, description: 'Glob pattern' },
      path: { type: 'string', required: false, description: 'Search root path' },
    },
    outputSchema: {
      matches: { type: 'array', description: 'Matching file paths' },
      count: { type: 'number', description: 'Number of matches' },
    },
    timeoutSeconds: 10,
    maxResultChars: 4000,
    retryPolicy: DEFAULT_RETRY_POLICIES.safe,
    auditLevel: 'basic',
    redactSensitive: false,
  },

  search_content: {
    name: 'search_content',
    purpose: 'Search content in files. Use for finding code patterns, keywords.',
    riskClass: 'read_workspace_data',
    sideEffects: 'read_only',
    permission: 'allow_within_cwd',
    inputSchema: {
      pattern: { type: 'string', required: true, description: 'Search pattern (regex supported)' },
      path: { type: 'string', required: false },
      file_pattern: { type: 'string', required: false, description: 'File glob pattern' },
    },
    outputSchema: {
      matches: { type: 'array', description: 'Search results with file:line:content' },
      count: { type: 'number' },
    },
    timeoutSeconds: 15,
    maxResultChars: 6000,
    retryPolicy: DEFAULT_RETRY_POLICIES.safe,
    auditLevel: 'basic',
    redactSensitive: false,
  },

  // Draft operations (always safe)
  draft_message: {
    name: 'draft_message',
    purpose: 'Draft a message, email, or communication without sending.',
    riskClass: 'draft_output',
    sideEffects: 'none',
    permission: 'allow',
    inputSchema: {
      content: { type: 'string', required: true },
      format: { type: 'string', required: false, enum: ['plain', 'markdown', 'html'] },
    },
    outputSchema: {
      draft: { type: 'string', description: 'Drafted message' },
      format: { type: 'string' },
    },
    timeoutSeconds: 5,
    maxResultChars: 5000,
    retryPolicy: DEFAULT_RETRY_POLICIES.none,
    auditLevel: 'none',
    redactSensitive: false,
  },

  draft_plan: {
    name: 'draft_plan',
    purpose: 'Draft an implementation plan without executing.',
    riskClass: 'draft_output',
    sideEffects: 'none',
    permission: 'allow',
    inputSchema: {
      objective: { type: 'string', required: true },
      constraints: { type: 'array', required: false },
    },
    outputSchema: {
      plan: { type: 'string', description: 'Drafted plan' },
      steps: { type: 'array', description: 'Proposed steps' },
      risks: { type: 'array', optional: true },
    },
    timeoutSeconds: 5,
    maxResultChars: 3000,
    retryPolicy: DEFAULT_RETRY_POLICIES.none,
    auditLevel: 'none',
    redactSensitive: false,
  },

  // External send (requires approval)
  send_message: {
    name: 'send_message',
    purpose: 'Send message to external service (Slack, email, etc.). Requires approval.',
    riskClass: 'external_send',
    sideEffects: 'sends_external',
    permission: 'approval_required',
    inputSchema: {
      channel: { type: 'string', required: true, enum: ['slack', 'email', 'discord', 'telegram'] },
      recipient: { type: 'string', required: true },
      content: { type: 'string', required: true },
    },
    outputSchema: {
      status: { type: 'string', description: 'Send status' },
      message_id: { type: 'string', optional: true },
      timestamp: { type: 'number', optional: true },
    },
    timeoutSeconds: 30,
    maxResultChars: 1000,
    retryPolicy: DEFAULT_RETRY_POLICIES.none,
    auditLevel: 'full',
    redactSensitive: true,
  },

  // Dangerous operations (deny by default)
  delete_file: {
    name: 'delete_file',
    purpose: 'Delete a file. DENIED by default - requires explicit approval with recovery plan.',
    riskClass: 'destructive_action',
    sideEffects: 'destructive',
    permission: 'deny_with_recovery',
    inputSchema: {
      path: { type: 'string', required: true },
      recovery_path: { type: 'string', required: true, description: 'Backup path for recovery' },
    },
    outputSchema: {
      status: { type: 'string' },
      recovery_backup: { type: 'string', optional: true },
    },
    timeoutSeconds: 10,
    maxResultChars: 500,
    retryPolicy: DEFAULT_RETRY_POLICIES.none,
    auditLevel: 'detailed',
    redactSensitive: false,
  },
}

/**
 * Tool registry class
 */
export class ToolRegistry {
  private schemas: Map<string, ToolSchema> = new Map()
  private customTools: Map<string, ToolSchema> = new Map()

  constructor() {
    // Load built-in tools
    for (const [name, schema] of Object.entries(BUILTIN_TOOL_SCHEMAS)) {
      this.schemas.set(name, schema)
    }
  }

  /**
   * Get tool schema by name
   */
  get(name: string): ToolSchema | undefined {
    return this.schemas.get(name) || this.customTools.get(name)
  }

  /**
   * Check if tool exists
   */
  has(name: string): boolean {
    return this.schemas.has(name) || this.customTools.has(name)
  }

  /**
   * Register custom tool
   */
  register(schema: ToolSchema): void {
    this.customTools.set(schema.name, schema)
  }

  /**
   * Get all tool names
   */
  getAllNames(): string[] {
    return [...this.schemas.keys(), ...this.customTools.keys()]
  }

  /**
   * Get tools by risk class
   */
  getByRiskClass(riskClass: RiskClass): ToolSchema[] {
    return [...this.schemas.values(), ...this.customTools.values()]
      .filter(s => s.riskClass === riskClass)
  }

  /**
   * Get tools requiring approval
   */
  getApprovalRequired(): ToolSchema[] {
    return [...this.schemas.values(), ...this.customTools.values()]
      .filter(s => s.permission.includes('approval'))
  }

  /**
   * Validate tool input
   */
  validateInput(name: string, input: Record<string, unknown>): { valid: boolean; errors: string[] } {
    const schema = this.get(name)
    if (!schema) {
      return { valid: false, errors: [`Unknown tool: ${name}`] }
    }

    const errors: string[] = []

    for (const [field, def] of Object.entries(schema.inputSchema)) {
      if (def.required && !(field in input)) {
        errors.push(`Missing required field: ${field}`)
        continue
      }

      if (field in input) {
        const value = input[field]

        // Type check
        if (def.type === 'string' && typeof value !== 'string') {
          errors.push(`Field ${field} must be string`)
        }
        if (def.type === 'number' && typeof value !== 'number') {
          errors.push(`Field ${field} must be number`)
        }
        if (def.type === 'boolean' && typeof value !== 'boolean') {
          errors.push(`Field ${field} must be boolean`)
        }
        if (def.type === 'array' && !Array.isArray(value)) {
          errors.push(`Field ${field} must be array`)
        }

        // Min/max for numbers
        if (def.type === 'number' && typeof value === 'number') {
          if (def.min !== undefined && value < def.min) {
            errors.push(`Field ${field} must be >= ${def.min}`)
          }
          if (def.max !== undefined && value > def.max) {
            errors.push(`Field ${field} must be <= ${def.max}`)
          }
        }

        // Enum check
        if (def.enum && typeof value === 'string' && !def.enum.includes(value)) {
          errors.push(`Field ${field} must be one of: ${def.enum.join(', ')}`)
        }

        // Pattern check
        if (def.pattern && typeof value === 'string') {
          const regex = new RegExp(def.pattern)
          if (!regex.test(value)) {
            errors.push(`Field ${field} must match pattern: ${def.pattern}`)
          }
        }
      }
    }

    return { valid: errors.length === 0, errors }
  }

  /**
   * Get visible tools for a session (filtered by permission)
   */
  getVisibleTools(sessionScope: string[]): ToolSchema[] {
    return [...this.schemas.values(), ...this.customTools.values()]
      .filter(s => {
        // Always show draft tools
        if (s.riskClass === 'draft_output') return true

        // Show workspace tools if in workspace scope
        if (s.permission === 'allow_within_cwd' && sessionScope.includes('workspace')) return true

        // Show read tools with proper scope
        if (s.riskClass.startsWith('read_') && sessionScope.includes('read')) return true

        // Show approval-required tools (but mark them)
        if (s.permission.includes('approval')) return true

        // Deny destructive by default
        if (s.permission === 'deny' || s.permission === 'deny_with_recovery') return false

        return true
      })
  }

  /**
   * Create structured result
   */
  createResult(
    toolCallId: string,
    toolName: string,
    status: StructuredToolResult['status'],
    data?: Record<string, unknown>,
    error?: string,
    metadata?: Partial<StructuredToolResult['metadata']>
  ): StructuredToolResult {
    return {
      toolCallId,
      toolName,
      status,
      data,
      error,
      metadata: {
        startTime: metadata?.startTime ?? Date.now(),
        endTime: metadata?.endTime ?? Date.now(),
        durationMs: metadata?.durationMs ?? 0,
        retryCount: metadata?.retryCount ?? 0,
        redactions: metadata?.redactions ?? [],
      },
    }
  }

  /**
   * Create denied result
   */
  createDeniedResult(
    toolCallId: string,
    toolName: string,
    reason: string,
    policy: PermissionPolicy,
    matchedRule?: string
  ): StructuredToolResult {
    return {
      toolCallId,
      toolName,
      status: 'denied',
      error: `Operation denied: ${reason}`,
      denialReason: reason,
      permissionDecision: {
        policy,
        matchedRule,
      },
      metadata: {
        startTime: Date.now(),
        endTime: Date.now(),
        durationMs: 0,
        retryCount: 0,
        redactions: [],
      },
    }
  }

  /**
   * Create approval required result
   */
  createApprovalRequiredResult(
    toolCallId: string,
    toolName: string,
    policy: PermissionPolicy
  ): StructuredToolResult {
    const schema = this.get(toolName)
    return {
      toolCallId,
      toolName,
      status: 'approval_required',
      error: 'Operation requires user approval',
      permissionDecision: {
        policy,
        scope: schema?.riskClass,
      },
      metadata: {
        startTime: Date.now(),
        endTime: Date.now(),
        durationMs: 0,
        retryCount: 0,
        redactions: [],
      },
    }
  }
}

// Singleton instance
let toolRegistryInstance: ToolRegistry | null = null

export function getToolRegistry(): ToolRegistry {
  if (!toolRegistryInstance) {
    toolRegistryInstance = new ToolRegistry()
  }
  return toolRegistryInstance
}

export function createToolRegistry(): ToolRegistry {
  return new ToolRegistry()
}