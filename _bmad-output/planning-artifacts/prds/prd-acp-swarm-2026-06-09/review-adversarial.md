# Adversarial Review: acp-swarm PRD

## Overall Verdict

The PRD is ambitious and well-structured, but suffers from strategic overreach for v1 MVP. The "Agent's Kubernetes" positioning is compelling but the gap between vision and v1 implementation is larger than acknowledged. Several hidden complexities (Reconcile Loop convergence, Queen decomposition quality, Worker failure handling) are underestimated, and success metrics are vulnerable to gaming. Needs sharper scope cuts and more realistic assumptions.

---

## Critical Findings

### C-1: "Domain-agnostic" vs coding-only MVP (§5, §2.4)
Vision claims universal applicability ("Agent 的 Kubernetes"), but Non-Goals explicitly states "v1 only supports coding domain." Creates cognitive mismatch.
**Fix:** Position v1 explicitly as "coding swarm orchestrator" with v2+ expanding domains.

### C-2: "Heterogeneous swarm" vs Queen-as-Claude-Code requirement (FR-16, FR-19)
Claims "any Agent can join" but mandates Queen must be `claude_code` type. Users without Claude Code cannot use Star topology.
**Fix:** Add fallback Queen mechanism or document as hard dependency.

### C-3: Reconcile Loop convergence not guaranteed (FR-4)
Loop may oscillate indefinitely (fix errors → introduce warnings → fix warnings → reintroduce errors). `max_iterations` accepts failure rather than solving convergence.
**Fix:** Add oscillation detection: "If same feedback appears N consecutive iterations, trigger Queen intervention."

### C-4: Queen Goal decomposition quality uncontrolled
No validation for decomposition correctness. Failure modes: overlapping file modifications, missing dependencies, wrong Worker assignment, unachievable CompletionCondition.
**Fix:** Add Decomposition Validator (file overlap check, skill matching, condition dry-run).

### C-5: No debugging journey for Worker developer (§2.4)
UJ-2 assumes everything works smoothly. Missing: registration failure, heartbeat missed, Goal parse error, convergence oscillation debugging.
**Fix:** Add UJ-2b: "Alex debugs a failing Worker" with iteration log inspection.

### C-6: SQLite persistence is v2 migration trap (FR-6, NFR-7)
Architecture designed for distributed operation but SQLite chosen for v1. Migration to PostgreSQL requires schema redesign, query rewrite, data migration.
**Fix:** Design `EventStore` trait abstraction layer allowing backend swap.

---

## High Findings

### H-1: CompletionCondition environment underspecified (FR-2)
Missing: required env vars, path fallback, timeout per condition type, retry policy for transient failures.

### H-2: Closed-source swarm threat minimized (§8)
If OpenAI/Anthropic open-source their swarm layer, acp-swarm loses differentiation. Need competitive response strategy.

### H-3: No Goal definition failure journey (UJ-1, UJ-3)
Users will make YAML syntax errors, reference wrong Worker IDs, create circular dependencies. Missing error recovery journey.

### H-4: GitHub Stars trivially gameable (SM-1)
Replace with: unique Worker registrations, users who completed Goal, users who created custom Workers.

### H-5: 80% convergence achievable via trivial Goals (SM-3)
Add stratification: overall 80%, >3 iterations 50%, >5 iterations 30%.

### H-6: Token self-report trust flaw (FR-27)
Worker can under-report tokens. Add verification: intercept API responses, parse CLI output for token markers.

### H-7: Queen Lease TTL=30s too aggressive (FR-18)
Real-world latency spikes, GC pauses can trigger mid-task expiration. Increase to 60-120s or add "busy Queen" mode.

---

## Summary

| Severity | Count |
|----------|-------|
| Critical | 6 |
| High | 7 |
| Medium | 6 |
| Low | 1 |

**Top 3 priority fixes:**
1. Resolve domain-agnostic vs coding-only contradiction
2. Add oscillation detection to Reconcile Loop
3. Create Worker debugging journey
