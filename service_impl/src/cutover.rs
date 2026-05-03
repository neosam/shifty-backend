//! Phase 4 — Cutover orchestration (Wave 1 + Wave 2).
//!
//! Wave 1 implemented (Plan 04-02):
//!   - `gen_service_impl!` DI block (10 sub-services per Architectural Map row 1)
//!   - Permission-Branch in `run` (HR for dry_run; cutover_admin for commit)
//!   - Heuristik-Cluster-Algorithmus per RESEARCH.md Operation 1
//!   - Pre-fetch of EmployeeWorkDetails per sales_person (C-Phase4-06)
//!   - Persistence to `absence_period` (direct DAO insert per Anti-Pattern guidance)
//!     + `absence_period_migration_source` mapping rows
//!     + `absence_migration_quarantine` rows
//!
//! Wave 2 (this plan, 04-05) extends `run` with:
//!   - `compute_gate`: per (sp, kategorie, jahr) compares `legacy_sum`
//!     (via `CutoverDao::sum_legacy_extra_hours`) against `derived_sum`
//!     (via `AbsenceService::derive_hours_for_range`). Tolerance < 0.01h
//!     absolute (D-Phase4-05). Produces a JSON diff-report file at
//!     `.planning/migration-backup/cutover-gate-{unix_timestamp}.json`
//!     (D-Phase4-06) plus `tracing::error!` per drift row.
//!   - `commit_phase`: backup carryover (D-Phase4-13) → rebuild carryover for
//!     scope (D-Phase4-12) → soft-delete migrated extra_hours (D-Phase4-10) →
//!     flip feature flag (D-Phase4-09). All in the same atomic Tx
//!     (D-Phase4-14).
//!   - Branch logic: dry_run OR !gate_passed → rollback;
//!     !dry_run AND gate_passed → commit_phase + commit Tx.
//!
//! The private helper `migrate_legacy_extra_hours_to_clusters` returns a locked
//! tuple `(MigrationStats, Arc<[Uuid]>)`. The `Arc<[Uuid]>` is the verbatim list
//! of `extra_hours.id` values that ended up in a migrated cluster (NOT the
//! quarantined ones). `commit_phase` consumes this list verbatim as the input
//! to `ExtraHoursService::soft_delete_bulk`.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::gen_service_impl;
use dao::absence::{AbsenceCategoryEntity, AbsenceDao, AbsencePeriodEntity};
use dao::cutover::{CutoverDao, LegacyExtraHoursRow, MigrationSourceRow, QuarantineRow};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::TransactionDao;
use service::absence::{AbsenceCategory, AbsenceService};
use service::carryover_rebuild::CarryoverRebuildService;
use service::cutover::{
    CutoverProfile, CutoverProfileBucket, CutoverRunResult, CutoverService, DriftRow, GateResult,
    QuarantineReason, CUTOVER_ADMIN_PRIVILEGE,
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
        //    acceptance criteria — Wave 2 commit_phase consumes `migrated_ids`
        //    verbatim in `ExtraHoursService::soft_delete_bulk`.
        let (migration_stats, migrated_ids) = self
            .migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone())
            .await?;

        // 4. Gate phase (Wave 2 / Plan 04-05). Always runs — even on dry_run —
        //    because it is the single source of truth for `gate_passed` and
        //    produces the diff-report file path returned in CutoverRunResult.
        let gate = self.compute_gate(run_id, ran_at, dry_run, tx.clone()).await?;

        // 5. Branch on dry_run + gate result (D-Phase4-08 + D-Phase4-14).
        if dry_run || !gate.passed {
            self.transaction_dao.rollback(tx).await?;
            return Ok(CutoverRunResult {
                run_id,
                ran_at,
                dry_run,
                gate_passed: gate.passed,
                total_clusters: migration_stats.clusters as u32,
                // No commit — `migrated_clusters` reflects the in-Tx work that
                // was rolled back. Conservatively report 0 so callers can
                // distinguish committed from rolled-back runs (REST handler in
                // Plan 04-06 surfaces this in the response body).
                migrated_clusters: 0,
                quarantined_rows: migration_stats.quarantined as u32,
                gate_drift_rows: gate.drift_rows.len() as u32,
                diff_report_path: Some(gate.diff_report_path.clone()),
            });
        }

        // 6. Commit phase (Wave 2). Only reached when !dry_run AND gate.passed.
        //    Pass the Plan-04-02 `migrated_ids` straight into the helper.
        self.commit_phase(run_id, ran_at, &gate, migrated_ids, tx.clone())
            .await?;

        // 7. Atomic flip — single commit point per D-Phase4-14.
        self.transaction_dao.commit(tx).await?;

        Ok(CutoverRunResult {
            run_id,
            ran_at,
            dry_run,
            gate_passed: true,
            total_clusters: migration_stats.clusters as u32,
            migrated_clusters: migration_stats.clusters as u32,
            quarantined_rows: migration_stats.quarantined as u32,
            gate_drift_rows: 0,
            diff_report_path: Some(gate.diff_report_path),
        })
    }

    /// Production-Data-Profile per SC-1 + C-Phase4-05.
    ///
    /// Reads ALL legacy `extra_hours` rows (Vacation/SickLeave/UnpaidLeave —
    /// regardless of mapping state) and bins them per (sales_person, category,
    /// year). Per bucket we compute:
    ///
    ///   * `row_count`            — number of legacy rows
    ///   * `sum_amount`           — Σ row.amount
    ///   * `fractional_count`     — rows where `|amount − contract_hours_per_day| > 0.001`
    ///   * `weekend_on_workday_only_contract_count` — rows landing on a NON-workday
    ///     of the active contract (with non-zero amount)
    ///   * `iso_53_indicator`    — `true` if any row falls into ISO week 53
    ///
    /// The aggregation needs the per-day contract lookup, so we pre-fetch
    /// `EmployeeWorkDetails` per sales_person up front (mirrors the cluster
    /// algorithm in `migrate_legacy_extra_hours_to_clusters`).
    ///
    /// Permission: HR (matches `gate-dry-run`; profile is non-destructive).
    /// Uses a fresh Tx that is rolled back at the end — profile is read-only
    /// from a state perspective; the only side-effect is writing the JSON file
    /// at `.planning/migration-backup/profile-{unix_timestamp}.json` (Unix-
    /// timestamp filename per Assumption A7 — Linux + Windows safe).
    async fn profile(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CutoverProfile, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let run_id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();
        let generated_at = time::PrimitiveDateTime::new(now.date(), now.time());

        let all_legacy = self
            .cutover_dao
            .find_all_legacy_extra_hours(tx.clone())
            .await?;

        // Pre-fetch contracts + sales-person names per distinct sp_id (mirror
        // cluster-algorithm pre-fetch from Plan 04-02 Task 2 — one service call
        // per sp; HashMap lookup per row).
        let distinct_sps: BTreeSet<Uuid> =
            all_legacy.iter().map(|r| r.sales_person_id).collect();
        let mut work_details_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = HashMap::new();
        let mut sp_names: HashMap<Uuid, Arc<str>> = HashMap::new();
        for sp_id in distinct_sps {
            let wd = self
                .employee_work_details_service
                .find_by_sales_person_id(sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            work_details_by_sp.insert(sp_id, wd);
            let sp = self
                .sales_person_service
                .get(sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            sp_names.insert(sp_id, sp.name.clone());
        }

        // Bucket: (sp_id, category_discriminator, year) -> aggregate counters.
        // We use a HashMap with a `u8` discriminator for the category so the
        // key implements `Hash`. The output Vec is sorted at the end for a
        // deterministic JSON diff across runs.
        let mut buckets: HashMap<(Uuid, u8, u32), CutoverProfileBucket> = HashMap::new();

        for row in all_legacy.iter() {
            // Skip non-legacy categories defensively (DAO already filters, but
            // the profile() call is read-only and stable-against-row-types).
            let (svc_cat, cat_disc): (AbsenceCategory, u8) = match &row.category {
                ExtraHoursCategoryEntity::Vacation => (AbsenceCategory::Vacation, 0),
                ExtraHoursCategoryEntity::SickLeave => (AbsenceCategory::SickLeave, 1),
                ExtraHoursCategoryEntity::UnpaidLeave => (AbsenceCategory::UnpaidLeave, 2),
                _ => continue,
            };
            let year = row.date_time.date().year() as u32;
            let day = row.date_time.date();
            let key = (row.sales_person_id, cat_disc, year);

            let work_details = work_details_by_sp
                .get(&row.sales_person_id)
                .map(|arc| arc.as_ref())
                .unwrap_or(&[]);

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
            let contract_hours = active_contract.map(|c| c.hours_per_day()).unwrap_or(0.0);
            let is_workday = active_contract
                .map(|c| c.has_day_of_week(day.weekday()))
                .unwrap_or(false);
            let is_weekend_on_workday_only = !is_workday && row.amount > 0.0;
            let is_fractional = (row.amount - contract_hours).abs() > 0.001;
            let is_iso_53 = day.iso_week() == 53;

            let entry = buckets.entry(key).or_insert_with(|| CutoverProfileBucket {
                sales_person_id: row.sales_person_id,
                sales_person_name: sp_names
                    .get(&row.sales_person_id)
                    .cloned()
                    .unwrap_or_else(|| Arc::from("")),
                category: svc_cat,
                year,
                row_count: 0,
                sum_amount: 0.0,
                fractional_count: 0,
                weekend_on_workday_only_contract_count: 0,
                iso_53_indicator: false,
            });
            entry.row_count += 1;
            entry.sum_amount += row.amount;
            if is_fractional {
                entry.fractional_count += 1;
            }
            if is_weekend_on_workday_only {
                entry.weekend_on_workday_only_contract_count += 1;
            }
            if is_iso_53 {
                entry.iso_53_indicator = true;
            }
        }

        // Persist JSON file (Unix timestamp for filesystem-safe filename —
        // Assumption A7 from 04-RESEARCH.md). Use `_nanos` for collision-safety
        // when tests run back-to-back (mirrors Plan 04-05 compute_gate path).
        std::fs::create_dir_all(".planning/migration-backup")
            .map_err(|_| ServiceError::InternalError)?;
        let unix_ts_nanos = generated_at.assume_utc().unix_timestamp_nanos();
        let profile_path = format!(
            ".planning/migration-backup/profile-{}.json",
            unix_ts_nanos
        );

        let generated_at_iso = generated_at
            .assume_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default();

        // Sort buckets for a stable JSON diff: (sp_id, category, year).
        let mut sorted_buckets: Vec<CutoverProfileBucket> = buckets.into_values().collect();
        sorted_buckets.sort_by(|a, b| {
            a.sales_person_id
                .cmp(&b.sales_person_id)
                .then_with(|| {
                    let a_cat = absence_category_order_key(&a.category);
                    let b_cat = absence_category_order_key(&b.category);
                    a_cat.cmp(&b_cat)
                })
                .then_with(|| a.year.cmp(&b.year))
        });

        let body = serde_json::json!({
            "run_id": run_id.to_string(),
            "generated_at": generated_at_iso,
            "buckets": sorted_buckets.iter().map(|b| serde_json::json!({
                "sales_person_id": b.sales_person_id.to_string(),
                "sales_person_name": b.sales_person_name.as_ref(),
                "category": format!("{:?}", b.category),
                "year": b.year,
                "row_count": b.row_count,
                "sum_amount": b.sum_amount,
                "fractional_count": b.fractional_count,
                "fractional_quote": if b.row_count > 0 {
                    b.fractional_count as f32 / b.row_count as f32
                } else { 0.0 },
                "weekend_on_workday_only_contract_count": b.weekend_on_workday_only_contract_count,
                "iso_53_indicator": b.iso_53_indicator,
            })).collect::<Vec<_>>(),
        });
        std::fs::write(
            &profile_path,
            serde_json::to_string_pretty(&body).map_err(|_| ServiceError::InternalError)?,
        )
        .map_err(|_| ServiceError::InternalError)?;

        // profile() is read-only — roll back the Tx so any side-effect inside
        // the sub-service calls (none expected, but defense-in-depth) does not
        // bleed into the database.
        self.transaction_dao.rollback(tx).await?;

        let buckets_arc: Arc<[CutoverProfileBucket]> = Arc::from(sorted_buckets);
        Ok(CutoverProfile {
            run_id,
            generated_at,
            buckets: buckets_arc,
            profile_path: Arc::from(profile_path.as_str()),
        })
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
            let extends = current.rows.last().is_some_and(|last| {
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

    /// Cutover gate per D-Phase4-05 + D-Phase4-06.
    ///
    /// Walks the global `(sales_person_id, year)` scope set produced by
    /// `CutoverDao::find_legacy_scope_set` and, per (sp, kategorie, jahr),
    /// compares `legacy_sum` (DAO sum over `extra_hours.amount` for the three
    /// legacy categories) against `derived_sum` (sum over the per-day output
    /// of `AbsenceService::derive_hours_for_range`, filtered by category).
    ///
    /// Tolerance: 0.01h absolute. Anything strictly above produces a
    /// `DriftRow` and a `tracing::error!` log line. The full gate result is
    /// also persisted as a JSON file at
    /// `.planning/migration-backup/cutover-gate-{unix_timestamp}.json`
    /// (Unix-timestamp filename per Assumption A7 in 04-RESEARCH.md — Linux +
    /// Windows safe; ISO with `:` is not Windows-safe).
    ///
    /// Reuses `derive_hours_for_range` verbatim per D-Phase2-08-A — no
    /// re-implementation of the conflict-resolution logic, so the gate
    /// measures EXACTLY what the post-cutover live read returns.
    pub(crate) async fn compute_gate(
        &self,
        cutover_run_id: Uuid,
        ran_at: time::PrimitiveDateTime,
        dry_run: bool,
        tx: <Deps as CutoverServiceDeps>::Transaction,
    ) -> Result<GateResult, ServiceError> {
        const DRIFT_THRESHOLD: f32 = 0.01;

        let scope = self.cutover_dao.find_legacy_scope_set(tx.clone()).await?;
        let mut drift_rows: Vec<DriftRow> = Vec::new();

        // The three legacy categories — fixed order so the diff-report is
        // stable across runs.
        let categories: [(ExtraHoursCategoryEntity, AbsenceCategory); 3] = [
            (ExtraHoursCategoryEntity::Vacation, AbsenceCategory::Vacation),
            (ExtraHoursCategoryEntity::SickLeave, AbsenceCategory::SickLeave),
            (
                ExtraHoursCategoryEntity::UnpaidLeave,
                AbsenceCategory::UnpaidLeave,
            ),
        ];

        for &(sp_id, year) in scope.iter() {
            // One sales-person lookup per (sp, year) tuple — used for the
            // DriftRow.sales_person_name field. Skipped if no drift rows are
            // emitted for this (sp, year), but cheap to fetch up front.
            let sp = self
                .sales_person_service
                .get(sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            let sp_name: Arc<str> = sp.name.clone();

            // One derive_hours_for_range call per (sp, year), then partition
            // by category — saves N×3 calls per scope tuple.
            let year_i32 = year as i32;
            let year_start = time::Date::from_calendar_date(year_i32, time::Month::January, 1)
                .map_err(|_| ServiceError::InternalError)?;
            let year_end = time::Date::from_calendar_date(year_i32, time::Month::December, 31)
                .map_err(|_| ServiceError::InternalError)?;
            let derived = self
                .absence_service
                .derive_hours_for_range(
                    year_start,
                    year_end,
                    sp_id,
                    Authentication::Full,
                    Some(tx.clone()),
                )
                .await?;

            for (category_dao, category_svc) in categories.iter() {
                let legacy_sum = self
                    .cutover_dao
                    .sum_legacy_extra_hours(sp_id, category_dao, year, tx.clone())
                    .await?;

                let derived_sum: f32 = derived
                    .values()
                    .filter(|r| r.category == *category_svc)
                    .map(|r| r.hours)
                    .sum();

                let drift = (legacy_sum - derived_sum).abs();
                if drift > DRIFT_THRESHOLD {
                    let (quarantined_count, reasons) = self
                        .cutover_dao
                        .count_quarantine_for_drift_row(
                            sp_id,
                            category_dao,
                            year,
                            cutover_run_id,
                            tx.clone(),
                        )
                        .await?;
                    tracing::error!(
                        "[cutover-gate] drift sp={} cat={:?} year={}: legacy={} derived={} drift={}",
                        sp_id,
                        category_svc,
                        year,
                        legacy_sum,
                        derived_sum,
                        drift
                    );
                    drift_rows.push(DriftRow {
                        sales_person_id: sp_id,
                        sales_person_name: sp_name.clone(),
                        category: *category_svc,
                        year,
                        legacy_sum,
                        derived_sum,
                        drift,
                        quarantined_extra_hours_count: quarantined_count,
                        quarantine_reasons: reasons,
                    });
                }
            }
        }

        // Diff-report file. Use the run-timestamp's Unix epoch in nanoseconds
        // as the filename suffix (Assumption A7 mitigation — colon-free,
        // monotonic, Linux + Windows filesystem-safe). Nanosecond precision
        // also prevents collisions for back-to-back runs in tests / rapid
        // operator retries.
        std::fs::create_dir_all(".planning/migration-backup")
            .map_err(|_| ServiceError::InternalError)?;
        let unix_ts_nanos = ran_at.assume_utc().unix_timestamp_nanos();
        let report_path = format!(
            ".planning/migration-backup/cutover-gate-{}.json",
            unix_ts_nanos
        );

        let run_at_iso = ran_at
            .assume_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_else(|_| String::new());

        let report_json = serde_json::json!({
            "gate_run_id": cutover_run_id.to_string(),
            "run_at": run_at_iso,
            "dry_run": dry_run,
            "drift_threshold": DRIFT_THRESHOLD,
            "total_drift_rows": drift_rows.len(),
            "drift": drift_rows.iter().map(|r| serde_json::json!({
                "sales_person_id": r.sales_person_id.to_string(),
                "sales_person_name": r.sales_person_name.as_ref(),
                "category": format!("{:?}", r.category),
                "year": r.year,
                "legacy_sum": r.legacy_sum,
                "derived_sum": r.derived_sum,
                "drift": r.drift,
                "quarantined_extra_hours_count": r.quarantined_extra_hours_count,
                "quarantine_reasons": r.quarantine_reasons.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
            })).collect::<Vec<_>>(),
            "passed": drift_rows.is_empty(),
        });

        std::fs::write(
            &report_path,
            serde_json::to_string_pretty(&report_json)
                .map_err(|_| ServiceError::InternalError)?,
        )
        .map_err(|_| ServiceError::InternalError)?;

        let passed = drift_rows.is_empty();
        Ok(GateResult {
            passed,
            drift_rows: Arc::from(drift_rows.into_boxed_slice()),
            diff_report_path: Arc::from(report_path.as_str()),
            scope_set: scope,
        })
    }

    /// Commit-phase orchestration per D-Phase4-09..14. Only called from
    /// `run()` when `!dry_run` AND `gate.passed`. All sub-service calls share
    /// the cutover Tx and rely on the outer `transaction_dao.commit(tx)` call
    /// in `run()` for atomicity.
    ///
    /// Step a (D-Phase4-13): backup `employee_yearly_carryover` for the gate
    /// scope set BEFORE we update it. Single multi-row INSERT-INTO-SELECT in
    /// the DAO.
    ///
    /// Step b (D-Phase4-12): rebuild `employee_yearly_carryover` per (sp,
    /// year) tuple — Carryover-Refresh-Scope = gate scope set.
    ///
    /// Step c (D-Phase4-10): soft-delete the legacy `extra_hours` rows that
    /// were merged into `absence_period` clusters. The id list comes verbatim
    /// from the locked Plan-04-02 contract — no re-fetch, no re-derivation.
    /// Quarantined rows are NOT in this list (they remain live for HR
    /// triage).
    ///
    /// Step d (D-Phase4-09): atomic feature-flag flip. After this point, any
    /// concurrent reader (e.g. `ReportingService`) sees the post-cutover
    /// world; the flip is observable to other transactions only after the
    /// outer `commit(tx)` succeeds.
    pub(crate) async fn commit_phase(
        &self,
        cutover_run_id: Uuid,
        ran_at: time::PrimitiveDateTime,
        gate: &GateResult,
        migrated_ids: Arc<[Uuid]>,
        tx: <Deps as CutoverServiceDeps>::Transaction,
    ) -> Result<(), ServiceError> {
        // Step a — pre-cutover carryover backup (D-Phase4-13).
        self.cutover_dao
            .backup_carryover_for_scope(cutover_run_id, ran_at, &gate.scope_set, tx.clone())
            .await?;

        // Step b — rebuild carryover per scope tuple (D-Phase4-12).
        for &(sp_id, year) in gate.scope_set.iter() {
            self.carryover_rebuild_service
                .rebuild_for_year(sp_id, year, Authentication::Full, Some(tx.clone()))
                .await?;
        }

        // Step c — soft-delete migrated extra_hours rows (D-Phase4-10).
        // `migrated_ids` is the verbatim Arc<[Uuid]> from
        // migrate_legacy_extra_hours_to_clusters in this same `run()`.
        self.extra_hours_service
            .soft_delete_bulk(
                migrated_ids,
                CUTOVER_MIGRATION_PROCESS,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        // Step d — atomic feature-flag flip (D-Phase4-09).
        self.feature_flag_service
            .set(
                "absence_range_source_active",
                true,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        Ok(())
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

/// Stable ordinal key for `AbsenceCategory`. Used by `profile()` to produce
/// a deterministic bucket-list ordering in the JSON output (the enum itself
/// does not implement `Ord`).
fn absence_category_order_key(c: &AbsenceCategory) -> u8 {
    match c {
        AbsenceCategory::Vacation => 0,
        AbsenceCategory::SickLeave => 1,
        AbsenceCategory::UnpaidLeave => 2,
    }
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
