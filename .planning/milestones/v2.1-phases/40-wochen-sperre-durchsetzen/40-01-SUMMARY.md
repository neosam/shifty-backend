---
phase: 40-wochen-sperre-durchsetzen
plan: 01
subsystem: api
tags: [rust, axum, service-layer, week-lock, http-423, dependency-injection, mockall]

# Dependency graph
requires:
  - phase: 39-kw-status-grundlage
    provides: WeekStatusService (get_week_status), WeekStatus enum (Unset/InPlanning/Planned/Locked)
provides:
  - "ServiceError::WeekLocked { year: u32, week: u8 } variant"
  - "HTTP 423 Locked mapping in rest error_handler (first 423 in codebase)"
  - "ShiftplanEditService::delete_booking trait method (get->assert->delete)"
  - "WeekStatusService wired as dep in ShiftplanEditServiceDeps + main.rs DI"
  - "pass-through assert_week_not_locked helper called at all 6 write heads"
  - "Test-harness (struct/impl/build_service/build_dependencies) updated for the new dep"
affects: [40-02, 40-03, 40-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Exhaustive error_handler match forces the compiler to enforce every new ServiceError arm"
    - "Basic-tier (WeekStatusService) as dep of Business-logic-tier (ShiftplanEditService), constructed Basic-before-Business in main.rs"
    - "Lock-gate helper reads status in the SAME transaction as the write (no-TOCTOU foundation for 40-03)"

key-files:
  created: []
  modified:
    - service/src/lib.rs
    - rest/src/lib.rs
    - service/src/shiftplan_edit.rs
    - service_impl/src/shiftplan_edit.rs
    - shifty_bin/src/main.rs
    - service_impl/src/test/shiftplan_edit.rs

key-decisions:
  - "assert_week_not_locked implemented as a private method on ShiftplanEditServiceImpl (needs self.week_status_service), not a free function"
  - "WeekStatusService dep uses the full <Context = Self::Context, Transaction = Self::Transaction> bound — no E0220/E0277 reduction needed (Open Question 1 resolved: full bound compiles)"
  - "Scaffold helper is deliberately pass-through: reads status (uses the dep, avoids dead-field lint) but always returns Ok — real enforcement + bypass land RED-first in 40-03"
  - "delete_booking handler re-routing deferred to 40-04 (rest/src/booking.rs untouched here)"

patterns-established:
  - "Pattern 1: pass-through gate scaffold — surface the cross-crate compile coupling green in one step so enforcement can be TDD-RED-first later"
  - "Pattern 2: gate call placed directly after check_permission, before first business logic, with context.clone()/tx.clone()"

requirements-completed: [WST-03, WST-04]

coverage:
  - id: D1
    description: "ServiceError::WeekLocked { year, week } variant + HTTP 423 Locked mapping in error_handler"
    requirement: "WST-03"
    verification:
      - kind: unit
        ref: "cargo build --workspace (exhaustive match enforces the 423 arm) + cargo clippy --workspace -- -D warnings"
        status: pass
    human_judgment: false
  - id: D2
    description: "ShiftplanEditService::delete_booking trait method + impl (get->assert_week_not_locked->delete order)"
    requirement: "WST-04"
    verification:
      - kind: unit
        ref: "cargo build --workspace; cargo test --workspace (existing shiftplan_edit harness compiles with the new method, 0 failed)"
        status: pass
    human_judgment: false
  - id: D3
    description: "WeekStatusService dep wired into ShiftplanEditServiceDeps + main.rs DI (Basic-before-Business ordering) + pass-through assert_week_not_locked at 6 write heads"
    requirement: "WST-03"
    verification:
      - kind: unit
        ref: "cargo test --workspace (MockWeekStatusService default get_week_status->Unset; existing non-bypass paths do not panic); cargo clippy --workspace -- -D warnings (no dead-field/dead-code)"
        status: pass
    human_judgment: false

# Metrics
duration: 18min
completed: 2026-07-02
status: complete
---

# Phase 40 Plan 01: Wochen-Sperre-Oberfläche (Scaffold) Summary

**Green-compiling backend surface for the week-lock: ServiceError::WeekLocked + HTTP 423 arm, delete_booking trait method, WeekStatusService wired into ShiftplanEditService, and a pass-through assert_week_not_locked helper at all 6 write heads — no enforcement yet (that is 40-03).**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-07-02
- **Completed:** 2026-07-02
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added `ServiceError::WeekLocked { year: u32, week: u8 }` with an #[error] message embedding year/week, plus the first HTTP 423 Locked arm in the codebase's exhaustive `error_handler`.
- Added `ShiftplanEditService::delete_booking` (trait + impl) with the mandatory get → assert_week_not_locked → delete order (mockall auto-generates the mock via #[automock]).
- Wired `WeekStatusService` (Basic-tier) as a new dep of `ShiftplanEditService` (Business-logic-tier) through the gen_service_impl! block and main.rs, moving the WeekStatusService construction ahead of ShiftplanEdit to preserve Basic-before-Business ordering.
- Added a deliberately pass-through `assert_week_not_locked` helper (reads status in-tx, always returns Ok) called at all 6 write heads: modify_slot, remove_slot, modify_slot_single_week, book_slot (non-shiftplanner), copy_week (target week only), delete_booking.
- Updated the test harness (struct field, assoc type, build_service, build_dependencies default `get_week_status -> Unset`) so all existing tests stay green.

## Task Commits

Each task was committed atomically:

1. **Task 1: WeekLocked-Variante + HTTP-423-Mapping (D-40-01)** - `864a4d7` (feat)
2. **Task 2: delete_booking-Trait + WeekStatusService-Dep + pass-through Helper + DI + Test-Harness** - `e39e678` (feat)

_No TDD tasks in this scaffold plan._

## Files Created/Modified
- `service/src/lib.rs` - Added WeekLocked variant to ServiceError enum (after PaidLimitExceeded)
- `rest/src/lib.rs` - Added WeekLocked -> HTTP 423 arm in error_handler
- `service/src/shiftplan_edit.rs` - Added delete_booking to the #[automock] trait
- `service_impl/src/shiftplan_edit.rs` - Added WeekStatusService dep, pass-through assert_week_not_locked helper, 5 gate calls + delete_booking impl (6th path)
- `shifty_bin/src/main.rs` - Added WeekStatusService assoc type + constructor field; moved week_status_service construction before ShiftplanEdit
- `service_impl/src/test/shiftplan_edit.rs` - MockWeekStatusService field/assoc-type/build_service + default get_week_status->Unset

## Decisions Made
- Kept the full `<Context = Self::Context, Transaction = Self::Transaction>` bound on the WeekStatusService dep — RESEARCH Open Question 1 (possible E0220/E0277) did not materialize; the full bound compiles, so no reduction to `<Context = Self::Context>` was needed.
- Helper is a method (needs self.week_status_service), pass-through only (reads status to use the dep, always Ok). Enforcement + shiftplan.edit bypass are intentionally deferred to 40-03.
- book_slot wraps the gate call in `if !is_shiftplanner` (forward-compatible with 40-03 enforcement); functionally a no-op in this scaffold since the helper always returns Ok.

## Deviations from Plan
None - plan executed exactly as written. `_context`/`_status` are underscore-bound in the pass-through helper to avoid unused-variable clippy warnings while the enforcement logic is deferred; this is inherent to the scaffold contract, not a scope change.

## Issues Encountered
- Constructor ordering: `week_status_service` was originally constructed after `shiftplan_edit_service` in main.rs. Moved the `let week_status_service = ...` binding ahead of the ShiftplanEdit construction (Basic-tier before Business-logic-tier), per the plan's contingency instruction. No other reference to it broke.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- The full Phase-40 backend surface compiles green with all existing tests passing; the lock does NOT block anything yet.
- 40-03 can now build the real enforcement (WeekStatus::Locked -> ServiceError::WeekLocked + shiftplan.edit bypass) RED-first on top of this scaffold. Enforcement is deliberately NOT active in this plan.
- 40-04 will re-route the DELETE /booking handler from booking_service() to shiftplan_edit_service().delete_booking().

---
*Phase: 40-wochen-sperre-durchsetzen*
*Completed: 2026-07-02*

## Self-Check: PASSED
All 6 modified source files, the SUMMARY.md, and both task commits (864a4d7, e39e678) verified present.
