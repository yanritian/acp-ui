---
name: skill-management
category: management
description: Skill CRUD, version management, and evolution
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["list", "view", "create", "update", "delete", "version_list", "rollback", "evolve"]
    required: true
  name:
    type: string
    required: false
---

# Skill Management

## Steps
1. Identify requested operation from action parameter
2. Route to appropriate handler (CRUD or version operation)
3. For evolution: analyze execution stats and apply improvements
4. Return result with metadata

## Tools Used
- skills_list, skill_view, skill_manage
- invoke_skill, create_skill
