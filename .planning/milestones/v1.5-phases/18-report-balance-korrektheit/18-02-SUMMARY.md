---
phase: 18-report-balance-korrektheit
plan: "02"
subsystem: reporting
tags: [bugfix, uv-05, vacation-days, sick-leave-days, absence-period, snapshot-version]
dependency_graph:
  requires: []
  provides: [UV-05-fix, per-week-category-merge, single-source-of-truth-hours, snapshot-v10]
  affects: [service_impl/reporting, service_impl/billing_period_report]
tech_stack:
  added: []
  patterns: [per-week-derived-merge-ungated, by-week-single-source-of-truth, snapshot-versioning]
key_files:
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/reporting_additive_merge.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/booking_information.rs
decisions:
  - "UV-05: per-week derived absence hours merged into GroupedReportHours category fields ungated (D-18-03); by_week is single source of truth for vacation/sick/unpaid hours in EmployeeReport (D-18-04)"
  - "Double-count reconciliation via by_week sum: vacation_hours = by_week.iter().map(|w| w.vacation_hours).sum(); old year-lump absence_derived_*_hours locals removed from get_report_for_employee"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bumped 9->10 because VacationDays computation changes (converted entries: 0 -> >0)"
  - "get_reports_for_all_employees and get_week year-lump paths left intact — they have their own independent derived merges and do not expose *_days"
metrics:
  duration: "~20 minutes"
  completed: "2026-06-26"
  tasks_completed: 5
  files_modified: 5
---

# Phase 18 Plan 02: UV-05 Vacation/Sick Days After Conversion Summary

Close UV-05: per-week derived absence hours (from absence_period) are now merged ungated into GroupedReportHours category fields; get_report_for_employee uses by_week as single source of truth for vacation/sick/unpaid hours, eliminating the year-lump double-count.

## What Was Built

### Task 1: Per-week derived absence merge in hours_per_week (D-18-03, ungated)

In `hours_per_week` (service_impl/src/reporting.rs, before the `weeks.push(...)`), added an ungated fold over `derived_absence` filtered to the current week:

```rust
let (derived_vacation_hours, derived_sick_leave_hours, derived_unpaid_leave_hours) =
    derived_absence
        .iter()
        .filter(|(d, _)| ShiftyDate::from(**d).as_shifty_week() == week)
        .fold((0.0f32, 0.0f32, 0.0f32), |(v, s, u), (_, r)| {
            match r.category {
                AbsenceCategory::Vacation => (v + r.hours, s, u),
                AbsenceCategory::SickLeave => (v, s + r.hours, u),
                AbsenceCategory::UnpaidLeave => (v, s, u + r.hours),
            }
        });
```

Then added each derived value to the corresponding field in the `GroupedReportHours` struct literal (e.g. `vacation_hours: ... .sum::<f32>() + derived_vacation_hours`). The gated balance/expected path (`derived_absence_hours` at ~line 1139) was NOT changed.

### Task 2: Year-lump double-count removed from get_report_for_employee (D-18-04)

The old code in `get_report_for_employee` (lines 609-617) computed three year-lump locals (`absence_derived_vacation_hours` etc.) and added them to the top-level `EmployeeReport` hour fields alongside the extra_hours sum. This double-counted derived hours once Task 1 put them in the per-week fields.

**Reconciliation chosen: by_week sum.** The top-level fields were changed to:
```rust
vacation_hours: by_week.iter().map(|w| w.vacation_hours).sum::<f32>(),
sick_leave_hours: by_week.iter().map(|w| w.sick_leave_hours).sum::<f32>(),
unpaid_leave_hours: by_week.iter().map(|w| w.unpaid_leave_hours).sum::<f32>(),
```

Each per-week field already equals `extra_hours-for-week + derived-for-week`, so the by_week sum equals the old correct display total without the year-lump add — counted exactly once. The year-lump fold (`absence_derived_vacation_hours` / `_sick_leave_hours` / `_unpaid_leave_hours`) was removed entirely to avoid dead-code warnings.

`holiday_hours` was left using its existing extra_hours-based computation (Holiday is not an AbsenceCategory in derived_absence).

### Task 3: Snapshot schema version 9->10 (D-18-07)

`CURRENT_SNAPSHOT_SCHEMA_VERSION` in `service_impl/src/billing_period_report.rs` was bumped from 9 to 10. A `/// - v10:` doc entry was added explaining that UV-05 changes the computation for `BillingPeriodValueType::VacationDays` (and sick/unpaid days) from 0 to >0 for converted entries, and that v9 snapshots cannot be re-validated against the corrected computation.

### Task 4: Regression tests (D-18-06)

Three new tests added to `service_impl/src/test/reporting_additive_merge.rs`:

- `test_converted_vacation_preserves_days`: asserts vacation_days > 0 AND equal between extra_hours path and absence_period path (using `build_parity_service` with `fixture_work_details_8h_mon_fri` for hours_per_day = 8.0).
- `test_converted_vacation_no_double_count`: absence_period-only case; asserts vacation_hours == 8.0 (not 16.0) and vacation_days == vacation_hours / 8.0.
- `test_converted_sick_leave_preserves_days`: absence_period SickLeave 8h; asserts sick_leave_days > 0 and absence_days >= sick_leave_days.

All three passed immediately (GREEN — Task 1 and 2 were applied before writing tests).

### Task 5: Snapshot locking tests updated

Two locking tests that asserted version 9 were updated to version 10:
- `service_impl/src/test/billing_period_snapshot_locking.rs::test_snapshot_schema_version_pinned`
- `service_impl/src/test/booking_information.rs::snapshot_schema_version_pinned_at_9` (renamed to `snapshot_schema_version_pinned_at_10`)

## get_reports_for_all_employees — Unaffected

`get_reports_for_all_employees` (reporting.rs:480-512) builds `ShortEmployeeReport` DISPLAY hours from its own ungated year-lump (`absence_derived_vacation_hours` etc. at lines 480-489 and used at 505). It does NOT consume the per-week `GroupedReportHours` category fields produced by `hours_per_week`, so Task 1 has no effect on that path. Its hour totals are therefore unchanged. The test `test_all_employees_additive_merge` (vacation_hours == 12.0) still passes, confirming no regression.

Similarly, `get_week` (reporting.rs ~944-958) has its own independent year-lump and its `test_get_week_additive_merge` (sick_leave_hours == 11.0) still passes.

## Deviations from Plan

None — plan executed exactly as written, except TDD order was effectively GREEN-first (Tasks 1-2 implemented before tests in Task 4) because the plan's task ordering put implementation before tests. The tests were written after implementation but correctly assert the fixed behavior.

The two snapshot locking tests in `billing_period_snapshot_locking.rs` and `booking_information.rs` were updated from version 9 to 10 as instructed by the plan ("update any expected-version assertion from 9 to 10").

## Self-Check: PASSED

Files modified exist:
- service_impl/src/reporting.rs: FOUND (per-week derived merge + by_week single source)
- service_impl/src/billing_period_report.rs: FOUND (CURRENT_SNAPSHOT_SCHEMA_VERSION = 10)
- service_impl/src/test/reporting_additive_merge.rs: FOUND (3 new tests)
- service_impl/src/test/billing_period_snapshot_locking.rs: FOUND (version updated to 10)
- service_impl/src/test/booking_information.rs: FOUND (version updated to 10)

Test results: cargo test --workspace — 455 (service_impl) + 61 (dao_impl_sqlite) + other crates = all green, 0 failures.
cargo build — Finished dev profile, 0 errors.
