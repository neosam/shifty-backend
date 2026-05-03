//! Phase 4 — SQLite implementation of `dao::cutover::CutoverDao`.
//!
//! Reads legacy `extra_hours` rows globally for the three legacy categories
//! (Vacation, SickLeave, UnpaidLeave) and writes to three Phase-4 audit tables
//! (`absence_migration_quarantine`, `absence_period_migration_source`,
//! `employee_yearly_carryover_pre_cutover_backup`). The carryover backup is
//! INSERT-INTO-SELECT against the live `employee_yearly_carryover` row — single
//! statement built via `QueryBuilder::push_tuples` for the (sp,year) IN-clause.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::QueryBuilder;
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

use dao::cutover::{CutoverDao, LegacyExtraHoursRow, MigrationSourceRow, QuarantineRow};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::DaoError;

use crate::{ResultDbErrorExt, TransactionImpl};

/// Maps `ExtraHoursCategoryEntity` (3 legacy categories only) to the persisted
/// string column. Out-of-scope categories panic — callers must filter.
fn legacy_category_to_str(c: &ExtraHoursCategoryEntity) -> &'static str {
    match c {
        ExtraHoursCategoryEntity::Vacation => "Vacation",
        ExtraHoursCategoryEntity::SickLeave => "SickLeave",
        ExtraHoursCategoryEntity::UnpaidLeave => "UnpaidLeave",
        other => panic!(
            "legacy_category_to_str called with non-legacy category: {:?}",
            other
        ),
    }
}

/// Inverse of `legacy_category_to_str` — only the 3 legacy categories are
/// recognized; anything else returns an EnumValueNotFound error.
fn legacy_category_from_str(s: &str) -> Result<ExtraHoursCategoryEntity, DaoError> {
    match s {
        "Vacation" => Ok(ExtraHoursCategoryEntity::Vacation),
        "SickLeave" => Ok(ExtraHoursCategoryEntity::SickLeave),
        "UnpaidLeave" => Ok(ExtraHoursCategoryEntity::UnpaidLeave),
        other => Err(DaoError::EnumValueNotFound(other.into())),
    }
}

pub struct CutoverDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl CutoverDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl CutoverDao for CutoverDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_legacy_extra_hours_not_yet_migrated(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError> {
        // The plain rows we need (id, sales_person_id, category, date_time, amount)
        // for the 3 legacy categories that have not yet been mapped. Sorted by
        // (sales_person_id, category, date_time) ASC so the cluster algorithm
        // can iterate sequentially.
        let rows = sqlx::query!(
            r#"SELECT eh.id            AS "id!: Vec<u8>",
                      eh.sales_person_id AS "sales_person_id!: Vec<u8>",
                      eh.category      AS "category!: String",
                      eh.date_time     AS "date_time!: String",
                      eh.amount        AS "amount!: f64"
               FROM extra_hours eh
               WHERE eh.deleted IS NULL
                 AND eh.category IN ('Vacation', 'SickLeave', 'UnpaidLeave')
                 AND eh.id NOT IN (
                     SELECT extra_hours_id FROM absence_period_migration_source
                 )
               ORDER BY eh.sales_person_id, eh.category, eh.date_time"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let mut out: Vec<LegacyExtraHoursRow> = Vec::with_capacity(rows.len());
        for r in rows.iter() {
            out.push(LegacyExtraHoursRow {
                id: Uuid::from_slice(&r.id)?,
                sales_person_id: Uuid::from_slice(&r.sales_person_id)?,
                category: legacy_category_from_str(&r.category)?,
                date_time: PrimitiveDateTime::parse(&r.date_time, &Iso8601::DATE_TIME)?,
                amount: r.amount as f32,
            });
        }
        Ok(out.into())
    }

    async fn find_all_legacy_extra_hours(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[LegacyExtraHoursRow]>, DaoError> {
        // Same shape as `find_legacy_extra_hours_not_yet_migrated` minus the
        // NOT-IN subquery — used by `CutoverService::profile()` (SC-1).
        let rows = sqlx::query!(
            r#"SELECT eh.id            AS "id!: Vec<u8>",
                      eh.sales_person_id AS "sales_person_id!: Vec<u8>",
                      eh.category      AS "category!: String",
                      eh.date_time     AS "date_time!: String",
                      eh.amount        AS "amount!: f64"
               FROM extra_hours eh
               WHERE eh.deleted IS NULL
                 AND eh.category IN ('Vacation', 'SickLeave', 'UnpaidLeave')
               ORDER BY eh.sales_person_id, eh.category, eh.date_time"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let mut out: Vec<LegacyExtraHoursRow> = Vec::with_capacity(rows.len());
        for r in rows.iter() {
            out.push(LegacyExtraHoursRow {
                id: Uuid::from_slice(&r.id)?,
                sales_person_id: Uuid::from_slice(&r.sales_person_id)?,
                category: legacy_category_from_str(&r.category)?,
                date_time: PrimitiveDateTime::parse(&r.date_time, &Iso8601::DATE_TIME)?,
                amount: r.amount as f32,
            });
        }
        Ok(out.into())
    }

    async fn upsert_migration_source(
        &self,
        row: &MigrationSourceRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let extra_hours_id = row.extra_hours_id.as_bytes().to_vec();
        let absence_period_id = row.absence_period_id.as_bytes().to_vec();
        let cutover_run_id = row.cutover_run_id.as_bytes().to_vec();
        let migrated_at = row.migrated_at.format(&Iso8601::DATE_TIME)?;
        sqlx::query!(
            r#"INSERT INTO absence_period_migration_source
               (extra_hours_id, absence_period_id, cutover_run_id, migrated_at)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(extra_hours_id) DO NOTHING"#,
            extra_hours_id,
            absence_period_id,
            cutover_run_id,
            migrated_at,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn upsert_quarantine(
        &self,
        row: &QuarantineRow,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let extra_hours_id = row.extra_hours_id.as_bytes().to_vec();
        let sales_person_id = row.sales_person_id.as_bytes().to_vec();
        let cutover_run_id = row.cutover_run_id.as_bytes().to_vec();
        let category_str = legacy_category_to_str(&row.category);
        let date_time = row.date_time.format(&Iso8601::DATE_TIME)?;
        let migrated_at = row.migrated_at.format(&Iso8601::DATE_TIME)?;
        let reason = row.reason.as_ref();
        let amount = row.amount as f64;
        sqlx::query!(
            r#"INSERT INTO absence_migration_quarantine
               (extra_hours_id, reason, sales_person_id, category, date_time, amount, cutover_run_id, migrated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(extra_hours_id) DO UPDATE SET
                 reason = excluded.reason,
                 cutover_run_id = excluded.cutover_run_id,
                 migrated_at = excluded.migrated_at"#,
            extra_hours_id,
            reason,
            sales_person_id,
            category_str,
            date_time,
            amount,
            cutover_run_id,
            migrated_at,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn find_legacy_scope_set(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[(Uuid, u32)]>, DaoError> {
        // (sales_person_id, year) tuples for every active legacy extra_hours row
        // with non-zero amount. Used for both the gate scope (D-Phase4-05) and
        // the carryover refresh scope (D-Phase4-12).
        let rows = sqlx::query!(
            r#"SELECT DISTINCT
                     eh.sales_person_id AS "sales_person_id!: Vec<u8>",
                     CAST(strftime('%Y', eh.date_time) AS INTEGER) AS "year!: i64"
               FROM extra_hours eh
               WHERE eh.deleted IS NULL
                 AND eh.category IN ('Vacation', 'SickLeave', 'UnpaidLeave')
                 AND eh.amount != 0.0
               ORDER BY eh.sales_person_id, CAST(strftime('%Y', eh.date_time) AS INTEGER)"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let mut out: Vec<(Uuid, u32)> = Vec::with_capacity(rows.len());
        for r in rows.iter() {
            out.push((Uuid::from_slice(&r.sales_person_id)?, r.year as u32));
        }
        Ok(out.into())
    }

    async fn sum_legacy_extra_hours(
        &self,
        sales_person_id: Uuid,
        category: &ExtraHoursCategoryEntity,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<f32, DaoError> {
        let sp_vec = sales_person_id.as_bytes().to_vec();
        let category_str = legacy_category_to_str(category);
        let row = sqlx::query!(
            r#"SELECT COALESCE(SUM(amount), 0.0) AS "sum!: f64"
               FROM extra_hours
               WHERE deleted IS NULL
                 AND sales_person_id = ?
                 AND category = ?
                 AND CAST(strftime('%Y', date_time) AS INTEGER) = ?"#,
            sp_vec,
            category_str,
            year,
        )
        .fetch_one(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(row.sum as f32)
    }

    async fn count_quarantine_for_drift_row(
        &self,
        sales_person_id: Uuid,
        category: &ExtraHoursCategoryEntity,
        year: u32,
        cutover_run_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<(u32, Arc<[Arc<str>]>), DaoError> {
        let sp_vec = sales_person_id.as_bytes().to_vec();
        let cutover_run_id_vec = cutover_run_id.as_bytes().to_vec();
        let category_str = legacy_category_to_str(category);

        // First: COUNT(*) for the (sp, category, year, run_id) tuple.
        let count_row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!: i64"
               FROM absence_migration_quarantine
               WHERE sales_person_id = ?
                 AND category = ?
                 AND CAST(strftime('%Y', date_time) AS INTEGER) = ?
                 AND cutover_run_id = ?"#,
            sp_vec,
            category_str,
            year,
            cutover_run_id_vec,
        )
        .fetch_one(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        // Second: distinct reasons for the same scope.
        let reason_rows = sqlx::query!(
            r#"SELECT DISTINCT reason AS "reason!: String"
               FROM absence_migration_quarantine
               WHERE sales_person_id = ?
                 AND category = ?
                 AND CAST(strftime('%Y', date_time) AS INTEGER) = ?
                 AND cutover_run_id = ?
               ORDER BY reason"#,
            sp_vec,
            category_str,
            year,
            cutover_run_id_vec,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        let reasons: Arc<[Arc<str>]> = reason_rows
            .iter()
            .map(|r| Arc::<str>::from(r.reason.as_str()))
            .collect();

        Ok((count_row.count as u32, reasons))
    }

    async fn backup_carryover_for_scope(
        &self,
        cutover_run_id: Uuid,
        backed_up_at: time::PrimitiveDateTime,
        scope: &[(Uuid, u32)],
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        // No-op for empty scope (avoids generating an invalid IN-clause).
        if scope.is_empty() {
            return Ok(());
        }

        let cutover_run_id_vec = cutover_run_id.as_bytes().to_vec();
        let backed_up_at_str = backed_up_at.format(&Iso8601::DATE_TIME)?;

        // Build the dynamic INSERT-INTO-SELECT with a tuple-IN-clause.
        // Schema reference: 20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql
        // Columns (in order): cutover_run_id, sales_person_id, year,
        //                     carryover_hours, vacation, created, deleted,
        //                     update_process, update_version, backed_up_at.
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "INSERT INTO employee_yearly_carryover_pre_cutover_backup \
             (cutover_run_id, sales_person_id, year, carryover_hours, vacation, \
              created, deleted, update_process, update_version, backed_up_at) \
             SELECT ",
        );
        qb.push_bind(cutover_run_id_vec);
        qb.push(", c.sales_person_id, c.year, c.carryover_hours, c.vacation, \
                  c.created, c.deleted, c.update_process, c.update_version, ");
        qb.push_bind(backed_up_at_str);
        qb.push(" FROM employee_yearly_carryover c WHERE (c.sales_person_id, c.year) IN ");
        qb.push_tuples(scope.iter(), |mut b, (sp_id, year)| {
            b.push_bind(sp_id.as_bytes().to_vec());
            b.push_bind(*year);
        });

        qb.build()
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }
}
