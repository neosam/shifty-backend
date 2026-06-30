---
phase: 34-feiertags-soll-schichtplan
plan: "01"
subsystem: backend-reporting
tags: [tdd, holiday, get_week, reporting, band-guard, hsp-01, hsp-02, hsp-03, hsp-04]
status: complete

dependency_graph:
  requires: []
  provides:
    - get_week-holiday-auto-credit
    - hol-03-regression-rebuilt
    - hsp-04-stichtag-gate-test
    - hsp-04-manual-wins-test
  affects:
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_holiday_auto_credit.rs

tech_stack:
  added: []
  patterns:
    - "4th injection point: build_derived_holiday_map per-employee per-week in get_week loop"
    - "Holiday-derived term separate from abense_hours_for_balance (HSP-03 band guard)"
    - "Clone per-employee extra hours slice from Arc<[&ExtraHours]> for manual-wins check"
    - "Dynamic-week guard: planned_hours <= 0.0 → holiday_derived_gated = 0.0"

key_files:
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_holiday_auto_credit.rs

decisions:
  - "D-34-01: holiday_derived_gated term reduces ONLY expected_hours, NOT dynamic_hours (HSP-03 band invariant)"
  - "D-34-02: Backend-only; no FE change, no i18n — WorkingHoursPerSalesPerson.holiday_hours already propagated"
  - "D-34-03: HOL-03 rebuilt in-place with positive assertions; 2 HSP-04 subtests added"
  - "D-34-04: CURRENT_SNAPSHOT_SCHEMA_VERSION stays 12 — get_week not read by billing_period_report.rs"
  - "Clone pattern: Vec<ExtraHours> via .map(|r| (*r).clone()) required because get_week collects extra_hours via .iter() yielding &ExtraHours references, not owned items"

metrics:
  duration: "10 minutes"
  completed_date: "2026-06-30"
  tasks_completed: 3
  files_modified: 2

requirements: [HSP-01, HSP-02, HSP-03, HSP-04]
---

# Phase 34 Plan 01: Feiertags-Soll im Schichtplan Summary

**One-liner:** Wired `build_derived_holiday_map` as 4th injection point in `get_week` — derived holidays reduce `expected_hours` (40→32) and fill `holiday_hours` (0→8) while `dynamic_hours` band stays 40h invariant (HSP-03).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | HOL-03 rebuild + 2 HSP-04 subtests | `4453767` | `reporting_holiday_auto_credit.rs` |
| 2 (GREEN) | 4th injection point in get_week | `712eda4` | `reporting.rs` |
| 3 | Snapshot version verified (stays 12) | — (verify-only, no code write) | `billing_period_report.rs` (read-only) |

## What Was Built

### Task 1 — RED: HOL-03 Rebuild + 2 HSP-04 Subtests

**File:** `service_impl/src/test/reporting_holiday_auto_credit.rs`

Rebuilt `test_holiday_auto_credit_no_year_view_impact` (HOL-03) in-place:
- Replaced panic-guard semantics (no expectations on special_day/toggle services) with positive assertions
- Added `special_day_service.expect_get_by_week()` mock: KW23/2024 Monday Holiday (2024-06-03)
- Overrode `toggle_service` with active cutoff `"2024-01-01"` (before holiday date → credit applies)
- Assertions: `dynamic_hours == 40.0` (HSP-03), `expected_hours == 32.0` (HSP-01), `holiday_hours == 8.0` (HSP-02)

Added `test_hsp04_before_cutoff`:
- Cutoff `"2024-12-31"` (after holiday 2024-06-03 → gate fires: `holiday_date < cutoff → skip`)
- Asserts: `expected_hours == 40.0`, `holiday_hours == 0.0`, `dynamic_hours == 40.0`
- Passes both before and after fix (regression guard: correct even without auto-credit)

Added `test_hsp04_manual_wins`:
- `find_by_week` returns `make_holiday_extra_hours(8.0, date!(2024-06-03))` + SpecialDay KW23 Monday
- Asserts: `holiday_hours == 8.0` (manual wins, NOT 16.0 double-count)
- Passes before fix (manual ExtraHours already sourced in get_week) and after (manual-wins built into `build_derived_holiday_map`)

RED state: HOL-03 failed with `"HSP-01: expected_hours must be 32h, got 40"` — `get_week` lacked the injection.

### Task 2 — GREEN: 4th Injection Point in `get_week`

**File:** `service_impl/src/reporting.rs` (~line 1067)

Added between `absence_derived_balance_total` binding and original `expected_hours` binding:

```rust
// Per-employee extra hours for manual-wins conflict check in build_derived_holiday_map.
// Type note: extra_hours in get_week collects via .iter() (yields &ExtraHours),
// so the per-employee bucket is Arc<[&ExtraHours]>. Clone to &[ExtraHours].
let employee_extra_hours_owned: Vec<ExtraHours> = employee_extra_hours
    .map(|arc| arc.iter().map(|r| (*r).clone()).collect())
    .unwrap_or_default();
let derived_holiday_map = self
    .build_derived_holiday_map(
        ShiftyWeek::new(year, week).as_date(DayOfWeek::Monday),
        ShiftyWeek::new(year, week).as_date(DayOfWeek::Sunday),
        &working_hours,
        &employee_extra_hours_owned,
        context.clone(),
    )
    .await?;
let derived_holiday_for_week: f32 = derived_holiday_map.values().sum();
// Dynamic-week guard (D-34-01(d)): no negative expected for 0.0-planned weeks.
let holiday_derived_gated =
    if !has_contract_row || planned_hours <= 0.0 { 0.0f32 } else { derived_holiday_for_week };
// HSP-02: holiday_hours shadowed to include derived contribution.
let holiday_hours = holiday_hours + holiday_derived_gated;
// HSP-01: expected_hours reduced by derived holiday.
// CRITICAL HSP-03: holiday_derived_gated NOT applied to dynamic_hours.
let expected_hours = planned_hours - abense_hours_for_balance - absence_derived_balance_total - holiday_derived_gated;
```

The `dynamic_hours` formula (line 1116) was NOT changed:
```rust
let dynamic_hours = dynamic_hours - abense_hours_for_balance - absence_derived_balance_total;
// holiday_derived_gated deliberately absent — HSP-03 band guard
```

`booking_information.rs` was NOT touched (propagation is automatic: `available_hours = report.expected_hours`, `holiday_hours = report.holiday_hours`).

**Key deviation from PATTERNS.md:** PATTERNS.md assumed `employee_extra_hours: Option<&Arc<[ExtraHours]>>`, but in `get_week` it is `Option<&Arc<[&ExtraHours]>>` (because `collect_to_hash_map_by` is applied after `.iter()` which yields references). Required cloning via `.map(|r| (*r).clone())` — deviation Rule 1 (compile error / type mismatch auto-fixed). ExtraHours implements Clone.

### Task 3 — Snapshot Verification

Grep evidence:
- `grep -c 'get_report_for_employee_range' billing_period_report.rs` → 5 matches (4 call sites + 1 comment referencing get_week as a separate path)
- Line 368: comment only, no actual `get_week` or `build_derived_holiday_map` call
- `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12` — unchanged
- D-34-04 confirmed: no bump needed

## Test Results

All 7 reporting_holiday tests green after Task 2:
- `test_holiday_auto_credit_no_year_view_impact` (HOL-03): `expected_hours=32.0`, `holiday_hours=8.0`, `dynamic_hours=40.0`
- `test_hsp04_before_cutoff`: `expected_hours=40.0`, `holiday_hours=0.0`, `dynamic_hours=40.0`
- `test_hsp04_manual_wins`: `holiday_hours=8.0` (not 16.0)
- `test_holiday_auto_credit_basic` (HOL-01): still green
- `test_holiday_auto_credit_equivalence` (HOL-02): still green
- `test_holiday_before_cutoff_skipped` (HCFG-01): still green
- `test_holiday_manual_wins` (HCFG-03): still green

service_impl: 517 tests, 0 failures.

Full workspace: 63 passed, 1 pre-existing failure (see Deferred Issues).

`cargo clippy --workspace -- -D warnings`: clean.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ExtraHours type mismatch in get_week context**
- **Found during:** Task 2 (GREEN — compile error)
- **Issue:** PATTERNS.md's injection template used `employee_extra_hours.map(|v| v.as_ref()).unwrap_or(&[])` assuming `Option<&Arc<[ExtraHours]>>`. In `get_week`, `collect_to_hash_map_by` is applied after `.iter()` which yields `&ExtraHours`, so the actual type is `Option<&Arc<[&ExtraHours]>>`. Passing that to `build_derived_holiday_map(&[ExtraHours])` was rejected by rustc with `E0308`.
- **Fix:** Collect via `.map(|arc| arc.iter().map(|r| (*r).clone()).collect()).unwrap_or_default()` into `Vec<ExtraHours>`. ExtraHours: Clone. Negligible overhead (per-employee slice, typically ≤20 entries/week).
- **Files modified:** `service_impl/src/reporting.rs`
- **Commit:** `712eda4`

## Deferred Issues

| Issue | File | Reason |
|-------|------|--------|
| `test_seed_twice_is_additive` fails with `ValidationError([Duplicate])` | `shifty_bin/src/integration_test/dev_seed.rs:94` | Pre-existing failure; `seed_dev_data_impl` is not idempotent (introduced commit `8ead369`, March 2026). Not caused by Phase 34 changes. |

## Known Stubs

None. All implemented functionality is complete and wired.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes introduced. Internal report computation only.

## Self-Check: PASSED

- [x] `service_impl/src/reporting.rs` modified with 4th injection point
- [x] `service_impl/src/test/reporting_holiday_auto_credit.rs` rebuilt HOL-03 + 2 new tests
- [x] Task 1 commit `4453767` exists
- [x] Task 2 commit `712eda4` exists
- [x] `cargo test -p service_impl reporting_holiday` — 7/7 green
- [x] `cargo clippy --workspace -- -D warnings` — clean
- [x] `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 12 (D-34-04)
- [x] `dynamic_hours` formula unchanged (HSP-03 band guard)
- [x] `booking_information.rs` untouched (D-34-01)
