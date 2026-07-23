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
use service::rebooking_batch::RebookingBatchService;
use service::rebooking_reconciliation::{
    RebookingDirection, RebookingReconciliationService, RebookingSuggestion,
};
use service::reporting::ReportingService;
use service::voluntary_stats::VoluntaryStatsService;
use service::{PermissionService, ServiceError};
use shifty_utils::{DayOfWeek, ShiftyDate};
use std::sync::Arc;
use uuid::Uuid;

gen_service_impl! {
    struct RebookingReconciliationServiceImpl: RebookingReconciliationService = RebookingReconciliationServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction, Context = Self::Context> = extra_hours_service,
        RebookingBatchService: RebookingBatchService<Transaction = Self::Transaction, Context = Self::Context> = rebooking_batch_service,
        ReportingService: ReportingService<Transaction = Self::Transaction, Context = Self::Context> = reporting_service,
        VoluntaryStatsService: VoluntaryStatsService<Transaction = Self::Transaction, Context = Self::Context> = voluntary_stats_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
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
        hours: f32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
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

        // state-conditional UPDATE ZUERST (Pitfall 12): nur bei rows_affected==1
        // (kein EntityAlreadyExists) folgen die Pair-Writes. So entsteht das Pair
        // genau einmal; der Double-Approve-Verlierer bekommt Conflict (→ 409).
        self.rebooking_batch_service
            .update_state(
                rebooking_batch_id,
                RebookingBatchState::Approved,
                None,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;

        // F5-Standard-Richtung: VolunteerToExtra (gedeckelter Mitarbeiter im
        // Minus bucht freiwillige Stunden auf ExtraWork um). Single write-path.
        self.rebook(
            batch.sales_person_id,
            RebookingDirection::VolunteerToExtra,
            hours,
            batch.iso_year,
            batch.iso_week,
            tx.clone(),
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
}
