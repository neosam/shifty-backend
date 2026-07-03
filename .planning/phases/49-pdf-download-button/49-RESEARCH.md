# Phase 49: pdf-download-button — Research

**Researched:** 2026-07-03
**Domain:** REST-Endpoint + FE-Anchor-Button + Business-Logic-Service (DRY-Refactor)
**Confidence:** HIGH

## Summary

Phase 49 fügt einen on-demand PDF-Download-Button in `shifty-dioxus/src/page/shiftplan.rs` hinzu, versteckt hinter WeekStatus ∈ {Planned, Locked}, backed durch einen neuen authentifizierten REST-Endpoint. Der Kern-Contribution ist **kein Feature-Neubau**, sondern eine **DRY-Refactor-Bewegung**: die Assemble-Blaupause aus `pdf_export_scheduler.rs:365–400` (View + SalesPerson-Filter + `pdf_render::render_shiftplan_week_pdf`) wird in einen neuen Business-Logic-Service `PdfShiftplanService` extrahiert; sowohl der neue REST-Handler als auch der bestehende Scheduler konsumieren diesen Service. **Keine neue Cargo-Dep, keine Migration, kein Snapshot-Bump, keine neue Datei-Struktur außer den 3 neuen Backend-Files** (service-trait, impl, rest-handler) + 2 Test-Files + FE-Änderungen in einer einzigen Datei.

Alle 16 Locked Decisions aus CONTEXT.md (D-49-01..D-49-16) sind mit existierenden Codebase-Mustern deckungsgleich implementierbar — es gibt einen exakten iCal-Anchor-Precedent für D-49-10/D-49-11, das `gen_service_impl!`-Muster für D-49-05, ein Nachbar-REST-Modul (`pdf_export_config.rs`) für D-49-02 und den Scheduler-Test als Blaupause für D-49-08.

**Primary recommendation:** Genau eine offene Design-Entscheidung: der 409-Signalisierungsweg (neuer `ServiceError`-Variant vs. Handler-Level-Gate). Planner sollte Handler-Level-Gate wählen — der Handler prüft `WeekStatusService` VOR dem Service-Aufruf, mappt {Unset, InPlanning} auf 409 mit einem hardcoded JSON-Body; damit bleibt die `ServiceError`-Enum sauber und der Service selbst muss keinen neuen Fehler-Typ einführen. Der Service prüft die Woche ZUSÄTZLICH (Defense-in-Depth), returned bei einem Race jedoch keinen speziellen Fehler — er nutzt einen bestehenden `ServiceError`-Variant (z.B. `ValidationError`) mit sprechender Meldung; Handler kann diesen Weg via zusätzliches Match-Arm ebenfalls auf 409 abbilden, ODER man akzeptiert im Race-Fall den default 422/400. Details in D-49-03/D-49-06.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**G1 — Endpoint-Shape & REST-Modul-Verortung**
- **D-49-01:** Endpoint = `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf`. Explizites `shiftplan_id` im Pfad (kein Query-Param, kein Hardcode). Weder Body noch Body-Optionen — GET liefert `application/pdf` mit `Content-Disposition: attachment; filename="schichtplan-{JJJJ}-KW{NN}.pdf"`.
- **D-49-02:** Eigenes REST-Modul `rest/src/pdf_shiftplan.rs` (Naming analog `pdf_export_config`). Mount via `generate_route()` in `rest/src/lib.rs`. Eigener `PdfShiftplanApiDoc`-Nest für Swagger.
- **D-49-03:** HTTP-Codes:
  - `200 application/pdf` bei Erfolg
  - `401` bei fehlender Auth
  - `404` wenn `shiftplan_id` nicht existiert (unverändert Basis-Verhalten des View-Service)
  - **`409 Conflict`** wenn `week_status ∈ {Unset, InPlanning}` (Defense-in-Depth, PDF-04)
  - `500` bei Renderer-Fehlern

**G4 — Selektierte KW entscheidet (Requirement-Deviation)**
- **D-49-04:** Deviation zu PDF-03 / SC 3: Button lädt die im UI selektierte KW (via `week`/`year`-Signals), nicht die heute-KW. Sichtbarkeit: Button sichtbar wenn `WeekStatus` der selektierten KW ∈ {Planned, Locked} — nutzt bestehenden `WEEK_STATUS_STORE`. REQUIREMENTS.md PDF-03 + Nicht-Ziel + ROADMAP-SC 3 werden im selben Commit umgeschrieben.

**G2 — DRY: PdfShiftplanService als Business-Logic-Service, Scheduler-Refactor**
- **D-49-05:** Neuer Business-Logic-Service `service::pdf_shiftplan::PdfShiftplanService`. Business-Logic-Tier, weil er `ShiftplanViewService` + `SalesPersonService` (+ `WeekStatusService`) konsumiert.
- **D-49-06:** Trait-API:
  ```rust
  async fn render_week_pdf(
      &self,
      shiftplan_id: Uuid,
      year: u32,
      calendar_week: u8,
      context: Authentication<Self::Context>,
      tx: Option<Self::Transaction>,
  ) -> Result<Vec<u8>, ServiceError>;
  ```
  Kümmert sich um: KW-Status-Vorprüfung (`WeekStatusService::get_week_status`), `ShiftplanViewService::get_shiftplan_week` (mit `context` des Aufrufers), Filter aktiver `SalesPerson`s (`deleted.is_none()`), Aufruf `pdf_render::render_shiftplan_week_pdf`. Rückgabe reine Bytes.
- **D-49-07:** Auth-Semantik: Service ruft konsumierte Services mit dem User-Context des Aufrufers auf (nicht `Authentication::Full`). Handler prüft nur „authentifiziert" (kein Admin-Gate, PDF-05).
- **D-49-08:** Scheduler-Refactor: `PdfExportScheduler::run_once_now` (§365–400) refactored — statt inline `shiftplan_view_service.get_shiftplan_week` + `sales_person_service` + `pdf_render::render_shiftplan_week_pdf` aufzurufen, ruft der Scheduler pro Iteration `PdfShiftplanService::render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)`. **Ein einziger Ort für die Assemble-Logik.** Scheduler bleibt zuständig für Cron-Schleife, WebDAV-Upload, Retry-Persistenz. `shiftplan_view_service`- und `sales_person_service`-Deps des Schedulers **entfallen** (oder bleiben, wenn anderswo noch nötig — Planner entscheidet via `grep`).
- **D-49-09:** DI-Wiring in `shifty_bin/src/main.rs`: `PdfShiftplanService` konstruiert **nach** `ShiftplanViewService` + `SalesPersonService` + `WeekStatusService` (Basic) und **vor** `PdfExportScheduler` (dessen Dep er neu wird).

**G3 — FE-Download-Mechanik & G5 — Button-Platzierung**
- **D-49-10:** FE-Button = simples `<a>`-Element (kein WASM-Fetch/Blob), analog iCal-Button in `shiftplan.rs:1123–1140`. Cookie-Auth durchgereicht vom Browser.
- **D-49-11:** Platzierung direkt neben iCal-Button (Toolbar-Row des Shiftplan-Headers). Gleiches Styling: `px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt`. Icon-Prefix `↓` (mono-Font), Label = i18n-Key `PdfDownload`. `target="_blank"` weglassen — `download`-Attribut reicht (`download="schichtplan-{yyyy}-KW{ww}.pdf"`).
- **D-49-12:** URL-Konstruktion:
  ```
  format!("{}/shiftplan/{}/{}/{}/pdf", backend_url, shiftplan_id, *year.read(), *week.read())
  ```
  Nutzt `selected_shiftplan_id` + `year` + `week` Signals. Bei `selected_shiftplan_id == None` → Button gar nicht rendern.
- **D-49-13:** Sichtbarkeit conditional:
  ```rust
  if let Some(sp_id) = selected_shiftplan_id.read().as_ref() {
      if matches!(*week_status.read(), WeekStatus::Planned | WeekStatus::Locked) {
          rsx! { a { href: …, download: …, "↓ {pdf_download_label}" } }
      }
  }
  ```
  Kein disabled, kein Tooltip, kein Fehler-Toast.
- **D-49-14:** i18n: **ein** neuer Key `PdfDownload` (de: „PDF", en: „PDF", cs: „PDF").

**Requirement-Deviation — Begleitende Doku-Updates**
- **D-49-15:** `.planning/REQUIREMENTS.md`:
  - PDF-03: „aktuelle Kalenderwoche (basierend auf heute)" → „aktuell im UI selektierte Kalenderwoche"
  - Nicht-Ziel „Wochenwahl über die UI-Navigation" streichen
  - PDF-04 unverändert
- **D-49-16:** `.planning/ROADMAP.md`:
  - Phase 49 Goal: „aktuelle Kalenderwoche (basierend auf heute)" → „aktuell im UI selektierte Kalenderwoche"
  - Phase 49 SC 3: „lädt IMMER die KW von heute" → Sichtbarkeit + Ziel-KW werden vom selektierten `week`/`year` gesteuert; Button versteckt (statt disabled)

### Claude's Discretion
- Naming: `PdfShiftplanService` vs. `PdfDownloadService`
- Test-Struktur: `service_impl/src/test/pdf_shiftplan.rs` (Unit, Mock-basiert) + optional End-to-End via `service_impl`-integration (siehe Muster `test/pdf_export_scheduler.rs::boot_trigger_reload_flow`)
- Scheduler-Refactor darf `shiftplan_view_service`- + `sales_person_service`-Deps von `PdfExportSchedulerDeps` streichen (falls nicht mehr anderweitig genutzt) — Planner entscheidet per `grep`
- OpenAPI: neuer `#[utoipa::path]` mit `responses(...)` inkl. 200 (binary), 409, 401, 404. Response-Body als raw `Vec<u8>` mit `content = "application/pdf"`
- Clippy-Gate wie üblich (`cargo clippy --workspace -- -D warnings`)

### Deferred Ideas (OUT OF SCOPE)
- Multi-Week-Batch-Download
- PDF-Preview im Browser statt Download
- Personal-PDF (nur eigene Bookings) — explizit Nicht-Ziel in REQUIREMENTS.md
- Fehler-Toast/Banner-UI (bewusst weggelassen)
- Loading-Spinner (Browser-Standard-UI reicht)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PDF-03 | Download-Button auf Schichtplan-Seite lädt PDF der aktuell im UI selektierten KW mit Dateiname `schichtplan-{JJJJ}-KW{NN}.pdf` | FE-Anchor-Button-Precedent (iCal §1123–1140); `selected_shiftplan_id`/`year`/`week`-Signals vorhanden; Backend-Endpoint via neuem `rest/src/pdf_shiftplan.rs` mit `Content-Disposition`-Header |
| PDF-04 | Frontend: Button nur sichtbar wenn WeekStatus ∈ {Planned, Locked}. Backend: HTTP 409 bei WeekStatus ∈ {Unset, InPlanning} (Defense-in-Depth) | FE nutzt bestehenden `WEEK_STATUS_STORE`; Backend-Gate in `PdfShiftplanService::render_week_pdf` via `WeekStatusService::get_week_status`; Handler-Level 409-Mapping über eigenen Match oder neuen ServiceError-Variant (siehe Pitfall 3) |
| PDF-05 | Kein Admin-Gate. Jeder authentifizierte User (Employee eingeschlossen) darf downloaden | Handler-Middleware `forbid_unauthenticated` reicht (siehe rest/src/lib.rs §691); kein zusätzliches `check_shiftplanner_privilege` oder Ähnliches |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| KW-Status-Vorprüfung + PDF-Assemble (View + SalesPerson-Filter + Render) | Business-Logic Service (`PdfShiftplanService`) | — | Konsumiert `ShiftplanViewService` + `SalesPersonService` + `WeekStatusService` → BL-Tier per CLAUDE.md §"Service-Tier-Konventionen" |
| PDF-Rendering (pure) | pure Modul (`service_impl/src/pdf_render.rs`) | — | Bereits vorhanden aus Phase 48; keine Änderung |
| Auth-Middleware (nur authentifiziert) | REST-Layer (bestehende Middleware) | — | `forbid_unauthenticated` läuft global — kein zusätzlicher Auth-Check im Handler |
| WeekStatus-Race-Gate → 409-Mapping | REST-Handler (`rest/src/pdf_shiftplan.rs`) | Business-Logic Service (Defense-in-Depth) | Handler prüft VOR Service-Aufruf; Service prüft nochmals — beide Wege liefern 409 |
| Cron-Schleife + WebDAV-Upload + Retry-Persistenz | Business-Logic Service (`PdfExportScheduler`) | — | Bleibt unverändert; konsumiert neu `PdfShiftplanService` statt inline zu orchestrieren |
| DI-Wiring | `shifty_bin/src/main.rs` | — | Konstruktionsreihenfolge: Basic (SalesPerson/View/WeekStatus) → PdfShiftplanService → PdfExportScheduler |
| FE-Button-Rendering (RSX + Sichtbarkeits-Guard) | Frontend (`shifty-dioxus/src/page/shiftplan.rs`) | — | Reine RSX-Änderung an existierender Toolbar-Row; kein neuer Component-File nötig |
| i18n-Label | Frontend i18n (`shifty-dioxus/src/i18n/{mod,de,en,cs}.rs`) | — | Neuer Key `PdfDownload` in Enum + 3 Übersetzungen |
| Requirements-/Roadmap-Deviations | GSD-Planning-Docs (`.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`) | — | Im selben Commit wie Code-Änderungen (D-49-15/16) |

## Standard Stack

Diese Phase installiert **KEINE** neuen Cargo-Deps (D-49 CONTEXT + REQUIREMENTS.md Nicht-Ziele). Alle benötigten Kisten sind bereits im Workspace vorhanden.

### Core (bereits vorhanden — kein Install nötig)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | Workspace-Pinned | HTTP-Framework | Bereits universell in `rest/` genutzt |
| `utoipa` | Workspace-Pinned | OpenAPI-Annotations | `#[utoipa::path]`-Standard in allen REST-Modulen |
| `async-trait` | Workspace-Pinned | async fn in traits | Standard-Muster in `service/*.rs` |
| `mockall` | Workspace-Pinned | Trait-Mocks | `#[automock]` an jedem Service-Trait |
| `uuid` | Workspace-Pinned | UUID-Typen | Standard-ID-Typ |
| `printpdf` | 0.7 (Phase 48) | PDF-Rendering | Bereits konsumiert von `pdf_render.rs` — wird 1:1 wiederverwendet |
| `dioxus` | 0.6.3 (Frontend) | RSX-Framework | Frontend-Standard; `<a>`-Element-Rendering trivial |

### Alternatives Considered (rejected)
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `<a href download>` FE-Anchor | reqwest + Blob-URL im WASM | Anchor-Weg funktioniert schon für iCal-Export → Symmetrie, keine Blob-Zwischenspeicherung im WASM-Heap, weniger Code |
| `PdfShiftplanService` als Business-Logic | Direkter Renderer-Aufruf im REST-Handler | Bricht DRY (Scheduler + Handler duplizieren die Assemble-Logik); kein Ort für den 409-Gate (Race-Protection); Test-Duplikation |
| Neue `ServiceError::WeekNotReleasable`-Variant | Handler-Level Pre-Check + generischer Service-Return | Neue Variante erfordert Match-Arm-Update im gesamten `error_handler` — mehr Files-Blast-Radius; Handler-Pre-Check ist chirurgischer |
| Neue Cargo-Dep für Content-Disposition-RFC-5987 | Handrolled UTF-8-ASCII-Header | Filename ist reines `[a-z0-9-]` (Format `schichtplan-2026-KW27.pdf`) — RFC-5987-Encoding nicht nötig; hand-rolled Header ist sicher |

**Installation:** Keine.

**Version verification:** Nicht anwendbar — keine neuen Packages.

## Package Legitimacy Audit

**Nicht anwendbar** — diese Phase installiert keine externen Packages. `printpdf` (Phase 48 legitimacy-audited), `axum`, `utoipa`, `async-trait`, `mockall`, `uuid`, `dioxus` sind alle bereits im Workspace-Cargo.toml. Kein neuer Registry-Zugriff im Rahmen dieser Phase.

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    shifty-dioxus/page/shiftplan.rs                        │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Toolbar-Row (§1123–1140)                                          │  │
│  │  ├─ iCal <a> (existing)                                            │  │
│  │  └─ PDF <a href download> (NEW)                                    │  │
│  │      guard: selected_shiftplan_id.is_some()                        │  │
│  │             && week_status ∈ {Planned, Locked}                     │  │
│  │      url:   {backend}/shiftplan/{sp_id}/{year}/{week}/pdf          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ HTTP GET (cookie-auth)
                                ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  Middleware: forbid_unauthenticated (bestehend, rest/lib.rs §691)        │
│    → 401 wenn nicht angemeldet                                            │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ Context: Authentication<User>
                                ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  rest/src/pdf_shiftplan.rs (NEW)                                          │
│  fn download_week_pdf(Path(sp_id, year, week), Extension(context))       │
│    ├─ error_handler wrapper (Standard-Pattern)                            │
│    ├─ [OPTIONAL Pre-Check] week_status_service.get_week_status()          │
│    │    → wenn ∈ {Unset, InPlanning}: return 409 + JSON                   │
│    ├─ pdf_shiftplan_service.render_week_pdf(sp_id, year, week, ctx, None)│
│    │    → Bytes                                                           │
│    └─ Response:                                                           │
│         status 200                                                        │
│         Content-Type: application/pdf                                     │
│         Content-Disposition: attachment; filename="…"                     │
│         Body: bytes                                                       │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  service_impl/src/pdf_shiftplan.rs (NEW) — Business-Logic-Tier            │
│  PdfShiftplanServiceImpl::render_week_pdf                                 │
│    1. week_status_service.get_week_status(y, w, ctx, tx)                 │
│       → wenn ∈ {Unset, InPlanning}: ServiceError::ValidationError(...)   │
│         (Defense-in-Depth; primärer Gate im REST-Handler)                 │
│    2. shiftplan_view_service.get_shiftplan_week(sp_id, y, w, ctx, tx)    │
│       → ShiftplanWeek (oder 404 wenn sp_id unbekannt)                    │
│    3. sales_person_service.get_all(ctx, tx)                              │
│       → filter(|sp| sp.deleted.is_none())                                 │
│    4. pdf_render::render_shiftplan_week_pdf(&week, &active, y, w)        │
│       → Vec<u8>                                                           │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ (SELBER Service wird auch vom Scheduler konsumiert)
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  service_impl/src/pdf_export_scheduler.rs (REFACTORED, §365–400 wird     │
│  ersetzt) — Cron-Loop-Iteration:                                          │
│  for offset in 0..horizon {                                               │
│      let bytes = pdf_shiftplan_service                                    │
│          .render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)│
│          .await?; ← EIN Aufruf statt 3 (view + get_all + render)          │
│      upload.upload_file(&folder, &filename, bytes).await?;                │
│  }                                                                        │
└──────────────────────────────────────────────────────────────────────────┘
```

**Kern-Property:** Der pure Renderer (`pdf_render::render_shiftplan_week_pdf`) hat GENAU EINEN Aufrufer nach dem Refactor: `PdfShiftplanService::render_week_pdf`. Sowohl REST-Handler als auch Scheduler gehen durch diesen Service.

### Recommended File Structure

**Neu:**
```
service/src/pdf_shiftplan.rs                       # Trait-Definition
service_impl/src/pdf_shiftplan.rs                  # Impl
service_impl/src/test/pdf_shiftplan.rs             # Unit-Tests (Mock-basiert)
rest/src/pdf_shiftplan.rs                          # REST-Handler + PdfShiftplanApiDoc
```

**Geändert:**
```
service/src/lib.rs                                 # pub mod pdf_shiftplan
service_impl/src/lib.rs                            # pub mod pdf_shiftplan
service_impl/src/test/mod.rs                       # mod pdf_shiftplan
service_impl/src/pdf_export_scheduler.rs           # §365–400 durch Service-Aufruf ersetzt; ggf. Deps abnehmen
rest/src/lib.rs                                    # mod, ApiDoc-nest, .nest("/shiftplan", …), RestStateDef-Erweiterung um PdfShiftplanService
shifty_bin/src/main.rs                             # PdfShiftplanServiceImpl konstruieren + Scheduler-Konstruktor-Params anpassen + State-Impl
shifty-dioxus/src/page/shiftplan.rs                # PDF-<a>-Block neben iCal-Button
shifty-dioxus/src/i18n/mod.rs                      # Key::PdfDownload
shifty-dioxus/src/i18n/{de,en,cs}.rs               # je 1 Übersetzung "PDF"
.planning/REQUIREMENTS.md                          # PDF-03 + Nicht-Ziel-Absatz (im selben Commit)
.planning/ROADMAP.md                               # Phase 49 Goal + SC 3 (im selben Commit)
```

### Pattern 1: Business-Logic-Service mit `gen_service_impl!`
**What:** Standard-Muster für neuen Service (siehe `pdf_export_scheduler.rs:53–80`).
**When to use:** Immer wenn ein neuer Service andere Services konsumiert.
**Example:**
```rust
// service/src/pdf_shiftplan.rs (Trait, analog service/src/pdf_export.rs)
use crate::{permission::Authentication, ServiceError};
use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use uuid::Uuid;

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

```rust
// service_impl/src/pdf_shiftplan.rs
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

#[async_trait::async_trait]
impl<Deps: PdfShiftplanServiceDeps + 'static>
    service::pdf_shiftplan::PdfShiftplanService for PdfShiftplanServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn render_week_pdf(
        &self,
        shiftplan_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Vec<u8>, ServiceError> {
        // Defense-in-Depth WeekStatus-Gate (primäres Gate im REST-Handler).
        let status = self
            .week_status_service
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

        let week = self
            .shiftplan_view_service
            .get_shiftplan_week(shiftplan_id, year, calendar_week, context.clone(), tx.clone())
            .await?;

        let all_sales_persons = self
            .sales_person_service
            .get_all(context, tx)
            .await?;
        let active: Vec<service::sales_person::SalesPerson> = all_sales_persons
            .iter()
            .filter(|sp| sp.deleted.is_none())
            .cloned()
            .collect();

        pdf_render::render_shiftplan_week_pdf(&week, &active, year, calendar_week)
    }
}
```
*Source:* Analog `service_impl/src/pdf_export_scheduler.rs:53–137` und `service/src/pdf_export.rs`.

### Pattern 2: REST-Handler mit Binary-Response + Content-Disposition
**What:** GET-Handler der Bytes zurückgibt.
**When to use:** Für Datei-Downloads.
**Example:**
```rust
// rest/src/pdf_shiftplan.rs
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
    Router::new()
        .route(
            "/{shiftplan_id}/{year}/{week}/pdf",
            get(download_week_pdf::<RestState>),
        )
}

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
        (status = 409, description = "WeekStatus ∈ {Unset, InPlanning} — kein Download"),
        (status = 500, description = "Interner Fehler beim Rendern"),
    ),
)]
pub async fn download_week_pdf<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((shiftplan_id, year, week)): Path<(Uuid, u32, u8)>,
) -> Response {
    error_handler(
        (async {
            // Primäres 409-Gate (Defense-in-Depth im Service).
            let status = rest_state
                .week_status_service()
                .get_week_status(year, week, context.clone().into(), None)
                .await?;
            if !matches!(status, WeekStatus::Planned | WeekStatus::Locked) {
                return Ok(Response::builder()
                    .status(409)
                    .header("Content-Type", "application/json")
                    .body(Body::new(
                        r#"{"error":"week-not-releasable"}"#.to_string(),
                    ))
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
    tags(
        (
            name = "PdfShiftplan",
            description = "On-demand PDF-Download der Wochen-Ansicht (Phase 49)",
        ),
    ),
    paths(download_week_pdf),
)]
pub struct PdfShiftplanApiDoc;
```
*Source:* Analog `rest/src/pdf_export_config.rs`.

### Pattern 3: FE-Anchor mit conditional Sichtbarkeit
**Example:**
```rust
// shifty-dioxus/src/page/shiftplan.rs — direkt nach dem iCal-<a>-Block, in der gleichen rsx!-Frame
{
    let backend_url_pdf = backend_url.clone();
    let pdf_label = i18n.t(Key::PdfDownload);
    let sp_id_opt = *selected_shiftplan_id.read();
    let y = *year.read();
    let w = *week.read();
    let ws = week_status.clone();
    rsx! {
        if let Some(sp_id) = sp_id_opt {
            if matches!(ws, WeekStatus::Planned | WeekStatus::Locked) {
                a {
                    class: "px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt",
                    href: format!("{}/shiftplan/{}/{}/{}/pdf", backend_url_pdf, sp_id, y, w),
                    download: format!("schichtplan-{y}-KW{w:02}.pdf"),
                    title: "{pdf_label}",
                    span { class: "font-mono", "↓" }
                    "{pdf_label}"
                }
            }
        }
    }
}
```
*Source:* Analog iCal-Anchor `shifty-dioxus/src/page/shiftplan.rs:1123–1140` mit Sichtbarkeits-Guard.

### Anti-Patterns to Avoid
- **`target="_blank"` + `download`** — Chrome/Firefox ignorieren `download` bei cross-origin oder wenn `_blank` genutzt wird; `_blank` weglassen (D-49-11).
- **Content-Disposition weglassen** — Browser nutzt sonst letztes Path-Segment (`pdf`) als Dateiname; unerlässlich (CONTEXT „Anti-Patterns").
- **`Authentication::Full` im REST-Handler** — der Handler MUSS den User-Context durchreichen (D-49-07), NUR der Scheduler ruft mit `Full`.
- **Neuer nest-Prefix `/shiftplan`** — konfliktfrei, aber Planner MUSS sanity-checken dass `.nest("/shiftplan", …)` in `rest/src/lib.rs::start_server` NICHT bereits vergeben ist (verified: nur `/shiftplan-info`, `/shiftplan-edit`, `/shiftplan-catalog` existieren — `/shiftplan` ist frei).
- **Blob-URL im WASM** — funktioniert, ist aber Overhead: iCal-Precedent nutzt reinen `<a>`, wir tun dasselbe.
- **`WEEK_STATUS_STORE` nicht aktualisiert wenn User in KW navigiert** — Planner muss verifizieren dass beim `year`/`week`-Signal-Change ein `WeekStatusAction::Load` gesendet wird (grep ergab: `shiftplan.rs:396, 579, 620` — 3 Trigger-Points; wenn KW-Navigation existiert, muss dort auch `WeekStatusAction::Load` gefeuert werden — verify via cargo-run).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PDF-Rendering | Neuen printpdf-Aufrufer | `pdf_render::render_shiftplan_week_pdf` | Phase 48 hat bereits einen deterministischen, getesteten Renderer |
| PDF-Assemble (View + Filter + Render) | Inline im REST-Handler ODER duplizieren zum Scheduler | Neuer `PdfShiftplanService` (Business-Logic-Tier) | D-49-05/D-49-08: Ein Ort für die Assemble-Logik; Scheduler ruft denselben Service |
| WeekStatus-Enum-Vergleich | String-Matching / manuelle Match-Arme | `matches!(status, WeekStatus::Planned \| WeekStatus::Locked)` | Enum existiert bereits mit 4 Varianten; `matches!` liefert exhaustive Compile-Time-Check bei Enum-Erweiterung |
| Sales-Person aktiv-Filter | Custom-Filter-Logik | `.iter().filter(\|sp\| sp.deleted.is_none()).cloned().collect::<Vec<_>>()` | Scheduler §351–355 nutzt exakt diese Zeile |
| Filename-Konvention | Neuer Format-String | `format!("schichtplan-{year}-KW{week:02}.pdf")` | Bereits konsistent mit Scheduler §402 und WebDAV-Export |
| REST-Test-Harness | Neuer Test-Setup | Bestehende `MockPdfShiftplanService` (via `#[automock]`) für Handler-Unit-Tests; `test/pdf_export_scheduler.rs::boot_trigger_reload_flow`-Muster für E2E | Beide Muster produktions-erprobt in Phase 48 |
| DTO für 409-Response | Neue rest-types-Struct | Hardcoded JSON-Literal im Handler (`r#"{"error":"week-not-releasable"}"#`) | Response-Body triviial; kein UI liest ihn (FE zeigt Button nur wenn Status stimmt); rest-types-Import-Bloat vermeiden |
| i18n-Runtime-Language-Switch | Neuer Mechanismus | Bestehender `i18n.t(Key::PdfDownload)`-Pattern | Bereits konsistent mit `PersonalCalendarExport` und ~1000 anderen Keys |

**Key insight:** Diese Phase ist zu 90% ein **Wiring-Job** (existierende Bausteine verkabeln) und zu 10% ein DRY-Refactor. Es gibt für jeden Baustein einen expliziten Precedent im Repo — der Planner sollte in seiner Plan-Struktur die Verweise aufs Precedent zeigen und nicht neu ausdenken.

## Runtime State Inventory

Diese Phase ist eine reine Additive-Change (neuer Endpoint + neuer Service + FE-Button + DRY-Refactor am Scheduler). Kein Rename, keine Migration, keine Runtime-State-Änderung. Die Kategorien:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — verifiziert: kein Datenmodell-Change, keine Migration in `migrations/sqlite/` nötig | keine |
| Live service config | None — verifiziert: `pdf_export_config`-Tabelle (Phase 48) bleibt unverändert; keine neuen Feature-Flags oder Toggles | keine |
| OS-registered state | None — verifiziert: der bestehende `tokio-cron-scheduler`-Job aus Phase 48 wird refactored (er ruft neuen Service statt inline zu orchestrieren), aber die Cron-Registrierung selbst wird nicht neu registriert. Neustart des Prozesses aktiviert den Refactor. | Prozess-Restart post-deploy (Standard) |
| Secrets/env vars | None — verifiziert: kein neuer Secret, keine ENV-Var-Änderung | keine |
| Build artifacts | None — verifiziert: `printpdf` bereits im Workspace-Cargo.toml; kein neuer Egg-Info-artiger Cache | Standard-`cargo build` |

## Common Pitfalls

### Pitfall 1: Router-Kollision mit `/shiftplan`
**What goes wrong:** Neuer `.nest("/shiftplan", pdf_shiftplan::generate_route())` kollidiert mit bestehendem `shiftplan_info` (`/shiftplan-info`), `shiftplan_edit`, `shiftplan_catalog`.
**Why it happens:** Prefix-Matching von Axum-Router — bei überlappenden Prefixes gewinnt der zuerst deklarierte.
**How to avoid:** Ich habe verifiziert (`rest/src/lib.rs` §653–656): die bestehenden Nests sind `/shiftplan-info`, `/shiftplan-edit`, `/shiftplan-catalog` — alle mit Bindestrich-Suffix. **`/shiftplan` (ohne Suffix) ist FREI** und kollidiert nicht.
**Warning signs:** `cargo test` mit einer neuen REST-Route-Klasse: wenn `GET /shiftplan-info/…`-Routes nicht mehr matchen, ist die Prefix-Kollision aktiv geworden.

### Pitfall 2: Scheduler-Refactor bricht `PdfExportSchedulerDeps`-Test-Setup
**What goes wrong:** Wenn der Planner die `shiftplan_view_service` + `sales_person_service` Deps aus `PdfExportSchedulerDeps` entfernt, aber der Test-File `service_impl/src/test/pdf_export_scheduler.rs` weiterhin `MockShiftplanViewService` + `MockSalesPersonService` als Deps setzt, bricht der Test-Build.
**Why it happens:** Die Tests konstruieren die `TestDeps`-Struct mit allen Feldern — Weglassen bricht die Impl.
**How to avoid:** Planner MUSS `grep -n "shiftplan_view_service\|sales_person_service" service_impl/src/pdf_export_scheduler.rs` VOR dem Refactor. Ergebnis heute (verifiziert): 3 Vorkommen in `pdf_export_scheduler.rs` (Zeile 62/70/117/119/127/129/347/366/477/479) — ALLE 3 dep-Einträge werden nach dem Refactor obsolet. Der Planner MUSS beide Dep-Zeilen aus `gen_service_impl!` + `new()`-Constructor + `test/pdf_export_scheduler.rs::TestDeps` entfernen; und `MockShiftplanViewService`+`MockSalesPersonService`-Expectations aus den Test-Cases abbauen (weil der Scheduler nichts mehr direkt aufruft — das macht der neu injizierte `MockPdfShiftplanService`).
**Warning signs:** `cargo test -p service_impl` schlägt fehl mit `missing field` oder `unused field`.

**ALTERNATIVE Empfehlung:** Weniger invasiv wäre, `shiftplan_view_service`+`sales_person_service` in den Scheduler-Deps zu LASSEN (auch wenn nicht mehr genutzt) und nur die `run_once_now`-Iteration umzustellen. Damit bricht nichts im Test-Harness. Trade-off: 2 unused fields → clippy-Warning `dead_code`. Empfehlung Planner: entweder ganz raus + Test-File anpassen, ODER mit `#[allow(dead_code)]` markieren und in einem Follow-up cleanen. **Für v2.3-Scope: raus und Tests mit anpassen** — das ist der DRY-Punkt der Phase.

### Pitfall 3: 409-Signalisierungsweg (offen)
**What goes wrong:** Es gibt keinen generischen `ServiceError::Conflict(String)`-Variant. Die möglichen Signalwege sind:
1. Handler-Level Pre-Check (empfohlen) → Handler prüft `WeekStatusService`, mappt {Unset, InPlanning} auf 409-Response direkt, ohne den Service zu rufen.
2. Neuer `ServiceError::WeekNotReleasable { year, week }`-Variant + neuer 409-Match-Arm im `error_handler` in `rest/src/lib.rs`.
3. Missbrauch von `ServiceError::ValidationError(...)` — mappt auf 422, nicht 409 → Falscher HTTP-Code.
4. Missbrauch von `ServiceError::WeekLocked { year, week }` — semantisch falsch (Locked bedeutet FE kann nicht mehr editieren, aber Download IST erlaubt für Locked).

**Why it happens:** Die Race-Condition zwischen Signal-Update und Klick ist selten, aber muss laut D-49-03 mit 409 signalisiert werden. Kein bestehender `ServiceError`-Variant mappt sauber auf 409 für „week not releasable".
**How to avoid:** **Empfehlung Planner:** Weg (1) — Handler prüft VOR Service-Aufruf via `week_status_service()`, gibt bei Nicht-Releasable direkt 409 mit hardcoded JSON-Body zurück. Der Service prüft ZUSÄTZLICH (Defense-in-Depth im Business-Logic-Tier), aber im Race-Fall wirft er einen generischen `ServiceError::ValidationError` — der wird zu 422 gemappt und ist unerreichbar (weil der Handler-Pre-Check schon 409 gefeuert hat, außer bei einem echten Race zwischen Handler-Pre-Check und Service-Call, was <1ms Fenster hat und akzeptabel ist). **Der Handler-Pre-Check ist die Wahrheit für die 409-Response**; der Service-Gate ist Test-Assertion-Wert.
**Warning signs:** Integrationstest `week_status_unset_returns_409`: wenn 422 kommt statt 409, wird der Service-Gate benutzt statt Handler-Pre-Check.

### Pitfall 4: `WeekStatus` FE-Signal aktualisiert nicht bei KW-Navigation
**What goes wrong:** Der `WEEK_STATUS_STORE` wird gefeuert bei Mount/Reload — aber wenn der User via prev/next-Woche navigiert, bleibt der Store am alten KW-Status hängen. Button zeigt Sichtbarkeit basierend auf **falschem** Status.
**Why it happens:** Signal-Kette: `week`-Signal ändert sich → braucht ein `use_effect` das `WeekStatusAction::Load { year, week }` feuert.
**How to avoid:** Grep-verifiziert: es gibt in `shifty-dioxus/src/page/shiftplan.rs` bereits 3 `WeekStatusAction::Load`-Trigger (Zeilen 396, 579, 620). Planner MUSS im Rahmen des FE-Plans **verifizieren**, dass beim Wechsel von `week`/`year` ein `WeekStatusAction::Load` gesendet wird. Falls ja: keine FE-Änderung an der Store-Logik nötig. Falls nein (Signal wird nicht bei KW-Wechsel gefeuert): das ist ein bestehender FE-Bug, dessen Fix in den Plan aufgenommen werden muss.
**Warning signs:** Browser-Test: navigiere von Planned-KW zu InPlanning-KW; wenn Button sichtbar bleibt, feuert die Store nicht auf KW-Wechsel.

### Pitfall 5: Content-Disposition-Filename-Encoding
**What goes wrong:** Sonderzeichen im Filename brechen den Header. Bei unserer Konvention (`schichtplan-2026-KW27.pdf`) sind alle Zeichen ASCII → kein RFC-5987-Encoding nötig.
**Why it happens:** Nur relevant wenn der Filename dynamisch aus Sales-Person-Namen o.ä. käme.
**How to avoid:** Filename ist deterministisch aus `{year}` (u32) + `{week:02}` (u8 mit Leading Zero) — beides pure ASCII. Trivial safe.
**Warning signs:** N/A.

## Code Examples

### Wiring the new service into `RestStateDef` (rest/src/lib.rs)
```rust
// Extend the trait
type PdfShiftplanService: service::pdf_shiftplan::PdfShiftplanService<Context = Context>
    + Send + Sync + 'static;

fn pdf_shiftplan_service(&self) -> Arc<Self::PdfShiftplanService>;

// Extend ApiDoc nest
(path = "/shiftplan", api = pdf_shiftplan::PdfShiftplanApiDoc),

// Extend start_server routing
.nest("/shiftplan", pdf_shiftplan::generate_route())
```
*Source:* Muster analog `pdf_export_config` in gleicher Datei (Zeile 581 + 650).

### DI-Wiring in `shifty_bin/src/main.rs`
```rust
// Nach shiftplan_view_service (Z.1118), sales_person_service (Z.870), week_status_service (Z.1081)
// aber VOR pdf_export_scheduler (Z.1212):
let pdf_shiftplan_service = Arc::new(
    service_impl::pdf_shiftplan::PdfShiftplanServiceImpl::<PdfShiftplanServiceDependencies> {
        shiftplan_view_service: shiftplan_view_service.clone(),
        sales_person_service: sales_person_service.clone(),
        week_status_service: week_status_service.clone(),
        permission_service: permission_service.clone(),
        transaction_dao: transaction_dao.clone(),
    },
);

// Scheduler-Konstruktor neu:
let pdf_export_scheduler = Arc::new(PdfExportSchedulerService::new(
    pdf_export_config_service.clone(),
    pdf_shiftplan_service.clone(),        // NEU
    shiftplan_service.clone(),             // Catalog (weiterhin für "erster aktiver Shiftplan")
    permission_service.clone(),
    clock_service.clone(),
    transaction_dao.clone(),
    Arc::new(ProductionWebDavUploadFactory) as Arc<dyn WebDavUploadFactory>,
));
```
Und im `RestStateDef`-Impl:
```rust
type PdfShiftplanService = service_impl::pdf_shiftplan::PdfShiftplanServiceImpl<
    PdfShiftplanServiceDependencies,
>;

fn pdf_shiftplan_service(&self) -> Arc<Self::PdfShiftplanService> {
    self.pdf_shiftplan_service.clone()
}
```

### Scheduler-Refactor — konkretes Ersetzen von §365–400
```rust
// PRE (aktuell §365–400):
let week_view = match self.shiftplan_view_service.get_shiftplan_week(...).await { ... };
let bytes = match pdf_render::render_shiftplan_week_pdf(&week_view, &active_sales_persons, y, w) { ... };

// POST (nach Refactor):
let bytes = match self
    .pdf_shiftplan_service
    .render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)
    .await
{
    Ok(b) => b,
    Err(e) => {
        let at = self.clock_service.date_time_now();
        let msg: Arc<str> = Arc::from(format!(
            "PDF-Rendering für KW{w:02}/{y} fehlgeschlagen: {e}"
        ));
        self.pdf_export_config_service
            .record_error(at, msg, Authentication::Full, None)
            .await?;
        return Ok(());
    }
};
```
Zusätzlich entfallen die vorgezogenen `all_sales_persons`/`active_sales_persons`-Berechnungen §347–355 (der Service macht sie jetzt).

**Achtung:** Der Scheduler ruft den `PdfShiftplanService` mit `Authentication::Full` auf, damit der Service die eingehenden Aufrufe an `WeekStatusService` + `ShiftplanViewService` + `SalesPersonService` ebenfalls mit `Full` autorisiert. Der `render_week_pdf`-Contract sagt "context des Aufrufers"; für den Scheduler ist der Aufrufer `Full`. Kein Widerspruch mit D-49-07.

**Wichtig — WeekStatus-Gate im Scheduler-Kontext:** Der `render_week_pdf`-Service prüft immer den WeekStatus. Für den Scheduler bedeutet das: unter dem alten Verhalten wurden ALLE KWs im horizon exportiert, unter dem neuen Verhalten werden nur `Planned/Locked`-KWs exportiert. **Das ist eine Verhaltensänderung** — Planner MUSS entscheiden ob (a) der Scheduler die WeekStatus-Prüfung überspringen darf (dann muss `render_week_pdf` einen optionalen "skip_gate"-Parameter kriegen ODER der Service den Gate NICHT einbauen und der Handler prüft nur) ODER (b) der Scheduler bewusst nur noch veröffentlichte Wochen exportiert (semantisch möglicherweise besser). **Empfehlung:** Weg (b) — nur veröffentlichte Wochen sind sinnvoll im WebDAV-Export; der Scheduler ignoriert Fehler pro Woche eh (`return Ok(())` bei Fehler). Aber das MUSS im Discuss-Phase-Deliverable D-49-06/D-49-08 nochmal explizit sein. **Für den Planner: Bitte user-clarify vor Coding.** Wenn Fallback: In Phase 49 den Gate ausschließlich im Handler prüfen (nicht im Service) und für den Service-internen Gate einen `skip_week_status_check`-Boolean-Parameter einfügen. Ich hänge das an Open Questions.

### FE-Signal-Read + i18n
```rust
// In shifty-dioxus/src/page/shiftplan.rs, oben in der Component (nach Z.167):
let pdf_download_str = i18n.t(Key::PdfDownload);

// i18n/mod.rs — nach Zeile 84 (PersonalCalendarExport):
PdfDownload,

// i18n/de.rs — analog PersonalCalendarExport-Block:
i18n.add_text(Locale::De, Key::PdfDownload, "PDF");

// i18n/en.rs:
i18n.add_text(Locale::En, Key::PdfDownload, "PDF");

// i18n/cs.rs:
i18n.add_text(Locale::Cs, Key::PdfDownload, "PDF");
```

## State of the Art

**Nicht anwendbar** — diese Phase touched keinen fast-moving-ecosystem-Code (kein neuer Cargo-Dep, kein Framework-Upgrade, keine neue API-Version). Alle konsumierten Crates sind bereits im Workspace pinned. Zeitloses "Wiring + DRY-Refactor"-Muster.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Der `WEEK_STATUS_STORE` feuert `WeekStatusAction::Load` bei KW-Navigation | Pitfall 4 | Wenn falsch: FE-Button zeigt Sichtbarkeit basierend auf falschem WeekStatus. Fix: `use_effect` an `week`/`year`-Signals einfügen. |
| A2 | Scheduler soll nur `Planned/Locked`-KWs exportieren (nicht `Unset/InPlanning`) | Code Examples §Scheduler-Refactor + Open Question Q1 | Wenn falsch (User will alle KWs weiter exportiert): `render_week_pdf` braucht `skip_week_status_check`-Parameter ODER der Gate lebt nur im Handler. |
| A3 | Der Handler-Pre-Check gegen `WeekStatusService` ist die Primär-Quelle für 409 (Service-Gate ist Defense-in-Depth mit 422/ValidationError) | Pitfall 3 + Code Examples §REST-Handler | Wenn falsch (User will Service als Primär-Quelle für 409): braucht neuen `ServiceError::WeekNotReleasable`-Variant + neuen Match-Arm im `error_handler`. Blast-Radius: 2 zusätzliche Files (`service/src/lib.rs` + `rest/src/lib.rs`). |
| A4 | Der bestehende iCal-Anchor-Precedent nutzt exakt dieses Styling und ist die richtige Vorlage | Pattern 3 | Wenn User später anderes Icon/Layout will: trivial nachträglich; kein Blocker. |
| A5 | Filename-Konvention `schichtplan-{year}-KW{week:02}.pdf` bleibt konsistent mit Phase 48 WebDAV-Export | Scheduler-Refactor + Code Examples | Verifiziert via `pdf_export_scheduler.rs:402` — dieselbe Konvention. |

## Open Questions (RESOLVED)

**Resolved während Plan-Phase 49 (Orchestrator-Directive, verankert in CONTEXT.md D-49-06/D-49-07/D-49-08).**

1. **Soll der Scheduler weiterhin ALLE KWs (auch Unset/InPlanning) exportieren, oder soll er nach dem Refactor nur noch `Planned/Locked` exportieren?**
   - What we know: Aktuell exportiert der Scheduler alle KWs im horizon. Der neue `PdfShiftplanService::render_week_pdf` prüft laut D-49-06 den WeekStatus.
   - What's unclear: Ob der Scheduler den Gate als Feature oder als Bug erlebt.
   - Recommendation: Weg (b) — nur veröffentlichte Wochen exportieren.
   - **RESOLVED:** Weg (b). Der Scheduler exportiert nach dem Refactor NUR `Planned/Locked`-KWs. Konsequenz von D-49-06 (Service prüft Gate) + D-49-08 (Scheduler nutzt Service). Der Scheduler ignoriert Fehler pro Woche (`return Ok(())` bei Fehler) — WeekStatus-Rejections werden zu normalen Skips. Dokumentiert in Plan 03 `must_haves.truths` unter D-49-08 (Q1). Keine Service-API-Änderung, kein `skip_week_status_check`-Parameter.

2. **Räumen wir `shiftplan_view_service` + `sales_person_service` aus den `PdfExportSchedulerDeps` nach dem Refactor komplett aus, oder lassen wir sie als `#[allow(dead_code)]` drin?**
   - What we know: Nach dem Refactor werden beide nicht mehr direkt vom Scheduler aufgerufen.
   - What's unclear: Ob Planner beim Refactor auch die Test-File `test/pdf_export_scheduler.rs::TestDeps` anpassen mag (bricht sonst der Test-Build).
   - Recommendation: Ganz raus.
   - **RESOLVED:** Vollständig entfernen — beide Deps raus aus `PdfExportSchedulerDeps` + `new()`-Constructor + `test/pdf_export_scheduler.rs::TestDeps`. Test-File-Anpassung liegt im selben Plan (Plan 03) wie der Scheduler-Refactor. Dokumentiert in Plan 03 `must_haves.truths` unter D-49-08 (Q2).

3. **[aus Pitfall 3 abgeleitet] 409-Signalisierungsweg — Handler-Pre-Check oder neue `ServiceError`-Variant?**
   - **RESOLVED:** Handler-Pre-Check via `WeekStatusService::get_week_status()` ist Primär-Quelle für 409 (chirurgischer Weg, kein neuer `ServiceError`-Variant, kein neuer `error_handler`-Match-Arm). Der Service-interne WeekStatus-Gate (D-49-06) bleibt als Defense-in-Depth für Race-Cases (Status ändert sich zwischen Handler-Pre-Check und Service-Call). Dokumentiert in Plan 02 `must_haves.truths` unter D-49-03.

## Environment Availability

Nicht anwendbar für diese Phase — keine neuen CLI-Tools, keine neuen Services, keine neuen Runtime-Abhängigkeiten. Nur `cargo build` + `cargo test` + `cargo clippy` (bereits im Setup verfügbar) plus `dx serve` / `dx build --release` für Frontend-Verifikation.

Optional für Browser-UAT: `dx serve` (dioxus-cli 0.6.x — bereits pinned im flake.nix, siehe Memory „Frontend dx-Version-Pin").

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (workspace default; Backend + Frontend eigene Workspaces) |
| Config file | keiner — nutzt cargo defaults; Nix-Build läuft zusätzlich `cargo clippy -- --deny warnings` |
| Quick run command | `cargo test -p service_impl pdf_shiftplan -- --nocapture` |
| Full suite command | Backend: `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` — Frontend: `cd shifty-dioxus && cargo test` |
| WASM-Build-Gate | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PDF-03 | REST GET `/shiftplan/{id}/{y}/{w}/pdf` liefert 200 + `application/pdf` + Content-Disposition-Header | REST-Integration (Axum-Router + Mock-Services) | `cargo test -p service_impl pdf_shiftplan::rest_returns_200_with_headers -- --nocapture` | ❌ Wave 0: neuer Test-File |
| PDF-03 | `PdfShiftplanService::render_week_pdf` liefert Bytes bei happy-path | Unit (Mock-basiert) | `cargo test -p service_impl pdf_shiftplan::happy_path_returns_bytes` | ❌ Wave 0 |
| PDF-03 | `PdfShiftplanService` filtert `deleted.is_some()` SalesPersons raus | Unit (Mock-basiert; Assert dass `pdf_render` mit gefilterter Liste aufgerufen wird — Assertion via Rekonstruktion aus Bytes bzw. Zwischen-Struct) | `cargo test -p service_impl pdf_shiftplan::filters_deleted_sales_persons` | ❌ Wave 0 |
| PDF-03 | Filename-Format `schichtplan-{yyyy}-KW{ww}.pdf` — Content-Disposition-Header korrekt | REST-Integration | `cargo test -p service_impl pdf_shiftplan::content_disposition_filename_format` | ❌ Wave 0 |
| PDF-04 | Backend: WeekStatus Unset → 409 | REST-Integration | `cargo test -p service_impl pdf_shiftplan::week_status_unset_returns_409` | ❌ Wave 0 |
| PDF-04 | Backend: WeekStatus InPlanning → 409 | REST-Integration | `cargo test -p service_impl pdf_shiftplan::week_status_in_planning_returns_409` | ❌ Wave 0 |
| PDF-04 | Backend: WeekStatus Planned → 200 | REST-Integration | `cargo test -p service_impl pdf_shiftplan::week_status_planned_returns_200` | ❌ Wave 0 |
| PDF-04 | Backend: WeekStatus Locked → 200 | REST-Integration | `cargo test -p service_impl pdf_shiftplan::week_status_locked_returns_200` | ❌ Wave 0 |
| PDF-04 | Backend: Race-Case — Service-internal Gate feuert bei Status-Change zwischen Handler und Service (Defense-in-Depth) | Unit (Mock) | `cargo test -p service_impl pdf_shiftplan::service_defense_in_depth_rejects` | ❌ Wave 0 |
| PDF-04 | Frontend: pure Predikat `should_show_pdf_button(week_status, shiftplan_id: Option<Uuid>)` — 8 Kombis (4 Status × 2 shiftplan_id-Zustände) | Frontend cargo-Test | `cd shifty-dioxus && cargo test should_show_pdf_button` | ❌ Wave 0: neue Fn + Testfile |
| PDF-05 | Backend: Employee-Auth-Context → 200 (kein Admin-Gate) | REST-Integration mit non-admin `Authentication::Authenticated` | `cargo test -p service_impl pdf_shiftplan::employee_auth_returns_200` | ❌ Wave 0 |
| PDF-05 | Backend: fehlende Auth → 401 (Middleware) | REST-Integration | `cargo test -p service_impl pdf_shiftplan::unauthenticated_returns_401` | ❌ Wave 0 |
| DRY-Refactor | Scheduler ruft `PdfShiftplanService` mit `Authentication::Full` (nicht mehr direkt View+SalesPerson+Render) | Bestehende Scheduler-Tests anpassen (`service_impl/src/test/pdf_export_scheduler.rs`) — Mock von `PdfShiftplanService` statt View+SalesPerson | `cargo test -p service_impl pdf_export_scheduler` | ✅ existiert, muss ANGEPASST werden |

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl pdf_shiftplan` (backend) + `cargo test -p rest` (falls REST-Test dorthin wandert) — <30s zusammen
- **Per wave merge:** `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` + Frontend `cargo test` + WASM-Build-Gate
- **Phase gate:** Volle Suite grün + Browser-UAT-Klick (manuell) + Nix-Build (implizit CI)

### Wave 0 Gaps
- [ ] `service/src/pdf_shiftplan.rs` — Trait-Definition (RED-first: leerer Trait, dann Impl-Zug)
- [ ] `service_impl/src/pdf_shiftplan.rs` — Impl
- [ ] `service_impl/src/test/pdf_shiftplan.rs` — 8 Unit-Tests
- [ ] `service_impl/src/test/mod.rs` — `mod pdf_shiftplan;`
- [ ] `rest/src/pdf_shiftplan.rs` — Handler + ApiDoc
- [ ] REST-Integration-Tests: entweder als in-memory-Setup in `service_impl/src/test/pdf_shiftplan.rs` (analog `test/pdf_export_scheduler.rs::boot_trigger_reload_flow`) ODER neuer File `rest/tests/pdf_shiftplan.rs` (aber rest/ hat aktuell nur `content_type_surface`+`openapi_surface`-Tests, kein Setup-Muster für Handler-Integration mit In-Memory-DB — daher empfehle service_impl-Weg)
- [ ] `shifty-dioxus/src/page/shiftplan.rs` — pure `should_show_pdf_button(WeekStatus, Option<Uuid>) -> bool` Fn + Test darunter (im gleichen File oder `shifty-dioxus/src/page/tests/`)
- [ ] `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md` Edits (im selben Commit wie Code-Änderungen — D-49-15/16)

## Security Domain

**`security_enforcement`-Konfig ist implizit true.**

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Bestehendes OIDC-Flow (Production) + `mock_auth`-Feature (Dev). Neuer Handler nutzt bestehende `forbid_unauthenticated`-Middleware — kein neuer Auth-Code. |
| V3 Session Management | no | Keine Session-Änderung; Cookie-Auth-Durchreichung durch Browser (`<a href>` schickt Cookie automatisch). |
| V4 Access Control | yes | KEIN Admin-Gate (PDF-05); alle authentifizierten User dürfen alle Shiftplan-Wochen sehen (im v2.3-Scope alle User haben Read-Zugriff auf Shiftplan-Views — konsistent mit `ShiftplanViewService` selbst). Verified via CONTEXT §D-49-07. |
| V5 Input Validation | yes | Path-Params: `Uuid` (Axum-typed), `u32` (year), `u8` (week). Axum weist ungültige Werte automatisch mit 400 ab. WeekStatus-Gate im Service (Defense-in-Depth). |
| V6 Cryptography | no | Keine Krypto-Operation; PDF-Bytes sind Standard-printpdf-Output aus Phase 48. |

### Known Threat Patterns for {Axum + Cookie-Auth-REST + WASM-Frontend}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Rohe PDF-Bytes leaken User-Daten an Unbefugte (IDOR) | Information Disclosure | PDF-05 sagt explizit „kein Admin-Gate" — alle authenticated User dürfen alle Wochen sehen. Middleware `forbid_unauthenticated` reicht. Für Anonymous-Access → 401. |
| PDF-Content-Injection über Booking-Namen | Injection (via Renderer) | `pdf_render.rs` nutzt `use_text`-API von `printpdf`, die Content-Stream-Escape macht (hex-encoded ASCII). Kein neuer Injection-Vektor durch diese Phase. |
| Directory-Traversal in Filename | Injection | Filename ist rein deterministisch (`{year}-KW{week:02}`); Path-Params sind typed → keine User-Input im Filename. |
| CSRF auf GET-Endpoint | Spoofing | GET ist per definitionem CSRF-safe (kein State-Change). Der Endpoint löst keinen Side-Effect aus. |
| Slot-/Sales-Person-Enumeration über 200/404-Timing | Info-Disclosure | Race-Condition: 404 wenn shiftplan_id nicht existiert, 200 sonst. Anfrage-Rate ist an authentifizierte User gebunden — akzeptabel. |
| Renderer-Panic-DoS | Denial of Service | `pdf_render::render_shiftplan_week_pdf` returned `Result<_, ServiceError::InternalError>` — mapped auf 500, kein Panic. |

### Package Legitimacy Audit (Security-Focus)
`printpdf` 0.7 wurde in Phase 48 legitimacy-audited und approved. Keine neuen Dependencies in Phase 49 → kein neuer Audit-Bedarf.

## Project Constraints (from CLAUDE.md)

### From repo-root `CLAUDE.md`
- **Service-Tier-Konvention**: `PdfShiftplanService` MUSS Business-Logic-Tier sein (konsumiert `ShiftplanViewService` + `SalesPersonService` + `WeekStatusService`) — verifiziert konform mit D-49-05. Konstruktionsreihenfolge in `shifty_bin/src/main.rs`: erst Basic, dann BL — verifiziert via D-49-09.
- **VCS**: `jj`-managed, GSD-Auto-Commit `commit_docs:true` — Executor committet via git, jj importiert.
- **i18n three-locale-rule**: Neuer Key `PdfDownload` muss in de/en/cs — verifiziert via D-49-14 (alle drei "PDF").

### From `shifty-backend/CLAUDE.md`
- **OpenAPI**: Jeder neue REST-Endpoint MUSS `#[utoipa::path]` haben — verifiziert im Code-Beispiel oben.
- **Transaction Management**: Service-Methoden nehmen `Option<Transaction>` — verifiziert in D-49-06 (Signature enthält `tx: Option<Self::Transaction>`).
- **Testing**: `mockall` für Unit-Tests (Mock-basiert), in-memory SQLite für Integration — beides in Wave-0-Test-Plan berücksichtigt.
- **Clippy-Gate**: `cargo clippy --workspace -- -D warnings` MUSS grün sein — nix build enforced das; jedes Task-Gate MUSS clippy separat laufen.
- **`sqlx prepare`**: Nicht anwendbar — kein neuer `query!`/`query_as!` in dieser Phase.

### From `shifty-backend/CLAUDE.local.md`
- **NixOS + nix develop**: Wenn sqlx-cli o.ä. gebraucht wird, `nix develop` — nicht anwendbar hier (keine Migration).
- **jj-commit für manuelle Commits**: Auto-Commit-Path (executor) ist etabliert und akzeptiert — kein Handbetrieb nötig.

### From `shifty-dioxus/CLAUDE.md`
- **Tailwind muss watchen bei Dev**: Standard-Setup.
- **i18n alle drei Locales**: bereits notiert.
- **`Locale::De` bug** — historisch gefixt, kein Impact hier.

## Sources

### Primary (HIGH confidence)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/49-pdf-download-button/49-CONTEXT.md` — 16 locked decisions [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/REQUIREMENTS.md` — PDF-01..PDF-05 [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/ROADMAP.md` — Phase 49 Section [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/pdf_render.rs` — pure Renderer + tests [VERIFIED: read directly, all 200+ lines]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/pdf_export_scheduler.rs` — §1–460 [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/pdf_export.rs` — Trait-Referenz [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/week_status.rs` — WeekStatus-Enum [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/shiftplan.rs` — ShiftplanViewService-Trait [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/sales_person.rs` — SalesPersonService::get_all-Signatur [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/lib.rs` — ServiceError-Enum [VERIFIED: read directly, alle Variants]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/lib.rs` — Router-Struktur, error_handler, RestStateDef [VERIFIED: read §1–753]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/pdf_export_config.rs` — REST-Handler-Vorlage [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/main.rs` §300–1230 — DI-Wiring-Reihenfolge [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/page/shiftplan.rs` §1120–1160 (iCal-Anchor) + §127–200 (Signals) [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/state/week_status.rs` — FE-Mirror-Enum [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/src/i18n/mod.rs` + `de.rs`+`en.rs`+`cs.rs` — i18n-Key-Struktur [VERIFIED: read directly]
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/pdf_export_scheduler.rs` §1–80 — Mock-Test-Muster [VERIFIED: read directly]

### Secondary (MEDIUM confidence)
- N/A — alle Claims verifiziert durch direktes Codebase-Reading.

### Tertiary (LOW confidence)
- Assumption A1 (`WEEK_STATUS_STORE` feuert bei KW-Nav) — indirekt aus 3 grep-Treffern für `WeekStatusAction::Load` in shiftplan.rs; nicht auf runtime-behavior verifiziert. [ASSUMED — Planner sollte einmal live grep + trace tun.]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — keine neuen Deps, alle Muster bereits im Repo
- Architecture: HIGH — DRY-Refactor mit exaktem Precedent (Scheduler §365–400)
- Pitfalls: HIGH — 5 konkrete Pitfalls direkt aus Code-Reading identifiziert (Router-Kollision verifiziert konfliktfrei; Scheduler-Deps-Streichung verifiziert impact-radius)
- Test-Strategie: HIGH — Scheduler-Test-Muster aus Phase 48 direkt übertragbar
- FE-Muster: HIGH — iCal-Anchor 1:1 kopierbar; Sichtbarkeits-Guard via `matches!` und `Option`-Unwrap trivial
- 409-Signalisierungsweg (Pitfall 3): MEDIUM — 2 valide Wege existieren; Empfehlung "Handler-Pre-Check" ist chirurgisch aber Planner MUSS in Plan-Phase entscheiden welchen Weg. Assumption A3.
- Scheduler-WeekStatus-Gate-Semantik (Open Q1): MEDIUM — semantische User-Entscheidung; nicht durch Codebase klärbar.

**Research date:** 2026-07-03
**Valid until:** 2026-08-03 (30 Tage; stable-domain-code, keine schnellbewegten Deps, keine neuen Features im Ecosystem-Umfeld)
