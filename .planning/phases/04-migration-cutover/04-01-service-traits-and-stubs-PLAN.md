---
plan: 04-01-service-traits-and-stubs
phase: 4
wave: 0
depends_on: [04-00-foundation-and-migrations]
requirements: [MIG-01, MIG-02, MIG-03, MIG-04, MIG-05]
files_modified:
  - service/src/lib.rs
  - service/src/cutover.rs
  - service/src/carryover_rebuild.rs
  - service/src/extra_hours.rs
  - dao/src/lib.rs
  - dao/src/cutover.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/cutover.rs
  - service_impl/src/test/carryover_rebuild.rs
autonomous: true
must_haves:
  truths:
    - "Trait `service::cutover::CutoverService` exists with `run(dry_run, ctx, tx)` + `profile(ctx, tx)` method signatures and `automock`-derived mocks."
    - "Trait `service::carryover_rebuild::CarryoverRebuildService` exists with `rebuild_for_year(sp, year, ctx, tx)` signature and `automock`-derived mocks."
    - "Trait `dao::cutover::CutoverDao` exists with all 8 read/write methods needed by Wave 1+2 (find_legacy_extra_hours_not_yet_migrated, find_all_legacy_extra_hours, upsert_quarantine, upsert_migration_source, find_legacy_scope_set, sum_legacy_extra_hours, count_quarantine_for_drift_row, backup_carryover_for_scope)."
    - "ServiceError has new variant `ExtraHoursCategoryDeprecated(ExtraHoursCategory)` (D-Phase4-09 + Operation 4 in RESEARCH.md)."
    - "ExtraHoursService trait has new method `soft_delete_bulk(ids, update_process, ctx, tx)` (C-Phase4-04 vorgegeben)."
    - "`service_impl/src/test/cutover.rs` and `service_impl/src/test/carryover_rebuild.rs` are scaffolded with all tests from the Per-Task Verification Map as `#[ignore] + unimplemented!()` stubs (Phase-3-Wave-0-pattern)."
    - "`cargo build --workspace` is GREEN."
  artifacts:
    - path: "service/src/cutover.rs"
      provides: "CutoverService trait + DTOs (CutoverRunResult, MigrationResult, GateResult, DriftRow, QuarantineReason enum, CutoverProfile)"
    - path: "service/src/carryover_rebuild.rs"
      provides: "CarryoverRebuildService trait — Business-Logic-Tier surface to break Reporting→Carryover cycle (Pitfall 1)"
    - path: "dao/src/cutover.rs"
      provides: "CutoverDao trait — global read across extra_hours + writes to quarantine, mapping, backup tables (8 async methods total)"
    - path: "service/src/extra_hours.rs"
      provides: "soft_delete_bulk added to existing ExtraHoursService trait"
    - path: "service_impl/src/test/cutover.rs"
      provides: "Test scaffolding (#[ignore] stubs) for Wave 1+2+3 implementations"
  key_links:
    - from: "service/src/cutover.rs"
      to: "service::carryover_rebuild::CarryoverRebuildService (Sub-Service-Dep)"
      via: "trait import — Wave 2 wires it via gen_service_impl!"
    - from: "ServiceError::ExtraHoursCategoryDeprecated"
      to: "rest/src/lib.rs::error_handler — Wave 2 maps to HTTP 403"
      via: "thiserror Display + match arm in error_handler"
    - from: "ExtraHoursService::soft_delete_bulk"
      to: "service_impl::cutover::CutoverServiceImpl::run COMMIT-PHASE step (c)"
      via: "Wave 2 calls it inside the atomic Tx with `Authentication::Full`"
---

<objective>
Wave 0 (companion to 04-00) — define every Rust trait, DTO, and ServiceError variant that Waves 1+2+3 will implement against, plus scaffold the `#[ignore] + unimplemented!()` test-stub files (Phase-3 Wave-0 pattern). NO logic, NO DAO impl. The output is exclusively contracts.

This plan separately exists from 04-00 because:
- 04-00 modifies migrations + Cargo.toml (operations / build-system),
- 04-01 modifies the Rust type-surface (service / dao crates),
- They are independent files (no overlap), so they share Wave 0 and run in parallel.

Purpose: Wave 1 plans (`04-02`, `04-03`, `04-04`) can each implement against fully-defined traits without needing to negotiate signatures mid-flight. Wave 2 plans can also pre-cite types in their `<interfaces>` blocks.

Output: 5 new files, 4 patches.
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

@service/src/lib.rs
@service/src/feature_flag.rs
@service/src/carryover.rs
@service/src/extra_hours.rs
@service/src/absence.rs
@dao/src/lib.rs
@dao/src/extra_hours.rs

<interfaces>
<!-- Existing trait conventions the new traits must match. -->

From `service/src/feature_flag.rs:36-52` (existing FeatureFlagService — model the new traits on this shape):
```rust
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait FeatureFlagService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn is_enabled(
        &self,
        key: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;
    async fn set(...) -> Result<(), ServiceError>;
}
```

From `service/src/lib.rs:62-123` (ServiceError enum — new variant inserted here):
```rust
pub enum ServiceError {
    ...
    #[error("Internal error")]
    InternalError,
}
```

From `service/src/extra_hours.rs:184-228` (existing ExtraHoursService trait — `soft_delete_bulk` is appended here):
```rust
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait ExtraHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;
    async fn find_by_sales_person_id_and_year(...) -> Result<Arc<[ExtraHours]>, ServiceError>;
    ...
    async fn delete(&self, id: Uuid, ctx: ..., tx: ...) -> Result<(), ServiceError>;
}
```

From `service/src/absence.rs:118-130` (DerivedDayHours struct — DriftRow uses AbsenceCategory):
```rust
pub struct DerivedDayHours {
    pub date: Date,
    pub category: AbsenceCategory,
    pub hours: f32,
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: ServiceError::ExtraHoursCategoryDeprecated + ExtraHoursService::soft_delete_bulk + lib.rs mod imports</name>
  <read_first>
    - service/src/lib.rs
    - service/src/extra_hours.rs (Z. 184-228 — trait definition; Z. 41-52 — ExtraHoursCategory enum)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Operation 4: ServiceError-Variante + error_handler-Mapping → 403"
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-09 / D-Phase4-10 / C-Phase4-04"
  </read_first>
  <action>
**1. Patch `service/src/lib.rs`:**

a) Insert two new `pub mod` lines in alphabetical order in the existing module-import block (lines 7-40). Add:
```rust
pub mod carryover_rebuild;
pub mod cutover;
```
Both alphabetical (after `carryover` and before `clock`, after `custom_extra_hours` and before `datetime_utils` — verify exact position by reading lines 7-40 first).

b) Add a new variant to the `ServiceError` enum (after `NotLatestBillingPeriod(Uuid)`, before `InternalError`):
```rust
    #[error("ExtraHours category {0:?} is deprecated; use POST /absence-period for this category")]
    ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory),
```

The choice of `Display` text matches Operation 4 in RESEARCH.md so the error_handler can extract the category via `format!("{:?}", err)` parsing OR — cleaner — via direct match. Wave 2 chooses the cleaner pattern.

**2. Patch `service/src/extra_hours.rs`:**

Append a new method to the `ExtraHoursService` trait (inside the `#[automock] #[async_trait] pub trait ExtraHoursService { ... }` block, after `delete` (~line 225)):
```rust
    /// Bulk soft-delete (Phase 4 cutover, C-Phase4-04). Marks every id as
    /// `deleted = NOW()` with a caller-provided `update_process` tag for audit.
    /// Bypasses per-row permission checks: caller MUST hold `cutover_admin` and
    /// pass the cutover-tx as `Some(tx)`. ANY id not present is silently
    /// ignored (idempotent for re-runs).
    async fn soft_delete_bulk(
        &self,
        ids: Arc<[Uuid]>,
        update_process: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
```

Make sure `Arc` and `Uuid` are already imported (they are — verify with `grep -n "use std::sync::Arc\|use uuid::Uuid" service/src/extra_hours.rs`).
  </action>
  <acceptance_criteria>
    - `grep -q 'pub mod cutover;' service/src/lib.rs` exits 0
    - `grep -q 'pub mod carryover_rebuild;' service/src/lib.rs` exits 0
    - `grep -q 'ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory)' service/src/lib.rs` exits 0
    - `grep -q 'async fn soft_delete_bulk' service/src/extra_hours.rs` exits 0
    - `cargo build -p service` exits 0 (will fail with E0583 if cutover.rs/carryover_rebuild.rs files do not yet exist — Task 2/3 create them; ordering inside Task 1 means we cannot finalize without 2+3 — execute Task 1, 2, 3 in sequence and run `cargo build -p service` only after Task 3)
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service</automated>
  </verify>
  <done>
    `service/src/lib.rs` mod imports present; ServiceError variant present; ExtraHoursService trait extended. (Build will fail until Task 2 and Task 3 land — that is expected; the in-task verify runs after Task 3.)
  </done>
</task>

<task type="auto">
  <name>Task 2: service/src/cutover.rs trait + DTOs + service/src/carryover_rebuild.rs trait</name>
  <read_first>
    - service/src/feature_flag.rs (full file — Phase-2 trait shape exemplar)
    - service/src/absence.rs (Z. 113-130 DerivedDayHours; Z. 28-50 AbsenceCategory; Z. 200-260 trait shape)
    - service/src/carryover.rs (existing CarryoverService — DO NOT extend; the new CarryoverRebuildService is separate)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "<domain> MIG-03 — CutoverRunResult / CutoverService surface"
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Architectural Responsibility Map" (rows 1-5) + § "Pitfall 1: Cycle …" + § "Pattern 3: Permission-Branch-Innerhalb-`run`-Method"
  </read_first>
  <action>
**Create `service/src/cutover.rs`** with the full trait + DTOs (Trait stubs only — no `_impl` here). The locked design:

```rust
//! Phase 4 — Cutover orchestration trait + DTOs.
//!
//! Business-Logic-Tier service per shifty-backend/CLAUDE.md § "Service-Tier-
//! Konventionen". Consumes AbsenceService + ExtraHoursService +
//! CarryoverRebuildService + FeatureFlagService + EmployeeWorkDetailsService +
//! CutoverDao + PermissionService + TransactionDao. NO Sub-Service depends on
//! CutoverService — no cycle.

use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::absence::AbsenceCategory;
use crate::permission::Authentication;
use crate::ServiceError;

pub const CUTOVER_ADMIN_PRIVILEGE: &str = "cutover_admin";

/// Reasons an extra_hours row landed in the quarantine table.
/// Persisted as snake_case strings; see D-Phase4-03 / specifics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarantineReason {
    AmountBelowContractHours,
    AmountAboveContractHours,
    ContractHoursZeroForDay,
    ContractNotActiveAtDate,
    Iso53WeekGap,
}

impl QuarantineReason {
    pub fn as_persisted_str(&self) -> &'static str {
        match self {
            Self::AmountBelowContractHours => "amount_below_contract_hours",
            Self::AmountAboveContractHours => "amount_above_contract_hours",
            Self::ContractHoursZeroForDay => "contract_hours_zero_for_day",
            Self::ContractNotActiveAtDate => "contract_not_active_at_date",
            Self::Iso53WeekGap => "iso_53_week_gap",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DriftRow {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub category: AbsenceCategory,
    pub year: u32,
    pub legacy_sum: f32,
    pub derived_sum: f32,
    pub drift: f32,
    pub quarantined_extra_hours_count: u32,
    pub quarantine_reasons: Arc<[Arc<str>]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GateResult {
    pub passed: bool,
    pub drift_rows: Arc<[DriftRow]>,
    pub diff_report_path: Arc<str>,
    /// Set of (sales_person_id, year) tuples that the gate evaluated, used
    /// downstream by the carryover refresh scope (D-Phase4-12).
    pub scope_set: Arc<[(Uuid, u32)]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverRunResult {
    pub run_id: Uuid,
    pub ran_at: time::PrimitiveDateTime,
    pub dry_run: bool,
    pub gate_passed: bool,
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,
    pub diff_report_path: Option<Arc<str>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverProfileBucket {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub category: AbsenceCategory,
    pub year: u32,
    pub row_count: u32,
    pub sum_amount: f32,
    pub fractional_count: u32,
    pub weekend_on_workday_only_contract_count: u32,
    pub iso_53_indicator: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutoverProfile {
    pub run_id: Uuid,
    pub generated_at: time::PrimitiveDateTime,
    pub buckets: Arc<[CutoverProfileBucket]>,
    pub profile_path: Arc<str>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait CutoverService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Single entry point — both `/admin/cutover/gate-dry-run` and
    /// `/admin/cutover/commit` call this with different `dry_run` values.
    /// Permission check inside (HR for dry_run; cutover_admin for commit) per
    /// Pattern 3 in RESEARCH.md (D-Phase4-08).
    async fn run(
        &self,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverRunResult, ServiceError>;

    /// Production-data profile (SC-1 / C-Phase4-05). Read-only — runs full
    /// extra_hours scan and writes `.planning/migration-backup/profile-{ts}.json`.
    /// Permission: HR. Separate from `run` because it must remain runnable on
    /// arbitrarily large data without affecting cutover state.
    async fn profile(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverProfile, ServiceError>;
}
```

**Create `service/src/carryover_rebuild.rs`** with the cycle-breaking rebuild trait (Pitfall 1 + Architectural Responsibility Map row 3):

```rust
//! Phase 4 — Cycle-breaking carryover rebuild surface.
//!
//! ## Locked decision (Plan 04-03 enforces)
//!
//! Per **C-Phase4-02** + **Pitfall 1** in RESEARCH.md, this service is a
//! NEW Business-Logic-Tier service that consumes ReportingService (Read) and
//! CarryoverService (Write). The existing `CarryoverService` (basic-tier)
//! stays basic — it MUST NOT consume ReportingService, otherwise:
//! `Reporting -> Carryover -> Reporting` cycle.
//!
//! Variant rejected: extending CarryoverService with `rebuild_for_year`. That
//! would force CarryoverService into the Business-Logic tier (breaks Phase 1-3
//! contracts that treat it as basic). Rejected per CLAUDE.md Service-Tier-
//! Konvention.
//!
//! Variant rejected: inlining the rebuild in CutoverServiceImpl. That makes
//! CutoverServiceImpl 6 deps wide instead of 5 and is not reusable for any
//! future bulk-rebuild surface (currently deferred per CONTEXT.md `<deferred>`).

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait CarryoverRebuildService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Re-derive employee_yearly_carryover for `(sales_person_id, year)` from
    /// ReportingService output. Caller passes the cutover Tx as `Some(tx)` so
    /// the read sees the post-flag-flip state. Permission: cutover_admin
    /// (called only inside the cutover Tx with `Authentication::Full`-bypass).
    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
```
  </action>
  <acceptance_criteria>
    - File `service/src/cutover.rs` exists; `grep -q 'pub trait CutoverService' service/src/cutover.rs` exits 0
    - `grep -q 'pub const CUTOVER_ADMIN_PRIVILEGE: &str = "cutover_admin"' service/src/cutover.rs` exits 0
    - `grep -q 'pub enum QuarantineReason' service/src/cutover.rs` exits 0
    - `grep -q 'pub struct CutoverRunResult' service/src/cutover.rs` exits 0
    - `grep -q 'pub struct CutoverProfile' service/src/cutover.rs` exits 0
    - File `service/src/carryover_rebuild.rs` exists; `grep -q 'pub trait CarryoverRebuildService' service/src/carryover_rebuild.rs` exits 0
    - Locked decision is documented at the top of `service/src/carryover_rebuild.rs` (`grep -q "C-Phase4-02" service/src/carryover_rebuild.rs` exits 0)
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service</automated>
  </verify>
  <done>
    Two new trait files compile inside the `service` crate; mocks are auto-generated.
  </done>
</task>

<task type="auto">
  <name>Task 3: dao/src/cutover.rs trait + dao/src/lib.rs mod registration</name>
  <read_first>
    - dao/src/lib.rs
    - dao/src/extra_hours.rs (existing DAO trait shape)
    - dao/src/absence.rs (Phase-1 DAO pattern with BLOB(16) PK + Entity TryFrom)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "<domain> MIG-01 (Migration tables) + MIG-02 (Gate scope-set)"
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Architectural Responsibility Map" + § "Code Examples Operation 1+2"
  </read_first>
  <action>
**Create `dao/src/cutover.rs`** with the full DAO trait the cutover service needs. NO impl (Plan 04-02 owns the SQLite impl). The trait is private to `dao::cutover` — used only by `service_impl::cutover`.

```rust
//! Phase 4 — Cutover DAO surface.
//!
//! Reads legacy extra_hours globally (cross-sp scan), writes to the three
//! Phase-4 audit tables (quarantine, mapping, carryover backup). Pre-cutover
//! carryover backup is INSERT-INTO-SELECT — the trait method takes only the
//! scope set + the cutover_run_id.

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::extra_hours::ExtraHoursEntity;
use crate::{DaoError, MockTransaction, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyExtraHoursRow {
    /// Mirror of `extra_hours.id` — the idempotency key (NOT logical_id; see D-Phase4-04).
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: crate::extra_hours::ExtraHoursCategoryEntity,
    pub date_time: time::PrimitiveDateTime,
    pub amount: f32,
}

impl From<&ExtraHoursEntity> for LegacyExtraHoursRow {
    fn from(e: &ExtraHoursEntity) -> Self {
        Self {
            id: e.id,
            sales_person_id: e.sales_person_id,
            category: e.category.clone(),
            date_time: e.date_time,
            amount: e.amount,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuarantineRow {
    pub extra_hours_id: Uuid,
    pub reason: Arc<str>,
    pub sales_person_id: Uuid,
    pub category: crate::extra_hours::ExtraHoursCategoryEntity,
    pub date_time: time::PrimitiveDateTime,
    pub amount: f32,
    pub cutover_run_id: Uuid,
    pub migrated_at: time::PrimitiveDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MigrationSourceRow {
    pub extra_hours_id: Uuid,
    pub absence_period_id: Uuid,
    pub cutover_run_id: Uuid,
    pub migrated_at: time::PrimitiveDateTime,
}

#[automock(type Transaction=MockTransaction;)]
#[async_trait]
pub trait CutoverDao {
    type Transaction: Transaction;

    /// Global read of all live `extra_hours` rows in the three legacy categories
    /// (Vacation, SickLeave, UnpaidLeave) that have NOT yet been mapped (i.e.,
    /// `extra_hours.id NOT IN (SELECT extra_hours_id FROM absence_period_migration_source)`).
    /// Returns sorted by (sales_person_id, category, date_time) ASC.
    async fn find_legacy_extra_hours_not_yet_migrated(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError>;

    /// All `extra_hours` rows in the three legacy categories regardless of mapping
    /// state — used by `CutoverService::profile()` (SC-1).
    async fn find_all_legacy_extra_hours(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError>;

    /// UPSERT (INSERT ... ON CONFLICT(extra_hours_id) DO NOTHING) into
    /// `absence_period_migration_source`.
    async fn upsert_migration_source(
        &self,
        row: &MigrationSourceRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// UPSERT (INSERT ... ON CONFLICT(extra_hours_id) DO UPDATE SET reason=excluded.reason)
    /// into `absence_migration_quarantine`. Re-run idempotent: same id but new
    /// reason overwrites the prior reason (for human re-classification scenarios).
    async fn upsert_quarantine(
        &self,
        row: &QuarantineRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Distinct (sales_person_id, year) for every `extra_hours` row in the three
    /// legacy categories with non-zero amount, regardless of mapping state — the
    /// gate scope set per D-Phase4-05 + D-Phase4-12.
    async fn find_legacy_scope_set(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[(Uuid, u32)]>, DaoError>;

    /// Per-(sp, category, year) sum of `extra_hours.amount` (Vacation/SickLeave/
    /// UnpaidLeave only). Used by gate.
    async fn sum_legacy_extra_hours(
        &self,
        sales_person_id: Uuid,
        category: &crate::extra_hours::ExtraHoursCategoryEntity,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<f32, DaoError>;

    /// Number of quarantine rows for the given (sp, category, year, run_id) —
    /// used to populate DriftRow.quarantined_extra_hours_count.
    async fn count_quarantine_for_drift_row(
        &self,
        sales_person_id: Uuid,
        category: &crate::extra_hours::ExtraHoursCategoryEntity,
        year: u32,
        cutover_run_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<(u32, Arc<[Arc<str>]>), DaoError>;

    /// INSERT INTO employee_yearly_carryover_pre_cutover_backup (...) SELECT (...)
    /// FROM employee_yearly_carryover WHERE (sales_person_id, year) IN scope_set.
    /// Single-statement (multi-row) insert per D-Phase4-13.
    async fn backup_carryover_for_scope(
        &self,
        cutover_run_id: Uuid,
        backed_up_at: time::PrimitiveDateTime,
        scope: &[(Uuid, u32)],
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
```

**Patch `dao/src/lib.rs`:** add `pub mod cutover;` in alphabetical order (after `carryover` if it exists, before `extra_hours`). Verify by reading the file first to find the existing module-import block.
  </action>
  <acceptance_criteria>
    - File `dao/src/cutover.rs` exists; `grep -q 'pub trait CutoverDao' dao/src/cutover.rs` exits 0
    - The trait MUST expose all 8 async methods (find_legacy_extra_hours_not_yet_migrated, find_all_legacy_extra_hours, upsert_migration_source, upsert_quarantine, find_legacy_scope_set, sum_legacy_extra_hours, count_quarantine_for_drift_row, backup_carryover_for_scope). Verify with a comment-stripped count: `[ "$(grep -v '^[[:space:]]*//' dao/src/cutover.rs | grep -c 'async fn')" -ge 8 ]` (count must be at least 8; comments are excluded so doc-comments do not inflate or deflate the number).
    - `grep -q 'pub mod cutover' dao/src/lib.rs` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p dao</automated>
  </verify>
  <done>
    `dao` crate compiles with new trait + mocks. The trait is consumable by `service_impl::cutover` in Plan 04-02.
  </done>
</task>

<task type="auto">
  <name>Task 4: Test scaffolding stubs (#[ignore] + unimplemented!()) for Wave 1+2+3 cutover/carryover_rebuild tests</name>
  <read_first>
    - service_impl/src/test/mod.rs
    - service_impl/src/test/absence.rs (Phase-3 `#[ignore]+unimplemented!()` Wave-0-Stub-Pattern — Z. 1-100)
    - .planning/phases/04-migration-cutover/04-VALIDATION.md § "Per-Task Verification Map" (all 26 test names)
  </read_first>
  <action>
**Patch `service_impl/src/test/mod.rs`:** add two new `pub mod` lines (alphabetical order):
```rust
pub mod carryover_rebuild;
pub mod cutover;
```

**Create `service_impl/src/test/cutover.rs`** with `#[ignore]+unimplemented!()` stubs for every test from VALIDATION.md Per-Task Verification Map that targets `service_impl/src/test/cutover` (per Phase-3 Wave-0 pattern documented in STATE.md "Phase-3-Wave-0-Stub-Pattern"). Each stub:
```rust
//! Phase 4 — service-level cutover tests.
//! Wave 0 scaffolds with `#[ignore] + unimplemented!()` so `cargo test --list`
//! makes the test surface visible immediately. Wave 1 implements the heuristic
//! tests, Wave 2 implements the gate-tolerance tests; both flip `#[ignore]` off.

#[tokio::test]
#[ignore = "wave-1-implements-heuristic-cluster"]
async fn cluster_merges_consecutive_workdays_with_exact_match() {
    unimplemented!("wave-1: implement Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1");
}

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_amount_below_contract() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_amount_above_contract() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_weekend_entry_workday_contract() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_contract_not_active() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-quarantine"]
async fn quarantine_iso_53_gap() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-idempotence"]
async fn idempotent_rerun_skips_mapped() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_pass_below_threshold() { unimplemented!("wave-2"); }

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_fail_above_threshold() { unimplemented!("wave-2"); }

#[tokio::test]
#[ignore = "wave-1-implements-forbidden-tests"]
async fn run_forbidden_for_unprivileged_user() { unimplemented!("wave-1"); }

#[tokio::test]
#[ignore = "wave-1-implements-forbidden-tests"]
async fn run_forbidden_for_hr_only_when_committing() { unimplemented!("wave-1"); }
```

**Create `service_impl/src/test/carryover_rebuild.rs`** with the `_forbidden`-stub:
```rust
//! Phase 4 — CarryoverRebuildService service-level tests.

#[tokio::test]
#[ignore = "wave-1-implements-forbidden"]
async fn rebuild_forbidden_for_unprivileged() {
    unimplemented!("wave-1: implement permission gate test");
}
```

**Verify all tests are visible** in `cargo test --list`:
```bash
cargo test -p service_impl test::cutover -- --list
cargo test -p service_impl test::carryover_rebuild -- --list
```
Each must show the expected test names with `(ignored)` annotation.
  </action>
  <acceptance_criteria>
    - File `service_impl/src/test/cutover.rs` exists with at least 11 `#[tokio::test]` annotations
    - File `service_impl/src/test/carryover_rebuild.rs` exists with at least 1 `#[tokio::test]` annotation
    - `grep -q 'pub mod cutover' service_impl/src/test/mod.rs` exits 0
    - `grep -q 'pub mod carryover_rebuild' service_impl/src/test/mod.rs` exits 0
    - `cargo test -p service_impl test::cutover -- --list 2>/dev/null | grep -c "ignored"` returns at least 11
    - `cargo test -p service_impl test::cutover` runs and reports `0 passed; 0 failed; >=11 ignored`
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::cutover</automated>
  </verify>
  <done>
    All Wave-1+2 test surfaces visible in `cargo test --list`. Subsequent waves remove the `#[ignore]` and replace `unimplemented!()` with the real test body.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Trait surface boundary | New traits define the public Rust surface; downstream impls must check permissions. The trait itself does not enforce — Wave 1+2 service impls do. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-01-01 | Spoofing | `CutoverService::run` is callable internally with any `Authentication<Context>` | mitigate | Permission check happens inside `run` (Pattern 3 RESEARCH.md). Wave-1+2 service impl is required to call `permission_service.check_permission(HR or CUTOVER_ADMIN, ctx)` as the first action — verified in Plan 04-02 task 1 acceptance criteria. |
| T-04-01-02 | Tampering | `ExtraHoursService::soft_delete_bulk` could be called with arbitrary IDs | mitigate | Trait doc explicitly says "caller MUST hold cutover_admin and pass the cutover-tx" — enforced in Plan 04-04 service impl (permission check). Untyped Vec<Uuid> is fine because the only caller (CutoverServiceImpl) constructs the list from its own internal mapping table reads. |
| T-04-01-03 | Information Disclosure | `DriftRow.sales_person_name` carries PII via Service surface | mitigate | Surface is service-internal; PII only crosses the trust boundary at the REST layer (Plan 04-06 wraps it in `CutoverGateDriftRowTO`) and the file-IO boundary (`.planning/migration-backup/*.json` — `.gitignore`d in Plan 04-00 Task 1). |
| T-04-01-04 | Elevation of Privilege | New `CUTOVER_ADMIN_PRIVILEGE` constant exposed | accept | The constant is just a string; the privilege exists in DB only after Plan 04-00 migration runs. No additional risk vs. existing FEATURE_FLAG_ADMIN_PRIVILEGE pattern. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN after all 4 tasks land
- `cargo test -p service_impl test::cutover` reports `>=11 ignored`
- `cargo test -p service_impl test::carryover_rebuild` reports `>=1 ignored`
- Pitfall-1 (Reporting→Carryover→Reporting Cycle) avoided: locked-decision documented at the top of `service/src/carryover_rebuild.rs`
- ServiceError variant present and properly typed (`ExtraHoursCategory` not `Arc<str>`)
- `ExtraHoursService::soft_delete_bulk` signature matches Plan 04-04's expected signature exactly
- CutoverDao surface exposes 8 methods (comment-stripped grep) — Plan 04-02 SQLite impl can build against it without renaming
</verification>

<success_criteria>
1. `service` crate exposes `cutover::CutoverService`, `carryover_rebuild::CarryoverRebuildService`, `extra_hours::ExtraHoursService::soft_delete_bulk`, and `ServiceError::ExtraHoursCategoryDeprecated`.
2. `dao` crate exposes `cutover::CutoverDao` with all 8 trait methods.
3. Test scaffolding visible via `cargo test --list` for Wave 1+2 service-layer tests.
4. Locked decision (CarryoverRebuildService variant chosen, not CarryoverService::rebuild_for_year) is documented in source.
5. Wave 1 plans 04-02, 04-03, 04-04 can implement against frozen contracts without further surface negotiation.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-01-SUMMARY.md` listing:
- Files created (paths)
- New trait method signatures (verbatim)
- ServiceError variant signature
- Locked decision: "CarryoverRebuildService new BL service (not CarryoverService::rebuild_for_year, not CutoverService inline)"
- Verification: `cargo build --workspace` exit + `cargo test -p service_impl test::cutover` ignored count
- Hand-off note for Plan 04-02: "service::cutover::{CutoverService, CutoverRunResult, GateResult, DriftRow, QuarantineReason, CutoverProfile} are the contracts to implement; dao::cutover::CutoverDao is the DAO surface (8 methods)."
</output>
