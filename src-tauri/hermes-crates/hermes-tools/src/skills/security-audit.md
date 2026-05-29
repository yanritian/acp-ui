---
name: security-audit
category: development
description: Security scanning and vulnerability detection
version: 1.0.0
parameters:
  target:
    type: string
    required: true
    description: File or URL to scan
  scan_type:
    type: enum
    values: ["osv", "url_safety", "code_audit", "full"]
    required: false
    default: "full"
---

# Security Audit

## Steps
1. Identify scan target and type
2. Run appropriate security checks (OSV, URL safety, code audit)
3. Collect findings with severity levels
4. Generate report with remediation suggestions
5. Log audit results

## Tools Used
- osv_check, url_safety
