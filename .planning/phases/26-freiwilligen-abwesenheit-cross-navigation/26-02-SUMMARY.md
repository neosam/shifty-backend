---
phase: 26-freiwilligen-abwesenheit-cross-navigation
plan: "02"
subsystem: service_impl/test
tags: [rust, test, booking_information, vfa, absence, volunteer, weekly_summary, regression]

requires:
  - phase: 26-freiwilligen-abwesenheit-cross-navigation
    plan: "01"
    provides: >
      period_overlaps_week helper + AbsenceService dep wired in get_weekly_summary
      (the implementation this plan's tests exercise)

provides:
  - VFA-02 full-service regression test in booking_information_vfa.rs
  - holiday-vs-absence asymmetry locked by executable CI guard (D-26-04)
  - CURRENT_SNAPSHOT_SCHEMA_VERSION==11 Phase-26-specific CI guard (D-26-02)

affects:
  - CI: any future change that accidentally couples holiday handling to committed_voluntary
    will now fail vfa02_holiday_vs_absence_asymmetry
  - CI: any unintentional snapshot schema bump during Phase-26 work will now fail
    phase26_vfa_no_snapshot_bump

tech-stack:
  added: []
  patterns:
    - "Full-service BookingInformationServiceImpl mock: all 13 deps (incl. MockAbsenceService)
      wired via TestDeps impl of BookingInformationServiceDeps"
    - "Per-week dispatch in mock: special_day_service.expect_get_by_week returning closure
      branches on (year, week) to return Holiday only for HOLIDAY_WEEK, empty otherwise"
    - "Asymmetry assertion: explicit third assert naming D-26-04 vs D-26-03 contrast"

key-files:
  created:
    - service_impl/src/test/booking_information_vfa.rs
  modified:
    - service_impl/src/test/mod.rs

key-decisions:
  - "Used returning(|year, week, _| ...) with inline branch in single expectation rather
    than two withf expectations — cleaner, avoids mockall LIFO ordering subtleties"
  - "ABSENCE_WEEK chosen as W20, HOLIDAY_WEEK as W15 — 5 weeks apart, no adjacency risk of
    a single absence period accidentally bleeding into the holiday week"
  - "MockBookingService and MockSalesPersonUnavailableService created with no expectations —
    get_weekly_summary does not call these services; no expectations means no panic on drop"
  - "EmployeeWorkDetails valid from (2026,1) to (2027,3) — covers all 52 2026 weeks plus the
    3 overflow iterations that get_weekly_summary runs into the following year"

metrics:
  duration: 20min
  completed: 2026-06-28T18:07:11Z
  tasks: 2
  files_created: 1
  files_modified: 1

status: complete
---

# Phase 26 Plan 02: VFA-02 Holiday-vs-Absence Asymmetry Regression Summary

**Full-service `get_weekly_summary` regression test locking the D-26-04 deliberate asymmetry: a `SpecialDayType::Holiday` does NOT reduce `committed_voluntary_hours`; an `AbsencePeriod` (any category) DOES reduce it to 0 (whole-week-out, D-26-03). Both exercised for the same volunteer in one test. Plus a Phase-26-specific snapshot-schema-version guard (D-26-02).**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-06-28T17:45:00Z
- **Completed:** 2026-06-28T18:07:11Z
- **Tasks:** 2
- **Files created:** 1
- **Files modified:** 1

## Accomplishments

- `service_impl/src/test/booking_information_vfa.rs` (new): full-service `BookingInformationServiceImpl` test with all 13 dependency mocks (shiftplan_report, slot, booking, sales_person, sales_person_unavailable, reporting, special_day, employee_work_details, absence, permission, clock, uuid, transaction). Two tests:
  - `vfa02_holiday_vs_absence_asymmetry`: builds the service, calls `get_weekly_summary(2026, ...)`, asserts `HOLIDAY_WEEK.committed_voluntary_hours ≈ 5.0` (holiday unchanged — D-26-04) and `ABSENCE_WEEK.committed_voluntary_hours ≈ 0.0` (absence reduces — D-26-03), plus an explicit asymmetry assertion.
  - `phase26_vfa_no_snapshot_bump`: pin `CURRENT_SNAPSHOT_SCHEMA_VERSION == 11` as a Phase-26-specific CI guard (D-26-02).
- `service_impl/src/test/mod.rs` updated: `pub mod booking_information_vfa;` registered.

## Task Commits

1. **Task 1 + Task 2 (atomic):** `93b5b3f` — `test(26): VFA-02 holiday-vs-absence asymmetry regression (26-02)`

## Files Created/Modified

- `service_impl/src/test/booking_information_vfa.rs` — new, 352 lines, 2 tests
- `service_impl/src/test/mod.rs` — 2 lines added (module registration)

## Decisions Made

- **Single returning closure with inline branch for special_day_service** (instead of two separate withf expectations): avoids mockall LIFO/FIFO ordering subtleties; a single closure dispatching on (year, week) is simpler and more readable.
- **HOLIDAY_WEEK=W15, ABSENCE_WEEK=W20**: 5 weeks apart, ensuring no single AbsencePeriod can accidentally span both weeks — the asymmetry is unambiguous.
- **BookingService and SalesPersonUnavailableService with no expectations**: `get_weekly_summary` does not call these services; creating their mocks with `::new()` and no expectations is correct — mockall only panics if a set expectation is unmet, not if a mock is never called.
- **EmployeeWorkDetails valid (2026,1)→(2027,3)**: covers all 55 iterations of `get_weekly_summary`'s week loop (52 weeks of 2026 + 3 overflow into 2027).

## Deviations from Plan

None — plan executed exactly as written.

## Gate Results

| Gate | Result | Notes |
|------|--------|-------|
| `cargo test -p service_impl booking_information_vfa` | PASS | 2 new tests green |
| `cargo test -p service_impl` | PASS | 494 tests total (492 existing + 2 new) |
| `cargo clippy --workspace -- -D warnings` | PASS | Clean |

## Known Stubs

None — the test file is complete; both assertions exercise the production code path directly with no stubs.

## Threat Flags

None — test-only file; no new network endpoints, auth paths, or schema changes.

## Self-Check

- [x] `service_impl/src/test/booking_information_vfa.rs` exists (created)
- [x] `service_impl/src/test/mod.rs` updated (module registered)
- [x] Commit `93b5b3f` exists
- [x] `vfa02_holiday_vs_absence_asymmetry` PASSES
- [x] `phase26_vfa_no_snapshot_bump` PASSES (version still 11)
- [x] `cargo test -p service_impl` — 494 tests, all pass
- [x] `cargo clippy --workspace -- -D warnings` — clean

## Self-Check: PASSED

---
*Phase: 26-freiwilligen-abwesenheit-cross-navigation*
*Completed: 2026-06-28*
