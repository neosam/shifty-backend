---
phase: 30-stale-daten-race-guard-fe
verified: 2026-06-29T00:00:00Z
status: passed
score: 6/6 (5 structural verified + 1 optional manual smoke deferred as UAT, user-accepted 2026-06-29)
behavior_unverified: 0
overrides_applied: 1
acceptance_note: "User accepted the structural verification 2026-06-29 (autonomous --interactive). All automatable must-haves verified; the optional rapid-week-click browser smoke is deferred as non-blocking UAT (plan marked it optional). Additionally code-review WR-01 was fixed within this phase: a fourth summary loader (working_hours_mini) under the shiftplan shared the same race and is now guarded with is_current_selection — all four week-keyed summary loaders are now guarded (strengthens SC2 beyond the 3 enumerated in CONTEXT). FE gates re-run green after the fix (WASM build, 688 tests)."
behavior_unverified_items:
  - truth: "SC1 / D-30-01: Rapid week-switching never shows summary-card data for a week other than the currently displayed one; a result for a different week is dropped, not written."
    test: "Run backend + frontend, click NextWeek/PreviousWeek rapidly (5-10 times in 2 seconds) while watching the summary cards under the shift plan."
    expected: "Summary cards never flash hours from a non-selected week; they either show the selected week's data or the loading/empty state. After rapid clicking settles, cards show only the final selected week's data."
    why_human: "Race condition correctness is a runtime invariant. The predicate is unit-tested and the structural ordering (set_selected_week before dispatch) is source-verified, but the actual absence of transient stale renders requires a live browser session with simulated rapid input — no grep or static analysis can rule out edge cases in the Dioxus coroutine scheduling."
human_verification:
  - test: "Manual smoke: rapid week navigation on the shiftplan page"
    expected: "Summary cards under the shift plan never display data from a non-selected week during or after rapid NextWeek/PreviousWeek clicks. Cards settle to the selected week's data within one loading cycle."
    why_human: "Runtime race condition invariant; not pixel-automatable per plan CONTEXT. Plan explicitly marks this as optional non-blocking manual smoke."
---

# Phase 30: Stale-Daten-Race Guard (FE) Verification Report

**Phase Goal:** Die Summary-Karten unter dem Schichtplan zeigen beim schnellen Wochenwechsel immer nur die Daten der aktuell gewählten Woche — kein gemischter Zustand aus verspäteten Loads.
**Verified:** 2026-06-29
**Status:** passed (structural; optional manual smoke deferred as UAT — user-accepted 2026-06-29; WR-01 4th-loader fix applied + re-verified)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1 / D-30-01: Rapid week-switching never shows summary-card data for a week other than the currently displayed one; a result for a different week is dropped, not written. | PRESENT_BEHAVIOR_UNVERIFIED | All three loaders are structurally guarded (source-confirmed). Ordering invariant (set_selected_week before dispatch) is source-verified. Race elimination at runtime cannot be confirmed without a live browser session. |
| 2 | SC2 / D-30-02: All THREE week loaders (WEEKLY_SUMMARY_STORE / load_summary_for_week, BOOKING_CONFLICTS_STORE / load_booking_conflict_week, reload_unavailable_days) guard their post-await store-write against ONE shared (year,week) truth (SELECTED_WEEK) — no partial-fix antipattern where one loader stays racy. | VERIFIED | grep gate `grep -l is_current_selection ... | wc -l` == 3. Source-confirmed in weekly_summary.rs:57-63, booking_conflict.rs:27-29, shiftplan.rs:376-378. All three import SELECTED_WEEK and is_current_selection from week_guard. |
| 3 | SC3 / D-30-01: A loader result that arrives after a week-switch is silently dropped — no store write, no error, no log spam; displayed data stays consistent with the selected week. | VERIFIED | All three guards are plain `if is_current_selection(...) { write }` with no else-branch, no ERROR_STORE write. Confirmed in weekly_summary.rs:57-64 (returns Ok(()) on mismatch), booking_conflict.rs:27-29, shiftplan.rs:376-378. |
| 4 | D-30-03 (render-guard): The summary cards (weekday_headers) render only when the WEEKLY_SUMMARY_STORE data belongs to the currently selected week; the existing data_loaded loading/empty UX is preserved (no new UX). | VERIFIED | shiftplan.rs:1149-1154: `weekday_headers: if weekly_summary.data_loaded && weekly_summary.weekly_summary.len() > 0 && matches!(weekly_summary.loaded_week, Some(yw) if is_current_selection(yw, (*year.read(), *week.read())))`. Falls through to existing `else { vec![] }` empty state on mismatch. No new UX introduced. |
| 5 | D-30-03 (testability): The staleness decision is a pure predicate is_current_selection((year,week), (year,week)) -> bool, unit-tested via cargo test (match -> write, mismatch -> drop) without Dioxus runtime/async. | VERIFIED | week_guard.rs:28-30: pure free function over tuples, no GlobalSignal read inside. Four unit tests cover: identical → true; stale week → false; stale year → false; result newer than selection → false. Executor reported 4/4 passing. |
| 6 | D-30-04: The year loader (load_weekly_summary_year) is NOT staleness-guarded (different view); but because it shares WEEKLY_SUMMARY_STORE it stamps a None week-key so year data can never mis-render as a selected week in the summary cards — consistency note, scope unchanged. | VERIFIED | weekly_summary.rs:39-47: `store.loaded_week = None` explicitly set in load_weekly_summary_year write block. Render-guard's `matches!(weekly_summary.loaded_week, Some(yw) ...)` correctly rejects None, keeping summary cards in empty state during a year-view load. |

**Score:** 5/6 truths verified (1 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `shifty-dioxus/src/service/week_guard.rs` | New module: SELECTED_WEEK GlobalSignal, set_selected_week, is_current_selection, 4 unit tests | VERIFIED | File exists and is substantive. Exports all three items. Pure predicate confirmed (no GlobalSignal read in is_current_selection). Unit tests cover all 4 behavior cases from the plan spec. |
| `shifty-dioxus/src/service/weekly_summary.rs` | WeeklySummaryStore gains loaded_week; load_summary_for_week guarded; load_weekly_summary_year stamps None | VERIFIED | loaded_week: Option<(u32, u8)> field confirmed at line 22. Guard at lines 57-64. Year loader stamps None at line 45. |
| `shifty-dioxus/src/service/booking_conflict.rs` | load_booking_conflict_week guards write against SELECTED_WEEK | VERIFIED | Guard at lines 27-29. Imports SELECTED_WEEK and is_current_selection from week_guard. |
| `shifty-dioxus/src/page/shiftplan.rs` | set_selected_week called before dispatch at 3 sites; reload_unavailable_days guarded; render-guard on weekday_headers | VERIFIED | set_selected_week at lines 330, 476, 511 — all before their respective dispatch points. reload_unavailable_days guard at line 376. Render-guard at lines 1149-1154. Imports all three from week_guard at line 38. |
| `shifty-dioxus/src/service/mod.rs` | pub mod week_guard registered | VERIFIED | Line 20: `pub mod week_guard;` between vacation_balance and weekly_summary (alphabetical position correct). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `set_selected_week` call (shiftplan.rs:330) | `WeeklySummaryAction::LoadWeek` dispatch (shiftplan.rs:334) | synchronous call before send | VERIFIED | set_selected_week at line 330 precedes the send at line 334. Ordering is correct for initial load. |
| `set_selected_week` call (shiftplan.rs:330) | `reload_unavailable_days` call (shiftplan.rs:382) | synchronous call before await | VERIFIED | set_selected_week at line 330 precedes reload at line 382. |
| `set_selected_week` call (shiftplan.rs:330) | `BookingConflictAction::LoadWeek` dispatch (shiftplan.rs:388-391) | synchronous call before send | VERIFIED | set_selected_week at line 330 precedes send at line 388. |
| `set_selected_week` call (shiftplan.rs:476) | `update_shiftplan()` dispatch (shiftplan.rs:477) | synchronous call before | VERIFIED | NextWeek: set_selected_week(next_weeks_year, next_weeks_week) at 476, update_shiftplan() at 477 (dispatches both LoadWeek actions internally). Ordering correct. |
| `set_selected_week` call (shiftplan.rs:511) | `update_shiftplan()` dispatch (shiftplan.rs:512) | synchronous call before | VERIFIED | PreviousWeek: set_selected_week at 511, update_shiftplan() at 512. Ordering correct. |
| `SELECTED_WEEK` global | `is_current_selection` predicate (all 3 loaders) | read after await | VERIFIED | All three loaders read `*SELECTED_WEEK.read()` after their await and pass to is_current_selection. Single shared truth — no loader-local copies that could drift. |
| `loaded_week` field (WeeklySummaryStore) | render-guard in weekday_headers | `matches!(...) if is_current_selection(...)` | VERIFIED | Render-guard uses is_current_selection(yw, selected) on the stored loaded_week field, same predicate as write-guards. Consistent one decision point. |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 4 predicate unit tests pass | `cargo test week_guard` (from shifty-dioxus/) | Executor reported 4/4 pass | PASS (trusted per verification focus — source confirms 4 #[test] functions) |
| WASM build succeeds | `cargo build --target wasm32-unknown-unknown` (from shifty-dioxus/) | Executor reported clean | PASS (trusted per verification focus) |
| grep gate: all 3 loaders share predicate | `grep -l is_current_selection src/service/weekly_summary.rs src/service/booking_conflict.rs src/page/shiftplan.rs | wc -l` | 3 (verified in this session) | PASS |
| Backend clippy clean | `cargo clippy --workspace -- -D warnings` (from repo root) | Executor reported clean | PASS (trusted per verification focus; phase touches no backend files) |
| Rapid week-switching shows no stale data | Live browser manual smoke | Not run (requires browser + running stack) | SKIP — routed to human verification |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| SHP-02 | 30-01-PLAN.md | `(year,week)`-Guard atomar über alle drei betroffenen Loader; Store-Write nach await nur bei Match + Render-Guard. Reines Frontend. | SATISFIED | All three loaders guarded via is_current_selection + SELECTED_WEEK. Render-guard in weekday_headers. REQUIREMENTS.md line 34-39 matches implementation scope exactly. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | No TBD/FIXME/XXX/TODO stubs found in any of the 5 modified files | — | — |

### Human Verification Required

#### 1. Manual Smoke: Rapid Week Navigation

**Test:** Run the backend (`cargo run` from shifty-backend/) and frontend (`dx serve` from shifty-dioxus/), navigate to the Shiftplan page, and click NextWeek and PreviousWeek rapidly (5-10 clicks within 2 seconds), then let the page settle.

**Expected:** The summary-card headers (showing hours per weekday) never display data from a non-selected week during or after rapid clicking. During loading the cards show the empty/loading state. After the final click settles, the cards display only the selected week's data.

**Why human:** This is a runtime race-condition invariant. The write-guard ordering (set_selected_week before dispatch) and the predicate logic (is_current_selection) are source-verified as structurally correct. But the actual absence of any transient stale render depends on Dioxus coroutine scheduling behavior at runtime, which cannot be inferred from static analysis alone. The plan itself designates this as "optional non-blocking manual smoke (not pixel-automatable, per CONTEXT)."

### Gaps Summary

No structural gaps found. All five required files exist and are substantive. All three loaders contain the guard. The ordering invariant (set_selected_week before dispatch) holds at all three call sites. The render-guard is wired. The predicate is pure and unit-tested. SHP-02 is structurally satisfied.

The single human_needed item is the runtime race-condition smoke test, which the plan itself classified as a non-blocking optional check. It is routed here for completeness per the verifier protocol, not because structural evidence is missing.

---

_Verified: 2026-06-29_
_Verifier: Claude (gsd-verifier)_
