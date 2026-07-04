# Phase 49: pdf-download-button — Pattern Map

**Mapped:** 2026-07-03
**Files analyzed:** 15 (4 new backend + 6 modified backend + 4 modified frontend + 2 docs; 1 modified rest-workspace)
**Analogs found:** 15 / 15 (every file has a repo-native precedent)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service/src/pdf_shiftplan.rs` (NEW) | service-trait | request-response (binary) | `service/src/pdf_export.rs` | exact (both Business-Logic-Tier PDF traits w/ Context+Transaction generics + `#[automock]`) |
| `service_impl/src/pdf_shiftplan.rs` (NEW) | service-impl | transform (View + SalesPerson-filter + render) | `service_impl/src/pdf_export_scheduler.rs` §53–74 + §347–400 | exact (same `gen_service_impl!` shape, same assemble sequence) |
| `service_impl/src/test/pdf_shiftplan.rs` (NEW) | test | unit (mock-based) | `service_impl/src/test/pdf_export_scheduler.rs` | exact |
| `rest/src/pdf_shiftplan.rs` (NEW) | rest-handler | request-response (binary download) | `rest/src/pdf_export_config.rs` (module scaffold) + `rest/src/sales_person.rs::ical` handler (binary response w/ Content-Disposition) | role-match (nearest for PDF module; the binary-response pattern comes from the iCal handler) |
| `service/src/lib.rs` (MOD) | module-reg | n/a | existing `pub mod pdf_export;` entry | exact |
| `service_impl/src/lib.rs` (MOD) | module-reg | n/a | existing `pub mod pdf_export_scheduler;` entry | exact |
| `service_impl/src/test/mod.rs` (MOD) | module-reg | n/a | existing `mod pdf_export_scheduler;` entry | exact |
| `service_impl/src/pdf_export_scheduler.rs` (MOD §347–400 refactor) | service-impl | transform (delegate) | current in-file inline block being replaced | self-analog (excise 3 calls, replace with 1 `PdfShiftplanService::render_week_pdf` call) |
| `service_impl/src/test/pdf_export_scheduler.rs` (MOD) | test | unit (mock-based) | current file — swap Mock deps | self-analog |
| `rest/src/lib.rs` (MOD) | routing + api-doc + state-trait | n/a | pdf_export_config wiring (line 581 nest + line 650 route) + shiftplan-info nest | exact |
| `shifty_bin/src/main.rs` (MOD) | DI wiring | construction-order | existing PdfExportScheduler construction | exact |
| `shifty-dioxus/src/page/shiftplan.rs` (MOD §1123–1140) | frontend RSX | request-response (`<a href download>`) | iCal-Anchor `shiftplan.rs:1128–1138` | exact (same styling, same pattern, adjacent placement) |
| `shifty-dioxus/src/i18n/mod.rs` (MOD) | i18n enum | n/a | existing `Key::PersonalCalendarExport` at line 84 | exact |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` (MOD) | i18n locale | n/a | existing `Key::PersonalCalendarExport` at line 41 in each locale | exact |
| `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md` (MOD) | docs | n/a | previous phase-scoped edits in same files | n/a — plain markdown edit |

## Pattern Assignments

### `service/src/pdf_shiftplan.rs` (NEW — service-trait)

**Analog:** `service/src/pdf_export.rs` (lines 1–52)

**Header + Business-Logic-Tier doc-comment pattern** (analog lines 1–17):
```rust
//! On-demand PDF-Download-Service (Phase 49).
//!
//! Business-Logic-Tier per Service-Tier-Konvention: kombiniert
//! [`crate::shiftplan::ShiftplanViewService`] (Read-Aggregat) +
//! [`crate::sales_person::SalesPersonService`] +
//! [`crate::week_status::WeekStatusService`] + pure Rendering
//! (`service_impl::pdf_render`).
//!
//! Wird konsumiert vom REST-Handler `GET /shiftplan/{id}/{y}/{w}/pdf` und
//! (nach Phase-49-Refactor) vom Nextcloud-Scheduler — genau EIN Ort für die
//! Assemble-Logik (View + Filter + Render).
```

**Trait signature pattern** (analog lines 25–52 — copy structure verbatim, swap methods):
```rust
use std::fmt::Debug;
use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait PdfShiftplanService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn render_week_pdf(
        &self,
        shiftplan_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<u8>, ServiceError>;
}
```

---

### `service_impl/src/pdf_shiftplan.rs` (NEW — service-impl)

**Analog:** `service_impl/src/pdf_export_scheduler.rs` (lines 47–80 for `gen_service_impl!`, lines 347–400 for the assemble body)

**`gen_service_impl!` pattern** (analog lines 53–80 minus `custom_fields` block):
```rust
use crate::gen_service_impl;
use crate::pdf_render;
use dao::TransactionDao;
use service::{
    permission::Authentication,
    sales_person::SalesPersonService,
    shiftplan::ShiftplanViewService,
    week_status::{WeekStatus, WeekStatusService},
    PermissionService, ServiceError,
};
use std::sync::Arc;
use uuid::Uuid;

gen_service_impl! {
    struct PdfShiftplanServiceImpl: service::pdf_shiftplan::PdfShiftplanService = PdfShiftplanServiceDeps {
        ShiftplanViewService: ShiftplanViewService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = shiftplan_view_service,
        SalesPersonService: SalesPersonService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = sales_person_service,
        WeekStatusService: WeekStatusService<
            Context = Self::Context,
            Transaction = Self::Transaction,
        > = week_status_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Assemble-Body pattern (View + Active-Filter + Render)** — verbatim excerpt from `pdf_export_scheduler.rs:347–400` that WILL BE MOVED here:
```rust
// From pdf_export_scheduler.rs:347-355 — active-sales-person filter (COPY into new service):
let all_sales_persons = self
    .sales_person_service
    .get_all(context.clone(), tx.clone())        // ← was Authentication::Full, None — parameterise
    .await?;
let active_sales_persons: Vec<service::sales_person::SalesPerson> = all_sales_persons
    .iter()
    .filter(|sp| sp.deleted.is_none())
    .cloned()
    .collect();

// From pdf_export_scheduler.rs:365-381 — view fetch (COPY, drop the error-persistence branch):
let week_view = self
    .shiftplan_view_service
    .get_shiftplan_week(shiftplan_id, year, calendar_week, context.clone(), tx.clone())
    .await?;

// From pdf_export_scheduler.rs:383-400 — render call (COPY, drop error-persistence branch):
pdf_render::render_shiftplan_week_pdf(&week_view, &active_sales_persons, year, calendar_week)
```

**Defense-in-Depth WeekStatus-Gate** (new logic, no direct analog — RESEARCH.md §Pattern 1):
```rust
let status = self.week_status_service
    .get_week_status(year, calendar_week, context.clone(), tx.clone())
    .await?;
if !matches!(status, WeekStatus::Planned | WeekStatus::Locked) {
    return Err(ServiceError::ValidationError(Arc::from([
        service::ValidationFailureItem {
            field: Arc::from("week_status"),
            reason: Arc::from(format!(
                "Woche KW{calendar_week:02}/{year} ist im Status {status:?} — kein Download",
            )),
        },
    ])));
}
```

---

### `service_impl/src/test/pdf_shiftplan.rs` (NEW — unit tests)

**Analog:** `service_impl/src/test/pdf_export_scheduler.rs`

**Test structure to mirror:**
- `TestDeps` struct holding `MockShiftplanViewService`, `MockSalesPersonService`, `MockWeekStatusService`, `MockPermissionService`, `MockTransactionDao`.
- Test cases per CONTEXT.md D-49-Claude's-Discretion:
  1. `happy_path_returns_bytes` — WeekStatus=Planned, view + get_all mocked, expect `Ok(Vec<u8>)`.
  2. `unset_returns_conflict_err` — WeekStatus=Unset → expect `ServiceError::ValidationError`.
  3. `in_planning_returns_conflict_err` — analog.
  4. `active_filter_excludes_deleted` — inject sales_persons with `deleted.is_some()`, verify they don't reach `pdf_render` (via `render_shiftplan_week_pdf`-call arg-capture).

**Look up mock-setup boilerplate from:** `service_impl/src/test/pdf_export_scheduler.rs::boot_trigger_reload_flow` (test-harness pattern reference in RESEARCH.md Don't-Hand-Roll table).

---

### `rest/src/pdf_shiftplan.rs` (NEW — REST-handler)

**Analog for module scaffold + `PdfShiftplanApiDoc`:** `rest/src/pdf_export_config.rs` (lines 18–39, 50–71)

**Imports + `generate_route()` pattern** (analog lines 18–39):
```rust
use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use service::{
    pdf_shiftplan::PdfShiftplanService,
    week_status::{WeekStatus, WeekStatusService},
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route(
        "/{shiftplan_id}/{year}/{week}/pdf",
        get(download_week_pdf::<RestState>),
    )
}
```

**Handler pattern — `error_handler` wrapper + Response::builder** (structure from `pdf_export_config.rs:52–71`, binary-body + Content-Disposition addition):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}/pdf",
    tags = ["PdfShiftplan"],
    params(
        ("shiftplan_id" = Uuid, Path, description = "Shiftplan-ID"),
        ("year" = u32, Path, description = "ISO-Kalenderjahr"),
        ("week" = u8, Path, description = "ISO-Kalenderwoche"),
    ),
    responses(
        (status = 200, description = "PDF-Bytes", content_type = "application/pdf", body = Vec<u8>),
        (status = 401, description = "Nicht authentifiziert"),
        (status = 404, description = "Shiftplan nicht gefunden"),
        (status = 409, description = "WeekStatus ∈ {Unset, InPlanning}"),
        (status = 500, description = "Renderer-Fehler"),
    ),
)]
pub async fn download_week_pdf<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((shiftplan_id, year, week)): Path<(Uuid, u32, u8)>,
) -> Response {
    error_handler(
        (async {
            // Primary 409-gate — Handler-Level Pre-Check per RESEARCH.md Pitfall 3.
            let status = rest_state
                .week_status_service()
                .get_week_status(year, week, context.clone().into(), None)
                .await?;
            if !matches!(status, WeekStatus::Planned | WeekStatus::Locked) {
                return Ok(Response::builder()
                    .status(409)
                    .header("Content-Type", "application/json")
                    .body(Body::new(r#"{"error":"week-not-releasable"}"#.to_string()))
                    .unwrap());
            }

            let bytes = rest_state
                .pdf_shiftplan_service()
                .render_week_pdf(shiftplan_id, year, week, context.into(), None)
                .await?;

            let filename = format!("schichtplan-{year}-KW{week:02}.pdf");
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/pdf")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                )
                .body(Body::new(bytes))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "PdfShiftplan", description = "On-demand PDF-Download der Wochen-Ansicht (Phase 49)")),
    paths(download_week_pdf),
)]
pub struct PdfShiftplanApiDoc;
```

---

### `rest/src/lib.rs` (MOD — wiring)

**Analog:** existing `pdf_export_config` wiring — 4 touch points.

**Touch point A — `RestStateDef` trait extension** (analog: existing `PdfExportConfigService`/`PdfExportScheduler` associated types + accessor fns):
```rust
type PdfShiftplanService: service::pdf_shiftplan::PdfShiftplanService<Context = Context>
    + Send + Sync + 'static;

fn pdf_shiftplan_service(&self) -> Arc<Self::PdfShiftplanService>;
```

**Touch point B — `mod` declaration** (add next to existing `pub mod pdf_export_config;`):
```rust
pub mod pdf_shiftplan;
```

**Touch point C — ApiDoc-Nest** (analog line 581):
```rust
(path = "/shiftplan", api = pdf_shiftplan::PdfShiftplanApiDoc),
```

**Touch point D — `.nest()` in `start_server`** (analog line 650). Verified conflict-free (existing nests are `/shiftplan-info`, `/shiftplan-edit`, `/shiftplan-catalog` — line 654–656 — a bare `/shiftplan` is unused):
```rust
.nest("/shiftplan", pdf_shiftplan::generate_route())
```

---

### `service_impl/src/pdf_export_scheduler.rs` (MOD — DRY refactor §347–400)

**Refactor pattern — replace inline assemble with service call:**

Excise **verbatim** the current lines 347–400 block (the `all_sales_persons` filter + `shiftplan_view_service.get_shiftplan_week` + `pdf_render::render_shiftplan_week_pdf`) inside the `for offset in 0..horizon` loop, and replace with:

```rust
let bytes = match self
    .pdf_shiftplan_service
    .render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)
    .await
{
    Ok(b) => b,
    Err(e) => {
        let at = self.clock_service.date_time_now();
        let msg: Arc<str> = Arc::from(format!(
            "PDF-Assemble/Render für KW{w:02}/{y} fehlgeschlagen: {e}"
        ));
        self.pdf_export_config_service
            .record_error(at, msg, Authentication::Full, None)
            .await?;
        return Ok(());
    }
};
```

**`gen_service_impl!`-Deps update** (analog lines 53–80): DROP `ShiftplanViewService` (lines 59–62) and `SalesPersonService` (lines 67–70), ADD:
```rust
PdfShiftplanService: service::pdf_shiftplan::PdfShiftplanService<
    Context = Self::Context,
    Transaction = Self::Transaction,
> = pdf_shiftplan_service,
```
(Planner may keep `ShiftplanService` — catalog lookup for the first active shiftplan is upstream of the loop and unrelated.)

**RESEARCH.md Pitfall 2:** Planner MUST also update `service_impl/src/test/pdf_export_scheduler.rs::TestDeps` to drop the two obsolete mocks and add `MockPdfShiftplanService`. Rewire expectations that formerly targeted `get_shiftplan_week` + `get_all` to instead target `render_week_pdf`.

---

### `shifty_bin/src/main.rs` (MOD — DI wiring)

**Analog:** existing construction of `PdfExportSchedulerService`.

**Construction-order rule (D-49-09):** Basic → PdfShiftplanService → PdfExportScheduler.

**Excerpt to add** (see RESEARCH.md §"DI-Wiring in shifty_bin/src/main.rs"):
```rust
let pdf_shiftplan_service = Arc::new(
    service_impl::pdf_shiftplan::PdfShiftplanServiceImpl::<PdfShiftplanServiceDependencies> {
        shiftplan_view_service: shiftplan_view_service.clone(),
        sales_person_service: sales_person_service.clone(),
        week_status_service: week_status_service.clone(),
        permission_service: permission_service.clone(),
        transaction_dao: transaction_dao.clone(),
    },
);
// Scheduler constructor swap: pass pdf_shiftplan_service, drop shiftplan_view_service + sales_person_service args.
```

Plus `RestStateDef`-impl-block:
```rust
type PdfShiftplanService = service_impl::pdf_shiftplan::PdfShiftplanServiceImpl<PdfShiftplanServiceDependencies>;

fn pdf_shiftplan_service(&self) -> Arc<Self::PdfShiftplanService> {
    self.pdf_shiftplan_service.clone()
}
```

---

### `shifty-dioxus/src/page/shiftplan.rs` (MOD — new button §~1140)

**Analog:** iCal-Anchor at `shiftplan.rs:1128–1138` (verbatim excerpt read at lines 1123–1140):

```rust
if let Some(sp_id) = sales_person_id {
    a {
        class: "px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt",
        target: "_blank",
        href: format!("{}/sales-person/{}/ical", backend_url, sp_id),
        title: "{personal_label}",
        span { class: "font-mono", "↓" }
        "iCal"
    }
}
```

**Pattern to write** — direct sibling block, INSERTED between iCal block (line 1140) and the `is_shiftplanner`-Booking-Log-Button (line 1141). See UI-SPEC.md §"Component Contract — PDF-Download-Button" and RESEARCH.md §"Pattern 3":

```rust
{
    let backend_url_pdf = backend_url.clone();
    let pdf_label = i18n.t(Key::PdfDownload);
    let sp_id_opt = *selected_shiftplan_id.read();
    let y = *year.read();
    let w = *week.read();
    let ws = *week_status.read();
    rsx! {
        if should_show_pdf_button(ws, sp_id_opt) {
            a {
                class: "px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt",
                href: format!("{}/shiftplan/{}/{}/{}/pdf", backend_url_pdf, sp_id_opt.unwrap(), y, w),
                download: format!("schichtplan-{y}-KW{w:02}.pdf"),
                title: "{pdf_label}",
                span { class: "font-mono", "↓" }
                "{pdf_label}"
            }
        }
    }
}
```

**Pure predicate for cargo-testability** (MEMORY: „Dioxus Browser-Test: Datepicker" — programmatic date-input events don't fire signals, so use pure-fn tests):
```rust
pub fn should_show_pdf_button(status: WeekStatus, shiftplan_id: Option<Uuid>) -> bool {
    shiftplan_id.is_some() && matches!(status, WeekStatus::Planned | WeekStatus::Locked)
}
```
Unit-test placement: same module (`page/shiftplan.rs`) `#[cfg(test)] mod tests`. Analog test-shape lives across the repo in numerous pure-fn tests.

**Key differences from iCal analog** (LOAD-BEARING):
- Drop `target: "_blank"` (RESEARCH.md Anti-Pattern: `_blank` disables `download`).
- Add `download: format!("schichtplan-{y}-KW{w:02}.pdf")`.
- Guard by `WeekStatus` in addition to `Option::is_some()`.

---

### `shifty-dioxus/src/i18n/mod.rs` (MOD — enum extension)

**Analog:** `Key::PersonalCalendarExport` at line 84.

**Pattern:** add one enum variant `PdfDownload,` inside `pub enum Key { … }` block (starts at line 57).

---

### `shifty-dioxus/src/i18n/{de,en,cs}.rs` (MOD — one row per locale)

**Analog:** `Key::PersonalCalendarExport` mapping at line 41 in each locale file.

**Pattern:** add one `Key::PdfDownload => "PDF",` (or `"PDF".to_string()` — whatever the exact map-literal syntax is at line 41; match the analog verbatim). Same string „PDF" in all three per D-49-14.

---

### `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md` (MOD — same commit)

Not code — plain markdown edits per D-49-15/D-49-16. No pattern extraction needed. Planner must:
- REQUIREMENTS.md PDF-03: rephrase „aktuelle Kalenderwoche (basierend auf heute)" → „aktuell im UI selektierte Kalenderwoche".
- REQUIREMENTS.md Nicht-Ziele: strike „Wochenwahl über die UI-Navigation".
- ROADMAP.md Phase 49 Goal + SC 3: mirror the deviation.

---

## Shared Patterns

### 1. `error_handler` wrapper for all REST handlers
**Source:** `rest/src/pdf_export_config.rs:56–70` (structural `error_handler((async { … Ok(Response::builder()…) }).await)`).
**Apply to:** `rest/src/pdf_shiftplan.rs` — MUST use this wrapper so `ServiceError` → HTTP mapping stays consistent.

### 2. `#[utoipa::path]` + `#[instrument(skip(rest_state))]` on every handler
**Source:** `rest/src/pdf_export_config.rs:41–51, 73–80`.
**Apply to:** `download_week_pdf` in `rest/src/pdf_shiftplan.rs`. Non-negotiable per backend CLAUDE.md §"OpenAPI Documentation".

### 3. `gen_service_impl!` macro for every service-impl
**Source:** `service_impl/src/pdf_export_scheduler.rs:53–80`.
**Apply to:** `service_impl/src/pdf_shiftplan.rs`. Non-negotiable per backend CLAUDE.md §"Service Implementation".

### 4. Business-Logic-Tier construction order in `shifty_bin/src/main.rs`
**Source:** existing PdfExportScheduler construction (post-Basic).
**Apply to:** PdfShiftplanServiceImpl construction — MUST be after `shiftplan_view_service`, `sales_person_service`, `week_status_service` (all Basic-Tier) and BEFORE `pdf_export_scheduler` (new dep).

### 5. Anchor-Download-Button in Toolbar-Row
**Source:** `shifty-dioxus/src/page/shiftplan.rs:1128–1138` (iCal).
**Apply to:** new PDF-button block. Copy every Tailwind class verbatim (UI-SPEC.md §"Component Contract" locked exact string).

### 6. i18n three-locale invariant
**Source:** `Key::PersonalCalendarExport` — appears once at `i18n/mod.rs:84` + once at line 41 in each of `de.rs`, `en.rs`, `cs.rs`.
**Apply to:** every new key. Missing a locale is a compile error (exhaustive match).

### 7. WeekStatus-Enum via `matches!` (exhaustive on enum-extension)
**Source:** existing use throughout `shifty-dioxus/src/page/shiftplan.rs`.
**Apply to:** both backend (`PdfShiftplanServiceImpl::render_week_pdf`) and frontend (`should_show_pdf_button` predicate).

## No Analog Found

None. Every file in this phase has a repo-native precedent. The only "novel" surface is the pure predicate `should_show_pdf_button` on the frontend — trivially small and tested per project convention.

## Metadata

**Analog search scope:**
- `service/src/*.rs` (trait templates)
- `service_impl/src/*.rs` + `service_impl/src/test/*.rs`
- `rest/src/*.rs`
- `shifty-dioxus/src/page/shiftplan.rs`, `shifty-dioxus/src/i18n/*.rs`
- `shifty_bin/src/main.rs`

**Files scanned:** ~15 (targeted; every analog was pre-identified by RESEARCH.md — no broad Glob search needed).

**Pattern extraction date:** 2026-07-03
