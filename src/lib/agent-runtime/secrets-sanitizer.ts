// Secrets Sanitizer - Prevent secrets exposure in logs and context
// Implements MVP Blueprint checklist: secrets hidden, sensitive data redacted

/**
 * Secret patterns to detect and redact
 */
export const SECRET_PATTERNS: Array<{
  name: string
  pattern: RegExp
  replacement: string
  description: string
}> = [
  // API Keys
  {
    name: 'openai_api_key',
    pattern: /sk-[a-zA-Z0-9]{20,}/g,
    replacement: '[OPENAI_API_KEY_REDACTED]',
    description: 'OpenAI API key',
  },
  {
    name: 'anthropic_api_key',
    pattern: /sk-ant-[a-zA-Z0-9]{20,}/g,
    replacement: '[ANTHROPIC_API_KEY_REDACTED]',
    description: 'Anthropic API key',
  },
  {
    name: 'generic_api_key',
    pattern: /(?:api[_-]?key|apikey)[\s]*[=:][\s]*['"]?[a-zA-Z0-9]{20,}['"]?/gi,
    replacement: '[API_KEY_REDACTED]',
    description: 'Generic API key',
  },

  // AWS Credentials
  {
    name: 'aws_access_key',
    pattern: /AKIA[A-Z0-9]{16}/g,
    replacement: '[AWS_ACCESS_KEY_REDACTED]',
    description: 'AWS access key ID',
  },
  {
    name: 'aws_secret_key',
    pattern: /(?:aws[_-]?secret[_-]?key|aws[_-]?secret[_-]?access[_-]?key)[\s]*[=:][\s]*['"]?[a-zA-Z0-9/+=]{40}['"]?/gi,
    replacement: '[AWS_SECRET_KEY_REDACTED]',
    description: 'AWS secret key',
  },

  // Passwords
  {
    name: 'password_assignment',
    pattern: /(?:password|passwd|pwd)[\s]*[=:][\s]*['"][^'"]{8,}['"]/gi,
    replacement: 'password="[REDACTED]"',
    description: 'Password in assignment',
  },
  {
    name: 'password_url',
    pattern: /:\/\/[^:]+:[^@]+@/g,
    replacement: '://[USER]:[PASSWORD_REDACTED]@',
    description: 'Password in URL',
  },

  // Tokens
  {
    name: 'bearer_token',
    pattern: /Bearer[\s]+[a-zA-Z0-9_-]{20,}/gi,
    replacement: 'Bearer [TOKEN_REDACTED]',
    description: 'Bearer token',
  },
  {
    name: 'jwt_token',
    pattern: /eyJ[a-zA-Z0-9_-]{10,}\.eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}/g,
    replacement: '[JWT_TOKEN_REDACTED]',
    description: 'JWT token',
  },
  {
    name: 'oauth_token',
    pattern: /(?:access_token|oauth_token)[\s]*[=:][\s]*['"]?[a-zA-Z0-9_-]{20,}['"]?/gi,
    replacement: '[OAUTH_TOKEN_REDACTED]',
    description: 'OAuth token',
  },

  // Private Keys
  {
    name: 'private_key_pem',
    pattern: /-----BEGIN[\s]+(?:RSA[\s]+)?PRIVATE[\s]+KEY-----[\s\S]*?-----END[\s]+(?:RSA[\s]+)?PRIVATE[\s]+KEY-----/g,
    replacement: '[PRIVATE_KEY_REDACTED]',
    description: 'PEM private key',
  },
  {
    name: 'ssh_private_key',
    pattern: /-----BEGIN[\s]+OPENSSH[\s]+PRIVATE[\s]+KEY-----[\s\S]*?-----END[\s]+OPENSSH[\s]+PRIVATE[\s]+KEY-----/g,
    replacement: '[SSH_KEY_REDACTED]',
    description: 'OpenSSH private key',
  },

  // Database Connection Strings
  {
    name: 'database_url_with_password',
    pattern: /(?:mysql|postgres|mongodb|redis):\/\/[^:]+:[^@]+@[^\s]+/gi,
    replacement: '[DATABASE_URL_REDACTED]',
    description: 'Database URL with credentials',
  },

  // Slack/Discord/Webhook tokens
  {
    name: 'slack_token',
    pattern: /xox[baprs]-[a-zA-Z0-9-]{10,}/g,
    replacement: '[SLACK_TOKEN_REDACTED]',
    description: 'Slack token',
  },
  {
    name: 'discord_token',
    pattern: /(?:discord[_-]?token)[\s]*[=:][\s]*['"]?[a-zA-Z0-9_-]{20,}['"]?/gi,
    replacement: '[DISCORD_TOKEN_REDACTED]',
    description: 'Discord token',
  },
  {
    name: 'webhook_url',
    pattern: /https?:\/\/hooks\.slack\.com\/services\/[A-Z0-9\/]+/gi,
    replacement: '[SLACK_WEBHOOK_REDACTED]',
    description: 'Slack webhook URL',
  },

  // Telegram Bot Token
  {
    name: 'telegram_token',
    pattern: /[0-9]{8,10}:[a-zA-Z0-9_-]{35}/g,
    replacement: '[TELEGRAM_TOKEN_REDACTED]',
    description: 'Telegram bot token',
  },

  // GitHub Token
  {
    name: 'github_token',
    pattern: /(?:ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9]{36}/g,
    replacement: '[GITHUB_TOKEN_REDACTED]',
    description: 'GitHub token',
  },

  // Stripe Key
  {
    name: 'stripe_key',
    pattern: /sk_live_[a-zA-Z0-9]{24,}/g,
    replacement: '[STRIPE_KEY_REDACTED]',
    description: 'Stripe live key',
  },

  // Email addresses (optional redaction)
  {
    name: 'email_address',
    pattern: /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g,
    replacement: '[EMAIL_REDACTED]',
    description: 'Email address',
  },

  // Credit card numbers
  {
    name: 'credit_card',
    pattern: /\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b/g,
    replacement: '[CARD_NUMBER_REDACTED]',
    description: 'Credit card number',
  },

  // IP addresses (optional redaction for internal IPs)
  {
    name: 'ip_address_private',
    pattern: /\b(?:10\.|172\.(?:1[6-9]|2[0-9]|3[01])\.|192\.168\.)[0-9.]+\b/g,
    replacement: '[PRIVATE_IP_REDACTED]',
    description: 'Private IP address',
  },
]

/**
 * Sensitive file patterns
 */
export const SENSITIVE_FILE_PATTERNS: Array<{
  pattern: string
  reason: string
  redact: boolean
}> = [
  { pattern: '.env', reason: 'Environment file with secrets', redact: true },
  { pattern: '.env.local', reason: 'Local environment secrets', redact: true },
  { pattern: '.env.production', reason: 'Production secrets', redact: true },
  { pattern: 'credentials.json', reason: 'Credential file', redact: true },
  { pattern: 'secrets.json', reason: 'Secrets file', redact: true },
  { pattern: '.npmrc', reason: 'NPM config with tokens', redact: true },
  { pattern: '.pypirc', reason: 'PyPI config with tokens', redact: true },
  { pattern: 'id_rsa', reason: 'SSH private key', redact: true },
  { pattern: 'id_ed25519', reason: 'SSH private key', redact: true },
  { pattern: '*.pem', reason: 'Certificate/key file', redact: true },
  { pattern: '*.key', reason: 'Key file', redact: true },
  { pattern: '*.p12', reason: 'Certificate with private key', redact: true },
  { pattern: '/etc/shadow', reason: 'System password file', redact: true },
  { pattern: '/etc/passwd', reason: 'System user file', redact: false },
]

/**
 * Secrets sanitizer class
 */
export class SecretsSanitizer {
  private patterns: typeof SECRET_PATTERNS
  private enabledPatterns: Set<string>
  private redactionLog: Array<{ pattern: string; location: string; timestamp: number }>

  constructor(options?: {
    enabledPatterns?: string[]
    disablePatterns?: string[]
  }) {
    this.patterns = [...SECRET_PATTERNS]
    this.enabledPatterns = new Set(this.patterns.map(p => p.name))
    this.redactionLog = []

    // Handle options
    if (options?.enabledPatterns) {
      this.enabledPatterns = new Set(options.enabledPatterns)
    }
    if (options?.disablePatterns) {
      for (const name of options.disablePatterns) {
        this.enabledPatterns.delete(name)
      }
    }
  }

  /**
   * Sanitize content to remove secrets
   */
  sanitize(content: string, location?: string): string {
    let sanitized = content
    const timestamp = Date.now()

    for (const pattern of this.patterns) {
      if (!this.enabledPatterns.has(pattern.name)) continue

      const matches = content.match(pattern.pattern)
      if (matches) {
        sanitized = sanitized.replace(pattern.pattern, pattern.replacement)

        // Log redaction
        for (const match of matches) {
          this.redactionLog.push({
            pattern: pattern.name,
            location: location || 'unknown',
            timestamp,
          })
        }
      }
    }

    return sanitized
  }

  /**
   * Check if content contains secrets
   */
  containsSecrets(content: string): { hasSecrets: boolean; detected: string[] } {
    const detected: string[] = []

    for (const pattern of this.patterns) {
      if (pattern.pattern.test(content)) {
        detected.push(pattern.name)
      }
    }

    return { hasSecrets: detected.length > 0, detected }
  }

  /**
   * Check if file is sensitive
   */
  isSensitiveFile(path: string): { sensitive: boolean; reason?: string } {
    for (const filePattern of SENSITIVE_FILE_PATTERNS) {
      if (path.includes(filePattern.pattern) || path.endsWith(filePattern.pattern)) {
        return { sensitive: true, reason: filePattern.reason }
      }
    }
    return { sensitive: false }
  }

  /**
   * Get redaction log
   */
  getRedactionLog(): typeof this.redactionLog {
    return [...this.redactionLog]
  }

  /**
   * Clear redaction log
   */
  clearRedactionLog(): void {
    this.redactionLog = []
  }

  /**
   * Get statistics
   */
  getStats(): {
    totalRedactions: number
    byPattern: Record<string, number>
    recentRedactions: number
  } {
    const byPattern: Record<string, number> = {}
    for (const entry of this.redactionLog) {
      byPattern[entry.pattern] = (byPattern[entry.pattern] ?? 0) + 1
    }

    const oneHourAgo = Date.now() - 3600000
    const recentRedactions = this.redactionLog.filter(e => e.timestamp > oneHourAgo).length

    return {
      totalRedactions: this.redactionLog.length,
      byPattern,
      recentRedactions,
    }
  }

  /**
   * Enable a pattern
   */
  enablePattern(name: string): void {
    this.enabledPatterns.add(name)
  }

  /**
   * Disable a pattern
   */
  disablePattern(name: string): void {
    this.enabledPatterns.delete(name)
  }

  /**
   * Get all pattern names
   */
  getPatternNames(): string[] {
    return this.patterns.map(p => p.name)
  }

  /**
   * Get enabled pattern names
   */
  getEnabledPatternNames(): string[] {
    return Array.from(this.enabledPatterns)
  }
}

/**
 * Sanitize object recursively
 */
export function sanitizeObject(
  obj: unknown,
  sanitizer: SecretsSanitizer,
  location?: string
): unknown {
  if (typeof obj === 'string') {
    return sanitizer.sanitize(obj, location)
  }

  if (Array.isArray(obj)) {
    return obj.map((item, i) => sanitizeObject(item, sanitizer, `${location}[${i}]`))
  }

  if (obj && typeof obj === 'object') {
    const sanitized: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
      const newLocation = location ? `${location}.${key}` : key
      sanitized[key] = sanitizeObject(value, sanitizer, newLocation)
    }
    return sanitized
  }

  return obj
}

/**
 * Sanitize JSON string
 */
export function sanitizeJsonString(
  jsonString: string,
  sanitizer: SecretsSanitizer,
  location?: string
): string {
  try {
    const obj = JSON.parse(jsonString)
    const sanitized = sanitizeObject(obj, sanitizer, location)
    return JSON.stringify(sanitized)
  } catch {
    // If not valid JSON, sanitize as string
    return sanitizer.sanitize(jsonString, location)
  }
}

// Singleton instance
let sanitizerInstance: SecretsSanitizer | null = null

export function getSecretsSanitizer(): SecretsSanitizer {
  if (!sanitizerInstance) {
    sanitizerInstance = new SecretsSanitizer()
  }
  return sanitizerInstance
}

export function createSecretsSanitizer(options?: {
  enabledPatterns?: string[]
  disablePatterns?: string[]
}): SecretsSanitizer {
  return new SecretsSanitizer(options)
}

/**
 * Default patterns to always enable (cannot be disabled)
 */
export const REQUIRED_PATTERNS = [
  'openai_api_key',
  'anthropic_api_key',
  'aws_access_key',
  'password_assignment',
  'private_key_pem',
  'credit_card',
]

/**
 * Patterns that can be optionally disabled
 */
export const OPTIONAL_PATTERNS = [
  'email_address',
  'ip_address_private',
]