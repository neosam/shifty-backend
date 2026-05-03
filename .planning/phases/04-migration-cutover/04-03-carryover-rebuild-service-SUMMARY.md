---
phase: 04-migration-cutover
plan: 03
subsystem: service
tags: [rust, business-logic-tier, dependency-injection, cycle-break, async-trait, mockall, jj]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: 04-01 — service::carryover_rebuild::CarryoverRebuildService trait + 1 #[ignore] forbidden-test stub
  - phase: 02-reporting-integration-snapshot-versioning
    provides: ReportingService::get_report_for_employee + EmployeeReport.balance_hours + EmployeeReport.vacation_carryover (read surface used by rebuild)
provides:
  - service_impl::carryover_rebuild::CarryoverRebuildServiceImpl — Business-Logic-Tier impl that breaks the Reporting -> Carryover -> Reporting cycle (Pitfall 1) by reading via ReportingService and writing via CarryoverService
  - service_impl::carryover_rebuild::CarryoverRebuildServiceDeps — auto-generated DI trait (4 deps: ReportingService, CarryoverService, PermissionService, TransactionDao)
  - rebuild_forbidden_for_unprivileged service-level forbidden-permission test (1 passed, 0 ignored)
affects: [04-05-cutover-gate-and-diff-report, 04-06-cutover-rest-and-openapi]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cycle-breaking BL service: CarryoverRebuildServiceImpl consumes ReportingService (Read) + CarryoverService (Write); CarryoverService stays basic-tier and continues to NOT consume ReportingService."
    - "Wave-0 stub flip pattern: drop #[ignore] + unimplemented!() and replace with real test body using a CarryoverRebuildDependencies harness (mirrors feature_flag.rs + cutover.rs multi-mock test pattern)."
    - "Authentication::Full bypass for service-internal calls: rebuild_for_year passes Authentication::Full to its own get_report_for_employee + get_carryover + set_carryover sub-calls (Backend-internal trust per service_impl/src/feature_flag.rs:36-41 exemplar)."

key-files:
  created:
    - "service_impl/src/carryover_rebuild.rs"
  modified:
    - "service_impl/src/lib.rs"
    - "service_impl/src/test/carryover_rebuild.rs"

key-decisions:
  - "Locked variant A (separate Business-Logic-Tier service) per CONTEXT.md C-Phase4-02. Variants B (inline in CutoverServiceImpl) + C (promote CarryoverService to BL) explicitly rejected for the reasons documented at the top of service/src/carryover_rebuild.rs."
  - "ReportingService method used: `get_report_for_employee(&sales_person_id, year, until_week=53, ctx, tx)` — verified against service/src/reporting.rs:209-216. The plan's placeholder used a value-not-reference signature; the real trait takes `&Uuid`."
  - "EmployeeReport field names used: `balance_hours: f32` -> Carryover.carryover_hours and `vacation_carryover: i32` -> Carryover.vacation. Both verified against service/src/reporting.rs:163-193 EmployeeReport struct."
  - "FULL_YEAR_UNTIL_WEEK = 53 const introduced inline (private) — covers ISO 53-week years; not exported because it is an implementation detail of the rebuild service."

patterns-established:
  - "Multi-mock test harness for BL services: struct CarryoverRebuildDependencies { reporting_service, carryover_service, permission_service } + impl CarryoverRebuildServiceDeps for it + `build_service(transaction_dao: MockTransactionDao)` constructor. Same shape as cutover.rs Wave 1 harness, scaled down for 4 deps."

requirements-completed: [MIG-04]

# Metrics
duration: ~10min
completed: 2026-05-03
---

# Phase 04 Plan 03: Carryover Rebuild Service Summary

**Wave-1 cycle-breaker plan: ships `CarryoverRebuildServiceImpl` as a separate Business-Logic-Tier service consuming ReportingService (Read) + CarryoverService (Write), plus the activated forbidden-permission test. Resolves Pitfall 1 (Reporting -> Carryover -> Reporting cycle) without promoting CarryoverService out of basic-tier.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-03T13:30Z (after Plan 04-02 commit)
- **Completed:** 2026-05-03T13:40Z
- **Tasks:** 2
- **Files modified:** 3 (1 created + 2 modified)

## Accomplishments

- `service_impl/src/carryover_rebuild.rs` ships the full cycle-breaking implementation:
  - `gen_service_impl!` DI block with 4 deps (`ReportingService`, `CarryoverService`, `PermissionService`, `TransactionDao`).
  - `rebuild_for_year(sp, year, ctx, tx)` permission-gates on `CUTOVER_ADMIN_PRIVILEGE`, opens the caller-passed Tx, reads `ReportingService::get_report_for_employee` (which internally uses `derive_hours_for_range` because the cutover Tx flips the feature flag), and UPSERTs the new `Carryover { carryover_hours = report.balance_hours, vacation = report.vacation_carryover }` row via `CarryoverService::set_carryover`.
  - Caller (CutoverServiceImpl::run, Plan 04-05) owns the Tx — no commit/rollback inside.
- `service_impl/src/test/carryover_rebuild.rs` flips the wave-0 `#[ignore]` off:
  - `rebuild_forbidden_for_unprivileged` test uses a `CarryoverRebuildDependencies` multi-mock harness, asserts that on `Err(Forbidden)` from PermissionService the Reporting/Carryover/TransactionDao surfaces are NEVER called (`.times(0)`), and verifies the error propagates verbatim.
- `cargo build --workspace` is GREEN; `cargo test -p service_impl test::carryover_rebuild` reports 1 passed / 0 failed / 0 ignored.
- Cycle break verified: `service_impl/src/carryover_rebuild.rs` has no code-level reference to `CutoverService` (only doc-comments mention `CutoverServiceImpl::run` as the invocation context).

## Task Commits

Each task committed atomically via jj (no git commit/add):

1. **Task 1: service_impl/src/carryover_rebuild.rs implementation + lib.rs mod registration** — jj change `twwlpwqt e3fda1e5` (feat)
2. **Task 2: rebuild_forbidden_for_unprivileged forbidden test (#[ignore] flipped off)** — jj change `woroklmt 0ce64a01` (test)

_Note: STATE.md / ROADMAP.md updates are intentionally not part of these commits — orchestrator owns that surface (per execution prompt + CLAUDE.local.md jj-only directive)._

## Files Created/Modified

### Created

- `service_impl/src/carryover_rebuild.rs` — `CarryoverRebuildServiceImpl` with `gen_service_impl!` DI block + `#[async_trait]` impl of `CarryoverRebuildService::rebuild_for_year`. Documents the locked decision (variant A) at the top.

### Modified

- `service_impl/src/lib.rs` — added `pub mod carryover_rebuild;` (alphabetical, after `carryover` and before `clock`).
- `service_impl/src/test/carryover_rebuild.rs` — replaced the `#[ignore = "wave-1-implements-forbidden"] unimplemented!()` stub with the real `rebuild_forbidden_for_unprivileged` test + `CarryoverRebuildDependencies` harness.

## DI Surface (verbatim)

```rust
gen_service_impl! {
    struct CarryoverRebuildServiceImpl: service::carryover_rebuild::CarryoverRebuildService = CarryoverRebuildServiceDeps {
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        CarryoverService: CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

## ReportingService method actually used

The plan's placeholder said `get_report_for_employee(sales_person_id, year, /*until_week*/ 53, ctx, tx)`. The real trait signature in `service/src/reporting.rs:209-216` takes `&Uuid` (not `Uuid`):

```rust
async fn get_report_for_employee(
    &self,
    sales_person_id: &Uuid,
    years: u32,
    until_week: u8,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<EmployeeReport, ServiceError>;
```

Implementation passes `&sales_person_id` and `FULL_YEAR_UNTIL_WEEK = 53`. The 53-week constant is necessary because some ISO years contain a week 53; using 52 would silently drop a week.

## EmployeeReport field names actually used

Verified against `service/src/reporting.rs:163-193`:
- `EmployeeReport.balance_hours: f32` -> `Carryover.carryover_hours: f32` (1:1)
- `EmployeeReport.vacation_carryover: i32` -> `Carryover.vacation: i32` (1:1, no `try_into` needed because both are `i32`)

The plan's placeholder anticipated a possible `try_into().unwrap_or(0)` cast — turned out unnecessary because both struct fields are already `i32`.

## CarryoverService::set_carryover semantics

Verified at `service_impl/src/carryover.rs:43-57`: `set_carryover` calls `CarryoverDao::upsert(...)` directly, so a single `set_carryover` call handles both insert-new and update-existing cases. The plan's "DELETE existing + UPSERT new — verify via reading service/src/carryover.rs" verification step concluded that no DELETE is needed; UPSERT is the documented contract. Implementation reuses `existing.created` (when an old row exists) so audit history is preserved across rebuilds; otherwise it stamps `now`.

## Test outcomes

| Test | Status |
|------|--------|
| `rebuild_forbidden_for_unprivileged` | passed |

Wave-0 `#[ignore]` count for carryover_rebuild dropped from 1 to 0.

## Decisions Made

None beyond what the plan locked. The locked variant A and the cycle-break invariant are documented at the top of both `service/src/carryover_rebuild.rs` (Plan 04-01) and `service_impl/src/carryover_rebuild.rs` (this plan).

## Deviations from Plan

### Adjustments to placeholder code in plan body

**1. [Sig fix] `get_report_for_employee` takes `&Uuid` not `Uuid`**
- **Found during:** Task 1 build of `service_impl`.
- **Issue:** Plan body at line 182 wrote `get_report_for_employee(sales_person_id, year, /*until_week*/ 53, ...)` with a value-typed `Uuid` arg.
- **Fix:** Pass `&sales_person_id`. The plan flagged this verbatim in its "Important verifications the executor MUST perform before committing" block, so it is an expected verification, not a deviation.
- **Files modified:** `service_impl/src/carryover_rebuild.rs`

**2. [Sig fix] `vacation_carryover` is already `i32`, no cast needed**
- **Found during:** Task 1 build of `service_impl`.
- **Issue:** Plan body at line 191 wrote `report.vacation_carryover.try_into().unwrap_or(0)` — defensive cast for an unknown source type.
- **Fix:** Direct assignment `let new_vacation: i32 = report.vacation_carryover;` because the source field is already `i32` per `service/src/reporting.rs:180`.
- **Files modified:** `service_impl/src/carryover_rebuild.rs`

Both adjustments are explicitly anticipated by the plan's verification block ("If the names differ, adapt and document in the SUMMARY"). Not deviations from the locked design — only refinements of the placeholder code in the plan body to match the real trait signatures.

### Deviation: acceptance-criteria grep for cycle break

**3. [Plan-acceptance-criterion clarification] `grep -q 'CutoverService\|CutoverServiceImpl'` matches doc-comments**
- **Found during:** Task 1 acceptance-criteria check.
- **Issue:** The plan's literal grep `grep -q 'CutoverService\|CutoverServiceImpl' service_impl/src/carryover_rebuild.rs` matches comment lines that describe the invocation context (e.g., "caller (CutoverServiceImpl::run) owns the cutover Tx"). Strict-pass would force removing those comments, harming readability without changing the cycle-break invariant.
- **Fix:** Used the equivalent code-level grep `grep -v '^[[:space:]]*//' service_impl/src/carryover_rebuild.rs | grep -q 'CutoverService'` — exits non-zero (PASS). The 4 occurrences of `CutoverService` in the file are all in comment lines documenting the cycle-break decision; the code surface has zero references.
- **Files modified:** none (purely an acceptance-criterion interpretation).
- **Verification:** `grep -n 'CutoverService' service_impl/src/carryover_rebuild.rs` returns 4 lines, all matching comment patterns (lines 11, 22, 79, 141 — all start with `//!` or `//`).

---

**Total deviations:** 0 design deviations; 2 placeholder-code refinements (anticipated by the plan's verification block); 1 acceptance-criterion clarification (cycle-break grep tightened to code-only).

**Impact on plan:** None. Locked decision (variant A) honored verbatim; trait signatures and EmployeeReport field names verified before commit; cycle-break invariant proven at the code level.

## Issues Encountered

None — execution was deterministic. Both `cargo build --workspace` and `cargo test -p service_impl test::carryover_rebuild` were green on the first attempt after each task.

## Verification

| Step | Command | Result |
| ---- | ------- | ------ |
| Workspace build | `cargo build --workspace` | exit 0 (`Finished dev profile in 13.65s` after Task 1; incremental thereafter) |
| Plan tests run | `cargo test -p service_impl test::carryover_rebuild` | 1 passed; 0 failed; 0 ignored; 347 filtered out |
| Cycle-break invariant | `grep -v '^[[:space:]]*//' service_impl/src/carryover_rebuild.rs \| grep -q 'CutoverService'` | exit 1 (PASS — code-only) |
| Files changed match plan | `jj diff -r 'twwlpwqt::woroklmt' --name-only` | 3 files (carryover_rebuild.rs, lib.rs, test/carryover_rebuild.rs) — exact match to plan `files_modified` |
| C-Phase4-02 documented | `grep -q "C-Phase4-02" service_impl/src/carryover_rebuild.rs` | exit 0 |
| #[ignore] removed | `grep -c '#\[ignore' service_impl/src/test/carryover_rebuild.rs` | 0 |
| Workspace tests | `cargo test --workspace` | All test runners pass; no regression in pre-existing tests (3 ignored counts unchanged from prior baseline). |

## Threat Surface Scan

Threat register (T-04-03-01..03) was honored:

- **T-04-03-01 (EoP):** Mitigated. `check_permission(CUTOVER_ADMIN_PRIVILEGE, ctx)` runs before any DB work. `rebuild_forbidden_for_unprivileged` test verifies the gate AND that no sub-service call leaks past it (`.times(0)` on Reporting, Carryover, TransactionDao).
- **T-04-03-02 (Tampering — wrong year):** Accepted (per plan). The cutover Tx scope set is sourced from `find_legacy_scope_set` and external callers are blocked by the permission gate.
- **T-04-03-03 (Spoofing — Authentication::Full):** Mitigated. No REST handler exists for `CarryoverRebuildService`; only `CutoverServiceImpl::run` (Plan 04-05) constructs `Authentication::Full` for service-internal calls. Verified by inspection of the Plan 04-02 cutover.rs (no Wave-2 REST surface ships in this plan).

No new threat surface introduced beyond the threat register.

## Hand-off Note for Plan 04-06 (Wave 2)

- **DI wiring in `shifty_bin/src/main.rs`:** add `CarryoverRebuildServiceDependencies` to the DI block. Construction order: AFTER `ReportingServiceImpl` and `CarryoverServiceImpl` (both already exist), BEFORE `CutoverServiceImpl` (which consumes `CarryoverRebuildService` per `service_impl/src/cutover.rs:60`). Concrete dep struct shape:
  ```rust
  // Place after ReportingService construction; consumes existing reporting_service + carryover_service Arcs.
  pub struct CarryoverRebuildServiceDependencies {
      type Context = ...; // matches the bin's context type
      type Transaction = ...; // matches the bin's transaction type
      type ReportingService = ReportingServiceImpl<...>;
      type CarryoverService = CarryoverServiceImpl<...>;
      type PermissionService = PermissionServiceImpl<...>;
      type TransactionDao = TransactionDaoImpl<...>;
  }
  ```
  No new DAO or migration required — the service writes via the existing `CarryoverDao::upsert` path.

- **Plan 04-05 cutover-gate:** the gate phase (`CutoverServiceImpl::run` after the migration phase) iterates `GateResult::scope_set: Arc<[(Uuid, u32)]>` and calls `carryover_rebuild_service.rebuild_for_year(sp, year, Authentication::Full, Some(tx.clone()))` per tuple. The cutover Tx must already have flipped `absence_range_source_active = true` BEFORE this loop so ReportingService reads from the new source.

## Self-Check: PASSED

Verification of summary claims:

- **Files created exist:**
  - `service_impl/src/carryover_rebuild.rs` — FOUND
  - `.planning/phases/04-migration-cutover/04-03-carryover-rebuild-service-SUMMARY.md` — FOUND (this file)
- **Files modified contain expected content:**
  - `service_impl/src/lib.rs` — `pub mod carryover_rebuild;` present (alphabetical between `carryover` and `clock`)
  - `service_impl/src/test/carryover_rebuild.rs` — `#[ignore]` count = 0; `rebuild_forbidden_for_unprivileged` defined as `#[tokio::test]`
- **jj changes exist:**
  - `twwlpwqt e3fda1e5` — Task 1 (feat)
  - `woroklmt 0ce64a01` — Task 2 (test)
- **Acceptance criteria met:**
  - 7/7 grep-based criteria for Task 1 pass (with the cycle-break grep tightened to code-level — see Deviation 3)
  - 2/2 criteria for Task 2 pass (0 `#[ignore]`, test green)
  - `cargo build --workspace` exit 0
  - `cargo test -p service_impl test::carryover_rebuild` reports 1 passed / 0 ignored

---

*Phase: 04-migration-cutover*
*Plan: 03 (carryover-rebuild-service)*
*Completed: 2026-05-03*
