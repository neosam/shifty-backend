---
plan: 04-03-carryover-rebuild-service
phase: 4
wave: 1
depends_on: [04-01-service-traits-and-stubs]
requirements: [MIG-04]
files_modified:
  - service_impl/src/lib.rs
  - service_impl/src/carryover_rebuild.rs
  - service_impl/src/test/carryover_rebuild.rs
autonomous: true
must_haves:
  truths:
    - "Locked decision: NEW Business-Logic-Tier service `CarryoverRebuildService` (variant A from CONTEXT.md C-Phase4-02). Variants B (CutoverService inline) and C (CarryoverService promoted to Business-Logic) are explicitly rejected."
    - "`CarryoverRebuildServiceImpl::rebuild_for_year(sp, year, ctx, tx)` derives the new carryover_hours + vacation values from `ReportingService` output (which reads via `derive_hours_for_range` because the FeatureFlag is true within the cutover Tx)."
    - "Permission: CUTOVER_ADMIN_PRIVILEGE (called only inside the cutover Tx with `Authentication::Full`-bypass — but the trait still enforces the gate for any external caller)."
    - "DI dependencies: ReportingService + CarryoverService + PermissionService + TransactionDao. NO consumption of CutoverService (no cycle)."
    - "`_forbidden`-test green."
  artifacts:
    - path: "service_impl/src/carryover_rebuild.rs"
      provides: "CarryoverRebuildServiceImpl with gen_service_impl! DI block"
    - path: "service_impl/src/test/carryover_rebuild.rs"
      provides: "_forbidden test (rebuild_forbidden_for_unprivileged) implementation"
  key_links:
    - from: "CarryoverRebuildServiceImpl::rebuild_for_year"
      to: "ReportingService::get_report_for_employee_range (Read)"
      via: "Standard Authentication::Full bypass for service-internal call"
    - from: "CarryoverRebuildServiceImpl::rebuild_for_year"
      to: "CarryoverService::set_carryover (Write)"
      via: "DELETE existing row + UPSERT new — actual SQL pattern depends on existing CarryoverService::set_carryover semantics; verify via reading service/src/carryover.rs"
    - from: "ReportingService"
      to: "AbsenceService::derive_hours_for_range (transitively, when feature_flag is on)"
      via: "Phase-2 wiring — within cutover Tx the flag is already true, so reads pull from absence_period source"
---

<objective>
Wave 1 — Implement the NEW Business-Logic-Tier service `CarryoverRebuildService` that resolves the Reporting → Carryover → Reporting cycle (Pitfall 1 in RESEARCH.md). The trait is already defined by Plan 04-01; this plan implements it with `ReportingService` (read) + `CarryoverService` (write) DI.

Locked decision (per CONTEXT.md C-Phase4-02 + RESEARCH.md "Architectural Responsibility Map" row 3):
**Variant A: separate `CarryoverRebuildService` (Business-Logic) consuming `CarryoverService` (Basic) + `ReportingService` (Business-Logic).** Reusable for the deferred bulk-rebuild surface; cleanest tier-respecting structure.
- Variant B (inline in CutoverService): rejected — CutoverService deps would balloon to 7+; the rebuild logic is reusable enough to deserve its own service.
- Variant C (promote CarryoverService to Business-Logic): rejected — breaks the Phase 1-3 contract that CarryoverService is basic; downstream ripple to multiple existing service constructions.

Purpose: A clean cycle-breaking service that the cutover Tx calls per (sp, year) tuple. Independent of Plan 04-02 (no file overlap, no logical dep) — they run in parallel within Wave 1.

Output: 1 new service impl file, 1 implemented forbidden test.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/04-migration-cutover/04-CONTEXT.md
@.planning/phases/04-migration-cutover/04-RESEARCH.md
@.planning/phases/04-migration-cutover/04-VALIDATION.md
@.planning/phases/04-migration-cutover/04-01-SUMMARY.md

@service/src/carryover_rebuild.rs
@service/src/carryover.rs
@service/src/reporting.rs
@service_impl/src/carryover.rs
@service_impl/src/reporting.rs

<interfaces>
<!-- Contracts the executor needs verbatim. -->

From `service/src/carryover_rebuild.rs` (Plan 04-01 trait — DO NOT modify):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait CarryoverRebuildService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
```

From `service/src/carryover.rs` (existing Basic-Tier surface — call `set_carryover`):
```rust
async fn get_carryover(&self, sp_id: Uuid, year: u32, ctx, tx) -> Result<Option<Carryover>, ServiceError>;
async fn set_carryover(&self, carryover: &Carryover, ctx, tx) -> Result<(), ServiceError>;
```

From `service/src/reporting.rs` (existing trait — verify exact method to derive `(carryover_hours, vacation)` for one (sp, year)). Likely candidates:
- `get_report_for_employee` (full year report — extract `balance_hours` and `vacation_carryover` fields)
- A range-based report — Plan-Phase MUST verify the actual method name and parameters by reading `service/src/reporting.rs` end-to-end.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: service_impl/src/carryover_rebuild.rs implementation</name>
  <read_first>
    - service/src/carryover_rebuild.rs (trait — Plan 04-01 output)
    - service/src/carryover.rs (Carryover struct — what `set_carryover` expects)
    - service_impl/src/carryover.rs (existing Basic impl — Z. 14-19 DI block as template; how `set_carryover` mutates DB)
    - service/src/reporting.rs (full file — find the method that reports per-employee + per-year carryover-relevant metrics)
    - service_impl/src/reporting.rs (Z. 460-540 — Phase-2 reporting-switch context; see how reports already incorporate `derive_hours_for_range` when flag is on)
    - service_impl/src/feature_flag.rs (DI block + Authentication::Full bypass pattern — Z. 1-67)
    - service/src/permission.rs (PermissionService::check_permission signature)
    - service/src/cutover.rs (CUTOVER_ADMIN_PRIVILEGE constant — Plan 04-01 export)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-12 / D-Phase4-14"
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Pitfall 1 — Cycle-Auflösung"
  </read_first>
  <action>
**Patch `service_impl/src/lib.rs`:** add `pub mod carryover_rebuild;` (alphabetical, before `clock` and after `carryover`).

**Create `service_impl/src/carryover_rebuild.rs`** with the implementation. Skeleton:

```rust
//! Phase 4 — CarryoverRebuildService implementation.
//!
//! Cycle-breaker: this service consumes ReportingService (read) and
//! CarryoverService (write). The existing CarryoverService remains
//! basic-tier and DOES NOT consume ReportingService — preserving the
//! `Reporting -> Carryover` directionality from Phase 1-3.

use async_trait::async_trait;
use uuid::Uuid;

use service::carryover::{Carryover, CarryoverService};
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::CUTOVER_ADMIN_PRIVILEGE;
use service::permission::{Authentication, PermissionService};
use service::reporting::ReportingService;
use service::ServiceError;
use shifty_macros::gen_service_impl;

gen_service_impl! {
    struct CarryoverRebuildServiceImpl: service::carryover_rebuild::CarryoverRebuildService = CarryoverRebuildServiceDeps {
        ReportingService: service::reporting::ReportingService = reporting_service,
        CarryoverService: service::carryover::CarryoverService = carryover_service,
        PermissionService: service::permission::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}

#[async_trait]
impl<Deps: CarryoverRebuildServiceDeps> CarryoverRebuildService for CarryoverRebuildServiceImpl<Deps>
where
    Self: Send + Sync,
{
    type Context = Deps::Context;
    type Transaction = <Deps::TransactionDao as dao::TransactionDao>::Transaction;

    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Permission: CUTOVER_ADMIN_PRIVILEGE. The cutover Tx caller passes
        // `Authentication::Full`, which bypasses the check (per the established
        // pattern in service_impl/src/feature_flag.rs:31-41).
        self.permission_service
            .check_permission(CUTOVER_ADMIN_PRIVILEGE, context.clone())
            .await?;

        // Use the caller-passed Tx (cutover holds it open).
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // 1. Read the up-to-date report for (sp, year). Within the cutover Tx
        //    the feature_flag is already `true`, so ReportingService internally
        //    pulls from `derive_hours_for_range` for the 3 absence categories.
        //
        //    EXACT METHOD NAME TO VERIFY by reading service/src/reporting.rs:
        //    likely `get_report_for_employee(sales_person_id, year, until_week=53, ctx, tx)`
        //    or similar. Adjust call accordingly.
        let report = self.reporting_service
            .get_report_for_employee(sales_person_id, year, /*until_week*/ 53, Authentication::Full, Some(tx.clone()))
            .await?;

        // 2. Extract carryover_hours + vacation from the report. Field names
        //    must match the actual `EmployeeReport` struct — VERIFY by reading
        //    service/src/reporting.rs report-struct definition.
        //    Likely fields: report.balance_hours (f32) and report.vacation_carryover (i32)
        //    or report.vacation_hours_left (i32).
        let new_carryover_hours: f32 = report.balance_hours;       // verify field
        let new_vacation: i32 = report.vacation_carryover.try_into().unwrap_or(0);  // verify

        // 3. Build the Carryover struct + write via CarryoverService::set_carryover.
        //    Reuse the existing version if a row exists (UPSERT semantics);
        //    otherwise create a fresh version Uuid.
        let now = time::OffsetDateTime::now_utc();
        let now = time::PrimitiveDateTime::new(now.date(), now.time());

        let existing = self.carryover_service
            .get_carryover(sales_person_id, year, Authentication::Full, Some(tx.clone()))
            .await?;

        let new_row = Carryover {
            sales_person_id,
            year,
            carryover_hours: new_carryover_hours,
            vacation: new_vacation,
            created: existing.as_ref().map(|c| c.created).unwrap_or(now),
            deleted: None,
            version: uuid::Uuid::new_v4(),
        };

        self.carryover_service
            .set_carryover(&new_row, Authentication::Full, Some(tx.clone()))
            .await?;

        // Do NOT commit — caller (CutoverService::run) holds the Tx open until
        // the entire atomic operation completes.

        Ok(())
    }
}
```

**Important verifications the executor MUST perform before committing the file:**
- Read `service/src/reporting.rs` end-to-end. Identify the EXACT method that returns per-employee + per-year carryover-relevant numbers. The placeholder `get_report_for_employee(...)` MUST be replaced with the verified method name + signature.
- Read the `EmployeeReport` (or whatever the return type is) and confirm the field names used here (`balance_hours`, `vacation_carryover`). If the names differ, adapt and document in the SUMMARY.
- Verify `gen_service_impl!` import path matches existing service_impl files (likely `use shifty_macros::gen_service_impl;`).
- If `set_carryover` does NOT do UPSERT (e.g., requires DELETE first), wire the additional call via the existing CarryoverDao surface.
  </action>
  <acceptance_criteria>
    - File `service_impl/src/carryover_rebuild.rs` exists; `grep -q 'impl<Deps: CarryoverRebuildServiceDeps> CarryoverRebuildService for CarryoverRebuildServiceImpl' service_impl/src/carryover_rebuild.rs` exits 0
    - `grep -q 'gen_service_impl!' service_impl/src/carryover_rebuild.rs` exits 0
    - `grep -q 'pub mod carryover_rebuild' service_impl/src/lib.rs` exits 0
    - `grep -q 'check_permission(CUTOVER_ADMIN_PRIVILEGE' service_impl/src/carryover_rebuild.rs` exits 0
    - `grep -q 'reporting_service' service_impl/src/carryover_rebuild.rs` exits 0
    - `grep -q 'carryover_service' service_impl/src/carryover_rebuild.rs` exits 0
    - `grep -q 'CutoverService\|CutoverServiceImpl' service_impl/src/carryover_rebuild.rs` exits 1 (NO consumption of CutoverService — no cycle)
    - `cargo build -p service_impl` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service_impl</automated>
  </verify>
  <done>
    `CarryoverRebuildServiceImpl` compiles; cycle-free (no CutoverService dep). The cutover Tx (Plan 04-05) can call `rebuild_for_year` per (sp, year) tuple.
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement `rebuild_forbidden_for_unprivileged` test in service_impl/src/test/carryover_rebuild.rs</name>
  <read_first>
    - service_impl/src/test/carryover_rebuild.rs (Wave-0 stub from Plan 04-01 Task 4)
    - service_impl/src/test/feature_flag.rs (forbidden-test pattern using MockPermissionService)
    - service_impl/src/test/carryover.rs (existing CarryoverService tests as boilerplate template)
    - service_impl/src/carryover_rebuild.rs (Task 1 above)
  </read_first>
  <action>
Replace the `#[ignore] + unimplemented!()` stub with the actual test:

```rust
//! Phase 4 — CarryoverRebuildService service-level tests.

use std::sync::Arc;
use uuid::Uuid;

use service::carryover_rebuild::CarryoverRebuildService;
use service::permission::{Authentication, MockPermissionService};
use service::reporting::MockReportingService;
use service::carryover::MockCarryoverService;
use service::ServiceError;
use service_impl::carryover_rebuild::{CarryoverRebuildServiceDeps, CarryoverRebuildServiceImpl};
use dao::MockTransactionDao;

#[tokio::test]
async fn rebuild_forbidden_for_unprivileged() {
    // Arrange: PermissionService returns Forbidden for CUTOVER_ADMIN_PRIVILEGE.
    let mut permission = MockPermissionService::new();
    permission
        .expect_check_permission()
        .returning(|_, _| Box::pin(async { Err(ServiceError::Forbidden) }));

    // Reporting + Carryover MUST NOT be called — wire .times(0).
    let mut reporting = MockReportingService::new();
    reporting.expect_get_report_for_employee().times(0);

    let mut carryover = MockCarryoverService::new();
    carryover.expect_set_carryover().times(0);
    carryover.expect_get_carryover().times(0);

    let mut tx_dao = MockTransactionDao::new();
    tx_dao.expect_use_transaction().times(0);

    // Build the service. Use whatever struct/builder the gen_service_impl! macro
    // generates — see service_impl/src/test/feature_flag.rs for the pattern.
    let svc = CarryoverRebuildServiceImpl {
        reporting_service: Arc::new(reporting),
        carryover_service: Arc::new(carryover),
        permission_service: Arc::new(permission),
        transaction_dao: Arc::new(tx_dao),
    };

    let result = svc
        .rebuild_for_year(Uuid::new_v4(), 2024, Authentication::Authenticated(()), None)
        .await;

    assert!(matches!(result, Err(ServiceError::Forbidden)));
}
```

Verify the actual struct shape generated by `gen_service_impl!` matches — adapt field names if they differ. Reference `service_impl/src/test/feature_flag.rs` for the exact constructor pattern.
  </action>
  <acceptance_criteria>
    - `grep -c '#\[ignore' service_impl/src/test/carryover_rebuild.rs` returns 0 (test must run, not be skipped)
    - `cargo test -p service_impl test::carryover_rebuild::rebuild_forbidden_for_unprivileged` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::carryover_rebuild</automated>
  </verify>
  <done>
    Forbidden test green; PermissionGate proven for CarryoverRebuildService.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| External REST surface | None — `CarryoverRebuildService` has NO REST handler. Only `CutoverServiceImpl::run` calls it. |
| Internal sub-service trust | `Authentication::Full` is passed by `CutoverServiceImpl::run` to bypass per-call permission re-checks (caller already gated). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-03-01 | Elevation of Privilege | `rebuild_for_year` could overwrite carryover values for arbitrary employees | mitigate | Permission gate `check_permission(CUTOVER_ADMIN_PRIVILEGE, ctx)` at the top of the method; forbidden test verifies. Cycle break ensures no other service can be tricked into calling it transitively. |
| T-04-03-02 | Tampering | Wrong year passed could rebuild a year that wasn't migrated | accept | The cutover Tx flow only calls `rebuild_for_year` for (sp, year) tuples in `gate_scope_set`, which is sourced from `find_legacy_scope_set`. External callers are blocked by the permission gate. Documented in trait doc. |
| T-04-03-03 | Spoofing | `Authentication::Full` is internal-only — no public REST surface exposes it | mitigate | No REST handler exists for `CarryoverRebuildService`; only `CutoverServiceImpl::run` constructs `Authentication::Full`. Verified by inspection. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test -p service_impl test::carryover_rebuild` reports >=1 passed, 0 ignored
- No file in `service_impl/src/cutover.rs` modified (Plan 04-02 owns it)
- `cargo run`-smoke deferred to Plan 04-06 DI wiring (Wave 2)
- Cycle break verified: `grep -q 'CutoverService' service_impl/src/carryover_rebuild.rs` returns 1 (no match)
</verification>

<success_criteria>
1. CarryoverRebuildService implementation lands cleanly with the documented variant-A decision.
2. Forbidden test green.
3. Wave-2 plan 04-06 can wire DI (CutoverServiceDependencies includes CarryoverRebuildService).
4. No cycle introduced (verified via grep).
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-03-SUMMARY.md` listing:
- Locked decision: variant A (separate CarryoverRebuildService BL) chosen; B + C explicitly rejected with reasoning
- ReportingService method actually used (verified vs. placeholder); EmployeeReport field names actually used
- DI deps used (4: ReportingService, CarryoverService, PermissionService, TransactionDao)
- Test outcomes: 1 passed
- Hand-off note for Plan 04-06: "CarryoverRebuildServiceDependencies needs to be added to shifty_bin/src/main.rs DI block; construct AFTER ReportingService."
</output>
