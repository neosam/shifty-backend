---
phase: 33-special-days-ui-einstellungen
plan: "01"
subsystem: api
tags: [rust, axum, sqlx, mockall, utoipa, special-days, tdd]

requires: []
provides:
  - "GET /special-days/for-year/{year} endpoint — DAO find_by_year → Service get_by_year → REST handler + route + ApiDoc"
  - "5-test mockall unit suite covering get_by_year delegation, shiftplanner gate on create/delete, nil-id guard, not-found guard"
affects: [33-02, 33-03, 33-04]

tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN cycle: trait stub (todo!()) → failing tests → real impl → all green"
    - "find_by_year DAO: single year param, WHERE deleted IS NULL, ORDER BY calendar_week ASC, day_of_week ASC"
    - "get_by_year service: ungated read (_context unused), delegates to DAO and maps SpecialDayEntity → SpecialDay"

key-files:
  created:
    - "service_impl/src/test/special_days.rs — 5 mockall unit tests"
  modified:
    - "dao/src/special_day.rs — find_by_year trait method"
    - "dao_impl_sqlite/src/special_day.rs — find_by_year SQLx impl"
    - "service/src/special_days.rs — get_by_year trait method"
    - "service_impl/src/special_days.rs — get_by_year impl"
    - "rest/src/special_day.rs — get_special_days_for_year handler + /for-year/{year} route + SpecialDayApiDoc entry"
    - "service_impl/src/test/mod.rs — pub mod special_days registration"

key-decisions:
  - "D-33-05: GET /special-days/for-year/{year} added as ungated read (consistent with for-week), DAO orders by (calendar_week, day_of_week)"
  - "D-33-01: shiftplanner gate on create/delete regression-tested; no backend permission change needed"

patterns-established:
  - "for-year chain: exact clone of for-week (DAO→Service→REST→ApiDoc), minus calendar_week param"
  - "REST: both #[utoipa::path] annotation AND ApiDoc paths() entry required — missing either breaks Swagger silently"

requirements-completed: [SPD-01, SPD-02, SPD-03, SPD-04]

coverage:
  - id: D1
    description: "find_by_year DAO method returns all non-deleted special days for a year ordered by (calendar_week, day_of_week); SQLx compile-time checked"
    requirement: SPD-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_get_by_year_delegates_and_maps"
        status: pass
      - kind: unit
        ref: "cargo build -p dao_impl_sqlite (SQLx macro validates query against offline schema)"
        status: pass
    human_judgment: false
  - id: D2
    description: "get_by_year service method delegates to find_by_year and maps entities to SpecialDay (ungated)"
    requirement: SPD-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_get_by_year_delegates_and_maps"
        status: pass
    human_judgment: false
  - id: D3
    description: "create is shiftplanner-gated: caller without shiftplanner privilege receives ServiceError::Forbidden"
    requirement: SPD-04
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_create_forbidden_without_shiftplanner"
        status: pass
    human_judgment: false
  - id: D4
    description: "create rejects non-nil id with ServiceError::IdSetOnCreate"
    requirement: SPD-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_create_rejects_nonnil_id"
        status: pass
    human_judgment: false
  - id: D5
    description: "delete is shiftplanner-gated: caller without shiftplanner privilege receives ServiceError::Forbidden"
    requirement: SPD-04
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_delete_forbidden_without_shiftplanner"
        status: pass
    human_judgment: false
  - id: D6
    description: "delete of missing/deleted id returns ServiceError::EntityNotFound"
    requirement: SPD-03
    verification:
      - kind: unit
        ref: "service_impl/src/test/special_days.rs#test_delete_not_found"
        status: pass
    human_judgment: false
  - id: D7
    description: "REST handler get_special_days_for_year wired at /for-year/{year} with #[utoipa::path] and SpecialDayApiDoc entry"
    requirement: SPD-02
    verification:
      - kind: unit
        ref: "cargo build -p rest (utoipa macro validates ApiDoc + handler signature)"
        status: pass
    human_judgment: false

duration: 14min
completed: "2026-06-30"
status: complete
---

# Phase 33 Plan 01: Special-Days Backend for-year Endpoint Summary

**for-year read chain DAO→Service→REST wired end-to-end with 5 mockall regression tests covering SPD-01..04 shiftplanner gate, nil-id guard, not-found guard, and year delegation**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-30T12:55:57Z
- **Completed:** 2026-06-30T13:09:39Z
- **Tasks:** 3 (RED → GREEN → REST)
- **Files modified:** 7

## Accomplishments

- `find_by_year` DAO method added to trait and implemented in SQLite DAO with SQLx compile-time checked SQL (`WHERE year = ? AND deleted IS NULL ORDER BY calendar_week ASC, day_of_week ASC`)
- `get_by_year` service method added to trait and implemented in SpecialDayServiceImpl (ungated read, delegates to DAO, maps to SpecialDay)
- REST handler `get_special_days_for_year` wired at `/for-year/{year}` with `#[utoipa::path]` annotation and ApiDoc entry
- 5-test mockall suite in `service_impl/src/test/special_days.rs` covering all SPD-01..04 behaviors (D-33-05 delegation, D-33-01 shiftplanner gates, SPD-01 nil-id, SPD-03 not-found)

## Task Commits

1. **Task 1 (RED): Add signatures + failing test module** — `3d67191` (test)
2. **Task 2 (GREEN): Implement find_by_year DAO + get_by_year service** — `751ecdb` (feat)
3. **Task 3: Wire REST handler + route + ApiDoc** — `dfe09cb` (feat)

## Files Created/Modified

- `service_impl/src/test/special_days.rs` — NEW: 5 mockall unit tests
- `service_impl/src/test/mod.rs` — added `pub mod special_days` registration
- `dao/src/special_day.rs` — added `find_by_year` to SpecialDayDao trait
- `dao_impl_sqlite/src/special_day.rs` — implemented `find_by_year` with SQLx query_as!
- `service/src/special_days.rs` — added `get_by_year` to SpecialDayService trait
- `service_impl/src/special_days.rs` — implemented `get_by_year` (ungated, delegates to DAO)
- `rest/src/special_day.rs` — added handler, route `/for-year/{year}`, ApiDoc entry

## Decisions Made

- Followed D-33-05 exactly: ungated read consistent with `get_by_week`; SQL orders by `(calendar_week, day_of_week)` ascending
- Used `().into()` auth pattern (via `From<Context> for Authentication<Context>`) in tests
- Used `test_forbidden`, `test_not_found`, `test_zero_id_error` helpers from `error_test.rs` (ServiceError doesn't impl PartialEq)

## Deviations from Plan

None — plan executed exactly as written.

The only minor adaptation was using the existing error-test helper functions (`test_forbidden`, etc.) instead of `assert_eq!` on `ServiceError` (which doesn't implement `PartialEq`). This is consistent with all other test modules in the codebase and not a deviation from intent.

## Issues Encountered

- Initial test file used `assert_eq!` on `ServiceError` (no PartialEq impl) — auto-fixed by switching to `test_forbidden`, `test_not_found`, `test_zero_id_error` helper functions from `error_test.rs` (Rule 1: auto-fix compile error)
- Initial test file imported `NoneTypeExt` but used `().into()` — fixed by removing unused import and adding `use service::special_days::SpecialDayService` for method dispatch

## Next Phase Readiness

- Backend for-year endpoint complete; FE api.rs can call `GET /special-days/for-year/{year}` in Plans 02-04
- Plans 02-03-04 can proceed: Settings Card-3 (Plan 02), Shiftplan dropdown (Plan 03), i18n (Plan 04)
- No blockers

## Self-Check: PASSED

- `service_impl/src/test/special_days.rs` — FOUND
- `rest/src/special_day.rs` contains `get_special_days_for_year` — FOUND (3 occurrences)
- Commits 3d67191, 751ecdb, dfe09cb — all present in git log
- `cargo test -p service_impl special_days` — 5 tests pass
- `cargo clippy --workspace -- -D warnings` — clean

---
*Phase: 33-special-days-ui-einstellungen*
*Completed: 2026-06-30*
