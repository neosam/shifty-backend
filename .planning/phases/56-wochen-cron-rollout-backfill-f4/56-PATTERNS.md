# Phase 56: Wochen-Cron + Rollout-Backfill (F4) — Pattern Map

**Mapped:** 2026-07-26
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service/src/voluntary_rebooking_scheduler.rs` | service (trait) | event-driven (cron) | `service/src/pdf_export.rs` | exact |
| `service_impl/src/voluntary_rebooking_scheduler.rs` | service (impl, BL-tier) | event-driven (cron) | `service_impl/src/pdf_export_scheduler.rs` | exact |
| `service_impl/src/voluntary_rebooking_gate.rs` | utility (chain-guard) | transform | `service_impl/src/shortday_gate.rs` | exact |
| `service_impl/src/rebooking_reconciliation.rs` (MODIFY) | service (impl, BL-tier) | CRUD | `service_impl/src/rebooking_reconciliation.rs` | self-analog |
| `rest/src/rebooking.rs` (MODIFY) | controller | request-response | `rest/src/rebooking.rs` (existing handlers) | self-analog |
| `rest-types/src/rebooking.rs` (MODIFY) | model (DTO) | request-response | `rest-types/src/rebooking.rs` (existing TOs) | self-analog |
| `shifty_bin/src/main.rs` (MODIFY) | config (DI wiring) | — | `shifty_bin/src/main.rs` line 1463–1471 (pdf_export_scheduler) | exact |
| `shifty-dioxus/src/loader.rs` (MODIFY) | utility (API loader) | request-response | `shifty-dioxus/src/loader.rs` line 962–982 (shortday_clipping) | exact |
| `shifty-dioxus/src/page/settings.rs` (MODIFY) | component (Admin-Card) | request-response | `shifty-dioxus/src/page/settings.rs` line 717–808 (Card 2b) | exact |
| `shifty-dioxus/Dioxus.toml` (MODIFY) | config (proxy) | — | `Dioxus.toml` line 106–107 (`/admin/cutover`) | exact |

---

## Pattern Assignments

### `service/src/voluntary_rebooking_scheduler.rs` (trait, event-driven)

**Analog:** `service/src/pdf_export.rs`

**Imports pattern** (lines 18–24):
```rust
use std::fmt::Debug;
use async_trait::async_trait;
use mockall::automock;
use crate::{permission::Authentication, ServiceError};
```

**Core trait pattern** (lines 25–52):
```rust
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait PdfExportScheduler {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn start(&self) -> Result<(), ServiceError>;

    // Note: NO reload_from_db needed — VoluntaryRebookingScheduler has no
    // config-reload trigger; Toggle is read at each run_once_now call.

    async fn run_once_now(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
```

**Adaptation note:** Omit `reload_from_db()` — the voluntary rebooking scheduler reads the toggle fresh each run; no config-reload path exists.

---

### `service_impl/src/voluntary_rebooking_scheduler.rs` (impl, event-driven)

**Analog:** `service_impl/src/pdf_export_scheduler.rs`

**Imports pattern** (lines 37–58):
```rust
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    clock::ClockService,
    permission::Authentication,
    PermissionService, ServiceError,
};
use shifty_utils::{ShiftyDate, ShiftyWeek};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::gen_service_impl;
```

**gen_service_impl! + custom_fields pattern** (lines 62–85):
```rust
gen_service_impl! {
    struct PdfExportSchedulerImpl: service::pdf_export::PdfExportScheduler = PdfExportSchedulerDeps {
        PdfExportConfigService: PdfExportConfigService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = pdf_export_config_service,
        // ... other deps ...
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
    ; custom_fields {
        scheduler: Arc<Mutex<Option<JobScheduler>>> = scheduler,
        current_job: Arc<Mutex<Option<Uuid>>> = current_job,
    }
}
```

**Adaptation for VoluntaryRebookingScheduler deps:**
```rust
gen_service_impl! {
    struct VoluntaryRebookingSchedulerImpl: service::voluntary_rebooking_scheduler::VoluntaryRebookingScheduler = VoluntaryRebookingSchedulerDeps {
        RebookingReconciliationService: RebookingReconciliationService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = rebooking_reconciliation_service,
        SalesPersonService: SalesPersonService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = sales_person_service,
        ToggleService: ToggleService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = toggle_service,
        WeekStatusService: WeekStatusService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = week_status_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
    ; custom_fields {
        scheduler: Arc<Mutex<Option<JobScheduler>>> = scheduler,
        current_job: Arc<Mutex<Option<Uuid>>> = current_job,
    }
}
```

**Constructor new() pattern** (lines 120–141): Initialises all Arc deps, sets `scheduler: Arc::new(Mutex::new(None))` and `current_job: Arc::new(Mutex::new(None))`.

**start() pattern** (lines 181–215): JobScheduler::new() lazily, then register Job::new_async with hardcoded 6-field cron (`"0 15 3 * * 1"` — Montag 03:15), then `sched.start()`. No `reload_from_db` call — dormant start when toggle is None is handled inside `run_once_now`.

**6-field cron Job::new_async pattern** (lines 248–265):
```rust
let job_result = Job::new_async("0 15 3 * * 1", move |_uuid, _lock| {
    let scheduler = scheduler_ref.clone();
    Box::pin(async move {
        if let Err(e) = scheduler.run_once_now(Authentication::Full).await {
            error!("voluntary-rebooking cron run failed: {e:?}");
        }
    })
});
match job_result {
    Ok(job) => match scheduler.add(job).await {
        Ok(job_id) => {
            *job_guard = Some(job_id);
            info!("voluntary-rebooking: cron job registered with schedule '0 15 3 * * 1'");
            Ok(())
        }
        Err(e) => {
            error!("voluntary-rebooking: could not add cron job: {e}");
            Err(ServiceError::InternalError)
        }
    },
    Err(e) => {
        error!("voluntary-rebooking: invalid cron expression: {e}");
        Err(ServiceError::InternalError)
    }
}
```

**clone_for_job() + Handle struct pattern** (lines 452–494):
```rust
struct PdfExportSchedulerHandle<Deps: PdfExportSchedulerDeps> {
    inner: Arc<PdfExportSchedulerImpl<Deps>>,
}

impl<Deps: PdfExportSchedulerDeps> Clone for PdfExportSchedulerHandle<Deps> {
    fn clone(&self) -> Self { Self { inner: self.inner.clone() } }
}

impl<Deps: PdfExportSchedulerDeps> PdfExportSchedulerImpl<Deps> {
    fn clone_for_job(&self) -> PdfExportSchedulerHandle<Deps> {
        PdfExportSchedulerHandle {
            inner: Arc::new(Self {
                // clone every Arc field individually:
                pdf_export_config_service: self.pdf_export_config_service.clone(),
                // ...
                scheduler: self.scheduler.clone(),
                current_job: self.current_job.clone(),
            }),
        }
    }
}
```

**ISO-Vorwoche arithmetic pattern** (lines 362–365 + RESEARCH Pattern 4):
```rust
let today = self.clock_service.date_now();
let today_shifty = ShiftyDate::from_date(today);
let current_week = ShiftyWeek::new(today_shifty.year(), today_shifty.week());
let prev_week = if current_week.week == 1 {
    ShiftyWeek::new(
        current_week.year - 1,
        time::util::weeks_in_year((current_week.year - 1) as i32),
    )
} else {
    ShiftyWeek::new(current_week.year, current_week.week - 1)
};
let (iso_year, iso_week) = (prev_week.year, prev_week.week);
```

**CRITICAL:** NEVER use `today.year()` + `today.iso_week()` directly — use `ShiftyDate::from_date(today)` which calls `to_iso_week_date()` internally. See RESEARCH Pitfall 5.

---

### `service_impl/src/voluntary_rebooking_gate.rs` (utility, transform)

**Analog:** `service_impl/src/shortday_gate.rs` (full file, 117 lines)

**TOGGLE_NAME constant** (line 42):
```rust
pub const TOGGLE_NAME: &str = "voluntary_rebooking_auto_active_from";
```

**parse_active_from() pattern** (lines 51–57):
```rust
pub fn parse_active_from(raw: Option<&str>) -> Option<Date> {
    let s = raw?;
    if s.is_empty() { return None; }
    Date::parse(s, &Iso8601::DEFAULT).ok()
}
```

**should_run_for_week() function** — analogous to `should_clip()` (lines 66–71) but operates on ISO week coordinates:
```rust
// shortday_gate::should_clip(booking_date: Date, active_from: Option<Date>) -> bool
// Adaptation: voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from) -> bool
// Use resolve_active_from_for_week pattern (lines 127–137) as the basis:
pub fn should_run_for_week(iso_year: u32, iso_week: u8, active_from: Option<Date>) -> bool {
    let Ok(monday) = Date::from_iso_week_date(iso_year as i32, iso_week, time::Weekday::Monday)
    else { return false; };
    match active_from {
        None => false,
        Some(cutoff) => monday >= cutoff,
    }
}
```

**read_active_from() with Unauthorized tolerance** (lines 105–117):
```rust
pub(crate) async fn read_active_from<S: ToggleService + ?Sized>(
    toggle_service: &S,
    context: Authentication<S::Context>,
) -> Result<Option<Date>, ServiceError> {
    match toggle_service
        .get_toggle_value(TOGGLE_NAME, context, None)
        .await
    {
        Ok(raw) => Ok(parse_active_from(raw.as_deref())),
        Err(ServiceError::Unauthorized) => Ok(None),
        Err(e) => Err(e),
    }
}
```

**Unit test pattern** (lines 243–337): Copy `parse_none_returns_none`, `parse_empty_returns_none`, `parse_malformed_returns_none`, `parse_iso_valid`, `should_clip_none_active_from_returns_false`, `should_clip_before_stichtag_returns_false`, `should_clip_at_or_after_stichtag_returns_true` — adapt for ISO-week dates.

---

### `service_impl/src/rebooking_reconciliation.rs` (MODIFY — add `run_for_week`, `backfill_range`)

**Analog:** Existing `service_impl/src/rebooking_reconciliation.rs` lines 82–129 (existing `rebook()` write-path)

**Trait extensions to add** (in `service/src/rebooking_reconciliation.rs`):
```rust
async fn run_for_week(
    &self,
    sales_person_id: Uuid,
    iso_year: u32,
    iso_week: u8,
    kind: RebookingBatchKind,   // AutoCron | AutoCronBackfill
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<RunForWeekOutcome, ServiceError>;

async fn backfill_range(
    &self,
    from: ShiftyWeek,
    dry_run: bool,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Vec<BackfillPreviewItem>, ServiceError>;
```

**ON CONFLICT idempotency pattern** (from RESEARCH Example 3, based on dao_impl_sqlite/src/rebooking_batch.rs lines 162–163, 421–442):
```rust
match self.rebooking_batch_service.create(&batch, &[], auth, Some(tx.clone())).await {
    Ok(_) => {
        // write pair ExtraHours via existing rebook() path
    }
    Err(ServiceError::EntityAlreadyExists(_)) => {
        // no-op — slot already claimed (D-02 Claim-on-Suggest or prior Cron run)
        info!("rebooking_cron: slot ({sp_id}, {iso_year}, {iso_week}) already claimed — skip");
    }
    Err(e) => return Err(e),
}
```

**WeekStatus::Locked handling** (D-04, from RESEARCH Example 4):
```rust
let status = self.week_status_service
    .get_week_status(iso_year, iso_week, Authentication::Full, Some(tx.clone()))
    .await?;
if status == WeekStatus::Locked {
    let skipped_batch = RebookingBatchEntity {
        kind: RebookingBatchKind::AutoCron,
        state: RebookingBatchState::SkippedLocked,
        ..
    };
    self.rebooking_batch_service.create(&skipped_batch, &[], auth, Some(tx.clone())).await.ok();
    return Ok(RunForWeekOutcome::Skipped { reason: SkipReason::WeekLocked });
}
```

**Outcome enums** (from RESEARCH Pattern 7):
```rust
pub enum RunForWeekOutcome {
    Booked { hours: f32 },
    Skipped { reason: SkipReason },
}
pub enum SkipReason {
    NoExcess,
    AlreadyProcessed,
    WeekLocked,
    ToggleOff,
}
```

---

### `rest/src/rebooking.rs` (MODIFY — add POST /admin/rebooking/backfill)

**Analog:** `rest/src/rebooking.rs` existing handler `create_manual_rebooking` (lines 51–87)

**Imports pattern** (lines 20–36):
```rust
use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::post,
    Extension, Json, Router,
};
use rest_types::rebooking::{BackfillRequestTO, BackfillPreviewTO};
use service::rebooking_reconciliation::RebookingReconciliationService;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};
```

**Handler pattern** (lines 64–87 of existing create_manual_rebooking):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/backfill",
    tags = ["Rebooking"],
    params(
        ("from" = String, Query, description = "ISO week 'YYYY-Www'"),
        ("dry_run" = bool, Query, description = "Preview only if true"),
    ),
    responses(
        (status = 200, description = "Backfill preview or applied result", body = BackfillPreviewTO),
        (status = 400, description = "Invalid from-week format"),
        (status = 403, description = "Forbidden (nicht HR-Admin)"),
    ),
)]
pub async fn trigger_backfill<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    // Query params: from (YYYY-Www), dry_run (bool)
) -> Response {
    error_handler(
        (async {
            let preview = rest_state
                .rebooking_reconciliation_service()
                .backfill_range(from_week, dry_run, context.into(), None)
                .await?;
            let to = BackfillPreviewTO::from_domain(&preview, dry_run);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

**Route registration** — add to `generate_route()` or a new `generate_admin_route()`:
```rust
pub fn generate_admin_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/backfill", post(trigger_backfill::<RestState>))
}
```

---

### `rest-types/src/rebooking.rs` (MODIFY — add BackfillRequestTO, BackfillPreviewTO)

**Analog:** Existing TOs in `rest-types/src/rebooking.rs` (lines 56–129)

**TO pattern** (lines 56–65 for ManualRebookingRequestTO):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillRequestTO {
    pub from: String,   // "YYYY-Www"
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillPreviewItemTO {
    pub sales_person_id: Uuid,
    pub iso_year: u32,
    pub iso_week: u8,
    pub voluntary_ist: f32,
    pub voluntary_soll: f32,
    pub committed_voluntary: f32,
    pub excess: f32,
    pub would_book: f32,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BackfillPreviewTO {
    pub dry_run: bool,
    pub items: Vec<BackfillPreviewItemTO>,
    pub total_candidates: usize,
    pub already_processed: usize,
}
```

**#[cfg(feature = "service-impl")] From mapping pattern** (lines 28–54 for RebookingDirectionTO):
```rust
#[cfg(feature = "service-impl")]
impl BackfillPreviewTO {
    pub fn from_domain(items: &[service::rebooking_reconciliation::BackfillPreviewItem], dry_run: bool) -> Self { /* ... */ }
}
```

---

### `shifty_bin/src/main.rs` (MODIFY — DI wiring + boot)

**Analog:** `shifty_bin/src/main.rs` lines 1440–1471 (pdf_export_scheduler construction)

**Type alias pattern** (lines 401–415):
```rust
pub struct VoluntaryRebookingSchedulerDependencies;
impl VoluntaryRebookingSchedulerDeps for VoluntaryRebookingSchedulerDependencies {
    type Context = ...;
    type Transaction = ...;
    // ... all dep type assignments ...
}
type VoluntaryRebookingSchedulerService =
    VoluntaryRebookingSchedulerImpl<VoluntaryRebookingSchedulerDependencies>;
```

**Construction in RestStateImpl::new()** (line 1463–1471 pattern):
```rust
let voluntary_rebooking_scheduler = Arc::new(VoluntaryRebookingSchedulerService::new(
    rebooking_reconciliation_service.clone(),
    sales_person_service.clone(),
    toggle_service.clone(),
    week_status_service.clone(),
    permission_service.clone(),
    clock_service.clone(),
    transaction_dao.clone(),
));
```

**Boot in main() after pdf_export_scheduler** (lines 1614–1621 pattern):
```rust
// Existing:
rest_state.pdf_export_scheduler.start().await
    .expect("Expected the pdf-export scheduler to start");

// Add after:
rest_state
    .voluntary_rebooking_scheduler
    .start()
    .await
    .expect("Expected the voluntary rebooking scheduler to start");
```

**RestStateImpl struct field + accessor** (lines 794–958 pattern):
```rust
// In RestStateImpl struct:
voluntary_rebooking_scheduler: Arc<VoluntaryRebookingSchedulerService>,

// In RestStateDef impl:
type VoluntaryRebookingScheduler = VoluntaryRebookingSchedulerService;
fn voluntary_rebooking_scheduler(&self) -> Arc<Self::VoluntaryRebookingScheduler> {
    self.voluntary_rebooking_scheduler.clone()
}
```

---

### `shifty-dioxus/src/loader.rs` (MODIFY — add toggle + backfill loaders)

**Analog:** `shifty-dioxus/src/loader.rs` lines 962–982 (shortday_clipping toggle loaders)

**Toggle getter/setter pattern** (lines 962–982):
```rust
pub async fn get_voluntary_rebooking_active_from(
    config: Config,
) -> Result<Option<String>, ShiftyError> {
    Ok(api::get_toggle_value(config, "voluntary_rebooking_auto_active_from").await?)
}

pub async fn set_voluntary_rebooking_active_from(
    config: Config,
    value: Option<&str>,
) -> Result<(), ShiftyError> {
    match value {
        Some(v) => api::set_toggle_value(config, "voluntary_rebooking_auto_active_from", v).await?,
        None => api::clear_toggle_value(config, "voluntary_rebooking_auto_active_from").await?,
    }
    Ok(())
}
```

**Backfill trigger pattern** (analogous to `trigger_pdf_export_now` lines 1008–1010):
```rust
pub async fn trigger_rebooking_backfill(
    config: Config,
    from: &str,
    dry_run: bool,
) -> Result<BackfillPreviewTO, ShiftyError> {
    api::post_rebooking_backfill(&config, from, dry_run).await
}
```

---

### `shifty-dioxus/src/page/settings.rs` (MODIFY — new Admin-Card)

**Analog:** `shifty-dioxus/src/page/settings.rs` lines 717–808 (Card 2b: Short-day slot-clipping activation date)

**Signal initialization pattern** (lines 722–725):
```rust
let mut rb_date_str: Signal<String> = use_signal(String::new);
let mut rb_date_str_loaded_empty = use_signal(|| false);
let mut rb_save_result: Signal<Option<bool>> = use_signal(|| None);
let mut rb_saving = use_signal(|| false);
```

**use_resource + use_effect load pattern** (lines 727–743):
```rust
let config_for_rb_load = config.clone();
let rb_resource =
    use_resource(move || loader::get_voluntary_rebooking_active_from(config_for_rb_load.clone()));

use_effect(move || {
    match &*rb_resource.read_unchecked() {
        Some(Ok(Some(date))) => {
            rb_date_str.set(date.clone());
            rb_date_str_loaded_empty.set(false);
        }
        Some(Ok(None)) => {
            rb_date_str.set(String::new());
            rb_date_str_loaded_empty.set(true);
        }
        _ => {}
    }
});
```

**Save handler with spawn pattern** (lines 745–775):
```rust
let on_save_rb = move |_| {
    if *rb_saving.read() { return; }
    let val = rb_date_str.read().clone();
    if val.is_empty() { return; }
    rb_saving.set(true);
    rb_save_result.set(None);
    let cfg = config_for_rb_save.clone();
    spawn(async move {
        match loader::set_voluntary_rebooking_active_from(cfg, Some(&val)).await {
            Ok(()) => { rb_save_result.set(Some(true)); rb_date_str_loaded_empty.set(false); }
            Err(_) => { rb_save_result.set(Some(false)); }
        }
        rb_saving.set(false);
    });
};
```

**Backfill section** — add a dry-run form below the toggle date editor (no direct analog, use the save-handler spawn pattern above as base with extra `from` and `dry_run` inputs + preview table render).

---

### `shifty-dioxus/Dioxus.toml` (MODIFY — add proxy)

**Analog:** `Dioxus.toml` lines 106–107 (`/admin/cutover` proxy)

**Proxy entry pattern** (lines 106–108):
```toml
[[web.proxy]]
backend = "http://localhost:3000/admin/cutover"
```

**New entry to add** (after existing `/admin/impersonate` at line 108):
```toml
[[web.proxy]]
backend = "http://localhost:3000/admin/rebooking"
```

---

## Shared Patterns

### Authentication (Full + HR-Admin gate)
**Source:** `service_impl/src/pdf_export_scheduler.rs` lines 292–298 + `rest/src/rebooking.rs` line 69
**Apply to:** All new service methods, all new REST handlers
```rust
// Cron-path: Full authentication (trusted caller)
self.permission_service.check_only_full_authentication(context.clone()).await?;

// REST handler: HR-Admin gate lives inside the service; REST just delegates via error_handler
error_handler(async { rest_state.service().method(context.into(), None).await? }.await)
```

### Error Handling (error_handler wrapper)
**Source:** `rest/src/rebooking.rs` line 69–87
**Apply to:** All new REST handlers in `rest/src/rebooking.rs`
```rust
error_handler(
    (async {
        // ... service call ...
        Ok(Response::builder().status(200)/* or 204 */.body(Body::empty()).unwrap())
    })
    .await,
)
```

### Transaction Management
**Source:** `service_impl/src/rebooking_reconciliation.rs` (existing; same pattern as all service_impl files)
**Apply to:** `run_for_week` and `backfill_range` implementations
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... DAO calls with Some(tx.clone()) ...
self.transaction_dao.commit(tx).await?;
Ok(result)
```

### Stichtag Toggle Guard (Chain-Audit D-06)
**Source:** `service_impl/src/shortday_gate.rs` lines 105–117
**Apply to:** All six consumer chains: `reporting.rs`, `voluntary_stats.rs` (F1-Ist), `voluntary_stats.rs` (F2), `voluntary_rebooking_scheduler.rs` (F4-Self), `billing_period_report.rs`, `booking_information.rs`
```rust
let active_from = voluntary_rebooking_gate::read_active_from(
    self.toggle_service.as_ref(),
    context.clone(),
).await?;
if !voluntary_rebooking_gate::should_run_for_week(iso_year, iso_week, active_from) {
    // return legacy_result — Rebooking-ExtraHours invisible for pre-Stichtag weeks
}
```

### UNIQUE-Constraint Idempotency
**Source:** `dao_impl_sqlite/src/rebooking_batch.rs` lines 162–163, 421–442
**Apply to:** `run_for_week` and `backfill_range` in `rebooking_reconciliation.rs`
```rust
fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.is_unique_violation())
}
// Service: map DaoError::EntityAlreadyExists → no-op (continue loop)
Err(ServiceError::EntityAlreadyExists(_)) => { /* skip, continue */ }
```

### OpenAPI Annotation
**Source:** `rest/src/rebooking.rs` lines 52–63
**Apply to:** `trigger_backfill` handler + add to ApiDoc struct
```rust
#[utoipa::path(
    post,
    path = "/backfill",
    tags = ["Rebooking"],
    params(("from" = String, Query, ...), ("dry_run" = bool, Query, ...)),
    responses((status = 200, body = BackfillPreviewTO), (status = 403, ...), ...),
)]
```

---

## Golden-Snapshot Fixture Pattern (D-01 Beweislast)

**Source:** `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` (WOP-03 muster)
**Target:** `service_impl/src/test/billing_period_straddling.rs` (new file)

**Byte-identity assertion** (from RESEARCH Example 5):
```rust
#[track_caller]
fn assert_billing_period_bit_eq(
    actual: &BillingPeriodValueRow, expected: &BillingPeriodValueRow, idx: usize
) {
    assert!(
        !actual.value.is_nan(),
        "row[{idx}] value is NaN — golden-snapshot cannot compare deterministically"
    );
    assert_eq!(
        actual.value.to_bits(), expected.value.to_bits(),
        "BillingPeriod value_type={:?} row[{idx}] diverges: actual={} (0x{:08x}), expected={} (0x{:08x})",
        actual.value_type,
        actual.value, actual.value.to_bits(),
        expected.value, expected.value.to_bits(),
    );
}
```

**Three straddling scenarios:**
1. `active_from = Some(future_date)` — Rebooking invisible (pre-Stichtag) → must equal v2.5 baseline
2. `active_from = Some(past_date)` — Rebooking visible (post-Stichtag) → still byte-identical (balance-neutral)
3. `active_from = Some(mid_period_date)` — Half pre, half post → byte-identical

**Gate:** If any of the 3 scenarios fail byte-identity → bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` from 12 to 13 in `service_impl/src/billing_period_report.rs:117`.

---

## No Analog Found

All files have close analogs. No files without an existing codebase match.

---

## Anti-Patterns to Avoid (Critical)

| Anti-pattern | Consequence | Correct Pattern |
|---|---|---|
| 5-field cron `"0 3 * * 1"` | Job never registers (tokio_cron_scheduler 0.15 needs 6 fields) | Use `"0 15 3 * * 1"` |
| `today.year()` + `today.iso_week()` | Wrong ISO year at Jan 1-3 | Use `ShiftyDate::from_date(today).as_shifty_week()` |
| Omit Toggle-guard from one of 6 chains | `balance_before != balance_after`, fails neutrality tests | Check all 6 chains in D-06 chain matrix |
| Skip Straddling-Fixture | D-01 Beweislast absent; version decision unvalidated | Write all 3 scenarios in `billing_period_straddling.rs` |
| Missing `[[web.proxy]]` for `/admin/rebooking` | 404 in `dx serve` dev mode | Add entry to Dioxus.toml |

---

## Metadata

**Analog search scope:** `service/src/`, `service_impl/src/`, `rest/src/`, `rest-types/src/`, `shifty_bin/src/`, `shifty-dioxus/src/`, `shifty-dioxus/Dioxus.toml`
**Files scanned:** ~15 analog files read (pdf_export_scheduler.rs, pdf_export.rs, shortday_gate.rs, rebooking.rs REST+types, main.rs, settings.rs, loader.rs, Dioxus.toml)
**Pattern extraction date:** 2026-07-26
