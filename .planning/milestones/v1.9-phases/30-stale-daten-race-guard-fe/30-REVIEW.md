---
phase: 30-stale-daten-race-guard-fe
reviewed: 2026-06-29T00:00:00Z
depth: deep
files_reviewed: 5
files_reviewed_list:
  - shifty-dioxus/src/service/week_guard.rs
  - shifty-dioxus/src/service/mod.rs
  - shifty-dioxus/src/service/weekly_summary.rs
  - shifty-dioxus/src/service/booking_conflict.rs
  - shifty-dioxus/src/page/shiftplan.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: resolved
resolution_note: "WR-01 (working_hours_mini unguarded — 4th summary loader with the same race) FIXED 2026-06-29: guarded its post-await write with is_current_selection((year,week), *SELECTED_WEEK.read()), mirroring booking_conflict.rs; set_selected_week already runs before its dispatch in all paths. Re-verified: WASM build clean, 688 FE tests green. IN-01 (data_loaded=false loading-flash) ACCEPTED as pre-existing/non-blocking (no permanent stuck state; current-week load always follows)."
---

# Phase 30: Code Review Report

**Reviewed:** 2026-06-29
**Depth:** deep
**Files Reviewed:** 5
**Status:** issues_found (1 warning, 1 info — race fix itself is correct)

## Summary

Phase 30 introduces a shared `SELECTED_WEEK` global signal and a pure `is_current_selection`
predicate to drop in-flight week-loader results that arrive after a week-switch. The concurrency
fix is sound for the three explicitly targeted loaders (`booking_conflict`, `weekly_summary` week
load, `reload_unavailable_days`). SELECTED_WEEK is always written synchronously before any
LoadWeek dispatch or `reload_unavailable_days` call in all three code paths (initial mount,
NextWeek, PreviousWeek). The `data_loaded = false` pre-await write is a pre-existing pattern and
the correct week's load always follows, so no store is left permanently stuck. The render-guard
(D-30-03) provides correct defense-in-depth using local signals that always agree with
SELECTED_WEEK. Tests cover the pure predicate adequately.

One unguarded loader of identical race-class was left out of scope.

---

## Warnings

### WR-01: `working_hours_mini_service` has no staleness guard — same race class, fourth loader

**File:** `shifty-dioxus/src/service/working_hours_mini.rs:22-33`

**Issue:** Phase 30 guards three of the four week-context loaders dispatched by `update_shiftplan()`.
`WorkingHoursMiniAction::LoadWorkingHoursMini(year, week, ...)` is dispatched at the same call
sites (initial mount and inside `update_shiftplan()`) but the service has no post-await
`is_current_selection` check. A slow HTTP response from week N can overwrite `WORKING_HOURS_MINI`
after the user has navigated to week N+1, displaying wrong per-employee hour bars.

The pattern is identical to the bug the phase was designed to eliminate:

```rust
// working_hours_mini.rs — current code (no guard)
async fn working_hours_mini_service(mut rx: ...) {
    while let Some(action) = rx.next().await {
        match action {
            WorkingHoursMiniAction::LoadWorkingHoursMini(year, week, fetch_balance) => {
                let working_hours = loader::load_working_hours_minified_for_week(
                    CONFIG.read().clone(), year, week, fetch_balance,
                ).await;
                match working_hours {
                    Ok(working_hours) => {
                        *WORKING_HOURS_MINI.write() = working_hours;  // no guard
                    }
                    ...
                }
            }
        }
    }
}
```

**Fix:** Apply the same guard pattern:

```rust
use crate::service::week_guard::{is_current_selection, SELECTED_WEEK};

WorkingHoursMiniAction::LoadWorkingHoursMini(year, week, fetch_balance) => {
    let working_hours = loader::load_working_hours_minified_for_week(
        CONFIG.read().clone(), year, week, fetch_balance,
    ).await;
    match working_hours {
        Ok(working_hours) => {
            if is_current_selection((year, week), *SELECTED_WEEK.read()) {
                *WORKING_HOURS_MINI.write() = working_hours;
            }
        }
        Err(err) => { /* unchanged */ }
    }
}
```

---

## Info

### IN-01: Stale `LoadSummaryForWeek` tasks set `data_loaded = false` before the guard can reject

**File:** `shifty-dioxus/src/service/weekly_summary.rs:52`

**Issue:** `load_summary_for_week` sets `data_loaded = false` unconditionally at the top of the
function — before the HTTP fetch and before the staleness guard is evaluated:

```rust
async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;   // fires for stale loads too
    let weekly_summary = loader::...().await?;
    if is_current_selection((year, week), *SELECTED_WEEK.read()) {
        // data_loaded = true only here
    }
    // if guard rejects: data_loaded stays false until the next load completes
    Ok(())
}
```

When the weekly_summary_service queue contains `[LoadWeek(stale), LoadWeek(current)]`, processing
the stale action sets `data_loaded = false` (hiding the summary cards) and then drops its result,
leaving cards hidden until the current-week load resolves. On fast networks this is imperceptible;
on slow connections it creates a "loading" flash on every week-switch even when the previous week's
cards were visible before navigation.

This is a pre-existing behavior pattern (the `data_loaded = false` pre-await write predates Phase
30); Phase 30 did not introduce the flicker but it also did not eliminate it for the stale-then-
current-queue scenario. A guard on the pre-await write is not straightforward because `year`/`week`
should match SELECTED_WEEK at that moment — the stale load only queues up when navigation happens
after dispatch. The simplest mitigation is to store the previous data and restore it on a rejected
result, but that adds complexity. Noting for awareness.

---

## Correctness Deep-Dive

The following questions from the review scope were each confirmed sound:

**Guard truth set before all dispatches?**
- Initial mount (line 330): `set_selected_week(*year.read(), *week.read())` is the first
  synchronous statement in the coroutine body, before the `weekly_summary_service.send(...)`,
  before any `.await`. All subsequent loads receive the correct truth. ✓
- NextWeek (line 476): `set_selected_week(next_weeks_year, next_weeks_week)` uses locally-computed
  values (not re-read from signals) immediately before `update_shiftplan()`. ✓
- PreviousWeek (line 511): same pattern. ✓

**Any LoadWeek dispatched WITHOUT first updating SELECTED_WEEK?**
- `update_shiftplan()` dispatches `BookingConflictAction::LoadWeek(*year.read(), *week.read())` and
  `WeeklySummaryAction::LoadWeek(*year.read(), *week.read())`. Both are always preceded by
  `set_selected_week(...)` in every call site. ✓

**`reload_unavailable_days` captures (year, week) before the await?**
- `req_year = *year.read()` and `req_week = *week.read()` are plain local variables bound before
  `loader::load_unavailable_sales_person_days_for_week(...).await`. The post-await guard
  `is_current_selection((req_year, req_week), *SELECTED_WEEK.read())` uses the pre-captured
  values. No post-await signal re-read of year/week. ✓

**TOCTOU window in single-threaded WASM?**
- Dioxus coroutines yield only at `.await` points; there is no preemption. `set_selected_week` is
  always called in a synchronous block before the first await, so no scheduler switch can occur
  between `set_selected_week` and the LoadWeek dispatch. ✓

**Can a dropped stale result leave a store permanently stuck on loading?**
- `WEEKLY_SUMMARY_STORE (data_loaded=false)`: The current week's LoadWeek is always in the queue
  immediately behind the stale one (dispatched by `update_shiftplan()` before the stale result
  arrives), so `data_loaded` is eventually restored to `true`. ✓
- `BOOKING_CONFLICTS_STORE`: No `data_loaded` field; worst case the old value persists briefly
  until the current-week load completes. ✓
- `unavailable_days`: No `data_loaded` field; old value persists briefly. ✓

**Render-guard correctness:**
- Uses `(*year.read(), *week.read())` from local component signals, which always agree with
  `SELECTED_WEEK` (both are updated together in every navigation path). Comparing against local
  signals rather than SELECTED_WEEK is slightly more direct (the local signals drive the visible
  header), and is correct. ✓

**Pure predicate tests (4/4):**
Covers same-year-same-week, stale-week, stale-year, and result-ahead-of-selection. Sufficient for
a one-liner equality predicate. ✓

---

_Reviewed: 2026-06-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
