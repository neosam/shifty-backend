//! Business-Logic-Tier Implementation von
//! [`service::voluntary_rebooking_scheduler::VoluntaryRebookingScheduler`]
//! (Phase 56, REB-AUTO-01).
//!
//! 1:1-Spiegel des [`crate::pdf_export_scheduler::PdfExportSchedulerImpl`]
//! (JobScheduler-Lifecycle, `clone_for_job`-Handle-Trick, 6-Feld-Cron), aber:
//!
//! - KEIN `reload_from_db`: der Stichtag-Toggle wird pro Lauf frisch in
//!   `run_for_week` gelesen — der Cron-Ausdruck ist statisch.
//! - Der Cron-Ausdruck ist HARDCODED `0 15 3 * * 1` (Montag 03:15,
//!   konfliktfrei vs. PDF-Export Montag 06:00, D-03). Ein 6-Feld-Ausdruck ist
//!   Pflicht (der v2.3.1-Hotfix bewies, dass 5-Feld-Ausdruecke unter
//!   `tokio-cron-scheduler` still nie feuern).
//!
//! `run_once_now` berechnet die letzte abgeschlossene ISO-Woche (Vorwoche) aus
//! `ClockService::date_now()` — jahresgrenzsicher via
//! `ShiftyDate::from_date().as_shifty_week()` + Step-Back (nie naives
//! `year()+week()`, T-56-08) — und delegiert pro gedeckelter `SalesPerson` an
//! [`service::rebooking_reconciliation::RebookingReconciliationService::run_for_week`].
//! Ein Fehler fuer EINE Person wird geloggt und bricht den Lauf NICHT ab
//! (T-56-07).

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::rebooking_batch::RebookingBatchKind;
use dao::TransactionDao;
use service::{
    clock::ClockService,
    employee_work_details::EmployeeWorkDetailsService,
    permission::Authentication,
    rebooking_reconciliation::{RebookingReconciliationService, RunForWeekOutcome},
    voluntary_rebooking_scheduler::VoluntaryRebookingScheduler,
    PermissionService, ServiceError,
};
use shifty_utils::{ShiftyDate, ShiftyWeek};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use uuid::Uuid;

use crate::gen_service_impl;

/// Woechentlicher Cron-Ausdruck (6-Feld, `sec min hour dom month dow`):
/// Montag 03:15:00. 6 Felder sind unter `tokio-cron-scheduler` Pflicht — ein
/// 5-Feld-Ausdruck registriert still einen nie-feuernden Job (v2.3.1-Hotfix).
/// Konfliktfrei vs. PDF-Export (Montag 06:00, D-03).
const VOLUNTARY_REBOOKING_CRON: &str = "0 15 3 * * 1";

gen_service_impl! {
    struct VoluntaryRebookingSchedulerImpl: service::voluntary_rebooking_scheduler::VoluntaryRebookingScheduler = VoluntaryRebookingSchedulerDeps {
        RebookingReconciliationService: RebookingReconciliationService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = rebooking_reconciliation_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = employee_work_details_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
    ; custom_fields {
        scheduler: Arc<Mutex<Option<JobScheduler>>> = scheduler,
        current_job: Arc<Mutex<Option<Uuid>>> = current_job,
    }
}

impl<Deps: VoluntaryRebookingSchedulerDeps> VoluntaryRebookingSchedulerImpl<Deps> {
    /// Konstruktor. Der `JobScheduler` wird lazy in `start()` initialisiert,
    /// damit `new()` synchron bleibt und die DI-Wiring-Reihenfolge in
    /// `shifty_bin/src/main.rs` einfach ist.
    pub fn new(
        rebooking_reconciliation_service: Arc<Deps::RebookingReconciliationService>,
        employee_work_details_service: Arc<Deps::EmployeeWorkDetailsService>,
        permission_service: Arc<Deps::PermissionService>,
        clock_service: Arc<Deps::ClockService>,
        transaction_dao: Arc<Deps::TransactionDao>,
    ) -> Self {
        Self {
            rebooking_reconciliation_service,
            employee_work_details_service,
            permission_service,
            clock_service,
            transaction_dao,
            scheduler: Arc::new(Mutex::new(None)),
            current_job: Arc::new(Mutex::new(None)),
        }
    }
}

/// Vorwoche einer `ShiftyWeek` mit korrektem ISO-Jahresgrenzfall (nie naives
/// `week - 1`): in KW 1 springt sie auf die letzte Woche des Vorjahres
/// (T-56-08). Spiegelt `rebooking_reconciliation::previous_week` (dort
/// modul-privat), damit der Scheduler die Vorwochen-Arithmetik teilt.
fn previous_week(week: ShiftyWeek) -> ShiftyWeek {
    if week.week <= 1 {
        let prev_year = week.year - 1;
        ShiftyWeek::new(prev_year, time::util::weeks_in_year(prev_year as i32))
    } else {
        ShiftyWeek::new(week.year, week.week - 1)
    }
}

#[async_trait]
impl<Deps: VoluntaryRebookingSchedulerDeps + 'static> VoluntaryRebookingScheduler
    for VoluntaryRebookingSchedulerImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn start(&self) -> Result<(), ServiceError> {
        // Initialize the JobScheduler lazily (needs a tokio runtime — safe here
        // because `start` is called from `main`).
        let mut sched_guard = self.scheduler.lock().await;
        if sched_guard.is_none() {
            let scheduler = JobScheduler::new().await.map_err(|e| {
                error!("voluntary-rebooking scheduler init failed: {e}");
                ServiceError::InternalError
            })?;
            *sched_guard = Some(scheduler);
        }

        let scheduler = sched_guard
            .as_mut()
            .expect("scheduler was just initialised above");

        // 6-Feld-Cron (T-56-09): ein 5-Feld-Ausdruck registriert still einen
        // nie-feuernden Job.
        let scheduler_ref = self.clone_for_job();
        let job = Job::new_async(VOLUNTARY_REBOOKING_CRON, move |_uuid, _lock| {
            let scheduler = scheduler_ref.clone();
            Box::pin(async move {
                // T-56-07: ein Fehler im Wochen-Lauf darf den Cron-Callback nicht
                // in einen Panic reissen — nur loggen, der naechste Slot laeuft
                // erneut.
                if let Err(e) = scheduler.run_once_now(Authentication::Full).await {
                    error!("voluntary-rebooking cron run failed: {e:?}");
                }
            })
        })
        .map_err(|e| {
            error!(
                "voluntary-rebooking: invalid cron expression '{VOLUNTARY_REBOOKING_CRON}': {e}"
            );
            ServiceError::InternalError
        })?;

        let job_id = scheduler.add(job).await.map_err(|e| {
            error!("voluntary-rebooking: could not add cron job: {e}");
            ServiceError::InternalError
        })?;
        *self.current_job.lock().await = Some(job_id);
        info!(
            "voluntary-rebooking: cron job registered with schedule '{VOLUNTARY_REBOOKING_CRON}' (Monday 03:15)"
        );

        scheduler.start().await.map_err(|e| {
            error!("voluntary-rebooking scheduler start failed: {e}");
            ServiceError::InternalError
        })?;
        Ok(())
    }

    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        // Trusted-Caller-Gate: der Cron-Path ruft mit Authentication::Full.
        self.permission_service
            .check_only_full_authentication(context.clone())
            .await?;

        // Vorwoche = die letzte abgeschlossene ISO-Woche (nie die laufende).
        // Jahresgrenzsicher via ShiftyDate::from_date().as_shifty_week() +
        // Step-Back (T-56-08 — nie naives year()+week()).
        let today = ShiftyDate::from_date(self.clock_service.date_now());
        let target = previous_week(today.as_shifty_week());
        let (iso_year, iso_week) = (target.year, target.week);

        // Kandidaten: alle gedeckelten SalesPersons mit gueltigem Vertrag in der
        // Vorwoche (dieselbe Quelle wie run_for_week/backfill_range —
        // employee_work_details all_for_week, cap-Filter). Ein Fehler HIER ist
        // ein harter Setup-Fehler (kein einzelner-Person-Skip) und wird
        // propagiert.
        let contracts = self
            .employee_work_details_service
            .all_for_week(iso_week, iso_year, Authentication::Full, None)
            .await?;

        let mut booked = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;
        for wd in contracts.iter() {
            if !wd.cap_planned_hours_to_expected {
                continue;
            }
            let sales_person_id = wd.sales_person_id;

            // T-56-07: pro-Person-Fehler wird geloggt, bricht den Lauf NICHT ab.
            match self
                .rebooking_reconciliation_service
                .run_for_week(
                    sales_person_id,
                    iso_year,
                    iso_week,
                    RebookingBatchKind::AutoCron,
                    Authentication::Full,
                    None,
                )
                .await
            {
                Ok(RunForWeekOutcome::Booked { hours }) => {
                    booked += 1;
                    info!(
                        "voluntary-rebooking: booked {hours}h for {sales_person_id} in KW{iso_week:02}/{iso_year}"
                    );
                }
                Ok(RunForWeekOutcome::Skipped { .. }) => {
                    skipped += 1;
                }
                Err(e) => {
                    failed += 1;
                    error!(
                        "voluntary-rebooking: run_for_week failed for {sales_person_id} in KW{iso_week:02}/{iso_year}: {e:?} — continuing"
                    );
                }
            }
        }

        info!(
            "voluntary-rebooking run finished for KW{iso_week:02}/{iso_year}: {booked} booked, {skipped} skipped, {failed} failed"
        );
        Ok(())
    }
}

/// Clone-Handle fuer den Cron-Callback: wir duerfen `self` nicht direkt in die
/// Closure moven (async trait method), also klonen wir alle `Arc`-Handles in
/// einen leichten Handle-Struct.
struct VoluntaryRebookingSchedulerHandle<Deps: VoluntaryRebookingSchedulerDeps> {
    inner: Arc<VoluntaryRebookingSchedulerImpl<Deps>>,
}

impl<Deps: VoluntaryRebookingSchedulerDeps> Clone for VoluntaryRebookingSchedulerHandle<Deps> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Deps: VoluntaryRebookingSchedulerDeps + 'static> VoluntaryRebookingSchedulerHandle<Deps> {
    async fn run_once_now(
        &self,
        context: Authentication<Deps::Context>,
    ) -> Result<(), ServiceError> {
        self.inner.run_once_now(context).await
    }
}

impl<Deps: VoluntaryRebookingSchedulerDeps> VoluntaryRebookingSchedulerImpl<Deps> {
    fn clone_for_job(&self) -> VoluntaryRebookingSchedulerHandle<Deps> {
        // Build a fresh Arc that shares the fields — the impl fields are already
        // Arc, so this only clones handles (cheap).
        VoluntaryRebookingSchedulerHandle {
            inner: Arc::new(Self {
                rebooking_reconciliation_service: self.rebooking_reconciliation_service.clone(),
                employee_work_details_service: self.employee_work_details_service.clone(),
                permission_service: self.permission_service.clone(),
                clock_service: self.clock_service.clone(),
                transaction_dao: self.transaction_dao.clone(),
                scheduler: self.scheduler.clone(),
                current_job: self.current_job.clone(),
            }),
        }
    }
}

impl<Deps: VoluntaryRebookingSchedulerDeps> Debug for VoluntaryRebookingSchedulerImpl<Deps> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoluntaryRebookingSchedulerImpl")
            .finish_non_exhaustive()
    }
}
