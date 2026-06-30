---
phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration
plan: 01
subsystem: database, api
tags: [toggle, sqlite, migration, rest, axum, sqlx, utoipa]

requires:
  - phase: 24-paid-capacity-enforcement-config
    provides: "Toggle infrastructure (DAO/Service/REST/migration pattern)"

provides:
  - "nullable value TEXT column on toggle table (migration 20260628000000)"
  - "holiday_auto_credit toggle seeded disabled with NULL value (migration 20260628000001)"
  - "ToggleEntity.value / Toggle.value / ToggleTO.value fields through full stack"
  - "get_toggle_value / set_toggle_value at DAO, Service, REST tiers"
  - "GET/PUT/DELETE /toggle/{name}/value endpoints, toggle_admin-gated, ISO date validated"
  - "enabled mirrors value presence (D-25-05): set=enabled, clear=disabled"
  - "regenerated .sqlx offline query cache"

affects:
  - 25-02-PLAN.md (reads holiday_auto_credit value for derive-on-read reporting)
  - 25-03-PLAN.md (frontend settings field writes this value)

tech-stack:
  added: []
  patterns:
    - "Toggle value column: nullable TEXT in toggle table, value presence drives enabled flag"
    - "ISO date validation in REST handler via time::Date::from_calendar_date without macros feature"
    - "GET value returns 200+JSON when set, 204 when absent"

key-files:
  created:
    - migrations/sqlite/20260628000000_toggle-value-column.sql
    - migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql
  modified:
    - dao/src/toggle.rs
    - dao_impl_sqlite/src/toggle.rs
    - service/src/toggle.rs
    - service_impl/src/toggle.rs
    - service_impl/src/test/toggle.rs
    - rest-types/src/lib.rs
    - rest/src/toggle.rs
    - .sqlx/ (regenerated offline cache)

key-decisions:
  - "PUT /toggle/{name}/value validates ISO date with time::Date::from_calendar_date (no extra time features needed)"
  - "GET returns 204 (not 200+null) when value absent — cleaner than JSON null for unset state"
  - "set_toggle_value(None) calls DAO which sets enabled=0; set_toggle_value(Some) sets enabled=1 (D-25-05)"
  - "Service set_toggle_value takes Option<String> (owned); REST layer passes owned string from JSON body"

patterns-established:
  - "toggle value REST handlers: get_toggle_value / set_toggle_value / clear_toggle_value follow enable/disable shape"

requirements-completed: [HCFG-01, HCFG-02]

coverage:
  - id: D1
    description: "toggle table has nullable value TEXT column; holiday_auto_credit seeded disabled (HCFG-01)"
    requirement: HCFG-01
    verification:
      - kind: integration
        ref: "migrations/sqlite/20260628000000_toggle-value-column.sql + 20260628000001_seed-holiday-auto-credit-toggle.sql applied to dev DB"
        status: pass
    human_judgment: false
  - id: D2
    description: "get_toggle_value / set_toggle_value round-trip through service layer; setting enables, clearing disables (D-25-05)"
    requirement: HCFG-01
    verification:
      - kind: unit
        ref: "service_impl::test::toggle::test_toggle_value_roundtrip"
        status: pass
      - kind: unit
        ref: "service_impl::test::toggle::test_set_toggle_value_requires_toggle_admin_privilege"
        status: pass
      - kind: unit
        ref: "service_impl::test::toggle::test_get_toggle_value_requires_authentication"
        status: pass
    human_judgment: false
  - id: D3
    description: "Admin can PUT/DELETE the stichtag value; non-admin rejected; malformed ISO date refused with 400 (HCFG-02)"
    requirement: HCFG-02
    verification:
      - kind: unit
        ref: "service_impl::test::toggle::test_set_toggle_value_requires_toggle_admin_privilege"
        status: pass
    human_judgment: true
    rationale: "REST-level 400 rejection of invalid dates and 403 for non-admin require manual HTTP verification against running server"
  - id: D4
    description: "All existing toggle SELECT/UPDATE queries compile under SQLX_OFFLINE=true after value column added"
    requirement: HCFG-01
    verification:
      - kind: unit
        ref: "SQLX_OFFLINE=true cargo build --workspace (passes)"
        status: pass
    human_judgment: false

duration: 45min
completed: 2026-06-28
status: complete
---

# Phase 25 Plan 01: Toggle Value Column + REST Value Endpoints Summary

**Nullable `value TEXT` column added to toggle table with `holiday_auto_credit` seeded off; `get_toggle_value`/`set_toggle_value` wired through DAO->Service->REST with ISO date validation and toggle_admin gate**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-06-28T00:00:00Z
- **Completed:** 2026-06-28
- **Tasks:** 3 (all swept into single commit — WIP was already partially present)
- **Files modified:** 9 source files + 2 migration files + .sqlx/ cache

## Accomplishments

- Migration adds nullable `value TEXT` to toggle table; `holiday_auto_credit` seeded disabled (HCFG-01 default off)
- `ToggleEntity.value` / `Toggle.value` / `ToggleTO.value` fields added through full DAO->Service->REST-types stack
- `get_toggle_value` and `set_toggle_value` implemented at DAO and Service tiers with proper permission gates (auth for reads, toggle_admin for writes)
- `set_toggle_value` mirrors `enabled` to value presence (D-25-05): Some(date) sets enabled=1, None sets enabled=0
- Three admin-gated REST endpoints: `GET /toggle/{name}/value` (200+body or 204), `PUT /toggle/{name}/value` (ISO date validated, 400 on bad input), `DELETE /toggle/{name}/value` (clears + disables)
- All three handlers have `#[utoipa::path]` annotations and are registered in `ToggleApiDoc`
- 3 new service-layer tests including `test_toggle_value_roundtrip` (set->get->clear->verify-disabled)
- `.sqlx` offline cache regenerated; workspace builds and all tests pass under `SQLX_OFFLINE=true`

## Task Commits

All plan work was committed in a single sweep commit:

1. **Tasks 1-3: Toggle value column + service + REST endpoints** - `9067ce6` (feat)

## Files Created/Modified

- `migrations/sqlite/20260628000000_toggle-value-column.sql` - `ALTER TABLE toggle ADD COLUMN value TEXT;`
- `migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql` - Seeds `holiday_auto_credit` (disabled, NULL value)
- `dao/src/toggle.rs` - `ToggleEntity.value: Option<String>` + `get_toggle_value`/`set_toggle_value` trait methods
- `dao_impl_sqlite/src/toggle.rs` - `ToggleDb.value`, all SELECT queries include `value`, `update_toggle` persists `value`, get/set impls
- `service/src/toggle.rs` - `Toggle.value: Option<Arc<str>>`, both From impls, trait methods
- `service_impl/src/toggle.rs` - `get_toggle_value`/`set_toggle_value` implementations
- `service_impl/src/test/toggle.rs` - Fixed struct initializers (`value: None`) + 3 new tests including roundtrip
- `rest-types/src/lib.rs` - `ToggleTO.value: Option<Arc<str>>` + both From impls
- `rest/src/toggle.rs` - `get_toggle_value`/`set_toggle_value`/`clear_toggle_value` handlers + route registration + OpenAPI
- `.sqlx/` - Regenerated offline query cache (4 renames, 2 new files)

## Decisions Made

- Used `time::Date::from_calendar_date` for ISO date validation in PUT handler — no extra `time` crate features required beyond what `rest/Cargo.toml` already enables via `serde-human-readable`
- GET returns 204 (empty) when value is absent, 200+JSON string when set — consistent with how disabled-state is "no data" not "null data"
- `set_toggle_value` service method accepts `Option<String>` (owned); REST DELETE handler passes `None` to the same method (single code path)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Test struct initializers missing `value` field**
- **Found during:** Pre-implementation check
- **Issue:** `default_toggle_entity()` and `default_toggle()` in `service_impl/src/test/toggle.rs` were missing `value: None` — would cause E0063 compile error
- **Fix:** Added `value: None` to both struct literal initializers
- **Files modified:** service_impl/src/test/toggle.rs
- **Verification:** `cargo test --package service_impl toggle` — 20 tests pass
- **Committed in:** 9067ce6

---

**Total deviations:** 1 auto-fixed (Rule 3 — blocking compile error)
**Impact on plan:** Fix was pre-existing WIP bug, not caused by new code. No scope creep.

## Issues Encountered

None — all gates passed cleanly after fixing the struct initializer.

## Threat Surface Scan

No new network endpoints beyond the planned three toggle value routes. All parameterized via SQLx (no string interpolation). ISO date validation guards against T-25-01 (input tampering). Admin gate enforces T-25-02 (elevation of privilege). No unplanned trust boundary extensions.

## Self-Check

- [x] `migrations/sqlite/20260628000000_toggle-value-column.sql` — EXISTS
- [x] `migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql` — EXISTS
- [x] `dao/src/toggle.rs` — has `value: Option<String>` and trait methods
- [x] `dao_impl_sqlite/src/toggle.rs` — has ToggleDb.value + get/set impls
- [x] `service/src/toggle.rs` — has Toggle.value + service trait methods
- [x] `service_impl/src/toggle.rs` — has both impls
- [x] `rest/src/toggle.rs` — has 3 handlers registered in routes + OpenAPI
- [x] Commit `9067ce6` — EXISTS
- [x] `cargo build --workspace SQLX_OFFLINE=true` — PASSED
- [x] `cargo test --workspace` — 20 toggle tests pass + all others green
- [x] `cargo clippy --workspace -- -D warnings` — CLEAN

## Self-Check: PASSED

## Next Phase Readiness

- Phase 25-02 (derive-on-read reporting) can now read `holiday_auto_credit` toggle value via `toggle_service.get_toggle_value("holiday_auto_credit", ...)` to determine the activation date
- Phase 25-03 (frontend settings UI) can now write the stichtag via `PUT /toggle/holiday_auto_credit/value` and clear it via `DELETE /toggle/holiday_auto_credit/value`

---
*Phase: 25-feiertags-auto-anrechnung-stichtag-konfiguration*
*Completed: 2026-06-28*
