# ACP-UI Known Issues and Remediation Plan

## Critical (Found in Code Review — Fixed)

| # | Issue | File | Status | Fix |
|---|-------|------|--------|-----|
| 1 | Command injection via unknown CLI prompts | agent_orchestration.rs | ✅ Fixed | Reject unknown CLIs, only allow registered safe CLIs |
| 2 | Column index mismatch in approval query | approval_engine.rs | ✅ Fixed | Corrected indices to match SELECT order |
| 3 | btoa stack overflow on large payloads | acp-protocol/index.ts | ✅ Fixed | Chunked encoding uint8ToBase64 helper |
| 4 | cancel_task used task_id as PID | agent_orchestration.rs | ✅ Fixed | Store real PIDs in ActiveTask |
| 5 | ApprovalDecision option_id lost on DB load | approval_engine.rs | ✅ Fixed | Store full decision as JSON |
| 6 | SyncEngine::new panics on init failure | sync_engine.rs + lib.rs | ✅ Fixed | Return Result with fallback |
| 7 | Feishu token never expires | bot_gateway.rs | ✅ Fixed | CachedToken with 2-hour expiry |

## Structural Issues (Not Yet Fixed)

### Issue 1: database.rs ALTER TABLE errors silently ignored

**File:** `src-tauri/src/database.rs` lines 626-630

**Problem:** Five ALTER TABLE statements discard errors with `let _ =`. If migration fails for reasons other than "column already exists", the failure is silently swallowed.

**Impact:** Schema may be partially migrated, causing runtime errors later.

**Fix:** Check error type, log non-duplicate failures.

### Issue 2: Sync Engine checksum is byte-sum (collision-prone)

**File:** `src-tauri/src/sync_engine.rs` lines 646-651

**Problem:** Simple byte sum means permutations of same bytes produce identical checksum.

**Impact:** Data integrity checks are unreliable.

**Fix:** Use sha2 crate for proper hash.

### Issue 3: Flutter hardcoded badge count

**File:** `acp_ui_flutter/lib/screens/main_screen.dart` line 65

**Problem:** Approval badge shows hardcoded `'3'` instead of actual pending count.

**Impact:** Misleading UI state on mobile.

**Fix:** Wire to approval state provider.

### Issue 4: Flutter feedback controller leaks state

**File:** `acp_ui_flutter/lib/widgets/approval_card.dart` lines 75-81

**Problem:** Feedback text from one approval persists when viewing a different request.

**Impact:** User sees wrong feedback text on different approvals.

**Fix:** Add didUpdateWidget to clear controller when request changes.

### Issue 5: Bot gateway 5 of 6 adapters are stubs

**File:** `src-tauri/src/bot_gateway.rs`

**Problem:** Only Feishu is implemented. Others return `supports_* = true` but do nothing.

**Impact:** Misleads callers into thinking functionality is available.

**Fix:** Return false for unsupported capabilities, or mark stubs clearly.

### Issue 6: get_task_history returns entire history

**File:** `src-tauri/src/agent_orchestration.rs` lines 498-500

**Problem:** No pagination, clones entire vector.

**Impact:** Memory issue in long-running systems.

**Fix:** Add limit/offset parameters.

### Issue 7: database.rs duplicate tool_calls tables

**File:** `src-tauri/src/database.rs` lines 783-808 and 985-1004

**Problem:** Two nearly identical tables with overlapping purpose.

**Impact:** Confusing schema, potential data inconsistency.

**Fix:** Consolidate or document difference.

### Issue 8: Flutter ProgressScreen filters 4 times

**File:** `acp_ui_flutter/lib/screens/progress_screen.dart`

**Problem:** `.where()` called 4 times per build.

**Impact:** Unnecessary computation on rebuild.

**Fix:** Compute once, reuse.

### Issue 9: Flutter send button visibility not reactive

**File:** `acp_ui_flutter/lib/screens/instruction_screen.dart` lines 330-338

**Problem:** suffixIcon reads controller text during build, only updates on parent rebuild.

**Impact:** Send button doesn't appear when user types until another state change.

**Fix:** Add listener to controller that calls setState.

### Issue 10: TaskCard shows 100% progress for failed tasks

**File:** `acp_ui_flutter/lib/widgets/task_card.dart` line 128

**Problem:** `TaskStatus.failed` returns 1.0 (100%) progress visually suggests completion.

**Impact:** Failed tasks look completed.

**Fix:** Return distinct value or use different color.

---

## Remediation Priority

| Priority | Issues | Effort |
|----------|--------|--------|
| **P0** | 2, 7 | Schema integrity |
| **P1** | 3, 4, 8, 9, 10 | Flutter UX fixes |
| **P2** | 1, 5, 6 | Bot gateway + pagination |
