# Feature: Export — PDF, iCal, WebDAV, Scheduler

> **In short:** Weekly shift plans as PDF (on-demand download or
> cron-triggered WebDAV push to Nextcloud) and personal shift blocks as
> an iCal feed for the calendar client — plus the associated admin
> configuration UI.

**Cluster ID:** F11
**Status:** in production
**First introduced:** milestones v2.2 (Phase 48 — Nextcloud export,
Scheduler loop, WebDAV client), v2.3 (Phase 49 — on-demand download,
Phase 50 — browser-look renderer). The iCal feed has existed since the
Block service (pre-v2).
**Responsible crates:**
`service::pdf_export`, `service::pdf_export_config`,
`service::pdf_shiftplan`, `service::ical`,
`service_impl::pdf_export_scheduler`, `service_impl::pdf_export_config`,
`service_impl::pdf_shiftplan`, `service_impl::pdf_render`,
`service_impl::webdav_client`, `service_impl::ical`,
`dao::pdf_export_config`, `dao_impl_sqlite::pdf_export_config`,
`rest::pdf_export_config`, `rest::pdf_shiftplan`, `rest::sales_person`
(iCal handler).

---

## 1. What is it? (Business perspective)

Shifty exports shift plans over three orthogonal channels:

1. **On-demand PDF download** — any authenticated employee can, in the
   week view, download the PDF of the current week with a click
   (`GET /shiftplan/{id}/{y}/{w}/pdf`). The PDF looks like the browser
   week view: landscape A4 grid with day columns and slot boxes labeled
   with names.
2. **Scheduler-driven Nextcloud export** — an admin configures WebDAV
   URL, user, app token, target folder, week horizon and a cron
   expression once. The server then renders the next *N* weeks on a
   schedule and drops them into the Nextcloud folder. The feature is off
   by default (`enabled=0`).
3. **iCal feed per Sales Person** — the frontend calendar-sync link
   `GET /sales-person/{id}/ical` delivers the planned blocks of the next
   weeks as `text/calendar` — the employee subscribes to the link in
   their own calendar client and sees their shifts there.

**Example workflow on-demand PDF (user view):**

1. Employee opens the week view → sees the download icon in the week.
2. Click the icon → the browser downloads
   `schichtplan-2026-KW27.pdf`.
3. On a non-releasable week (status `Unset` / `InPlanning`) the server
   answers with 409 and the frontend shows an error message — the
   download does not happen.

**Example workflow Nextcloud export (admin view):**

1. Admin opens *Settings → PDF export to Nextcloud*.
2. Enters cloud URL, user, app token, target folder, week horizon and
   cron expression, activates `enabled`, and saves.
3. Scheduler reload registers the new cron expression without a server
   restart. After the next cron tick, a file
   `schichtplan-YYYY-KWnn.pdf` lies in the WebDAV folder for each week
   in the horizon.
4. Optional: button *"Export now"* triggers an immediate run.
5. Status display (`last_success_at`, `last_error_at`,
   `last_error_message`) shows success or the most recent error detail.

## 2. Business Rules

- **PDF week status gate (D-49-06):** a weekly PDF may only be delivered
  for weeks in status `Planned` or `Locked`. `Unset` and `InPlanning`
  return 409 (handler pre-check,
  `rest/src/pdf_shiftplan.rs:130`) or `ValidationError` (service gate,
  `service_impl/src/pdf_shiftplan.rs:140`).
- **Active Sales Persons (D-49-05, D-48-PDF-ACTIVE-ONLY):** the renderer
  only receives non-deleted Sales Persons; filtering happens in the
  assemble service via `filter_active`
  (`service_impl/src/pdf_shiftplan.rs:89`). The renderer itself does not
  filter.
- **Filename convention:** `schichtplan-{YYYY}-KW{NN:02}.pdf` (ASCII, no
  RFC 5987 encoding needed). Defined in
  `service/src/pdf_shiftplan.rs:68` `filename_for(year, week)` and
  consumed by the REST handler and the Scheduler.
- **Auth passthrough in PDF assemble (D-49-07):** the `PdfShiftplanService`
  passes the *caller's* context to all consumed services;
  **never** internally escalated to `Authentication::Full`. Quote from
  `service/src/pdf_shiftplan.rs:21`: *"never internally escalated to
  `Authentication::Full`"* — i.e. if an employee without special rights
  triggers the download, the chain (WeekStatus, View, SalesPersons) runs
  with exactly that restricted auth. The Scheduler is the only caller
  that uses `Authentication::Full` — legitimate because it is
  internally trusted.
- **Admin gate PDF export config (D-48-ADMIN):**
  `get`/`update`/`trigger` on `/pdf-export-config` require the
  `admin` role. Non-admin → 403
  (`service_impl/src/pdf_export_config.rs:54`).
- **Token merge semantics (D-48-REST):** in the PUT body,
  `webdav_app_token = None` means "keep existing value";
  `Some(v)` means "set new value". The merge sits in the basic service
  (`service_impl/src/pdf_export_config.rs:77`).
- **Token never in response (T-48-02):** `From<&PdfExportConfig> for
  PdfExportConfigTO` always sets `webdav_app_token` to `None`
  (`rest-types/src/lib.rs:2271`). The token never leaves the server in
  an HTTP response.
- **Scheduler-only status recorder:** `record_success` /
  `record_error` check `check_only_full_authentication`
  (`service_impl/src/pdf_export_config.rs:118`) — only the Scheduler
  (`Authentication::Full`) may write; no admin-facing public API path.
  The `trigger_export_now` handler checks admin, then internally spawns
  with full auth (`rest/src/pdf_export_config.rs:147-153`).
- **v1 multi-shiftplan simplification:** the Scheduler exports **exactly
  one** shift plan per week — the first non-deleted from
  `ShiftplanService::get_all` (`service_impl/src/pdf_export_scheduler.rs:346`).
  Multi-shiftplan aggregation is a follow-up.
- **Scheduler only exports releasable weeks (D-49-08 / Q1):** the
  assemble delegates to `PdfShiftplanService::render_week_pdf`; weeks in
  `Unset`/`InPlanning` come back as `ValidationError`, are logged via
  `record_error`, and skipped with `continue` — later planned weeks in
  the horizon are still tried
  (`service_impl/src/pdf_export_scheduler.rs:379-395`, v2.3.1
  improvement).
- **Retry & classification WebDAV:** per upload run, 3 attempts with
  backoff `2s/4s/8s` (`webdav_client.rs:29`). 2xx = success,
  MKCOL 405 = "folder exists" = success, 5xx / IO = transient, 4xx
  (except MKCOL 405) = permanent, no retry (`webdav_client.rs:90`).
  Per request 30s timeout (T-48-10).
- **Upload abort semantics:** on the first upload error in a run, the
  loop aborts (`return Ok(())`), so that the next cron slot starts
  over — no continuing with a presumably broken WebDAV endpoint
  (`service_impl/src/pdf_export_scheduler.rs:422`).
- **`record_success` only on actual upload (v2.3.1):** only if
  `succeeded_count > 0` is success persisted — otherwise the UI would
  suggest success even though no week landed in the cloud folder
  (`service_impl/src/pdf_export_scheduler.rs:433`).
- **Boot tolerance:** a broken cron expression or a failed initial reload
  does NOT prevent the backend from starting — the Scheduler starts
  dormant and persists the diagnosis via `record_error`
  (`service_impl/src/pdf_export_scheduler.rs:202-206`).
- **iCal time window:** the iCal feed delivers the next 12 weeks from
  "now minus 2 weeks" (i.e. 2 weeks of past + 10 weeks of future) —
  `service_impl/src/block.rs:218`.
- **iCal TZID:** the TZID value comes from `ConfigService.get_config().timezone`
  and is set per `DTSTART`/`DTEND`
  (`service_impl/src/ical.rs:24-33`) — no recurrence rules.
- **iCal without session gate:** the `/ical` path is intentionally let
  through by the session middleware without auth
  (`rest/src/session.rs:281`) so that calendar clients can subscribe to
  the feed without cookie / OIDC login. Compensation: the URL contains
  the Sales Person UUID as a hard-to-guess token.

## 3. Data Model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `pdf_export_config` | Single-row config of the Nextcloud export (analogous to `paid_limit_config` / `holiday_stichtag_config`) | `id` (fixed UUID `…0000048`), `enabled`, `nextcloud_url`, `webdav_user`, `webdav_app_token` (plaintext, D-48-01), `target_folder`, `weeks_horizon`, `cron_schedule`, `last_success_at`, `last_error_at`, `last_error_message`, `update_process`, `update_version` |

For iCal and on-demand PDF, there is **no dedicated persistence** — both
read from `shiftplan` / `booking` / `sales_person` / `week_status`.

### Migrations

- `migrations/sqlite/20260703000000_create-pdf-export-config.sql` — base
  table + seed row (`enabled=0`, `weeks_horizon=8`,
  `cron_schedule='0 6 * * 1'`).
- `migrations/sqlite/20260704000000_fix-pdf-export-cron-6-field.sql` —
  v2.3.1 hotfix: data fix for `cron_schedule`. `tokio-cron-scheduler`
  0.15 uses `croner` 3.0 in 6-field format (`sec min hour dom mon dow`);
  the original migration however seeded the 5-field pattern. Length-based
  detection (exactly four spaces ⇒ 5 fields) prepends `'0 '`.

### Relationships

The config row is not bound to any foreign key — pure application
config. The plaintext token in the DB is an intentional ops decision
(D-48-01); protection comes via filesystem permissions.

## 4. Service API

Four traits, two tiers (per service tier convention):

### 4.1 `service::pdf_export_config::PdfExportConfigService` (Basic)

Entity manager for the config row. Consumes only DAO + Permission +
Clock + Uuid + Transaction — **no** domain service as dependency
(`service/src/pdf_export_config.rs:9`).

Methods:

- `get(context, tx) -> PdfExportConfig` — admin-gated.
- `update(update, context, tx) -> PdfExportConfig` — admin-gated, token
  merge inside.
- `record_success(at, context, tx)` — full-auth only (Scheduler).
- `record_error(at, message, context, tx)` — full-auth only.

### 4.2 `service::pdf_shiftplan::PdfShiftplanService` (Business Logic)

Assembler for the DRY core `render_week_pdf`. A single entry point for
the REST handler and the Scheduler.

Order in the assemble path
(`service_impl/src/pdf_shiftplan.rs:126-170`):

1. `WeekStatusService::get_week_status` — gate.
2. `ShiftplanViewService::get_shiftplan_week` — expensive read.
3. `SalesPersonService::get_all` + `filter_active`.
4. `pdf_render::render_shiftplan_week_pdf` — pure function, returns
   `Vec<u8>`.

### 4.3 `service::pdf_export::PdfExportScheduler` (Business Logic)

Encapsulates the cron loop, WebDAV upload, retry persistence. Methods:

- `start()` — at app boot; initializes `JobScheduler` and calls
  `reload_from_db()` (boot-tolerant).
- `reload_from_db()` — after `PUT /pdf-export-config`: remove old job,
  register new cron (`service_impl/src/pdf_export_scheduler.rs:217`).
- `run_once_now(context)` — synchronous single run. The cron callback
  calls with `Full`; the REST trigger calls after admin check with
  `Full`.

### 4.4 `service::ical::IcalService`

Purely synchronous, dependency-less trait with one method:
`convert_blocks_to_ical_string(blocks, title, timezone) -> Arc<str>`.
The actual "iCal for Sales Person" endpoint hangs on
`BlockService::get_blocks_for_next_weeks_as_ical`
(`service_impl/src/block.rs:218`), which consumes `IcalService` as a
dependency.

### Auth gates (overview)

| Method | Gate |
| --- | --- |
| `PdfExportConfigService::{get,update}` | `admin` privilege |
| `PdfExportConfigService::{record_success,record_error}` | `Authentication::Full` |
| `PdfShiftplanService::render_week_pdf` | No dedicated gate — passes `context` through to consumed services |
| `PdfExportScheduler::run_once_now` | `check_only_full_authentication` (REST trigger converts admin → Full) |
| `IcalService` / iCal handler | No gate (middleware bypass, see §2) |

### TX behavior

- `PdfExportConfigServiceImpl` opens TX via
  `transaction_dao.use_transaction(tx)`, commits at the end of each
  method.
- `PdfShiftplanServiceImpl::render_week_pdf` consumes `tx` and passes it
  to all sub-calls (`.clone()`) — **does not commit itself**; the caller
  decides.
- The Scheduler calls all sub-services with `tx=None` — each opens its
  own TX, isolation per read.

### Dependencies

- `PdfExportConfigServiceImpl`: `PdfExportConfigDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `PdfShiftplanServiceImpl`: `ShiftplanViewService`, `SalesPersonService`,
  `WeekStatusService`, `PermissionService`, `TransactionDao`.
- `PdfExportSchedulerImpl`: `PdfExportConfigService`, `PdfShiftplanService`,
  `ShiftplanService` (catalog), `PermissionService`, `ClockService`,
  `TransactionDao` + `WebDavUploadFactory` (custom field).
- `IcalServiceImpl`: none — pure conversion.
- `WebDavClient`: no trait in the `service` crate, direct impl in
  `service_impl`. Abstraction outward via the `WebDavUpload` trait
  (`service_impl/src/webdav_client.rs:63`) so that the Scheduler
  remains mockable in tests.

**[To verify]** — `webdav_client` intentionally lives only in
`service_impl`, without a trait in the `service` crate. Consequence:
tests that do not want a real HTTP endpoint inject a mock via
`WebDavUploadFactory`.

## 5. REST Endpoints

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/pdf-export-config` | Current config (token masked) | — | `PdfExportConfigTO` | 403 (non-admin) |
| `PUT` | `/pdf-export-config` | Set config; empty token keeps existing value; triggers `reload_from_db` | `PdfExportConfigTO` | `PdfExportConfigTO` | 403, 500 |
| `POST` | `/pdf-export-config/trigger` | Immediate single run (`tokio::spawn`) | — | 204 No Content | 403, 500 |
| `GET` | `/shiftplan/{shiftplan_id}/{year}/{week}/pdf` | On-demand weekly PDF, `application/pdf` + `Content-Disposition: attachment; filename="schichtplan-YYYY-KWnn.pdf"` | — | Bytes | 401, 404, **409 `{"error":"week-not-releasable"}`**, 422 (ValidationError as fallback from the service gate), 500 |
| `GET` | `/sales-person/{id}/ical` | iCal feed of the next 12 weeks (2 weeks of past + 10 weeks of future) for the given Sales Person; `text/calendar` | — | Body as iCal text | 404, 500 |

DTOs see `rest-types::PdfExportConfigTO` (`rest-types/src/lib.rs:2244`).
For iCal and PDF download there is no JSON DTO — the responses are byte
or text streams.

Response flow `POST /pdf-export-config/trigger`
(`rest/src/pdf_export_config.rs:147`): the handler calls
`pdf_export_config_service.get(context, ..)` as an **admin gate**, then
`tokio::spawn`s `scheduler.run_once_now(Authentication::Full)` and
answers 204 — the cron path and the trigger path share the same
full-auth caller.

Response flow `GET /shiftplan/{...}/pdf`
(`rest/src/pdf_shiftplan.rs:117`): fast-path pre-check pulls
`WeekStatusService::get_week_status`; non-releasable weeks
short-circuit with 409 JSON before the expensive render path starts.

## 6. Frontend Integration

- **Pages:**
  - `shifty-dioxus/src/page/settings.rs:940` — card *"PDF export to
    Nextcloud"* (admin-gated). Form for all fields from
    `PdfExportForm`, save button, "Export now" trigger button, status
    row (`last_success_at` / `last_error_message`).
  - `shifty-dioxus/src/page/shiftplan.rs:1176` — download anchor
    `href="{backend}/shiftplan/{sp_id}/{y}/{w}/pdf"` with
    `download="schichtplan-{y}-KW{w:02}.pdf"`. No frontend week-status
    guard — the server decides via 409.
- **API client:** `shifty-dioxus/src/api.rs:1869-1907` — three wrapper
  functions `get_pdf_export_config`, `put_pdf_export_config`,
  `trigger_pdf_export`.
- **Loader:** `shifty-dioxus/src/loader.rs:966-987` — bridge between
  the API and the `PdfExportForm` state including form↔DTO conversion
  (`pdf_export_form_from_response`, `pdf_export_form_to_put_body`).
- **State:** `shifty-dioxus/src/state/pdf_export_config.rs` —
  `PdfExportForm`, `clamp_weeks_horizon`.
- **i18n keys:** `SettingsPdfExportTitle`, `SettingsPdfExportHelp`, …
  (`shifty-dioxus/src/i18n/de.rs:1278` ff.). Cs/En counterparts also
  maintained.
- **Proxy:** `Dioxus.toml` proxies `/pdf-export-config`,
  `/shiftplan` **and** `/sales-person` — i.e. both the config CRUD
  route and the PDF download route (which hangs under `/shiftplan`,
  `rest/src/lib.rs:666`) and the iCal route (under `/sales-person`,
  `rest/src/lib.rs:640`) work without an additional proxy entry.
  **[To verify]** — there is no dedicated
  `[[web.proxy]] backend=".../pdf-export-config"` entry; it is not
  covered by the generic `/settings` neighbor setup. Grep confirms: no
  match for `pdf-export` in `Dioxus.toml`. For `dx serve --hot-reload`
  this should be added if the card shows 404 (pattern from the F-Memory
  note *"Dioxus.toml proxy for new backend endpoints"*).

## 7. Edge cases

For the central edge case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), section
[*14. Export & external integrations*](../domain/edge-cases.md#14-export--externe-integrationen).

Feature-specific edges:

- **Week status not releasable (D-49-06):** REST pre-check returns 409
  with the stable `{"error":"week-not-releasable"}`; the service-internal
  gate catches race windows and direct-impl callers (Scheduler) and maps
  to 422 (`ValidationError`).
- **Auth passthrough (D-49-07):** *quote from
  `service/src/pdf_shiftplan.rs:21`:* "(D-49-07); never internally
  escalated to `Authentication::Full`." The entire PDF assemble chain
  respects the restricted user auth rather than bypassing it — unlike
  the reporting / booking-information aggregate that explicitly pulls
  full auth. For employee roles, the ShiftplanView may thus return less
  information; the renderer accordingly reflects only what the user is
  allowed to see in the frontend anyway.
- **Empty time range:** the renderer draws an empty grid with day
  headers and a timestamp row; no special handling. For "no Sunday
  booking at all", the Sunday column is dropped
  (`service_impl/src/pdf_render.rs:34`).
- **Special Days overlay:** the current renderer does **not** overlay a
  holiday marker on the grid cell — a holiday = empty cell like any
  other. **[To verify]** — noted in the backlog.
- **Sales Person with a very long name:** the slot box grows vertically
  to fit all names (UAT revision D-50-04); column overflow → `+ N more`
  marker at the bottom (`service_impl/src/pdf_render.rs:56-60`).
- **iCal TZID:** the TZID is stamped 1:1 from `ConfigService.timezone`
  (`service_impl/src/ical.rs:26-32`). With a wrongly configured
  `timezone` (e.g. `UTC` even though slots are planned in local time),
  calendar clients shift the display. Recurrence rules (`RRULE`) are
  **not** emitted — each week is a single window.
- **iCal past window:** `now - 2 weeks` as start guarantees that a
  calendar client that first pulls the iCal on Tuesday still sees the
  current weekend.
- **WebDAV auth error:** 401 → classification `Permanent`, no retry,
  `record_error` persists *"WebDAV upload … permanently failed (401)"*
  (`service_impl/src/pdf_export_scheduler.rs:407`).
  The WebDAV error never contains the Basic-Auth header
  (`WebDavError` display masked; `header_value.set_sensitive(true)` in
  `webdav_client.rs:160`).
- **WebDAV transient exhaustion:** 3 attempts with `2s/4s/8s`;
  afterwards `Transient { attempts: 3 }` → `record_error`, the
  Scheduler ends the run (`return Ok(())`); the cron tries again on the
  next tick.
- **Folder already exists:** MKCOL 405 is treated as success
  (`webdav_client.rs:91-99`) — Nextcloud standard behavior for existing
  folders.
- **Invalid cron expression:** `Job::new_async` fails → the error is
  persisted in `pdf_export_config.last_error_message`, the Scheduler
  stays dormant (`service_impl/src/pdf_export_scheduler.rs:270-285`).
- **No active shift plan:** the Scheduler persists *"No active shift
  plan available"* and returns `Ok(())`
  (`service_impl/src/pdf_export_scheduler.rs:352-358`).
- **All weeks in the horizon failed (v2.3.1):**
  `succeeded_count == 0` → no `record_success`, otherwise the UI would
  suggest success (`service_impl/src/pdf_export_scheduler.rs:433`).
- **`now_local()` `IndeterminateOffset`:** the renderer falls back to
  `now_utc` and logs `warn!`
  (`service_impl/src/pdf_shiftplan.rs:114`) — no panic for a purely
  cosmetic footer info (D-50-12).

## 8. Tests

### Unit / in-memory

- **`service_impl/src/test/pdf_export_config.rs`** (~570 LOC):
  - `get_non_admin_forbidden`, `update_non_admin_forbidden` — 403 in
    the basic service.
  - `update_with_empty_token_keeps_existing`,
    `update_with_set_token_replaces_existing` — token merge semantics.
  - `snapshot_version_unchanged_grep_gate` — guard against unnecessary
    schema-version bumps.
- **`service_impl/src/test/pdf_shiftplan.rs`** (~420 LOC):
  - `happy_path_returns_bytes` — bytes return for a Planned week.
  - `week_status_locked_returns_bytes`,
    `week_status_unset_returns_validation_error`,
    `week_status_in_planning_returns_validation_error` — gate matrix.
  - `filters_deleted_sales_persons` +
    `service_render_does_not_leak_deleted_sales_persons` — D-49-05.
  - `service_forwards_caller_context_to_dependencies` — D-49-07 (auth
    passthrough).
  - `view_error_bubbles_up`, `sales_person_error_bubbles_up`,
    `week_status_error_bubbles_up` — error propagation.
  - `content_disposition_filename_format_helper` — `filename_for`.
  - `now_local_fallback_to_utc_on_indeterminate_offset` — D-50-12
    fallback wiring.
- **`service_impl/src/test/pdf_export_scheduler.rs`** (~840 LOC):
  - `disabled_config_skips_run` — enabled=false ⇒ no-op.
  - `incomplete_config_records_error` — mandatory fields missing.
  - `happy_path_renders_horizon_and_uploads` — horizon loop end-to-end.
  - `webdav_transient_fail_after_3_retries_records_error` +
    `permanent_401_records_error_immediately` — retry behavior.
  - `year_week_wraps_correctly` — ISO week transition at year boundary.
  - `scheduler_calls_pdf_shiftplan_service_with_full_auth` — full auth
    on the cron path.
  - `scheduler_skips_week_on_validation_error` +
    `scheduler_continues_past_validation_error_for_later_weeks` —
    v2.3.1 per-week skip.
  - `boot_trigger_reload_flow` — boot sequence.
- **`service_impl/src/webdav_client.rs`** embedded tests (line 341 ff.,
  wiremock-based):
  - Happy path, MKCOL 405 = success, MKCOL 201 + PUT 201,
    transient-retry success on the 3rd attempt, permanent 401 without
    retry, transient exhausted, Debug-impl leak guard (T-48-08),
    `classify(...)` unit tests.
- **`rest/src/pdf_shiftplan.rs`** embedded handler-helper tests (line
  179 ff.):
  - `week_status_allows_download` status matrix,
  - `not_releasable_returns_409_json_with_stable_error_code`,
  - `pdf_response_sets_pdf_content_type_and_filename` +
    leading-zero + KW52 variants.
- **`service_impl/src/test/block.rs`** covers the iCal chain via
  `MockIcalService` (`service_impl/src/test/block.rs:200-233`).

### Integration

Full-stack router coverage of the PDF handlers is intentionally NOT in
the `rest` crate, because `RestStateDef` has 37 assoc types; it belongs
to the `shifty_bin` integration suite (see comment in
`rest/src/pdf_shiftplan.rs:165-176`).

### Known gaps

- **iCal**: no dedicated snapshot test of the TZID output for
  configured non-UTC zones; only the conversion contract is mocked.
- **PDF renderer**: byte determinism was intentionally given up in
  Phase 50 (`FIXED_METADATA_TIMESTAMP` remains for trailer/metadata,
  but the visible timestamp header varies). Snapshot tests of the
  visual layout do not exist — regressions are caught by UAT.
- **Multi-shiftplan week**: v1 simplification "first non-deleted shift
  plan" is tested but not documented as a business rule outside the
  module docs.

## 9. History & Context

- **Phase 48 (v2.2)** — Nextcloud PDF export EXP-01/02/03:
  - EXP-01: base renderer (`printpdf` 0.7) + WebDAV client + cron loop.
  - EXP-02: admin config table + REST + frontend card.
  - EXP-03: retry persistence, trigger endpoint, reload-after-update
    flow.
  - Decisions: D-48-01 (token plaintext), D-48-02 (token response
    mask), D-48-ADMIN (admin-only), D-48-REST (token merge), D-48-BASIC
    (tier classification), D-48-PDF (`printpdf` choice),
    D-48-PDF-ACTIVE-ONLY (non-deleted filter).
- **v2.3.1** — boot tolerance + per-week skip + cron 6-field fix:
  - Data-fix migration `20260704000000_fix-pdf-export-cron-6-field.sql`.
  - `run_once_now`: ValidationError → `continue` instead of abort.
  - `record_success` only if `succeeded_count > 0`.
- **Phase 49 (v2.3)** — On-demand weekly download PDF-03/04/05:
  - New business-logic service `PdfShiftplanService` as the DRY core
    for the REST handler and the Scheduler.
  - Decisions: D-49-03 (409 body format), D-49-05 (active filter in the
    assemble), D-49-06 (status gate `Planned`/`Locked`), D-49-07 (no
    full-auth bypass), D-49-08 (scheduler delegation).
- **Phase 50 (v2.3)** — Browser-look renderer:
  - Landscape A4 grid, hybrid stack layout, created-at header, row
    alignment across day columns.
  - Decisions: D-50-01/02/…/17 (layout), D-50-11 (no implicit TZ
    conversion in the renderer), D-50-12 (`now_local` UTC fallback),
    D-50-13 (byte determinism given up).
- **iCal** — pre-v2 feature; no dedicated phase. Sits on the
  BlockService and serves personal calendar sync.
- Context reads (planning artifacts):
  `.planning/phases-archive/…-48-*`, `…-49-*`, `…-50-*`.

---

**Conclusion:** the export cluster splits into two paths: the local
on-demand download (user auth, status gate, no full-auth bypass) and
the scheduler-driven Nextcloud push (admin config, full-auth internals,
retry with persistence). The iCal feed lives alongside as an old,
middleware-bypassed pre-v2 module and awaits TZID hardening.

*Last verification against code:* see git blame of this file.
