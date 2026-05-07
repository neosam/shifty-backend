---
phase: 05-slot-paid-capacity-warning
plan: 03
subsystem: service
tags: [service, slot, dto, fixture-migration, max_paid_employees, rule-3-shim]

# Dependency graph
requires:
  - phase: 05-slot-paid-capacity-warning
    plan: 01
    provides: "DAO-tier `SlotEntity.max_paid_employees` + 4 read sites + INSERT/UPDATE; 2 Rule-3 shims marked `// Phase 5 Plan 01`"
provides:
  - "service::slot::Slot carries `max_paid_employees: Option<u8>`"
  - "Both `From` impls (`From<&SlotEntity> for Slot`, `From<&Slot> for SlotEntity`) round-trip the field"
  - "`SlotServiceImpl::create_slot` and `update_slot` propagate the field via existing `..slot.clone()` spread (no production-code change required)"
  - "`max_paid_employees` is mutable in-place per D-11 (NOT in `ModificationNotAllowed` validation list); `SHIFTPLANNER_PRIVILEGE` gate covers the permission requirement transitively"
  - "5 owned test files + 3 forward-compat shim sites carry `max_paid_employees: None` so workspace builds and the entire test suite (366 service_impl + 56 integration + others = 451+) stays green"
affects:
  - 05-04 (Shiftplan view: `current_paid_count` derivation — file `test/shiftplan.rs` already has the field-shim in place)
  - 05-05 (REST DTO surface: `SlotTO.max_paid_employees` — Rule-3 shim placed in `rest-types/src/lib.rs::From<&SlotTO>` ready for Plan 05-05 to replace)
  - 05-06 (ShiftplanEditService warning emission — consumes `slot.max_paid_employees`)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Service-tier DTO field add via `..slot.clone()` spread — no constructor edit needed in `create_slot`/`update_slot`"
    - "Rule-3 forward-compat shim across plan boundaries: when domain DTO gains a field one plan before its REST/test mirror, add the field literally to fixtures or hardcode `None` in the cross-tier `From` impl, with an inline comment naming the follow-up plan"
    - "Spread-based fixture override: `Slot { max_paid_employees: Some(N), ..generate_default_slot() }` lets new tests assert paid-limit semantics without duplicating the full literal"

key-files:
  created: []
  modified:
    - "service/src/slot.rs (Slot struct + both From impls; replaces Plan 05-01 Rule-3 None shim)"
    - "service_impl/src/test/slot.rs (2 default fixtures + 2 new fixture helpers + 3 new tests)"
    - "service_impl/src/test/booking.rs (slot_service.expect_get_slot fixture)"
    - "service_impl/src/test/block.rs (default_slot + second_slot)"
    - "service_impl/src/test/absence.rs (default_slot_monday)"
    - "service_impl/src/test/shiftplan_edit.rs (monday_slot)"
    - "service_impl/src/test/shiftplan.rs (Rule-3 shim in default_slot + slot_with_day_and_time — Plan 05-04 owns the rest)"
    - "rest-types/src/lib.rs (Rule-3 shim in From<&SlotTO> for Slot — Plan 05-05 owns the SlotTO field)"
    - "shifty_bin/src/integration_test/booking_absence_conflict.rs (Rule-3 shim in inline Slot literal)"

key-decisions:
  - "Production code in `service_impl/src/slot.rs` requires NO edit: both `create_slot` and `update_slot` already construct the returned `Slot` via `..slot.clone()` spread, so `max_paid_employees` flows through transparently (verified at lines 248-252 and 340-343)"
  - "Field deliberately NOT added to `ModificationNotAllowed` list (D-11) — the field is mutable in-place; existing `SHIFTPLANNER_PRIVILEGE` gate at lines 292-294 satisfies the permission Pflicht-Test (CONTEXT.md § REST-Test) transitively"
  - "Used spread-based fixture variants `generate_slot_with_paid_limit` / `generate_default_slot_entity_with_paid_limit` for the 3 new tests instead of duplicating full literals"
  - "Applied Rule-3 forward-compat shims to 3 OUT-OF-SCOPE sites that block workspace build under sequential execution (Plan 03 was originally specced as Wave-2 PARALLEL with Plan 04 — see Deviations)"

patterns-established:
  - "Cross-plan Rule-3 shim with explicit hand-off comment: `// Phase 5 Plan 03 (Rule 3 - blocker fix): ... Plan 05-XX will replace this with ...`"

requirements-completed: [D-02, D-10, D-11]

# Metrics
duration: 9min
completed: 2026-05-04
---

# Phase 5 Plan 03: Slot Service Wiring + Service-Tier Fixture Migration Summary

**`service::slot::Slot` now carries `max_paid_employees: Option<u8>` end-to-end through the service tier; 5 owned test files migrated; 3 new tests verify create-with-limit, update-changes-limit, and update-clears-limit semantics; permission requirement covered transitively via the existing `SHIFTPLANNER_PRIVILEGE` gate.**

## Performance

- **Duration:** ~9 min
- **Started:** 2026-05-04T05:52:51Z
- **Completed:** 2026-05-04T06:02:17Z
- **Tasks:** 2
- **Files modified:** 9 (5 owned test files + 1 service DTO file + 3 Rule-3 shim sites)

## Accomplishments

- `service::slot::Slot` extended with `pub max_paid_employees: Option<u8>` field, positioned immediately after `min_resources` (matches Pattern-Map's slot-capacity grouping).
- Both `From<&SlotEntity> for Slot` and `From<&Slot> for SlotEntity` round-trip the field; the second impl REPLACES Plan 05-01's Rule-3 hardcoded-`None` shim with `slot.max_paid_employees`.
- `service_impl/src/slot.rs` requires **no** production-code edit: both `create_slot` and `update_slot` already build the returned `Slot` via `..slot.clone()` spread, so the new field flows through automatically. Field is **not** added to `ModificationNotAllowed`, satisfying D-11 (mutable in-place). The `SHIFTPLANNER_PRIVILEGE` permission gate at lines 292-294 covers the permission Pflicht-Test transitively.
- 5 owned test files updated:
  - `service_impl/src/test/slot.rs` — both default fixtures (`generate_default_slot` + `generate_default_slot_entity`) carry the new field, plus 2 new fixture helpers (`generate_slot_with_paid_limit`, `generate_default_slot_entity_with_paid_limit`) and 3 new tests.
  - `service_impl/src/test/booking.rs` — inline `slot_service.expect_get_slot` literal.
  - `service_impl/src/test/block.rs` — `default_slot()` + `second_slot()`.
  - `service_impl/src/test/absence.rs` — `default_slot_monday()`.
  - `service_impl/src/test/shiftplan_edit.rs` — `monday_slot()`.
- 3 new service-tier tests verify the Phase-5 contract:
  - `test_create_slot_with_paid_limit` — creates a slot with `Some(3)`, asserts both the persisted `SlotEntity` mock and the returned `Slot` carry the value.
  - `test_update_slot_changes_max_paid_employees` — mutates `Some(2) → Some(5)` (no `ModificationNotAllowed`).
  - `test_update_slot_clears_max_paid_employees` — mutates `Some(5) → None` (clearing the limit).

## Task Commits

Each task was committed atomically via `jj`:

1. **Task 1: Add max_paid_employees to service::slot::Slot + both From impls** — change `vvrtoyxl` (commit `2a823d59`) — `feat(05-03)`
2. **Task 2: Wire SlotService + 5 owned test files (+ 3 Rule-3 shim sites)** — change `prpvypok` (commit `3cabfb6b`) — `feat(05-03)`

## Files Created/Modified

### Owned by Plan 03

- **Modified:** `service/src/slot.rs`
  - `Slot` struct: added `pub max_paid_employees: Option<u8>` after `min_resources` (line 19).
  - `From<&SlotEntity> for Slot`: added `max_paid_employees: slot.max_paid_employees,` after `min_resources` (line 32).
  - `From<&Slot> for SlotEntity`: REPLACED Plan 05-01's Rule-3 hardcoded-`None` shim (and its 4-line comment) with the real `slot.max_paid_employees` flow (line 47).
- **Modified:** `service_impl/src/test/slot.rs`
  - `generate_default_slot()`: added `max_paid_employees: None` (line ~39).
  - `generate_default_slot_entity()`: REPLACED Plan 05-01's Rule-3 placeholder comment with a normal field declaration `max_paid_employees: None` (line ~53).
  - NEW `generate_slot_with_paid_limit(max: u8) -> Slot` fixture helper.
  - NEW `generate_default_slot_entity_with_paid_limit(max: u8) -> SlotEntity` fixture helper.
  - NEW `test_create_slot_with_paid_limit`, `test_update_slot_changes_max_paid_employees`, `test_update_slot_clears_max_paid_employees` (appended to end of file).
  - **No** other inline `Slot { ... }` / `SlotEntity { ... }` literals required edits — all 24 inline `Slot {` and 5 inline `SlotEntity {` literals already used `..generate_default_slot()` / `..generate_default_slot_entity()` spread, so they auto-inherit the new field.
- **Modified:** `service_impl/src/test/booking.rs` — added `max_paid_employees: None` to the inline `Slot { ... }` literal at the `slot_service.expect_get_slot` mock (line ~163).
- **Modified:** `service_impl/src/test/block.rs` — added `max_paid_employees: None` to `default_slot()` (line ~104) and `second_slot()` (line ~120).
- **Modified:** `service_impl/src/test/absence.rs` — added `max_paid_employees: None` to `default_slot_monday()` (line ~139).
- **Modified:** `service_impl/src/test/shiftplan_edit.rs` — added `max_paid_employees: None` to `monday_slot()` (line ~75).

### Rule-3 Forward-Compat Shims (out-of-scope but mechanically required)

- **Modified:** `service_impl/src/test/shiftplan.rs` — added `max_paid_employees: None` (with `// Phase 5 Plan 03 (Rule 3 - blocker fix)` comment naming Plan 05-04) to:
  - `default_slot()` at line ~35.
  - `slot_with_day_and_time()` at line ~321.
  - The 2 inline literals at lines 503/507 already used spread, so no edit required there.
- **Modified:** `rest-types/src/lib.rs` — added Rule-3 shim to:
  - `From<&service::slot::Slot> for SlotTO`: explicit `let _ = slot.max_paid_employees;` to consume the field (Plan 05-05 will round-trip it via a real `SlotTO.max_paid_employees` field).
  - `From<&SlotTO> for service::slot::Slot`: hardcoded `max_paid_employees: None,` with comment naming Plan 05-05.
- **Modified:** `shifty_bin/src/integration_test/booking_absence_conflict.rs` — added `max_paid_employees: None` (with shim comment) to the inline `Slot { ... }` literal in `create_monday_slot` (line ~84).

## Test Inventory

```
$ cargo test -p service_impl --lib slot::
running 33 tests  →  33 passed (3 NEW + 30 pre-existing)

NEW Phase-5 tests:
- test::slot::test_create_slot_with_paid_limit            ... ok
- test::slot::test_update_slot_changes_max_paid_employees ... ok
- test::slot::test_update_slot_clears_max_paid_employees  ... ok

$ cargo test (workspace-wide)
- service_impl lib: 366 passed
- shifty_bin integration: 56 passed
- dao_impl_sqlite: 10 passed
- cutover service: 11 passed
- other lib targets: 8 passed
- Total: 451+ passed, 0 failed
```

## Decisions Made

1. **No edit needed in `service_impl/src/slot.rs`.** Both `create_slot` (lines 248-252) and `update_slot` (lines 340-343) already construct the returned `Slot` via `..slot.clone()` spread, so the new field propagates automatically. Verified by reading the constructor block.
2. **Field NOT added to `ModificationNotAllowed` list** — D-11 explicitly requires it to be mutable. The existing `SHIFTPLANNER_PRIVILEGE` check at lines 292-294 covers the permission Pflicht-Test (CONTEXT.md § REST-Test) transitively. Acceptance criterion: `grep -B 2 -A 6 "ModificationNotAllowed" service_impl/src/slot.rs | grep -c "max_paid_employees"` returns 0 — verified.
3. **Spread-based fixture variants** for the 3 new tests: `Slot { max_paid_employees: Some(N), ..generate_default_slot() }`. Avoids duplicating the full struct literal and works without `Default` derive (which the plan deliberately rejected).
4. **Rule-3 shims applied to 3 out-of-scope sites** (`test/shiftplan.rs`, `rest-types/src/lib.rs`, `shifty_bin/.../booking_absence_conflict.rs`). See "Deviations from Plan" below.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Forward-compat shim in `service_impl/src/test/shiftplan.rs`**
- **Found during:** Task 2 (`cargo test -p service_impl --lib slot:: --no-run`)
- **Issue:** The plan explicitly forbade touching `test/shiftplan.rs` (owned by Plan 05-04, originally specced as a Wave-2 parallel sibling). However, the `cargo test --lib slot::` filter only filters which tests *run*, not which compile. The whole `service_impl` test target compiles together; once `service::Slot` gained the new mandatory field, the 2 inline `Slot { ... }` literals in `default_slot()` (line 35) and `slot_with_day_and_time()` (line 321) of `test/shiftplan.rs` failed with `error[E0063]: missing field 'max_paid_employees'`. Plan 03 cannot be verified standalone without resolving these.
- **Why the plan's parallelism assumption broke down:** This executor ran sequentially (Plan 04 not yet executed), so Plan 04's owned fixture migration is not yet on disk.
- **Fix:** Added the minimum mechanical `max_paid_employees: None` field to both literals with explicit `// Phase 5 Plan 03 (Rule 3 - blocker fix): ...` comments naming Plan 05-04. The 2 inline literals at lines 503/507 already use `..slot_with_day_and_time(...)` spread, so they auto-inherit and required no edit. Plan 05-04's task list (read-aggregation tests + `current_paid_count` derivation + new tests) remains intact; this shim only resolves the compile blocker.
- **Files modified:** `service_impl/src/test/shiftplan.rs`
- **Verification:** `cargo test -p service_impl --lib slot::` exits 0, all 33 tests pass.
- **Committed in:** change `prpvypok` (commit `3cabfb6b`)

**2. [Rule 3 — Blocking] Forward-compat shim in `rest-types/src/lib.rs`**
- **Found during:** Task 2 (`cargo build` workspace-wide)
- **Issue:** `From<&SlotTO> for service::slot::Slot` failed compilation with `error[E0063]: missing field 'max_paid_employees'` because `SlotTO` does not yet carry the field (Plan 05-05's scope). Without a fix, the entire REST tier and `shifty_bin` would not build.
- **Fix:** Hardcoded `max_paid_employees: None,` in `From<&SlotTO> for service::slot::Slot` and explicit field consumption (`let _ = slot.max_paid_employees;`) in `From<&service::slot::Slot> for SlotTO`. Both carry `// Phase 5 Plan 03 (Rule 3 - blocker fix): ... Plan 05-05 will ...` comments. Plan 05-05's REST-DTO scope is unchanged: it will add the `SlotTO.max_paid_employees` field, replace these shims with real round-trip code, and add the OpenAPI/utoipa metadata.
- **Files modified:** `rest-types/src/lib.rs`
- **Verification:** Workspace `cargo build` passes; `cargo test` workspace-wide green (451+ tests).
- **Committed in:** change `prpvypok` (commit `3cabfb6b`)

**3. [Rule 3 — Blocking] Forward-compat shim in `shifty_bin/src/integration_test/booking_absence_conflict.rs`**
- **Found during:** Task 2 (`cargo test --no-run` after fixing #1 and #2)
- **Issue:** The integration test `booking_absence_conflict::create_monday_slot` constructs an inline `Slot { ... }` literal (no spread); compilation failed with E0063 once `service::Slot` got the new field.
- **Fix:** Added `max_paid_employees: None` (with shim comment) to the inline literal. The integration test does not exercise the paid-cap-warning path (Plan 05-06's scope), so default no-limit semantics are correct.
- **Files modified:** `shifty_bin/src/integration_test/booking_absence_conflict.rs`
- **Verification:** All 56 shifty_bin integration tests pass.
- **Committed in:** change `prpvypok` (commit `3cabfb6b`)

---

**Total deviations:** 3 auto-fixed (all Rule 3 — blocking).

**Impact on plan:** All three deviations are minimal mechanical shims required because the plan's wave-2-parallel assumption (Plan 03 + Plan 04 land their fixture migrations together) does not hold under sequential execution. Plan 05-04 still owns `test/shiftplan.rs` for its read-aggregation logic and new tests; Plan 05-05 still owns `SlotTO`'s field and round-trip; Plan 05-06 still owns the warning-emission integration test changes. The shims are explicitly marked with `// Phase 5 Plan 03 (Rule 3 - blocker fix): ... Plan 05-XX will replace ...` and represent the same forward-compat pattern Plan 05-01 already established.

## Issues Encountered

None — the only friction came from the wave-2-parallel assumption which was resolved deterministically via Rule-3 shims (mirroring Plan 05-01's pattern).

## User Setup Required

None.

## Next Phase Readiness

- **Plan 05-04 (Wave 2):** Inherits `service::Slot.max_paid_employees` populated end-to-end. Plan 05-04 can now add `current_paid_count: u8` to `service::ShiftplanSlot`, derive it inline in `build_shiftplan_day` (using already-resolved `is_paid` per booking), and add its 4 read-aggregation tests in `test/shiftplan.rs`. The 2 mechanical `max_paid_employees: None` shims placed by this plan can stay as-is or be folded into Plan 05-04's migration — either is fine.
- **Plan 05-05 (Wave 3):** Inherits the REST `From<&SlotTO>` shim ready for replacement. Plan 05-05 will add `pub max_paid_employees: Option<u8>` to `SlotTO` (with `#[serde(default)]` for backward-compat per Pattern-Map), replace the shim with real round-trip, and add the `WarningTO` 5th variant.
- **Plan 05-06 (Wave 3):** Inherits a populated `slot.max_paid_employees` to read in `book_slot_with_conflict_check` for the limit check.

## Self-Check: PASSED

- `service::slot::Slot` carries `pub max_paid_employees: Option<u8>` exactly once. Verified.
- Both `From` impls map the field through (no more hardcoded `None` from Plan 05-01). Verified.
- `service_impl/src/slot.rs` line 292-294 still has `SHIFTPLANNER_PRIVILEGE` check. Verified.
- `service_impl/src/slot.rs` `ModificationNotAllowed` block does NOT contain `max_paid_employees` (D-11). Verified via grep.
- `service_impl/src/test/slot.rs` has 4 occurrences of `max_paid_employees: None` (2 default fixtures + 2 helpers) plus the 3 new tests' usages. Verified.
- All 4 cross-file owned fixtures (`booking.rs`, `block.rs`, `absence.rs`, `shiftplan_edit.rs`) carry `max_paid_employees: None`. Verified.
- 3 new tests all run and pass: `test_create_slot_with_paid_limit`, `test_update_slot_changes_max_paid_employees`, `test_update_slot_clears_max_paid_employees`. Verified.
- `cargo build` workspace-wide succeeds. Verified.
- `cargo test` workspace-wide succeeds with 451+ tests passing, 0 failing. Verified.
- jj history shows 2 atomic task changes + 1 docs change (this SUMMARY + STATE/ROADMAP updates). Verified.

---
*Phase: 05-slot-paid-capacity-warning*
*Completed: 2026-05-04*
