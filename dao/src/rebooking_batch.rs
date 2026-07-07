//! Phase 54 (VOL-ACCT / D-54-DM-01): Basic-Tier Rebooking-Batch DAO.
//!
//! Konsumenten (F3 manuell / F4 Cron / F5 Alert) folgen ab Phase 55/56.
//! Der UNIQUE-Partial-Index `rebooking_batch_week_unique_idx` enforced
//! `(sales_person_id, iso_year, iso_week)` global ueber alle kinds
//! (Claim-on-Suggest: hr_suggestion(state=pending) beansprucht die
//! Wochen-Slot direkt via UNIQUE, keine eigene State-Machine).

use std::sync::Arc;

use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

/// Diskriminator fuer den Ursprung eines Batches. String-Konversion in
/// `dao_impl_sqlite`; hier reines Domain-Enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingBatchKind {
    /// Manuell durch HR angelegt (Phase 55 F3).
    Manual,
    /// Vom Alert-System vorgeschlagen (Phase 55 F5).
    HrSuggestion,
    /// Vom Auto-Cron erzeugt (Phase 56 F4).
    AutoCron,
    /// Backfill-Lauf des Auto-Cron (Phase 56 F4).
    AutoCronBackfill,
}

/// Lebenszyklus-State eines Batches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingBatchState {
    /// Vorgeschlagen / offen; noch keine Approved-Auswirkung auf extra_hours.
    Pending,
    /// Genehmigt; extra_hours-Pair-Rows angelegt.
    Approved,
    /// Abgelehnt; extra_hours-Pair-Rows NICHT angelegt.
    Rejected,
    /// Cron uebersprungen, weil die Woche gesperrt ist (Phase 56 F4-Gate).
    SkippedLocked,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RebookingBatchEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub kind: RebookingBatchKind,
    pub state: RebookingBatchState,
    pub created: time::PrimitiveDateTime,
    pub approved: Option<time::PrimitiveDateTime>,
    pub approved_by: Option<Arc<str>>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RebookingBatchEntryEntity {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub sales_person_id: Uuid,
    /// Betrag der Umbuchung (positiv).
    pub hours: f32,
    /// Snapshot Stundenkonto vor Rebooking (Audit).
    pub balance_before: f32,
    /// F1-Ist zum Zeitpunkt des Rebookings (Audit).
    pub voluntary_actual: f32,
    /// F2-Soll zum Zeitpunkt des Rebookings (Audit).
    pub voluntary_committed: f32,
    /// FK auf extra_hours (-N VolunteerWork); NULL bis approved.
    pub extra_hours_out_id: Option<Uuid>,
    /// FK auf extra_hours (+N ExtraWork); NULL bis approved.
    pub extra_hours_in_id: Option<Uuid>,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait RebookingBatchDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError>;

    /// Liefert den **aktiven** (deleted IS NULL) Batch fuer den globalen
    /// UNIQUE-Slot (sales_person_id, iso_year, iso_week).
    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        tx: Self::Transaction,
    ) -> Result<Option<RebookingBatchEntity>, DaoError>;

    /// Schreibt Batch + Entries in EINEM DB-Trip (Rollback bei UNIQUE-Violation).
    async fn create_batch_with_entries(
        &self,
        batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn list_entries_for_batch(
        &self,
        batch_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[RebookingBatchEntryEntity]>, DaoError>;
}
