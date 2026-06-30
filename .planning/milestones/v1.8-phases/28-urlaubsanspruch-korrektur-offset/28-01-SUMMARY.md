---
phase: 28-urlaubsanspruch-korrektur-offset
plan: 01
subsystem: backend-data-layer
tags: [dao, sqlite, service, basic-service, soft-delete, hr-gate, migration, sqlx]
requires: []
provides:
  - "table vacation_entitlement_offset (id PK, sales_person_id, year, offset_days signed, soft-delete cols, partial unique index)"
  - "dao::vacation_entitlement_offset::VacationEntitlementOffsetEntity"
  - "dao::vacation_entitlement_offset::VacationEntitlementOffsetDao (trait, #[automock])"
  - "dao_impl_sqlite::vacation_entitlement_offset::VacationEntitlementOffsetDaoImpl"
  - "service::vacation_entitlement_offset::VacationEntitlementOffset (domain)"
  - "service::vacation_entitlement_offset::VacationEntitlementOffsetService (trait, #[automock] -> MockVacationEntitlementOffsetService)"
  - "service_impl::vacation_entitlement_offset::VacationEntitlementOffsetServiceImpl (Basic, gen_service_impl!)"
affects:
  - "Plan 28-02 (VacationBalanceService consumes the offset Basic service)"
tech-stack:
  added: []
  patterns:
    - "Per-(person,year) soft-delete aggregate cloned from carryover (+id PK +partial unique index)"
    - "Basic (Entity-Manager) service via gen_service_impl! — deps = Dao/Permission/Clock/Uuid/Transaction only"
    - "HR_PRIVILEGE gate on every method (read + write) before any DAO access"
    - "sqlx compile-time query!/query_as! with SQLX_OFFLINE=true + checked-in .sqlx cache"
key-files:
  created:
    - migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql
    - dao/src/vacation_entitlement_offset.rs
    - dao_impl_sqlite/src/vacation_entitlement_offset.rs
    - service/src/vacation_entitlement_offset.rs
    - service_impl/src/vacation_entitlement_offset.rs
    - service_impl/src/test/vacation_entitlement_offset.rs
    - ".sqlx/ (4 new query cache entries)"
  modified:
    - dao/src/lib.rs
    - dao_impl_sqlite/src/lib.rs
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
decisions:
  - "D-28-01: id BLOB PK + partial UNIQUE INDEX (sales_person_id, year) WHERE deleted IS NULL (RESEARCH-recommended form A1)"
  - "CRUD write model (find -> create-or-update) chosen over carryover's ON CONFLICT upsert, to avoid conflicting with the partial index"
  - "delete-not-found returns ServiceError::EntityNotFoundGeneric (no id available for the (person,year) key)"
metrics:
  duration: ~25m
  completed: 2026-06-29
status: complete
---

# Phase 28 Plan 01: Vacation-Entitlement-Offset Data Layer Summary

Snapshot-safe data layer for the per-(person, year) signed vacation-entitlement offset: an additive soft-delete table, its DAO trait + sqlite impl, and a HR-gated Basic `VacationEntitlementOffsetService` performing find/get/set/delete — all green under build/test/clippy.

## What was built

**Task 1 — Migration + DAO (commit uvtqmurw b0c8f398, `feat`)**
- `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql`: table `vacation_entitlement_offset` with `id BLOB PK`, `sales_person_id`, `year`, signed `offset_days INTEGER`, `created`, `deleted`, `update_process`, `update_version`, FK to `sales_person(id)`, plus a partial `UNIQUE INDEX (sales_person_id, year) WHERE deleted IS NULL`. Applied additively with `sqlx migrate run` (no reset).
- `dao/src/vacation_entitlement_offset.rs`: `VacationEntitlementOffsetEntity` + `VacationEntitlementOffsetDao` trait (`find_by_sales_person_id_and_year`, `find_by_id`, `create`, `update`) with `#[automock]`.
- `dao_impl_sqlite/src/vacation_entitlement_offset.rs`: `VacationEntitlementOffsetDaoImpl` + `Db` row + `TryFrom<&Db>`; all reads filter `deleted IS NULL`; parameterized `query!`/`query_as!`. Transaction mechanics cloned from carryover (`tx.tx.lock().await.as_mut()`).
- Regenerated `.sqlx/` offline cache (4 new entries) so the `SQLX_OFFLINE=true` gated build resolves the new queries.

**Task 2 — Basic service (commit nqmprmzo 72749b4a, `feat`)**
- `service/src/vacation_entitlement_offset.rs`: `VacationEntitlementOffset` domain + `From<&Entity>`/`TryFrom<&domain>` conversions + `VacationEntitlementOffsetService` trait (`get`/`set`/`delete`) with `#[automock(type Context=(); type Transaction=dao::MockTransaction;)]`.
- `service_impl/src/vacation_entitlement_offset.rs`: `gen_service_impl!` Basic impl with deps **Dao/Permission/Clock/Uuid/Transaction only** (D-28-06, no domain-service dep). Every method runs `use_transaction` → `check_permission(HR_PRIVILEGE)` **before any DAO access** → work → `commit`. `set` = find-active → update-or-create; `delete` = soft-delete active row.

**Task 3 — Tests (commit spzyqrqk acb4de02, `test`)**
- `service_impl/src/test/vacation_entitlement_offset.rs`: 7 `#[tokio::test]` cases using mockall fixtures (template: sales_person_unavailable test) — set-creates, get-returns-Some, set-updates-existing-row (no duplicate, create `.times(0)`), delete-soft-deletes, delete-not-found, **HR-gate set → Forbidden with zero DAO writes (`create`/`update` `.times(0)`)**, and HR-gate get → Forbidden.

## must_haves coverage

- D-28-01: table + DAO twin with id PK, signed offset, partial unique index, `deleted IS NULL` reads, sqlx compile-time validated — done.
- D-28-06: service is Basic — only Dao/Permission/Clock/Uuid/Transaction deps; the only `service::*Service` import is `PermissionService` — done.
- D-28-06b: HR_PRIVILEGE enforced on get/set/delete; non-HR set returns `Forbidden` and DAO write mock asserts zero calls — done.

## Deviations from Plan

**[Rule 2 — extra coverage] Added `find_by_id` SELECT and two extra tests beyond the named behaviors**
- The DAO trait includes `find_by_id` (named in `<artifacts_this_phase_produces>`) for future REST `DELETE /{id}`; it is implemented and compile-validated though not yet consumed.
- Added `test_delete_hr_not_found` and `test_get_non_hr_forbidden` beyond the 4 named behaviors to lock the read-gate and not-found paths. No plan instruction contradicted.

No other deviations — plan executed as written. No reporting/billing file touched (snapshot-safe half).

## Gate results

- `cargo build --workspace` (SQLX_OFFLINE=true): success.
- `cargo test -p service_impl vacation_entitlement_offset`: 7 passed, 0 failed.
- `cargo test --workspace`: all crates pass, 0 failed (service_impl lib: 501 passed incl. the 7 new tests; integration suites green).
- `cargo clippy --workspace -- -D warnings`: clean, exit 0.
- `sqlx migrate info`: `20260629000000/installed create-vacation-entitlement-offset`. No `database reset` used. `.sqlx` regenerated (4 new entries).

## Known Stubs

None. The aggregate is fully wired and tested; `find_by_id` is implemented (not stubbed), awaiting the REST plan to consume it.

## Self-Check: PASSED

- Files created exist: migration, dao/*, dao_impl_sqlite/*, service/*, service_impl/*, test/* — all present.
- Commits exist in jj log: uvtqmurw b0c8f398, nqmprmzo 72749b4a, spzyqrqk acb4de02.
