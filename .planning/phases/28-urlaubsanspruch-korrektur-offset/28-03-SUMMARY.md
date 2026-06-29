---
phase: 28-urlaubsanspruch-korrektur-offset
plan: 03
subsystem: vacation-entitlement / billing-snapshot
tags: [bugfix, off-by-one, snapshot-versioning, vacation-entitlement]
requires:
  - service::employee_work_details::EmployeeWorkDetails::vacation_days_for_year (pre-existing)
  - service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION (pre-existing)
provides:
  - corrected year-start vacation proration (subtracts days strictly before start)
  - CURRENT_SNAPSHOT_SCHEMA_VERSION = 12
affects:
  - BillingPeriodValueType::VacationEntitlement (recomputed value changes)
tech-stack:
  added: []
  patterns: [snapshot-schema-versioning, tdd-regression-tests]
key-files:
  created: []
  modified:
    - service/src/employee_work_details.rs
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/booking_information.rs
    - service_impl/src/test/booking_information_vfa.rs
decisions:
  - "D-28-04: fix ONLY the year-START branch (ordinal-1); year-END branch already correct, left untouched"
  - "D-28-05: bump snapshot schema version 11->12 because VacationEntitlement (not VacationDays) computation changed"
metrics:
  duration_minutes: 9
  completed: 2026-06-29
  tasks: 2
  files: 5
status: complete
---

# Phase 28 Plan 03: Urlaubsanspruch Off-by-One Fix + Snapshot Bump Summary

One-line: Fixed the year-start vacation-entitlement proration off-by-one in `vacation_days_for_year` (a 1.1. start now subtracts 0 instead of ~1/365 of the annual entitlement) and carried the mandatory `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump 11→12 it triggers via the persisted `BillingPeriodValueType::VacationEntitlement`.

## What Was Built

### Task 1 — Off-by-one proration fix + regression tests
- `service/src/employee_work_details.rs:171-181`: changed the year-START proration ratio from `ordinal() / days_in_year` to `(ordinal() - 1) / days_in_year` — the days STRICTLY before the contract start. A 1.1. start (`ordinal() == 1`) now subtracts 0. The year-END branch (`1.0 - ordinal/days_in_year`) was left UNCHANGED (RESEARCH Pitfall 4: already correct/symmetric).
- Added a `#[cfg(test)] mod vacation_days_for_year_tests` with 5 regression tests:
  - `full_year_contract_no_proration` (1.1.–31.12. → exactly `vacation_days`, no proration — the core off-by-one regression)
  - `mid_year_start_subtracts_prior_days` (1.7. start → subtracts `(ordinal-1)/days_in_year`)
  - `year_end_on_dec_31_subtracts_zero` (pins the year-END branch to prevent a future "symmetry" regression)
  - `out_of_range_returns_zero` (year outside `[from_year, to_year]` → 0.0, unchanged)
  - `single_year_both_bounds_prorate` (mid-year start AND mid-year end in one year)
- TDD: tests written first, confirmed RED (4 of 5 failed on the buggy code: full-year returned 17.95, mid-year 9.0247 vs 9.074), then GREEN after the one-line fix.

### Task 2 — Snapshot schema-version bump 11→12 + guard
- `service_impl/src/billing_period_report.rs:108`: `CURRENT_SNAPSHOT_SCHEMA_VERSION` 11 → **12**, with a new `- v12:` doc-comment history entry naming `VacationEntitlement` (reporting.rs:853 ← :803) and explicitly noting `VacationDays` (taken vacation) is UNAFFECTED.
- `service_impl/src/test/billing_period_snapshot_locking.rs`: `test_snapshot_schema_version_pinned` now asserts `== 12` with the Phase 28 VAC-OFFSET-01 rationale (citing VacationEntitlement); module-head doc bullet updated. `test_billing_period_value_type_surface_locked` left unchanged (no enum variant added).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Two additional version-pin guard tests required updating**
- **Found during:** Task 2 (`cargo test --workspace` reconciliation, anticipated by the prompt's `<expected_test_churn>`)
- **Issue:** Two pre-existing "no-bump" regression guards assert the ABSOLUTE value of `CURRENT_SNAPSHOT_SCHEMA_VERSION`, so they fail once the constant moves:
  - `service_impl/src/test/booking_information.rs` `snapshot_schema_version_pinned_at_10()` — asserted `== 11`
  - `service_impl/src/test/booking_information_vfa.rs` `phase26_vfa_no_snapshot_bump()` — asserted `== 11`
- **Fix:** Updated both literals 11 → 12 (these legitimately track the buggy/old value) and extended their rationale comments to record that Phase 28 (not Phase 15/25/26) is responsible for the bump. Their guard intent (catch an *accidental* bump from their own phase) is preserved. Also corrected two stale module-head doc comments ("is 8" / "remains 11").
- **Files modified:** `service_impl/src/test/booking_information.rs`, `service_impl/src/test/booking_information_vfa.rs`
- **Commit:** 568e2053

No reporting/billing VALUE tests regressed — existing reporting fixtures use entitlement values that the ~1/365 correction did not tip across an asserted boundary, so only the absolute-version-pin guards needed touching.

## Pre-existing Tests Updated

| Test | Old → New | Why |
|------|-----------|-----|
| `booking_information::snapshot_schema_version_pinned_at_10` | `assert == 11` → `assert == 12` | Pins the absolute constant; Phase 28 legitimately bumped it. Intent (no-bump-from-Phase-15/25) preserved via comment. |
| `booking_information_vfa::phase26_vfa_no_snapshot_bump` | `assert == 11` → `assert == 12` | Pins the absolute constant; Phase 28 bump is unrelated to VFA. Guard still catches accidental Phase-26 bumps. |
| `billing_period_snapshot_locking::test_snapshot_schema_version_pinned` | `assert == 11` → `assert == 12` | The planned guard update — pins the new schema version with Phase 28 rationale. |

(The first two are the auto-fixed churn; the third is the planned change in Task 2.)

## Snapshot Version Confirmation

`CURRENT_SNAPSHOT_SCHEMA_VERSION` is **12** everywhere:
- Single source of truth: `service_impl/src/billing_period_report.rs:108` = `12` (writer at :390 sources it via the constant — no hardcoded literal).
- Guard `billing_period_snapshot_locking.rs` pins `== 12`.
- No-bump guards `booking_information.rs` / `booking_information_vfa.rs` re-pinned `== 12`.
- `grep -rn` confirms no stray hardcoded `11` schema-version pin remains.

## Gate Results

- `cargo build --workspace`: OK (Finished `dev` profile).
- `cargo test --workspace`: ALL GREEN — 0 failures. Key suites: `service` lib incl. 5 new vacation tests, `service_impl` 501 + 61 integration tests, all `test result: ok`.
- `cargo test -p service vacation_days_for_year`: 5 passed / 0 failed.
- `cargo test -p service_impl test_snapshot_schema_version_pinned`: 1 passed.
- `cargo clippy --workspace -- -D warnings`: clean (zero warnings).

## jj Commits

- `85171364` fix(28): correct year-start vacation proration off-by-one (VAC-OFFSET-01)
- `568e2053` fix(28): bump billing snapshot schema version 11->12 (D-28-05)

## Success Criteria

- [x] Year-start off-by-one fixed; full-year contract no longer prorated (D-28-04).
- [x] Snapshot version bumped 11→12, documented (v12 history entry), and guarded (D-28-05).
- [x] Year-end branch untouched; VacationDays untouched (only VacationEntitlement changes).

## Self-Check: PASSED

- service/src/employee_work_details.rs: FOUND
- service_impl/src/billing_period_report.rs: FOUND
- service_impl/src/test/billing_period_snapshot_locking.rs: FOUND
- Commit 85171364: FOUND
- Commit 568e2053: FOUND
