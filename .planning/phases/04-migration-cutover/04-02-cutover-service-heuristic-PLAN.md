---
plan: 04-02-cutover-service-heuristic
phase: 4
wave: 1
depends_on: [04-00-foundation-and-migrations, 04-01-service-traits-and-stubs]
requirements: [MIG-01]
files_modified:
  - dao_impl_sqlite/src/lib.rs
  - dao_impl_sqlite/src/cutover.rs
  - service_impl/src/lib.rs
  - service_impl/src/cutover.rs
  - service_impl/src/test/cutover.rs
autonomous: true
must_haves:
  truths:
    - "`CutoverDao` impl in `dao_impl_sqlite/src/cutover.rs` provides all 8 trait methods using SQLx queries against the Wave-0 schemas."
    - "`CutoverServiceImpl` exists in `service_impl/src/cutover.rs` with `gen_service_impl!` DI block (8 sub-services per Architectural Responsibility Map row 1)."
    - "Heuristik-Cluster-Algorithmus (RESEARCH.md Operation 1) is implemented as a private helper, returning `Result<(MigrationStats, Arc<[Uuid]>), ServiceError>` where the `Arc<[Uuid]>` is the list of `extra_hours.id` values that were merged into clusters (= soft-delete-eligible) — Plan 04-05 commit_phase consumes this list verbatim."
    - "Pre-fetch optimization for EmployeeWorkDetails per sales_person (C-Phase4-06) is in place."
    - "`run(dry_run, ctx, tx)` opens the Tx, runs the heuristic + persists migration source/quarantine rows + ROLLS BACK (Wave 1 stops there — Wave 2 adds gate + commit)."
    - "Per-(sp, kategorie) cluster-merging logic produces 1 absence_period row per cluster + N migration_source mapping rows."
    - "`_forbidden`-tests (HR for dry_run / cutover_admin for commit) are GREEN (Permission Pattern 3)."
    - "5 quarantine-reason-classification tests + cluster-merge test + idempotence test are GREEN."
  artifacts:
    - path: "dao_impl_sqlite/src/cutover.rs"
      provides: "SQLx implementation of CutoverDao (8 methods)"
    - path: "service_impl/src/cutover.rs"
      provides: "CutoverServiceImpl skeleton with run() Wave-1-half-implementation (Migration phase only); migrate_legacy_extra_hours_to_clusters returns (MigrationStats, Arc<[Uuid]>)"
    - path: "service_impl/src/test/cutover.rs"
      provides: "8 implemented tests (cluster-merge + 5 quarantine + idempotence + 2 forbidden)"
  key_links:
    - from: "service_impl::cutover::CutoverServiceImpl"
      to: "dao::absence::AbsenceDao::create"
      via: "Direct DAO insert per Anti-Pattern guidance (NOT AbsenceService::create — would trigger Forward-Warning loop)"
    - from: "service_impl::cutover::CutoverServiceImpl"
      to: "service::employee_work_details::EmployeeWorkDetailsService::find_by_sales_person_id"
      via: "Pre-fetched HashMap<sp_id, Arc<[EmployeeWorkDetails]>> for Per-Tag-Lookup (C-Phase4-06)"
    - from: "Cluster algorithm"
      to: "Strict-Match D-Phase4-02"
      via: "If amount != contract_hours_at(day) OR contract_hours == 0 OR cross-year-boundary -> quarantine, break cluster"
    - from: "migrate_legacy_extra_hours_to_clusters return tuple `(MigrationStats, Arc<[Uuid]>)`"
      to: "Plan 04-05 Task 1 commit_phase soft_delete_bulk call"
      via: "The `Arc<[Uuid]>` arm is the verbatim id-list passed into ExtraHoursService::soft_delete_bulk(...) inside the cutover Tx — locked contract; Plan 04-05 consumes it directly without renaming."
---

<objective>
Wave 1 — Implement (a) the SQLite-backed `CutoverDao` and (b) the migration-phase of `CutoverServiceImpl::run` (heuristic clustering + quarantine + persistence to absence_period + mapping rows). This plan stops short of the gate (Plan 04-05) and the carryover/flip (Plan 04-06). The Tx ALWAYS rolls back at the end of `run` in Wave 1 — no commit logic yet.

Purpose: Land the largest piece of new domain logic (Heuristik) on its own with focused mock-based tests. Splitting prevents context blow-up and lets the gate plan rely on a known-good Migration phase.

Output: 1 new DAO impl file, 1 new service impl file, 8 implemented service-mock tests.
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
@.planning/phases/04-migration-cutover/04-00-SUMMARY.md
@.planning/phases/04-migration-cutover/04-01-SUMMARY.md

@service/src/cutover.rs
@service/src/carryover_rebuild.rs
@dao/src/cutover.rs
@service_impl/src/absence.rs
@service_impl/src/test/absence.rs
@dao_impl_sqlite/src/absence.rs
@dao_impl_sqlite/src/extra_hours.rs

<interfaces>
<!-- Contracts the executor consumes verbatim. -->

From `service_impl/src/feature_flag.rs:31-67` (gen_service_impl! pattern + Authentication::Full bypass):
```rust
gen_service_impl! {
    struct FeatureFlagServiceImpl: service::feature_flag::FeatureFlagService = FeatureFlagServiceDeps {
        FeatureFlagDao: dao::feature_flag::FeatureFlagDao = feature_flag_dao,
        PermissionService: service::permission::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}
```

From `service_impl/src/absence.rs:563-653` (3-Sub-Service-Tx-precedent — verbatim Tx-forwarding):
```rust
let bookings = self.booking_service.get_for_week(iso_week, iso_year as u32, Authentication::Full, tx.clone().into()).await?;
let slot = self.slot_service.get_slot(&b.slot_id, Authentication::Full, tx.clone().into()).await?;
let manual_all = self.sales_person_unavailable_service.get_all_for_sales_person(sp_id, Authentication::Full, tx.clone().into()).await?;
```

From `service_impl/src/absence.rs:418-510` (per-day contract-hours lookup + DateRange iteration):
```rust
work_details.iter().find(|wh| {
    wh.deleted.is_none()
        && wh.from_date().map(|d| d.to_date() <= day).unwrap_or(false)
        && wh.to_date().map(|d| day <= d.to_date()).unwrap_or(false)
})
```

From `dao/src/cutover.rs` (Plan 04-01 trait):
```rust
pub trait CutoverDao {
    async fn find_legacy_extra_hours_not_yet_migrated(&self, tx) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError>;
    async fn upsert_migration_source(&self, row: &MigrationSourceRow, tx) -> Result<(), DaoError>;
    async fn upsert_quarantine(&self, row: &QuarantineRow, tx) -> Result<(), DaoError>;
    // ... see plan 04-01 task 3 for full surface
}
```

From `service/src/cutover.rs` (Plan 04-01 surface — what `run` returns at end of Wave 1):
```rust
pub struct CutoverRunResult {
    pub run_id: Uuid,
    pub ran_at: time::PrimitiveDateTime,
    pub dry_run: bool,
    pub gate_passed: bool,         // Wave 1: always false (gate not implemented yet)
    pub total_clusters: u32,
    pub migrated_clusters: u32,
    pub quarantined_rows: u32,
    pub gate_drift_rows: u32,      // Wave 1: 0
    pub diff_report_path: Option<Arc<str>>,  // Wave 1: None
}
```

From `service/src/employee_work_details.rs:220-278` (verify with grep — the trait method to pre-fetch by sales_person_id; if exact name differs, use whatever lists all contracts for an sp).

From `dao/src/absence.rs` (the trait method `create(entity, update_process, tx)` for direct DAO insert per Anti-Pattern guidance).
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: dao_impl_sqlite/src/cutover.rs — SQLx impl of CutoverDao (8 methods)</name>
  <read_first>
    - dao_impl_sqlite/src/lib.rs
    - dao_impl_sqlite/src/extra_hours.rs (Phase-1 SQLx-pattern: WHERE deleted IS NULL, BLOB(16) PK)
    - dao_impl_sqlite/src/absence.rs (Phase-1 INSERT pattern with PrimitiveDateTime)
    - dao_impl_sqlite/src/lib.rs (TransactionImpl + transaction_dao_impl macro at Z. 295-338)
    - dao/src/cutover.rs (Plan 04-01 trait)
    - migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql
    - migrations/sqlite/20260503000001_create-absence-period-migration-source.sql
    - migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql
    - migrations/sqlite/20240618125847_paid-sales-persons.sql (extra_hours schema)
    - migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql
  </read_first>
  <action>
**Create `dao_impl_sqlite/src/cutover.rs`** with the SQLx-backed `CutoverDaoImpl`. Use `sqlx::query!` (compile-time-checked) only when feasible; for the dynamic IN-clause queries (e.g., the legacy-categories filter or the scope-set IN clause for backup_carryover_for_scope), use `sqlx::query_with` + `QueryBuilder::push_tuples` per existing codebase patterns.

Skeleton with the eight methods:

```rust
//! Phase 4 — SQLite implementation of dao::cutover::CutoverDao.

use std::sync::Arc;
use async_trait::async_trait;
use sqlx::QueryBuilder;
use uuid::Uuid;

use dao::cutover::{
    CutoverDao, LegacyExtraHoursRow, MigrationSourceRow, QuarantineRow,
};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::DaoError;

use crate::TransactionImpl;

pub struct CutoverDaoImpl;

#[async_trait]
impl CutoverDao for CutoverDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_legacy_extra_hours_not_yet_migrated(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError> {
        // SQL: SELECT id, sales_person_id, category, date_time, amount
        //      FROM extra_hours
        //      WHERE deleted IS NULL
        //        AND category IN ('Vacation', 'SickLeave', 'UnpaidLeave')
        //        AND id NOT IN (SELECT extra_hours_id FROM absence_period_migration_source)
        //      ORDER BY sales_person_id, category, date_time;
        // Map each row to LegacyExtraHoursRow.
        todo!("implement per Plan 04-02 Task 1")
    }

    async fn find_all_legacy_extra_hours(...) -> Result<...> {
        // Same as above WITHOUT the NOT-IN clause. Used by profile().
    }

    async fn upsert_migration_source(...) -> Result<(), DaoError> {
        // INSERT INTO absence_period_migration_source (extra_hours_id, absence_period_id, cutover_run_id, migrated_at)
        // VALUES (?, ?, ?, ?) ON CONFLICT(extra_hours_id) DO NOTHING;
    }

    async fn upsert_quarantine(...) -> Result<(), DaoError> {
        // INSERT INTO absence_migration_quarantine (extra_hours_id, reason, sales_person_id, category, date_time, amount, cutover_run_id, migrated_at)
        // VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(extra_hours_id) DO UPDATE SET
        //   reason = excluded.reason, cutover_run_id = excluded.cutover_run_id, migrated_at = excluded.migrated_at;
    }

    async fn find_legacy_scope_set(...) -> Result<Arc<[(Uuid, u32)]>, DaoError> {
        // SELECT DISTINCT sales_person_id, CAST(strftime('%Y', date_time) AS INTEGER) as year
        // FROM extra_hours
        // WHERE deleted IS NULL
        //   AND category IN ('Vacation', 'SickLeave', 'UnpaidLeave')
        //   AND amount != 0;
        // ORDER BY sales_person_id, year
    }

    async fn sum_legacy_extra_hours(...) -> Result<f32, DaoError> {
        // SELECT COALESCE(SUM(amount), 0.0)
        // FROM extra_hours
        // WHERE deleted IS NULL
        //   AND sales_person_id = ?
        //   AND category = ?
        //   AND CAST(strftime('%Y', date_time) AS INTEGER) = ?;
    }

    async fn count_quarantine_for_drift_row(...) -> Result<(u32, Arc<[Arc<str>]>), DaoError> {
        // First: SELECT COUNT(*) ... ; second: SELECT DISTINCT reason ... ; ASSEMBLE.
    }

    async fn backup_carryover_for_scope(
        &self,
        cutover_run_id: Uuid,
        backed_up_at: time::PrimitiveDateTime,
        scope: &[(Uuid, u32)],
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        // QueryBuilder::new("INSERT INTO employee_yearly_carryover_pre_cutover_backup (
        //   cutover_run_id, sales_person_id, year, carryover_hours, vacation, created, deleted, update_process, update_version, backed_up_at
        // ) SELECT ?, c.sales_person_id, c.year, c.carryover_hours, c.vacation, c.created, c.deleted, c.update_process, c.update_version, ?
        //   FROM employee_yearly_carryover c
        //   WHERE (c.sales_person_id, c.year) IN ");
        // QueryBuilder::push_tuples(scope, |b, (sp, yr)| { b.push_bind(sp).push_bind(*yr); });
        // .build().execute(tx).await
    }
}
```

Replace each `todo!` with the actual SQL using `sqlx::query!` where possible. For the scope-set IN-clause use `QueryBuilder` (sqlx 0.8 — example pattern is in existing `dao_impl_sqlite/src/absence.rs`).

**Patch `dao_impl_sqlite/src/lib.rs`:** add `pub mod cutover;` (alphabetical position).

After implementing, run `cargo sqlx prepare --workspace --all-targets` if the workspace uses offline mode (check via `ls .sqlx/` or `[package.metadata.sqlx]` in `dao_impl_sqlite/Cargo.toml`). If offline mode is active, run inside `nix-shell` per CLAUDE.local.md.
  </action>
  <acceptance_criteria>
    - File `dao_impl_sqlite/src/cutover.rs` exists; `grep -q 'impl CutoverDao for CutoverDaoImpl' dao_impl_sqlite/src/cutover.rs` exits 0
    - `grep -c 'async fn' dao_impl_sqlite/src/cutover.rs` returns >= 8
    - `grep -q 'pub mod cutover' dao_impl_sqlite/src/lib.rs` exits 0
    - `cargo build -p dao_impl_sqlite` exits 0
    - `cargo build --workspace` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p dao_impl_sqlite</automated>
  </verify>
  <done>
    All 8 DAO methods implemented in SQLx; workspace compiles; offline-mode `.sqlx` cache (if present) regenerated.
  </done>
</task>

<task type="auto">
  <name>Task 2: service_impl/src/cutover.rs Wave-1 — DI block + private cluster algorithm + run() migration-phase + Tx-rollback</name>
  <read_first>
    - service_impl/src/feature_flag.rs (gen_service_impl! pattern, Z. 31-67)
    - service_impl/src/absence.rs (Z. 45-62 DI block; Z. 418-510 derive_hours_for_range Per-Tag-Lookup; Z. 563-653 multi-Sub-Service-Tx)
    - service/src/cutover.rs (Plan 04-01 trait + DTOs)
    - service/src/permission.rs (HR_PRIVILEGE constant location)
    - dao/src/cutover.rs
    - dao/src/absence.rs (AbsenceDao::create signature)
    - service/src/employee_work_details.rs (EmployeeWorkDetailsService::find_by_sales_person_id signature)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Operation 1: Heuristik-Cluster-Algorithmus" (verbatim implementation guide)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-01 / D-Phase4-02 / D-Phase4-04 / D-Phase4-08 / C-Phase4-03 / C-Phase4-06"
  </read_first>
  <action>
**Create `service_impl/src/cutover.rs`** containing the `CutoverServiceImpl` Wave-1 surface. Wave 2 plans (04-05, 04-06) extend this file; structure the code so additions land in clearly-named private methods.

```rust
//! Phase 4 — Cutover orchestration (Wave 1: Migration phase only; Wave 2 adds gate + commit).

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use dao::cutover::{
    CutoverDao, LegacyExtraHoursRow, MigrationSourceRow, QuarantineRow,
};
use dao::extra_hours::ExtraHoursCategoryEntity;
use service::absence::AbsenceService;
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::{
    CutoverProfile, CutoverRunResult, CutoverService, CUTOVER_ADMIN_PRIVILEGE,
    QuarantineReason,
};
use service::employee_work_details::EmployeeWorkDetailsService;
use service::extra_hours::ExtraHoursService;
use service::feature_flag::FeatureFlagService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::ServiceError;
use shifty_macros::gen_service_impl;

gen_service_impl! {
    struct CutoverServiceImpl: service::cutover::CutoverService = CutoverServiceDeps {
        CutoverDao: dao::cutover::CutoverDao = cutover_dao,
        AbsenceDao: dao::absence::AbsenceDao = absence_dao,
        AbsenceService: service::absence::AbsenceService = absence_service,
        ExtraHoursService: service::extra_hours::ExtraHoursService = extra_hours_service,
        CarryoverRebuildService: service::carryover_rebuild::CarryoverRebuildService = carryover_rebuild_service,
        FeatureFlagService: service::feature_flag::FeatureFlagService = feature_flag_service,
        EmployeeWorkDetailsService: service::employee_work_details::EmployeeWorkDetailsService = employee_work_details_service,
        SalesPersonService: service::sales_person::SalesPersonService = sales_person_service,
        PermissionService: service::permission::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}

#[async_trait]
impl<Deps: CutoverServiceDeps> CutoverService for CutoverServiceImpl<Deps>
where
    Self: Send + Sync,
{
    type Context = Deps::Context;
    type Transaction = <Deps::TransactionDao as dao::TransactionDao>::Transaction;

    async fn run(
        &self,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverRunResult, ServiceError> {
        // 1. Permission check (Pattern 3 RESEARCH.md):
        let required = if dry_run { HR_PRIVILEGE } else { CUTOVER_ADMIN_PRIVILEGE };
        self.permission_service
            .check_permission(required, context.clone())
            .await?;

        // 2. Open Tx via TransactionDao::use_transaction(tx) (Pattern 1 RESEARCH.md):
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let run_id = uuid::Uuid::new_v4();
        let ran_at = time::OffsetDateTime::now_utc();
        let ran_at = time::PrimitiveDateTime::new(ran_at.date(), ran_at.time());

        // 3. Migration phase
        // LOCKED CONTRACT (per Plan 04-05 Task 1 dependency): the helper returns
        // BOTH the stats AND the list of merged extra_hours.id values. The id-list
        // is consumed verbatim by the Wave-2 commit_phase as input to
        // ExtraHoursService::soft_delete_bulk(...). DO NOT split into a separate
        // "_with_ids" method — Wave-1 already returns the tuple, Wave-2 just uses it.
        let (migration_stats, _migrated_ids) = self
            .migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone())
            .await?;

        // Wave-1 stop point: ALWAYS rollback (no gate, no commit).
        // Wave 2 plan 04-05 extends this with: gate -> branch -> commit/rollback,
        // and threads `_migrated_ids` into commit_phase.
        self.transaction_dao.rollback(tx).await?;

        Ok(CutoverRunResult {
            run_id,
            ran_at,
            dry_run,
            gate_passed: false,            // Wave 1 placeholder
            total_clusters: migration_stats.clusters as u32,
            migrated_clusters: migration_stats.clusters as u32,
            quarantined_rows: migration_stats.quarantined as u32,
            gate_drift_rows: 0,            // Wave 1 placeholder
            diff_report_path: None,        // Wave 1 placeholder
        })
    }

    async fn profile(...) -> Result<CutoverProfile, ServiceError> {
        // Wave 1 stub: implement in Plan 04-07 Task 1.
        Err(ServiceError::InternalError)
    }
}

pub(crate) struct MigrationStats { pub clusters: usize, pub quarantined: usize }

impl<Deps: CutoverServiceDeps> CutoverServiceImpl<Deps> {
    /// Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1 (verbatim).
    /// Pre-fetched contracts per C-Phase4-06.
    ///
    /// LOCKED RETURN-CONTRACT: returns `(MigrationStats, Arc<[Uuid]>)` where the
    /// `Arc<[Uuid]>` is the deduplicated, in-cluster-merge-order list of
    /// `extra_hours.id` values that were grouped into one or more
    /// `absence_period` rows (i.e., are eligible for soft-delete in Plan 04-05's
    /// commit_phase). Quarantined rows are NOT included in this list.
    ///
    /// Plan 04-05 commit_phase consumes this list verbatim as the `ids` argument
    /// to `ExtraHoursService::soft_delete_bulk(ids, "phase-4-cutover-migration",
    /// Authentication::Full, Some(tx.clone())).await`.
    pub(crate) async fn migrate_legacy_extra_hours_to_clusters(
        &self,
        cutover_run_id: Uuid,
        migrated_at: time::PrimitiveDateTime,
        tx: <<Deps as CutoverServiceDeps>::TransactionDao as dao::TransactionDao>::Transaction,
    ) -> Result<(MigrationStats, Arc<[Uuid]>), ServiceError> {
        // Step 1: read legacy
        let all_legacy = self.cutover_dao
            .find_legacy_extra_hours_not_yet_migrated(tx.clone()).await?;

        // Step 2: pre-fetch contracts per sp
        let mut work_details_by_sp: HashMap<Uuid, _> = HashMap::new();
        let sps: BTreeSet<Uuid> = all_legacy.iter().map(|r| r.sales_person_id).collect();
        for sp_id in sps {
            let wd = self.employee_work_details_service
                .find_by_sales_person_id(sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            work_details_by_sp.insert(sp_id, wd);
        }

        // Step 3: cluster greedy per (sp, category), with year-boundary breaks.
        // For each row:
        //   a) lookup active contract at row.date_time.date()
        //   b) if no contract -> quarantine "contract_not_active_at_date", break cluster
        //   c) if !contract.has_day_of_week(weekday) -> quarantine "contract_hours_zero_for_day", break
        //   d) expected = contract.hours_per_day(); if (amount - expected).abs() > 0.001:
        //         reason = if amount < expected { AmountBelowContractHours } else AmountAboveContractHours
        //         quarantine, break
        //   e) extends_cluster = current_cluster.last() matches sp+category AND
        //         is_consecutive_workday(last_date, day, contract) AND
        //         last_date.year() == day.year()  // Year-boundary break covers ISO-53 too
        //   f) if !extends && !empty -> close current_cluster (push to migrations), clear
        //   g) push row to current_cluster
        //
        // After loop: if !empty -> close final cluster.

        let mut current: Vec<&LegacyExtraHoursRow> = Vec::new();
        let mut migrations: Vec<(service::absence::AbsencePeriod, Vec<Uuid>)> = Vec::new();
        let mut quarantine: Vec<(Uuid, QuarantineReason, &LegacyExtraHoursRow)> = Vec::new();

        // ... (implement per RESEARCH.md Operation 1 verbatim, adapting to the
        //      Plan 04-01 trait signatures) ...

        // Step 4: persist + collect migrated_ids in cluster-iteration order.
        // CONTRACT: every extra_hours.id present in `migrations[*].1` is appended
        // to `migrated_ids` exactly once and IN THE SAME ORDER as it appears in
        // the source-id vec; this is the list returned to the caller and fed
        // directly into ExtraHoursService::soft_delete_bulk in Plan 04-05.
        let mut migrated_ids: Vec<Uuid> = Vec::with_capacity(all_legacy.len());

        for (period, source_ids) in &migrations {
            self.absence_dao.create(
                period.try_into()?,
                "phase-4-cutover-migration",
                tx.clone(),
            ).await?;
            for src_id in source_ids {
                self.cutover_dao.upsert_migration_source(&MigrationSourceRow {
                    extra_hours_id: *src_id,
                    absence_period_id: period.id,
                    cutover_run_id,
                    migrated_at,
                }, tx.clone()).await?;
                migrated_ids.push(*src_id);
            }
        }

        for (eh_id, reason, row) in &quarantine {
            self.cutover_dao.upsert_quarantine(&QuarantineRow {
                extra_hours_id: *eh_id,
                reason: reason.as_persisted_str().into(),
                sales_person_id: row.sales_person_id,
                category: row.category.clone(),
                date_time: row.date_time,
                amount: row.amount,
                cutover_run_id,
                migrated_at,
            }, tx.clone()).await?;
        }

        let stats = MigrationStats { clusters: migrations.len(), quarantined: quarantine.len() };
        let migrated_ids_arc: Arc<[Uuid]> = Arc::from(migrated_ids.into_boxed_slice());
        Ok((stats, migrated_ids_arc))
    }
}

// Helper: is `next_day` the next workday after `prev_day` according to the contract's workday mask?
fn is_consecutive_workday(
    prev_day: time::Date,
    next_day: time::Date,
    contract: &service::employee_work_details::EmployeeWorkDetails,
) -> bool {
    // Walk forward from prev_day+1 until the first day where contract.has_day_of_week(d.weekday())
    // is true. Return next_day == that day.
    let mut d = prev_day.next_day().expect("date overflow");
    while !contract.has_day_of_week(d.weekday()) {
        d = match d.next_day() { Some(d) => d, None => return false };
        // Safety: bail after 14 iterations to avoid infinite loops on pathological contracts
        if (d - prev_day).whole_days() > 14 { return false; }
    }
    d == next_day
}
```

**Patch `service_impl/src/lib.rs`:** add `pub mod cutover;` (alphabetical).

**Notes for the executor:**
- Verify `EmployeeWorkDetailsService::find_by_sales_person_id` exists with the expected signature; if it returns `Arc<[EmployeeWorkDetails]>`, use that. If the actual method name differs (e.g., `find_for_sales_person`), adapt the call but document the actual method in the SUMMARY.
- Verify `AbsencePeriod::try_into` to `dao::absence::AbsencePeriodEntity` exists. If not, construct the entity directly.
- The `unwrap_or` chain in cluster-detection assumes `from_date()` and `to_date()` are `Option`-returning methods on `EmployeeWorkDetails` (per absence.rs:478). Verify and adapt.
- `gen_service_impl!` macro path: import per existing `service_impl/src/feature_flag.rs:1` (likely `use shifty_macros::gen_service_impl;` or similar — verify).
  </action>
  <acceptance_criteria>
    - File `service_impl/src/cutover.rs` exists; `grep -q 'impl<Deps: CutoverServiceDeps> CutoverService for CutoverServiceImpl' service_impl/src/cutover.rs` exits 0
    - `grep -q 'gen_service_impl!' service_impl/src/cutover.rs` exits 0
    - `grep -q 'fn migrate_legacy_extra_hours_to_clusters' service_impl/src/cutover.rs` exits 0
    - **Return-type signature lock:** `grep -q 'Result<(MigrationStats, Arc<\[Uuid\]>), ServiceError>' service_impl/src/cutover.rs` exits 0 (the ONLY allowed return type for `migrate_legacy_extra_hours_to_clusters` — Plan 04-05 depends on this exact tuple shape)
    - `grep -q 'pub mod cutover' service_impl/src/lib.rs` exits 0
    - `grep -q 'transaction_dao.rollback' service_impl/src/cutover.rs` exits 0 (Wave 1 always rollbacks)
    - `cargo build -p service_impl` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service_impl</automated>
  </verify>
  <done>
    `CutoverServiceImpl` compiles; Wave-1 happy path runs heuristic + persists results + rolls back. Wave 2 will replace the rollback with branch logic and consume the returned `Arc<[Uuid]>` migrated-ids list verbatim.
  </done>
</task>

<task type="auto">
  <name>Task 3: Activate 8 service-mock tests in service_impl/src/test/cutover.rs (cluster-merge + 5 quarantine + idempotence + 2 forbidden)</name>
  <read_first>
    - service_impl/src/test/cutover.rs (Wave-0 stubs from Plan 04-01 Task 4)
    - service_impl/src/test/absence.rs (mockall pattern with multi-service mocks; Z. 1-200 for setup)
    - service_impl/src/test/feature_flag.rs (Authentication::Full bypass test pattern)
    - service_impl/src/cutover.rs (Task 2 above)
    - dao/src/cutover.rs (MockCutoverDao surface)
    - .planning/phases/04-migration-cutover/04-VALIDATION.md § "Per-Task Verification Map" (rows MIG-01 + Forbidden)
  </read_first>
  <action>
Replace the `#[ignore] + unimplemented!()` stubs in `service_impl/src/test/cutover.rs` for these 8 tests:

1. `cluster_merges_consecutive_workdays_with_exact_match` — fixture: 5 consecutive Mon-Fri Vacation rows with amount = 8.0; contract: 8h/day Mo-Fr. Expect: 1 absence_period (from_date=Mon, to_date=Fri), 5 mapping rows, 0 quarantine. **Tuple-contract check:** `let (stats, migrated_ids) = svc.migrate_legacy_extra_hours_to_clusters(run_id, ts, tx).await.unwrap();` — assert `stats.clusters == 1`, `stats.quarantined == 0`, AND `migrated_ids.len() == 5` (proves the locked `(MigrationStats, Arc<[Uuid]>)` return-shape from Task 2 acceptance criteria).
2. `quarantine_amount_below_contract` — fixture: 1 row Mon, amount = 4.0 (8h contract). Expect: 0 absence_period, 0 mapping, 1 quarantine with reason `amount_below_contract_hours`. Tuple-shape assert: `migrated_ids.len() == 0`.
3. `quarantine_amount_above_contract` — fixture: 1 row, amount = 10.0 (8h contract). Expect: reason `amount_above_contract_hours`. Tuple-shape: `migrated_ids.len() == 0`.
4. `quarantine_weekend_entry_workday_contract` — fixture: 1 Vacation row Saturday, amount = 8.0; contract: Mo-Fr. Expect: reason `contract_hours_zero_for_day`.
5. `quarantine_contract_not_active` — fixture: 1 Vacation row dated BEFORE the only EmployeeWorkDetails.from_date. Expect: reason `contract_not_active_at_date`.
6. `quarantine_iso_53_gap` — fixture: 1 Vacation row in week 53 of 2020 (Dec 28-31, 2020) + 1 row in Jan 1-3, 2021. Cluster MUST break at year boundary (e.g., year-equality check in is_consecutive_workday OR explicit `iso_53_week_gap` reason). Expect: 2 separate absence_period rows (one per year). If the algorithm uses year-boundary break, NO quarantine row (test asserts both rows clustered into separate AbsencePeriods); if it uses the explicit `iso_53_week_gap` reason, the test asserts the reason. Plan-Phase locks **year-boundary-break** as the simpler approach (CONTEXT.md `<specifics>` recommendation: "Cluster brichst auf bei jeglichem Year-Boundary"). If the executor finds an algorithmic edge case that requires the explicit reason, document it in the SUMMARY and flag for Wave-3 integration test.
7. `idempotent_rerun_skips_mapped` — first call: standard fixture, 1 cluster migrates. Second call: mock `find_legacy_extra_hours_not_yet_migrated` returns empty (already-mapped rows are filtered out by the SQL). Expect: 0 clusters, 0 quarantine, AND `migrated_ids.len() == 0` (tuple-shape preserved on no-op runs).
8. `run_forbidden_for_unprivileged_user` AND `run_forbidden_for_hr_only_when_committing` — use MockPermissionService to return `Err(ServiceError::Forbidden)` for the relevant privilege; assert the service returns `Err(ServiceError::Forbidden)` BEFORE any DAO is touched (use `MockCutoverDao` with `expect_*().times(0)`).

For each test:
- Remove `#[ignore = "wave-1-..."]`
- Replace `unimplemented!("...")` with the test body
- Construct the `CutoverServiceImpl` via `gen_service_impl!`-generated builder/struct with mocks
- Use existing test patterns from `service_impl/src/test/absence.rs` for mock setup boilerplate

**Important:** the AbsenceDao mock needs to be set up to accept `create` calls for the test 1 + test 7 happy paths. Use `expect_create().times(N).returning(|_, _, _| Ok(()))`.
  </action>
  <acceptance_criteria>
    - `grep -c '#\[ignore' service_impl/src/test/cutover.rs` returns at most 3 (Wave-2 gate-tolerance tests + any uncovered surface). The 8 tests above MUST NOT be `#[ignore]`d.
    - Tuple-shape verification present in test 1: `grep -q 'migrated_ids.len()' service_impl/src/test/cutover.rs` exits 0
    - `cargo test -p service_impl test::cutover::cluster_merges_consecutive_workdays_with_exact_match` exits 0
    - `cargo test -p service_impl test::cutover::quarantine_amount_below_contract` exits 0
    - `cargo test -p service_impl test::cutover::quarantine_amount_above_contract` exits 0
    - `cargo test -p service_impl test::cutover::quarantine_weekend_entry_workday_contract` exits 0
    - `cargo test -p service_impl test::cutover::quarantine_contract_not_active` exits 0
    - `cargo test -p service_impl test::cutover::quarantine_iso_53_gap` exits 0
    - `cargo test -p service_impl test::cutover::idempotent_rerun_skips_mapped` exits 0
    - `cargo test -p service_impl test::cutover::run_forbidden_for_unprivileged_user` exits 0
    - `cargo test -p service_impl test::cutover::run_forbidden_for_hr_only_when_committing` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::cutover</automated>
  </verify>
  <done>
    8 tests green; cluster algorithm + quarantine classification + idempotence + permission gate are all proven via mock-based unit tests; the `(MigrationStats, Arc<[Uuid]>)` return-shape is exercised end-to-end so Plan 04-05 has a known-good upstream contract.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Service → DAO | All DAO calls inside `migrate_legacy_extra_hours_to_clusters` MUST receive the cutover Tx (`tx.clone()`); no Sub-Service opens its own Tx. |
| `Authentication::Full` bypass for sub-service calls | `EmployeeWorkDetailsService` is called with `Authentication::Full` inside the cutover Tx — service-internal trust boundary. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-02-01 | Elevation of Privilege | `CutoverServiceImpl::run(dry_run=true)` callable by HR; commits forbidden | mitigate | Permission branch (HR for dry_run, CUTOVER_ADMIN for commit) verified by 2 forbidden tests in Task 3. Wave 2 commit logic preserves the same gate. |
| T-04-02-02 | Tampering | Cluster algorithm could mis-merge rows across sales_persons or categories | mitigate | Cluster-extension condition explicitly checks `last.sales_person_id == eh.sales_person_id && last.category == eh.category` (Operation 1, step e). Test 1 (single-sp single-category cluster) + cross-sp scenarios (Wave 3 integration tests) cover this. |
| T-04-02-03 | Tampering | DAO insert into `absence_period` bypasses AbsenceService Forward-Warning loop | accept | Per Anti-Pattern guidance in RESEARCH.md — Migration is a privileged operation, no booking conflicts to detect. The trade-off is documented. |
| T-04-02-04 | Information Disclosure | DAO returns full `LegacyExtraHoursRow` to service | accept | Service-internal struct; never crosses REST boundary unwrapped. |
| T-04-02-05 | Denial of Service | Heuristic algorithm O(N) over all legacy extra_hours could be slow on large datasets | mitigate | Pre-fetch optimization (C-Phase4-06) reduces EmployeeWorkDetails lookup to O(distinct_sp). Wave 3 plan 04-07 includes a smoke test with realistic fixture size. |
| T-04-02-06 | Repudiation | Migration writes to absence_period without audit trail beyond `update_process` | mitigate | `cutover_run_id` is persisted in mapping + quarantine tables; `update_process = 'phase-4-cutover-migration'` on the absence_period row provides cross-reference. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test -p service_impl test::cutover` reports >= 8 passed (and <= 3 ignored — gate-tolerance + any others)
- `cargo test -p dao_impl_sqlite` GREEN (compile-time queries against new schemas pass)
- All MIG-01 verification map rows that target `service_impl/src/test/cutover` have a corresponding implemented test
- Tx is ALWAYS rolled back at the end of `run` in Wave 1 (verified via `grep -q 'transaction_dao.rollback' service_impl/src/cutover.rs`)
- `migrate_legacy_extra_hours_to_clusters` returns `(MigrationStats, Arc<[Uuid]>)` — verified by both the `grep` acceptance criterion in Task 2 AND by the `migrated_ids.len()` assertions in Task 3 tests
- No code in `rest/`, `shifty_bin/main.rs` touched (Wave 1 is service-layer only — Wave 2 wires DI)
</verification>

<success_criteria>
1. All 8 service-level tests green: 1 cluster-merge + 5 quarantine + 1 idempotence + 2 forbidden.
2. Heuristik-Cluster-Algorithmus matches RESEARCH.md Operation 1 specification.
3. Pre-fetch optimization (C-Phase4-06) is in place — EmployeeWorkDetailsService is called once per distinct sp.
4. Year-boundary cluster break implemented (covers ISO-53 edge case automatically).
5. `migrate_legacy_extra_hours_to_clusters` return-shape locked as `(MigrationStats, Arc<[Uuid]>)` — Plan 04-05 commit_phase has a stable upstream contract.
6. Workspace compiles; downstream Plan 04-04 (extra_hours flag-gate) and Plan 04-05 (gate logic) can run in parallel after this lands.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-02-SUMMARY.md` listing:
- DAO impl summary: 8 SQL queries with verbatim WHERE clauses
- Service impl summary: cluster algorithm pseudocode + which `EmployeeWorkDetailsService` method was used (verify-and-document)
- Test outcomes: 8 passed, list any leftover `#[ignore]`d tests with reason
- Locked decisions: year-boundary cluster break (covers ISO-53), direct AbsenceDao::create (not AbsenceService::create), `(MigrationStats, Arc<[Uuid]>)` return-shape for migrate_legacy_extra_hours_to_clusters
- Hand-off note for Plan 04-05: "CutoverServiceImpl::migrate_legacy_extra_hours_to_clusters returns `(MigrationStats, Arc<[Uuid]>)`; the `Arc<[Uuid]>` is the input to soft_delete_bulk in commit_phase. Plan 04-05 calls `let (migration_stats, migrated_ids) = self.migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone()).await?;` directly — no method-name change, no separate `_with_ids` helper."
</output>
