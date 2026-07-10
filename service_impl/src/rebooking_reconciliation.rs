//! Phase 55 Plan 01 (F3 + F5): Business-Logic
//! `RebookingReconciliationService` — orchestriert Pair-ExtraHours,
//! Batch/Entry-Persistenz und Reporting-Snapshot in EINER Transaktion.
//!
//! Tier-Klassifizierung: **Business-Logic-Service** (MEMORY
//! `feedback_service_tier_convention`). Konsumiert:
//! - `ExtraHoursService` (Basic) — schreibt beide Pair-Rows mit
//!   `ExtraHoursSource::Rebooking` (VOL-ACCT-03).
//! - `RebookingBatchService` (Basic) — persistiert Batch + Entry und
//!   fuehrt state-conditional UPDATE fuer Approve/Reject (HR-ALERT-03).
//! - `ReportingService` (Business-Logic) — liefert IST/DANN-Snapshot
//!   (`balance_hours`, `volunteer_hours`) fuer Audit + Modal.
//! - `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`.
//!
//! **Kein Zyklus:** `RebookingBatchService` bleibt Basic (kennt diesen
//! Service NICHT), `ReportingService` konsumiert diesen Service NICHT.
//!
//! **Permissionsmodell:** HR-only. Jede public Trait-Methode gatet als
//! erste await-Operation `PermissionService::check_permission(HR_PRIVILEGE,
//! ...)` (T-55-02).

use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    rebooking_batch::{RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchKind, RebookingBatchState},
    TransactionDao,
};
use service::{
    clock::ClockService,
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService, ExtraHoursSource},
    permission::{Authentication, HR_PRIVILEGE},
    rebooking_batch::RebookingBatchService,
    rebooking_reconciliation::{
        proposed_rebooking_hours, RebookingDirection, RebookingReconciliationService,
        RebookingSuggestion,
    },
    reporting::ReportingService,
    uuid_service::UuidService,
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::{DayOfWeek, ShiftyWeek};
use uuid::Uuid;

use crate::gen_service_impl;

const REBOOKING_RECONCILIATION_PROCESS: &str = "rebooking-reconciliation-service";

gen_service_impl! {
    struct RebookingReconciliationServiceImpl:
        RebookingReconciliationService = RebookingReconciliationServiceDeps {
        ExtraHoursService: ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
        RebookingBatchService: RebookingBatchService<Context = Self::Context, Transaction = Self::Transaction> = rebooking_batch_service,
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: RebookingReconciliationServiceDeps> RebookingReconciliationServiceImpl<Deps> {
    /// Baut die zwei Pair-`ExtraHours`-Payloads (`-N` / `+N`) fuer eine
    /// Rebooking-Buchung. Beide Rows tragen `ExtraHoursSource::Rebooking`
    /// (VOL-ACCT-03) — damit greift der Filter in
    /// `service_impl::reporting` und die Rebooking-Neutralitaet in
    /// Read-Aggregaten haelt.
    fn build_pair_payloads(
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        direction: RebookingDirection,
        hours: f32,
    ) -> (ExtraHours, ExtraHours) {
        // Datumsanker der Woche: Montag der ISO-Woche 00:00 (D-55-05 —
        // HR-gewaehlte Woche). ShiftyWeek::new normalisiert Ueberlauf
        // (KW 53 wo es keine gibt → naechstes Jahr KW 1).
        let monday = ShiftyWeek::new(iso_year, iso_week).as_date(DayOfWeek::Monday);
        let midnight = time::Time::from_hms(0, 0, 0).expect("00:00:00 is a valid time");
        let date_time = time::PrimitiveDateTime::new(monday.to_date(), midnight);

        let (out_category, in_category) = match direction {
            RebookingDirection::VolunteerToExtra => {
                (ExtraHoursCategory::VolunteerWork, ExtraHoursCategory::ExtraWork)
            }
            RebookingDirection::ExtraToVolunteer => {
                (ExtraHoursCategory::ExtraWork, ExtraHoursCategory::VolunteerWork)
            }
        };

        let out_row = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount: -hours,
            category: out_category,
            description: Arc::<str>::from("Rebooking-Pair (out)"),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
            source: ExtraHoursSource::Rebooking,
        };
        let in_row = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount: hours,
            category: in_category,
            description: Arc::<str>::from("Rebooking-Pair (in)"),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
            source: ExtraHoursSource::Rebooking,
        };
        (out_row, in_row)
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
        iso_year: u32,
        iso_week: u8,
        direction: RebookingDirection,
        hours: f32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError> {
        // 1. HR-Gate als erste await-Operation (T-55-02).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // 2. Sanity: nur positive Mengen. Richtung entscheidet Vorzeichen.
        if hours <= 0.0 || !hours.is_finite() {
            return Err(ServiceError::ValidationError(Arc::from(vec![
                ValidationFailureItem::InvalidValue(Arc::<str>::from("hours")),
            ])));
        }

        // 3. EINE Transaktion — Atomarität (REB-MANUAL-01).
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // 4. Reporting-Snapshot fuer Audit (balance_before, voluntary_actual).
        //    voluntary_committed bleibt beim Manual-Pfad 0.0 (Audit-nur;
        //    Soll-Snapshot ist fuer den Manual-Pfad nicht erforderlich, der
        //    Suggest-Pfad in suggest_for_week / hydrate_pending_to_suggestion
        //    baut die Soll-Werte fuer die Anzeige).
        let report = self
            .reporting_service
            .get_report_for_employee(
                &sales_person_id,
                iso_year,
                iso_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance_before = report.balance_hours;
        let voluntary_actual = report.volunteer_hours;
        let voluntary_committed = 0.0_f32;

        // 5. Pair-ExtraHours schreiben (Marker Rebooking).
        let (out_payload, in_payload) =
            Self::build_pair_payloads(sales_person_id, iso_year, iso_week, direction, hours);
        let out_row = self
            .extra_hours_service
            .create(&out_payload, context.clone(), Some(tx.clone()))
            .await?;
        let in_row = self
            .extra_hours_service
            .create(&in_payload, context.clone(), Some(tx.clone()))
            .await?;

        // 6. Batch + Entry mit FKs auf beide ExtraHours-Rows.
        let batch_payload = RebookingBatchEntity {
            id: Uuid::nil(),
            sales_person_id,
            iso_year,
            iso_week,
            kind: RebookingBatchKind::Manual,
            state: RebookingBatchState::Approved,
            created: time::PrimitiveDateTime::MIN,
            approved: Some(self.clock_service.date_time_now()),
            approved_by: None,
            deleted: None,
            version: Uuid::nil(),
        };
        let entry_payload = RebookingBatchEntryEntity {
            id: Uuid::nil(),
            batch_id: Uuid::nil(),
            sales_person_id,
            hours,
            balance_before,
            voluntary_actual,
            voluntary_committed,
            extra_hours_out_id: Some(out_row.id),
            extra_hours_in_id: Some(in_row.id),
            created: time::PrimitiveDateTime::MIN,
            deleted: None,
            version: Uuid::nil(),
        };
        let batch = self
            .rebooking_batch_service
            .create(
                &batch_payload,
                &[entry_payload],
                context.clone(),
                Some(tx.clone()),
            )
            .await?;

        // 7. Commit — Atomarität geschlossen.
        self.transaction_dao.commit(tx).await?;
        Ok(batch)
    }

    async fn suggest_for_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Reporting-Snapshot fuer Predicate + Anzeige.
        let report = self
            .reporting_service
            .get_report_for_employee(
                &sales_person_id,
                iso_year,
                iso_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance_before = report.balance_hours;
        let voluntary_ist_before = report.volunteer_hours;
        // Manual-Pfad-Analogie: Soll wird beim `hydrate_pending_to_suggestion`
        // vollstaendig aus EmployeeWorkDetails / voluntary_stats gebaut. Fuer
        // den Suggest-Persist-Pfad reicht der bereits berechnete
        // proposed_hours-Wert; das Soll fliesst per hydrate spaeter beim
        // Anzeigen ein. Hier persistieren wir den Snapshot mit
        // voluntary_committed=0.0 (Audit-nur — der spaetere Approve
        // ueberschreibt nicht, weil der Entry-Snapshot ohnehin write-once
        // ist; die Anzeige zieht die aktuellen Werte via hydrate).
        let voluntary_soll_before = 0.0_f32;
        let proposed_hours = proposed_rebooking_hours(balance_before, voluntary_ist_before);

        // Pending-Batch + Entry (NULL-FKs — Claim-on-Suggest, D-54-DM-01).
        let batch_payload = RebookingBatchEntity {
            id: Uuid::nil(),
            sales_person_id,
            iso_year,
            iso_week,
            kind: RebookingBatchKind::HrSuggestion,
            state: RebookingBatchState::Pending,
            created: time::PrimitiveDateTime::MIN,
            approved: None,
            approved_by: None,
            deleted: None,
            version: Uuid::nil(),
        };
        let entry_payload = RebookingBatchEntryEntity {
            id: Uuid::nil(),
            batch_id: Uuid::nil(),
            sales_person_id,
            hours: proposed_hours,
            balance_before,
            voluntary_actual: voluntary_ist_before,
            voluntary_committed: voluntary_soll_before,
            extra_hours_out_id: None,
            extra_hours_in_id: None,
            created: time::PrimitiveDateTime::MIN,
            deleted: None,
            version: Uuid::nil(),
        };
        let batch = self
            .rebooking_batch_service
            .create(
                &batch_payload,
                &[entry_payload],
                context.clone(),
                Some(tx.clone()),
            )
            .await?;

        // DANN-Werte Backend-berechnet (D-55-03, Fat-Backend).
        let balance_after = balance_before + proposed_hours;
        let voluntary_ist_after = voluntary_ist_before - proposed_hours;
        let voluntary_soll_after = voluntary_soll_before; // Soll aendert sich nicht.
        let suggestion = RebookingSuggestion {
            batch_id: batch.id,
            sales_person_id,
            iso_year,
            iso_week,
            proposed_hours,
            balance_before,
            voluntary_ist_before,
            voluntary_soll_before,
            voluntary_delta_before: voluntary_ist_before - voluntary_soll_before,
            balance_after,
            voluntary_ist_after,
            voluntary_soll_after,
            voluntary_delta_after: voluntary_ist_after - voluntary_soll_after,
        };

        self.transaction_dao.commit(tx).await?;
        Ok(suggestion)
    }

    async fn approve_suggestion(
        &self,
        batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Batch laden — muss existieren und im State Pending sein.
        let batch = self
            .rebooking_batch_service
            .find_by_id(batch_id, context.clone(), Some(tx.clone()))
            .await?
            .ok_or(ServiceError::EntityNotFound(batch_id))?;
        if batch.state != RebookingBatchState::Pending {
            return Err(ServiceError::BatchAlreadyResolved);
        }

        // Reporting-Snapshot + Pair-Rows schreiben. Wir rekonstruieren die
        // Richtung ueber den Balance-Snapshot: negatives Defizit → Approve
        // buche VolunteerToExtra (Umbuchung Ehrenamt → bezahlt gleicht
        // Balance aus). Positives Balance ist im HR-Alert-Pfad nicht
        // vorgesehen (Predicate greift dann nicht), aber wir handhaben es
        // defensiv als VolunteerToExtra ebenso.
        let report = self
            .reporting_service
            .get_report_for_employee(
                &batch.sales_person_id,
                batch.iso_year,
                batch.iso_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let hours = proposed_rebooking_hours(report.balance_hours, report.volunteer_hours);
        if hours > 0.0 {
            let (out_payload, in_payload) = Self::build_pair_payloads(
                batch.sales_person_id,
                batch.iso_year,
                batch.iso_week,
                RebookingDirection::VolunteerToExtra,
                hours,
            );
            self.extra_hours_service
                .create(&out_payload, context.clone(), Some(tx.clone()))
                .await?;
            self.extra_hours_service
                .create(&in_payload, context.clone(), Some(tx.clone()))
                .await?;
        }

        // State-conditional UPDATE (HR-ALERT-03, T-55-01).
        let now = self.clock_service.date_time_now();
        let affected = self
            .rebooking_batch_service
            .update_state_conditional(
                batch_id,
                RebookingBatchState::Pending,
                RebookingBatchState::Approved,
                Some(now),
                None, // approved_by wird von rest/handler-Layer gefuellt (Plan 55-02).
                context.clone(),
                Some(tx.clone()),
            )
            .await?;
        if affected == 0 {
            return Err(ServiceError::BatchAlreadyResolved);
        }

        // Frisch geladenes Entity zurueckgeben.
        let updated = self
            .rebooking_batch_service
            .find_by_id(batch_id, context.clone(), Some(tx.clone()))
            .await?
            .ok_or(ServiceError::EntityNotFound(batch_id))?;

        self.transaction_dao.commit(tx).await?;
        Ok(updated)
    }

    async fn reject_suggestion(
        &self,
        batch_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let batch = self
            .rebooking_batch_service
            .find_by_id(batch_id, context.clone(), Some(tx.clone()))
            .await?
            .ok_or(ServiceError::EntityNotFound(batch_id))?;
        if batch.state != RebookingBatchState::Pending {
            return Err(ServiceError::BatchAlreadyResolved);
        }

        // KEIN ExtraHours-Schreiben (D-55-07). UNIQUE-Slot bleibt belegt.
        let now = self.clock_service.date_time_now();
        let affected = self
            .rebooking_batch_service
            .update_state_conditional(
                batch_id,
                RebookingBatchState::Pending,
                RebookingBatchState::Rejected,
                Some(now),
                None,
                context.clone(),
                Some(tx.clone()),
            )
            .await?;
        if affected == 0 {
            return Err(ServiceError::BatchAlreadyResolved);
        }

        let updated = self
            .rebooking_batch_service
            .find_by_id(batch_id, context.clone(), Some(tx.clone()))
            .await?
            .ok_or(ServiceError::EntityNotFound(batch_id))?;

        self.transaction_dao.commit(tx).await?;
        Ok(updated)
    }

    async fn list_pending_for_sales_person(
        &self,
        sales_person_id: Option<Uuid>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingSuggestion]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let batches = match sales_person_id {
            Some(id) => {
                self.rebooking_batch_service
                    .find_pending_for_sales_person(id, context.clone(), Some(tx.clone()))
                    .await?
            }
            None => {
                self.rebooking_batch_service
                    .list_all_pending(context.clone(), Some(tx.clone()))
                    .await?
            }
        };

        // Hydratisieren: pro Batch das IST/DANN-Bundle bauen.
        let mut suggestions: Vec<RebookingSuggestion> = Vec::with_capacity(batches.len());
        for batch in batches.iter() {
            let suggestion = self
                .hydrate_pending_to_suggestion(batch, context.clone(), Some(tx.clone()))
                .await?;
            suggestions.push(suggestion);
        }

        self.transaction_dao.commit(tx).await?;
        Ok(suggestions.into())
    }

    async fn hydrate_pending_to_suggestion(
        &self,
        batch: &RebookingBatchEntity,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingSuggestion, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // IST-Snapshot aus dem Reporting-Aggregat (Rebooking-Filter greift
        // schon zentral in reporting.rs → keine Doppel-Zaehlung).
        let report = self
            .reporting_service
            .get_report_for_employee(
                &batch.sales_person_id,
                batch.iso_year,
                batch.iso_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let balance_before = report.balance_hours;
        let voluntary_ist_before = report.volunteer_hours;
        // Soll: Plan 01 haelt den Soll-Snapshot auf 0.0 (Audit-nur — der
        // vollstaendige Voluntary-Stats-Soll-Aggregator ist HR-only und
        // waere hier ein zusaetzlicher Cross-Service-Cycle mit
        // VoluntaryStatsService; Plan 55-02 haengt VoluntaryStatsService
        // im REST-Handler nach, wenn das FE tatsaechlich Soll-Zahlen
        // anzeigen will. Fat-Backend-Regel D-55-03 bleibt gewahrt: die
        // Delta-Felder werden hier trotzdem Backend-berechnet, nur mit
        // dem konservativen Soll=0.0).
        let voluntary_soll_before = 0.0_f32;

        let proposed_hours = proposed_rebooking_hours(balance_before, voluntary_ist_before);
        let balance_after = balance_before + proposed_hours;
        let voluntary_ist_after = voluntary_ist_before - proposed_hours;
        let voluntary_soll_after = voluntary_soll_before;

        let suggestion = RebookingSuggestion {
            batch_id: batch.id,
            sales_person_id: batch.sales_person_id,
            iso_year: batch.iso_year,
            iso_week: batch.iso_week,
            proposed_hours,
            balance_before,
            voluntary_ist_before,
            voluntary_soll_before,
            voluntary_delta_before: voluntary_ist_before - voluntary_soll_before,
            balance_after,
            voluntary_ist_after,
            voluntary_soll_after,
            voluntary_delta_after: voluntary_ist_after - voluntary_soll_after,
        };

        self.transaction_dao.commit(tx).await?;
        Ok(suggestion)
    }
}

// `REBOOKING_RECONCILIATION_PROCESS` wird derzeit nur intern als
// Doc-/Prozess-Marker gehalten (Aufruf ueber Basic-Services trägt eigene
// Prozess-Tags). Silencing damit clippy nicht ueber unused constant
// stolpert, bis Plan 55-02 audit_process-Felder ergänzt.
#[allow(dead_code)]
const _PROCESS_TAG: &str = REBOOKING_RECONCILIATION_PROCESS;
