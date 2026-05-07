---
phase: 04-migration-cutover
plan: 01
subsystem: api
tags: [rust, traits, mockall, async-trait, automock, service-layer, dao-layer, jj]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: 04-00 — three Phase-4 audit tables + cutover_admin privilege seed; uuid v4 feature on dao crates
  - phase: 01-absence-domain-foundation
    provides: AbsenceCategory enum (Vacation/SickLeave/UnpaidLeave) — referenced by DriftRow + CutoverProfileBucket
  - phase: 02-reporting-integration-snapshot-versioning
    provides: FeatureFlagService trait shape — exemplar for the new CutoverService surface
provides:
  - service::cutover::{CutoverService, CutoverRunResult, GateResult, DriftRow, QuarantineReason, CutoverProfile, CutoverProfileBucket, CUTOVER_ADMIN_PRIVILEGE}
  - service::carryover_rebuild::CarryoverRebuildService — Business-Logic-Tier surface that breaks the Reporting -> Carryover -> Reporting cycle (Pitfall 1)
  - dao::cutover::{CutoverDao, LegacyExtraHoursRow, QuarantineRow, MigrationSourceRow} — 8 async DAO methods for Wave 1+2
  - service::ServiceError::ExtraHoursCategoryDeprecated(ExtraHoursCategory) — D-Phase4-09 + RESEARCH.md Operation 4
  - service::extra_hours::ExtraHoursService::soft_delete_bulk — C-Phase4-04 (Plan 04-04 implements body)
  - 12 #[ignore]+unimplemented!() test stubs (11 cutover + 1 carryover_rebuild) per Phase-3 Wave-0 pattern
affects: [04-02-cutover-service-heuristic, 04-03-carryover-rebuild-service, 04-04-extra-hours-flag-gate-and-soft-delete, 04-05-cutover-gate-and-diff-report, 04-06-cutover-rest-and-openapi, 04-07-integration-tests-and-profile]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cycle-breaking Business-Logic Service: CarryoverRebuildService is a NEW BL service (consumes ReportingService Read + CarryoverService Write); CarryoverService stays basic-tier."
    - "Permission-Branch-Innerhalb-run: CutoverService::run takes dry_run flag and branches HR-vs-cutover_admin per Pattern 3 in 04-RESEARCH.md."
    - "Phase-3 Wave-0 stub pattern reused: #[ignore=\"wave-N-...\"] + unimplemented!(\"wave-N\") so cargo test --list shows the surface immediately while keeping cargo test --workspace green."
    - "Workspace-build stubs (Rule 3): trait extensions add a stub impl in service_impl + a stub match arm in rest::error_handler so the workspace compiles without leaking trait surface to Wave 2 plans."

key-files:
  created:
    - "service/src/cutover.rs"
    - "service/src/carryover_rebuild.rs"
    - "dao/src/cutover.rs"
    - "service_impl/src/test/cutover.rs"
    - "service_impl/src/test/carryover_rebuild.rs"
  modified:
    - "service/src/lib.rs"
    - "service/src/extra_hours.rs"
    - "dao/src/lib.rs"
    - "service_impl/src/extra_hours.rs"
    - "rest/src/lib.rs"
    - "service_impl/src/test/mod.rs"

key-decisions:
  - "CarryoverRebuildService is a separate Business-Logic-Tier service (NOT CarryoverService::rebuild_for_year, NOT inlined in CutoverServiceImpl). Documented at the top of service/src/carryover_rebuild.rs per locked decision C-Phase4-02."
  - "ServiceError::ExtraHoursCategoryDeprecated wraps ExtraHoursCategory directly (typed) so the rest error_handler can match-extract the category for the 403 body (vs. Arc<str>+format-parse)."
  - "Trait extensions (soft_delete_bulk + ServiceError variant) require corresponding stub impls in service_impl + match arm in rest so cargo build --workspace stays green; both stubs use unimplemented!()/HTTP 403 placeholder bodies that Wave 2 plans rewrite."

patterns-established:
  - "jj per-task split: write all task changes into one working copy, then jj split <files> -m <task-msg> incrementally to land each task as its own change without rebuilding the workspace per task. Avoids intermediate-build-fail noise that the plan explicitly accepts."
  - "Conventional commit prefixes: feat() for trait/DTO surface (Tasks 1-3), test() for #[ignore] scaffolding + Rule-3 build-fix stubs (Task 4)."

requirements-completed: [MIG-01, MIG-02, MIG-03, MIG-04, MIG-05]

# Metrics
duration: ~12min
completed: 2026-05-03
---

# Phase 04 Plan 01: Service Traits and Stubs Summary

**Wave-0 contracts-only plan: CutoverService + CarryoverRebuildService traits, 8-method CutoverDao surface, ServiceError variant, ExtraHoursService::soft_delete_bulk extension, and 12 #[ignore]+unimplemented!() test stubs ready for Wave 1+2 implementations.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-03T12:50Z
- **Completed:** 2026-05-03T13:02Z
- **Tasks:** 4
- **Files modified:** 11 (5 created + 6 modified)

## Accomplishments

- `service` crate exposes the full Phase-4 trait + DTO surface: `CutoverService` (run + profile), `CarryoverRebuildService` (rebuild_for_year), `CUTOVER_ADMIN_PRIVILEGE` const, plus 6 DTOs (`CutoverRunResult`, `GateResult`, `DriftRow`, `QuarantineReason` enum, `CutoverProfile`, `CutoverProfileBucket`) — Wave 1 plans (04-02/03/04) can implement against frozen contracts.
- `service::ServiceError::ExtraHoursCategoryDeprecated(ExtraHoursCategory)` variant added; `service::extra_hours::ExtraHoursService::soft_delete_bulk` trait method added — both required by Plan 04-04 + Plan 04-06.
- `dao` crate exposes `CutoverDao` with all 8 async methods (find_legacy_extra_hours_not_yet_migrated, find_all_legacy_extra_hours, upsert_migration_source, upsert_quarantine, find_legacy_scope_set, sum_legacy_extra_hours, count_quarantine_for_drift_row, backup_carryover_for_scope) plus 3 row DTOs.
- 11 cutover + 1 carryover_rebuild `#[ignore]+unimplemented!()` test stubs are visible via `cargo test --list` so Wave-1+2 plans can flip `#[ignore]` off without renaming.
- `cargo build --workspace` is GREEN; `cargo test -p service_impl test::cutover` reports 11 ignored; `cargo test -p service_impl test::carryover_rebuild` reports 1 ignored.

## Task Commits

Each task committed atomically via jj (no git commit/add):

1. **Task 1: ServiceError + soft_delete_bulk + lib.rs mod imports** — jj change `poywyyxy 6b9e44c5` (feat)
2. **Task 2: service/src/cutover.rs + service/src/carryover_rebuild.rs** — jj change `lmnrzymk 2c64eaa7` (feat)
3. **Task 3: dao/src/cutover.rs + dao/src/lib.rs mod registration** — jj change `xwrxznqr 6b56e8bd` (feat)
4. **Task 4: Test scaffolding stubs + workspace-build stubs (Rule 3)** — jj change `uyqvmtrq 6865f6a5` (test)

**Plan metadata:** jj change `mpkzvrnt ee30209a` (docs: complete service-traits-and-stubs plan)

_Note: STATE.md / ROADMAP.md updates are intentionally not part of these commits — orchestrator owns that surface (per execution prompt)._

## Files Created/Modified

### Created

- `service/src/cutover.rs` — CutoverService trait (run + profile) + 6 DTOs + QuarantineReason enum (5 variants) + `CUTOVER_ADMIN_PRIVILEGE` const. Business-Logic-Tier per CLAUDE.md.
- `service/src/carryover_rebuild.rs` — CarryoverRebuildService trait with `rebuild_for_year`. Locked decision documented at top: cycle-breaking BL service vs. (rejected) CarryoverService extension or CutoverServiceImpl inlining.
- `dao/src/cutover.rs` — CutoverDao trait with 8 async methods + 3 row DTOs (LegacyExtraHoursRow, QuarantineRow, MigrationSourceRow) + `From<&ExtraHoursEntity>` impl on LegacyExtraHoursRow.
- `service_impl/src/test/cutover.rs` — 11 `#[ignore]+unimplemented!()` test stubs (1 cluster heuristic, 5 quarantine reasons, 1 idempotence, 2 gate tolerance, 2 forbidden tests).
- `service_impl/src/test/carryover_rebuild.rs` — 1 `#[ignore]+unimplemented!()` forbidden-test stub.

### Modified

- `service/src/lib.rs` — added `pub mod carryover_rebuild;` + `pub mod cutover;` (alphabetical) + `ServiceError::ExtraHoursCategoryDeprecated(ExtraHoursCategory)` variant.
- `service/src/extra_hours.rs` — added `soft_delete_bulk(ids, update_process, ctx, tx)` trait method on `ExtraHoursService` per C-Phase4-04.
- `dao/src/lib.rs` — added `pub mod cutover;` (alphabetical, after `custom_extra_hours`).
- `service_impl/src/extra_hours.rs` — Rule 3 stub: added `soft_delete_bulk` impl on `ExtraHoursServiceImpl` (calls `unimplemented!("Plan 04-04 implements ...")`) so `service_impl` compiles.
- `rest/src/lib.rs` — Rule 3 stub: added match arm for `ServiceError::ExtraHoursCategoryDeprecated` returning HTTP 403 with `err.to_string()` body. Plan 04-06 (Wave 2) shapes the body per D-Phase4-09.
- `service_impl/src/test/mod.rs` — added `pub mod carryover_rebuild;` + `pub mod cutover;` (alphabetical).

## New trait method signatures (verbatim)

```rust
// service/src/cutover.rs
pub trait CutoverService {
    type Context: ...;
    type Transaction: dao::Transaction;

    async fn run(
        &self,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverRunResult, ServiceError>;

    async fn profile(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverProfile, ServiceError>;
}

// service/src/carryover_rebuild.rs
pub trait CarryoverRebuildService {
    type Context: ...;
    type Transaction: dao::Transaction;

    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}

// service/src/extra_hours.rs (appended)
async fn soft_delete_bulk(
    &self,
    ids: Arc<[Uuid]>,
    update_process: &str,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError>;

// service/src/lib.rs (ServiceError variant)
#[error("ExtraHours category {0:?} is deprecated; use POST /absence-period for this category")]
ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory),
```

## Verification

| Step | Command | Result |
| ---- | ------- | ------ |
| Workspace build | `cargo build --workspace` | exit 0 (`Finished dev profile in 46.65s` initially; subsequent runs are incremental) |
| Cutover tests visible | `cargo test -p service_impl test::cutover -- --list` | 11 tests listed (one per name in the Per-Task Verification Map) |
| Carryover-rebuild tests visible | `cargo test -p service_impl test::carryover_rebuild -- --list` | 1 test listed |
| Cutover tests run gated | `cargo test -p service_impl test::cutover` | 0 passed; 0 failed; 11 ignored |
| Carryover-rebuild tests run gated | `cargo test -p service_impl test::carryover_rebuild` | 0 passed; 0 failed; 1 ignored |
| Workspace test compile | `cargo test --workspace --no-run` | exit 0 |
| 8 DAO methods present | `grep -v '^[[:space:]]*//' dao/src/cutover.rs \| grep -c 'async fn'` | 8 |
| ServiceError variant present | `grep -q 'ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory)' service/src/lib.rs` | exit 0 |
| soft_delete_bulk present | `grep -q 'async fn soft_delete_bulk' service/src/extra_hours.rs` | exit 0 |
| C-Phase4-02 documented | `grep -q "C-Phase4-02" service/src/carryover_rebuild.rs` | exit 0 |
| CUTOVER_ADMIN_PRIVILEGE present | `grep -q 'CUTOVER_ADMIN_PRIVILEGE: &str = "cutover_admin"' service/src/cutover.rs` | exit 0 |

## Decisions Made

None beyond what the plan locked. The locked-decision in `service/src/carryover_rebuild.rs` (CarryoverRebuildService = new BL service, NOT CarryoverService::rebuild_for_year, NOT inlined in CutoverServiceImpl) follows C-Phase4-02 verbatim. All trait shapes follow the Phase-2 `FeatureFlagService` exemplar from `<interfaces>` in the plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking-Fix] Stub `soft_delete_bulk` impl in service_impl::extra_hours::ExtraHoursServiceImpl**
- **Found during:** Task 4 verification (`cargo build --workspace` failed with E0046 "not all trait items implemented, missing: `soft_delete_bulk`")
- **Issue:** Adding `soft_delete_bulk` to the `ExtraHoursService` trait in Task 1 forces `ExtraHoursServiceImpl` to implement it; without an impl, `service_impl` fails to compile and the workspace cannot be tested in Wave 0. Plan must-have `cargo build --workspace is GREEN` requires a stub.
- **Fix:** Added an `unimplemented!("Plan 04-04 implements ExtraHoursService::soft_delete_bulk")` body. The doc-comment explicitly states this is a Wave-0 placeholder; Plan 04-04 (extra-hours-flag-gate-and-soft-delete) replaces it with the real impl.
- **Files modified:** `service_impl/src/extra_hours.rs`
- **Verification:** `cargo build --workspace` exits 0; the only call sites are the `#[ignore]`d cutover tests, so the `unimplemented!()` is never executed in Wave 0.
- **Committed in:** jj change `uyqvmtrq 6865f6a5` (Task 4 commit)

**2. [Rule 3 - Blocking-Fix] Stub match arm for `ServiceError::ExtraHoursCategoryDeprecated` in rest::error_handler**
- **Found during:** Task 4 verification (`cargo build --workspace` failed with E0004 "non-exhaustive patterns" in `rest/src/lib.rs::error_handler`)
- **Issue:** Adding the new ServiceError variant in Task 1 breaks the exhaustive match in `error_handler`. Plan 04-06 (Wave 2) is responsible for the body shape per D-Phase4-09, but the workspace must compile in Wave 0 for `cargo build --workspace` to be green.
- **Fix:** Added a match arm returning HTTP 403 with `err.to_string()` body — the simplest contract-honoring response (status code from D-Phase4-09; body is a placeholder Wave 2 reshapes per the OpenAPI snapshot).
- **Files modified:** `rest/src/lib.rs`
- **Verification:** `cargo build --workspace` exits 0; `cargo test --workspace --no-run` exits 0.
- **Committed in:** jj change `uyqvmtrq 6865f6a5` (Task 4 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 3 — Blocking-Fix)
**Impact on plan:** Both stubs are required for the must-have `cargo build --workspace is GREEN`. Each fix is explicitly framed as a Wave-0 placeholder that the responsible Wave 1+2 plan replaces. No scope creep; surface contracts (trait signatures + ServiceError variant) are unchanged from the plan.

## Issues Encountered

None — execution was deterministic. The intermediate jj commits (Tasks 1+2 only) are not workspace-buildable, which the plan explicitly accepts ("ordering inside Task 1 means we cannot finalize without 2+3 — execute Task 1, 2, 3 in sequence and run `cargo build -p service` only after Task 3"). Final state after Task 3 = `cargo build -p service` green; final state after Task 4 = `cargo build --workspace` green + `cargo test --workspace --no-run` green.

## Threat Surface Scan

Threat register (T-04-01-01..04) was honored:

- **T-04-01-01 (Spoofing):** Mitigated by trait-doc — `CutoverService::run` doc explicitly tells implementors to permission-check inside (HR for dry_run, cutover_admin for commit). Plan 04-02 task 1 verifies the impl.
- **T-04-01-02 (Tampering):** Mitigated by trait-doc on `soft_delete_bulk` — explicitly says caller must hold `cutover_admin` and pass cutover-tx. Plan 04-04 enforces; Wave-0 stub is `unimplemented!()` so cannot be misused.
- **T-04-01-03 (Information Disclosure):** Mitigated — `DriftRow.sales_person_name` is service-internal; Wave 2 (Plan 04-06) wraps in REST DTO. Phase-4 `.gitignore` rule already protects `.planning/migration-backup/*.json`.
- **T-04-01-04 (Elevation of Privilege):** Accepted — `CUTOVER_ADMIN_PRIVILEGE` const is a `&str`; the privilege itself was seeded via the 04-00 migration. No new auth surface.

No new threat surface introduced beyond the threat register.

## Hand-off Note for Wave 1 (Plans 04-02 / 04-03 / 04-04)

- **Plan 04-02 (cutover-service-heuristic):** Implement `service::cutover::CutoverService` against `dao::cutover::CutoverDao` (8 methods). The 11 `#[ignore]`d tests in `service_impl/src/test/cutover.rs` cover the heuristic + idempotence + forbidden surface; flip `#[ignore]` off and replace `unimplemented!()` with the test body.
- **Plan 04-03 (carryover-rebuild-service):** Implement `service::carryover_rebuild::CarryoverRebuildService` consuming `ReportingService` (Read) + `CarryoverService` (Write). The 1 `#[ignore]`d test in `service_impl/src/test/carryover_rebuild.rs` covers the forbidden-permission gate.
- **Plan 04-04 (extra-hours-flag-gate-and-soft-delete):** Replace the `unimplemented!()` body of `ExtraHoursServiceImpl::soft_delete_bulk` with the real bulk-soft-delete logic (caller-tag `update_process`, idempotent re-runs).
- **Plan 04-06 (cutover-rest-and-openapi, Wave 2):** Reshape the `ServiceError::ExtraHoursCategoryDeprecated` match arm body in `rest::error_handler` per D-Phase4-09 (currently returns 403 + `err.to_string()`; Wave 2 returns the OpenAPI-snapshotted body shape).
- **Wave-2 gate-tolerance tests (`gate_tolerance_pass_below_threshold` / `gate_tolerance_fail_above_threshold`):** Live in `service_impl/src/test/cutover.rs` with `#[ignore=\"wave-2-implements-gate-tolerance\"]`. Plan 04-05 implements the body.

## Self-Check: PASSED

Verification of summary claims:

- **Files created exist:**
  - `service/src/cutover.rs` — FOUND
  - `service/src/carryover_rebuild.rs` — FOUND
  - `dao/src/cutover.rs` — FOUND
  - `service_impl/src/test/cutover.rs` — FOUND
  - `service_impl/src/test/carryover_rebuild.rs` — FOUND
- **Files modified contain expected content:**
  - `service/src/lib.rs` — `pub mod cutover;` + `pub mod carryover_rebuild;` + `ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory)` present
  - `service/src/extra_hours.rs` — `async fn soft_delete_bulk` present
  - `dao/src/lib.rs` — `pub mod cutover;` present
  - `service_impl/src/extra_hours.rs` — stub `soft_delete_bulk` impl with `unimplemented!()` present
  - `rest/src/lib.rs` — match arm for `ExtraHoursCategoryDeprecated` returning 403 present
  - `service_impl/src/test/mod.rs` — `pub mod cutover;` + `pub mod carryover_rebuild;` present
- **jj changes exist:**
  - `poywyyxy 6b9e44c5` — Task 1 (feat)
  - `lmnrzymk 2c64eaa7` — Task 2 (feat)
  - `xwrxznqr 6b56e8bd` — Task 3 (feat)
  - `uyqvmtrq 6865f6a5` — Task 4 (test)
- **Acceptance criteria met:**
  - 8 DAO methods present (`grep -c 'async fn'` excluding comments → 8)
  - 11 cutover tests + 1 carryover_rebuild test all `(ignored)`
  - `cargo build --workspace` exit 0
  - C-Phase4-02 documented in `service/src/carryover_rebuild.rs`

---

*Phase: 04-migration-cutover*
*Plan: 01 (service-traits-and-stubs)*
*Completed: 2026-05-03*
