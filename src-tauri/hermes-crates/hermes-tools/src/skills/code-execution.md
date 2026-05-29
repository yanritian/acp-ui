---
name: code-execution
category: operational
description: Execute and debug code in multiple languages
version: 1.0.0
parameters:
  code:
    type: string
    required: true
    description: Code to execute
  language:
    type: string
    required: false
    default: "python"
    description: Programming language
  timeout:
    type: integer
    required: false
    default: 30
    description: Execution timeout in seconds
---

# Code Execution

## Description
Execute code snippets in a sandboxed environment with support for multiple languages.

## Steps
1. Validate the code and language parameter
2. Create a temporary execution environment
3. Run the code with timeout protection
4. Capture stdout, stderr, and exit code
5. Return structured results with output and any errors

## Tools Used
- execute_code
- terminal

## Output Format
Return: exit_code, stdout, stderr, duration_ms.
