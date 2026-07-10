//! Phase 54 (VOL-ACCT / D-54-DM-01): Basic-Tier Rebooking-Batch Service.
//!
//! Tier-Klassifizierung: **Basic-Service (Entity-Manager)**. Der Service haengt
//! ausschliesslich von DAO + Permission + Clock + Uuid + Transaction ab —
//! KEIN Domain-Service als Dependency, damit kein Zyklus mit dem spaeteren
//! Business-Logic `RebookingReconciliationService` (Phase 55) entsteht.
//!
//! Permissionsmodell: `find_by_id`, `find_by_sales_person_year_week` und
//! `create` sind allesamt HR-gated. Ein nicht-HR-Aufrufer erhaelt
//! `ServiceError::Forbidden`.
//!
//! Phase 55 wird zusaetzliche Methoden hinzufuegen (update_state
//! approve/reject, list_pending). In Phase 54 halten wir das Trait minimal,
//! damit die Basic-Tier-Konvention (Basic-Service ohne Domain-Service-Dep)
//! klar sichtbar bleibt.
//!
//! `automock` erzeugt `MockRebookingBatchService` fuer downstream Tests.

use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::rebooking_batch::{RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchState};
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingBatchService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Liefert den aktiven Batch fuer die uebergebene id (oder `None`, wenn
    /// keiner existiert bzw. soft-deleted ist). HR-gated.
    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError>;

    /// Liefert den aktiven Batch fuer den globalen UNIQUE-Slot
    /// `(sales_person_id, iso_year, iso_week)` (oder `None`). HR-gated.
    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError>;

    /// Wave-1-Contract: Batch + Entries werden in EINER Transaktion
    /// gespeichert. Ein bereits aktiver Batch fuer denselben UNIQUE-Slot
    /// `(sales_person_id, iso_year, iso_week)` fuehrt zu
    /// `ServiceError::EntityAlreadyExists(batch.id)` — der DB-UNIQUE-Index
    /// `rebooking_batch_week_unique_idx` (Migration 20260707000000) ist die
    /// Autoritaet (D-54-DM-01, Claim-on-Suggest); der Service ergaenzt einen
    /// Pre-Check innerhalb derselben Transaktion, damit der Aufrufer nie eine
    /// Panic sieht.
    ///
    /// Der Service uebernimmt `kind` und `state` unveraendert vom Aufrufer;
    /// `id`, `version`, `created` auf Batch- und Entry-Ebene werden defensiv
    /// gesetzt (falls `Uuid::nil()` bzw. Default-Zeitstempel uebergeben wird,
    /// werden frische Werte aus UuidService/ClockService gezogen).
    async fn create(
        &self,
        batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError>;

    /// Phase 55 (D-55-02): Liefert alle aktiven `state='pending'`-Batches
    /// fuer einen SalesPerson. HR-gated. Konsumiert vom
    /// `ShortEmployeeReportTO.has_pending_rebooking`-Prediktor (Plan 55-02)
    /// und vom `RebookingReconciliationService::list_pending_for_sales_person`.
    async fn find_pending_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingBatchEntity]>, ServiceError>;

    /// Phase 55 (D-55-02): Liefert alle aktiven `state='pending'`-Batches
    /// **phase-weit** (ueber alle Personen). HR-gated. Konsumiert vom
    /// `RebookingReconciliationService::list_pending_for_sales_person(None, ...)`
    /// fuer den `GET /rebooking-suggestions`-Endpoint (Plan 55-02).
    async fn list_all_pending(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingBatchEntity]>, ServiceError>;

    /// Phase 55 (HR-ALERT-03, T-55-01): state-conditional UPDATE fuer
    /// Approve/Reject-Race-Schutz. HR-gated.
    ///
    /// Erzeugt intern eine frische `new_version` via `UuidService` und
    /// reicht sie an den DAO durch. Rueckgabe: Anzahl affected rows. `0`
    /// bedeutet, dass der Batch bereits nicht mehr im `expected_state` war
    /// (parallele HR-Aktion) — der Aufrufer (Reconciliation-Service) mappt
    /// das auf `ServiceError::BatchAlreadyResolved`.
    #[allow(clippy::too_many_arguments)]
    async fn update_state_conditional(
        &self,
        batch_id: Uuid,
        expected_state: RebookingBatchState,
        new_state: RebookingBatchState,
        approved: Option<time::PrimitiveDateTime>,
        approved_by: Option<Arc<str>>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<u64, ServiceError>;
}
