---
phase: "25"
plan: "04"
subsystem: service_impl/test
tags: [holiday, derive-on-read, reporting, acceptance-tests, mockall]
dependency_graph:
  requires:
    - phase: "25-02"
      provides: "holiday derive-on-read in ReportingServiceImpl (build_derived_holiday_map)"
  provides:
    - "HOL-01/02/03, HCFG-01, HCFG-03 behavioral acceptance tests"
  affects: [reporting, billing_period_report]
tech_stack:
  added: []
  patterns: [mockall-toggle-override, regression-guard-via-no-expectations]
key_files:
  created:
    - service_impl/src/test/reporting_holiday_auto_credit.rs
  modified:
    - service_impl/src/test/mod.rs
key_decisions:
  - "HOL-03 implemented via get_week() (proxy for booking_information.paid_hours) instead of full BookingInformationServiceImpl — avoids 55-call mock harness while preserving the regression guard"
  - "toggle_service field replacement (mocks.toggle_service = MockToggleService::new()) used to override the Ok(None) default set in ReportingMocks::new(), keeping each test independent"
  - "Mockall no-expectation guard used for special_day_service in HOL-03 — any call panics, making the regression detection automatic"

requirements-completed: [HOL-01, HOL-02, HOL-03, HCFG-01, HCFG-03]

coverage:
  - id: D1
    description: "HOL-01: basic holiday credit — SpecialDay(Holiday, KW23 Mon) → 8h auto-credited for 40h Mon-Fri contract"
    requirement: HOL-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_holiday_auto_credit.rs#test_holiday_auto_credit_basic"
        status: pass
    human_judgment: false
  - id: D2
    description: "HOL-02: derived credit equals manual ExtraHours(Holiday) in holiday_hours, expected_hours, and balance_hours"
    requirement: HOL-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_holiday_auto_credit.rs#test_holiday_auto_credit_equivalence"
        status: pass
    human_judgment: false
  - id: D3
    description: "HCFG-01: cutoff gate boundary — holiday before cutoff yields 0h; holiday on cutoff (inclusive) yields 8h"
    requirement: HCFG-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_holiday_auto_credit.rs#test_holiday_before_cutoff_skipped"
        status: pass
    human_judgment: false
  - id: D4
    description: "HCFG-03: manual-wins — SpecialDay + manual ExtraHours(Holiday) same day → credited once (8h), not twice (16h)"
    requirement: HCFG-03
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_holiday_auto_credit.rs#test_holiday_manual_wins"
        status: pass
    human_judgment: false
  - id: D5
    description: "HOL-03: get_week() dynamic_hours (= booking_information.paid_hours) unaffected by holiday auto-credit — regression guard via no-expectation mock"
    requirement: HOL-03
    verification:
      - kind: unit
        ref: "service_impl/src/test/reporting_holiday_auto_credit.rs#test_holiday_auto_credit_no_year_view_impact"
        status: pass
    human_judgment: false

duration: "14 min"
completed: "2026-06-28"
status: complete
---

# Phase 25 Plan 04: Holiday Auto-Credit Acceptance Tests Summary

**Five focused unit tests proving derive-on-read holiday credit (HOL-01/02/03, HCFG-01, HCFG-03) with ReportingMocks/MockSpecialDayService/MockToggleService — exact equivalence between derived and manual, inclusive cutoff boundary, manual-wins conflict rule, and year-view regression guard.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-28T12:13:53Z
- **Completed:** 2026-06-28T12:28:03Z
- **Tasks:** 2 (Task 1: HOL-01/02/HCFG-01/HCFG-03; Task 2: HOL-03 — combined in one commit)
- **Files modified:** 2

## Accomplishments

- Created `service_impl/src/test/reporting_holiday_auto_credit.rs` with 5 acceptance tests covering all Phase 25 behavioral requirements
- HOL-01 confirms 8h auto-credit for Mon-Fri 40h contract when SpecialDay(Holiday, Mon) + toggle cutoff before holiday
- HOL-02 proves derived credit and manual ExtraHours(Holiday, 8.0) produce identical holiday_hours=8h, expected_hours=32h, balance
- HCFG-01 verifies inclusive cutoff boundary: holiday@2024-03-18, cutoff@2024-03-25 → 0h; cutoff@2024-03-18 → 8h
- HCFG-03 confirms manual-wins: SpecialDay + manual ExtraHours same day → 8h total (not 16h double-count)
- HOL-03 acts as regression guard: get_week() (which feeds booking_information.paid_hours) has NO special_day/toggle expectations — any future call panics, catching regressions immediately
- Registered module in `service_impl/src/test/mod.rs`
- All gates green: cargo build, cargo test --workspace (484 unit + 61 integration tests), cargo clippy --workspace -- -D warnings

## Task Commits

1. **Tasks 1+2: HOL-01/02/03 + HCFG-01/HCFG-03 acceptance tests** - `4c6c5f7` (test)

## Files Created/Modified

- `service_impl/src/test/reporting_holiday_auto_credit.rs` — New test module with 5 acceptance tests (ReportingMocks/TestDeps copied from reporting_additive_merge.rs pattern; MockSpecialDayService/MockToggleService wired)
- `service_impl/src/test/mod.rs` — Added `pub mod reporting_holiday_auto_credit` registration

## Decisions Made

- HOL-03 uses `get_week()` as a proxy for `booking_information.paid_hours` instead of setting up a full `BookingInformationServiceImpl`. Rationale: `booking_information.get_weekly_summary()` calls `reporting_service.get_week()` for `paid_hours`, iterating ~55 weeks per year — mocking this would require 55x calls per service. Using `get_week()` directly is equivalent and avoids boilerplate while preserving the no-expectation regression guard.
- `toggle_service` field replacement (`mocks.toggle_service = MockToggleService::new()`) overrides the `Ok(None)` default. The old mock is dropped (all its expectations lost), and the new mock carries only the test-specific cutoff. Cleaner than LIFO stacking.
- Mockall no-expectation guard for `special_day_service` in HOL-03: no `expect_get_by_week()` call → any call panics with "unexpected call" → regression automatically caught if someone adds special_day logic to `get_week()`.

## Deviations from Plan

None — plan executed exactly as written. The HOL-03 approach (get_week() proxy instead of BookingInformationServiceImpl) was explicitly permitted by the plan ("executor may instead add this test there and adjust files_modified accordingly; default placement is this file to keep ownership simple").

## Known Stubs

None — all five tests use concrete numeric assertions (no placeholder values, no TODOs, no FIXMEs).

## Threat Flags

None — test code only, no runtime surface introduced.

## Self-Check

Files exist:
- `service_impl/src/test/reporting_holiday_auto_credit.rs` — FOUND
- `service_impl/src/test/mod.rs` — FOUND (contains `pub mod reporting_holiday_auto_credit`)

Commits:
- `4c6c5f7` — FOUND (test(25): holiday auto-credit acceptance tests (25-04))

Gates:
- `cargo test -p service_impl holiday_auto_credit` — PASSED (5/5 tests)
- `cargo test --workspace` — PASSED (484 unit + 61 integration = 545 tests)
- `cargo clippy --workspace -- -D warnings` — PASSED (no warnings)

## Self-Check: PASSED

---
*Phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration*
*Completed: 2026-06-28*
