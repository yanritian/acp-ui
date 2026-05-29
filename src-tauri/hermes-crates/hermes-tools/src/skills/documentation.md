---
name: documentation
category: development
description: Documentation generation and maintenance
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["generate", "update", "translate", "summarize"]
    required: true
  target:
    type: string
    required: true
    description: Source code or existing doc to work from
  format:
    type: enum
    values: ["markdown", "jsdoc", "openapi", "readme"]
    required: false
    default: "markdown"
---

# Documentation

## Steps
1. Analyze source code or existing documentation
2. Extract key information (functions, types, APIs)
3. Generate documentation in requested format
4. For update: diff against existing and apply changes
5. Return generated content with metadata

## Tools Used
- read_file, write_file, search_files
