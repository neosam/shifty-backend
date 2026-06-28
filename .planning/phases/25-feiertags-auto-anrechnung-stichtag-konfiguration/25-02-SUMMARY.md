---
phase: "25"
plan: "02"
subsystem: service_impl/reporting
tags: [holiday, derive-on-read, reporting, snapshot]
dependency_graph:
  requires: [25-01]
  provides: [holiday-auto-credit-reporting]
  affects: [billing_period_report, reporting]
tech_stack:
  added: []
  patterns: [derive-on-read, dual-write-holiday-absense]
key_files:
  created: []
  modified:
    - service_impl/src/reporting.rs
    - shifty_bin/src/main.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/booking_information.rs
    - service_impl/src/test/reporting_cap_overflow.rs
    - service_impl/src/test/reporting_additive_merge.rs
    - service_impl/src/test/reporting_no_contract_volunteer.rs
decisions:
  - "Derive-on-read: holiday hours computed in build_derived_holiday_map at report time; no ExtraHours rows written"
  - "Treat Unauthorized from ToggleService as automation-off (empty map) so integration tests with mock-auth work"
  - "Dual write: derived holiday added to both holiday_hours AND absense_hours (reduces expected_hours in balance)"
  - "Three injection points: hours_per_week fn, get_reports_for_all_employees, get_report_for_employee_range"
metrics:
  duration: "~2 sessions (resumed)"
  completed: "2026-06-28"
  tasks_completed: 3
  files_modified: 8
status: complete
---

# Phase 25 Plan 02: Derive-on-read Holiday Auto-Credit in ReportingService Summary

Holiday hours are now credited automatically at report-read time by consulting the `holiday_auto_credit` toggle value (an ISO cutoff date) and the `SpecialDay` table — no ExtraHours rows written, purely compute-on-read.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Wire SpecialDayService+ToggleService into ReportingServiceImpl; reorder main.rs DI; thread derived_holiday param | 1423c4a |
| 2 | build_derived_holiday_map + three injection points (dual-write holiday_hours + absense_hours) | f6783f1 |
| 3 | Bump CURRENT_SNAPSHOT_SCHEMA_VERSION 10→11 + update all pinned-version tests | edb6072 |

## What Was Built

`ReportingServiceImpl` now has two new dependencies: `SpecialDayService` and `ToggleService`. A private `build_derived_holiday_map` method:

1. Reads the `holiday_auto_credit` toggle value (ISO date string) — returns empty map if unset or Unauthorized.
2. Iterates every ISO week in the requested date range via `ShiftyWeek::iter_until`.
3. Fetches `SpecialDay` entries per week; filters to `SpecialDayType::Holiday`.
4. Computes the concrete calendar date using `time::Date::from_iso_week_date` (year-boundary safe).
5. Gates on `holiday_date >= cutoff` (HCFG-01).
6. Skips if a manual `ExtraHours(Holiday)` already covers the employee+day (HCFG-03 conflict skip).
7. Credits `wh.holiday_hours()` if the employee's contract covers that weekday.

The derived map is injected at all three reporting paths:
- `hours_per_week` free function: sums derived hours for the week, adds to both `absence_hours` and `holiday_hours` in `GroupedReportHours`.
- `get_reports_for_all_employees`: per-employee map computed before the fold, added to `holiday_hours` + `absense_hours`.
- `get_report_for_employee_range`: map computed and passed to `hours_per_week`; `EmployeeReport.holiday_hours` summed from per-week breakdown.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ToggleService Unauthorized error propagated to integration tests**
- Found during: Task 2 test run
- Issue: `build_derived_holiday_map` propagated `Unauthorized` from `ToggleServiceImpl.get_toggle_value` when integration tests use mock-auth (no real user ID). This broke 7 integration tests in `shifty_bin`.
- Fix: Catch `ServiceError::Unauthorized` from `get_toggle_value` and return empty map (automation off) rather than propagating.
- Files modified: `service_impl/src/reporting.rs`
- Commit: f6783f1

**2. [Rule 1 - Bug] booking_information test also pins version at 10**
- Found during: Task 3 test run (`snapshot_schema_version_pinned_at_10` in booking_information.rs)
- Issue: A second pinned-version test in `service_impl/src/test/booking_information.rs` (not listed in the plan) also asserted `== 10`; failed after bumping `CURRENT_SNAPSHOT_SCHEMA_VERSION`.
- Fix: Updated assert to `== 11` with v11 rationale comment.
- Files modified: `service_impl/src/test/booking_information.rs`
- Commit: edb6072

**3. [Rule 2 - Missing critical functionality] Test files missing new mock deps**
- Found during: Task 2 test run
- Issue: Three test files (`reporting_cap_overflow.rs`, `reporting_additive_merge.rs`, `reporting_no_contract_volunteer.rs`) had `TestDeps` impls and `ReportingServiceImpl` struct literals missing the new `SpecialDayService` and `ToggleService` fields.
- Fix: Added `MockSpecialDayService` and `MockToggleService` imports, `type` assignments in `TestDeps`, and mock instances returning `Ok(None)` (automation off by default) in all affected struct literals.
- Files modified: all three test files
- Commit: f6783f1

## Self-Check

Files exist:
- `service_impl/src/reporting.rs` — FOUND
- `service_impl/src/billing_period_report.rs` — FOUND (version = 11)
- `service_impl/src/test/billing_period_snapshot_locking.rs` — FOUND (asserts 11)

Commits:
- 1423c4a — FOUND (Task 1)
- f6783f1 — FOUND (Task 2)
- edb6072 — FOUND (Task 3)

`cargo test --workspace` — PASSED (all 61 integration tests + 479 unit tests)
`cargo clippy --workspace -- -D warnings` — PASSED (no warnings)

## Self-Check: PASSED
