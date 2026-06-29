---
phase: 31-abwesenheit-nicht-verf-gbar-markierung-im-schichtplan-fe
plan: "01"
subsystem: shifty-dioxus (frontend)
tags: [frontend, dioxus, shiftplan, absence, discourage-marker, phase-31]
status: complete

dependency_graph:
  requires:
    - Phase 30 week_guard (is_current_selection, SELECTED_WEEK, set_selected_week)
    - Existing loader::load_absence_periods_by_sales_person
    - Existing discourage_weekdays / WeekView (unchanged)
  provides:
    - absence_marker::absence_periods_to_discourage_days (pure, unit-tested helper)
    - person_absences signal + reload_absence_days closure in shiftplan.rs
    - Union-merge of absence days into discourage_weekdays
  affects:
    - shifty-dioxus/src/service/mod.rs
    - shifty-dioxus/src/service/absence_marker.rs (new)
    - shifty-dioxus/src/page/shiftplan.rs

tech_stack:
  added: []
  patterns:
    - Pure helper with exhaustive category match (no wildcard) for compiler-enforced drift prevention
    - Before-await (year,week) capture + is_current_selection write-gate (Phase 30 pattern)
    - Union-merge into existing discourage_weekdays without WeekView prop change

key_files:
  created:
    - shifty-dioxus/src/service/absence_marker.rs
  modified:
    - shifty-dioxus/src/service/mod.rs
    - shifty-dioxus/src/page/shiftplan.rs

decisions:
  - All 3 AbsenceCategory variants (Vacation, SickLeave, UnpaidLeave) trigger the marker — exhaustive match enforces this at compile time
  - Half-day absences (DayFraction::Half) are silently skipped — mirrors shiftplan_edit.rs:538 exactly (D-31-01 / SC2)
  - reload_absence_days closure is guarded by is_current_selection with (year,week) captured before await (D-31-06 / SC3)
  - No new i18n keys, no WeekView prop changes, no new marker type

metrics:
  duration: "~25 minutes"
  completed: "2026-06-29T18:55:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 2
---

# Phase 31 Plan 01: Absence → Nicht-Verfügbar Marker in Shiftplan (FE) Summary

Pure frontend join that union-merges the current person's full-day absence periods into the existing `discourage_weekdays` signal — using a new `absence_marker.rs` helper with exhaustive category match and a guarded `reload_absence_days` closure mirroring Phase 30's week-guard pattern.

## What Was Built

### Task 1: Pure helper `absence_periods_to_discourage_days` + unit tests

Created `shifty-dioxus/src/service/absence_marker.rs` with:

- `pub fn absence_periods_to_discourage_days(absences: &[AbsencePeriod], week_monday: time::Date) -> Vec<Weekday>` — for each day offset 0..=6, includes the `Weekday` iff any absence has `day_fraction == Full`, triggers `category_triggers_marker`, and overlaps the concrete date.
- Private `fn category_triggers_marker(c: AbsenceCategory) -> bool` — exhaustive `match` over all 3 variants returning `true`; no wildcard arm so a future 4th variant forces review (D-31-01 / SC2 / zero-drift).
- 7 unit tests covering: Vacation Full Tue–Thu, SickLeave Full single day, UnpaidLeave Full single day, Half-day empty, out-of-week empty, full-week 7 days, partial overlap at start.

Registered via `pub mod absence_marker;` added alphabetically in `service/mod.rs`.

### Task 2: Wire `person_absences` signal + guarded `reload_absence_days` + union-merge

Modified `shifty-dioxus/src/page/shiftplan.rs`:

1. Added `let person_absences: Signal<Rc<[AbsencePeriod]>> = use_signal(|| [].into());` alongside `unavailable_days`.
2. Added `use crate::service::absence_marker;` and `use crate::state::absence_period::AbsencePeriod;` imports.
3. Added `person_absences` to the coroutine `to_owned![...]` capture list.
4. Defined `reload_absence_days` closure immediately after `reload_unavailable_days` — identical pattern: `to_owned![current_sales_person, person_absences]`, captures `req_year`/`req_week` before the await, loads all absences via `loader::load_absence_periods_by_sales_person`, discards markers, writes only if `is_current_selection((req_year, req_week), *SELECTED_WEEK.read())` (D-31-06 / SC3).
5. Called `reload_absence_days(config.clone()).await;` at exactly 4 trigger sites: initial load, NextWeek, PreviousWeek, UpdateSalesPerson. NOT at ToggleAvailability.
6. Added `person_absences` to the `to_owned![...]` before the WeekView RSX.
7. Updated `discourage_weekdays` to union-merge: collects unavailable_days weekdays into a `Vec<Weekday>`, extends with `absence_marker::absence_periods_to_discourage_days(person_absences.read().as_ref(), date)`, passes as `Rc<[Weekday]>`. WeekView prop type unchanged.

## Verification Results

| Gate | Result |
|------|--------|
| `cargo test absence_marker` | 7/7 pass |
| `cargo build --target wasm32-unknown-unknown` | Succeeded |
| `cargo clippy --workspace -- -D warnings` (backend) | Clean |
| `cargo test` (full dioxus suite) | 695/695 pass |
| `grep -c 'reload_absence_days(config.clone()).await'` | 4 (correct) |
| `grep -c 'absence_periods_to_discourage_days'` in shiftplan.rs | 1 (correct) |
| ToggleAvailability site has NO `reload_absence_days` | Confirmed |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The `person_absences` signal starts empty (`[].into()`) and is populated by `reload_absence_days` on first render, just like `unavailable_days`. No placeholder text, no hardcoded data.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes beyond what the threat model already covered (T-31-01 person-scope + T-31-02 stale-state guard).

## Changed Files

- `shifty-dioxus/src/service/absence_marker.rs` — new pure helper + 7 unit tests
- `shifty-dioxus/src/service/mod.rs` — added `pub mod absence_marker;`
- `shifty-dioxus/src/page/shiftplan.rs` — new signal, new closure, 4 trigger calls, union-merge

Working tree is intentionally left dirty (commit_docs: false — user commits manually with jj).

## Self-Check: PASSED

- `shifty-dioxus/src/service/absence_marker.rs` exists and tests pass.
- `shifty-dioxus/src/service/mod.rs` contains `pub mod absence_marker;`.
- `shifty-dioxus/src/page/shiftplan.rs` has 4 `reload_absence_days` trigger calls (not 5, not 3).
- WASM build clean.
- Backend clippy clean.
- Full FE test suite: 695 pass.
