---
name: testing
category: development
description: Test generation and execution
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["generate", "run", "coverage", "fix"]
    required: true
  target:
    type: string
    required: true
    description: File or module to test
  framework:
    type: string
    required: false
    default: "vitest"
---

# Testing

## Steps
1. Analyze target code for testable functions
2. Generate test cases covering happy path, edge cases, errors
3. Run tests with coverage reporting
4. For failures: diagnose and suggest fixes
5. Return results with pass/fail counts and coverage

## Tools Used
- execute_code, terminal, read_file
