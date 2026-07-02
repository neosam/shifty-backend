---
phase: 40-wochen-sperre-durchsetzen
plan: 03
subsystem: service
tags: [rust, service-layer, week-lock, enforcement, tdd, mockall, toctou, security]

# Dependency graph
requires:
  - phase: 40-wochen-sperre-durchsetzen
    provides: "pass-through assert_week_not_locked at 6 write heads, WeekStatusService dep, ServiceError::WeekLocked, delete_booking trait (get->assert->delete)"
provides:
  - "Real week-lock enforcement: assert_week_not_locked blocks non-shiftplan.edit callers in Locked weeks with ServiceError::WeekLocked"
  - "shiftplan.edit bypass (D-40-02) at the helper level (before the status read)"
  - "In-transaction (no-TOCTOU) lock check on all 6 write paths"
  - "Full lock-enforcement test matrix (T-40-01..17) in service_impl/src/test/shiftplan_edit_lock.rs"
affects: [40-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Permission-bypass-before-read: check_permission(shiftplan.edit) short-circuits the status lookup for editors"
    - "TOCTOU-safe gate: status read + write share one transaction; write mock times(0) proves no write before the gate"
    - "delete_booking order: get (read year/week) -> assert_week_not_locked -> delete; delete mock times(0) when Locked"

key-files:
  created:
    - service_impl/src/test/shiftplan_edit_lock.rs
  modified:
    - service_impl/src/shiftplan_edit.rs
    - service_impl/src/test/mod.rs

key-decisions:
  - "Bypass gates on shiftplan.edit (NOT SHIFTPLANNER_PRIVILEGE) — consistent with D-40-02 and the FE is_shift_editor; self-booker/-unbooker (no shiftplan.edit) are correctly blocked in Locked weeks (hard lock incl. self)"
  - "Bypass runs BEFORE the status read (spares the lookup for editors); non-editors read the status in the SAME transaction as the write (no TOCTOU)"
  - "Fixtures duplicated locally in shiftplan_edit_lock.rs (rather than exporting the private fixtures from shiftplan_edit.rs) so Task 1 touches only the two plan-scoped files"

requirements-completed: [WST-03, WST-04]

coverage:
  - id: D1
    description: "assert_week_not_locked blocks non-shiftplan.edit callers in Locked weeks with ServiceError::WeekLocked; shiftplan.edit holders bypass"
    requirement: "WST-03"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl shiftplan_edit_lock (T-40-07/08/09 book_slot; T-40-02/04/05/06 slot paths; T-40-10/11 copy_week)"
        status: pass
    human_judgment: false
  - id: D2
    description: "delete_booking blocks self-unbooking in Locked weeks (WST-04) and reads year/week via get BEFORE delete"
    requirement: "WST-04"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl shiftplan_edit_lock (T-40-12/13/14/15 + T-40-17 delete-order)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Lock check runs in-transaction (no TOCTOU); no write effect before the gate"
    requirement: "WST-03"
    verification:
      - kind: unit
        ref: "cargo test -p service_impl shiftplan_edit_lock (T-40-16 write mock times(0); T-40-17 delete mock times(0))"
        status: pass
    human_judgment: false

# Metrics
duration: 11min
completed: 2026-07-02
status: complete
---

# Phase 40 Plan 03: Wochen-Sperre-Enforcement (TDD) Summary

**The real week-lock enforcement: assert_week_not_locked now blocks non-shiftplan.edit callers in Locked weeks with ServiceError::WeekLocked (shiftplan.edit holders bypass), driven RED-first by the 16-test T-40-01..17 matrix across all 6 write paths + TOCTOU + delete-order.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-07-02
- **Completed:** 2026-07-02
- **Tasks:** 2 (TDD: RED + GREEN)
- **Files created:** 1, **modified:** 2

## Accomplishments
- Replaced the 40-01 pass-through `assert_week_not_locked` with real enforcement: `check_permission("shiftplan.edit")` bypass first, then an in-transaction `get_week_status`; `WeekStatus::Locked` → `Err(ServiceError::WeekLocked { year, week })`.
- All 6 write paths now block non-editors in a Locked week: `modify_slot`, `modify_slot_single_week`, `remove_slot`, `book_slot_with_conflict_check` (non-editor branch), `copy_week_with_conflict_check` (target week), `delete_booking`. shiftplan.edit holders bypass unchanged.
- Added the full lock-enforcement matrix `service_impl/src/test/shiftplan_edit_lock.rs` (T-40-01..17): 6 paths × {Locked, Open}, TOCTOU (write mock times(0)), and delete_booking order (delete mock times(0)).
- Hard lock includes self-unbooking: the bypass gates on shiftplan.edit, so a self-booker/-unbooker (who lacks shiftplan.edit) is correctly blocked in Locked weeks (WST-04 closure).

## Task Commits

Each TDD gate was committed atomically:

1. **Task 1 (RED): lock-enforcement matrix** — `3f54e91` (test) — 16 tests; T-40-07/12/16/17 RED against the pass-through helper.
2. **Task 2 (GREEN): enforce week lock on all six write paths** — `eb7a0f7` (feat) — bypass + in-tx read + WeekLocked; all 16 tests green.

## Files Created/Modified
- `service_impl/src/test/shiftplan_edit_lock.rs` (created) — T-40-01..17 matrix reusing `build_dependencies`/`ShiftplanEditDependencies` from the shiftplan_edit test module.
- `service_impl/src/shiftplan_edit.rs` (modified) — `assert_week_not_locked` helper body: shiftplan.edit bypass → in-tx `get_week_status` → `WeekLocked` on `Locked`.
- `service_impl/src/test/mod.rs` (modified) — registered `mod shiftplan_edit_lock;`.

## Decisions Made
- Bypass gates on `shiftplan.edit` (not `SHIFTPLANNER_PRIVILEGE`), consistent with D-40-02 and the FE `is_shift_editor`; this is what makes the self-unbook block (WST-04) work without a separate self-guard.
- Bypass runs before the status read (editors never trigger the lookup); non-editors read in the same transaction as the write (TOCTOU-safe, T-40-16/17 enforce times(0)).
- Local fixture duplication in the new test module keeps Task 1 scoped to the two plan files (no export churn in shiftplan_edit.rs).

## Deviations from Plan
None - plan executed exactly as written. The RED phase failed exactly the 4 expected enforcement tests (T-40-07/12/16/17); the other 12 tests (Forbidden-first, editor-bypass, open-week, non-existent-id) were green in both phases by design.

## Gate Results
- `cargo test -p service_impl shiftplan_edit_lock`: 16 passed, 0 failed.
- `cargo test --workspace`: green (no regressions).
- `cargo clippy --workspace -- -D warnings`: clean.
- No new `query!`/`query_as!` → no `sqlx prepare` needed.

## Next Phase Readiness
- 40-04 re-routes `DELETE /booking` handler from `booking_service()` to `shiftplan_edit_service().delete_booking()` and documents the OpenAPI 423 — the service-layer enforcement it depends on is now live.

---
*Phase: 40-wochen-sperre-durchsetzen*
*Completed: 2026-07-02*

## Self-Check: PASSED
Both source files, the SUMMARY.md, and both task commits (3f54e91, eb7a0f7) verified present.
