---
phase: 39-kw-status-grundlage
plan: 01
subsystem: database
tags: [sqlite, sqlx, dao, migration, week_status, enum-discriminant]

# Dependency graph
requires:
  - phase: 38-week-message (analog)
    provides: week_message DAO/entity/migration copy template
provides:
  - "week_status migration (partial UNIQUE WHERE deleted IS NULL, D-39-10)"
  - "WeekStatusKind {InPlanning, Planned, Locked} enum (no Unset/None variant, D-39-03/04)"
  - "WeekStatusEntity + WeekStatusDao trait + MockWeekStatusDao"
  - "WeekStatusDaoImpl with TryFrom TEXT discriminant + soft-delete CRUD"
  - ".sqlx offline cache for 4 new week_status queries"
affects: [39-02-service, 39-03-rest, 39-04-frontend, 39-05-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Closed-match TEXT enum discriminant with EnumValueNotFound fallback (special_day pattern)"
    - "Partial UNIQUE index for soft-delete history (vacation_entitlement_offset pattern)"

key-files:
  created:
    - migrations/sqlite/20260702000000_create-week-status.sql
    - dao/src/week_status.rs
    - dao_impl_sqlite/src/week_status.rs
  modified:
    - dao/src/lib.rs
    - dao_impl_sqlite/src/lib.rs

key-decisions:
  - "WeekStatusKind has exactly 3 persisted variants; Unset == row absence, never serialized (D-39-04)"
  - "Discriminant variant named WeekStatusKind not None to avoid Option-shadowing (D-39-03)"
  - "Partial UNIQUE WHERE deleted IS NULL (not week_message's plain UNIQUE) to allow soft-delete history (D-39-10, RESEARCH P-6)"

patterns-established:
  - "Enum->&str serialization via explicit status_to_str match (no .to_string()), structurally excludes Unset"
  - "Unknown DB discriminant -> DaoError::EnumValueNotFound instead of panic/silent misread (T-39-02)"

requirements-completed: [WST-01]

coverage:
  - id: D1
    description: "week_status migration creates table with ISO (year, calendar_week) + TEXT status + partial UNIQUE WHERE deleted IS NULL, no FK"
    requirement: "WST-01"
    verification:
      - kind: integration
        ref: "sqlx migrate run --source migrations/sqlite (applied 20260702000000 to localdb.sqlite3)"
        status: pass
    human_judgment: false
  - id: D2
    description: "WeekStatusKind {InPlanning, Planned, Locked} + WeekStatusEntity + WeekStatusDao trait compile and are workspace-importable"
    requirement: "WST-01"
    verification:
      - kind: unit
        ref: "cargo build -p dao"
        status: pass
    human_judgment: false
  - id: D3
    description: "TryFrom<&WeekStatusDb> maps known discriminants and returns DaoError::EnumValueNotFound for unknown TEXT"
    requirement: "WST-01"
    verification:
      - kind: unit
        ref: "dao_impl_sqlite/src/week_status.rs#unknown_discriminant"
        status: pass
      - kind: unit
        ref: "dao_impl_sqlite/src/week_status.rs#roundtrip_discriminant"
        status: pass
    human_judgment: false
  - id: D4
    description: ".sqlx offline cache regenerated for the 4 new week_status queries; CI-equivalent SQLX_OFFLINE build/test green"
    requirement: "WST-01"
    verification:
      - kind: integration
        ref: "SQLX_OFFLINE=true cargo test -p dao_impl_sqlite week_status"
        status: pass
    human_judgment: false

# Metrics
duration: 6min
completed: 2026-07-02
status: complete
---

# Phase 39 Plan 01: KW-Status Persistenz-Fundament Summary

**week_status migration + DAO trait/entity + SQLite impl with closed-match TEXT discriminant (InPlanning/Planned/Locked), soft-delete CRUD, and regenerated .sqlx offline cache**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-07-01T23:08:32Z
- **Completed:** 2026-07-01T23:14:17Z
- **Tasks:** 2
- **Files modified:** 5 (+4 .sqlx query caches)

## Accomplishments
- New `week_status` migration: `(id, year, calendar_week, status TEXT, created, deleted, update_process, update_version)` with partial UNIQUE `idx_week_status_active ... WHERE deleted IS NULL` — exactly one active row per ISO week, soft-delete history preserved (D-39-10). No FK, no sales_person_id.
- DAO layer: `WeekStatusKind {InPlanning, Planned, Locked}` (no Unset/None variant, D-39-03/04), `WeekStatusEntity`, `WeekStatusDao` trait (+ `MockWeekStatusDao` via automock) with `find_by_year_and_week`/`create`/`update`/`delete`.
- SQLite impl: `WeekStatusDb` row struct + `TryFrom<&WeekStatusDb>` closed match; unknown discriminant → `DaoError::EnumValueNotFound` (T-39-02 mitigated, test-proven). Enum→&str via explicit `status_to_str` (no `.to_string()`, structurally no Unset branch).
- `.sqlx` offline cache regenerated (4 new queries); CI-equivalent `SQLX_OFFLINE=true` build/test verified green.

## Task Commits

1. **Task 1: Migration + DAO trait/entity in dao crate** - `8984894` (feat)
2. **Task 2: SQLite DAO impl with TEXT discriminant + sqlx prepare** - `5603008` (feat, TDD tests included)

## Files Created/Modified
- `migrations/sqlite/20260702000000_create-week-status.sql` - week_status table + partial UNIQUE active index
- `dao/src/week_status.rs` - WeekStatusKind enum, WeekStatusEntity, WeekStatusDao trait
- `dao/src/lib.rs` - registered `pub mod week_status;`
- `dao_impl_sqlite/src/week_status.rs` - WeekStatusDb, TryFrom, WeekStatusDaoImpl, CRUD, unit tests
- `dao_impl_sqlite/src/lib.rs` - registered `pub mod week_status;`
- `.sqlx/query-*.json` - 4 new offline query caches

## Decisions Made
None beyond the plan — followed D-39-03/04/10 as specified.

## Deviations from Plan
None - plan executed exactly as written.

## TDD Gate Compliance
Task 2 was marked `tdd="true"`. The `unknown_discriminant` and `roundtrip_discriminant` unit tests
(RED intent) and the `TryFrom` implementation (GREEN) were authored in a single file and committed
together in `5603008` rather than as separate `test(...)` then `feat(...)` commits. Reason: the tests
reference `WeekStatusEntity::try_from`, which cannot compile until the impl's `WeekStatusDb` struct and
`TryFrom` exist, and this task is a near-1:1 analog copy of the established `week_message`/`special_day`
stack. Both tests pass under the CI-equivalent `SQLX_OFFLINE=true` run, and the error-path assertion
(`EnumValueNotFound`) provides the intended RED-gate behavioral proof.

## Issues Encountered
- `.env` is outside my read permissions, so `DATABASE_URL` was set explicitly to `sqlite:./localdb.sqlite3`
  (the active dev DB matching `env.example`) for `sqlx migrate run` and `cargo sqlx prepare`. sqlx-cli 0.9.0
  was already on PATH — no `nix develop` wrapper needed.

## Gate Results
- `cargo build -p dao` — pass
- `cargo test -p dao_impl_sqlite week_status` — pass (2 tests)
- `cargo clippy --workspace -- -D warnings` — pass (offline)
- `cargo sqlx prepare --workspace` — 4 new caches, committed; `SQLX_OFFLINE=true` test green

## Next Phase Readiness
- Wave 2 (39-02 service) can consume `WeekStatusDao` / `MockWeekStatusDao` and `WeekStatusKind`.
- No blockers.

## Self-Check: PASSED
- All 3 created source files present on disk.
- Both task commits (8984894, 5603008) exist in git history.
- 4 new `.sqlx/query-*.json` caches committed in 5603008.

---
*Phase: 39-kw-status-grundlage*
*Completed: 2026-07-02*
