# Open-Source Strategy Review: acp-swarm PRD

## Overall Verdict — Score: 6.5/10

Strong foundation, compelling differentiation, but needs scope discipline, documentation investment, and adoption-focused metrics to become a 10/10 open-source launch.

---

## Critical Findings

### C-1: No README Strategy (Missing)
PRD assumes developers clone and run `cargo run --example quickstart`, but no explicit README structure or "getting started in 5 minutes" guarantee. UJ-1 requires 30 minutes—6x too long for first impression.
**Fix:** Add README requirements: first 3 lines = what + why + one command, animated GIF, prereq check script, badges.

### C-2: MVP Scope Too Broad (Section 6.1)
28 functional requirements across 9 feature areas is NOT a wedge—it's a full product. Kubernetes v1 only did container orchestration.
**Fix:** Split into v0.1 (Sharp Wedge) and v1.0 (Full Product):
- **v0.1 (Weeks 1-4):** Worker Protocol + Goal-Driven Engine + Chain topology + CLI only.
- **v1.0 (Weeks 5-12):** Add Dashboard, Star topology, Queen election, metrics, token budget.

### C-3: No Contributor Journey (Missing)
Targets "开源社区贡献者" as reader but provides no mechanism for contribution. No FR defines "how to add a new Worker type" or "how to contribute a topology adapter."
**Fix:** Add Worker Adapter Template (`cargo generate`), Topology Adapter Interface, CONTRIBUTING.md, 5 "good first issue" labels.

### C-4: Documentation Scope Not Defined (Section 6.1)
Documentation listed as bullet point with no detail. Successful open-source projects spend 30-40% of effort on docs.
**Fix:** Add explicit FRs: README requirements, Getting Started Guide, ADRs, API auto-generation, Example Gallery (10 .goal files).

---

## High Findings

### H-1: Missing "One Killer Use Case" Hook (§1, §2.4)
"Let multiple agents collaborate on complex programming tasks" is too abstract. Need one viral use case: "Fix all TypeScript errors—Codex finds them, Claude Code fixes them, 5-minute demo."

### H-2: Prerequisites Not Addressed in Demo Flow (§2.4, UJ-1)
UJ-1 assumes "Codex CLI AND Claude Code CLI already configured." Major friction point. Most developers don't have both.
**Fix:** Add "One-Agent Demo Mode" — run with only Codex OR only Claude Code using single-Worker Chain topology.

### H-3: SDK Documentation Underestimated (§4.4, FR-14/15)
SDKs defined but no documentation requirements. "SDKs without great docs don't get adopted."
**Fix:** Each SDK must have: Quickstart (5 min), API reference, 3 working examples, all tested in CI.

### H-4: Worker Protocol Not Marketed as Extension Point (§4.2)
Presented as technical spec, not platform play. No Worker Registry concept, no Certified Workers badge.
**Fix:** Prepare protocol with versioning and stability guarantees. Plan Worker Registry and Badge System for v2.

### H-5: SM-1 (GitHub Stars) is Vanity Metric (§7)
Stars correlate poorly with adoption. Replace with "Weekly Active Swarms" — 100 unique repos running acp-swarm weekly.

### H-6: Missing Retention Metric (§7)
Measures acquisition (stars, workers, videos) but not retention. Do users come back?
**Fix:** Add SM-7: "30-Day Retention" — % of users who ran acp-swarm in week 4 after running in week 1. Target: 30%+.

---

## Medium Findings

### M-1: No "Wow Moment" Explicitly Defined (§2.4, UJ-1)
"Goal status from Iterating to Converged" is underwhelming. Add Live Diff Visualization and Error Count Ticker.

### M-2: Dashboard in MVP Increases Friction (§4.7)
Many developers prefer CLI-only demos. Make Dashboard optional for v0.1.

### M-3: No Demo Video Requirement (Missing)
Projects with demo videos get 2-3x more stars. Add FR: 60-second maintainer demo video.

### M-4: No Plugin/Extension Architecture (§5)
Limits contribution to forking. Add Topology Plugin Interface for v1.1.

### M-5: SM-2 Lacks Depth (§7)
Split into: Official Workers Active (30+) and Community Workers Registered (10+).

---

## What Would Make This a 10/10 Open-Source Launch

### Phase 1: Sharpen the Wedge (Weeks 1-2)
1. **Cut MVP scope in half.** Remove Dashboard, Queen election, token budget, metrics from v0.1.
2. **Define a viral hook.** "Fix all TypeScript errors with 2 agents in 5 minutes."
3. **Lower prerequisites.** Allow demo with ONE agent.

### Phase 2: Documentation as Product (Weeks 3-4)
4. **Write docs before code.** README, quickstart, 3 examples BEFORE engine code.
5. **Create 60-second demo video.** Problem → one command → success.
6. **Add contributor documentation.** CONTRIBUTING.md, good first issues, Worker adapter template.

### Phase 3: Ecosystem Hooks (Weeks 5-8)
7. **Make Worker Protocol versioned and stable.** Publish as standalone spec.
8. **Add "Worker Showcase" in README.** List 3rd-party Workers.
9. **Create `#showcase` Discord channel.**

### Phase 4: Measure Adoption, Not Applause (Launch & Beyond)
10. **Track weekly active swarms, not GitHub stars.**
11. **Measure 30-day retention.**
12. **Iterate based on what users actually do.**

---

## Summary Table

| Finding | Severity | Fix Priority |
|---------|----------|--------------|
| No README Strategy | Critical | P0 |
| Prerequisites Not Addressed | High | P0 |
| MVP Scope Too Broad | Critical | P0 |
| Missing Killer Use Case Hook | High | P1 |
| No Contributor Journey | Critical | P1 |
| SDK Documentation Underestimated | High | P1 |
| No Wow Moment Defined | Medium | P2 |
| Dashboard in MVP Increases Friction | Medium | P2 |
| Documentation Scope Not Defined | Critical | P0 |
| No Demo Video Requirement | Medium | P2 |
| Worker Protocol Not Marketed | High | P1 |
| No Plugin Architecture | Medium | P2 |
| SM-1 is Vanity Metric | High | P1 |
| SM-2 Lacks Depth | Medium | P2 |
| Missing Retention Metric | High | P1 |
