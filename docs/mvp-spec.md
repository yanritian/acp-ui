# MVP Agent Specification: ACP-UI

## 1. Objective

ACP-UI is a multi-platform Agent Client Protocol (ACP) client that enables:
- **Primary User**: Developers and AI-assisted coding teams
- **Job-to-be-done**: Execute coding tasks safely through AI agents with full audit trails, budget control, and approval gates

**Success Criteria**:
- Agent completes task within budget
- All mutations have approval records
- No secrets leaked to context
- Full execution trace available

---

## 2. MVP Scope and Assumptions

### Included in MVP
- Single-agent harness with model-tool-observation loop
- Budget control (steps, tokens, time, cost)
- Permission matrix (Level 0-4 autonomy)
- Planning mode with mutation blocking
- Context compaction with state preservation
- Structured tool results
- Secrets sanitization
- Basic Evals (prompt injection, approval bypass)

### Explicit Assumptions
- Agent operates within a workspace directory
- External sends require approval
- Destructive actions denied by default
- Hermes integration for advanced agent capabilities

### Non-goals (Deferred)
- Multi-agent orchestration (Phase 2)
- Sandbox execution (Phase 2)
- Flutter feature parity (Phase 3)
- Production deployment automation

---

## 3. Autonomy and Risk Level

### Default: Level 2 (Approval-gated)

| Level | Name | Description | Use Case |
|-------|------|-------------|----------|
| 0 | Answer-only | Agent reads and answers only | Code review agents |
| 1 | Draft-only | Agent drafts, humans commit | Planning assistants |
| 2 | Approval-gated | Approval for destructive/external | Default for coding agents |
| 3 | Policy-bounded | Budget-based gates only | Trusted automation |
| 4 | Autonomous | Full autonomy with logging | Admin agents |

### Risk Classes
- `read_public_data`: Safe
- `read_workspace_data`: Allowed within cwd
- `write_workspace`: Approval required
- `external_send`: Approval required
- `destructive_action`: Denied with recovery path
- `shell_execution`: Sandbox required (Phase 2)

---

## 4. Core Agentic Loop

```typescript
// Simplified loop (see acp-session-runner.ts for full implementation)
for (step < maxSteps) {
  context = buildContext(session)

  if (context.needsCompaction) {
    session = compact(session)
    rehydrate(session, { activePlan, approvalState, todoList })
  }

  output = model.generate(context, tools)

  if (output.finalAnswer) return finalize(output)

  for (call of output.toolCalls) {
    tool = registry.get(call.name)
    args = validate(tool, call.args)
    decision = permissions.evaluate(tool, args)

    if (decision.denied) result = deniedResult(decision.reason)
    else if (decision.approvalRequired) return pauseForApproval(call)
    else result = execute(tool, args)

    result = limitResult(result, maxChars)
    session.addResult(call.id, result)
  }
}

return stop("Budget exceeded", session)
```

---

## 5. Context and Instruction Architecture

### Trust Levels
| Section Type | Trust Level | Examples |
|--------------|-------------|----------|
| Trusted | `trusted` | system_instructions, developer_instructions |
| Semi-trusted | `semi_trusted` | user_input, agent_output |
| Untrusted | `untrusted` | retrieved_content, tool_results |

### Cache-aware Ordering
1. Stable system/developer instructions
2. Domain policy (permission matrix)
3. Tool schemas (deterministic order)
4. Retrieved context
5. Recent tool results
6. Current user request (volatile)

### Secrets Handling
- Sanitized before adding to context
- Patterns: API keys, passwords, tokens, private keys, JWT
- Redaction logged in metadata

---

## 6. Tool Registry (Minimal Set)

| Tool | Risk Class | Permission | Timeout | Max Result |
|------|------------|------------|---------|------------|
| `read_file` | read_workspace_data | allow_within_cwd | 10s | 8000 chars |
| `read_directory` | read_workspace_data | allow_within_cwd | 5s | 4000 chars |
| `write_file` | write_workspace | approval_required | 15s | 1000 chars |
| `edit_file` | write_workspace | approval_required | 10s | 500 chars |
| `bash_command` | shell_execution | sandbox_required | 60s | 10000 chars |
| `search_files` | read_workspace_data | allow_within_cwd | 10s | 4000 chars |
| `search_content` | read_workspace_data | allow_within_cwd | 15s | 6000 chars |
| `draft_message` | draft_output | allow | 5s | 5000 chars |
| `draft_plan` | draft_output | allow | 5s | 3000 chars |
| `send_message` | external_send | approval_required | 30s | 1000 chars |
| `delete_file` | destructive_action | deny_with_recovery | 10s | 500 chars |

---

## 7. Planning Behavior

### Triggers
- Task is ambiguous
- Risky side effects possible
- User preferences matter
- Multiple strategies available

### Planning Mode Rules
- **Allowed**: read, search, inspect, ask, draft plan
- **Blocked**: write, delete, send, bash, external

### Plan Artifact Fields
```json
{
  "objective": "...",
  "scope": "...",
  "risks": ["..."],
  "steps": ["..."],
  "approval_points": ["..."],
  "validation_method": "...",
  "rollback_path": "...",
  "done_condition": "..."
}
```

---

## 8. Goal-like Loop Behavior

### Criteria for Goal Mode
- Single coherent objective
- Measurable done condition
- Explicit budget
- Validation method exists

### Stop Rules
- Done condition met
- Budget reached
- Approval required
- Risk exceeds policy
- Source data missing
- User changes objective

---

## 9. Context, Memory, and Auto-compaction

### Compaction Trigger
- Before context limit (proactive, not reactive)
- When `contextUsage > 80%`

### Rehydration Artifacts
- `activePlan`
- `goalState`
- `approvalState`
- `todoList`
- `loadedScopedInstructions`

### Summary Format
```text
Current objective: ...
User constraints: ...
Active plan: ...
Actions taken: ...
Decisions made: ...
Pending tasks: ...
Next recommended step: ...
Do not redo: ...
```

---

## 10. Skills and Connectors

### Skills (Progressive Disclosure)
- Skill index shown first
- Full instructions loaded on selection
- Permission bounded per skill

### MCP/External Connectors
- Namespaced by source
- Scoped credentials
- Risk class mapping
- Approval-gated for risky calls

---

## 11. Safety and Approval Policy

### Prompt Injection Handling
- Retrieved content marked `untrusted`
- No execution from untrusted sources
- Patterns detected: markdown injection, context escape

### Approval Flow
- Three-state: Approved / Denied / RequiresConfirmation
- Model cannot self-approve
- Approval records persisted

### Secrets
- Never in context
- Sanitized at context boundary
- 20+ patterns detected

---

## 12. Observability and Evals

### Trace Events
- `run_id`, `model_version`, `tools_exposed`
- `tool_calls`, `permission_decisions`
- `approval_requests`, `compaction_events`
- `cost`, `latency`, `tokens`

### Evals Coverage
| Category | Test Cases |
|----------|------------|
| Prompt injection | Injection patterns, retrieved content injection |
| Approval bypass | Permission gate requirements, self-approval prevention |
| Context overflow | Compaction trigger, state preservation |
| Budget control | Step/token/time/cost limits |

---

## 13. Minimal Implementation Path

### Phase 1 (Current - MVP)
1. ✅ Typed event/session state
2. ✅ Context builder with stable prefix
3. ✅ Model call wrapper (ACP SDK)
4. ✅ Typed tool registry
5. ✅ Local schema validation
6. ✅ Permission engine (Level 0-4)
7. ✅ Structured tool results
8. ✅ Manual loop with budgets
9. ✅ Tracing (telemetry)
10. ✅ Planning mode mutation blocking
11. ✅ Secrets sanitization
12. ✅ Context compaction

### Phase 2 (Next)
1. 🔲 Sandbox execution
2. 🔲 Hermes Dashboard real data
3. 🔲 Tool misuse tests
4. 🔲 Flutter feature sync

---

## 14. First Release Checklist

- [x] Agent has one primary job-to-be-done
- [x] Autonomy level explicit (Level 0-4)
- [x] High-risk actions approval-gated
- [x] Every tool has schema, risk class, timeout, output limit
- [x] Every tool result structured
- [x] Loop has step, token, time, cost budgets
- [x] Context builder separates trusted/untrusted
- [x] Prompt prefix stable for caching
- [x] Plans and approvals stored outside prompt
- [x] Auto-compaction preserves active state
- [x] Secrets not visible to model
- [x] Traces available for every run
- [x] Evals cover injection, bypass, overflow
- [ ] Rollout starts with monitored users

---

## 15. Known Gaps

| Gap | Priority | Target Phase |
|-----|----------|--------------|
| Sandbox execution not implemented | P1 | Phase 2 |
| Hermes Dashboard uses mock data | P1 | Phase 2 |
| Tool misuse tests missing | P1 | Phase 2 |
| Flutter feature parity | P2 | Phase 3 |
| Multi-agent orchestration | P2 | Phase 3 |

---

*Generated: 2026-06-04*
*Standard: MVP Agent Blueprint (agents-best-practices skill)*