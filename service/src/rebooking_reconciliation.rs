//! Phase 55 (REB-MANUAL / HR-ALERT / VOL-ACCT-03): Business-Logic-Tier
//! `RebookingReconciliationService`.
//!
//! Tier-Klassifizierung: **Business-Logic-Service**. Er kombiniert mehrere
//! Aggregate (ExtraHours + RebookingBatch) und pflegt die Cross-Entity-
//! Invariante der freiwilligen Umbuchung. Er konsumiert die Basic-Services
//! `ExtraHoursService` + `RebookingBatchService` sowie die Business-Logic-
//! Services `ReportingService` + `VoluntaryStatsService` (kein Zyklus:
//! keiner dieser Services kennt den RebookingReconciliationService).
//!
//! Herzstueck ist EIN interner Pair-Write-Path (im Impl), den F3 (Manual) und
//! F5-Approve (HrSuggestion) beide fuer die Pair-ExtraHours-Erzeugung nutzen
//! (Single write-path, Pitfall 15).
//!
//! F5-Lifecycle ist Modell B (Claim-on-Suggest, D-55-B): `suggest` claimt via
//! `RebookingBatchService::create_pending_suggestion` einen pending-Batch und
//! liefert IST/DANN; `approve`/`reject` transitionieren den state
//! (approve zusaetzlich → Pair).
//!
//! Permissionsmodell: alle vier Methoden sind HR-gated (T-55-05); ein
//! nicht-HR-Aufrufer erhaelt `ServiceError::Forbidden`.
//!
//! `automock` erzeugt `MockRebookingReconciliationService` fuer downstream Tests.

use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use uuid::Uuid;

/// Richtung einer manuellen Umbuchung (D-55-06): das Modal waehlt die Richtung
/// explizit — kein Delta-Vorzeichen-Ableiten im Trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingDirection {
    /// -N VolunteerWork, +N ExtraWork (Standard-Ausgleich fuer gedeckelte
    /// Mitarbeiter im Minus).
    VolunteerToExtra,
    /// -N ExtraWork, +N VolunteerWork (Korrektur ueberzahlter Zusagen).
    ExtraToVolunteer,
}

/// Backend-computed IST/DANN-Vorschau fuer den F5-HR-Alert (HR-ALERT-02).
///
/// Alle DANN-Felder sind vom Backend berechnet (kein FE-Arithmetik-Vorgriff):
/// `balance_after = balance + proposed_hours`,
/// `voluntary_ist_after = voluntary_ist - proposed_hours`. Die 55-03-REST-
/// Schicht mappt diese Domain-Struktur auf die `RebookingSuggestionTO`.
#[derive(Clone, Debug, PartialEq)]
pub struct RebookingSuggestion {
    /// Die neu erzeugte (oder bereits offene, idempotent) pending-Batch-id.
    pub pending_rebooking_id: Uuid,
    /// Vorgeschlagene Umbuchungs-Stunden = `min(|balance|, voluntary_ist)`.
    pub proposed_hours: f32,
    /// IST: Stundenkonto-Balance vor Umbuchung.
    pub balance: f32,
    /// IST: freiwillige Ist-Stunden vor Umbuchung.
    pub voluntary_ist: f32,
    /// DANN: Balance nach Umbuchung (`balance + proposed_hours`).
    pub balance_after: f32,
    /// DANN: freiwillige Ist-Stunden nach Umbuchung
    /// (`voluntary_ist - proposed_hours`).
    pub voluntary_ist_after: f32,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingReconciliationService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// F3 (REB-MANUAL-01/02): one-shot manuelle Umbuchung. Erzeugt atomar das
    /// -N/+N-ExtraHours-Pair (source=Rebooking) + einen `rebooking_batch`
    /// (kind=Manual, state=Approved) + einen Entry mit den beiden Pair-ids —
    /// alles in EINER Transaktion. UNIQUE-Slot-Kollision → `EntityAlreadyExists`.
    /// KEIN suggest/pending-Zwischenschritt. HR-gated.
    #[allow(clippy::too_many_arguments)]
    async fn rebook_manual(
        &self,
        sales_person_id: Uuid,
        direction: RebookingDirection,
        hours: f32,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// F5 (HR-ALERT-02/04, Modell B Claim): claimt via
    /// `RebookingBatchService::create_pending_suggestion` einen pending-Batch
    /// (kind=HrSuggestion, state=Pending) fuer den UNIQUE-Slot und liefert die
    /// backend-computed IST/DANN-Vorschau. Idempotent bei bereits offenem
    /// pending (liefert dieselbe id); entschiedener Batch → `EntityAlreadyExists`
    /// (→ 409). HR-gated.
    async fn suggest(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError>;

    /// F5 (HR-ALERT-03 Decide): hebt den pending-Batch via
    /// `RebookingBatchService::update_state` auf `Approved` (state-conditional
    /// UPDATE ZUERST, Pitfall 12), danach — nur bei Erfolg — das
    /// -N/+N-ExtraHours-Pair (VolunteerToExtra) in derselben Tx. Double-Approve-
    /// Verlierer (rows_affected==0) → `EntityAlreadyExists` (→ 409). HR-gated.
    async fn approve(
        &self,
        rebooking_batch_id: Uuid,
        hours: f32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// F5 (HR-ALERT-03 Decide): hebt den pending-Batch via
    /// `RebookingBatchService::update_state` auf `Rejected` — KEINE Pair-Rows.
    /// HR-gated.
    async fn reject(
        &self,
        rebooking_batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
