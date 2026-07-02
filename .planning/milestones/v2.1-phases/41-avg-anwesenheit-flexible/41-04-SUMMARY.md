---
phase: 41-avg-anwesenheit-flexible
plan: 04
subsystem: frontend
tags: [reporting, attendance, avg-02, avg-03, i18n, dioxus, wasm, employee-view]

requires:
  - phase: 41-avg-anwesenheit-flexible
    provides: 41-03 EmployeeAttendanceStatisticsTO (rest-types) + GET /report/{id}/attendance-statistics endpoint
provides:
  - "shifty-dioxus i18n: Key::AvgHoursPerAttendanceDay / *Description / *Empty (de/en/cs) + i18n_attendance_keys_present_in_all_locales"
  - "shifty-dioxus api: get_employee_attendance_statistics -> Result<Option<Rc<EmployeeAttendanceStatisticsTO>>, reqwest::Error>"
  - "shifty-dioxus EmployeeStore.attendance_statistics field + load_employee_data wiring"
  - "shifty-dioxus employee_view: attendance_statistics prop + TupleRow rendering in HR-stats block"
affects: []

tech-stack:
  added: []
  patterns:
    - "Nullable HR statistic consumed FE-side as Option<Rc<TO>>: JSON null -> None (row hidden), inner Option<f32> None -> EN-DASH empty state"
    - "Store field always re-set (even None) in load_employee_data to clear stale state across employee switches (Pitfall 5 / T-41-09)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/service/employee.rs
    - shifty-dioxus/src/component/employee_view.rs

key-decisions:
  - "Row placed directly after 'Ø Std/Woche' and before 'Einbezogene Wochen' in the existing should_show_hr_stats section (D-AVG-07)"
  - "Two-level Option (D-AVG-05/06): outer None (non-flexible/non-HR) -> row not rendered; inner average None (<2 days) -> EN-DASH '–' text-ink-muted + title=Empty"
  - "/report proxy already present in Dioxus.toml (shared with weekly-statistics) — no proxy change needed"

patterns-established:
  - "Nullable HR statistic: single 200 path, JSON null -> None -> hidden row; empty inner value -> dimmed EN-DASH with a11y title"

requirements-completed: [AVG-02, AVG-03]

coverage:
  - id: D1
    description: "3 i18n keys (label/description/empty) in de/en/cs + completeness test"
    requirement: "AVG-03"
    verification:
      - kind: test
        ref: "cargo test i18n_attendance_keys_present_in_all_locales (green)"
        status: pass
    human_judgment: false
  - id: D2
    description: "FE api loader (Option return) + EmployeeStore field + load_employee_data wiring"
    requirement: "AVG-02"
    verification:
      - kind: build
        ref: "cargo build --target wasm32-unknown-unknown (green)"
        status: pass
    human_judgment: false
  - id: D3
    description: "TupleRow rendering in HR-stats block: number / EN-DASH empty / hidden-when-None"
    requirement: "AVG-02"
    verification:
      - kind: test
        ref: "attendance_row_shows_number_when_some / _shows_endash_when_inner_none / _absent_when_none (green)"
        status: pass
      - kind: build
        ref: "cargo build --target wasm32-unknown-unknown + cargo test -p shifty-dioxus (green, only pre-existing unrelated impersonation test fails)"
        status: pass
    human_judgment: false

duration: ~12min
completed: 2026-07-02
status: complete
---

# Phase 41 Plan 04: FE Ø-Anwesenheit — i18n + loader + EmployeeStore + TupleRow Summary

**Frontend vertical slice completing AVG-02/AVG-03: three de/en/cs i18n keys (+ completeness test), the `get_employee_attendance_statistics` loader returning `Option<Rc<EmployeeAttendanceStatisticsTO>>` (JSON `null` → `None`), an `EmployeeStore.attendance_statistics` field wired into `load_employee_data`, and a `TupleRow` rendered next to "Ø Std/Woche" in the HR-stats block — showing `format_hours(avg, 2)`, an EN-DASH "–" empty state for <2 attendance days, and no row at all for non-flexible employees.**

## Performance
- **Duration:** ~12 min
- **Tasks:** 3
- **Files created/modified:** 7

## Accomplishments
- i18n: `Key::AvgHoursPerAttendanceDay`, `AvgHoursPerAttendanceDayDescription`, `AvgHoursPerAttendanceDayEmpty` added to `mod.rs`; exact UI-SPEC de/en/cs texts in `de.rs`/`en.rs`/`cs.rs`; `#[test] i18n_attendance_keys_present_in_all_locales` (3 keys × 3 locales, non-empty/non-"??").
- api.rs: `get_employee_attendance_statistics(config, id, year, until_week) -> Result<Option<Rc<EmployeeAttendanceStatisticsTO>>, reqwest::Error>` — `error_for_status_ref()` for HTTP errors, then `json::<Option<..>>()` so server `null` maps to `None`, wrapped as `Ok(opt.map(Rc::new))`.
- employee.rs: `EmployeeStore.attendance_statistics: Option<Rc<EmployeeAttendanceStatisticsTO>>` + `None` in Default; `load_employee_data` loads with `year`/`until_week` and always re-sets the field (even `None`) to clear stale state across employee switches.
- employee_view.rs: `attendance_statistics` prop on `EmployeeViewPlain`, passed from `EMPLOYEE_STORE`; `TupleRow` inserted directly after `AverageWorkedHoursPerWeek` and before `StatisticsIncludedWeeks` inside the `should_show_hr_stats` section. `Some(avg)` → `span font-mono tabular-nums` `format_hours(avg,2)`; inner `None` → `span font-mono tabular-nums text-ink-muted title=Empty` "–"; outer `None` → no row. Description slot carries the inline explanation.
- 3 SSR tests added mirroring the row: number / EN-DASH empty / absent-when-None.

## Task Commits
1. **Task 1: i18n keys (de/en/cs) + completeness test** — `607fdbd` (feat)
2. **Task 2: api loader + EmployeeStore field + load_employee_data wiring** — `5e140a5` (feat)
3. **Task 3: TupleRow rendering in HR-stats block + SSR tests** — `5b28f1d` (feat)

## Files Created/Modified
- `shifty-dioxus/src/i18n/mod.rs` — 3 enum keys + completeness test (modified)
- `shifty-dioxus/src/i18n/de.rs` / `en.rs` / `cs.rs` — translations (modified)
- `shifty-dioxus/src/api.rs` — loader with Option return + TO import (modified)
- `shifty-dioxus/src/service/employee.rs` — store field + Default + loader wiring (modified)
- `shifty-dioxus/src/component/employee_view.rs` — prop + TupleRow + prop pass-through + 3 SSR tests (modified)

## Decisions Made
- Row placement after "Ø Std/Woche", before "Einbezogene Wochen" (D-AVG-07).
- Two-level Option handling (D-AVG-05/06): outer None → hidden row, inner None → EN-DASH empty state with a11y `title`.
- `/report` dev-proxy already present in `Dioxus.toml` (shared with weekly-statistics) — no config change.

## Deviations from Plan
None — plan executed as written. (Added 3 SSR rendering tests beyond the required i18n completeness test, per the "always have tests for the changes" project rule; no scope change.)

## Issues Encountered
- WASM build needs `nix develop` (bare `cargo build --target wasm32-unknown-unknown` fails with `linker lld not found`) — ran all WASM/clippy gates inside the nix dev shell.

## Scope Guard
- No new FE dependencies (T-41-SC).
- `/report` proxy unchanged; no Dioxus.toml edit.
- No backend changes; snapshot schema untouched.

## Gate Results
- `cargo test i18n_attendance_keys_present_in_all_locales`: green
- `cargo build --target wasm32-unknown-unknown` (nix develop): green
- `cargo test -p shifty-dioxus`: 745 passed, 1 failed — the failure is the pre-existing, unrelated `i18n_impersonation_keys_match_german_reference` (documented as not this plan's concern); all 4 Phase-41 tests (i18n completeness + 3 attendance SSR) green
- `cargo clippy --workspace -- -D warnings` (backend, nix develop): clean

## Known Stubs
None — the row is wired end-to-end to the live `EmployeeAttendanceStatisticsTO` from the 41-03 endpoint.

## Next Phase Readiness
- Phase 41 fully delivered (backend 41-01/02/03 + frontend 41-04). AVG-01/02/03 complete.
- No blockers.

## Self-Check: PASSED
- FOUND: shifty-dioxus/src/i18n/mod.rs (3 keys + test)
- FOUND: shifty-dioxus/src/api.rs (get_employee_attendance_statistics)
- FOUND: shifty-dioxus/src/service/employee.rs (attendance_statistics field + wiring)
- FOUND: shifty-dioxus/src/component/employee_view.rs (prop + TupleRow + SSR tests)
- FOUND commit: 607fdbd (Task 1)
- FOUND commit: 5e140a5 (Task 2)
- FOUND commit: 5b28f1d (Task 3)

---
*Phase: 41-avg-anwesenheit-flexible*
*Completed: 2026-07-02*
