---
phase: 24-paid-capacity-enforcement-config
plan: 02
subsystem: api
tags: [rust, service, enforcement, toggle, paid-capacity, shiftplanner, gate-fix]

# Dependency graph
requires:
  - phase: 24-01
    provides: ServiceError::PaidLimitExceeded { current, max } + HTTP 409 mapping + toggle seed
provides:
  - pre-persist hard-block guard in book_slot_with_conflict_check (non-shiftplanner over limit → PaidLimitExceeded)
  - ToggleService wired into ShiftplanEditService (D-24-08)
  - Shiftplanner ∨ self gate replacing HR ∨ self (D-24-04)
  - Hard-block tests (4 new) + migrated gate tests
affects:
  - 24-03 (frontend i18n — consumes the 409 contract confirmed here)
  - 24-05 (frontend block handler — matches HTTP 409 PaidLimitExceeded)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "pre-persist guard: if let Some(max) = slot.max_paid_employees → toggle check → shiftplanner bypass → count + booked_is_paid → prospective > max → Err(PaidLimitExceeded)"
    - "Shiftplanner ∨ self gate: join!(check_permission(SHIFTPLANNER_PRIVILEGE), verify_user_is_sales_person) → sp_perm.or(self_perm)?"
    - "MockToggleService in test deps with default is_enabled returning Ok(false) (soft) — hard-block tests checkpoint + override"

key-files:
  created: []
  modified:
    - service_impl/src/shiftplan_edit.rs
    - shifty_bin/src/main.rs
    - service_impl/src/test/shiftplan_edit.rs

key-decisions:
  - "pre-persist guard placed AFTER slot lookup, BEFORE booking_service.create — returning Err early means no transaction commit, no booking persisted (D-24-08)"
  - "Shiftplanner bypass checked via check_permission(...).is_ok() pattern (not join!) — consistent with sales_person_shiftplan.rs:84-88 analog"
  - "toggle_dao + toggle_service construction moved ABOVE shiftplan_edit_service in main.rs (Basic-before-Business DI rule; PATTERNS.md DI-ordering finding)"
  - "Default MockToggleService returns Ok(false) so all existing tests with slot limits remain unaffected without per-test toggle setup"

# Metrics
duration: 30min
completed: 2026-06-27
---

# Phase 24 Plan 02: Booking Enforcement — Hard-Block Guard + Gate Fix Summary

**ToggleService wired into ShiftplanEditService; pre-persist hard-block guard returns PaidLimitExceeded (409) for non-shiftplanners over paid limit; gate corrected from HR to Shiftplanner (D-24-04); 4 new hard-block tests + migrated gate tests; all gates green.**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-06-27T00:20:00Z
- **Completed:** 2026-06-27T00:50:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

### Task 1: Wire ToggleService dependency + reorder DI (D-24-08)
- Added `use service::toggle::ToggleService;` to imports in `service_impl/src/shiftplan_edit.rs`
- Added `ToggleService: service::toggle::ToggleService<...> = toggle_service` to `gen_service_impl!` block
- Replaced `HR_PRIVILEGE` import with `SHIFTPLANNER_PRIVILEGE` (gate fix comes in Task 2)
- Added `type ToggleService = ToggleService;` to `ShiftplanEditServiceDependencies` in `shifty_bin/src/main.rs`
- Added `toggle_service: toggle_service.clone()` to `ShiftplanEditServiceImpl { ... }` literal in `main.rs`
- MOVED `toggle_dao` + `toggle_service` construction from line ~1010 to ABOVE `shiftplan_edit_service` block (~line 907), satisfying Basic-before-Business DI ordering rule

### Task 2: Pre-persist hard-block guard + gate fix (D-24-02, D-24-04, D-24-08)
- Fixed booking gate from `HR ∨ self` to `Shiftplanner ∨ self` (renamed bindings `hr/sp` → `sp_perm/self_perm`, updated comment)
- Inserted hard-block guard after slot lookup, before `booking_service.create`:
  - Reads toggle fresh per booking via `toggle_service.is_enabled("paid_limit_hard_enforcement", Authentication::Full, ...)`
  - If hard: checks `permission_service.check_permission(SHIFTPLANNER_PRIVILEGE, context).is_ok()` for bypass
  - If non-shiftplanner: counts existing paid via `count_paid_bookings_in_slot_week`, fetches `get_all_paid` to check if booked person is paid, computes prospective count
  - If `prospective > max`: returns `Err(ServiceError::PaidLimitExceeded { current: prospective, max })` BEFORE commit
- Soft-warning block (`:529-548`) left intact for soft mode and shiftplanner-over-limit case
- `copy_week_with_conflict_check` untouched (deprecated)
- Updated doc comment in `count_paid_bookings_in_slot_week` to reflect new gate wording

### Task 3: Tests — hard-block scenarios + migrate gate comment (D-24-04 blast radius)
- Added `MockToggleService` to `ShiftplanEditDependencies` struct + `ShiftplanEditServiceDeps` impl + `build_service` literal
- Added default `expect_is_enabled() → Ok(false)` in `build_dependencies` (soft mode default — all existing tests unaffected)
- Updated `build_dependencies` parameter name/comment from `permission_grants_hr` to `permission_grants_shiftplanner` (D-24-04 gate rename)
- Updated `test_book_slot_with_conflict_check_forbidden` comment to reflect Shiftplanner ∨ self gate
- Added 4 new hard-block tests:
  1. `test_hard_block_non_shiftplanner_over_limit` — toggle ON + non-SP + paid over limit → `Err(PaidLimitExceeded { current: 3, max: 2 })`, `create` never called
  2. `test_hard_block_shiftplanner_bypasses` — toggle ON + SP → `Ok(...)` persists, soft warning fires
  3. `test_soft_mode_over_limit_warns_not_blocks` — toggle OFF → `Ok(...)` persists, `PaidEmployeeLimitExceeded` warning fires (D-24-01 regression)
  4. `test_hard_block_unpaid_never_blocked` — toggle ON + non-SP + unpaid person → `Ok(...)` persists, no block, no paid warning

## Task Commits

No commits made — per vcs_jj_only instruction, all changes left in working copy for manual jj commit by user.

## Files Created/Modified

- `service_impl/src/shiftplan_edit.rs` — ToggleService dep added, gate fixed (HR→Shiftplanner), pre-persist hard-block guard inserted, doc comment updated
- `shifty_bin/src/main.rs` — ToggleService type added to ShiftplanEditServiceDependencies, toggle_service wired into ShiftplanEditServiceImpl literal, toggle construction reordered above shiftplan_edit (Basic-before-Business)
- `service_impl/src/test/shiftplan_edit.rs` — MockToggleService added to deps struct/impl/build, 4 new hard-block tests, gate comment updated

## Decisions Made

- Used `check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()).is_ok()` pattern (not join!) for the shiftplanner bypass in the enforcement guard — mirrors the analog in `sales_person_shiftplan.rs:84-88` and avoids confusing the gate-check with the pre-existing gate join
- Pre-persist guard placed after slot lookup (where `slot.max_paid_employees` is in hand) and before `booking_service.create` — the early `return Err(...)` exits before `transaction_dao.commit`, ensuring no booking is persisted on block
- MockToggleService default returns `Ok(false)` in `build_dependencies`, so all existing paid-limit tests (using `slot_with_paid_limit`) continue to exercise the soft path without modification

## Deviations from Plan

None — plan executed exactly as written. All three tasks match their specifications and acceptance criteria.

## Issues Encountered

- First build attempt produced "unused imports" errors because Task 1 (add imports) was committed before Task 2 (use them). Resolved by proceeding directly to Task 2 in the same session, as expected.
- All three backend gates (cargo build, cargo test, cargo clippy -D warnings) passed on first attempt after completing all three tasks.

## Gate Results

- `cargo build --workspace` — PASSED
- `cargo test --workspace` — PASSED (475 unit + 61 integration + new hard-block tests = 536+ tests, 0 failed)
- `cargo clippy --workspace -- -D warnings` — PASSED (no warnings)

## Snapshot Schema Version

Not bumped. This plan does not add, remove, or change any persisted `billing_period_sales_person` value_types. The enforcement guard only affects the booking path — it returns an error or allows a booking to proceed; it does not touch the billing period snapshot schema. No `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump required.

## Known Stubs

None — this plan only modifies the booking service path and tests. No UI stubs or placeholder data flows.

## Threat Flags

No new threat surface introduced beyond what was in the plan's threat model (T-24-04 through T-24-07). The bypass gate uses the acting user's real `context` (not `Authentication::Full`), satisfying T-24-04 (non-shiftplanner cannot bypass). The new gate (T-24-05) is now `Shiftplanner ∨ self` — confirmed by `test_book_slot_with_conflict_check_forbidden` that non-SP non-self is still forbidden.

## Self-Check: PASSED

- [x] `service_impl/src/shiftplan_edit.rs` contains `SHIFTPLANNER_PRIVILEGE` (gate fix) — verified by grep
- [x] `service_impl/src/shiftplan_edit.rs` does NOT contain `HR_PRIVILEGE` — verified by grep returning empty
- [x] `service_impl/src/shiftplan_edit.rs` contains `paid_limit_hard_enforcement` (toggle key) — line 440
- [x] `service_impl/src/shiftplan_edit.rs` contains `PaidLimitExceeded` (error returned) — line 476
- [x] `service_impl/src/shiftplan_edit.rs` `copy_week_with_conflict_check` starts at line 617, no toggle/PaidLimitExceeded in that function
- [x] `shifty_bin/src/main.rs` contains `toggle_service:` in ShiftplanEditServiceImpl literal — verified
- [x] `shifty_bin/src/main.rs` toggle construction appears before shiftplan_edit construction — verified
- [x] `service_impl/src/test/shiftplan_edit.rs` contains `test_hard_block_non_shiftplanner_over_limit` — verified
- [x] `service_impl/src/test/shiftplan_edit.rs` contains `test_hard_block_shiftplanner_bypasses` — verified
- [x] `service_impl/src/test/shiftplan_edit.rs` contains `test_soft_mode_over_limit_warns_not_blocks` — verified
- [x] `service_impl/src/test/shiftplan_edit.rs` contains `test_hard_block_unpaid_never_blocked` — verified
- [x] `cargo build --workspace` — PASSED
- [x] `cargo test --workspace` — PASSED (19/19 shiftplan_edit tests, 0 failed total)
- [x] `cargo clippy --workspace -- -D warnings` — PASSED

---
*Phase: 24-paid-capacity-enforcement-config*
*Completed: 2026-06-27*
