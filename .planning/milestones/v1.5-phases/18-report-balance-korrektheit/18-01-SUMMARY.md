---
phase: 18-report-balance-korrektheit
plan: "01"
subsystem: backend
tags: [vacation-balance, carryover, regression-test, UV-04]
dependency_graph:
  requires: []
  provides: [UV-04-pinned]
  affects: [service_impl/vacation_balance, service_impl/test/vacation_balance]
tech_stack:
  added: []
  patterns: [mockall-withf-argument-pinning]
key_files:
  created: []
  modified:
    - service_impl/src/vacation_balance.rs
    - service_impl/src/test/vacation_balance.rs
decisions:
  - "D-18-01: compute_balance was ALREADY reading year-1 (fix pre-existed) — Task 1 was a no-op confirming presence"
  - "D-18-02: Added carryover_read_uses_prior_year with withf matcher pinning year-1; negative flip to year confirmed test fails"
metrics:
  duration: ~10min
  completed: 2026-06-25
---

# Phase 18 Plan 01: UV-04 Vacation-Balance Carryover Year-1 Fix Summary

One-liner: Verified year-1 carryover read in compute_balance and added regression test with withf matcher pinning the argument against reversion.

## Tasks Completed

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1 | Verify compute_balance reads year-1 | No-op confirm | Fix already present at vacation_balance.rs:249 |
| 2 | Regression test carryover_read_uses_prior_year | Added | withf pins year == TEST_YEAR - 1; negative check passed |
| 3 | Workspace test + build gate | Green | cargo test --workspace + cargo build both 0 |

## Task 1: Was the year-1 read already present?

YES — no code change was needed. `service_impl/src/vacation_balance.rs` line 249 already contains:

```rust
.get_carryover(sales_person_id, year - 1, Authentication::Full, Some(tx))
```

This matches `reporting.rs:662-672` which uses `from_date.year() - 1`. Task 1 was a no-op confirmation.

## Task 2: Regression test

Added `carryover_read_uses_prior_year` to `service_impl/src/test/vacation_balance.rs` (after the existing carryover tests at line 849+). The test:

- Uses `.withf(|_sp, year, _auth, _tx| *year == TEST_YEAR - 1)` to pin the year argument.
- Expects `carryover_days == 7` from the mock return value.
- Includes a doc comment referencing `reporting.rs:662-672` per D-18-02.

Negative check: temporarily flipping production code to `year` (without `- 1`) caused the test to fail with "MockCarryoverService::get_carryover: No matching expectation found" — confirming the matcher catches the regression.

## Deviations from Plan

None — plan executed exactly as written. The existing `get_carryover_is_called_with_previous_year` test (already using `.with(mockall::predicate::eq(TEST_YEAR - 1), ...)`) was kept intact; `carryover_read_uses_prior_year` was added as an additional test using the `withf` style explicitly requested by the plan.

## Test Commands Run

```
cargo test -p service_impl carryover_read_uses_prior_year  -> 1 passed
cargo test --workspace                                      -> all passed (452 service_impl tests, 0 failures)
cargo build                                                 -> Finished dev profile
```

## Self-Check: PASSED

- `service_impl/src/test/vacation_balance.rs` contains `carryover_read_uses_prior_year`: confirmed
- `service_impl/src/vacation_balance.rs` contains `year - 1` at line 249: confirmed
- `cargo test --workspace`: green
- `cargo build`: green
