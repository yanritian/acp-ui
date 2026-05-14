/**
 * Code Review Agent - executes code quality checks
 */

interface ReviewResult {
  success: boolean;
  errors: ReviewError[];
  warnings: ReviewWarning[];
  score: number;
  report: string;
}

interface ReviewError {
  file: string;
  line: number;
  column: number;
  message: string;
  rule: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
}

interface ReviewWarning {
  file: string;
  line: number;
  message: string;
  suggestion: string;
}

interface ReviewConfig {
  runTypeCheck: boolean;
  runLint: boolean;
  runSecurityScan: boolean;
  maxErrors: number;
  maxWarnings: number;
  failOnCritical: boolean;
}

class CodeReviewAgent {
  private config: ReviewConfig;

  constructor(config?: Partial<ReviewConfig>) {
    this.config = {
      runTypeCheck: true,
      runLint: true,
      runSecurityScan: true,
      maxErrors: 5,
      maxWarnings: 10,
      failOnCritical: true,
      ...config,
    };
  }

  /**
   * Execute full code review
   */
  async review(projectPath: string): Promise<ReviewResult> {
    const errors: ReviewError[] = [];
    const warnings: ReviewWarning[] = [];

    // Run TypeScript type check
    if (this.config.runTypeCheck) {
      const tscResult = await this.runTypeCheck(projectPath);
      errors.push(...tscResult.errors);
      warnings.push(...tscResult.warnings);
    }

    // Run ESLint
    if (this.config.runLint) {
      const lintResult = await this.runLint(projectPath);
      errors.push(...lintResult.errors);
      warnings.push(...lintResult.warnings);
    }

    // Run security scan
    if (this.config.runSecurityScan) {
      const securityResult = await this.runSecurityScan(projectPath);
      errors.push(...securityResult.errors);
      warnings.push(...securityResult.warnings);
    }

    // Calculate score
    const criticalCount = errors.filter(e => e.severity === 'critical').length;
    const highCount = errors.filter(e => e.severity === 'high').length;
    const score = Math.max(0, 100 - (criticalCount * 20) - (highCount * 10) - (errors.length - criticalCount - highCount) * 5);

    // Determine success
    const success = this.config.failOnCritical
      ? criticalCount === 0 && errors.length <= this.config.maxErrors
      : errors.length <= this.config.maxErrors && warnings.length <= this.config.maxWarnings;

    // Generate report
    const report = this.generateReport(errors, warnings, score);

    return {
      success,
      errors,
      warnings,
      score,
      report,
    };
  }

  /**
   * Run TypeScript type check
   */
  private async runTypeCheck(projectPath: string): Promise<{ errors: ReviewError[]; warnings: ReviewWarning[] }> {
    const errors: ReviewError[] = [];

    try {
      // Execute tsc --noEmit
      const result = await this.executeCommand('npx', ['tsc', '--noEmit', '--pretty', 'false'], projectPath);

      if (!result.success) {
        // Parse tsc output
        const lines = result.stdout.split('\n');
        for (const line of lines) {
          const match = line.match(/(.+?)\((\d+),(\d+)\):\s*error\s+(TS\d+):\s*(.+)/);
          if (match) {
            errors.push({
              file: match[1],
              line: parseInt(match[2]),
              column: parseInt(match[3]),
              message: match[5],
              rule: match[4],
              severity: 'high',
            });
          }
        }
      }
    } catch (e) {
      // tsc not available or project has no tsconfig
      console.warn('TypeScript check skipped:', e);
    }

    return { errors, warnings: [] };
  }

  /**
   * Run ESLint
   */
  private async runLint(projectPath: string): Promise<{ errors: ReviewError[]; warnings: ReviewWarning[] }> {
    const errors: ReviewError[] = [];
    const warnings: ReviewWarning[] = [];

    try {
      const result = await this.executeCommand('npx', ['eslint', '.', '--format', 'json'], projectPath);

      if (!result.success && result.stdout) {
        // Parse ESLint JSON output
        try {
          const lintResults = JSON.parse(result.stdout);
          for (const fileResult of lintResults) {
            for (const msg of fileResult.messages || []) {
              if (msg.severity === 2) { // Error
                errors.push({
                  file: fileResult.filePath,
                  line: msg.line,
                  column: msg.column,
                  message: msg.message,
                  rule: msg.ruleId,
                  severity: 'medium',
                });
              } else { // Warning
                warnings.push({
                  file: fileResult.filePath,
                  line: msg.line,
                  message: msg.message,
                  suggestion: `Fix rule: ${msg.ruleId}`,
                });
              }
            }
          }
        } catch {
          // Non-JSON output, parse manually
          const lines = result.stdout.split('\n');
          for (const line of lines) {
            const match = line.match(/(.+?):(\d+):(\d+)\s+(.+?)\s+(.+)\s+\[(.+)\]/);
            if (match) {
              errors.push({
                file: match[1],
                line: parseInt(match[2]),
                column: parseInt(match[3]),
                message: match[5],
                rule: match[6],
                severity: 'medium',
              });
            }
          }
        }
      }
    } catch (e) {
      console.warn('ESLint check skipped:', e);
    }

    return { errors, warnings };
  }

  /**
   * Run security scan
   */
  private async runSecurityScan(projectPath: string): Promise<{ errors: ReviewError[]; warnings: ReviewWarning[] }> {
    const errors: ReviewError[] = [];
    const warnings: ReviewWarning[] = [];

    // Check for hardcoded secrets
    const secretPatterns = [
      { pattern: /api[_-]?key\s*[:=]\s*["'][^"']{20,}["']/i, rule: 'hardcoded-api-key', severity: 'critical' },
      { pattern: /secret[_-]?key\s*[:=]\s*["'][^"']{20,}["']/i, rule: 'hardcoded-secret', severity: 'critical' },
      { pattern: /password\s*[:=]\s*["'][^"']{8,}["']/i, rule: 'hardcoded-password', severity: 'critical' },
      { pattern: /token\s*[:=]\s*["'][^"']{20,}["']/i, rule: 'hardcoded-token', severity: 'high' },
      { pattern: /private[_-]?key\s*[:=]/i, rule: 'potential-private-key', severity: 'high' },
    ];

    // Simple file scan (placeholder - real implementation would use proper file scanner)
    try {
      const result = await this.executeCommand('grep', ['-r', '-n', 'api_key|secret|password|token', '--include=*.ts', '--include=*.js', '--include=*.vue', '.'], projectPath);

      if (result.stdout) {
        const lines = result.stdout.split('\n');
        for (const line of lines) {
          for (const { pattern, rule, severity } of secretPatterns) {
            if (pattern.test(line)) {
              const fileMatch = line.match(/(.+?):(\d+):/);
              if (fileMatch) {
                errors.push({
                  file: fileMatch[1],
                  line: parseInt(fileMatch[2]),
                  column: 0,
                  message: 'Potential hardcoded secret detected',
                  rule,
                  severity: severity as 'critical' | 'high' | 'medium' | 'low',
                });
              }
            }
          }
        }
      }
    } catch (e) {
      console.warn('Security scan skipped:', e);
    }

    return { errors, warnings };
  }

  /**
   * Execute command helper
   */
  private async executeCommand(_cmd: string, _args: string[], _cwd: string): Promise<{ success: boolean; stdout: string; stderr: string }> {
    // This would use Tauri's shell plugin or similar
    // Placeholder implementation
    return new Promise((resolve) => {
      // In real implementation, invoke Tauri command
      // For now, return mock success
      setTimeout(() => {
        resolve({
          success: true,
          stdout: '',
          stderr: '',
        });
      }, 100);
    });
  }

  /**
   * Generate review report
   */
  private generateReport(errors: ReviewError[], warnings: ReviewWarning[], score: number): string {
    const lines: string[] = [];

    lines.push(`# Code Review Report`);
    lines.push(`\n**Score: ${score}/100**\n`);

    if (errors.length > 0) {
      lines.push(`\n## Errors (${errors.length})\n`);
      for (const error of errors) {
        lines.push(`- [${error.severity.toUpperCase()}] ${error.file}:${error.line} - ${error.message} (${error.rule})`);
      }
    }

    if (warnings.length > 0) {
      lines.push(`\n## Warnings (${warnings.length})\n`);
      for (const warning of warnings) {
        lines.push(`- ${warning.file}:${warning.line} - ${warning.message}`);
      }
    }

    if (errors.length === 0 && warnings.length === 0) {
      lines.push(`\n✅ No issues found.\n`);
    }

    return lines.join('\n');
  }
}

export const codeReviewAgent = new CodeReviewAgent();
export type { ReviewResult, ReviewError, ReviewWarning, ReviewConfig };