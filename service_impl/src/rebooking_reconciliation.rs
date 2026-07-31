//! Phase 55 (REB-MANUAL / HR-ALERT / VOL-ACCT-03): Business-Logic-Tier
//! `RebookingReconciliationService`-Implementierung.
//!
//! Konsumiert die Basic-Services `ExtraHoursService` + `RebookingBatchService`
//! und die Business-Logic-Services `ReportingService` + `VoluntaryStatsService`
//! (Service-Tier-Konvention CLAUDE.md — Business-Logic darf Basic + andere
//! Business-Logic konsumieren, solange kein Zyklus entsteht).
//!
//! Tx-Orchestrierung (Vorbild `voluntary_stats.rs` / `rebooking_batch.rs`):
//! HR-Gate ZUERST, dann `use_transaction` → alle Cross-Service-Calls mit
//! `Some(tx.clone())` + `Authentication::Full` → `commit` erst am Ende. Kein
//! Basic-Service oeffnet im BL-Kontext seine eigene Tx (kein Teil-Commit).
//!
//! Single Pair-write-path `rebook` (Pitfall 15): F3 (rebook_manual) und
//! F5-approve teilen sich denselben internen Pair-Erzeuger fuer die
//! -N/+N-ExtraHours-Rows. Nur das Batch-Handling unterscheidet sich (F3 legt
//! einen Manual-Approved-Batch an; F5-approve transitioniert den bestehenden
//! pending-HrSuggestion-Batch, D-55-B).

use crate::gen_service_impl;
use async_trait::async_trait;
use dao::rebooking_batch::{
    RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchKind, RebookingBatchState,
};
use dao::TransactionDao;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService, ExtraHoursSource};
use service::permission::{Authentication, HR_PRIVILEGE};
use service::clock::ClockService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::rebooking_batch::RebookingBatchService;
use service::rebooking_reconciliation::{
    BackfillPreviewItem, RebookingDirection, RebookingReconciliationService, RebookingSuggestion,
    RunForWeekOutcome, SkipReason,
};
use service::reporting::ReportingService;
use service::toggle::ToggleService;
use service::voluntary_stats::VoluntaryStatsService;
use service::week_status::{WeekStatus, WeekStatusService};
use service::{PermissionService, ServiceError};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use std::sync::Arc;
use uuid::Uuid;

gen_service_impl! {
    struct RebookingReconciliationServiceImpl: RebookingReconciliationService = RebookingReconciliationServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction, Context = Self::Context> = extra_hours_service,
        RebookingBatchService: RebookingBatchService<Transaction = Self::Transaction, Context = Self::Context> = rebooking_batch_service,
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        VoluntaryStatsService: VoluntaryStatsService<Transaction = Self::Transaction, Context = Self::Context> = voluntary_stats_service,
        WeekStatusService: WeekStatusService<Transaction = Self::Transaction, Context = Self::Context> = week_status_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        ToggleService: ToggleService<Transaction = Self::Transaction, Context = Self::Context> = toggle_service,
        ClockService: ClockService = clock_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

/// Kleinster buchbarer Uebertrag in Stunden (WR-02). Alles darunter ist
/// f32-Rauschen und wird wie „kein Ueberschuss" behandelt.
///
/// Warum eine Schwelle und nicht `> 0.0`: `balance` ist eine f32-Summe ueber
/// Shiftplan-, ExtraHours- und Carryover-Beitraege. Nach einem exakt
/// ausgleichenden Uebertrag landet sie praktisch nie auf bit-exakt `0.0`,
/// sondern auf einem Residuum wie `-1.2e-6`. Der Dry-Run des Backfill
/// verschaerft das noch: seine `dry_run_consumed`-Buchfuehrung (CR-02) rechnet
/// den vorgemerkten Uebertrag auf die gelesene Balance an und erzeugt dabei
/// dasselbe Residuum ohne jeden Datenbank-Roundtrip.
///
/// Ein strikter `> 0.0`-Guard laesst den Write-Pfad auf so ein Residuum
/// anlaufen. Teuer ist dabei nicht das `-0.0000012 / +0.0000012`-Paar
/// (Datenrauschen), sondern der Batch-Claim: er belegt den UNIQUE-Slot
/// `(person, iso_year, iso_week)`, sodass eine spaetere legitime manuelle
/// Umbuchung oder ein HR-`suggest` fuer dieselbe Woche in
/// `EntityAlreadyExists` (→ HTTP 409) laeuft, obwohl fachlich nie etwas
/// gebucht wurde.
///
/// Pendant auf dem HR-Pfad: `rebook_manual` validiert `hours <= 0.0` hart.
pub(crate) const MIN_BOOKABLE_HOURS: f32 = 0.01;

/// Reine, testbare Ueberschuss-Formel (F4 core, REB-AUTO-01 + REB-AUTO-06):
///
/// ```text
/// week_excess = min( max(ist - (soll + committed_voluntary), 0), max(-balance, 0) )
/// ```
///
/// Der erste Term ist der freiwillige Wochen-Ueberschuss (REB-AUTO-01): was ueber
/// die Freiwilligen-Zusage hinaus gearbeitet wurde, geklemmt auf `>= 0`.
///
/// Der zweite Term ist der **Deckel** (REB-AUTO-06, Phase 57): der Headroom des
/// bezahlten Stundenkontos. In Kontobegriffen — die Automatik gleicht das
/// bezahlte Stundenkonto aus, hebt es aber **nie ueber 0**. Steht die Balance
/// bereits auf 0 oder im Plus, gibt es nichts auszugleichen und es wird nichts
/// uebertragen (Invariante I-4), auch wenn ein freiwilliger Ueberschuss vorliegt.
///
/// **Abgrenzung zum HR-Pfad (D-57-01, Invariante I-5):**
/// `crate::reporting::proposed_hours(balance, voluntary_ist) = min(|balance|, ist)`
/// bekommt bewusst KEINEN solchen Deckel — der HR-Vorschlag darf die
/// Freiwilligen-Zusage unterschreiten, weil ein Mensch die Entscheidung traegt.
/// Die Automatik darf das nicht und gleicht nur den unstrittigen Teil aus.
///
/// Nicht-endliche Eingaben (NaN/Inf) ergeben in allen vier Parametern `0.0`
/// (defensiver Skip statt Panik / Muell-Buchung, T-57-07).
pub(crate) fn week_excess(ist: f32, soll: f32, committed_voluntary: f32, balance: f32) -> f32 {
    if !ist.is_finite()
        || !soll.is_finite()
        || !committed_voluntary.is_finite()
        || !balance.is_finite()
    {
        return 0.0;
    }
    let unconstrained = ist - (soll + committed_voluntary);
    let unconstrained = if unconstrained > 0.0 { unconstrained } else { 0.0 };
    let headroom = if -balance > 0.0 { -balance } else { 0.0 };
    unconstrained.min(headroom)
}

/// Reine Helper-fn (testbar, keine Deps): leitet aus der Richtung die
/// (Quell-Kategorie, Ziel-Kategorie) fuer das Pair ab. Das Pair schreibt
/// immer `-hours` in die Quell-Kategorie und `+hours` in die Ziel-Kategorie.
fn pair_categories(direction: RebookingDirection) -> (ExtraHoursCategory, ExtraHoursCategory) {
    match direction {
        // Standard-Ausgleich: freiwillige Stunden werden zu ExtraWork.
        RebookingDirection::VolunteerToExtra => {
            (ExtraHoursCategory::VolunteerWork, ExtraHoursCategory::ExtraWork)
        }
        // Korrektur: ExtraWork wird zu freiwilligen Stunden.
        RebookingDirection::ExtraToVolunteer => {
            (ExtraHoursCategory::ExtraWork, ExtraHoursCategory::VolunteerWork)
        }
    }
}

/// ISO-(year, week) → `PrimitiveDateTime` am Montag der Woche (Mitternacht).
/// Die konkrete Uhrzeit ist fuer die Wochen-Aggregation irrelevant; wichtig ist
/// nur, dass `date_time` in die richtige ISO-Woche faellt.
fn week_anchor_date_time(iso_year: u32, iso_week: u8) -> Result<time::PrimitiveDateTime, ServiceError> {
    let date = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Monday)
        .map_err(|_| ServiceError::InternalError)?
        .to_date();
    Ok(time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT))
}

impl<Deps: RebookingReconciliationServiceDeps> RebookingReconciliationServiceImpl<Deps> {
    /// Einziger interner Pair-Write-Path (Pitfall 15): schreibt das
    /// -N (Quell-Kategorie) / +N (Ziel-Kategorie) ExtraHours-Pair mit
    /// `source=Rebooking` in der uebergebenen Transaktion und liefert die beiden
    /// erzeugten Row-ids `(out_id, in_id)` zurueck. Ruft KEINEN commit — die
    /// Klammer besitzt der Aufrufer (rebook_manual / approve).
    async fn rebook(
        &self,
        sales_person_id: Uuid,
        direction: RebookingDirection,
        hours: f32,
        iso_year: u32,
        iso_week: u8,
        tx: Deps::Transaction,
    ) -> Result<(Uuid, Uuid), ServiceError> {
        let (source_category, target_category) = pair_categories(direction);
        let date_time = week_anchor_date_time(iso_year, iso_week)?;

        let out_row = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount: -hours,
            category: source_category,
            description: Arc::from("rebooking (out)"),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
            source: ExtraHoursSource::Rebooking,
        };
        let out = self
            .extra_hours_service
            .create(&out_row, Authentication::Full, Some(tx.clone()))
            .await?;

        let in_row = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount: hours,
            category: target_category,
            description: Arc::from("rebooking (in)"),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
            source: ExtraHoursSource::Rebooking,
        };
        let inn = self
            .extra_hours_service
            .create(&in_row, Authentication::Full, Some(tx.clone()))
            .await?;

        Ok((out.id, inn.id))
    }

    /// Baut eine `RebookingBatchEntity` fuer den F4-Auto-Cron/Backfill (kein
    /// Entry — der Batch dient als Slot-Claim + Audit; das Pair haengt indirekt
    /// via UNIQUE-Slot). `id`/`created`/`version` werden vom Basic-Service
    /// defensiv gesetzt.
    fn make_batch(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        kind: RebookingBatchKind,
        state: RebookingBatchState,
    ) -> RebookingBatchEntity {
        RebookingBatchEntity {
            id: Uuid::nil(),
            sales_person_id,
            iso_year,
            iso_week,
            kind,
            state,
            created: time::PrimitiveDateTime::MIN,
            approved: None,
            approved_by: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    /// Liest die Wochen-Freiwilligen-Kennzahlen `(ist, soll, committed)` fuer die
    /// ISO-Woche (Range Mo..=So) aus `VoluntaryStats`. `committed == soll ==
    /// soll_total` (committed-voluntary-target); `ist == ist_total`. `None`-Felder
    /// (z. B. Nicht-HR / Nicht-Freiwilliger) werden defensiv auf `0.0` gemappt.
    async fn week_voluntary_figures(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        tx: Deps::Transaction,
    ) -> Result<(f32, f32, f32), ServiceError> {
        let from_date = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Monday)
            .map_err(|_| ServiceError::InternalError)?;
        let to_date = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
            .map_err(|_| ServiceError::InternalError)?;

        let stats = self
            .voluntary_stats_service
            .get_voluntary_stats(
                sales_person_id,
                from_date,
                to_date,
                Authentication::Full,
                Some(tx),
            )
            .await?;
        let ist = stats.ist_total.unwrap_or(0.0);
        let soll = stats.soll_total.unwrap_or(0.0);
        // committed == soll_total (der committed-voluntary-target der Woche).
        Ok((ist, soll, soll))
    }
}

/// Vorwoche einer `ShiftyWeek` mit korrektem ISO-Jahresgrenzfall (nie naives
/// `week - 1`): in KW 1 springt sie auf die letzte Woche des Vorjahres.
fn previous_week(week: ShiftyWeek) -> ShiftyWeek {
    if week.week <= 1 {
        let prev_year = week.year - 1;
        ShiftyWeek::new(prev_year, time::util::weeks_in_year(prev_year as i32))
    } else {
        ShiftyWeek::new(week.year, week.week - 1)
    }
}

#[async_trait]
impl<Deps: RebookingReconciliationServiceDeps> RebookingReconciliationService
    for RebookingReconciliationServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn rebook_manual(
        &self,
        sales_person_id: Uuid,
        direction: RebookingDirection,
        hours: f32,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // HR-Gate ZUERST (T-55-05), vor jedem Datenabruf / jeder Tx.
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        // WR-02: serverseitige Validierung des Betrags (der FE-Modal-Guard ist
        // nicht autoritativ). hours<=0 wuerde ein 0-/vorzeichen-invertiertes Pair
        // erzeugen — hart ablehnen (inkl. NaN/Inf), bevor eine Tx/Row entsteht.
        if hours <= 0.0 || !hours.is_finite() {
            return Err(ServiceError::ValidationError(Arc::from([
                service::ValidationFailureItem::InvalidValue("hours must be > 0".into()),
            ])));
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // ① + ② Pair-ExtraHours via single write-path.
        let (out_id, in_id) = self
            .rebook(sales_person_id, direction, hours, iso_year, iso_week, tx.clone())
            .await?;

        // ③ Manual-Batch (state=Approved) + Entry mit den Pair-ids. UNIQUE-Slot-
        // Kollision propagiert als EntityAlreadyExists (→ 409).
        let batch = RebookingBatchEntity {
            id: Uuid::nil(),
            sales_person_id,
            iso_year,
            iso_week,
            kind: RebookingBatchKind::Manual,
            state: RebookingBatchState::Approved,
            created: time::PrimitiveDateTime::MIN,
            approved: None,
            approved_by: None,
            deleted: None,
            version: Uuid::nil(),
        };
        let entry = RebookingBatchEntryEntity {
            id: Uuid::nil(),
            batch_id: Uuid::nil(),
            sales_person_id,
            hours,
            balance_before: 0.0,
            voluntary_actual: 0.0,
            voluntary_committed: 0.0,
            extra_hours_out_id: Some(out_id),
            extra_hours_in_id: Some(in_id),
            created: time::PrimitiveDateTime::MIN,
            deleted: None,
            version: Uuid::nil(),
        };
        self.rebooking_batch_service
            .create(&batch, &[entry], Authentication::Full, Some(tx.clone()))
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn suggest(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError> {
        // HR-Gate ZUERST (T-55-05).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Modell B: existiert bereits ein offener pending-Batch fuer den Slot?
        // → idempotent dieselbe id zurueck; sonst neuen Claim.
        let pending_rebooking_id = match self
            .rebooking_batch_service
            .find_pending_for_sales_person(
                sales_person_id,
                iso_year,
                iso_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?
        {
            Some(existing) => existing.id,
            None => {
                // Claim: create_pending_suggestion. Ein entschiedener Batch im
                // Slot propagiert EntityAlreadyExists (→ 409, D-55-B).
                let claimed = self
                    .rebooking_batch_service
                    .create_pending_suggestion(
                        sales_person_id,
                        iso_year,
                        iso_week,
                        Authentication::Full,
                        Some(tx.clone()),
                    )
                    .await?;
                claimed.id
            }
        };

        // IST backend-computed: balance aus dem Range-Report, voluntary_ist aus
        // VoluntaryStats. Range = die betroffene ISO-Woche (Mo..=So).
        let from_date = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Monday)
            .map_err(|_| ServiceError::InternalError)?;
        let to_date = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
            .map_err(|_| ServiceError::InternalError)?;

        let report = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person_id,
                from_date,
                to_date,
                false,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance = report.balance_hours;

        let stats = self
            .voluntary_stats_service
            .get_voluntary_stats(
                sales_person_id,
                from_date,
                to_date,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let voluntary_ist = stats.ist_total.unwrap_or(0.0);

        self.transaction_dao.commit(tx).await?;

        // DANN backend-computed (kein FE-Arithmetik-Vorgriff, HR-ALERT-02):
        // proposed_hours = min(|balance|, voluntary_ist).
        let proposed = crate::reporting::proposed_hours(balance, voluntary_ist);
        Ok(RebookingSuggestion {
            pending_rebooking_id,
            proposed_hours: proposed,
            balance,
            voluntary_ist,
            balance_after: balance + proposed,
            voluntary_ist_after: voluntary_ist - proposed,
        })
    }

    async fn approve(
        &self,
        rebooking_batch_id: Uuid,
        _hours: f32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // WR-04: Approver-Principal VOR dem HR-Gate lesen (check_permission
        // konsumiert den Context). Full-Auth (kein Nutzer-Context) liefert None.
        let approved_by = self
            .permission_service
            .current_user_id(context.clone())
            .await?;

        // HR-Gate ZUERST (T-55-05).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Slot-Koordinaten aus dem Batch lesen (fuer die Pair-Woche).
        let batch = self
            .rebooking_batch_service
            .find_by_id(rebooking_batch_id, Authentication::Full, Some(tx.clone()))
            .await?
            .ok_or(ServiceError::EntityNotFound(rebooking_batch_id))?;

        // CR-02 (Fat-Backend/Modell-B): der genehmigte Betrag ist backend-
        // authoritativ. Der Client-Wert (`_hours`) wird VERWORFEN; stattdessen
        // recomputet approve `proposed_hours = min(|balance|, voluntary_ist)` fuer
        // die Batch-Woche mit exakt derselben Berechnung wie `suggest`
        // (Wochen-Range Mo..=So). So kann HR weder mehr als die freiwilligen
        // Ist-Stunden noch negative Stunden buchen.
        let from_date = ShiftyDate::new(batch.iso_year, batch.iso_week, DayOfWeek::Monday)
            .map_err(|_| ServiceError::InternalError)?;
        let to_date = ShiftyDate::new(batch.iso_year, batch.iso_week, DayOfWeek::Sunday)
            .map_err(|_| ServiceError::InternalError)?;

        let report = self
            .reporting_service
            .get_report_for_employee_range(
                &batch.sales_person_id,
                from_date,
                to_date,
                false,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance = report.balance_hours;

        let stats = self
            .voluntary_stats_service
            .get_voluntary_stats(
                batch.sales_person_id,
                from_date,
                to_date,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let voluntary_ist = stats.ist_total.unwrap_or(0.0);
        let voluntary_committed = stats.soll_total.unwrap_or(0.0);

        let proposed = crate::reporting::proposed_hours(balance, voluntary_ist);
        if proposed <= 0.0 || !proposed.is_finite() {
            // Nichts umzubuchen (kein Defizit oder keine freiwilligen Ist-Stunden)
            // → Validierungsfehler, kein state-Update, kein Pair.
            return Err(ServiceError::ValidationError(Arc::from([
                service::ValidationFailureItem::InvalidValue(
                    "nothing to rebook (proposed hours <= 0)".into(),
                ),
            ])));
        }

        // state-conditional UPDATE ZUERST (Pitfall 12): nur bei rows_affected==1
        // (kein EntityAlreadyExists) folgen die Pair-Writes. So entsteht das Pair
        // genau einmal; der Double-Approve-Verlierer bekommt Conflict (→ 409).
        // WR-04: approved_by traegt den anfragenden HR-Principal (Audit).
        self.rebooking_batch_service
            .update_state(
                rebooking_batch_id,
                RebookingBatchState::Approved,
                approved_by,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        // F5-Standard-Richtung: VolunteerToExtra (gedeckelter Mitarbeiter im
        // Minus bucht freiwillige Stunden auf ExtraWork um). Single write-path,
        // backend-authoritativer Betrag `proposed`.
        let (out_id, in_id) = self
            .rebook(
                batch.sales_person_id,
                RebookingDirection::VolunteerToExtra,
                proposed,
                batch.iso_year,
                batch.iso_week,
                tx.clone(),
            )
            .await?;

        // CR-01: Audit-Kette schliessen — Entry an den BESTEHENDEN pending-Batch
        // haengen (nicht via create/neuen Batch), mit den Pair-ids + den
        // Audit-Snapshots (symmetrisch zu rebook_manual, aber hier backend-befuellt).
        let entry = RebookingBatchEntryEntity {
            id: Uuid::nil(),
            batch_id: rebooking_batch_id,
            sales_person_id: batch.sales_person_id,
            hours: proposed,
            balance_before: balance,
            voluntary_actual: voluntary_ist,
            voluntary_committed,
            extra_hours_out_id: Some(out_id),
            extra_hours_in_id: Some(in_id),
            created: time::PrimitiveDateTime::MIN,
            deleted: None,
            version: Uuid::nil(),
        };
        self.rebooking_batch_service
            .create_entries_for_batch(
                rebooking_batch_id,
                &[entry],
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn reject(
        &self,
        rebooking_batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // HR-Gate ZUERST (T-55-05).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // reject: nur state-Transition, KEINE Pair-Rows (D-55-B).
        self.rebooking_batch_service
            .update_state(
                rebooking_batch_id,
                RebookingBatchState::Rejected,
                None,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn run_for_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        kind: RebookingBatchKind,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RunForWeekOutcome, ServiceError> {
        // F4-Self-Guard (D-06 write side): Stichtag-Toggle EINMAL lesen und
        // pruefen, ob die Woche >= active_from liegt. Toggle aus / vor Stichtag →
        // ToggleOff, kein Datenabruf, kein Write. `read_active_from` toleriert
        // Unauthorized (Cron-Kontext ohne User) als "Feature aus".
        let active_from = crate::voluntary_rebooking_gate::read_active_from(
            self.toggle_service.as_ref(),
            context.clone(),
        )
        .await?;
        if !crate::voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from) {
            return Ok(RunForWeekOutcome::Skipped {
                reason: SkipReason::ToggleOff,
            });
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // D-04: gesperrte Woche → SkippedLocked-Audit-Batch (idempotent) + Skip.
        let week_status = self
            .week_status_service
            .get_week_status(iso_year, iso_week, Authentication::Full, Some(tx.clone()))
            .await?;
        if week_status == WeekStatus::Locked {
            // Audit-Row: kind wie uebergeben, state=SkippedLocked, kein Entry.
            // Ein UNIQUE-Konflikt (bereits ein Batch im Slot) wird ignoriert
            // (`.ok()`), damit ein zweiter Locked-Lauf ein sauberes No-op bleibt.
            let audit_batch = self.make_batch(
                sales_person_id,
                iso_year,
                iso_week,
                kind,
                RebookingBatchState::SkippedLocked,
            );
            let _ = self
                .rebooking_batch_service
                .create(&audit_batch, &[], Authentication::Full, Some(tx.clone()))
                .await
                .ok();
            self.transaction_dao.commit(tx).await?;
            return Ok(RunForWeekOutcome::Skipped {
                reason: SkipReason::WeekLocked,
            });
        }

        // Balance-Deckel (REB-AUTO-06, D-57-07): der Uebertrag darf das bezahlte
        // Stundenkonto nie ueber 0 heben. Gebraucht wird dafuer die KUMULIERTE
        // Jahres-Balance bis einschliesslich der verarbeiteten Woche INKLUSIVE
        // Vorjahres-Carryover — nicht die Balance zum Laufzeitpunkt, sonst wuerde
        // ein Backfill ueber mehrere historische Wochen denselben Spielraum
        // mehrfach vergeben.
        //
        // ACHTUNG (Pitfall 3): das ist ein strukturell ANDERER Range als in
        // `suggest`/`approve` (isolierte ISO-Woche Mo..=So, include_carryover =
        // false). Dieses Muster darf hier NICHT kopiert werden.
        //
        // Der Read steht bewusst NACH dem Toggle- und dem Locked-Gate, damit ein
        // ausgeschalteter Cron / eine gesperrte Woche ohne Report-Roundtrip
        // zurueckkehrt.
        //
        // ACHTUNG (CR-01): der Range-Start ist an ISO-Koordinaten verankert, NICHT
        // an `ShiftyDate::first_day_in_year(iso_year)`. Letzteres geht ueber
        // `from_ymd(iso_year, 1, 1)` und speichert ein ISO-Tripel — faellt der
        // 1. Januar auf Fr/Sa/So (2027, 2028, 2021, 2022, 2023), liegt er im
        // VORHERIGEN ISO-Jahr. `get_report_for_employee_range` liest daraus
        // `from_date.year() - 1` fuer den Carryover und iteriert `hours_per_week`
        // ab `from_date.as_shifty_week()`; ein Kalender-Anker wuerde also das
        // falsche Carryover-Jahr laden und eine volle Vorjahres-Woche in die
        // „year-to-date"-Balance ziehen. Der Wert steuert `max(-balance, 0)` und
        // damit direkt den geschriebenen Betrag.
        let year_start = ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)
            .map_err(|_| ServiceError::InternalError)?;
        let week_end = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
            .map_err(|_| ServiceError::InternalError)?;
        let report = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person_id,
                year_start,
                week_end,
                true, // include_carryover — D-57-07
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance = report.balance_hours;

        // Ueberschuss = Ist - committed_voluntary (beide fuer die Wochen-Range
        // Mo..=So), gedeckelt auf den Headroom der Balance. `committed` ==
        // VoluntaryStats::soll_total (committed-voluntary-target).
        let (ist, _soll, committed) = self
            .week_voluntary_figures(sales_person_id, iso_year, iso_week, tx.clone())
            .await?;
        let excess = week_excess(ist, 0.0, committed, balance);
        // WR-02: Mindest-Buchungsgroesse statt `<= 0.0`. Ein f32-Residuum wie
        // `1.2e-6` wuerde sonst ein Micro-Pair schreiben UND den UNIQUE-Slot
        // der Woche verbrennen (siehe MIN_BOOKABLE_HOURS).
        if excess < MIN_BOOKABLE_HOURS {
            self.transaction_dao.commit(tx).await?;
            return Ok(RunForWeekOutcome::Skipped {
                reason: SkipReason::NoExcess,
            });
        }

        // Approved-Batch ZUERST (er claimt den UNIQUE-Slot). Konflikt (frueherer
        // Cron / Manual / pending HR-Suggestion → Claim-on-Suggest, D-02) →
        // AlreadyProcessed, KEIN Pair (REB-AUTO-03 Idempotenz).
        let batch = self.make_batch(
            sales_person_id,
            iso_year,
            iso_week,
            kind,
            RebookingBatchState::Approved,
        );
        match self
            .rebooking_batch_service
            .create(&batch, &[], Authentication::Full, Some(tx.clone()))
            .await
        {
            Ok(_) => {}
            Err(ServiceError::EntityAlreadyExists(_)) => {
                self.transaction_dao.commit(tx).await?;
                return Ok(RunForWeekOutcome::Skipped {
                    reason: SkipReason::AlreadyProcessed,
                });
            }
            Err(e) => return Err(e),
        }

        // Single Pair-Write-Path (Pitfall 15): VolunteerToExtra ueber den
        // Ueberschuss.
        self.rebook(
            sales_person_id,
            RebookingDirection::VolunteerToExtra,
            excess,
            iso_year,
            iso_week,
            tx.clone(),
        )
        .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(RunForWeekOutcome::Booked { hours: excess })
    }

    async fn backfill_range(
        &self,
        from: ShiftyWeek,
        dry_run: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<BackfillPreviewItem>, ServiceError> {
        // HR-Admin-Gate ZUERST (T-56-04) — vor jedem Datenabruf.
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // Obergrenze: die letzte abgeschlossene Vorwoche (nie die laufende Woche;
        // T-56-05 DoS-Bound). Aufloesung ueber ShiftyDate::from_date (nie naives
        // year()+week()), Vorwoche via ShiftyWeek-Arithmetik.
        let today = ShiftyDate::from_date(self.clock_service.date_now());
        let upper = previous_week(today.as_shifty_week());

        let mut items: Vec<BackfillPreviewItem> = Vec::new();

        // Toggle einmal lesen (F4-Self-Guard, D-06). Der Backfill teilt exakt die
        // Cutoff-Semantik von run_for_week.
        let active_from = crate::voluntary_rebooking_gate::read_active_from(
            self.toggle_service.as_ref(),
            context.clone(),
        )
        .await?;

        // Fuer die reine Vorschau (dry_run) darf keine Write-Tx geoeffnet werden;
        // wir nutzen ein Option<Transaction> nur fuer die Read-Aufrufe.
        let read_tx = self.transaction_dao.use_transaction(tx).await?;

        // CR-02 / Invariante I-7: im Dry-Run muss der bereits *vorgemerkte*
        // Headroom pro Person ueber die Wochen mitgefuehrt werden. Der Apply-Pfad
        // verbraucht ihn implizit — jedes geschriebene Paar hebt die Balance, und
        // SQLite sieht die Rows innerhalb der geteilten `read_tx`. Der Dry-Run
        // schreibt nichts, saehe also in JEDER Woche denselben unverbrauchten
        // Headroom und meldete eine Summe, die ueber dem liegt, was ein
        // Apply-Lauf tatsaechlich bucht (Balance −10, je 4 h Ueberschuss in drei
        // Wochen: Preview 12 statt 10). Der Fehler waechst linear mit der Anzahl
        // der Backfill-Wochen. Die Map bleibt im Apply-Modus leer.
        let mut dry_run_consumed: std::collections::HashMap<Uuid, f32> =
            std::collections::HashMap::new();

        for week in from.iter_until(&upper) {
            let (iso_year, iso_week) = (week.year, week.week);

            // Kandidaten: alle gedeckelten SalesPersons mit gueltigem Vertrag in
            // dieser Woche.
            let contracts = self
                .employee_work_details_service
                .all_for_week(iso_week, iso_year, Authentication::Full, Some(read_tx.clone()))
                .await?;

            for wd in contracts.iter() {
                if !wd.cap_planned_hours_to_expected {
                    continue;
                }
                let sales_person_id = wd.sales_person_id;

                // Derselbe Balance-Deckel wie in `run_for_week` (REB-AUTO-06),
                // nur mit der bereits offenen `read_tx`. Ein Platzhalter (0.0
                // oder ein ungeprueftes Feld) ist hier VERBOTEN: er kompiliert
                // klaglos und deckelt jeden Ueberschuss auf 0, wodurch der
                // Dry-Run-Preview von dem abwiche, was der spaetere Apply-Lauf
                // ueber `run_for_week` tatsaechlich bucht. Der Doc-Kommentar auf
                // `BackfillPreviewItem.excess` verspricht „der Ueberschuss, der
                // gebucht wuerde" — das ist die Invariante I-7.
                //
                // Chronologische Korrektheit ohne eigene Buchfuehrung: der
                // Wochen-Loop teilt EINE `read_tx`, und SQLite sieht innerhalb
                // derselben Transaktion die in Woche N-1 geschriebenen Rows.
                //
                // Der Range-Start ist wie in `run_for_week` an ISO-Koordinaten
                // verankert (CR-01) — `first_day_in_year` waere ein Kalender-Anker
                // und faellt in Jahren mit Fr/Sa/So-Neujahr ins vorherige ISO-Jahr.
                let year_start = ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)
                    .map_err(|_| ServiceError::InternalError)?;
                let week_end = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
                    .map_err(|_| ServiceError::InternalError)?;
                let report = self
                    .reporting_service
                    .get_report_for_employee_range(
                        &sales_person_id,
                        year_start,
                        week_end,
                        true, // include_carryover — D-57-07
                        Authentication::Full,
                        Some(read_tx.clone()),
                    )
                    .await?;
                // Im Dry-Run schlaegt sich der in frueheren Wochen vorgemerkte
                // Uebertrag noch nicht in `report.balance_hours` nieder (es gibt
                // keine Rows) — deshalb hier explizit anrechnen. Ein gebuchtes
                // Paar ueber `h` hebt die Balance um `h`, also addiert sich der
                // verbrauchte Headroom auf die gelesene Balance (CR-02).
                let balance = report.balance_hours
                    + dry_run_consumed
                        .get(&sales_person_id)
                        .copied()
                        .unwrap_or(0.0);

                let (ist, soll, committed) = self
                    .week_voluntary_figures(sales_person_id, iso_year, iso_week, read_tx.clone())
                    .await?;
                let excess = week_excess(ist, 0.0, committed, balance);

                // Cutoff-Gate: vor Stichtag → ToggleOff-Skip (kein Write).
                let toggle_on =
                    crate::voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from);

                let mut item = BackfillPreviewItem {
                    sales_person_id,
                    iso_year,
                    iso_week,
                    voluntary_ist: ist,
                    voluntary_soll: soll,
                    committed_voluntary: committed,
                    excess,
                    // WR-02: dieselbe Mindest-Buchungsgroesse wie im Schreibpfad —
                    // sonst verspricht der Preview eine Buchung, die
                    // `run_for_week` verweigert (Invariante I-7).
                    would_book: excess >= MIN_BOOKABLE_HOURS && toggle_on,
                    skipped: true,
                    skip_reason: None,
                };

                if !toggle_on {
                    item.skip_reason = Some(SkipReason::ToggleOff);
                    items.push(item);
                    continue;
                }
                // WR-02: der Guard steht VOR dem `dry_run`-Zweig und deckt damit
                // beide Modi ab. Im Dry-Run ist er sogar dringlicher: die
                // `dry_run_consumed`-Buchfuehrung (CR-02) erzeugt das Residuum
                // selbst, sobald der Headroom exakt aufgebraucht ist.
                if excess < MIN_BOOKABLE_HOURS {
                    item.skip_reason = Some(SkipReason::NoExcess);
                    items.push(item);
                    continue;
                }

                if dry_run {
                    // Vorschau: als buchbar markieren, aber NICHT schreiben.
                    // Der Headroom gilt trotzdem als vergeben — sonst wuerde die
                    // naechste Woche derselben Person ihn erneut ausschoepfen
                    // (CR-02, Invariante I-7).
                    *dry_run_consumed.entry(sales_person_id).or_insert(0.0) += excess;
                    item.skipped = false;
                    items.push(item);
                    continue;
                }

                // Apply: Single-Write-Path via run_for_week (kind=AutoCronBackfill).
                let outcome = self
                    .run_for_week(
                        sales_person_id,
                        iso_year,
                        iso_week,
                        RebookingBatchKind::AutoCronBackfill,
                        context.clone(),
                        Some(read_tx.clone()),
                    )
                    .await?;
                match outcome {
                    RunForWeekOutcome::Booked { .. } => {
                        item.skipped = false;
                    }
                    RunForWeekOutcome::Skipped { reason } => {
                        item.skip_reason = Some(reason);
                        item.would_book = false;
                    }
                }
                items.push(item);
            }
        }

        // dry_run committet nur die Read-Tx (kein Write erfolgt); apply committet
        // die geschriebenen Batches. In beiden Faellen ein sauberer Commit.
        self.transaction_dao.commit(read_tx).await?;
        Ok(items)
    }
}
