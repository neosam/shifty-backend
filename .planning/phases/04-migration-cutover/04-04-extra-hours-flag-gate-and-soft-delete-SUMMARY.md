---
phase: 04-migration-cutover
plan: 04
subsystem: api
tags: [rust, service-layer, dao-layer, flag-gate, soft-delete, mockall, jj]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: 04-01 — `ExtraHoursService::soft_delete_bulk` trait method + `ServiceError::ExtraHoursCategoryDeprecated(ExtraHoursCategory)` variant + Wave-0 `unimplemented!()` stub
  - phase: 02-reporting-integration-snapshot-versioning
    provides: `FeatureFlagService::is_enabled` API + `absence_range_source_active` flag key + `Authentication::Full` bypass pattern
provides:
  - "Real `ExtraHoursServiceImpl::soft_delete_bulk` body — Plan 04-05 commit-phase calls it inside the cutover Tx with `Authentication::Full` to bulk-soft-delete the migrated legacy rows."
  - "Service-layer flag-gate on `ExtraHoursServiceImpl::create` — D-Phase4-09: post-cutover, POST `/extra-hours` with Vacation/SickLeave/UnpaidLeave returns `ExtraHoursCategoryDeprecated(category)` (mapped to HTTP 403 in `rest::error_handler` already by Plan 04-01 stub)."
  - "`ExtraHoursDao::soft_delete_bulk` DAO trait method + SQLx impl — single UPDATE with `WHERE deleted IS NULL AND id IN (...)` (idempotent re-runs)."
  - "New DI dep `FeatureFlagService` on `ExtraHoursServiceImpl` — wired in `shifty_bin/src/main.rs` with the constructor reorder that puts `feature_flag_service` BEFORE `extra_hours_service`."
affects: [04-05-cutover-gate-and-diff-report, 04-06-cutover-rest-and-openapi]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Service-layer flag-gate (D-Phase4-09 — Architectural Responsibility Map row 9): the deprecation-of-old-API check lives in the service `create()`, NOT in the REST handler. Better test isolation; single source of truth for any future REST surface that creates ExtraHours."
    - "Permission-FIRST ordering for bulk-write privileged ops: `check_permission(CUTOVER_ADMIN_PRIVILEGE, ctx)` runs BEFORE `use_transaction` and BEFORE the DAO call. Verified by `soft_delete_bulk_forbidden_for_unprivileged_user` via `expect_soft_delete_bulk().times(0) + expect_use_transaction().times(0)`."
    - "Flag-check skipped on non-deprecated categories: `if matches!(category, Vacation|SickLeave|UnpaidLeave)` wraps the `is_enabled` call so ExtraWork/Holiday/Volunteer/Custom POSTs avoid a pointless flag-read. Verified by `create_extra_work_succeeds_when_flag_on` via `expect_is_enabled().times(0)`."
    - "Tx-forwarding contract on bulk-write privileged ops: `soft_delete_bulk` calls `use_transaction` but does NOT commit — the cutover commit-phase (Plan 04-05) holds the Tx until the final atomic commit."

key-files:
  created:
    - "service_impl/src/test/extra_hours.rs"
  modified:
    - "dao/src/extra_hours.rs"
    - "dao_impl_sqlite/src/extra_hours.rs"
    - "service_impl/src/extra_hours.rs"
    - "service_impl/src/test/mod.rs"
    - "shifty_bin/src/main.rs"

key-decisions:
  - "Use a single-line guard `if matches!(category, Vacation|SickLeave|UnpaidLeave)` to short-circuit the flag-check for non-deprecated categories, instead of always reading `is_enabled` and matching after. Saves one DAO call per non-deprecated POST and lets the test pin `expect_is_enabled().times(0)` for the ExtraWork case."
  - "Place the flag-gate AFTER the existing permission check (`hr_permission.or(sales_person_permission)?`) and BEFORE the DAO insert. Rationale: unauthorized callers must still see `Forbidden` (not `ExtraHoursCategoryDeprecated`); deprecated requests make zero state change because the early `return Err(...)` lets the Tx roll back via Drop (Pattern-1 Tx-forwarding contract)."
  - "Use `clock_service.date_time_now()` + `uuid_service.new_uuid(\"extra_hours_service::soft_delete_bulk version\")` for deleted_at + new_version inside `soft_delete_bulk`. Consistent with the existing `create()` path; testable with `MockClockService` + `MockUuidService`."
  - "Reorder DI in `shifty_bin/src/main.rs`: `feature_flag_dao` + `feature_flag_service` move BEFORE `extra_hours_service`. The plan hand-off note flagged this as a Plan 04-06 task, but the workspace MUST build after Plan 04-04 (per the must-have `cargo build --workspace GREEN`), so the reorder lives here."

patterns-established:
  - "jj per-task commits: 4 tasks → 4 separate `jj describe -m ... && jj new` commits. Each task is self-contained on the build axis (Task 1 + Task 3 alone would leave `service_impl` red — but the orchestrator runs ALL 4 tasks as a unit, so the working state at the end of the plan is workspace-green)."
  - "Conventional commit prefixes: `feat()` for trait/impl additions (Tasks 1-3), `test()` for new test module (Task 4)."

requirements-completed: [MIG-05]

# Metrics
duration: ~12min
completed: 2026-05-03
---

# Phase 04 Plan 04: ExtraHours Flag-Gate + Soft-Delete Summary

**Wave-1 surgical patch to existing `ExtraHoursService`: (a) flag-gated `create()` for the deprecated Vacation/SickLeave/UnpaidLeave categories (D-Phase4-09), (b) real `soft_delete_bulk()` impl with `CUTOVER_ADMIN_PRIVILEGE`-first permission gate (C-Phase4-04), (c) new `FeatureFlagService` DI dep + `ExtraHoursDao::soft_delete_bulk` trait + SQLx impl + 5 tests.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-03T13:39Z
- **Completed:** 2026-05-03T13:51Z
- **Tasks:** 4
- **Files modified:** 6 (1 created + 5 modified)

## Accomplishments

- `dao::extra_hours::ExtraHoursDao::soft_delete_bulk(ids, deleted_at, update_process, new_version, tx)` trait method exists; SQLx impl in `dao_impl_sqlite::extra_hours::ExtraHoursDaoImpl` issues a single `UPDATE ... WHERE deleted IS NULL AND id IN (...)` via `QueryBuilder` for idempotent re-runs.
- `service_impl::extra_hours::ExtraHoursServiceImpl::create` is flag-gated: post-cutover (`absence_range_source_active = true`) any `Vacation` / `SickLeave` / `UnpaidLeave` POST returns `ServiceError::ExtraHoursCategoryDeprecated(category)`. The check runs AFTER permission verification and BEFORE the DAO insert; deprecated requests cause zero state change (Tx rolls back via Drop on early Err).
- `service_impl::extra_hours::ExtraHoursServiceImpl::soft_delete_bulk` is implemented for real (replacing the Plan-04-01 `unimplemented!()` stub). It permission-gates `CUTOVER_ADMIN_PRIVILEGE` STRICTLY BEFORE any Tx/DAO work and forwards the caller-provided `update_process` tag verbatim. Tx is held by the caller (cutover commit phase, Plan 04-05) — no commit here.
- `FeatureFlagService` is a new DI dep on `ExtraHoursServiceImpl`. The `gen_service_impl!` block, `ExtraHoursServiceDeps` impl in `shifty_bin/src/main.rs`, and the actual `Arc::new(ExtraHoursServiceImpl{...})` constructor were updated. Construction order in `main.rs` was reordered so `feature_flag_service` is built BEFORE `extra_hours_service`.
- 5 new tests in `service_impl/src/test/extra_hours.rs` (new module): 3 cover the D-Phase4-09 flag-gate matrix (flag-off+Vacation, flag-on+Vacation, flag-on+ExtraWork), 1 covers the `soft_delete_bulk` happy path with verbatim id/update_process forwarding, 1 is the `_forbidden` counterpart that proves the permission gate sits BEFORE both `use_transaction` and the DAO call.
- `cargo build --workspace` is GREEN; `cargo test -p service_impl test::extra_hours` reports 5/5 passed; `cargo test --workspace` is GREEN with no regressions (351 passed in `service_impl` lib tests, 32 passed in `shifty_bin` integration tests).

## Task Commits

Each task committed atomically via jj (no `git commit`/`git add`):

1. **Task 1: ExtraHoursDao::soft_delete_bulk trait method** — jj change `nxlwspro d5e21295` (feat)
2. **Task 2: SQLx impl of ExtraHoursDao::soft_delete_bulk** — jj change `unntvkpl 70473ee6` (feat)
3. **Task 3: ExtraHoursServiceImpl flag-gated create + soft_delete_bulk impl + new FeatureFlagService DI dep** — jj change `zylvnupw 47ec75a6` (feat)
4. **Task 4: 5 new tests in service_impl/src/test/extra_hours.rs + mod.rs registration** — jj change `nuokuonp a23003aa` (test)

_Note: STATE.md / ROADMAP.md updates are intentionally not part of these commits — orchestrator owns that surface (per execution prompt + CLAUDE.local.md jj-only directive)._

## Files Created/Modified

### Created

- `service_impl/src/test/extra_hours.rs` — new test module: `ExtraHoursDependencies` builder + 5 `#[tokio::test]` test functions covering the flag-gate matrix and the `soft_delete_bulk` happy + forbidden paths. Uses `MockExtraHoursDao`, `MockFeatureFlagService`, `MockPermissionService`, `MockSalesPersonService`, `MockClockService`, `MockUuidService`, `MockCustomExtraHoursService`, `MockTransactionDao` from existing test infrastructure.

### Modified

- `dao/src/extra_hours.rs` — appended `async fn soft_delete_bulk(ids, deleted_at, update_process, new_version, tx)` to the `ExtraHoursDao` trait. Mockall auto-generates `MockExtraHoursDao::expect_soft_delete_bulk()`.
- `dao_impl_sqlite/src/extra_hours.rs` — added `use sqlx::{QueryBuilder, Sqlite}` and a real `soft_delete_bulk` impl using a single dynamic `UPDATE` with an `IN (...)` clause; `WHERE deleted IS NULL` makes re-runs idempotent.
- `service_impl/src/extra_hours.rs` — (a) added `service::cutover::CUTOVER_ADMIN_PRIVILEGE` + `service::feature_flag::FeatureFlagService` + `service::extra_hours::ExtraHoursCategory` imports, (b) added `FeatureFlagService` field to `gen_service_impl!`, (c) inserted the flag-gate block in `create()` after the permission check and before the DAO insert, (d) replaced the `unimplemented!()` body of `soft_delete_bulk` with the real impl (permission-FIRST, no commit).
- `service_impl/src/test/mod.rs` — added `pub mod extra_hours;` (alphabetical, between `employee_work_details` and the rest).
- `shifty_bin/src/main.rs` — (a) added `type FeatureFlagService = FeatureFlagService` to `ExtraHoursServiceDependencies` impl block, (b) moved `feature_flag_dao` + `feature_flag_service` constructor BEFORE `extra_hours_service` constructor, (c) added `feature_flag_service: feature_flag_service.clone()` to the `ExtraHoursServiceImpl{...}` initializer.

## Test-helper sweep results

`grep -rn 'ExtraHoursServiceImpl {' service_impl/src/test/ shifty_bin/src/`:

- `service_impl/src/test/extra_hours.rs` — NEW test-helper builder; constructed at the new `feature_flag_service` field intentionally.
- `shifty_bin/src/main.rs:777` — constructor updated with `feature_flag_service: feature_flag_service.clone()` + DI reorder.

No existing `ExtraHoursServiceImpl{...}` call-sites pre-Plan-04-04 (Plan 04-04 is the first plan to add a service-level test for `ExtraHoursService`). The plan's `<read_first>` test-helper sweep correctly anticipated this surface — the only test-time constructor lives in the new file we created in Task 4.

## Verification

| Step | Command | Result |
| ---- | ------- | ------ |
| DAO trait method | `grep -q 'async fn soft_delete_bulk' dao/src/extra_hours.rs` | exit 0 |
| DAO impl | `grep -q 'fn soft_delete_bulk' dao_impl_sqlite/src/extra_hours.rs` | exit 0 |
| DAO impl idempotent guard | `grep -q 'WHERE deleted IS NULL' dao_impl_sqlite/src/extra_hours.rs` | exit 0 |
| service_impl flag-gate | `grep -A2 'is_enabled' service_impl/src/extra_hours.rs \| grep absence_range_source_active` | match |
| service_impl deprecated variant | `grep -q 'ExtraHoursCategoryDeprecated' service_impl/src/extra_hours.rs` | exit 0 |
| service_impl bulk method | `grep -q 'fn soft_delete_bulk' service_impl/src/extra_hours.rs` | exit 0 |
| service_impl CUTOVER_ADMIN | `grep -q 'CUTOVER_ADMIN_PRIVILEGE' service_impl/src/extra_hours.rs` | exit 0 |
| service_impl new dep | `grep -q 'feature_flag_service' service_impl/src/extra_hours.rs` | exit 0 |
| `cargo build -p dao` | | exit 0 |
| `cargo build -p dao_impl_sqlite` | | exit 0 |
| `cargo build -p service_impl` | | exit 0 |
| `cargo build --workspace` | | exit 0 |
| `cargo test -p service_impl test::extra_hours` | | 5 passed; 0 failed; 0 ignored |
| `cargo test --workspace` | | all green; 0 failures |
| `cargo run` smoke | dev binary boots, scheduler runs carryover update | OK |

## Decisions Made

None beyond what the plan locked. The locked-decisions are documented in the YAML frontmatter `key-decisions` section. The two notable ones:

- **Single-line `matches!` guard around `is_enabled`** — minor optimization that lets Test 3 (`create_extra_work_succeeds_when_flag_on`) pin `expect_is_enabled().times(0)`. The plan's RESEARCH.md Operation 5 sketch reads the flag unconditionally; the guard makes the impl strictly cheaper without changing behavior.
- **DI reorder in `main.rs` lives here** — the plan's hand-off note flagged this as a Plan 04-06 task, but Plan 04-04's must-have `cargo build --workspace GREEN` requires the reorder to happen now. Plan 04-06 will inherit a workspace where `feature_flag_service` is already in the right position.

## Deviations from Plan

None — all 4 tasks executed as written. The plan's RESEARCH.md Operation 5 sketch suggested a `commit-then-error` pattern; the impl uses rollback-on-Drop (cleaner, matches the Pattern-1 Tx-forwarding contract). The plan explicitly notes this preference at task 3, so this is not a deviation.

**Total deviations:** 0.
**Impact on plan:** None.

## Issues Encountered

None — execution was deterministic.

## Threat Surface Scan

Threat register (T-04-04-01..04) was honored:

- **T-04-04-01 (Elevation of Privilege — `soft_delete_bulk` could erase arbitrary rows):** Mitigated. `check_permission(CUTOVER_ADMIN_PRIVILEGE, ctx)` is the FIRST line of the impl, BEFORE `use_transaction` and BEFORE the DAO call. Verified by `soft_delete_bulk_forbidden_for_unprivileged_user`: the test sets `expect_soft_delete_bulk().times(0) + expect_use_transaction().times(0)` — the test fails if the impl calls either before the permission check denies.
- **T-04-04-02 (Spoofing — flag-gate could be bypassed via REST handler):** Mitigated by service-layer placement. The check lives in `ExtraHoursServiceImpl::create`, so any current or future REST surface that creates ExtraHours flows through it. REST handler in `rest/src/extra_hours.rs` was intentionally left untouched (Plan 04-06 owns REST changes).
- **T-04-04-03 (Time-of-check-Time-of-use — race between flag-check and Tx):** Mitigated. The flag-check happens INSIDE the same Tx as the DAO insert (`Some(tx.clone())` passed to `is_enabled`). SQLite SERIALIZABLE isolation gives a consistent view; if the cutover commit Tx is in flight, the `create` Tx is queued by SQLite's WriteLock, so by the time the flag-read returns the Tx is fully consistent.
- **T-04-04-04 (Information Disclosure — error message reveals deprecated category):** Accepted per plan — the category name in the error is intentionally public-facing (clients need to know which category is deprecated to migrate to `/absence-period`). No PII.

No new threat surface introduced beyond the threat register.

## Hand-off Note for Plan 04-05 (cutover-gate-and-diff-report)

`ExtraHoursService::soft_delete_bulk(ids, update_process, ctx, tx)` is callable. The cutover commit phase calls it with `Authentication::Full` + the cutover Tx (per CONTEXT D-Phase4-14 — the cutover Tx is held by `CutoverServiceImpl::run`). The `Arc<[Uuid]>` consumed by `soft_delete_bulk` is the verbatim output of `migrate_legacy_extra_hours_to_clusters` from Plan 04-02 (locked tuple-shape contract `(MigrationStats, Arc<[Uuid]>)`).

Sample call site (Plan 04-05 Task 1 commit_phase):

```rust
self.extra_hours_service
    .soft_delete_bulk(
        migrated_ids.clone(),                  // Arc<[Uuid]> from migrate_legacy_extra_hours_to_clusters
        "phase-4-cutover-migration",            // verbatim per D-Phase4-10
        Authentication::Full,                   // service-internal trust (outer permission already checked)
        Some(tx.clone()),                       // cutover Tx held by CutoverServiceImpl::run
    )
    .await?;
```

## Hand-off Note for Plan 04-06 (cutover-rest-and-openapi)

The plan's original hand-off note says: "DI re-order in shifty_bin/src/main.rs MANDATORY: FeatureFlagService construction MUST happen BEFORE ExtraHoursService construction (currently FeatureFlagService is at Z. 795, ExtraHoursService at Z. 770 — must swap)."

**Status: ALREADY DONE.** Plan 04-04 inherited the build-must-be-green constraint and performed the reorder in this commit. Plan 04-06 inherits a workspace with `feature_flag_service` constructed at line ~771 and `extra_hours_service` constructed immediately after at line ~778. No further reordering needed.

Plan 04-06's actual responsibilities remain: (a) reshape the `ServiceError::ExtraHoursCategoryDeprecated` match arm body in `rest::error_handler` per D-Phase4-09 (currently returns `403 + err.to_string()`; Plan 04-06 returns the OpenAPI-snapshotted JSON body shape `{ "error": "extra_hours_category_deprecated", "category": "vacation", "message": "Use POST /absence-period for this category" }`), (b) the cutover REST endpoints + utoipa annotations + OpenAPI snapshot test.

## Self-Check: PASSED

Verification of summary claims:

- **Files created exist:**
  - `service_impl/src/test/extra_hours.rs` — FOUND (new test module with 5 tests)
- **Files modified contain expected content:**
  - `dao/src/extra_hours.rs` — `async fn soft_delete_bulk` present
  - `dao_impl_sqlite/src/extra_hours.rs` — `fn soft_delete_bulk` + `WHERE deleted IS NULL` + `QueryBuilder` present
  - `service_impl/src/extra_hours.rs` — `feature_flag_service` field, `ExtraHoursCategoryDeprecated`, `CUTOVER_ADMIN_PRIVILEGE`, `is_enabled("absence_range_source_active"`, real `soft_delete_bulk` body all present
  - `service_impl/src/test/mod.rs` — `pub mod extra_hours;` present
  - `shifty_bin/src/main.rs` — `type FeatureFlagService = FeatureFlagService` in `ExtraHoursServiceDependencies` block; `feature_flag_service: feature_flag_service.clone()` in the constructor; constructor reorder verified visually
- **jj changes exist:**
  - `nxlwspro d5e21295` — Task 1 (feat: DAO trait method)
  - `unntvkpl 70473ee6` — Task 2 (feat: DAO SQLx impl)
  - `zylvnupw 47ec75a6` — Task 3 (feat: service impl + DI)
  - `nuokuonp a23003aa` — Task 4 (test: 5 tests)
- **Acceptance criteria met:**
  - 5 new tests added; all 5 pass; 0 ignored
  - `cargo build --workspace` exits 0
  - `cargo test --workspace` exits 0 with no regressions
  - `cargo run` smoke test boots cleanly
  - All 8 grep-acceptance-criteria across the 4 tasks satisfied

---

*Phase: 04-migration-cutover*
*Plan: 04 (extra-hours-flag-gate-and-soft-delete)*
*Completed: 2026-05-03*
