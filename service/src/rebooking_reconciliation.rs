//! Phase 55 (F3 REB-MANUAL + F5 HR-ALERT): Business-Logic Rebooking-
//! Reconciliation-Service.
//!
//! Tier-Klassifizierung: **Business-Logic-Service**. Orchestriert
//! `ExtraHoursService` (Basic — Pair-Row-Schreiben), `RebookingBatchService`
//! (Basic — Batch/Entry-Persistenz + state-conditional UPDATE) und
//! `ReportingService` (Business-Logic — IST/DANN-Snapshot fuer Audit + Modal)
//! in **einer** Transaktion (REB-MANUAL-01 Atomarität).
//!
//! Konsumierbar von REST (Plan 55-02: `POST /rebooking/manual`,
//! `GET /rebooking-suggestions`, `POST /rebooking-suggestions/{id}/approve`,
//! `POST /rebooking-suggestions/{id}/reject`).
//!
//! Permissionsmodell: **HR-only.** Jede public Trait-Methode gatet als
//! erste await-Operation `PermissionService::check_permission(HR_PRIVILEGE, ...)`
//! (T-55-02).
//!
//! Kein Undo (D-55-04): Weder Approve noch Reject kann rueckgaengig gemacht
//! werden. Approved-Batches persistieren inkl. Pair-ExtraHours; Rejected-Batches
//! bleiben im Audit — der UNIQUE-Slot fuer `(sp, iso_year, iso_week)` bleibt bis
//! zur naechsten ISO-Woche belegt.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

/// Richtung einer manuellen Rebooking-Buchung (D-55-06). Die Menge
/// (`hours`) wird stets als **positive** `f32` uebergeben; das Enum
/// entscheidet, welche `ExtraHoursCategory` die `-N`-Row und welche die
/// `+N`-Row bekommt.
///
/// - `VolunteerToExtra`: `-N VolunteerWork` + `+N ExtraWork` — HR bucht
///   freiwillige Stunden in bezahlte Ueberstunden um (Balance-Ausgleich).
/// - `ExtraToVolunteer`: `-N ExtraWork` + `+N VolunteerWork` — Umkehrpfad
///   (Korrektur, falls Ueberstunden faelschlich als bezahlt gebucht wurden).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingDirection {
    /// -N VolunteerWork -> +N ExtraWork (Balance ausgleichen).
    VolunteerToExtra,
    /// -N ExtraWork -> +N VolunteerWork (Umkehrpfad).
    ExtraToVolunteer,
}

/// D-55-01: Reine Funktion zur Auswertung des HR-Alert-Predicates fuer die
/// Employee-Overview-Warnzeile.
///
/// Predicate: `cap_active && balance <= -0.5 && voluntary_ist > 0.0`.
///
/// - `cap_active`: `cap_planned_hours_to_expected` fuer den betreffenden
///   Vertrag/Wochenscope aktiv (nur dann kann Ueberlauf ins Ehrenamt
///   entstehen, sonst ist der Predicate strukturell irrelevant).
/// - `balance <= -0.5`: Float-Noise-Tolerance. Ein strict `< 0`-Predicate
///   wuerde 0.0001-Defizite als Alert zeigen; `<= -0.5h` deckt sich mit der
///   UI-Granularitaet (Stunden mit einer Nachkommastelle).
/// - `voluntary_ist > 0.0`: nur wenn die Person tatsaechlich Ehrenamt-
///   Stunden geleistet hat, gibt es etwas zum Rebooken.
///
/// Truth-Table + Grenzfaelle: siehe
/// `service_impl/src/test/rebooking_reconciliation.rs::predicate_truth_table`.
pub fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}

/// D-55-03: Reine Funktion fuer die vorgeschlagene Rebooking-Menge.
///
/// Formel: `min(|balance|, voluntary_ist).max(0.0)`.
///
/// - Deckt das Balance-Defizit aus (`|balance|`).
/// - Aber hoechstens so viel wie tatsaechlich als Ehrenamt geleistet wurde
///   (`voluntary_ist`) — sonst wuerde man mehr rebooken als geleistet und
///   die Rebooking-Neutralitaet der Read-Aggregate (VOL-ACCT-03) brechen.
/// - `.max(0.0)`: falls `voluntary_ist` negativ ist (Datenkorruption /
///   Testfall), gibt der Vorschlag keine Umbuchung zurueck.
///
/// Fat-Backend-Regel (MEMORY `feedback_fat_backend_thin_client`): das
/// Frontend fuehrt KEINE Arithmetik durch, sondern spiegelt nur den vom
/// Backend berechneten Wert.
pub fn proposed_rebooking_hours(balance: f32, voluntary_ist: f32) -> f32 {
    balance.abs().min(voluntary_ist).max(0.0)
}

/// Domain-Analog zum spaeteren `RebookingSuggestionTO` (rest-types, Plan
/// 55-02). Enthaelt IST/DANN-Werte fuer das HR-Alert-Modal + Manual-Preview.
///
/// D-55-03 (Fat-Backend): sowohl `voluntary_delta_before` als auch
/// `voluntary_delta_after` sind **Backend-berechnete** Felder — das FE
/// darf `voluntary_ist_before - voluntary_soll_before` NICHT selbst
/// rechnen. Precedent: MEMORY `feedback_fat_backend_thin_client`.
#[derive(Clone, Debug, PartialEq)]
pub struct RebookingSuggestion {
    /// Batch-Id des `rebooking_batch`-Eintrags (Pending fuer HrSuggestion,
    /// Approved fuer Manual). Bei `hydrate_pending_to_suggestion` liefert
    /// der batch selbst die Id; bei `rebook_manual` gibt der Service das
    /// frisch angelegte Entity zurueck.
    pub batch_id: Uuid,
    /// Betroffener SalesPerson.
    pub sales_person_id: Uuid,
    /// ISO-Wochenjahr der zu rebookenden Woche (HR-gewaehlt, D-55-05).
    pub iso_year: u32,
    /// ISO-Woche der zu rebookenden Woche (HR-gewaehlt, D-55-05).
    pub iso_week: u8,
    /// Vom Backend berechnete Rebooking-Menge (D-55-03,
    /// `min(|balance|, voluntary_ist)`). Stets `>= 0.0`.
    pub proposed_hours: f32,
    /// Balance-Stundenkonto vor Anwendung des Vorschlags.
    pub balance_before: f32,
    /// F1-Ist (voluntary_ist) vor Anwendung des Vorschlags.
    pub voluntary_ist_before: f32,
    /// F2-Soll (voluntary_soll) vor Anwendung des Vorschlags.
    pub voluntary_soll_before: f32,
    /// Backend-berechnetes Delta = `voluntary_ist_before - voluntary_soll_before`
    /// (D-55-03, Fat-Backend).
    pub voluntary_delta_before: f32,
    /// Balance-Stundenkonto NACH hypothetischer Anwendung des Vorschlags.
    pub balance_after: f32,
    /// F1-Ist NACH hypothetischer Anwendung des Vorschlags.
    pub voluntary_ist_after: f32,
    /// F2-Soll NACH hypothetischer Anwendung des Vorschlags. **Soll aendert
    /// sich durch Umbuchung nicht** — bleibt identisch zu `voluntary_soll_before`.
    pub voluntary_soll_after: f32,
    /// Backend-berechnetes Delta = `voluntary_ist_after - voluntary_soll_after`
    /// (D-55-03, Fat-Backend).
    pub voluntary_delta_after: f32,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait RebookingReconciliationService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// F3 REB-MANUAL-01/02: manuell durch HR angelegte Umbuchung.
    ///
    /// HR-gated. Schreibt in **einer** Transaktion:
    /// 1. Zwei `ExtraHours`-Rows (Marker `ExtraHoursSource::Rebooking`,
    ///    Betrag `-hours` und `+hours` je nach `direction`).
    /// 2. Ein `rebooking_batch` (`kind=Manual`, `state=Approved`).
    /// 3. Ein `rebooking_batch_entry` mit FK auf beide ExtraHours-Rows.
    ///
    /// Die Woche wird HR-seitig gewaehlt (D-55-05) — retrospektive Buchungen
    /// fuer alte ISO-Wochen sind erlaubt. UNIQUE-Slot-Kollisionen
    /// (`(sp, iso_year, iso_week)` bereits belegt) werden vom
    /// `RebookingBatchService::create` unveraendert als
    /// `ServiceError::EntityAlreadyExists` propagiert (HTTP 409 in Plan 02).
    #[allow(clippy::too_many_arguments)]
    async fn rebook_manual(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        direction: RebookingDirection,
        hours: f32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<dao::rebooking_batch::RebookingBatchEntity, ServiceError>;

    /// F5 HR-ALERT: HR erzeugt einen Pending-Vorschlag (Claim-on-Suggest,
    /// D-54-DM-01). HR-gated.
    ///
    /// Schreibt **keine** ExtraHours-Rows — nur `rebooking_batch`
    /// (`kind=HrSuggestion`, `state=Pending`) + Entry mit `NULL`-FKs.
    /// Die UNIQUE-Slot-Reservierung passiert direkt via DB-Index; damit
    /// existiert nur **ein** offener Vorschlag pro Woche pro Person.
    ///
    /// Rueckgabe: `RebookingSuggestion` mit IST/DANN-Bundle (Backend-
    /// berechnet, D-55-03) fuer die Modal-Anzeige.
    async fn suggest_for_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError>;

    /// F5 HR-ALERT Approve: state-conditional UPDATE `Pending → Approved`
    /// + Pair-ExtraHours-Rows in derselben Transaktion. HR-gated.
    ///
    /// Race-Schutz (HR-ALERT-03, T-55-01): der DAO-`update_state_conditional`
    /// prueft `WHERE state='pending'` und liefert die Anzahl der affected
    /// rows. Bei zwei parallelen HR-Approve-Klicks gewinnt genau einer;
    /// der zweite bekommt `ServiceError::BatchAlreadyResolved`.
    async fn approve_suggestion(
        &self,
        batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<dao::rebooking_batch::RebookingBatchEntity, ServiceError>;

    /// F5 HR-ALERT Reject: state-conditional UPDATE `Pending → Rejected`.
    /// **Kein** ExtraHours-Schreiben. HR-gated.
    ///
    /// UNIQUE-Slot bleibt belegt bis zur naechsten ISO-Woche (D-55-07):
    /// dieselbe Person kann in derselben Woche keinen neuen Vorschlag
    /// erhalten, aber der Rejected-Batch bleibt im Audit-Trail persistent.
    async fn reject_suggestion(
        &self,
        batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<dao::rebooking_batch::RebookingBatchEntity, ServiceError>;

    /// F5 HR-ALERT Liste offener Vorschlaege (Read). HR-gated.
    ///
    /// - `Some(sales_person_id)`: liefert alle offenen (`state=Pending`)
    ///   Batches fuer diese Person (typischerweise 0 oder 1 wegen
    ///   UNIQUE-Slot; ueber alle iso_year/iso_week hinweg).
    /// - `None`: liefert alle offenen Suggestions **phase-weit** (fuer den
    ///   `GET /rebooking-suggestions`-Endpoint in Plan 55-02).
    ///
    /// Die Rueckgabe sind **hydrierte** `RebookingSuggestion`-Structs
    /// (nicht rohe Batch-Entities): der Service ruft intern
    /// `hydrate_pending_to_suggestion` fuer jeden Eintrag und liefert
    /// `RebookingSuggestion` inklusive IST/DANN-Delta-Bundle (D-55-02, D-55-03).
    async fn list_pending_for_sales_person(
        &self,
        sales_person_id: Option<Uuid>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingSuggestion]>, ServiceError>;

    /// Baut aus einem persistierten `RebookingBatchEntity` eine
    /// vollstaendige `RebookingSuggestion` mit IST/DANN-Bundle.
    ///
    /// Intern von `list_pending_for_sales_person` und `suggest_for_week`
    /// genutzt (keine Code-Duplikation); auch als Hilfsverb fuer das
    /// spaetere `GET /rebooking-suggestions/{id}`-Detail-Endpoint (Plan
    /// 55-02) nutzbar. HR-gated.
    ///
    /// Backend-computed Deltas nach D-55-03 (Fat-Backend):
    /// `voluntary_delta_before = voluntary_ist_before - voluntary_soll_before`,
    /// `voluntary_delta_after = voluntary_ist_after - voluntary_soll_after`.
    async fn hydrate_pending_to_suggestion(
        &self,
        batch: &dao::rebooking_batch::RebookingBatchEntity,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError>;
}
