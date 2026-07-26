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
use dao::rebooking_batch::RebookingBatchKind;
use dao::MockTransaction;
use mockall::automock;
use shifty_utils::ShiftyWeek;
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

/// F4 (REB-AUTO-01/03): Grund, warum der Auto-Cron eine (SalesPerson, ISO-Woche)
/// NICHT gebucht hat. Rein diagnostisch (Audit / Backfill-Preview); jeder Grund
/// bedeutet "kein -N/+N-Pair geschrieben".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// `Ist_vorwoche <= Soll_vorwoche + committed_voluntary_vorwoche` — kein
    /// Ueberschuss, nichts umzubuchen.
    NoExcess,
    /// UNIQUE-Slot `(sales_person_id, iso_year, iso_week)` bereits belegt
    /// (frueherer Cron-Lauf, manuelle Umbuchung oder pending HR-Suggestion —
    /// Claim-on-Suggest, D-02). Idempotenz-Skip (REB-AUTO-03).
    AlreadyProcessed,
    /// Die ISO-Woche ist `WeekStatus::Locked` (D-04). Es wird ein
    /// `SkippedLocked`-Audit-Batch persistiert, aber kein Pair.
    WeekLocked,
    /// Der Stichtag-Toggle ist nicht gesetzt oder der Montag der ISO-Woche liegt
    /// strikt vor `active_from` (F4-Self-Guard, D-06 write side).
    ToggleOff,
}

/// F4 (REB-AUTO-01): Ergebnis eines `run_for_week`-Laufs fuer genau eine
/// (SalesPerson, ISO-Woche).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunForWeekOutcome {
    /// Ueberschuss gebucht: `hours` freiwillige Stunden wurden als Ausgleich auf
    /// ExtraWork umgebucht (ein `source=Rebooking`-Pair + ein Approved-Batch).
    Booked { hours: f32 },
    /// Nichts gebucht — Grund im `reason`.
    Skipped { reason: SkipReason },
}

/// F4 (REB-AUTO-04): Ein Kandidaten-Item des Backfill-Laufs. Beschreibt fuer
/// eine (SalesPerson, ISO-Woche) die berechneten Groessen und ob (bzw. warum
/// nicht) gebucht wuerde. Im `dry_run` ist dies reine Vorschau (kein Write).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackfillPreviewItem {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    /// F1-Ist der Woche (`VoluntaryStats::ist_total`).
    pub voluntary_ist: f32,
    /// F2-Soll der Woche (`VoluntaryStats::soll_total`).
    pub voluntary_soll: f32,
    /// Committed-voluntary-Beitrag der Woche (= `voluntary_soll` fuer die
    /// Single-Week-Range).
    pub committed_voluntary: f32,
    /// `max(ist - (soll + committed), 0)` — der Ueberschuss, der gebucht wuerde.
    pub excess: f32,
    /// `true`, wenn `excess > 0` und die Woche nicht geskippt wuerde.
    pub would_book: bool,
    /// `true`, wenn diese Woche NICHT gebucht wurde/wuerde.
    pub skipped: bool,
    /// Grund fuer `skipped`; `None`, wenn gebucht.
    pub skip_reason: Option<SkipReason>,
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

    /// F4 (REB-AUTO-01/03): verarbeitet genau EINE ISO-Woche fuer EINEN
    /// gedeckelten `SalesPerson`. Trigger-agnostischer Motor — der Scheduler
    /// (Plan 03) und der Backfill (Plan 04) rufen dieselbe Methode.
    ///
    /// Ablauf (spiegelt `suggest`, nutzt den EINEN internen `rebook`-Pair-Write-
    /// Path, Pitfall 15):
    /// 1. F4-Self-Guard (D-06 write side): `voluntary_rebooking_gate::
    ///    should_run_for_week` — vor Stichtag / Toggle aus → `Skipped{ToggleOff}`.
    /// 2. `WeekStatus::Locked` → `SkippedLocked`-Audit-Batch + `Skipped{WeekLocked}`
    ///    (D-04); der Audit-Write ist idempotent (UNIQUE-Konflikt ignoriert).
    /// 3. Ueberschuss = `Ist - (Soll + committed_voluntary)` fuer die Wochen-Range
    ///    (Mo..=So); `<= 0` → `Skipped{NoExcess}`.
    /// 4. Sonst `rebooking_batch_service.create(kind, state=Approved)` +
    ///    `rebook(VolunteerToExtra, hours=excess)` in EINER Tx. UNIQUE-Konflikt
    ///    (frueherer Cron / Manual / pending HR-Suggestion) → `Skipped{AlreadyProcessed}`
    ///    (REB-AUTO-03 Idempotenz + Claim-on-Suggest D-02).
    ///
    /// `kind` erlaubt dem Backfill (Plan 04), `AutoCronBackfill` statt `AutoCron`
    /// zu buchen, ohne einen zweiten Write-Path zu erzeugen.
    #[allow(clippy::too_many_arguments)]
    async fn run_for_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        kind: RebookingBatchKind,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RunForWeekOutcome, ServiceError>;

    /// F4 (REB-AUTO-04): iteriert die historischen ISO-Wochen von `from` bis zur
    /// letzten abgeschlossenen Vorwoche (nie die laufende Woche) und liefert pro
    /// (gedeckelter `SalesPerson`, ISO-Woche) ein [`BackfillPreviewItem`].
    ///
    /// - `dry_run == true`: reine Vorschau — schreibt NICHTS (keine Tx, kein
    ///   `create`).
    /// - `dry_run == false`: delegiert pro Kandidat an [`run_for_week`] mit
    ///   `kind=AutoCronBackfill` (Single-Write-Path) und persistiert die Batches.
    ///
    /// HR-Admin-gated (T-56-04): `check_permission(HR_PRIVILEGE)` ist der erste
    /// await, vor jedem Datenabruf. Die Iterations-Obergrenze ist auf die letzte
    /// abgeschlossene Vorwoche geklemmt (T-56-05, DoS-Bound).
    async fn backfill_range(
        &self,
        from: ShiftyWeek,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<BackfillPreviewItem>, ServiceError>;
}
