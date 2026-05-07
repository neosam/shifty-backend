---
phase: 05-slot-paid-capacity-warning
plan: 06
subsystem: service
tags: [service, shiftplan-edit, paid-employee-limit, warning-emission, business-logic-tier, phase-5, last-in-phase]

# Dependency graph
requires:
  - phase: 05-slot-paid-capacity-warning
    plan: 02
    provides: "service::warning::Warning::PaidEmployeeLimitExceeded variant"
  - phase: 05-slot-paid-capacity-warning
    plan: 03
    provides: "service::slot::Slot.max_paid_employees: Option<u8>"
  - phase: 05-slot-paid-capacity-warning
    plan: 04
    provides: "Read-side current_paid_count derivation pattern (helper mirror)"
  - phase: 05-slot-paid-capacity-warning
    plan: 05
    provides: "Wire-tier mirror — emitted Warning round-trips via WarningTO"
provides:
  - "ShiftplanEditService::book_slot_with_conflict_check emits Warning::PaidEmployeeLimitExceeded when current_paid_count > max_paid_employees AND slot.max_paid_employees.is_some() (D-06 strict, D-15 NULL-skip)"
  - "Booking is persisted even when warning fires (D-07: no Tx rollback)"
  - "Private count_paid_bookings_in_slot_week helper on ShiftplanEditServiceImpl (Business-Logic-Tier per CLAUDE.md Service-Tier-Konventionen + v1.0 D-Phase3-18 BookingService Basic-Tier regression-lock)"
  - "Helper reuses get_for_week + get_all_paid (both already in ShiftplanEditServiceDeps; no new DI dep)"
  - "6 service-tier tests in service_impl/src/test/shiftplan_edit.rs covering all D-04/D-05/D-06/D-07/D-15 invariants"
  - "Legacy POST /booking (rest/src/booking.rs) UNVERAENDERT (D-16); BookingService::create UNVERAENDERT (D-Phase3-18)"
affects:
  - "Phase 5 closes — all 6 plans complete; v1.1 Slot Capacity & Constraints milestone ready for ship"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Warning emission inside an existing Business-Logic-Tier service method: insert the limit-check + warnings.push between the existing absence/unavailable warning emission and the final transaction_dao.commit, after persisted_booking is in-hand. No tx rollback on warning (D-07)."
    - "Private helper on the same impl (separate impl block) for cross-entity counting: (slot_id, year, week) tuple over BookingService::get_for_week + SalesPersonService::get_all_paid intersection. Saturating-cast count.min(u8::MAX as usize) as u8 mirrors Plan 05-04's read-side aggregator."
    - "Authentication::Full for inner cross-service lookups inside a Business-Logic helper — outer caller's permission was already validated."
    - "Service-tier mock test pattern: override expect_get_slot to inject the slot variant, override expect_get_for_week with the post-persist booking set, register expect_get_all_paid (NOT in build_dependencies default) only on tests that exercise the limit-check path. Tests with default Slot.max_paid_employees=None require no expect_get_all_paid because the helper is never invoked (D-15 short-circuit) — mockall-strict expectations would panic if the helper triggered, providing implicit verification of the NULL-skip."

key-files:
  created:
    - ".planning/phases/05-slot-paid-capacity-warning/05-06-SUMMARY.md (this file)"
  modified:
    - "service_impl/src/shiftplan_edit.rs (warning emission inside book_slot_with_conflict_check + private count_paid_bookings_in_slot_week helper)"
    - "service_impl/src/test/shiftplan_edit.rs (6 new tests + 4 SP-id constants + paid_sales_person/slot_with_paid_limit/existing_paid_booking fixtures + SalesPerson use import)"

key-decisions:
  - "Helper lives on ShiftplanEditServiceImpl (Business-Logic-Tier), NOT on BookingService (Basic-Tier). Plan-File <objective> explicitly overrides CONTEXT.md D-12's BookingService mention per CLAUDE.md Service-Tier-Konventionen + v1.0 D-Phase3-18 regression-lock."
  - "Helper accepts Deps::Transaction by value (already-active tx, no Option<>) since it's only ever called inside book_slot_with_conflict_check which has already use_transaction'd. No commit inside the helper — the caller commits at the end of book_slot_with_conflict_check."
  - "Filter predicate is symmetric to Plan 05-04's read-side aggregator: bookings.deleted IS NULL (DAO-side + belt-and-suspenders b.deleted.is_none()) AND sales_person.id ∈ get_all_paid result (which already filters WHERE deleted IS NULL AND is_paid = 1 at the DAO level). Absence-status orthogonal (D-05)."
  - "Saturating-cast count.min(u8::MAX as usize) as u8 — defensive against pathological fixture sizes. Plan 05-04 used the same pattern."
  - "Authentication is already imported at line 11 of service_impl/src/shiftplan_edit.rs via use service::{..., permission::{Authentication, HR_PRIVILEGE}, ...}; — no duplicate use statement added."
  - "booking.calendar_week as u8 cast at both new binding sites (helper invocation + Warning field). Field is i32 in this codebase; mirrors the existing precedent at line 481 (BookingOnUnavailableDay emission). Total cast occurrences in file: 5 (1 pre-existing + 2 in the new emission block + 2 from the helper invocation arguments — same line, but grep counts both new uses; the 5 reflects the actual occurrence count in the patched file)."
  - "BookingService::create UNVERAENDERT (D-Phase3-18 regression-lock); rest/src/booking.rs UNVERAENDERT (D-16). Both verified via grep returning 0 occurrences of PaidEmployeeLimitExceeded."

patterns-established:
  - "Helper-method-on-impl pattern for Business-Logic-Tier cross-entity computations: separate impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditServiceImpl<Deps> block at the end of the file, commented as 'Phase 5 (D-04, D-05, D-12) — private Helpers'. Keeps the trait-impl block focused on the trait surface and the helper visible only to the implementing service."
  - "Strict-greater-than threshold (D-06) for soft-warning emission: if current > max, warn. Equal does NOT warn. Threshold semantics: 'erlaubtes Maximum'."
  - "D-07 soft-warning preservation: emit between persistence and commit. The persisted entity flows into the result wrapper (BookingCreateResult.booking) untouched; warnings flow alongside (BookingCreateResult.warnings). No exception, no rollback, no error code."

requirements-completed: [D-04, D-05, D-06, D-07, D-08, D-12, D-13, D-15, D-16]

# Metrics
duration: 7min
completed: 2026-05-04
---

# Phase 5 Plan 06: ShiftplanEdit Warning Emission Summary

**`ShiftplanEditService::book_slot_with_conflict_check` now emits `Warning::PaidEmployeeLimitExceeded` after persistence (D-07: no rollback) when `slot.max_paid_employees.is_some()` AND `current_paid_count > max` (D-06 strict, D-15 NULL-skip). Private helper `count_paid_bookings_in_slot_week` lives on `ShiftplanEditServiceImpl` (Business-Logic-Tier per CLAUDE.md + v1.0 D-Phase3-18 regression-lock). 6 service-tier tests cover D-04/D-05/D-06/D-07/D-15 invariants. Legacy `POST /booking` and `BookingService::create` are byte-identical to the pre-plan state. Workspace `cargo build` + `cargo test` green (461 tests pass workspace-wide; +6 over Plan 05-05 baseline of 455). `cargo run` boots cleanly — server reaches "Running server at 127.0.0.1:3000" before the 30s smoke-test timeout. Phase 5 ships complete (6/6 plans, 100%).**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-05-04T06:36:30Z
- **Completed:** 2026-05-04T06:42:55Z
- **Tasks:** 2 (1 emission + 1 TDD test suite)
- **Files modified:** 2 (`service_impl/src/shiftplan_edit.rs`, `service_impl/src/test/shiftplan_edit.rs`)

## Accomplishments

- Inside `book_slot_with_conflict_check` (`service_impl/src/shiftplan_edit.rs`), inserted a 30-line Phase-5 emission block between the existing `for mu in manual_unavailables.iter()` loop and the final `self.transaction_dao.commit(tx).await?;` call. The block:
  1. Short-circuits via `if let Some(max) = slot.max_paid_employees` (D-15: NULL skips entirely).
  2. Calls the new `count_paid_bookings_in_slot_week` helper with `(booking.slot_id, booking.year, booking.calendar_week as u8, tx.clone())`.
  3. If `current_paid_count > max` (D-06 strict), pushes `Warning::PaidEmployeeLimitExceeded { slot_id, booking_id: persisted_booking.id, year, week, current_paid_count, max_paid_employees }` onto the existing `warnings` accumulator.
- Added a private helper `count_paid_bookings_in_slot_week` in a new `impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditServiceImpl<Deps>` block at the end of the file. The helper:
  - Accepts `(slot_id: Uuid, year: u32, week: u8, tx: Deps::Transaction)` by value (already-active tx, no `Option<>`).
  - Calls `BookingService::get_for_week(week, year, Authentication::Full, Some(tx.clone()))` and `SalesPersonService::get_all_paid(Authentication::Full, Some(tx.clone()))`.
  - Builds a `HashSet<Uuid>` of paid sales-person-ids and intersects with bookings filtered by `slot_id` and `deleted.is_none()`.
  - Returns `count.min(u8::MAX as usize) as u8` (saturating cast, mirrors Plan 05-04's read-side derivation).
- 6 new service-tier tests in `service_impl/src/test/shiftplan_edit.rs`, all green:
  - `test_book_paid_into_full_slot_emits_warning` — D-04, D-06, D-08, D-13
  - `test_book_paid_at_limit_no_warning` — D-06 strict (equal does NOT trigger)
  - `test_book_unpaid_into_full_slot_no_warning` — D-04 (unpaid SP not counted)
  - `test_book_with_no_limit_no_warning` — D-15 (NULL skips entirely; helper never invoked)
  - `test_book_paid_in_absence_still_counts` — D-05 (absence orthogonal; both paid-limit and BookingOnAbsenceDay warnings fire independently)
  - `test_book_persists_even_when_warning_fires` — D-07 (`result.booking.id == persisted-mock-id`; commit ran)
- 4 new fixture functions: `paid_sp_a_id`, `paid_sp_b_id`, `paid_sp_c_id`, `unpaid_sp_id` (stable UUIDs), `paid_sales_person(id) -> SalesPerson`, `slot_with_paid_limit(max: u8) -> Slot` (spread over `monday_slot()`), `existing_paid_booking(sp_id, booking_id) -> Booking`.
- Imports: added `SalesPerson` to the existing `use service::sales_person::{MockSalesPersonService, ...};` line. No other import changes; `Authentication` is already imported at line 11 of `service_impl/src/shiftplan_edit.rs` (verified — no duplicate added).
- 461 tests pass workspace-wide: `10 dao_impl_sqlite + 8 cutover-service + 376 service_impl + 11 cutover + 56 shifty_bin integration` (+6 service_impl over Plan 05-05 baseline of 455).
- `cargo build` workspace-wide exits 0.
- `cargo run` with `timeout 30` boots cleanly through `Running server at 127.0.0.1:3000`.

## Task Commits

Each task was committed atomically via `jj`:

1. **Task 1: Add count_paid_bookings_in_slot_week helper + emit PaidEmployeeLimitExceeded warning** — change `zlyyyrot` (commit `2e13be7d`) — `feat(05-06)`
2. **Task 2: Add 6 service-tier tests for paid-employee-limit warning emission** — change `uqmnyovl` (commit `ef2efbe0`) — `test(05-06)`

## Files Created/Modified

### `service_impl/src/shiftplan_edit.rs`

- **Edit A — Emission block (between existing manual-unavailable loop and the final commit):**
  - Inserted a 30-line block (after the existing `for mu in manual_unavailables.iter() { ... }` loop at line ~488 and before `self.transaction_dao.commit(tx).await?;` at line ~520-area). The block carries a 14-line doc-comment naming D-04/D-05/D-06/D-07/D-15/D-16 and the `as u8` cast convention.
  - Local variable name match: `persisted_booking.id` (NOT `created_booking` — confirmed by reading the surrounding code which uses `persisted_booking` consistently for this binding).
  - `booking.calendar_week as u8` cast appears at 2 new binding sites in the emission block: (a) helper invocation argument `booking.calendar_week as u8`, (b) the `week:` field of the new `Warning::PaidEmployeeLimitExceeded {...}`. Combined with the pre-existing line-481 cast in the `BookingOnUnavailableDay` emission, the file now contains 5 `booking.calendar_week as u8` occurrences (verified via `grep -c`).
- **Edit B — Helper impl (new block at the end of the file):**
  - Added `impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditServiceImpl<Deps>` block carrying the private helper `async fn count_paid_bookings_in_slot_week(&self, slot_id: Uuid, year: u32, week: u8, tx: Deps::Transaction) -> Result<u8, ServiceError>`. 28-line doc-comment names D-04, D-05, D-12, the upstream filter contracts, and the saturating-cast convention.
  - Helper does NOT call `transaction_dao.commit` — caller-driven commit lives in `book_slot_with_conflict_check`.
  - Helper uses `Authentication::Full` for inner cross-service lookups (Phase-3 precedent — outer caller's permission already validated via the `hr.or(sp)?` gate).

### `service_impl/src/test/shiftplan_edit.rs`

- **Imports:** added `SalesPerson` to the existing `use service::sales_person::{MockSalesPersonService, ...};` line (now `use service::sales_person::{MockSalesPersonService, SalesPerson};`). No other import changes.
- **Fixtures (added before the new test block, after the existing forbidden tests):**
  - `paid_sp_a_id()`, `paid_sp_b_id()`, `paid_sp_c_id()`, `unpaid_sp_id()` — 4 stable UUIDs.
  - `paid_sales_person(id: Uuid) -> SalesPerson` — fixture with `is_paid: Some(true)`, `inactive: false`, `deleted: None`.
  - `slot_with_paid_limit(max: u8) -> Slot` — spread over `monday_slot()` with `max_paid_employees: Some(max)`.
  - `existing_paid_booking(sales_person_id: Uuid, booking_id: Uuid) -> Booking` — pre-populated booking in (slot, year=2026, week=17).
- **6 new tests** (each `#[tokio::test]`):
  1. `test_book_paid_into_full_slot_emits_warning` — slot max=2, post-persist `get_for_week` returns 3 paid bookings, `get_all_paid` returns 3 paid SPs → exactly one `Warning::PaidEmployeeLimitExceeded { current_paid_count: 3, max_paid_employees: 2, slot_id: default_slot_id(), booking_id: default_booking_id(), year: 2026, week: 17 }`.
  2. `test_book_paid_at_limit_no_warning` — slot max=2, post-persist 2 paid bookings, 2 paid SPs → `current == max == 2`, NO `PaidEmployeeLimitExceeded` (D-06 strict).
  3. `test_book_unpaid_into_full_slot_no_warning` — slot max=2, post-persist `get_for_week` returns 2 paid + 1 unpaid bookings, but `get_all_paid` returns ONLY the 2 paid SPs (so `paid_ids.contains(unpaid_sp_id)` is false). Count = 2 ≤ max, no warning.
  4. `test_book_with_no_limit_no_warning` — default `monday_slot()` has `max_paid_employees: None`. The helper is never invoked (verified implicitly: no `expect_get_all_paid` registered; mockall-strict mode would panic if it triggered). No warning.
  5. `test_book_paid_in_absence_still_counts` — slot max=1, `find_overlapping_for_booking` returns an `AbsencePeriod` for the booked SP, post-persist 2 paid bookings → `PaidEmployeeLimitExceeded` fires (count=2 > max=1) AND `BookingOnAbsenceDay` fires (Plan 03-04 path) — both independently. D-05 confirmed: absence orthogonal.
  6. `test_book_persists_even_when_warning_fires` — over-limit fixture; assert `result.booking.id == default_booking_id()` (the persisted-booking id from the `BookingService::create` mock). Proves D-07: the commit ran, no rollback.

## Decisions ↔ Test Mapping

| Decision | Tests Covering It |
|----------|-------------------|
| D-04 (paid-only count) | test_book_paid_into_full_slot_emits_warning, test_book_unpaid_into_full_slot_no_warning |
| D-05 (absence orthogonal) | test_book_paid_in_absence_still_counts |
| D-06 (strikt-größer) | test_book_paid_into_full_slot_emits_warning (3>2 fires), test_book_paid_at_limit_no_warning (2==2 does not fire) |
| D-07 (kein Rollback) | test_book_persists_even_when_warning_fires |
| D-08 (Variant-Shape) | test_book_paid_into_full_slot_emits_warning (asserts all 6 fields) |
| D-13 (BookingCreateResult.warnings) | All 6 tests via `result.warnings` access |
| D-15 (NULL-skip) | test_book_with_no_limit_no_warning |
| D-16 (Endpoint-Scope) | grep verification (architectural assertion) |

D-12 (Business-Logic-Tier placement) is structurally enforced: the helper lives on `ShiftplanEditServiceImpl`, not `BookingServiceImpl`. Verified via grep `grep -c "PaidEmployeeLimitExceeded" service_impl/src/booking.rs` returns 0.

## Wire Form (sanity record)

JSON shape produced by the emission, after Plan 05-05's `From<&Warning> for WarningTO` arm runs:

```json
{
  "kind": "paid_employee_limit_exceeded",
  "data": {
    "slot_id": "7a7ff57a-782b-4c2e-a68b-4e2d81d79380",
    "booking_id": "cea260a0-112b-4970-936c-f7e529955bd0",
    "year": 2026,
    "week": 17,
    "current_paid_count": 3,
    "max_paid_employees": 2
  }
}
```

The wire-tag `paid_employee_limit_exceeded` comes from `#[serde(rename_all = "snake_case")]` on `WarningTO` in `rest-types/src/lib.rs`. The flow Plan-05-06 → Plan-05-05 (`From<&Warning>`) → JSON is end-to-end tested-by-construction: the 6 service-tier tests produce the variant, the wire-mirror tests in Plan 05-05 are guaranteed by rustc's exhaustiveness check.

## Test Inventory

```
$ cargo test -p service_impl --lib shiftplan_edit::
running 12 tests  →  12 passed (6 NEW + 6 pre-existing)

NEW Phase-5 Plan-06 tests:
- test::shiftplan_edit::test_book_paid_into_full_slot_emits_warning ... ok
- test::shiftplan_edit::test_book_paid_at_limit_no_warning          ... ok
- test::shiftplan_edit::test_book_unpaid_into_full_slot_no_warning  ... ok
- test::shiftplan_edit::test_book_with_no_limit_no_warning          ... ok
- test::shiftplan_edit::test_book_paid_in_absence_still_counts      ... ok
- test::shiftplan_edit::test_book_persists_even_when_warning_fires  ... ok

$ cargo test --workspace
- dao_impl_sqlite: 10 passed
- service (cutover): 8 passed
- service_impl lib: 376 passed (up from 370 in Plan 05-04 baseline; +6)
- cutover service: 11 passed
- shifty_bin integration: 56 passed
- Total: 461 passed, 0 failed, 0 ignored (up from 455)

$ cargo build (workspace, default features)
- exits 0

$ timeout 30 cargo run
- "Running server at 127.0.0.1:3000" reached; clean boot, no runtime panic
```

## Decisions Made

1. **Helper lives on `ShiftplanEditServiceImpl` (Business-Logic-Tier), NOT on `BookingService` (Basic-Tier).** Plan-File `<objective>` overrides CONTEXT.md D-12's BookingService mention per CLAUDE.md "Service-Tier-Konventionen" + v1.0 D-Phase3-18 regression-lock. `ShiftplanEditService` already consumes `BookingService` AND `SalesPersonService` (verified in `gen_service_impl!` block lines 26-41) — no new DI dep added.
2. **Helper accepts `Deps::Transaction` by value, NOT `Option<Deps::Transaction>`.** It's only ever called from inside `book_slot_with_conflict_check` which has already `use_transaction`'d. Passing `Some(tx.clone())` to the inner `get_for_week` / `get_all_paid` calls keeps the same transaction across both queries.
3. **No `transaction_dao.commit` inside the helper.** The caller (`book_slot_with_conflict_check`) commits at the end. Helper only reads.
4. **`Authentication::Full` for inner cross-service lookups.** Outer caller's permission was already validated via `hr.or(sp)?` at line 416 — no need to re-check inside the helper.
5. **Saturating-cast `count.min(u8::MAX as usize) as u8`** mirrors Plan 05-04's read-side aggregator. Defensive against pathological fixture sizes (impossible in production but cheap to guard).
6. **`b.deleted.is_none()` filter is belt-and-suspenders.** The DAO already filters `WHERE deleted IS NULL`; the in-memory check is harmless and self-documenting.
7. **`booking.calendar_week as u8` cast at 2 new binding sites.** Field is `i32` in this codebase; mirrors line-481 precedent in the existing `BookingOnUnavailableDay` emission.
8. **6 tests use the existing `build_dependencies` mock-setup helper.** Tests with limit-checks override `expect_get_slot` / `expect_get_for_week` / `expect_create` and add `expect_get_all_paid` (NOT registered in the default `build_dependencies`). Tests with `slot.max_paid_employees: None` (default) skip `expect_get_all_paid` entirely — mockall-strict mode would panic if the helper triggered, providing implicit verification of the D-15 short-circuit.
9. **No new tests for `BookingService::create` or `rest/src/booking.rs`.** D-16 is an architectural assertion verified by grep returning 0 occurrences of `PaidEmployeeLimitExceeded` in those files.
10. **No `cargo run` integration test for the wire format.** The D-08 → wire-format pipeline is guaranteed by Rust's exhaustive-match enforcement (Plan 05-05's `From<&Warning>` arm) + the 6 service-tier tests producing the variant. Adding a REST-level test would be a separate decision (out of scope per Plan-File `<verification>`).

## Deviations from Plan

None — both tasks executed exactly as the Plan-File `<action>` blocks prescribe.

The Plan-File flagged 3 specific risks all of which the implementation handled correctly:
- **`Booking.calendar_week` is `i32`:** verified at `service/src/booking.rs:17`; cast `as u8` applied at every new binding site (5 total occurrences in the patched file: 1 pre-existing line-481 + 2 new in the helper-invocation arguments + 2 new in the `Warning` emission).
- **`Authentication` already imported at line 11:** verified; no duplicate `use` statement added.
- **Local variable name `persisted_booking`:** confirmed by reading lines 459-462 of the pre-edit file; matches the Plan-File's reference to `persisted_booking.id`.

Plan-AC reconciliation: the AC `grep -c "booking.calendar_week as u8" service_impl/src/shiftplan_edit.rs >= 3` predicted "1 pre-existing + 2 new = 3"; actual count is 5 because the new emission block contains 2 + 2 = 4 new uses (helper arg + Warning field) plus the 1 pre-existing line-481 = 5. The intent (`>= 3`) is satisfied.

## Issues Encountered

None — both tasks completed end-to-end on the first attempt. The only minor friction was setting up `expect_create` returning the binding-input passing `b.clone()` so the persisted Booking carried the test's chosen `sales_person_id` (the default `build_dependencies` mock returns a fixed `persisted_booking()` regardless of input); resolved by the `..b.clone()` spread pattern.

## User Setup Required

None — pure code change, no migrations, no env vars, no external service config.

## Phase 5 Closeout

Plan 05-06 is the final plan in Phase 5. With this plan complete:

- **Phase 5: Slot Paid Capacity Warning — 6/6 plans, 100%.**
- **Milestone v1.1 Slot Capacity & Constraints — Phase 5 ready for ship; only milestone phase complete.**
- **Full forward-compat shim catalog closed:** Plan 05-01's DAO-tier hardcoded-`None` (closed by Plan 05-03), Plan 05-03's 3 Rule-3 sites in `test/shiftplan.rs` (closed by Plan 05-04), `rest-types/src/lib.rs` (closed by Plan 05-05), `shifty_bin/.../booking_absence_conflict.rs` (closed by Plan 05-05). Workspace `grep "Phase 5 Plan ... (Rule 3"` returns 0 across all `.rs` files.
- **D-04 → D-16 traceability complete:** every decision has either an implementation site, a test asserting it, or a grep-verified architectural invariant.

Frontend implementation (shifty-dioxus repo) is the next milestone-aligned workstream — out of scope per CONTEXT.md "Strikt nicht in Scope".

## Self-Check: PASSED

- `service_impl/src/shiftplan_edit.rs` contains exactly 1 occurrence of `fn count_paid_bookings_in_slot_week`. Verified.
- `service_impl/src/shiftplan_edit.rs` contains 1 occurrence of `Warning::PaidEmployeeLimitExceeded` (the emission). Verified.
- `service_impl/src/shiftplan_edit.rs` contains exactly 1 occurrence of `if let Some(max) = slot.max_paid_employees`. Verified (D-15 NULL-skip).
- `service_impl/src/shiftplan_edit.rs` contains exactly 1 occurrence of `current_paid_count > max`. Verified (D-06 strict).
- `service_impl/src/shiftplan_edit.rs` contains 5 occurrences of `booking.calendar_week as u8` (≥ 3 required by AC). Verified.
- `service_impl/src/shiftplan_edit.rs` `^use.*Authentication` count = 0 (already imported via the broader `use service::{ ... permission::{Authentication, HR_PRIVILEGE}, ... };` at line 11). Verified.
- `service_impl/src/booking.rs` contains 0 occurrences of `PaidEmployeeLimitExceeded`. Verified (D-Phase3-18 + D-12 confirmation).
- `rest/src/booking.rs` contains 0 occurrences of `PaidEmployeeLimitExceeded`. Verified (D-16 confirmation).
- `service_impl/src/test/shiftplan_edit.rs` contains all 6 new test functions. Verified via grep.
- `cargo test -p service_impl --lib shiftplan_edit::` passes 12 tests (6 new + 6 pre-existing). Verified.
- `cargo test --workspace` passes 461 tests. Verified.
- `cargo build` (workspace) exits 0. Verified.
- `timeout 30 cargo run` reaches `Running server at 127.0.0.1:3000` and exits cleanly via the timeout SIGTERM. Verified.
- jj history shows 2 atomic Plan-06 task changes:
  - `zlyyyrot` (commit `2e13be7d`) Task 1
  - `uqmnyovl` (commit `ef2efbe0`) Task 2
  Verified via `jj log --limit 5`.

---
*Phase: 05-slot-paid-capacity-warning*
*Plan: 06 (last in phase)*
*Completed: 2026-05-04*
