# PRD Quality Review: acp-swarm MVP

**Document:** `_bmad-output/planning-artifacts/prds/prd-acp-swarm-2026-06-09/prd.md`
**Reviewer:** Automated Quality Gate
**Date:** 2026-06-09

---

## Overall Verdict

**Adequate.** This PRD is well-structured, honest about scope, and provides sufficient detail for implementation in most areas. The main gaps are: (1) ambiguous done-ness for Queen election and Reconcile Loop feedback, (2) missing data model and error catalog for downstream workflows, and (3) Dashboard and Token Budget sections that feel structurally inflated. The PRD can proceed to implementation with targeted clarification on the three High-severity findings.

---

## 1. Decision-readiness — ADEQUATE

The PRD provides sufficient information for a decision-maker to understand the product thesis and make go/no-go decisions. The core value proposition is clear: "Goal-Driven 模型" as the differentiator against Codex Ultracode and Claude Dynamic Workflows (closed-source competitors).

**Strengths:**
- Clear thesis statement at Vision (§1): acp-swarm as "Kubernetes for Agents, Browser for AI"
- Explicit non-goals (§5) demarcate scope boundaries
- Success metrics (§7) provide measurable targets: 1000 GitHub stars, 80% goal convergence rate

**Weaknesses:**
- **Critical gap**: No cost/benefit analysis for the 28 functional requirements. FR-18 through FR-20 (Queen election) add complexity—what's the cost if we defer this to v2?
- Trade-offs under-specified: §8 Open Questions lists 7 items, but only 2 have explicit owners and revisit timelines.
- Investment sizing absent: No engineering effort estimates (story points, weeks, FTEs) for any FR.

### Findings
- **medium** Open Questions #2 and #6 need resolution before v1 commit (§8) — Worker failure strategy and ACP protocol versioning are decision-blocking. *Fix:* Add resolution owners and deadlines.
- **medium** Add rough effort estimates (T-shirt sizing) to each feature section (§4) — A decision-maker cannot assess ROI. *Fix:* Add S/M/L/XL estimates per feature.
- **low** Success metric SM-2 (50 independent worker instances) seems disconnected from target audience size (§7) — Where does 50 come from? *Fix:* Add justification or adjust target.

---

## 2. Substance over theater — STRONG

This PRD largely avoids empty posturing. Content is earned, not furniture.

**Earned content (good):**
- FR-1 through FR-5 (Goal-Driven Engine) provide concrete data structures, state machines, and algorithmic flows
- FR-11 `.goal` file schema includes actual YAML with required/optional fields
- Glossary (§3) is precise with 13 domain terms

**Theater detected (minor):**
- **Vision theater**: The analogy "Agent 的 Kubernetes，AI 的浏览器" appears only in tagline form. Never explains *how* acp-swarm achieves Kubernetes-style reconciliation vs. a simple retry loop.
- **Persona theater**: §2.1 "Alex，Agent 开发者，30 岁，全栈工程师" includes arbitrary age and job title that don't inform any FR.
- **NFR theater**: NFR-3 "Dashboard 支持 100 个 Worker 和 1000 个并发 Goal（不卡顿）" uses "不卡顿" which is subjective.

### Findings
- **low** Vision analogy needs 1-2 sentences explaining *why* the comparison holds (§1) — Not just "Kubernetes for Agents" but what specific K8s patterns apply. *Fix:* Add concrete K8s analogy (Reconcile Loop = K8s controller, Goal = K8s desired state).
- **low** Replace "不卡顿" with measurable latency target (NFR-3) — *Fix:* "Dashboard renders updated state within 500ms of event via WebSocket."
- **positive** Glossary and FR data structures are genuinely useful, not performative.

---

## 3. Strategic coherence — STRONG

The PRD has a clear thesis: **"Goal-Driven orchestration with heterogeneous agents is the open-source alternative to closed swarm systems."** All features serve this thesis.

**Coherent alignment:**
- FR-4 (Reconcile Loop) directly implements the Goal-Driven thesis
- FR-6 through FR-10 (Worker Protocol) enable the "heterogeneous agents" promise
- §5 Non-Goals explicitly excludes single-agent use cases

**Minor incoherence:**
- FR-21 through FR-24 (Dashboard) feel like feature bloat. The thesis is "orchestration engine," not "monitoring platform."
- FR-18 through FR-20 (Queen election) add significant complexity for fault tolerance, which is orthogonal to the Goal-Driven thesis.

### Findings
- **medium** Justify Dashboard as core feature vs. optional demo tool (§4.7) — Reference to SM-1 is thin. *Fix:* Add paragraph explaining Dashboard is critical for open-source adoption (demo-ability, developer experience).
- **medium** Explain why Queen election is v1-critical vs. fixed-leader simplification (§4.6) — What breaks without it? *Fix:* Add risk analysis: "Without Queen election, single point of failure when Queen Worker crashes."
- **positive** All 28 FRs trace to UJ-1, UJ-2, or UJ-3.

---

## 4. Done-ness clarity — ADEQUATE

Most FRs provide sufficient detail for an engineer to know "done." Some FRs have ambiguous acceptance criteria.

**Clear done-ness (good):**
- FR-1: Goal struct fields enumerated; `is_terminal()` method specified
- FR-2: 8 CompletionCondition types with field names
- FR-6 through FR-10: HTTP endpoints with request/response bodies
- FR-27/28: Token budget thresholds specified (50%, 80%, 100%)

**Ambiguous done-ness (problems):**
- FR-4 Reconcile Loop: Step 4 says "追加反馈到 Goal 描述末尾" but doesn't specify feedback format (free-form text? structured JSON?).
- FR-18/19/20 Queen Lease: No Queen-specific API endpoints defined. FR-6 through FR-10 define Worker Protocol but Queen has extra responsibilities.
- FR-25 Events: Lists 9 event types but doesn't specify event schema (JSON? Protobuf? What fields?).

### Findings
- **high** FR-4 needs feedback format specification (§4.1) — Is feedback free-form text or structured JSON with fields like evaluator_id, condition_results, suggestions? *Fix:* Add feedback schema example.
- **high** FR-18/19/20 need Queen-specific API endpoints or explicit Worker Protocol reuse (§4.6) — How does Queen renew lease? How is election result broadcast? *Fix:* Add 2-3 Queen-specific endpoints or clarify reuse.
- **medium** FR-25 needs event schema definition (§4.8) — What fields does each event have? What serialization format? *Fix:* Add event schema table.
- **low** FR-16 Star topology says "Queen 必须是 `claude_code` 类型" but doesn't define fallback behavior — What if no claude_code worker registered? *Fix:* Add error handling: "Return error 'No eligible Queen worker'."

---

## 5. Scope honesty — STRONG

The PRD is commendably honest about scope boundaries and assumptions.

**Explicit omissions (good):**
- §5 Non-Goals lists 10 items that v1 explicitly will NOT do
- §6.2 Out of Scope for MVP lists 8 items with clear "留给 v2+" annotations
- [ASSUMPTION] tags appear throughout and are indexed in §9

**Minor gaps:**
- §2.3 Non-Users lists "追求生产就绪的企业用户" as non-user, but NFR-4/5/6 hint at reliability requirements. Are these for demo robustness or production?
- FR-3 `Failed(String)` state has no recovery semantics. Terminal? Retriable?

### Findings
- **low** Clarify whether NFR-4/5/6 are for demo robustness or production aspirations (§4 NFRs) — *Fix:* Add note: "These NFRs ensure demo stability; production-grade reliability deferred to v2."
- **low** FR-3 `Failed` state needs recovery semantics (§4.1) — Terminal or retriable? *Fix:* Add: "Failed is terminal. To retry, create new Goal with same description."
- **positive** 7 [ASSUMPTION] tags properly indexed; non-goals clearly enumerated.

---

## 6. Downstream usability — ADEQUATE

The PRD provides sufficient structure for UX/architecture/story workflows, but some gaps exist.

**Good downstream extraction:**
- Glossary (§3) provides consistent terminology
- FRs globally numbered (FR-1 to FR-28) with "Consequences" subsections
- User Journeys (§2.4) have numbered steps—convertible to test scenarios
- API Contracts (§10) provides endpoint list

**Gaps for downstream:**
- No wireframes or UI specs for Dashboard (FR-21 to FR-24)
- No data model diagram showing Goal, Worker, Lease, Event relationships
- No error catalog (HTTP status codes, error response schema)
- No sequence diagram for Reconcile Loop (FR-4)

### Findings
- **high** Add data model diagram (ERD or equivalent) showing entity relationships — Architecture team needs this to design database schema and API responses. *Fix:* Add Mermaid ERD or link to separate architecture doc.
- **high** Add error catalog to API Contracts (§10) — Implementation needs HTTP status codes and error response schema. *Fix:* Add table: endpoint | error code | error body | HTTP status.
- **medium** Add wireframe sketches for Dashboard (FR-21 to FR-24) — UX team needs visual reference. *Fix:* Add ASCII wireframes or link to Figma.
- **medium** Add sequence diagram for Reconcile Loop (FR-4) — Critical for both arch and UX. *Fix:* Add Mermaid sequence diagram.

---

## 7. Shape fit — ADEQUATE

The PRD shape mostly matches the product, but some structural choices feel forced.

**Good fit:**
- Goal-Driven model (FR-1 to FR-5) placed first as core
- Worker Protocol (FR-6 to FR-10) naturally follows
- Topology engine (FR-16, FR-17) fits swarm metaphor

**Questionable shape choices:**
- Dashboard as top-level feature (§4.7) equal weight with Worker Protocol inflates its importance
- Token Budget as separate section (§4.9) could fold into Goal-Driven Engine
- Queen election as top-level (§4.6) treats it as equal to Worker Protocol

### Findings
- **medium** Consider restructuring feature sections (§4) — Current: 9 flat sections. Better: (1) Core Engine (FR-1 to FR-5), (2) Integration (FR-6 to FR-15), (3) Coordination (FR-16 to FR-20), (4) Observability (FR-21 to FR-28). *Fix:* Reorganize with clear hierarchy.
- **low** Merge §4.9 Token Budget into §4.1 Goal-Driven Engine — Token budget is a property of Goal, not standalone feature. *Fix:* Move FR-27/28 under FR-1.
- **low** Clarify whether Dashboard is MVP-critical or demo-optional (§4.7) — *Fix:* Add note: "Dashboard is MVP-critical for open-source demo-ability; deferred to v1.1 if scope pressure."

---

## Mechanical Notes

### Glossary Drift
- "Worker" vs "Agent": Glossary defines Worker, but PRD occasionally uses "Agent" interchangeably (§1). Acceptable but consistency would be better.
- "Queen" vs "Queen Worker": Glossary defines Queen as "特殊角色的 Worker", but FR-16 uses "Queen (claude-code-1)" without clarifying. Minor.
- "Goal" vs "Task": Consistent throughout. Good.

### ID Continuity
- FRs globally numbered FR-1 to FR-28: consistent.
- NFRs NFR-1 to NFR-12: consistent.
- Success Metrics SM-1 to SM-6, plus SM-C1, SM-C2: consistent.
- User Journeys UJ-1, UJ-2, UJ-3: consistent.

### Cross-references
- FR-1 → FR-4: good
- FR-5 → RFC-001 §2.2.2: good
- FR-11 → RFC-001 §2.2.2/§2.2.3: good
- §9 Assumptions Index → FR numbers: good
- **Broken**: §4.1 FR-5 [NOTE FOR PM] duplicates §8 Open Questions #1 without cross-reference.
- **Inconsistent**: UJ-2 mentions `docs/worker-sdk-python.md` but §4.4 says Python SDK is v2+.

---

## Findings Summary

| Severity | Dimension | Finding |
|----------|-----------|---------|
| **high** | Done-ness | FR-4 feedback format unspecified (free-form vs. structured) |
| **high** | Done-ness | FR-18/19/20 need Queen-specific API endpoints |
| **high** | Downstream | Add data model diagram (ERD) |
| **high** | Downstream | Add error catalog to API Contracts |
| **medium** | Decision | Open Questions #2, #6 need resolution paths |
| **medium** | Decision | Add rough effort estimates (T-shirt sizing) |
| **medium** | Coherence | Justify Dashboard as core vs. demo tool |
| **medium** | Coherence | Explain why Queen election is v1-critical |
| **medium** | Downstream | Add wireframe sketches for Dashboard |
| **medium** | Downstream | Add sequence diagram for Reconcile Loop |
| **medium** | Shape | Restructure feature sections hierarchically |
| **low** | Decision | SM-2 (50 worker instances) lacks justification |
| **low** | Substance | Vision analogy needs concrete K8s comparison |
| **low** | Substance | Replace "不卡顿" with latency target |
| **low** | Scope | Clarify NFR-4/5/6 purpose (demo vs. production) |
| **low** | Scope | FR-3 `Failed` state needs recovery semantics |
| **low** | Shape | Merge §4.9 Token Budget into §4.1 |
| **low** | Mechanical | Fix UJ-2 reference to Python SDK (contradicts §4.4) |
