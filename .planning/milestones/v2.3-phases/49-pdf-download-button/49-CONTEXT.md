# Phase 49: pdf-download-button - Context

**Gathered:** 2026-07-03
**Status:** Ready for planning
**Mode:** discuss (Textform, 5 Gray Areas + 2 Rückfragen zu G2/G4)

<domain>
## Phase Boundary

Ein On-Demand-PDF-Download-Button auf der Schichtplan-Seite (`shifty-dioxus/src/page/shiftplan.rs`), der die **im UI selektierte KW** des **aktuell ausgewählten Shiftplans** über einen neuen authentifizierten (nicht-admin-gegateten) REST-Endpoint rendert und ausliefert. Der Renderer selbst bleibt der v2.2-Renderer (Phase 50 tauscht ihn transparent). Kein Snapshot-Bump, keine Migration, keine neue Cargo-Dep.

**Wichtige Requirement-Anpassung in dieser Phase (siehe D-49-04):** PDF-03 „lädt die aktuelle KW basierend auf heute" wird umformuliert auf „lädt die aktuell im UI selektierte KW". Das Nicht-Ziel „Wochenwahl über die UI-Navigation" wird gestrichen. Diese REQUIREMENTS.md- und ROADMAP-Edits fahren im selben Commit wie dieses CONTEXT.md.

</domain>

<decisions>
## Implementation Decisions

### G1 — Endpoint-Shape & REST-Modul-Verortung
- **D-49-01:** Endpoint = `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf`. Explizites `shiftplan_id` im Pfad (kein Query-Param, kein Hardcode auf „ersten im Catalog"). Weder Body noch Body-Optionen — GET liefert `application/pdf` mit `Content-Disposition: attachment; filename="schichtplan-{JJJJ}-KW{NN}.pdf"`.
- **D-49-02:** Eigenes REST-Modul `rest/src/pdf_shiftplan.rs` (Naming analog `pdf_export_config`). Wird via `generate_route()` in `rest/src/lib.rs` gemountet. Eigener `PdfShiftplanApiDoc`-Nest für Swagger.
- **D-49-03:** HTTP-Status-Codes:
  - `200 application/pdf` bei Erfolg,
  - `401` bei fehlender Auth,
  - `404` wenn `shiftplan_id` nicht existiert (unverändert Basis-Verhalten des View-Service),
  - **`409 Conflict`** wenn `week_status ∈ {Unset, InPlanning}` (Defense-in-Depth, PDF-04),
  - `500` bei Renderer-Fehlern.

### G4 — Selektierte KW entscheidet (Requirement-Deviation)
- **D-49-04:** **Deviation zu PDF-03 / SC 3:** Der Button lädt die **im UI selektierte KW** des Shiftplans (via `week`/`year`-Signals im shiftplan.rs), nicht die heute-KW. Analog dazu die Sichtbarkeit: Button ist genau dann sichtbar, wenn der `WeekStatus` der **selektierten** KW ∈ {`Planned`, `Locked`} — nutzt den bestehenden `WEEK_STATUS_STORE`, kein Extra-Fetch nötig. REQUIREMENTS.md PDF-03, das Nicht-Ziel „Wochenwahl über die UI-Navigation" und ROADMAP-SC 3 werden im selben Commit auf diese Semantik umgeschrieben (Backend-URL bleibt strukturell wie D-49-01, nimmt einfach die vom FE gelieferten Werte).

### G2 — DRY: PdfDownloadService als Business-Logic-Service, Scheduler-Refactor
- **D-49-05:** Neuer Business-Logic-Service `service::pdf_shiftplan::PdfShiftplanService` (Alternative-Naming: `PdfDownloadService` — Planner wählt den lesbareren). Verortung analog `PdfExportScheduler` (Business-Logic-Tier per CLAUDE.md-Konvention, weil er `ShiftplanViewService` + `SalesPersonService` konsumiert).
- **D-49-06:** Trait-API (Kern-Methode):
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
  Der Service selbst kümmert sich um: KW-Status-Vorprüfung (`WeekStatusService::get_week_status` → wenn ∈ {`Unset`, `InPlanning`} → `ServiceError::Conflict` o.ä., den REST-Handler auf 409 mappt), `ShiftplanViewService::get_shiftplan_week` (mit `Authentication::Full`? nein — mit dem `context` des Aufrufers, siehe D-49-07), Filter aktive `SalesPerson`s (`deleted.is_none()`), und ruft `pdf_render::render_shiftplan_week_pdf`. Rückgabe reine Bytes.
- **D-49-07:** Auth-Semantik: Der Service ruft `ShiftplanViewService` und `SalesPersonService` mit dem User-Context des Aufrufers auf (nicht `Authentication::Full`). Das FE ist auth-geschützt, der Handler prüft nur „authentifiziert" (kein Admin-Gate — PDF-05). Wenn der User keinen Read-Zugriff auf den Shiftplan hätte, würden die konsumierten Services das gaten — im v2.3-Scope aber alle authentifizierten User haben Read-Zugriff auf Shiftplan-Views.
- **D-49-08 — Scheduler-Refactor (DRY-Auflösung):** `PdfExportScheduler::run_once_now` (bisher `service_impl/src/pdf_export_scheduler.rs:365–400`) wird refactored: statt inline `shiftplan_view_service.get_shiftplan_week` + `sales_person_service` + `pdf_render::render_shiftplan_week_pdf` aufzurufen, ruft der Scheduler pro Iteration `PdfShiftplanService::render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)`. **Ein einziger Ort für die Assemble-Logik.** Der Scheduler bleibt zuständig für die Cron-Schleife, WebDAV-Upload, Retry-Persistenz, `record_success`/`record_error`. `shiftplan_view_service`- und `sales_person_service`-Deps des Schedulers entfallen (oder bleiben, wenn der Refactor sie noch anderswo braucht — Planner entscheidet).
- **D-49-09:** DI-Wiring in `shifty_bin/src/main.rs`: `PdfShiftplanService` (Business-Logic) wird konstruiert **nach** `ShiftplanViewService` + `SalesPersonService` + `WeekStatusService` (Basic) und **vor** `PdfExportScheduler` (dessen Dep er neu wird). Konsistent mit der bestehenden Reihenfolgen-Regel.

### G3 — FE-Download-Mechanik & G5 — Button-Platzierung
- **D-49-10:** FE-Button = simples `<a>`-Element (kein WASM-Fetch/Blob), analog dem existierenden iCal-Button in `shiftplan.rs:1130–1138`. Cookie-Auth wird vom Browser durchgereicht.
- **D-49-11:** Platzierung: **direkt neben dem iCal-Button** (`shiftplan.rs:1123–1140`, in der Toolbar-Row des Shiftplan-Headers). Gleiches Styling: `px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt`. Icon-Prefix `↓` (mono-Font wie iCal), Label = i18n-Key `PdfDownload`. `target="_blank"` weglassen — Download-Attribut reicht (`download="schichtplan-{yyyy}-KW{ww}.pdf"`).
- **D-49-12:** URL-Konstruktion im FE:
  ```
  format!("{}/shiftplan/{}/{}/{}/pdf", backend_url, shiftplan_id, *year.read(), *week.read())
  ```
  Nutzt `selected_shiftplan_id` + `year` + `week` Signals aus `shiftplan.rs`. Wenn `selected_shiftplan_id` `None` ist (Catalog leer / noch nicht geladen), wird der Button gar nicht gerendert.
- **D-49-13:** Sichtbarkeit conditional:
  ```
  if let Some(sp_id) = selected_shiftplan_id.read().as_ref() {
      if matches!(*week_status.read(), WeekStatus::Planned | WeekStatus::Locked) {
          rsx! { a { href: …, download: …, "↓ {pdf_download_label}" } }
      }
  }
  ```
  Kein disabled-Zustand, kein Tooltip, kein Fehler-Toast (per User-Entscheidung — Button nur sichtbar wenn klick funktioniert).
- **D-49-14:** i18n reduziert auf **einen** neuen Key `PdfDownload` (de: „PDF", en: „PDF", cs: „PDF") — Icon + Kürzel-Label wie beim iCal-Button. Falls User später einen längeren String will („PDF herunterladen"), trivial nachzuziehen. Kein Tooltip-Key nötig (kein disabled-Zustand).

### Requirement-Deviation — Begleitende Doku-Updates (im selben Commit)
- **D-49-15:** `.planning/REQUIREMENTS.md`:
  - PDF-03 — „aktuelle Kalenderwoche (basierend auf heute)" → „aktuell im UI selektierte Kalenderwoche". Verifikations-Passus unverändert (manueller UAT-Klick).
  - Nicht-Ziel „Wochenwahl über die UI-Navigation" wird gestrichen (bewusst umgekehrt).
  - PDF-04 unverändert (Backend-Gate auf `week_status` bleibt, gilt jetzt für die vom FE gelieferte KW).
- **D-49-16:** `.planning/ROADMAP.md`:
  - Phase 49 „Goal" — „aktuelle Kalenderwoche (basierend auf heute)" → „aktuell im UI selektierte Kalenderwoche".
  - Phase 49 SC 3 — „lädt IMMER die KW von heute, nicht die im UI navigierte Woche" → Sichtbarkeit + Ziel-KW werden vom selektierten `week`/`year` gesteuert; Button versteckt (statt disabled) außerhalb von {Planned, Locked}.

### Claude's Discretion
- Naming: `PdfShiftplanService` vs. `PdfDownloadService` — Planner nimmt den, der im REST-Kontext am wenigsten Missverständnisse erzeugt.
- Test-Struktur:
  - Backend-Unit: `service_impl/src/test/pdf_shiftplan.rs` — Mock von View/SalesPerson/WeekStatus, prüft (a) Happy-Path liefert Bytes, (b) `Unset`/`InPlanning` → Conflict-Error, (c) SalesPerson-Filter `deleted.is_none()`, (d) `pdf_render`-Aufruf mit erwarteten Args.
  - Backend-Integrationstest: `rest/tests/pdf_shiftplan.rs` (oder analog zu bestehenden Rest-Tests) — 200/409/401-Matrix + Content-Type + Content-Disposition Header.
  - FE: cargo-Test auf reine Predikat-Fn `should_show_pdf_button(week_status: WeekStatus, shiftplan_id: Option<Uuid>) -> bool` — analog `should_show_badge` aus Phase 39 (Memory: „Dioxus Browser-Test: Datepicker" — programmatisches Setzen von Datepicker triggert Signals nicht, daher pure-fn-Test statt Browser-E2E).
- Scheduler-Refactor darf `shiftplan_view_service`- + `sales_person_service`-Deps von `PdfExportSchedulerDeps` streichen, wenn sie nach dem Refactor nur noch via `PdfShiftplanService` erreicht werden. Planner entscheidet nach `grep`.
- OpenAPI: neuer `#[utoipa::path]` mit `responses(...)` inkl. 200 (binary), 409, 401, 404. Response-Body als raw `Vec<u8>` mit `content = "application/pdf"`.
- Clippy-Gate wie üblich (`cargo clippy --workspace -- -D warnings`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — v2.3 Requirements PDF-01..PDF-05 (PDF-03 wird in dieser Phase edited, s. D-49-15).
- `.planning/ROADMAP.md` — Phase 49 (wird in dieser Phase edited, s. D-49-16).

### Phase 48 (Vorgänger — Renderer, Config, Scheduler)
- `.planning/milestones/v2.2-phases/48-nextcloud-pdf-webdav/48-CONTEXT.md` — Renderer- + Scheduler-Entscheidungen. Insb. `printpdf` (D-48-1), Filename-Konvention (D-48-5), Retry (D-48-7).
- `service_impl/src/pdf_render.rs` — pure Rendering-Modul (`render_shiftplan_week_pdf`).
- `service_impl/src/pdf_export_scheduler.rs` §365–443 — Assemble-Blaupause (View + SalesPerson + Render + WebDAV). §365–400 wird durch `PdfShiftplanService::render_week_pdf` ersetzt (D-49-08).
- `service/src/pdf_export.rs` — Scheduler-Trait als Referenz für „Business-Logic-Service-Trait mit Context/Transaction generic parameters".

### Phase 39/40 (WeekStatus)
- `service/src/week_status.rs` — `WeekStatus`-Enum (`Unset`/`InPlanning`/`Planned`/`Locked`) + Service-Trait. Row-Absenz == `Unset` (D-39-04).
- `rest-types/src/lib.rs` §1351–1386 — `WeekStatusKindTO` + `From`/`Into`.
- `shifty-dioxus/src/state/week_status.rs` §11–25 — FE-Mirror-Enum.

### Service-Tier-Konvention & DI
- `CLAUDE.md` (Repo-Root) §"Service-Tier-Konventionen" — Basic vs. Business-Logic-Regeln. `PdfShiftplanService` ist per Definition Business-Logic (konsumiert `ShiftplanViewService`, `SalesPersonService`, `WeekStatusService`).
- `shifty_bin/src/main.rs` — DI-Konstruktionsreihenfolge; neue Service-Konstruktion einreihen (D-49-09).

### FE-Referenzen
- `shifty-dioxus/src/page/shiftplan.rs` §1123–1140 — iCal-Button-Muster, direkt daneben landet der PDF-Button (D-49-11).
- `shifty-dioxus/src/api.rs` — REST-Client-Muster; **kein** neuer Eintrag nötig, weil der PDF-Endpoint per `<a href>` statt reqwest angesprochen wird.
- `shifty-dioxus/src/i18n/mod.rs` (Key-Enum) + `en.rs`/`de.rs`/`cs.rs` — neuer Key `PdfDownload`.
- `shifty-dioxus/CLAUDE.md` §"i18n System" — alle drei Locales pflegen.

### Backend-Konventionen
- `CLAUDE.md` (backend) §"REST API" — `#[utoipa::path]` + `ToSchema` + `error_handler`-Wrapper.
- `CLAUDE.md` (backend) §"Transaction Management" — `Option<Transaction>`-Muster für die neue Trait-Methode.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Renderer:** `pdf_render::render_shiftplan_week_pdf(&week, &sales_persons, year, week) -> Result<Vec<u8>, ServiceError>` — pure, keine Deps, wird 1:1 wiederverwendet.
- **View-Service:** `service::shiftplan::ShiftplanViewService::get_shiftplan_week(shiftplan_id, year, week, auth, tx) -> ShiftplanWeek` — liefert die Wochen-Daten.
- **SalesPerson-Service:** `service::sales_person::SalesPersonService::get_all(auth, tx)` — Basis für Aktiv-Filter (`.filter(|sp| sp.deleted.is_none())`), Blaupause im Scheduler §351–355.
- **WeekStatus-Service:** `service::week_status::WeekStatusService::get_week_status(year, calendar_week, auth, tx) -> WeekStatus` — für 409-Gate.
- **FE-Toolbar-Slot:** `shifty-dioxus/src/page/shiftplan.rs:1123–1140` — iCal-`<a>`-Anchor-Muster inkl. Styling-Klassen ist die Vorlage.
- **FE-Signals:** `selected_shiftplan_id: Signal<Option<Uuid>>`, `year: Signal<u32>`, `week: Signal<u8>`, `week_status: Signal<WeekStatus>` (aus `WEEK_STATUS_STORE`) — alle bereits vorhanden.

### Established Patterns
- **Service-Tier-Konvention** (backend CLAUDE.md): `PdfShiftplanService` ist Business-Logic, weil er andere Domain-Services konsumiert.
- **Trait-Definition-Muster:** `service/src/pdf_export.rs` als Template (Context/Transaction generics + `#[automock]`).
- **`gen_service_impl!`-Muster** für die Implementation in `service_impl/src/pdf_shiftplan.rs`.
- **REST-Handler:** Axum + `#[utoipa::path]` + `error_handler`-Wrapper (siehe `rest/src/pdf_export_config.rs` als Nachbar).
- **FE-Download via `<a href download>`:** iCal-Precedent, funktioniert mit Cookie-Auth.
- **Konditionale Sichtbarkeit im FE:** `if matches!(*sig.read(), Variant1 | Variant2) { rsx! { … } }` — steht mehrfach in shiftplan.rs.

### Integration Points
- **Neues Backend-Trait:** `service/src/pdf_shiftplan.rs` (oder `pdf_download.rs` — Naming-Diskretion).
- **Neue Impl:** `service_impl/src/pdf_shiftplan.rs` (+ Test-Modul `service_impl/src/test/pdf_shiftplan.rs`).
- **Neuer REST-Handler:** `rest/src/pdf_shiftplan.rs` mit `generate_route()` + `PdfShiftplanApiDoc`; Registration in `rest/src/lib.rs` (mod, ApiDoc-Nest, `.nest("/shiftplan/…")` — Achtung Konflikt mit bestehender `shiftplan_info`-Route, siehe D-49-02: eigener Nest, aber die Path-Struktur mit `{shiftplan_id}` kann als Sub-Route unter `/shiftplan-info` ODER als komplett eigener Nest `/shiftplan-pdf/{shiftplan_id}/{year}/{week}` geführt werden — Planner entscheidet je nach Router-Kollisionen).
- **Refactor:** `service_impl/src/pdf_export_scheduler.rs:365–400` ruft neuen Service (D-49-08), Deps in `PdfExportSchedulerDeps` ggf. reduzieren.
- **DI:** `shifty_bin/src/main.rs` — neue Konstruktion einreihen, Scheduler-Konstruktor bekommt neuen Service als Dep.
- **FE-Neu:** `shifty-dioxus/src/page/shiftplan.rs` — neuer `<a>`-Block neben iCal-Button (Zeile ~1140), reine RSX-Änderung.
- **FE-i18n:** neuer `Key::PdfDownload` in `shifty-dioxus/src/i18n/mod.rs` + Übersetzungen in de/en/cs.
- **REQUIREMENTS.md + ROADMAP.md:** Edits im selben Commit (D-49-15, D-49-16).

### Anti-Patterns / Landmines
- **Router-Kollision** (`/shiftplan/…`): Es gibt bereits `.nest("/shiftplan-info", …)` und `.nest("/shiftplan-catalog", …)` und `.nest("/shiftplan-edit", …)`. Ein neuer Nest `/shiftplan` würde nicht kollidieren (keine bestehende Route heißt genau so), aber Planner soll sanity-checken. Alternative: Nest heißt `/shiftplan-pdf`, URL wird `/shiftplan-pdf/{shiftplan_id}/{year}/{week}/pdf` — etwas redundant, aber kollisions-sicher.
- **Scheduler-Auth:** Der Scheduler ruft heute mit `Authentication::Full`. Beim Refactor MUSS `PdfShiftplanService::render_week_pdf` weiterhin `Authentication::Full`-Aufrufe des Schedulers akzeptieren (D-49-07 spricht vom User-Context des Aufrufers — der Scheduler ist ebenfalls ein Aufrufer, mit `Full` als Context).
- **Content-Disposition:** Der Filename `schichtplan-{yyyy}-KW{ww:02}.pdf` MUSS im Header stehen — sonst nutzt der Browser den letzten Path-Segment (`pdf`) als Dateiname. Verifikation im Integrationstest.
- **`selected_shiftplan_id` kann `None` sein:** FE-Rendering-Guard nötig; kein Panic-Path.

</code_context>

<specifics>
## Specific Ideas

- **Icon-Prefix:** `↓` (mono-font), analog iCal-Button. Kurzes Label `PDF` — kein „PDF herunterladen" (User-Präferenz: neben iCal-Button sitzen, Symmetrie).
- **Kein Tooltip, kein Toast, kein Fehler-Banner** — Button ist unsichtbar wenn Status ∉ {Planned, Locked}. Der einzige übrige Fehlerfall ist ein Race (Status wechselt zwischen Signal-Update und Klick) — dann liefert das Backend 409, der Browser zeigt eine 409-Response an. Akzeptiert (Rand-Case, kein Recovery-UX geplant).
- **Backend-Response bei 409:** `application/json` mit einer minimalen `{ "error": "week-not-releasable" }`-Struktur ODER text/plain — Planner wählt konform zum bestehenden `ServiceError`-Mapping-Muster.

</specifics>

<deferred>
## Deferred Ideas

- **Download über beliebige KW (Multi-Week-Batch)** — kein Scope, Single-Week ist die Anforderung.
- **PDF-Preview im Browser** (statt Download) — nicht diskutiert, kein Scope.
- **Personal-PDF (nur eigene Bookings)** — explizit als Nicht-Ziel in REQUIREMENTS.md („Keine Sales-Person-Filterung").
- **Fehler-Toast/Banner-UI** — bewusst weggelassen (User-Entscheidung G5). Falls später gewünscht, trivialer Nachtrag.
- **Loading-Spinner** — beim `<a href>`-Ansatz nicht benötigt (Browser zeigt Standard-Download-UI).

</deferred>

---

*Phase: 49-pdf-download-button*
*Context gathered: 2026-07-03*
