---
phase: 30-stale-daten-race-guard-fe
plan: "01"
subsystem: frontend
status: complete
tags: [concurrency, race-guard, dioxus, wasm, frontend]
decisions:
  - D-30-01: Staleness-check after await (not cancellation); late-arriving results silently dropped
  - D-30-02: One shared GlobalSignal<(u32,u8)> (SELECTED_WEEK) as the single guard truth for all three loaders
  - D-30-03: Render-guard via loaded_week field in WeeklySummaryStore; reuses is_current_selection predicate
  - D-30-04: Year loader stamps loaded_week=None so year data never satisfies the week render-guard
dependency_graph:
  requires: []
  provides:
    - crate::service::week_guard (SELECTED_WEEK, set_selected_week, is_current_selection)
    - guarded WEEKLY_SUMMARY_STORE writes in load_summary_for_week
    - guarded BOOKING_CONFLICTS_STORE writes in load_booking_conflict_week
    - guarded unavailable_days writes in reload_unavailable_days closure
    - render-guard on weekday_headers in shiftplan.rs
  affects:
    - shifty-dioxus/src/service/week_guard.rs (new)
    - shifty-dioxus/src/service/mod.rs
    - shifty-dioxus/src/service/weekly_summary.rs
    - shifty-dioxus/src/service/booking_conflict.rs
    - shifty-dioxus/src/page/shiftplan.rs
tech_stack:
  added:
    - GlobalSignal<(u32,u8)> for guard truth (week_guard.rs)
  patterns:
    - Post-await staleness check: compare (year,week) pair after each loader await before writing store
    - Shared guard truth: single GlobalSignal updated synchronously before every dispatch
    - Pure predicate: is_current_selection as a free fn over tuples, unit-testable without Dioxus runtime
    - Render-guard: Option<(u32,u8)> loaded_week field in store, matched against selected week in RSX
key_files:
  created:
    - shifty-dioxus/src/service/week_guard.rs
  modified:
    - shifty-dioxus/src/service/mod.rs
    - shifty-dioxus/src/service/weekly_summary.rs
    - shifty-dioxus/src/service/booking_conflict.rs
    - shifty-dioxus/src/page/shiftplan.rs
metrics:
  duration: ~30 min
  completed: 2026-06-29
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 4
---

# Phase 30 Plan 01: Stale-Week Race Guard (FE) Summary

**One-liner:** `(year,week)`-staleness guard with shared `SELECTED_WEEK` GlobalSignal and pure `is_current_selection` predicate wired atomically across all three week loaders and a render-guard in the summary-card headers.

## Objective

Eliminate the race condition where rapid week-switching could cause the shiftplan summary cards to transiently display data from a non-selected week. When the user clicks NextWeek or PreviousWeek quickly, a late-arriving API response for the old week must be silently dropped, never written to the store, and never rendered.

## What Was Built

### Task 1: Shared Guard Truth + Pure Predicate (week_guard.rs)

New module `shifty-dioxus/src/service/week_guard.rs` exports:

- `pub static SELECTED_WEEK: GlobalSignal<(u32, u8)>` — the single shared truth for all three week loaders (D-30-02). Initializes lazily from `js::get_current_year()`/`get_current_week()` (same helpers shiftplan.rs uses), overwritten synchronously on every week change before dispatching loaders.
- `pub fn set_selected_week(year: u32, week: u8)` — imperative setter for the guard truth; called in shiftplan.rs BEFORE every LoadWeek dispatch and reload_unavailable_days call.
- `pub fn is_current_selection(result_yw: (u32, u8), selected_yw: (u32, u8)) -> bool` — pure free function over tuples; no GlobalSignal read inside, making it unit-testable without a Dioxus runtime (D-30-03 testability requirement).

Four unit tests cover all behavior cases from the plan spec:
- `match_same_year_and_week_allows_write` — identical tuples → true
- `mismatch_stale_week_drops_write` — different week → false
- `mismatch_stale_year_drops_write` — different year → false
- `result_ahead_of_selection_drops_write` — result newer than selection → false

Registered via `pub mod week_guard;` in `service/mod.rs` between `vacation_balance` and `weekly_summary`.

### Task 2: Guard Wired Into All Three Loaders + Render-Guard

**weekly_summary.rs:**
- `WeeklySummaryStore` gains `pub loaded_week: Option<(u32, u8)>`, defaulting to `None`.
- `load_summary_for_week`: reads `SELECTED_WEEK` after the await, writes the store (including `loaded_week = Some((year, week))`) only if `is_current_selection` returns true; otherwise returns `Ok(())` without touching the store (no error, no log — SC3).
- `load_weekly_summary_year`: stamps `loaded_week = None` when it writes, ensuring year data can never satisfy the week render-guard (D-30-04).

**booking_conflict.rs:**
- `load_booking_conflict_week`: reads `SELECTED_WEEK` after the await, writes `BOOKING_CONFLICTS_STORE` only on a match; silently drops on mismatch.

**shiftplan.rs:**
- Import: `use crate::service::week_guard::{is_current_selection, set_selected_week, SELECTED_WEEK};`
- `set_selected_week` called synchronously BEFORE the first LoadWeek dispatch (initial setup), BEFORE `update_shiftplan()` in NextWeek handler, and BEFORE `update_shiftplan()` in PreviousWeek handler.
- `reload_unavailable_days` closure: captures `req_year` and `req_week` as local values before the await; writes `unavailable_days` only if `is_current_selection((req_year, req_week), *SELECTED_WEEK.read())` is true after the await.
- `weekday_headers` render-guard: the populated `vec![...]` branch now additionally requires `matches!(weekly_summary.loaded_week, Some(yw) if is_current_selection(yw, (*year.read(), *week.read())))`, ensuring even a write that somehow slips through is hidden by the render layer. The existing `else { vec![] }` empty/loading state is reused without new UX (D-30-03).

## Verification Gates

All gates pass:

1. **`cargo test week_guard`** (from shifty-dioxus/) — 4 predicate tests pass.
   Note: `cargo test --lib week_guard` fails because `shifty-dioxus` has no lib target (binary-only crate). `cargo test week_guard` achieves the same result by running test functions matching the module path pattern.

2. **`cargo build --target wasm32-unknown-unknown`** (from shifty-dioxus/) — WASM build succeeds with pre-existing warnings only (no new errors or warnings from this phase).

3. **`grep -l is_current_selection src/service/weekly_summary.rs src/service/booking_conflict.rs src/page/shiftplan.rs | wc -l` == 3** — all three loaders provably share the one guard predicate (SC2, no partial fix).

4. **`cargo clippy --workspace -- -D warnings`** (from repo root) — backend gate passes clean.

5. **Full FE `cargo test`** — 688 tests pass, 0 failures.

## Deviations from Plan

### Deviation: `cargo test --lib` not applicable to binary crate

**Rule:** Out-of-scope pre-existing condition, documented for clarity.
**Found during:** Task 1 verification.
**Issue:** The plan specifies `cargo test --lib week_guard` but `shifty-dioxus` has no `[lib]` target in `Cargo.toml` — it is a binary crate (`src/main.rs`). `cargo test --lib` errors with "no library targets found".
**Fix:** Used `cargo test week_guard` instead, which finds and runs the same 4 tests via the binary test binary (the `#[cfg(test)]` module in `week_guard.rs` is compiled into the binary test binary, accessible as `service::week_guard::tests::*`). All 4 tests pass.

No other deviations from plan. Code executed exactly as specified.

## Known Stubs

None. The guard is fully wired. No placeholder values, no TODO stubs in the delivered files.

## Threat Flags

None. This phase introduces no new network endpoints, auth paths, file access patterns, or schema changes. The only new surface is a client-side `GlobalSignal<(u32, u8)>` in WASM memory; threat T-30-01 (tampering) and T-30-02 (info disclosure) were assessed in the plan's threat model and accepted/mitigated respectively.

## Self-Check

- [x] `shifty-dioxus/src/service/week_guard.rs` exists
- [x] `shifty-dioxus/src/service/mod.rs` contains `pub mod week_guard;`
- [x] `shifty-dioxus/src/service/weekly_summary.rs` contains `is_current_selection` guard
- [x] `shifty-dioxus/src/service/booking_conflict.rs` contains `is_current_selection` guard
- [x] `shifty-dioxus/src/page/shiftplan.rs` contains `set_selected_week`, `is_current_selection`, `SELECTED_WEEK` usages
- [x] 4 unit tests pass (`cargo test week_guard`)
- [x] WASM build passes (`cargo build --target wasm32-unknown-unknown`)
- [x] grep gate == 3
- [x] Backend clippy clean (`cargo clippy --workspace -- -D warnings`)
- [x] 688 FE tests pass (`cargo test`)
- [x] No commits made (working tree left dirty per jj-managed VCS constraint)

## Self-Check: PASSED
