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
    /// Kein buchbarer Ueberschuss. Zwei Ursachen fallen hier zusammen: der
    /// Wochen-Ist erreicht die Freiwilligen-Zusage nicht
    /// (`Ist <= committed_voluntary`), ODER das bezahlte Stundenkonto hat keinen
    /// Headroom mehr (`balance >= 0`, Invariante I-4, REB-AUTO-06). Auch ein
    /// Rest unterhalb der Mindest-Buchungsgroesse von 0.01 h faellt hierunter
    /// (f32-Rauschen, WR-02).
    NoExcess,
    /// UNIQUE-Slot `(sales_person_id, iso_year, iso_week)` bereits belegt
    /// (frueherer Cron-Lauf, manuelle Umbuchung oder pending HR-Suggestion —
    /// Claim-on-Suggest, D-02). Idempotenz-Skip (REB-AUTO-03).
    ///
    /// **Achtung (WR-13):** ein zweiter Lauf ueber eine bereits vom Cron
    /// verarbeitete Woche meldet in der Regel [`SkipReason::NoExcess`], nicht
    /// diese Variante. Seit VOL-ACCT-04 senkt das `-h`-VolunteerWork-Bein den
    /// Ist-Wert, aus dem der Ueberschuss berechnet wird; der Excess-Guard steht
    /// vor dem Slot-Claim und greift zuerst. `AlreadyProcessed` bleibt der Grund,
    /// wenn der Slot von einer FREMDEN Quelle belegt ist (manuelle Umbuchung,
    /// pending HR-Suggestion) oder der Ueberschuss aus anderen Gruenden erneut
    /// positiv ist. Der UNIQUE-Slot ist in beiden Faellen die Absicherung gegen
    /// Doppel-Writes; betroffen ist nur die Skip-Reason-Statistik.
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
///
/// **Ausnahme `skip_reason == Some(SkipReason::ToggleOff)` (WR-06):** liegt die
/// Woche vor dem Stichtag, sind `voluntary_ist`, `voluntary_soll`,
/// `committed_voluntary` und `excess` **nicht berechnet** und stehen auf `0.0`.
/// Das Cutoff-Gate laeuft bewusst VOR den teuren Reads (ein year-to-date-Report
/// je (Person, Woche) ist selbst O(Wochen im Jahr)) — ein historischer Backfill,
/// dessen `from` vor dem Stichtag liegt, wuerde sonst die gesamte Report-Arbeit
/// leisten, nur um jede Zeile anschliessend zu verwerfen.
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
    /// `min( max(ist - committed, 0), max(-balance, 0) )` — der Ueberschuss, der
    /// gebucht wuerde, **gedeckelt auf den Headroom des bezahlten
    /// Stundenkontos** (REB-AUTO-06, Phase 57). Die `balance` ist die
    /// year-to-date-Balance inklusive Vorjahres-Carryover bis einschliesslich
    /// dieser Woche (D-57-07) — nicht die isolierte Wochen-Balance.
    ///
    /// Im Apply-Modus traegt das Feld nach dem Lauf den Betrag, den
    /// `run_for_week` tatsaechlich gebucht hat (WR-07).
    ///
    /// Im Dry-Run fuehrt `backfill_range` den ueber frueheren Wochen bereits
    /// vorgemerkten Uebertrag pro Person mit, damit die Summe der Preview-Werte
    /// dem entspricht, was ein Apply-Lauf bucht (Invariante I-7, CR-02) — ohne
    /// diese Buchfuehrung saehe jede Woche denselben unverbrauchten Headroom.
    pub excess: f32,
    /// `true`, wenn `excess >= MIN_BOOKABLE_HOURS` (0.01 h — darunter ist der
    /// Rest f32-Rauschen, WR-02) und die Woche nicht geskippt wuerde.
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
    /// 3. **Balance-Read (REB-AUTO-06, D-57-07):** die kumulierte
    ///    year-to-date-Balance vom Montag der ISO-Woche 1 des `iso_year` bis
    ///    Sonntag der verarbeiteten Woche, `include_carryover = true`. Der Read
    ///    steht bewusst NACH den beiden Gates (kein Report-Roundtrip fuer einen
    ///    ausgeschalteten Cron / eine gesperrte Woche) und ist an ISO-Koordinaten
    ///    verankert, nicht am Kalender-Neujahr (`first_day_in_year` faellt in
    ///    Jahren mit Fr/Sa/So-Neujahr ins vorherige ISO-Jahr).
    /// 4. Ueberschuss = `min( max(Ist - committed_voluntary, 0),
    ///    max(-balance, 0) )` — der freiwillige Wochen-Ueberschuss (Range Mo..=So),
    ///    gedeckelt auf den Headroom des bezahlten Stundenkontos: die Automatik
    ///    gleicht das Konto aus, hebt es aber nie ueber 0 (Invariante I-4).
    ///    Unterhalb der Mindest-Buchungsgroesse von 0.01 h → `Skipped{NoExcess}`
    ///    (ein f32-Residuum darf kein Micro-Pair erzeugen und keinen UNIQUE-Slot
    ///    belegen, WR-02).
    /// 5. Sonst `rebooking_batch_service.create(kind, state=Approved)` +
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
