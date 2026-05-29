---
name: file-operations
category: operational
description: Read, write, search, and patch files
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["read", "write", "search", "patch", "delete"]
    required: true
  path:
    type: string
    required: true
    description: File or directory path
  content:
    type: string
    required: false
    description: Content for write/patch operations
---

# File Operations

## Steps
1. Validate path is within allowed working directory
2. Perform requested operation (read/write/search/patch/delete)
3. Return result with file metadata
4. Log operation for audit trail

## Tools Used
- read_file, write_file, search_files, patch
