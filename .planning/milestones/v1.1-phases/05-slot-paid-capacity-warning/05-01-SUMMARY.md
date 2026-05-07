---
phase: 05-slot-paid-capacity-warning
plan: 01
subsystem: database
tags: [sqlite, sqlx, dao, slot, migration, max_paid_employees]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: stable v1.0 DAO/service architecture (Slot domain unchanged)
provides:
  - "SQLite schema: nullable `slot.max_paid_employees INTEGER` column"
  - "`SlotEntity.max_paid_employees: Option<u8>` field"
  - "All 4 SQLite SlotDao read sites + create_slot INSERT + update_slot UPDATE handle the new field"
affects:
  - 05-03 (Slot service wiring + From impls + service-tier fixture migration)
  - 05-04 (Shiftplan view: current_paid_count derivation)
  - 05-05 (REST DTO surface: SlotTO.max_paid_employees)
  - 05-06 (ShiftplanEditService warning emission consumes max_paid_employees)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Nullable INTEGER column without DEFAULT and without NOT NULL (no backfill)"
    - "SQLx read pattern for nullable INTEGER → `Option<u8>`: `row.col.map(|n| n as u8)`"

key-files:
  created:
    - "migrations/sqlite/20260503221640_add-max-paid-employees-to-slot.sql"
  modified:
    - "dao/src/slot.rs (SlotEntity gains `max_paid_employees: Option<u8>`)"
    - "dao_impl_sqlite/src/slot.rs (4 SELECTs + 4 read constructors + INSERT + UPDATE)"
    - "service/src/slot.rs (Rule 3 patch: From<&Slot> for SlotEntity hardcodes None until Plan 05-03)"
    - "service_impl/src/test/slot.rs (Rule 3 patch: generate_default_slot_entity gets max_paid_employees: None)"

key-decisions:
  - "Apply migration via `nix develop --command sqlx migrate run` (additive, non-destructive)"
  - "Bare `as u8` cast at read sites — consistent with min_resources precedent; no DaoError variant added"
  - "Extend `update_slot` UPDATE to persist max_paid_employees in-place (D-11 implication; no temporal-replay concerns)"
  - "DO NOT touch min_resources gap in update_slot — explicit out-of-scope per CONTEXT.md"
  - "Skip DAO integration test (no `dao_impl_sqlite/tests/` infra) — service-tier mock tests in Plan 05-03+ cover roundtrip"

patterns-established:
  - "Nullable Slot-capacity column: copy `min_resources` migration shape, strip DEFAULT and NOT NULL"
  - "Rule 3 forward-compat shim: when a DAO field is added before its service-layer mirror, hardcode `None` in `From<&Slot> for SlotEntity` with a comment pointing to the follow-up plan"

requirements-completed: [D-01, D-02, D-15]

# Metrics
duration: 7min
completed: 2026-05-04
---

# Phase 5 Plan 01: Foundation (SQLite Migration + SlotEntity + DAO Wiring) Summary

**Nullable `slot.max_paid_employees INTEGER` column added end-to-end through the DAO tier (migration applied, `SlotEntity` extended, all 4 SQLite read sites + INSERT + UPDATE wired).**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-05-04T05:40:54Z
- **Completed:** 2026-05-04T05:47:38Z
- **Tasks:** 3
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments

- New migration `20260503221640_add-max-paid-employees-to-slot.sql` applied to the local SQLite DB; column is nullable with no DEFAULT, leaving all existing slot rows at implicit `NULL` ("no limit").
- `SlotEntity` carries `max_paid_employees: Option<u8>` immediately after `min_resources`, matching the Pattern-Map's recommended slot-capacity grouping.
- `dao_impl_sqlite::SlotDaoImpl` updated at all 4 read sites (`get_slots`, `get_slot`, `get_slots_for_week`, `get_slots_for_week_all_plans`) plus `create_slot` INSERT and `update_slot` UPDATE — the field round-trips cleanly through SQLite.
- 448 workspace tests pass (`cargo test`): 363 service_impl + 56 shifty_bin integration + 11 cutover service + 10 dao + 8 other.

## Task Commits

Each task was committed atomically via `jj`:

1. **Task 1: Add SQLite migration for nullable max_paid_employees column** — change `kwzorvor` (commit `d323f166`) — `feat(05-01)`
2. **Task 2: Add max_paid_employees field to SlotEntity (DAO trait)** — change `sxrnxsqm` (commit `b055027f`) — `feat(05-01)`
3. **Task 3: Wire max_paid_employees through SQLite SlotDao impl** — change `nvmyyxyq` (commit `d8bfe07e`) — `feat(05-01)`

**Plan metadata:** change `yzknospy` — `docs(05-01)` (this SUMMARY + STATE/ROADMAP updates).

_Note: Task 3 was marked `tdd="true"` in the plan, but the plan explicitly allows skipping the DAO integration test if no `dao_impl_sqlite/tests/` infrastructure exists ("the behavior is instead exercised at the service-tier mock layer in Plan 03"). The directory does not exist, so the test was deliberately skipped — Plan 05-03 will exercise the roundtrip through service-tier mock tests._

## Files Created/Modified

- **Created:** `migrations/sqlite/20260503221640_add-max-paid-employees-to-slot.sql` — `ALTER TABLE slot ADD COLUMN max_paid_employees INTEGER` (no DEFAULT, no NOT NULL).
- **Modified:** `dao/src/slot.rs` — added `pub max_paid_employees: Option<u8>` to `SlotEntity` immediately after `min_resources`.
- **Modified:** `dao_impl_sqlite/src/slot.rs`:
  - Edits A1–A4: SELECT-list now includes `max_paid_employees` between `min_resources` and `valid_from` at all four query sites (`get_slots` line 29; `get_slot` line 67; `get_slots_for_week` line 111; `get_slots_for_week_all_plans` line 159).
  - Edit B (×4): each `SlotEntity { … }` constructor (around lines 36–56, 73–94, 122–144, 171–194) now sets `max_paid_employees: row.max_paid_employees.map(|n| n as u8),` immediately after `min_resources`.
  - Edit C: `create_slot` (around lines 197–232) introduces a `let max_paid_employees = slot.max_paid_employees;` binding, adds the column to the INSERT column-list (between `min_resources` and `shiftplan_id`), and adds a corresponding bind argument and `?` placeholder.
  - Edit D: `update_slot` (around lines 234–256) adds `let max_paid_employees = slot.max_paid_employees;` and extends the UPDATE SET clause to include `max_paid_employees = ?` between `deleted = ?` and `update_version = ?`. **`min_resources` UPDATE gap explicitly NOT touched** — out of scope per `05-CONTEXT.md` "Strikt nicht in Scope".
- **Modified (Rule 3 deviation):** `service/src/slot.rs` — `From<&Slot> for SlotEntity` hardcodes `max_paid_employees: None` with a comment pointing at Plan 05-03 (which will add the field to `service::Slot` and replace the hardcoded `None` with `slot.max_paid_employees`).
- **Modified (Rule 3 deviation):** `service_impl/src/test/slot.rs` — `generate_default_slot_entity()` fixture sets `max_paid_employees: None` so the full test suite continues to compile.

## Decisions Made

- Skipped the DAO-tier integration test for Task 3's TDD spec (no `dao_impl_sqlite/tests/` infrastructure exists; the plan explicitly defers to service-tier coverage in Plan 05-03).
- Used the bare `as u8` cast (`row.max_paid_employees.map(|n| n as u8)`) consistent with the existing `min_resources` precedent — Pattern-Map noted a `try_from`+`DaoError` variant as belt-and-suspenders but explicitly recommended against introducing it for a single phase.
- Extended `update_slot` UPDATE to persist `max_paid_employees` in-place (Pitfalls Summary #3 + D-11). The `min_resources` gap remains untouched — it's explicitly out of Phase-5 scope.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Forward-compat `None` placeholder for `service::Slot → SlotEntity`**
- **Found during:** Task 3 (workspace `cargo build` after extending SlotEntity)
- **Issue:** `dao::SlotEntity` gained the new mandatory `max_paid_employees` field, but `service::Slot` (Plan 05-03's scope) has not been extended yet. The `From<&Slot> for SlotEntity` impl in `service/src/slot.rs` therefore failed to compile, blocking the workspace build (`error[E0063]: missing field 'max_paid_employees'`).
- **Fix:** Hardcoded `max_paid_employees: None` in the `From<&Slot> for SlotEntity` impl with an inline comment pointing to Plan 05-03, which will replace it with `slot.max_paid_employees` once the field is added to `service::Slot`.
- **Files modified:** `service/src/slot.rs`
- **Verification:** `cargo build` workspace-wide passes; downstream `cargo test` shows 448 tests green (no behavioural regression — slots still create without a paid limit, identical to pre-Phase-5 behaviour).
- **Committed in:** change `nvmyyxyq` (Task 3 commit `d8bfe07e`)

**2. [Rule 3 — Blocking] Test-fixture `None` for `generate_default_slot_entity`**
- **Found during:** Task 3 (workspace `cargo build --tests`)
- **Issue:** `service_impl/src/test/slot.rs::generate_default_slot_entity()` constructs a `SlotEntity` literal that did not include the new field, breaking compilation of all 363 `service_impl` lib tests (`error[E0063]`).
- **Fix:** Added `max_paid_employees: None` to the fixture with an inline comment noting Plan 05-03 will add a paid-limit fixture variant.
- **Files modified:** `service_impl/src/test/slot.rs`
- **Verification:** `cargo build --tests` succeeds; `cargo test` shows 363/363 service_impl tests pass.
- **Committed in:** change `nvmyyxyq` (Task 3 commit `d8bfe07e`)

---

**Total deviations:** 2 auto-fixed (both Rule 3 — blocking)
**Impact on plan:** Both fixes are minimal forward-compat shims that keep the workspace and test suite green between Plan 05-01 (DAO tier) and Plan 05-03 (service tier). The plan acknowledged the DAO/service split, but did not call out that the existing `From<&Slot> for SlotEntity` impl and the central test fixture would need transitional `None` values. Plan 05-03 must replace both with the real `slot.max_paid_employees` flow. No scope creep — `service::Slot` was NOT extended here.

## Issues Encountered

None — the plan executed as written. The two auto-fixes were minimal, mechanical, and isolated to the two specific compile sites where the new mandatory field needed a placeholder.

## User Setup Required

None — migration was applied to the local DB during Task 1 via `nix develop --command sqlx migrate run --source migrations/sqlite`. No environment variables, dashboard config, or external service changes.

## Next Phase Readiness

- **Wave 2 (Plans 05-03 and 05-04) unblocked:** the DAO tier is done; both plans can now extend `service::Slot` and `service_impl/src/shiftplan.rs` against a stable `SlotEntity`/`SlotDao` surface.
- Plan 05-03 must replace the two `None` placeholders introduced as Rule 3 fixes (`service/src/slot.rs:From<&Slot> for SlotEntity` and `service_impl/src/test/slot.rs:generate_default_slot_entity`) with `slot.max_paid_employees` once `service::Slot` carries the field. Verify via grep: `grep -n "Phase 5 Plan 01 (Rule 3" service/src/slot.rs service_impl/src/test/slot.rs`.
- Plan 05-04 inherits a clean `SlotEntity` carrying the field — no additional foundation work needed.

## Self-Check: PASSED

- File `migrations/sqlite/20260503221640_add-max-paid-employees-to-slot.sql` exists and contains `ADD COLUMN max_paid_employees INTEGER` with no `NOT NULL` / `DEFAULT`. Verified.
- Schema applied: `nix develop --command sqlite3 localdb.sqlite3 ".schema slot"` shows `max_paid_employees INTEGER` in the `slot` table. Verified.
- `dao/src/slot.rs` has `pub max_paid_employees: Option<u8>` exactly once, positioned after `pub min_resources`. Verified.
- `dao_impl_sqlite/src/slot.rs` has 14 occurrences of `max_paid_employees`, 4 read-site `row.max_paid_employees.map(|n| n as u8)`, INSERT and UPDATE both contain the column. Verified.
- `cargo build` workspace-wide passes. Verified.
- `cargo test` workspace-wide: 448 tests pass, 0 failed. Verified.
- jj history shows 4 atomic changes for plan 05-01 (3 task changes + 1 docs change for SUMMARY/STATE/ROADMAP). Verified via `jj log --limit 5`.

---
*Phase: 05-slot-paid-capacity-warning*
*Completed: 2026-05-04*
