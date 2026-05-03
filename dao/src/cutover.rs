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

/// Result of `count_quarantine_for_drift_row`: (row_count, distinct_reasons).
pub type QuarantineCountForDriftRow = (u32, Arc<[Arc<str>]>);

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
    ) -> Result<QuarantineCountForDriftRow, DaoError>;

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
