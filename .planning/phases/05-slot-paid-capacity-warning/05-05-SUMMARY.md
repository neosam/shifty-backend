---
phase: 05-slot-paid-capacity-warning
plan: 05
subsystem: rest
tags: [rest-types, dto, slot, warning, shiftplan-view, utoipa, serde, phase-5]

# Dependency graph
requires:
  - phase: 05-slot-paid-capacity-warning
    plan: 03
    provides: "service::slot::Slot.max_paid_employees: Option<u8> + From impls round-trip the field; Rule-3 shim in `From<&SlotTO> for Slot`"
  - phase: 05-slot-paid-capacity-warning
    plan: 02
    provides: "service::warning::Warning::PaidEmployeeLimitExceeded variant (5th); workspace E0004 in rest-types/src/lib.rs:1705 awaiting From-arm"
  - phase: 05-slot-paid-capacity-warning
    plan: 04
    provides: "service::shiftplan::ShiftplanSlot.current_paid_count: u8 always populated"
provides:
  - "SlotTO carries `max_paid_employees: Option<u8>` with `#[serde(default)]` (D-10) and `ToSchema` (auto via existing derive)"
  - "WarningTO has 5th variant `PaidEmployeeLimitExceeded { slot_id, booking_id, year, week, current_paid_count, max_paid_employees }` with wire-tag `paid_employee_limit_exceeded` (D-08)"
  - "ShiftplanSlotTO carries `current_paid_count: u8` (always populated, mirrors service-tier; D-09)"
  - "From<&service::slot::Slot> for SlotTO + From<&SlotTO> for service::slot::Slot pass through max_paid_employees end-to-end (replaces Plan 05-03 Rule-3 shim)"
  - "From<&service::warning::Warning> for WarningTO covers all 5 variants exhaustively — workspace E0004 from Plan 05-02 resolved"
  - "From<&service::shiftplan::ShiftplanSlot> for ShiftplanSlotTO passes current_paid_count through"
  - "All Plan-05-03 Rule-3 forward-compat shim markers removed from .rs files (workspace grep `Phase 5 Plan 03 (Rule 3` returns 0)"
affects:
  - "05-06 (ShiftplanEditService warning emission): pushes Warning::PaidEmployeeLimitExceeded into BookingCreateResult.warnings; the wire-mirror is in place so frontend can deserialize once Plan 05-06 emits"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wire-tier mirror for additive service-tier field: extend the corresponding `*TO` struct + both `From` impls; the existing `ToSchema` derive on the DTO covers OpenAPI auto-generation for the new field — no per-field schema annotation needed for plain `Option<u8>`"
    - "Wire-tier mirror for additive enum variant: append the new variant inside `WarningTO`, ride the existing `#[serde(tag, content, rename_all = \"snake_case\")]` for the auto-generated wire-tag, then add the matching arm in `From<&Warning> for WarningTO`. No wildcard arm existed (Pattern-Map confirmed), so the addition is purely additive and rustc enforces exhaustiveness"
    - "`#[serde(default)]` on optional new fields keeps API consumers backward-compatible (Pattern-Map Pitfall #8)"
    - "Wave-coupled landing: Plan 05-02 (producer) + Plan 05-05 (consumer) in the same wave — Plan 05-02's intentional E0004 in `rest-types/src/lib.rs:1705` is the wave-boundary signal that the From-arm is the next thing to add. Plan 05-05 closes the loop"

key-files:
  created: []
  modified:
    - "rest-types/src/lib.rs (3 additions: SlotTO field at line 320; WarningTO 5th variant at line 1711 + From-arm at line 1763; ShiftplanSlotTO field at line 985 + From-impl pass-through; both Plan-05-03 Rule-3 shims in this file replaced with real round-trip)"
    - "shifty_bin/src/integration_test/booking_absence_conflict.rs (Plan-05-03 Rule-3 marker comment replaced with permanent Phase-5 annotation; the `max_paid_employees: None` value stays — correct no-limit semantics for this booking-vs-absence test)"

key-decisions:
  - "SlotTO field placed immediately after `min_resources` (mirrors the service-tier field grouping established by Plan 03 — both are slot-capacity knobs)"
  - "WarningTO 5th variant placed at the end of the enum, after `AbsenceOverlapsManualUnavailable` (additive, byte-preserves the 4 existing variants and their wire-tags). The variant carries 6 structured fields matching the service-tier `Warning::PaidEmployeeLimitExceeded` shape — bind-by-name in the From-arm so field-order is purely cosmetic"
  - "Existing `From<&Warning> for WarningTO` had NO wildcard arm (Pattern-Map confirmed — all 4 arms are explicit). Adding the 5th explicit arm is purely additive; no removal needed"
  - "ShiftplanSlotTO.current_paid_count is `u8`, not `Option<u8>` — mirrors Plan 04's service-tier choice (D-09: always populated regardless of `slot.max_paid_employees`)"
  - "Plan-05-03 Rule-3 marker in `shifty_bin/.../booking_absence_conflict.rs` resolved with permanent comment + kept `max_paid_employees: None` value (correct default-no-limit semantics for this test which exercises the booking-vs-absence path, not paid-cap-warning path)"
  - "Updated DTO header comment from `Tag-Enum (4 Varianten)` to `Tag-Enum (5 Varianten)` for accuracy"

patterns-established:
  - "Cross-tier mirror procedure for an additive Phase-5-style enum variant: (1) append variant to domain enum with full struct fields; (2) workspace cargo build E0004 fires at the downstream From-impl; (3) add the matching `Service::X { ..fields } => Self::X { fields-deref }` arm — exhaustiveness is enforced by rustc, no test needed for this contract"

requirements-completed: [D-08, D-09, D-10]

# Metrics
duration: 6min
completed: 2026-05-04
---

# Phase 5 Plan 05: REST DTO Surface Summary

**`rest-types/src/lib.rs` extended with 3 additive wire-tier mirrors of the Phase-5 service-tier additions: `SlotTO.max_paid_employees: Option<u8>` (D-10), `WarningTO::PaidEmployeeLimitExceeded` 5th variant + `From<&Warning>` arm (D-08, resolves Plan 05-02's intentional workspace E0004), and `ShiftplanSlotTO.current_paid_count: u8` (D-09). Both Plan-05-03 Rule-3 forward-compat shim markers (in this file's `From<&Slot>`/`From<&SlotTO>` impls and in `shifty_bin/.../booking_absence_conflict.rs`) removed; workspace grep `Phase 5 Plan 03 (Rule 3` returns 0 across .rs files.**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-05-04T06:23:19Z
- **Completed:** 2026-05-04T06:29:15Z
- **Tasks:** 3
- **Files modified:** 2 (`rest-types/src/lib.rs`, `shifty_bin/src/integration_test/booking_absence_conflict.rs`)

## Accomplishments

- `SlotTO` carries `pub max_paid_employees: Option<u8>` (line 320) with `#[serde(default)]` for backward-compat (D-10, Pattern-Map Pitfall #8). Both `From` impls (`From<&Slot> for SlotTO`, `From<&SlotTO> for Slot`) round-trip the value end-to-end. The Plan-05-03 Rule-3 shims previously sitting in both impls (the `let _ = slot.max_paid_employees;` consumer in the forward direction and the hardcoded `max_paid_employees: None,` placeholder in the reverse direction) are GONE — replaced with real pass-through.
- `WarningTO` has a 5th variant `PaidEmployeeLimitExceeded { slot_id, booking_id, year, week, current_paid_count, max_paid_employees }` (line 1711). The wire-tag (auto-derived from the enum-level `#[serde(rename_all = "snake_case")]`) is `paid_employee_limit_exceeded`. The matching arm in `impl From<&service::warning::Warning> for WarningTO` (line 1763) makes the match exhaustive — Plan 05-02's intentional workspace E0004 in `rest-types/src/lib.rs:1705` is RESOLVED.
- `ShiftplanSlotTO` carries `pub current_paid_count: u8` (line 985), mirroring the service-tier field added by Plan 04 (D-09). The `From<&ShiftplanSlot> for ShiftplanSlotTO` impl passes the value through.
- `BlockTO` (lines 1438-1448 area) and `BookingCreateResultTO` / `CopyWeekResultTO` / `AbsencePeriodCreateResultTO` (lines 1794-1837 area) inherit the new field/variant **automatically** through their embedded `Vec<SlotTO>` / `Vec<WarningTO>` — NOT touched in this plan.
- The doc-comment header for the Phase-3 wrapper-DTO block (line 1644) updated from `Tag-Enum (4 Varianten)` to `Tag-Enum (5 Varianten)`.
- Plan-05-03 Rule-3 marker comments in `shifty_bin/src/integration_test/booking_absence_conflict.rs` (line 85 area) replaced with a permanent Phase-5 annotation. The `max_paid_employees: None` value itself stays — this integration test exercises the booking-vs-absence conflict path, not the paid-capacity-warning path (Plan 05-06's scope), so default no-limit semantics are correct.
- Workspace `cargo build` exits 0 — exhaustiveness of `From<&Warning>` enforced by rustc (no E0004).
- Workspace `cargo test` exits 0 — 455 tests pass (10 dao + 8 cutover-service + 370 service_impl + 11 cutover + 56 shifty_bin integration), 0 failed, 0 ignored — same count as Plan 05-04 baseline (Plan 05-05 adds no new tests; the wire-mirror is purely structural).
- `grep -rn "Phase 5 Plan 03 (Rule 3" .` across all `.rs` files returns 0 matches. The forward-compat shim catalog opened by Plan 05-01 / 05-03 is fully closed.

## Task Commits

Each task was committed atomically via `jj`:

1. **Task 1: Extend SlotTO with max_paid_employees + bridge From impls** — change `srtrltup` (commit `de79cd6f`) — `feat(05-05)`
2. **Task 2: Add WarningTO::PaidEmployeeLimitExceeded variant + From-arm** — change `pvtkzlzq` (commit `c60119ff`) — `feat(05-05)`
3. **Task 3: Extend ShiftplanSlotTO with current_paid_count + resolve last Plan-05-03 Rule-3 shim** — change `ozosrtrt` (commit `945a1703`) — `feat(05-05)`

## Files Created/Modified

- **Modified:** `rest-types/src/lib.rs`
  - **SlotTO struct** (line 305-326): inserted `pub max_paid_employees: Option<u8>` at line 320 (immediately after `pub min_resources: u8`) with `#[serde(default)]` and a 5-line doc-comment naming D-10 / D-15.
  - **`impl From<&service::slot::Slot> for SlotTO`** (lines 327-348 area): added `max_paid_employees: slot.max_paid_employees,` to the `Self { ... }` constructor; removed the Plan-05-03 Rule-3 marker comment + the `let _ = slot.max_paid_employees;` shim.
  - **`impl From<&SlotTO> for service::slot::Slot`** (lines 349-365 area): replaced the hardcoded `max_paid_employees: None,` Rule-3 shim (with its 4-line marker comment) with the real `max_paid_employees: slot.max_paid_employees,` pass-through.
  - **WarningTO enum** (line 1666 area): appended a 5th variant `PaidEmployeeLimitExceeded { slot_id: Uuid, booking_id: Uuid, year: u32, week: u8, current_paid_count: u8, max_paid_employees: u8 }` after `AbsenceOverlapsManualUnavailable`. 6-line doc-comment names D-08 / D-06 / D-07 and the auto-derived wire-tag `paid_employee_limit_exceeded`.
  - **`impl From<&service::warning::Warning> for WarningTO`** (lines 1702-1746 area): added a 5th match arm at the end (line 1763 area) `service::warning::Warning::PaidEmployeeLimitExceeded { ... } => Self::PaidEmployeeLimitExceeded { ... }`. Bind-by-name destructuring with deref-on-construction matches the existing 4 arms' style. **No wildcard arm existed in the impl before this plan** — addition is purely additive.
  - **DTO header comment** (line 1644): `// * `WarningTO`             — Tag-Enum (4 Varianten), JSON-Form` → `(5 Varianten)`.
  - **ShiftplanSlotTO struct** (line 978-986 area): inserted `pub current_paid_count: u8` at line 985 (after `pub bookings: Vec<ShiftplanBookingTO>`) with a 6-line doc-comment naming D-09 / D-04 / D-05.
  - **`impl From<&service::shiftplan::ShiftplanSlot> for ShiftplanSlotTO`** (lines 1018-1026 area): added `current_paid_count: slot.current_paid_count,` to the `Self { ... }` constructor.
- **Modified:** `shifty_bin/src/integration_test/booking_absence_conflict.rs`
  - Replaced the Plan-05-03 Rule-3 marker comment block at line 85 area with a permanent Phase-5 annotation explaining `None` = no limit / no check / no warning, and noting that this integration test exercises the booking-vs-absence conflict path (Plan 05-06 owns the paid-cap-warning path).
  - The `max_paid_employees: None` value itself is unchanged — correct default-no-limit semantics for this test.

## Wire Form (sanity record)

JSON shape of the new variant — produced by `serde_json::to_string(&WarningTO::PaidEmployeeLimitExceeded { ... })`:

```json
{
  "kind": "paid_employee_limit_exceeded",
  "data": {
    "slot_id": "8b1a4e7c-…",
    "booking_id": "f3d…",
    "year": 2026,
    "week": 18,
    "current_paid_count": 3,
    "max_paid_employees": 2
  }
}
```

Wire-tag auto-derived from `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]` at line 1665.

## Test Inventory

```
$ cargo test (workspace-wide)
- dao_impl_sqlite: 10 passed
- service (cutover): 8 passed
- service_impl lib: 370 passed (no new tests; wire-mirror is structural)
- cutover service: 11 passed
- shifty_bin integration: 56 passed
- Total: 455 passed, 0 failed, 0 ignored

$ cargo build (workspace, default features)
- exits 0 (no E0004 — From<&Warning> exhaustive)
```

Plan 05-05 deliberately adds no new tests. The exhaustive-match contract for `From<&Warning>` is enforced by rustc at compile time; the `From<&Slot>` / `From<&SlotTO>` / `From<&ShiftplanSlot>` round-trips are bind-by-name struct conversions whose correctness is guaranteed by Rust's named-field type system. Adding tests for "fields are mapped through" would test the Rust compiler, not application logic. Plan 05-06 will add the booking-pfad emission tests that exercise the wire-mirror end-to-end (BookingCreateResultTO.warnings will contain the new variant).

## Decisions Made

1. **SlotTO field at line 320, immediately after `min_resources`.** Mirrors the service-tier grouping (Plan 03 placed `Slot.max_paid_employees` right after `Slot.min_resources`) — both are slot-capacity knobs.
2. **WarningTO variant appended at the end.** Additive — preserves existing 4 variants' byte-positions and their auto-derived wire-tags. Same approach as Plan 02 used in `service::warning::Warning`.
3. **No wildcard arm to remove.** Pattern-Map predicted (`existing arms are explicit (no wildcard), so the new arm is purely additive`) — verified during implementation. The match is now 5-arms-explicit, which is the correct shape.
4. **Plan-05-03 Rule-3 shim in `From<&SlotTO>` resolved with real pass-through.** The hardcoded `max_paid_employees: None,` placeholder + its 4-line marker comment was the deferred work that Plan 05 always owned.
5. **Plan-05-03 Rule-3 shim in `From<&Slot> for SlotTO` resolved by removing the `let _ = slot.max_paid_employees;` consumer + its marker comment, and adding the real `max_paid_employees: slot.max_paid_employees,` field assignment to the constructor. Same plan-level ownership as #4.
6. **Plan-05-03 Rule-3 shim in `shifty_bin/src/integration_test/booking_absence_conflict.rs` resolved by replacing the Rule-3 marker comment with a permanent Phase-5 annotation; the `max_paid_employees: None` value stays.** This integration test exercises booking-vs-absence (Plan 03 territory), not paid-cap-warning (Plan 06 territory), so default no-limit is correct. The plan's `<predecessor_context>` block explicitly authorized this resolution: "if `max_paid_employees: None` is the correct default for a no-limit-test, you can keep `None` but remove the Rule-3 marker comment and replace it with a permanent comment explaining the field's role."
7. **No tests added in this plan.** Plan-Tasks `<acceptance_criteria>` does not require new tests; the wire-mirror's correctness is enforced by rustc (exhaustive match) and Rust's named-field type system (struct conversions). Plan 05-06 will add the integration tests that exercise the wire format end-to-end.
8. **`BlockTO` and `BookingCreateResultTO` not touched.** Both inherit the new field/variant automatically through their embedded `Vec<SlotTO>` / `Vec<WarningTO>`. The plan's `<interfaces>` block documented this cascade explicitly.

## Deviations from Plan

None — plan executed exactly as written.

The plan's `<predecessor_context>` block specified two Rule-3-shim resolution sites for this plan (`rest-types/src/lib.rs` ~line 345 and `shifty_bin/src/integration_test/booking_absence_conflict.rs` line ~79). Both were resolved as authorized by the plan text.

The plan's `<output>` block called for documenting:
- Exact line numbers for the 3 inserted struct fields and the 5th `WarningTO` variant — DONE in "Files Created/Modified" above (lines 320, 985, 1711, 1763).
- Whether the existing `From<&Warning>` impl had a wildcard arm and whether it was removed — DONE: no wildcard arm existed (Pattern-Map confirmed; verified in-place); the addition is purely additive.
- Confirmation that `BlockTO` and `BookingCreateResultTO` were NOT touched — DONE: both inherit through embedding; no manual edit.
- Wire-form sanity JSON shape — DONE in "Wire Form (sanity record)" above.

## Issues Encountered

None — three single-file additions, each compile-checked individually before commit. The intentional workspace E0004 left by Plan 05-02 (in `rest-types/src/lib.rs:1705`) was resolved by Task 2 exactly as the wave-3 design predicted.

## User Setup Required

None.

## Next Phase Readiness

- **Plan 05-06 (Wave 3, last in phase)** unblocked. The wire-mirror is in place: `WarningTO::PaidEmployeeLimitExceeded` is on the wire and round-trips through `From<&Warning>`, so once Plan 05-06's `ShiftplanEditService::book_slot_with_conflict_check` pushes `Warning::PaidEmployeeLimitExceeded { ... }` into `BookingCreateResult.warnings`, the REST response (`BookingCreateResultTO.warnings`) will carry the new variant in its JSON form `{"kind":"paid_employee_limit_exceeded","data":{...}}` automatically — no further wire work needed.
- The full forward-compat shim catalog opened by Plan 05-01 (DAO-tier hardcoded-`None`) and Plan 05-03 (3 Rule-3 sites: `service_impl/src/test/shiftplan.rs`, `rest-types/src/lib.rs`, `shifty_bin/.../booking_absence_conflict.rs`) is now fully closed: Plan 05-03 closed its own first 2 sites at the time of execution, Plan 05-04 closed `test/shiftplan.rs`, and Plan 05-05 closed the remaining 2 sites in `rest-types/src/lib.rs` and `booking_absence_conflict.rs`.

## Self-Check: PASSED

- File `rest-types/src/lib.rs` contains exactly 1 occurrence of `pub max_paid_employees: Option<u8>` (line 320). Verified.
- The line preceding the field declaration is `#[serde(default)]`. Verified via grep.
- File `rest-types/src/lib.rs` contains 4 occurrences of `PaidEmployeeLimitExceeded` (variant declaration + From-arm pattern + From-arm constructor + From-arm continuation). Verified.
- New variant declares all 6 expected fields. Verified.
- File `rest-types/src/lib.rs` contains exactly 1 occurrence of `pub current_paid_count: u8` (line 985). Verified.
- `cargo build -p rest-types --features service-impl` exits 0. Verified.
- `cargo build` (workspace) exits 0 — no E0004. Verified.
- `cargo test` (workspace) exits 0; 455 tests pass. Verified.
- `grep -rn "Phase 5 Plan 03 (Rule 3" .` across `.rs` files returns 0 matches. Verified.
- jj history shows 3 atomic Plan-05 task changes:
  - `srtrltup` (`de79cd6f`) Task 1
  - `pvtkzlzq` (`c60119ff`) Task 2
  - `ozosrtrt` (`945a1703`) Task 3
  Verified via `jj log -r 'all() & description("05-05")'`.

---
*Phase: 05-slot-paid-capacity-warning*
*Completed: 2026-05-04*
