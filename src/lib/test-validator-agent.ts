/**
 * Test Validator Agent - executes tests and validates coverage
 */

interface TestResult {
  success: boolean;
  passed: number;
  failed: number;
  skipped: number;
  total: number;
  coverage: number;
  duration: number;
  errors: TestError[];
  report: string;
}

interface TestError {
  testName: string;
  file: string;
  message: string;
  stack?: string;
}

interface CoverageInfo {
  lines: { covered: number; total: number };
  functions: { covered: number; total: number };
  branches: { covered: number; total: number };
  statements: { covered: number; total: number };
}

interface TestConfig {
  minCoverage: number;
  timeoutMs: number;
  retryAttempts: number;
  testCommand: string;
  coverageCommand: string;
}

class TestValidatorAgent {
  private config: TestConfig;

  constructor(config?: Partial<TestConfig>) {
    this.config = {
      minCoverage: 80,
      timeoutMs: 60000,
      retryAttempts: 3,
      testCommand: 'npm test',
      coverageCommand: 'npm run test:coverage',
      ...config,
    };
  }

  /**
   * Run tests and validate
   */
  async validate(projectPath: string): Promise<TestResult> {
    // Run tests
    const testResult = await this.runTests(projectPath);

    // Run coverage if tests passed
    if (testResult.success) {
      const coverageResult = await this.runCoverage(projectPath);
      testResult.coverage = coverageResult.lines ?
        (coverageResult.lines.covered / coverageResult.lines.total) * 100 : 0;

      // Check coverage threshold
      if (testResult.coverage < this.config.minCoverage) {
        testResult.success = false;
        testResult.errors.push({
          testName: 'Coverage Check',
          file: '',
          message: `Coverage ${testResult.coverage.toFixed(1)}% is below minimum ${this.config.minCoverage}%`,
        });
      }
    }

    // Generate report
    testResult.report = this.generateReport(testResult);

    return testResult;
  }

  /**
   * Run tests with retry
   */
  private async runTests(projectPath: string): Promise<TestResult> {
    const result: TestResult = {
      success: false,
      passed: 0,
      failed: 0,
      skipped: 0,
      total: 0,
      coverage: 0,
      duration: 0,
      errors: [],
      report: '',
    };

    for (let attempt = 0; attempt < this.config.retryAttempts; attempt++) {
      console.log(`Running tests (attempt ${attempt + 1})...`);

      const startTime = Date.now();
      const testOutput = await this.executeCommand(projectPath, this.config.testCommand);
      result.duration = Date.now() - startTime;

      // Parse test output
      const parsed = this.parseTestOutput(testOutput);
      result.passed = parsed.passed;
      result.failed = parsed.failed;
      result.skipped = parsed.skipped;
      result.total = parsed.total;
      result.errors = parsed.errors;

      // Check success
      result.success = result.failed === 0 && result.errors.length === 0;

      if (result.success) {
        console.log('Tests passed!');
        break;
      }

      console.log(`Tests failed (attempt ${attempt + 1}/${this.config.retryAttempts})`);

      // Wait before retry
      if (attempt < this.config.retryAttempts - 1) {
        await this.delay(2000);
      }
    }

    return result;
  }

  /**
   * Run coverage
   */
  private async runCoverage(projectPath: string): Promise<CoverageInfo> {
    const coverageOutput = await this.executeCommand(projectPath, this.config.coverageCommand);
    return this.parseCoverageOutput(coverageOutput);
  }

  /**
   * Parse test output (Jest/Vitest format)
   */
  private parseTestOutput(output: string): { passed: number; failed: number; skipped: number; total: number; errors: TestError[] } {
    const errors: TestError[] = [];

    // Parse Jest output
    const summaryMatch = output.match(/Tests:\s+(\d+) passed,?\s*(\d+) failed,?\s*(\d+) skipped,?\s*(\d+) total/);
    if (summaryMatch) {
      const passed = parseInt(summaryMatch[1]);
      const failed = parseInt(summaryMatch[2]);
      const skipped = parseInt(summaryMatch[3]);
      const total = parseInt(summaryMatch[4]);

      // Extract failure details
      const failMatches = output.matchAll(/FAIL\s+(.+)/g);
      for (const match of failMatches) {
        const file = match[1];
        // Extract test name and error
        const errorMatch = output.match(/●\s+(.+?)\n\s+(.+?)/);
        if (errorMatch) {
          errors.push({
            testName: errorMatch[1],
            file,
            message: errorMatch[2],
          });
        }
      }

      return { passed, failed, skipped, total, errors };
    }

    // Parse Vitest output
    const vitestMatch = output.match(/(\d+)\s+passed.*?(\d+)\s+failed.*?(\d+)\s+skipped/);
    if (vitestMatch) {
      return {
        passed: parseInt(vitestMatch[1]),
        failed: parseInt(vitestMatch[2]),
        skipped: parseInt(vitestMatch[3]),
        total: parseInt(vitestMatch[1]) + parseInt(vitestMatch[2]) + parseInt(vitestMatch[3]),
        errors,
      };
    }

    // Default: assume success if no obvious failures
    const hasFail = output.toLowerCase().includes('fail') || output.toLowerCase().includes('error');
    return {
      passed: hasFail ? 0 : 1,
      failed: hasFail ? 1 : 0,
      skipped: 0,
      total: 1,
      errors: hasFail ? [{ testName: 'Unknown', file: '', message: output }] : [],
    };
  }

  /**
   * Parse coverage output
   */
  private parseCoverageOutput(output: string): CoverageInfo {
    // Parse coverage summary
    const linesMatch = output.match(/All files\s*\|\s*(\d+\.?\d*)\s*\|\s*(\d+\.?\d*)/);
    const functionsMatch = output.match(/All files.*?\|\s*\d+\.?\d*\s*\|\s*(\d+\.?\d*)\s*\|\s*(\d+\.?\d*)/);

    // Simplified parsing - real implementation would parse full coverage report
    return {
      lines: linesMatch ? {
        covered: parseFloat(linesMatch[1]),
        total: parseFloat(linesMatch[2]),
      } : { covered: 0, total: 100 },
      functions: functionsMatch ? {
        covered: parseFloat(functionsMatch[1]),
        total: parseFloat(functionsMatch[2]),
      } : { covered: 0, total: 100 },
      branches: { covered: 0, total: 100 },
      statements: { covered: 0, total: 100 },
    };
  }

  /**
   * Execute command
   */
  private async executeCommand(_projectPath: string, _command: string): Promise<string> {
    // TODO(Phase 4): use Tauri shell
    return '';
  }

  /**
   * Delay helper
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Generate test report
   */
  private generateReport(result: TestResult): string {
    const lines: string[] = [];

    lines.push('# Test Validation Report');
    lines.push(`\n## Summary`);
    lines.push(`- **Status**: ${result.success ? '✅ PASSED' : '❌ FAILED'}`);
    lines.push(`- **Passed**: ${result.passed}`);
    lines.push(`- **Failed**: ${result.failed}`);
    lines.push(`- **Skipped**: ${result.skipped}`);
    lines.push(`- **Total**: ${result.total}`);
    lines.push(`- **Coverage**: ${result.coverage.toFixed(1)}%`);
    lines.push(`- **Duration**: ${(result.duration / 1000).toFixed(2)}s`);

    if (result.errors.length > 0) {
      lines.push(`\n## Failures`);
      for (const error of result.errors) {
        lines.push(`\n### ${error.testName}`);
        lines.push(`- File: ${error.file}`);
        lines.push(`- Error: ${error.message}`);
        if (error.stack) {
          lines.push(`\nStack trace:`);
          lines.push(error.stack);
        }
      }
    }

    if (result.coverage < this.config.minCoverage) {
      lines.push(`\n## Coverage Warning`);
      lines.push(`Coverage ${result.coverage.toFixed(1)}% is below threshold ${this.config.minCoverage}%`);
    }

    return lines.join('\n');
  }
}

export const testValidatorAgent = new TestValidatorAgent();
export type { TestResult, TestError, CoverageInfo, TestConfig };