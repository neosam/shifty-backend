//! Phase 4 — Cutover orchestration (Wave 1: Migration phase only).
//!
//! Wave 1 implements:
//!   - `gen_service_impl!` DI block (8 sub-services per Architectural Map row 1)
//!   - Permission-Branch in `run` (HR for dry_run; cutover_admin for commit)
//!   - Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1
//!   - Pre-fetch of EmployeeWorkDetails per sales_person (C-Phase4-06)
//!   - Persistence to `absence_period` (direct DAO insert per Anti-Pattern guidance)
//!     + `absence_period_migration_source` mapping rows
//!     + `absence_migration_quarantine` rows
//!   - **ALWAYS rollback** at the end of `run` — Wave 2 plans replace this with
//!     gate logic + commit/rollback branch.
//!
//! The private helper `migrate_legacy_extra_hours_to_clusters` returns a locked
//! tuple `(MigrationStats, Arc<[Uuid]>)`. The `Arc<[Uuid]>` is the verbatim list
//! of `extra_hours.id` values that ended up in a migrated cluster (NOT the
//! quarantined ones). Plan 04-05 commit_phase consumes this list verbatim as
//! the input to `ExtraHoursService::soft_delete_bulk`.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::gen_service_impl;
use dao::absence::{AbsenceCategoryEntity, AbsenceDao, AbsencePeriodEntity};
use dao::cutover::{CutoverDao, LegacyExtraHoursRow, MigrationSourceRow, QuarantineRow};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::TransactionDao;
use service::absence::AbsenceService;
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::{
    CutoverProfile, CutoverRunResult, CutoverService, QuarantineReason, CUTOVER_ADMIN_PRIVILEGE,
};
use service::employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService};
use service::extra_hours::ExtraHoursService;
use service::feature_flag::FeatureFlagService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::SalesPersonService;
use service::{PermissionService, ServiceError};

/// Process tag persisted in `extra_hours.update_process` (Wave 2 soft-delete) +
/// `absence_period.update_process` (Wave 1 migration insert). Wave-1 only uses
/// the absence_period side; Wave-2 04-05 reuses the same constant for soft-delete.
pub(crate) const CUTOVER_MIGRATION_PROCESS: &str = "phase-4-cutover-migration";

/// Minor numeric tolerance for the strict-match heuristic. Anything outside this
/// epsilon → quarantine. Must stay tighter than the cutover-gate tolerance
/// (Plan 04-05 uses 0.01) so a row that passes the heuristic cannot single-
/// handedly cross the gate threshold.
const CONTRACT_HOURS_EPSILON: f32 = 0.001;

gen_service_impl! {
    struct CutoverServiceImpl: service::cutover::CutoverService = CutoverServiceDeps {
        CutoverDao: CutoverDao<Transaction = Self::Transaction> = cutover_dao,
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        ExtraHoursService: ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
        CarryoverRebuildService: CarryoverRebuildService<Context = Self::Context, Transaction = Self::Transaction> = carryover_rebuild_service,
        FeatureFlagService: FeatureFlagService<Context = Self::Context, Transaction = Self::Transaction> = feature_flag_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MigrationStats {
    pub clusters: usize,
    pub quarantined: usize,
}

/// Internal cluster representation: extends until the (sp, category, day) chain
/// breaks. Stored only inside `migrate_legacy_extra_hours_to_clusters`.
struct InProgressCluster<'a> {
    rows: Vec<&'a LegacyExtraHoursRow>,
}

#[async_trait]
impl<Deps: CutoverServiceDeps> CutoverService for CutoverServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn run(
        &self,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverRunResult, ServiceError> {
        // 1. Permission gate per RESEARCH.md Pattern 3:
        //    - dry_run = HR may explore
        //    - commit  = cutover_admin only
        let required = if dry_run {
            HR_PRIVILEGE
        } else {
            CUTOVER_ADMIN_PRIVILEGE
        };
        self.permission_service
            .check_permission(required, context.clone())
            .await?;

        // 2. Open the cutover Tx (single Tx pattern, D-Phase4-14).
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let run_id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();
        let ran_at = time::PrimitiveDateTime::new(now.date(), now.time());

        // 3. Migration phase. The tuple shape is LOCKED per Plan 04-02 task 2
        //    acceptance criteria — Plan 04-05 consumes `migrated_ids` verbatim
        //    in `ExtraHoursService::soft_delete_bulk`.
        let (migration_stats, _migrated_ids) = self
            .migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone())
            .await?;

        // 4. Wave-1 stop point — ALWAYS rollback. Wave 2 (Plan 04-05) replaces
        //    this with gate-then-commit logic.
        self.transaction_dao.rollback(tx).await?;

        Ok(CutoverRunResult {
            run_id,
            ran_at,
            dry_run,
            gate_passed: false, // Wave 1 placeholder; Wave 2 (04-05) sets the real value.
            total_clusters: migration_stats.clusters as u32,
            migrated_clusters: migration_stats.clusters as u32,
            quarantined_rows: migration_stats.quarantined as u32,
            gate_drift_rows: 0, // Wave 1 placeholder.
            diff_report_path: None, // Wave 1 placeholder.
        })
    }

    async fn profile(
        &self,
        _context: Authentication<Self::Context>,
        _tx: Option<Self::Transaction>,
    ) -> Result<CutoverProfile, ServiceError> {
        // Implemented in Plan 04-07 Task 1.
        Err(ServiceError::InternalError)
    }
}

impl<Deps: CutoverServiceDeps> CutoverServiceImpl<Deps> {
    /// Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1 (verbatim).
    ///
    /// **Locked return contract** (Plan 04-05 commit_phase consumer):
    ///
    /// Returns `(MigrationStats, Arc<[Uuid]>)` where the `Arc<[Uuid]>` is the
    /// deduplicated, in-cluster-merge-order list of `extra_hours.id` values that
    /// were grouped into one or more `absence_period` rows (= eligible for
    /// soft-delete in Plan 04-05's commit_phase). Quarantined rows are NOT
    /// included in this list.
    ///
    /// Plan 04-05 commit_phase consumes this list verbatim as the `ids` argument
    /// to `ExtraHoursService::soft_delete_bulk(ids, "phase-4-cutover-migration",
    /// Authentication::Full, Some(tx.clone())).await`.
    pub(crate) async fn migrate_legacy_extra_hours_to_clusters(
        &self,
        cutover_run_id: Uuid,
        migrated_at: time::PrimitiveDateTime,
        tx: <Deps as CutoverServiceDeps>::Transaction,
    ) -> Result<(MigrationStats, Arc<[Uuid]>), ServiceError> {
        // Step 1: Read all not-yet-migrated legacy extra_hours rows.
        let all_legacy = self
            .cutover_dao
            .find_legacy_extra_hours_not_yet_migrated(tx.clone())
            .await?;

        // Step 2: Pre-fetch EmployeeWorkDetails per distinct sales_person_id
        // (C-Phase4-06). One service call per sp; HashMap lookup per row.
        let distinct_sps: BTreeSet<Uuid> =
            all_legacy.iter().map(|r| r.sales_person_id).collect();
        let mut work_details_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = HashMap::new();
        for sp_id in distinct_sps {
            let wd = self
                .employee_work_details_service
                .find_by_sales_person_id(sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            work_details_by_sp.insert(sp_id, wd);
        }

        // Step 3: Iterate sorted rows and greedily extend clusters per (sp,cat).
        let mut current = InProgressCluster { rows: Vec::new() };
        // Holds (DateRange-equivalent representation, source ids).
        let mut migrations: Vec<MigratedCluster> = Vec::new();
        let mut quarantine: Vec<QuarantinedRow> = Vec::new();

        for row in all_legacy.iter() {
            let day = row.date_time.date();
            let work_details = work_details_by_sp
                .get(&row.sales_person_id)
                .map(|arc| arc.as_ref())
                .unwrap_or(&[]);

            // (a) lookup active contract at row.day
            let active_contract = work_details.iter().find(|wh| {
                if wh.deleted.is_some() {
                    return false;
                }
                let from_date = match wh.from_date() {
                    Ok(d) => d.to_date(),
                    Err(_) => return false,
                };
                let to_date = match wh.to_date() {
                    Ok(d) => d.to_date(),
                    Err(_) => return false,
                };
                from_date <= day && day <= to_date
            });

            let Some(contract) = active_contract else {
                close_current_cluster(&mut current, &mut migrations);
                quarantine.push(QuarantinedRow {
                    row: (*row).clone(),
                    reason: QuarantineReason::ContractNotActiveAtDate,
                });
                continue;
            };

            // (b) workday check (D-Phase4-01)
            if !contract.has_day_of_week(day.weekday()) {
                close_current_cluster(&mut current, &mut migrations);
                quarantine.push(QuarantinedRow {
                    row: (*row).clone(),
                    reason: QuarantineReason::ContractHoursZeroForDay,
                });
                continue;
            }

            // (c) strict-match check (D-Phase4-02)
            let expected = contract.hours_per_day();
            if expected <= 0.0 {
                // Defensive: a contract with all-false workdays would have
                // hours_per_day() = expected_hours / 0 -> NaN/inf. Treat as
                // workday-zero quarantine.
                close_current_cluster(&mut current, &mut migrations);
                quarantine.push(QuarantinedRow {
                    row: (*row).clone(),
                    reason: QuarantineReason::ContractHoursZeroForDay,
                });
                continue;
            }
            if (row.amount - expected).abs() > CONTRACT_HOURS_EPSILON {
                let reason = if row.amount < expected {
                    QuarantineReason::AmountBelowContractHours
                } else {
                    QuarantineReason::AmountAboveContractHours
                };
                close_current_cluster(&mut current, &mut migrations);
                quarantine.push(QuarantinedRow {
                    row: (*row).clone(),
                    reason,
                });
                continue;
            }

            // (d) extends current cluster?
            let extends = current.rows.last().map_or(false, |last| {
                last.sales_person_id == row.sales_person_id
                    && last.category == row.category
                    && last.date_time.date().year() == day.year()
                    && is_consecutive_workday(last.date_time.date(), day, contract)
            });
            if !extends && !current.rows.is_empty() {
                close_current_cluster(&mut current, &mut migrations);
            }
            current.rows.push(row);
        }
        // Final close
        if !current.rows.is_empty() {
            close_current_cluster(&mut current, &mut migrations);
        }

        // Step 4: Persist. Order:
        //   1. INSERT absence_period (one per cluster) via direct DAO call
        //      (per Anti-Pattern guidance — bypass AbsenceService Forward-
        //       Warning loop; Migration is privileged).
        //   2. INSERT absence_period_migration_source (one per source row).
        //   3. UPSERT absence_migration_quarantine (one per quarantined row).
        let mut migrated_ids: Vec<Uuid> = Vec::with_capacity(all_legacy.len());

        for cluster in migrations.iter() {
            // Build the AbsencePeriodEntity directly. id == logical_id (first
            // version). version is a fresh uuid.
            let entity = AbsencePeriodEntity {
                id: cluster.absence_period_id,
                logical_id: cluster.absence_period_id,
                sales_person_id: cluster.sales_person_id,
                category: cluster.category.clone(),
                from_date: cluster.from_date,
                to_date: cluster.to_date,
                description: Arc::from(""),
                created: migrated_at,
                deleted: None,
                version: Uuid::new_v4(),
            };
            self.absence_dao
                .create(&entity, CUTOVER_MIGRATION_PROCESS, tx.clone())
                .await?;

            for src_id in cluster.source_ids.iter() {
                self.cutover_dao
                    .upsert_migration_source(
                        &MigrationSourceRow {
                            extra_hours_id: *src_id,
                            absence_period_id: cluster.absence_period_id,
                            cutover_run_id,
                            migrated_at,
                        },
                        tx.clone(),
                    )
                    .await?;
                migrated_ids.push(*src_id);
            }
        }

        for q in quarantine.iter() {
            self.cutover_dao
                .upsert_quarantine(
                    &QuarantineRow {
                        extra_hours_id: q.row.id,
                        reason: Arc::from(q.reason.as_persisted_str()),
                        sales_person_id: q.row.sales_person_id,
                        category: q.row.category.clone(),
                        date_time: q.row.date_time,
                        amount: q.row.amount,
                        cutover_run_id,
                        migrated_at,
                    },
                    tx.clone(),
                )
                .await?;
        }

        let stats = MigrationStats {
            clusters: migrations.len(),
            quarantined: quarantine.len(),
        };
        let migrated_ids_arc: Arc<[Uuid]> = Arc::from(migrated_ids.into_boxed_slice());
        Ok((stats, migrated_ids_arc))
    }
}

/// A finalized cluster ready for INSERT into `absence_period`.
struct MigratedCluster {
    absence_period_id: Uuid,
    sales_person_id: Uuid,
    category: AbsenceCategoryEntity,
    from_date: time::Date,
    to_date: time::Date,
    source_ids: Vec<Uuid>,
}

struct QuarantinedRow {
    row: LegacyExtraHoursRow,
    reason: QuarantineReason,
}

/// Close the current cluster and push it onto `migrations` if non-empty.
/// `current.rows` is cleared regardless.
fn close_current_cluster(
    current: &mut InProgressCluster<'_>,
    migrations: &mut Vec<MigratedCluster>,
) {
    if current.rows.is_empty() {
        return;
    }
    // Invariant by construction: rows share sales_person_id + category and are
    // sorted by date_time ASC; first row is the from_date, last is to_date.
    let first = current.rows.first().expect("non-empty by guard");
    let last = current.rows.last().expect("non-empty by guard");
    let from_date = first.date_time.date();
    let to_date = last.date_time.date();
    let source_ids: Vec<Uuid> = current.rows.iter().map(|r| r.id).collect();

    migrations.push(MigratedCluster {
        absence_period_id: Uuid::new_v4(),
        sales_person_id: first.sales_person_id,
        category: extra_hours_category_to_absence(&first.category),
        from_date,
        to_date,
        source_ids,
    });
    current.rows.clear();
}

/// Map the extra_hours legacy category enum to the absence_period category enum.
/// Caller MUST ensure the input is one of {Vacation, SickLeave, UnpaidLeave};
/// other variants panic (the cluster algorithm filters via SQL category-IN
/// upstream, so no runtime panic is reachable for well-formed data).
fn extra_hours_category_to_absence(c: &ExtraHoursCategoryEntity) -> AbsenceCategoryEntity {
    match c {
        ExtraHoursCategoryEntity::Vacation => AbsenceCategoryEntity::Vacation,
        ExtraHoursCategoryEntity::SickLeave => AbsenceCategoryEntity::SickLeave,
        ExtraHoursCategoryEntity::UnpaidLeave => AbsenceCategoryEntity::UnpaidLeave,
        other => panic!(
            "extra_hours_category_to_absence called with non-legacy category: {:?}",
            other
        ),
    }
}

/// Walks forward from `prev_day + 1` until it hits the next contract-workday;
/// returns `next_day == that-day`. Caller MUST ensure `prev_day < next_day`.
/// Bails (returns false) after 14 iterations to avoid a pathological all-false
/// workday-mask infinite loop (defensive — strict-match check above already
/// rejects amount > 0 for non-workdays so this should never fire).
fn is_consecutive_workday(
    prev_day: time::Date,
    next_day: time::Date,
    contract: &EmployeeWorkDetails,
) -> bool {
    if next_day <= prev_day {
        return false;
    }
    let mut d = match prev_day.next_day() {
        Some(d) => d,
        None => return false,
    };
    let mut steps = 0u32;
    while !contract.has_day_of_week(d.weekday()) {
        d = match d.next_day() {
            Some(d) => d,
            None => return false,
        };
        steps += 1;
        if steps > 14 {
            return false;
        }
    }
    d == next_day
}
