//! Business-Logic-Tier Implementation von
//! [`service::pdf_export::PdfExportScheduler`] (Phase 48 EXP-01 + EXP-03).
//!
//! Kombiniert die Bausteine aus 48-01 (Config), 48-02 (PDF-Render) und 48-03
//! (WebDAV-Upload) zum Cron-getriebenen Nextcloud-Push. Der Scheduler wird in
//! `shifty_bin/src/main.rs` beim Boot mit
//! [`PdfExportSchedulerImpl::start`] gestartet; Config-Änderungen via
//! `PUT /pdf-export-config` triggern
//! [`PdfExportSchedulerImpl::reload_from_db`].
//!
//! ## Token-Leak-Guard
//!
//! Diese Datei enthält KEINE `tracing`-Aufrufe, die `webdav_app_token`
//! oder ähnliche Sensitive-Config-Felder loggen. Der einzige Weg wie das
//! Token nach außen dringt ist der WebDAV-Basic-Auth-Header, den der
//! [`crate::webdav_client::WebDavClient`] intern mit
//! `header_value.set_sensitive(true)` markiert.
//!
//! ## v1-Vereinfachung: nur der erste aktive Shiftplan
//!
//! `run_once_now` rendert PRO WOCHE genau EINEN Shiftplan (den ersten
//! non-deleted aus `shiftplan_service.get_all()`), weil das für den v1-Use-
//! Case (ein „Planungs"-Shiftplan pro Woche) der Regelfall ist. Multi-
//! Shiftplan-Zusammenführung im PDF ist ein Follow-up wenn benötigt.
//!
//! ## Phase 49 Refactor (D-49-08 + Q1)
//!
//! Der Scheduler delegiert das PDF-Assemble jetzt an
//! [`service::pdf_shiftplan::PdfShiftplanService::render_week_pdf`] — DRY-Kern
//! der Phase 49. Konsequenz: Der Service prüft den `WeekStatus` und liefert
//! nur für `Planned`/`Locked`-Weeks Bytes; andere Weeks kommen als
//! [`ServiceError::ValidationError`] zurück und werden per `record_error` +
//! `return Ok(())` als per-Week-Skip behandelt (Q1 im Discussion-Log).
//! Der Scheduler exportiert nach diesem Refactor also NUR noch releasbare
//! Wochen — kein leaky Export unfertiger Wochenplaene ins WebDAV-Storage.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    clock::ClockService,
    pdf_export::PdfExportScheduler,
    pdf_export_config::PdfExportConfigService,
    pdf_shiftplan::PdfShiftplanService,
    permission::Authentication,
    shiftplan_catalog::ShiftplanService,
    PermissionService, ServiceError,
};
use shifty_utils::{ShiftyDate, ShiftyWeek};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::gen_service_impl;
use crate::webdav_client::{WebDavClient, WebDavError, WebDavUpload};

const INCOMPLETE_CONFIG_MSG: &str = "Konfiguration unvollständig";

gen_service_impl! {
    struct PdfExportSchedulerImpl: service::pdf_export::PdfExportScheduler = PdfExportSchedulerDeps {
        PdfExportConfigService: PdfExportConfigService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = pdf_export_config_service,
        PdfShiftplanService: PdfShiftplanService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = pdf_shiftplan_service,
        ShiftplanService: ShiftplanService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = shiftplan_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
    ; custom_fields {
        webdav_upload_factory: Arc<dyn WebDavUploadFactory> = webdav_upload_factory,
        scheduler: Arc<Mutex<Option<JobScheduler>>> = scheduler,
        current_job: Arc<Mutex<Option<Uuid>>> = current_job,
    }
}

/// Factory für den WebDAV-Upload — pro Lauf wird ein Client aus der aktuellen
/// Config gebaut. In Tests kann ein Mock-Upload injiziert werden, in der
/// Produktion baut die [`ProductionWebDavUploadFactory`] einen echten
/// [`WebDavClient`].
pub trait WebDavUploadFactory: Send + Sync + 'static {
    fn build(
        &self,
        base_url: &str,
        user: &str,
        app_token: &str,
    ) -> Result<Arc<dyn WebDavUpload>, WebDavError>;
}

/// Production-Factory: baut einen echten [`WebDavClient`] pro Lauf.
pub struct ProductionWebDavUploadFactory;

impl WebDavUploadFactory for ProductionWebDavUploadFactory {
    fn build(
        &self,
        base_url: &str,
        user: &str,
        app_token: &str,
    ) -> Result<Arc<dyn WebDavUpload>, WebDavError> {
        let client = WebDavClient::new(Arc::<str>::from(base_url), user, app_token)?;
        Ok(Arc::new(client))
    }
}

impl<Deps: PdfExportSchedulerDeps> PdfExportSchedulerImpl<Deps> {
    /// Konstruktor. Der `JobScheduler` wird lazy in `start()` initialisiert,
    /// damit `new()` synchron bleibt und die DI-Wiring-Reihenfolge in
    /// `shifty_bin/src/main.rs` einfach ist.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pdf_export_config_service: Arc<Deps::PdfExportConfigService>,
        pdf_shiftplan_service: Arc<Deps::PdfShiftplanService>,
        shiftplan_service: Arc<Deps::ShiftplanService>,
        permission_service: Arc<Deps::PermissionService>,
        clock_service: Arc<Deps::ClockService>,
        transaction_dao: Arc<Deps::TransactionDao>,
        webdav_upload_factory: Arc<dyn WebDavUploadFactory>,
    ) -> Self {
        Self {
            pdf_export_config_service,
            pdf_shiftplan_service,
            shiftplan_service,
            permission_service,
            clock_service,
            transaction_dao,
            webdav_upload_factory,
            scheduler: Arc::new(Mutex::new(None)),
            current_job: Arc::new(Mutex::new(None)),
        }
    }
}

/// Snapshot dessen was `run_once_now` aus der Config braucht — vermeidet
/// dass der Klartext-Token länger als nötig im Prozess wandert.
struct RunConfig {
    base_url: Arc<str>,
    user: Arc<str>,
    app_token: Arc<str>,
    target_folder: Arc<str>,
    weeks_horizon: u32,
}

fn extract_run_config(
    cfg: &service::pdf_export_config::PdfExportConfig,
) -> Option<RunConfig> {
    let base_url = cfg.nextcloud_url.clone()?;
    let user = cfg.webdav_user.clone()?;
    let app_token = cfg.webdav_app_token.clone()?;
    let target_folder = cfg.target_folder.clone()?;
    if base_url.is_empty()
        || user.is_empty()
        || app_token.is_empty()
        || target_folder.is_empty()
    {
        return None;
    }
    Some(RunConfig {
        base_url,
        user,
        app_token,
        target_folder,
        weeks_horizon: cfg.weeks_horizon.max(1),
    })
}

#[async_trait]
impl<Deps: PdfExportSchedulerDeps + 'static> PdfExportScheduler for PdfExportSchedulerImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn start(&self) -> Result<(), ServiceError> {
        // Initialize the JobScheduler lazily (needs a tokio runtime — safe here
        // because `start` is called from `main`).
        let mut sched_guard = self.scheduler.lock().await;
        if sched_guard.is_none() {
            let scheduler = JobScheduler::new()
                .await
                .map_err(|e| {
                    error!("pdf-export scheduler init failed: {e}");
                    ServiceError::InternalError
                })?;
            *sched_guard = Some(scheduler);
        }
        drop(sched_guard);
        // v2.3.1: boot-tolerance — ein fehlerhafter Config-Zustand in der DB
        // (z.B. ungültiger Cron-Ausdruck) darf den GESAMTEN Backend-Start
        // nicht verhindern. `reload_from_db` persistiert den Fehler bereits
        // via `record_error`; wir loggen zusätzlich eine Boot-Warnung und
        // starten den Scheduler dormant. Ein späteres PUT auf
        // `/pdf-export-config` mit gültiger Config triggert einen neuen
        // Reload und aktiviert den Job dann live.
        if let Err(e) = self.reload_from_db().await {
            warn!(
                "pdf-export: initial reload at boot failed ({e:?}) — scheduler starts dormant; fix config via PUT /pdf-export-config"
            );
        }
        let sched_guard = self.scheduler.lock().await;
        if let Some(sched) = sched_guard.as_ref() {
            sched.start().await.map_err(|e| {
                error!("pdf-export scheduler start failed: {e}");
                ServiceError::InternalError
            })?;
        }
        Ok(())
    }

    async fn reload_from_db(&self) -> Result<(), ServiceError> {
        let cfg = self
            .pdf_export_config_service
            .get(Authentication::Full, None)
            .await?;

        // Remove any previously registered job. We do this whether or not
        // the new config is enabled — a "disabled" state simply means no
        // active job registration.
        let mut sched_guard = self.scheduler.lock().await;
        let scheduler = match sched_guard.as_mut() {
            Some(s) => s,
            None => {
                // Not initialised yet — `start()` will call reload_from_db.
                return Ok(());
            }
        };
        let mut job_guard = self.current_job.lock().await;
        if let Some(job_id) = job_guard.take() {
            if let Err(e) = scheduler.remove(&job_id).await {
                warn!("pdf-export: could not remove previous cron job: {e}");
            }
        }

        if !cfg.enabled {
            info!("pdf-export: scheduler reload complete (disabled)");
            return Ok(());
        }

        let cron_schedule = cfg.cron_schedule.to_string();
        let scheduler_ref = self.clone_for_job();
        let job_result = Job::new_async(cron_schedule.as_str(), move |_uuid, _lock| {
            let scheduler = scheduler_ref.clone();
            Box::pin(async move {
                if let Err(e) = scheduler.run_once_now(Authentication::Full).await {
                    error!("pdf-export cron run failed: {e:?}");
                }
            })
        });
        match job_result {
            Ok(job) => match scheduler.add(job).await {
                Ok(job_id) => {
                    *job_guard = Some(job_id);
                    info!(
                        "pdf-export: cron job registered with schedule '{}'",
                        cron_schedule
                    );
                    Ok(())
                }
                Err(e) => {
                    error!("pdf-export: could not add cron job: {e}");
                    Err(ServiceError::InternalError)
                }
            },
            Err(e) => {
                error!("pdf-export: invalid cron expression '{}': {e}", cron_schedule);
                // Persist a diagnostic so the admin sees the failure in the UI.
                let at = self.clock_service.date_time_now();
                let msg: Arc<str> =
                    Arc::from(format!("Cron-Ausdruck ungültig: {cron_schedule}"));
                if let Err(err) = self
                    .pdf_export_config_service
                    .record_error(at, msg, Authentication::Full, None)
                    .await
                {
                    warn!("pdf-export: could not persist cron-parse error: {err:?}");
                }
                Err(ServiceError::InternalError)
            }
        }
    }

    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        // Trusted-Caller-Gate: Cron-Path calls with Authentication::Full;
        // REST-Handler stellt sicher dass Admin geprüft ist und wandelt dann
        // in Authentication::Full um für den Full-Auth-only Config-Recorder.
        self.permission_service
            .check_only_full_authentication(context.clone())
            .await?;

        let cfg = self
            .pdf_export_config_service
            .get(Authentication::Full, None)
            .await?;

        if !cfg.enabled {
            info!("pdf-export skipped (disabled)");
            return Ok(());
        }

        let run_cfg = match extract_run_config(&cfg) {
            Some(v) => v,
            None => {
                info!("pdf-export skipped (incomplete config)");
                let at = self.clock_service.date_time_now();
                let msg: Arc<str> = Arc::from(INCOMPLETE_CONFIG_MSG);
                self.pdf_export_config_service
                    .record_error(at, msg, Authentication::Full, None)
                    .await?;
                return Ok(());
            }
        };

        // Build the WebDAV upload client via the factory. Any failure to
        // build is persisted as record_error and returns Ok(()).
        let upload: Arc<dyn WebDavUpload> = match self.webdav_upload_factory.build(
            &run_cfg.base_url,
            &run_cfg.user,
            &run_cfg.app_token,
        ) {
            Ok(u) => u,
            Err(e) => {
                let at = self.clock_service.date_time_now();
                let msg: Arc<str> = Arc::from(format!("WebDAV-Client-Init fehlgeschlagen: {e}"));
                self.pdf_export_config_service
                    .record_error(at, msg, Authentication::Full, None)
                    .await?;
                return Ok(());
            }
        };

        // Pull the shiftplans and sales-persons once for the whole horizon.
        let all_shiftplans = self
            .shiftplan_service
            .get_all(Authentication::Full, None)
            .await?;
        let active_shiftplan = all_shiftplans
            .iter()
            .find(|s| s.deleted.is_none());
        let shiftplan_id = match active_shiftplan {
            Some(s) => s.id,
            None => {
                let at = self.clock_service.date_time_now();
                let msg: Arc<str> = Arc::from("Kein aktiver Shiftplan vorhanden");
                self.pdf_export_config_service
                    .record_error(at, msg, Authentication::Full, None)
                    .await?;
                return Ok(());
            }
        };

        // Determine the horizon of ISO weeks.
        let now_date = self.clock_service.date_now();
        let start_shifty = ShiftyDate::from_date(now_date);
        let start_week = ShiftyWeek::new(start_shifty.year(), start_shifty.week());
        let mut cursor = start_week;
        let horizon = run_cfg.weeks_horizon as usize;
        let mut succeeded_count: usize = 0;
        for _offset in 0..horizon {
            let (y, w) = (cursor.year, cursor.week);
            cursor = cursor.next();
            // Phase 49 D-49-08: Delegate assemble (WeekStatus-Gate + View +
            // active-SalesPersons + Render) an den PdfShiftplanService.
            // D-49-07: Aufrufer-Kontext = `Authentication::Full` (Scheduler
            // ist trusted; Cron-Callback ruft `run_once_now` mit Full).
            // v2.3.1: ValidationError (Status != Planned/Locked) wird per
            // `record_error` geloggt und die Schleife läuft mit `continue`
            // weiter — vorher brach der komplette Horizon beim ersten
            // Draft-Week ab und ließ spätere Planned-Wochen ungeexportiert.
            let bytes = match self
                .pdf_shiftplan_service
                .render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)
                .await
            {
                Ok(b) => b,
                Err(e) => {
                    let at = self.clock_service.date_time_now();
                    let msg: Arc<str> = Arc::from(format!(
                        "PDF-Assemble/Render fuer KW{w:02}/{y} fehlgeschlagen: {e}"
                    ));
                    self.pdf_export_config_service
                        .record_error(at, msg, Authentication::Full, None)
                        .await?;
                    continue;
                }
            };

            let filename = crate::pdf_shiftplan::filename_for(y, w);
            if let Err(e) = upload
                .upload_file(&run_cfg.target_folder, &filename, bytes)
                .await
            {
                let at = self.clock_service.date_time_now();
                let msg: Arc<str> = match &e {
                    WebDavError::Transient { attempts, .. } => Arc::from(format!(
                        "WebDAV-Upload für KW{w:02}/{y} nach {attempts} Versuchen (transient) fehlgeschlagen"
                    )),
                    WebDavError::Permanent { status, .. } => Arc::from(format!(
                        "WebDAV-Upload für KW{w:02}/{y} permanent fehlgeschlagen ({status})"
                    )),
                    WebDavError::Io(_) => Arc::from(format!(
                        "WebDAV-Upload für KW{w:02}/{y} — Netzwerkfehler"
                    )),
                };
                // Log a token-free diagnostic (WebDavError's Display never
                // includes the Basic-Auth header — see 48-03 T-48-08).
                error!("pdf-export upload failed for KW{w:02}/{y}: {e}");
                self.pdf_export_config_service
                    .record_error(at, msg, Authentication::Full, None)
                    .await?;
                // Do not attempt further weeks — surface first failure and
                // let the next cron slot retry from scratch (per plan).
                return Ok(());
            }

            succeeded_count += 1;
        }

        // v2.3.1: nur `record_success` wenn tatsächlich mind. eine Woche
        // hochgeladen wurde. Sind alle Wochen im Horizon fehlgeschlagen
        // (`succeeded_count == 0`), bleibt `record_error` die einzige
        // Persistenz — sonst würde die UI Erfolg suggerieren obwohl nichts
        // exportiert wurde.
        if succeeded_count > 0 {
            let at = self.clock_service.date_time_now();
            self.pdf_export_config_service
                .record_success(at, Authentication::Full, None)
                .await?;
            info!(
                "pdf-export success: {}/{} week(s) uploaded to Nextcloud",
                succeeded_count, horizon
            );
        } else {
            info!(
                "pdf-export run finished with 0/{} weeks uploaded — see last_error_message",
                horizon
            );
        }
        Ok(())
    }
}

/// Clone-Handle für den Cron-Callback: wir dürfen `self` nicht direkt in die
/// Closure moven (async trait method), also klonen wir alle `Arc`-Handles in
/// einen leichten Handle-Struct.
struct PdfExportSchedulerHandle<Deps: PdfExportSchedulerDeps> {
    inner: Arc<PdfExportSchedulerImpl<Deps>>,
}

impl<Deps: PdfExportSchedulerDeps> Clone for PdfExportSchedulerHandle<Deps> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Deps: PdfExportSchedulerDeps + 'static> PdfExportSchedulerHandle<Deps> {
    async fn run_once_now(
        &self,
        context: Authentication<Deps::Context>,
    ) -> Result<(), ServiceError> {
        self.inner.run_once_now(context).await
    }
}

impl<Deps: PdfExportSchedulerDeps> PdfExportSchedulerImpl<Deps> {
    fn clone_for_job(&self) -> PdfExportSchedulerHandle<Deps> {
        // Build a fresh Arc that shares the fields — cheaper than `Arc::new`
        // wrapping self because the impl fields are already Arc.
        PdfExportSchedulerHandle {
            inner: Arc::new(Self {
                pdf_export_config_service: self.pdf_export_config_service.clone(),
                pdf_shiftplan_service: self.pdf_shiftplan_service.clone(),
                shiftplan_service: self.shiftplan_service.clone(),
                permission_service: self.permission_service.clone(),
                clock_service: self.clock_service.clone(),
                transaction_dao: self.transaction_dao.clone(),
                webdav_upload_factory: self.webdav_upload_factory.clone(),
                scheduler: self.scheduler.clone(),
                current_job: self.current_job.clone(),
            }),
        }
    }
}

impl<Deps: PdfExportSchedulerDeps> Debug for PdfExportSchedulerImpl<Deps> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfExportSchedulerImpl")
            .finish_non_exhaustive()
    }
}
