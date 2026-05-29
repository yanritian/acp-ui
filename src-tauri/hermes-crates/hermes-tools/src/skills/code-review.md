---
name: code-review
category: development
description: Code review and quality analysis
version: 1.0.0
parameters:
  target:
    type: string
    required: true
    description: File path or diff to review
  focus:
    type: enum
    values: ["all", "security", "performance", "style", "bugs"]
    required: false
    default: "all"
---

# Code Review

## Steps
1. Load target code or diff
2. Parse AST and identify code structures
3. Run checks based on focus area
4. Generate findings with line numbers and suggestions
5. Summarize with overall quality score

## Tools Used
- read_file, search_files
