# Feature: Export — PDF, iCal, WebDAV, Scheduler

> **Kurzform:** Wochen-Schichtpläne als PDF (On-Demand-Download oder
> cron-getriggerter WebDAV-Push nach Nextcloud) und persönliche
> Einsatz-Blöcke als iCal-Feed für den Kalender-Client — plus die
> zugehörige Admin-Konfig-Oberfläche.

**Cluster-ID:** F11
**Status:** produktiv
**Erstmalig eingeführt:** Milestones v2.2 (Phase 48 — Nextcloud-Export,
Scheduler-Loop, WebDAV-Client), v2.3 (Phase 49 — On-Demand-Download,
Phase 50 — Browser-Look-Renderer). iCal-Feed existiert bereits seit dem
Block-Service (Pre-v2).
**Zuständige Crates:**
`service::pdf_export`, `service::pdf_export_config`,
`service::pdf_shiftplan`, `service::ical`,
`service_impl::pdf_export_scheduler`, `service_impl::pdf_export_config`,
`service_impl::pdf_shiftplan`, `service_impl::pdf_render`,
`service_impl::webdav_client`, `service_impl::ical`,
`dao::pdf_export_config`, `dao_impl_sqlite::pdf_export_config`,
`rest::pdf_export_config`, `rest::pdf_shiftplan`, `rest::sales_person`
(iCal-Handler).

---

## 1. Was ist das? (Fachlich)

Shifty exportiert Schichtpläne über drei orthogonale Kanäle:

1. **On-Demand-PDF-Download** — Jeder authentifizierte Employee kann in
   der Wochenansicht per Klick das PDF der aktuellen Woche
   herunterladen (`GET /shiftplan/{id}/{y}/{w}/pdf`). Das PDF sieht aus
   wie die Browser-Wochenansicht: Landscape-A4-Grid mit Tages-Spalten
   und namensbelegten Slot-Boxen.
2. **Scheduler-getriebener Nextcloud-Export** — Ein Admin
   konfiguriert einmalig WebDAV-URL, User, App-Token, Zielordner,
   Wochen-Horizont und einen Cron-Ausdruck. Der Server rendert dann
   zeitgesteuert die nächsten *N* Wochen und legt sie im Nextcloud-
   Ordner ab. Das Feature ist per Default aus (`enabled=0`).
3. **iCal-Feed pro Sales Person** — Der Frontend-Kalender-Sync-Link
   `GET /sales-person/{id}/ical` liefert die geplanten Blöcke der
   nächsten Wochen als `text/calendar` — der Employee abonniert den
   Link im eigenen Kalender-Client und sieht seine Schichten dort.

**Beispiel-Workflow On-Demand-PDF (User-Sicht):**

1. Employee öffnet Wochenansicht → sieht Download-Icon in der
   Woche.
2. Klick auf Icon → Browser lädt
   `schichtplan-2026-KW27.pdf` herunter.
3. Bei nicht-releasbarer Woche (Status `Unset` / `InPlanning`)
   antwortet der Server mit 409 und das FE zeigt eine Fehlermeldung —
   der Download bleibt aus.

**Beispiel-Workflow Nextcloud-Export (Admin-Sicht):**

1. Admin öffnet *Settings → PDF-Export nach Nextcloud*.
2. Trägt Cloud-URL, User, App-Token, Zielordner, Wochen-Horizont
   und Cron-Ausdruck ein, aktiviert `enabled` und speichert.
3. Der Scheduler-Reload registriert den neuen Cron-Ausdruck ohne
   Server-Restart. Nach dem nächsten Cron-Tick liegt für jede Woche im
   Horizont eine Datei `schichtplan-YYYY-KWnn.pdf` im WebDAV-Ordner.
4. Optional: Button *„Jetzt exportieren"* triggert einen sofortigen
   Lauf.
5. Status-Anzeige (`last_success_at`, `last_error_at`,
   `last_error_message`) zeigt Erfolg oder das jüngste Fehler-Detail.

## 2. Fachliche Regeln

- **PDF-Wochenstatus-Gate (D-49-06):** Ein Wochen-PDF darf nur für
  Wochen im Status `Planned` oder `Locked` ausgeliefert werden. `Unset`
  und `InPlanning` liefern 409 (Handler-Pre-Check,
  `rest/src/pdf_shiftplan.rs:130`) bzw. `ValidationError` (Service-Gate,
  `service_impl/src/pdf_shiftplan.rs:140`).
- **Aktive Sales Persons (D-49-05, D-48-PDF-ACTIVE-ONLY):** Der
  Renderer bekommt nur non-deleted Sales Persons; Filter passiert im
  Assemble-Service via `filter_active`
  (`service_impl/src/pdf_shiftplan.rs:89`). Der Renderer selbst filtert
  nicht.
- **Filename-Konvention:** `schichtplan-{JJJJ}-KW{NN:02}.pdf` (ASCII,
  keine RFC-5987-Codierung nötig). Definiert in
  `service/src/pdf_shiftplan.rs:68` `filename_for(year, week)` und
  konsumiert von REST-Handler und Scheduler.
- **Auth-Weitergabe im PDF-Assemble (D-49-07):** Der `PdfShiftplanService`
  reicht den *Aufrufer*-Kontext an alle konsumierten Services weiter;
  **niemals** wird intern auf `Authentication::Full` hochgehebelt. Zitat
  aus `service/src/pdf_shiftplan.rs:21`: *„niemals wird intern auf
  `Authentication::Full` hochgehebelt."* — d.h. wenn ein Employee ohne
  Sonderrechte den Download aufruft, wird die Chain (WeekStatus, View,
  SalesPersons) mit genau dieser eingeschränkten Auth durchlaufen. Der
  Scheduler ist der einzige Aufrufer, der `Authentication::Full`
  einsetzt — legitim, weil er intern trusted ist.
- **Admin-Gate PDF-Export-Config (D-48-ADMIN):**
  `get`/`update`/`trigger` auf `/pdf-export-config` erfordern die
  `admin`-Rolle. Non-Admin → 403
  (`service_impl/src/pdf_export_config.rs:54`).
- **Token-Merge-Semantik (D-48-REST):** Im PUT-Body bedeutet
  `webdav_app_token = None` „bestehenden Wert behalten",
  `Some(v)` „neuen Wert setzen". Der Merge sitzt im Basic-Service
  (`service_impl/src/pdf_export_config.rs:77`).
- **Token niemals in Response (T-48-02):** `From<&PdfExportConfig> for
  PdfExportConfigTO` setzt `webdav_app_token` immer auf `None`
  (`rest-types/src/lib.rs:2271`). Der Token verlässt niemals den Server
  in einer HTTP-Response.
- **Scheduler-only Status-Recorder:** `record_success` /
  `record_error` prüfen `check_only_full_authentication`
  (`service_impl/src/pdf_export_config.rs:118`) — nur der Scheduler
  (`Authentication::Full`) darf schreiben; kein admin-facing
  Public-API-Pfad. Der `trigger_export_now`-Handler prüft Admin, spawnt
  dann intern mit Full-Auth
  (`rest/src/pdf_export_config.rs:147-153`).
- **v1-Vereinfachung Multi-Shiftplan:** Der Scheduler exportiert pro
  Woche **genau einen** Shiftplan — den ersten non-deleted aus
  `ShiftplanService::get_all` (`service_impl/src/pdf_export_scheduler.rs:346`).
  Multi-Shiftplan-Aggregation ist Follow-up.
- **Scheduler exportiert nur releasbare Wochen (D-49-08 / Q1):** Das
  Assemble delegiert an `PdfShiftplanService::render_week_pdf`; Wochen
  in `Unset`/`InPlanning` kommen als `ValidationError` zurück, werden
  via `record_error` protokolliert und mit `continue` übersprungen
  — spätere Planned-Wochen im Horizon werden weiter versucht
  (`service_impl/src/pdf_export_scheduler.rs:379-395`, v2.3.1
  Verbesserung).
- **Retry & Klassifikation WebDAV:** Pro Upload-Lauf 3 Versuche mit
  Backoff `2s/4s/8s` (`webdav_client.rs:29`). 2xx = Erfolg,
  MKCOL 405 = „Ordner existiert" = Erfolg, 5xx / IO = transient, 4xx
  (außer MKCOL 405) = permanent, kein Retry (`webdav_client.rs:90`).
  Pro Request 30s Timeout (T-48-10).
- **Upload-Abbruch-Semantik:** Beim ersten Upload-Fehler in einem Lauf
  bricht die Loop ab (`return Ok(())`), damit die nächste Cron-Slot-
  Runde von vorn anfängt — kein Weitermachen mit vermutlich kaputtem
  WebDAV-Endpoint (`service_impl/src/pdf_export_scheduler.rs:422`).
- **`record_success` nur bei tatsächlichem Upload (v2.3.1):** Nur wenn
  `succeeded_count > 0` wird Erfolg persistiert — sonst würde die UI
  Erfolg suggerieren obwohl gar keine Woche im Cloud-Ordner landete
  (`service_impl/src/pdf_export_scheduler.rs:433`).
- **Boot-Toleranz:** Ein defekter Cron-Ausdruck oder fehlgeschlagener
  Initial-Reload verhindert den Backend-Start NICHT — der Scheduler
  startet dormant und persistiert die Diagnose via `record_error`
  (`service_impl/src/pdf_export_scheduler.rs:202-206`).
- **iCal-Zeitfenster:** Der iCal-Feed liefert die 12 nächsten Wochen ab
  „jetzt minus 2 Wochen" (also 2 Wochen Vergangenheit + 10 Wochen
  Zukunft) — `service_impl/src/block.rs:218`.
- **iCal-TZID:** Der TZID-Wert kommt aus `ConfigService.get_config().timezone`
  und wird pro `DTSTART`/`DTEND` gesetzt
  (`service_impl/src/ical.rs:24-33`) — keine Recurrence-Rules.
- **iCal ohne Session-Gate:** Der `/ical`-Pfad wird von der
  Session-Middleware bewusst ohne Auth durchgelassen
  (`rest/src/session.rs:281`), damit Kalender-Clients den Feed ohne
  Cookie/OIDC-Login abonnieren können. Kompensation: die URL enthält
  die Sales-Person-UUID als schwer erratbaren Token.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `pdf_export_config` | Single-Row-Konfig des Nextcloud-Exports (analog `paid_limit_config` / `holiday_stichtag_config`) | `id` (fixe UUID `…0000048`), `enabled`, `nextcloud_url`, `webdav_user`, `webdav_app_token` (Klartext, D-48-01), `target_folder`, `weeks_horizon`, `cron_schedule`, `last_success_at`, `last_error_at`, `last_error_message`, `update_process`, `update_version` |

Für iCal und On-Demand-PDF gibt es **keine eigene Persistenz** — beides
liest aus `shiftplan` / `booking` / `sales_person` / `week_status`.

### Migrations

- `migrations/sqlite/20260703000000_create-pdf-export-config.sql` —
  Basis­tabelle + Seed-Row (`enabled=0`, `weeks_horizon=8`,
  `cron_schedule='0 6 * * 1'`).
- `migrations/sqlite/20260704000000_fix-pdf-export-cron-6-field.sql` —
  v2.3.1-Hotfix: Data-Fix für `cron_schedule`. `tokio-cron-scheduler`
  0.15 nutzt `croner` 3.0 im 6-Feld-Format (`sec min hour dom mon dow`);
  die Ursprungs-Migration seedete jedoch das 5-Feld-Muster. Length-based
  Detection (genau vier Leerzeichen ⇒ 5 Felder) stellt ein
  `'0 '` voran.

### Beziehungen

Der Config-Row ist an keinen Fremdschlüssel gebunden — reine
Applikations-Konfig. Das Klartext-Token in der DB ist bewusste
Ops-Entscheidung (D-48-01); der Schutz kommt über
Dateisystem-Permissions.

## 4. Service-API

Vier Traits, zwei Tiers (per Service-Tier-Konvention):

### 4.1 `service::pdf_export_config::PdfExportConfigService` (Basic)

Entity-Manager für die Config-Row. Konsumiert nur DAO + Permission +
Clock + Uuid + Transaction — **kein** Domain-Service als Dependency
(`service/src/pdf_export_config.rs:9`).

Methoden:

- `get(context, tx) -> PdfExportConfig` — admin-gated.
- `update(update, context, tx) -> PdfExportConfig` — admin-gated,
  Token-Merge inside.
- `record_success(at, context, tx)` — nur Full-Auth (Scheduler).
- `record_error(at, message, context, tx)` — nur Full-Auth.

### 4.2 `service::pdf_shiftplan::PdfShiftplanService` (Business-Logic)

Assembler für den DRY-Kern `render_week_pdf`. Ein einziger Einstieg für
REST-Handler und Scheduler.

Reihenfolge im Assemble-Path
(`service_impl/src/pdf_shiftplan.rs:126-170`):

1. `WeekStatusService::get_week_status` — Gate.
2. `ShiftplanViewService::get_shiftplan_week` — teurer Read.
3. `SalesPersonService::get_all` + `filter_active`.
4. `pdf_render::render_shiftplan_week_pdf` — pure Funktion, gibt
   `Vec<u8>` zurück.

### 4.3 `service::pdf_export::PdfExportScheduler` (Business-Logic)

Kapselt Cron-Loop, WebDAV-Upload, Retry-Persistenz. Methoden:

- `start()` — beim App-Boot; initialisiert `JobScheduler` und ruft
  `reload_from_db()` (boot-tolerant).
- `reload_from_db()` — nach `PUT /pdf-export-config`: Alt-Job entfernen,
  neuen Cron registrieren (`service_impl/src/pdf_export_scheduler.rs:217`).
- `run_once_now(context)` — synchroner Einzel-Lauf. Cron-Callback ruft
  mit `Full`; REST-Trigger ruft nach Admin-Check mit `Full`.

### 4.4 `service::ical::IcalService`

Rein synchrones, deps-loses Trait mit einer Methode:
`convert_blocks_to_ical_string(blocks, title, timezone) -> Arc<str>`.
Der eigentliche „iCal-für-Sales-Person"-Endpoint hängt am
`BlockService::get_blocks_for_next_weeks_as_ical`
(`service_impl/src/block.rs:218`), der `IcalService` als Dep konsumiert.

### Auth-Gates (Übersicht)

| Methode | Gate |
| --- | --- |
| `PdfExportConfigService::{get,update}` | `admin`-Privileg |
| `PdfExportConfigService::{record_success,record_error}` | `Authentication::Full` |
| `PdfShiftplanService::render_week_pdf` | Kein eigenes Gate — reicht `context` an konsumierte Services weiter |
| `PdfExportScheduler::run_once_now` | `check_only_full_authentication` (REST-Trigger konvertiert Admin → Full) |
| `IcalService` / iCal-Handler | Kein Gate (Middleware-Bypass, siehe §2) |

### TX-Verhalten

- `PdfExportConfigServiceImpl` öffnet TX via
  `transaction_dao.use_transaction(tx)`, committet am Ende jeder Methode.
- `PdfShiftplanServiceImpl::render_week_pdf` konsumiert `tx` und reicht
  ihn an alle Sub-Calls (`.clone()`) — **committet selbst nicht**;
  Aufrufer entscheidet.
- Scheduler ruft alle Sub-Services mit `tx=None` — jeder öffnet seine
  eigene TX, Isolation-per-Read.

### Dependencies

- `PdfExportConfigServiceImpl`: `PdfExportConfigDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao`.
- `PdfShiftplanServiceImpl`: `ShiftplanViewService`, `SalesPersonService`,
  `WeekStatusService`, `PermissionService`, `TransactionDao`.
- `PdfExportSchedulerImpl`: `PdfExportConfigService`, `PdfShiftplanService`,
  `ShiftplanService` (Catalog), `PermissionService`, `ClockService`,
  `TransactionDao` + `WebDavUploadFactory` (Custom-Field).
- `IcalServiceImpl`: keine — pure Konvertierung.
- `WebDavClient`: kein Trait im `service`-Crate, direkte Impl in
  `service_impl`. Abstraktion nach außen via `WebDavUpload`-Trait
  (`service_impl/src/webdav_client.rs:63`), damit der Scheduler in Tests
  mockbar bleibt.

**[Zu prüfen]** — `webdav_client` liegt bewusst nur in `service_impl`,
ohne Trait im `service`-Crate. Konsequenz: Tests, die keinen echten
HTTP-Endpoint wollen, injizieren via `WebDavUploadFactory` einen Mock.

## 5. REST-Endpoints

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/pdf-export-config` | Aktuelle Konfig (Token maskiert) | — | `PdfExportConfigTO` | 403 (Non-Admin) |
| `PUT` | `/pdf-export-config` | Konfig setzen; leeres Token behält bestehenden Wert; triggert `reload_from_db` | `PdfExportConfigTO` | `PdfExportConfigTO` | 403, 500 |
| `POST` | `/pdf-export-config/trigger` | Sofortiger Einzel-Lauf (`tokio::spawn`) | — | 204 No Content | 403, 500 |
| `GET` | `/shiftplan/{shiftplan_id}/{year}/{week}/pdf` | On-Demand-Wochen-PDF, `application/pdf` + `Content-Disposition: attachment; filename="schichtplan-YYYY-KWnn.pdf"` | — | Bytes | 401, 404, **409 `{"error":"week-not-releasable"}`**, 422 (ValidationError als Fallback aus Service-Gate), 500 |
| `GET` | `/sales-person/{id}/ical` | iCal-Feed der nächsten 12 Wochen (2 Wochen Vergangenheit + 10 Zukunft) für die angegebene Sales Person; `text/calendar` | — | Body als iCal-Text | 404, 500 |

DTOs siehe `rest-types::PdfExportConfigTO` (`rest-types/src/lib.rs:2244`).
Für iCal und PDF-Download gibt es kein JSON-DTO — die Antworten sind
Byte- bzw. Text-Streams.

Response-Fluss `POST /pdf-export-config/trigger`
(`rest/src/pdf_export_config.rs:147`): Handler ruft
`pdf_export_config_service.get(context, ..)` als **Admin-Gate**,
`tokio::spawn`t dann `scheduler.run_once_now(Authentication::Full)` und
antwortet 204 — der Cron-Path und der Trigger-Path teilen sich den
selben Full-Auth-Aufrufer.

Response-Fluss `GET /shiftplan/{...}/pdf`
(`rest/src/pdf_shiftplan.rs:117`): Fast-path-Pre-Check zieht
`WeekStatusService::get_week_status`; nicht-releasbare Weeks kurzschließen
mit 409 JSON, bevor der teure Render-Path startet.

## 6. Frontend-Integration

- **Pages:**
  - `shifty-dioxus/src/page/settings.rs:940` — Card *„PDF-Export nach
    Nextcloud"* (admin-gated). Formular für alle Felder aus
    `PdfExportForm`, Save-Button, „Jetzt exportieren"-Trigger-Button,
    Status-Row (`last_success_at` / `last_error_message`).
  - `shifty-dioxus/src/page/shiftplan.rs:1176` — Download-Anchor
    `href="{backend}/shiftplan/{sp_id}/{y}/{w}/pdf"` mit
    `download="schichtplan-{y}-KW{w:02}.pdf"`. Kein FE-Weekstatus-Guard
    — der Server entscheidet per 409.
- **API-Client:** `shifty-dioxus/src/api.rs:1869-1907` — drei
  Wrapper-Funktionen `get_pdf_export_config`, `put_pdf_export_config`,
  `trigger_pdf_export`.
- **Loader:** `shifty-dioxus/src/loader.rs:966-987` — bridge zwischen
  API und `PdfExportForm`-State inklusive Form-↔-DTO-Konvertierung
  (`pdf_export_form_from_response`, `pdf_export_form_to_put_body`).
- **State:** `shifty-dioxus/src/state/pdf_export_config.rs` —
  `PdfExportForm`, `clamp_weeks_horizon`.
- **i18n-Keys:** `SettingsPdfExportTitle`, `SettingsPdfExportHelp`, …
  (`shifty-dioxus/src/i18n/de.rs:1278` ff.). Cs/En-Pendants dort
  ebenfalls gepflegt.
- **Proxy:** `Dioxus.toml` proxied `/pdf-export-config`,
  `/shiftplan` **und** `/sales-person` — d.h. sowohl die
  Config-CRUD-Route als auch die PDF-Download-Route (die unter
  `/shiftplan` hängt, `rest/src/lib.rs:666`) und die iCal-Route
  (unter `/sales-person`, `rest/src/lib.rs:640`) gehen ohne
  zusätzlichen Proxy-Eintrag. **[Zu prüfen]** — Es gibt keinen
  eigenen `[[web.proxy]] backend=".../pdf-export-config"`-Eintrag; er
  wird über das generische `/settings`-Nachbar-Setup nicht abgedeckt.
  Grep bestätigt: kein Match auf `pdf-export` in `Dioxus.toml`. Für
  `dx serve --hot-reload` sollte das nachgeholt werden, wenn die
  Card 404 zeigt (Muster aus F-Memory *„Dioxus.toml Proxy für neue
  Backend-Endpoints"*).

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektion
[*14. Export & Externe Integrationen*](../domain/edge-cases.md#14-export--externe-integrationen).

Feature-spezifische Kanten:

- **Wochenstatus nicht releasbar (D-49-06):** REST-Pre-Check antwortet
  409 mit stabilem `{"error":"week-not-releasable"}`; das
  Service-interne Gate fängt Race-Windows und Direct-Impl-Aufrufer
  (Scheduler) ab und mappt zu 422 (`ValidationError`).
- **Auth-Weitergabe (D-49-07):** *Zitat aus
  `service/src/pdf_shiftplan.rs:21`:* „(D-49-07); niemals wird intern
  auf `Authentication::Full` hochgehebelt." Die gesamte PDF-Assemble-
  Chain respektiert die eingeschränkte User-Auth statt sie zu
  bypassen — im Unterschied zum reporting/booking-information-Aggregat,
  das explizit Full-Auth zieht. Für Employee-Rollen kann der
  ShiftplanView also weniger Informationen zurückliefern; der Renderer
  spiegelt entsprechend nur das, was der User ohnehin im FE sehen darf.
- **Leerer Zeitraum:** Renderer zeichnet ein leeres Grid mit
  Tages-Headern und einer Timestamp-Zeile; keine Sonderbehandlung. Für
  „gar keine Sunday-Buchung" wird die Sonntag-Spalte weggelassen
  (`service_impl/src/pdf_render.rs:34`).
- **Special-Days-Overlay:** Der aktuelle Renderer legt **keinen**
  Feiertags-Marker über die Grid-Zelle — Feiertag = leere Zelle wie
  jede andere. **[Zu prüfen]** — im Backlog notiert.
- **Sales Person mit sehr langem Namen:** Slot-Box wächst vertikal, um
  alle Namen zu fassen (UAT-Revision D-50-04); Column-Overflow → `+ N
  weitere`-Marker unten (`service_impl/src/pdf_render.rs:56-60`).
- **iCal-TZID:** Der TZID wird 1:1 aus `ConfigService.timezone`
  gestempelt (`service_impl/src/ical.rs:26-32`). Bei falsch
  konfiguriertem `timezone` (z.B. `UTC` obwohl Slots in Local geplant
  sind) verschieben Kalender-Clients die Anzeige. Recurrence-Rules
  (`RRULE`) werden **nicht** emittiert — jede Woche ist ein
  Einzelfenster.
- **iCal-Vergangenheitsfenster:** `now - 2 Wochen` als Start
  garantiert, dass ein Kalender-Client, der das ical zuerst am
  Dienstag abruft, auch das laufende Wochenende noch sieht.
- **WebDAV-Auth-Fehler:** 401 → Klassifikation `Permanent`, kein Retry,
  `record_error` persistiert *„WebDAV-Upload … permanent
  fehlgeschlagen (401)"* (`service_impl/src/pdf_export_scheduler.rs:407`).
  Der WebDAV-Fehler enthält niemals den Basic-Auth-Header
  (`WebDavError`-Display maskiert; `header_value.set_sensitive(true)`
  in `webdav_client.rs:160`).
- **WebDAV-Transient-Erschöpfung:** 3 Versuche mit `2s/4s/8s`; danach
  `Transient { attempts: 3 }` → `record_error`, Scheduler beendet den
  Lauf (`return Ok(())`), Cron versucht es beim nächsten Tick erneut.
- **Ordner existiert bereits:** MKCOL 405 gilt als Erfolg
  (`webdav_client.rs:91-99`) — Nextcloud-Standard-Verhalten für
  existierende Ordner.
- **Ungültiger Cron-Ausdruck:** `Job::new_async` schlägt fehl → Fehler
  in `pdf_export_config.last_error_message` persistiert, Scheduler
  bleibt dormant (`service_impl/src/pdf_export_scheduler.rs:270-285`).
- **Kein aktiver Shiftplan:** Scheduler persistiert *„Kein aktiver
  Shiftplan vorhanden"* und returned `Ok(())`
  (`service_impl/src/pdf_export_scheduler.rs:352-358`).
- **Alle Wochen im Horizon gescheitert (v2.3.1):**
  `succeeded_count == 0` → kein `record_success`, sonst würde die UI
  Erfolg suggerieren (`service_impl/src/pdf_export_scheduler.rs:433`).
- **`now_local()` `IndeterminateOffset`:** Renderer fällt auf
  `now_utc` zurück und loggt `warn!`
  (`service_impl/src/pdf_shiftplan.rs:114`) — kein Panic wegen einer
  reinen Footer-Info (D-50-12).

## 8. Tests

### Unit / In-Memory

- **`service_impl/src/test/pdf_export_config.rs`** (~570 LOC):
  - `get_non_admin_forbidden`, `update_non_admin_forbidden` — 403 im
    Basic-Service.
  - `update_with_empty_token_keeps_existing`,
    `update_with_set_token_replaces_existing` — Token-Merge-Semantik.
  - `snapshot_version_unchanged_grep_gate` — Guard gegen unnötige
    Schema-Version-Bumps.
- **`service_impl/src/test/pdf_shiftplan.rs`** (~420 LOC):
  - `happy_path_returns_bytes` — Bytes-Return für Planned-Week.
  - `week_status_locked_returns_bytes`,
    `week_status_unset_returns_validation_error`,
    `week_status_in_planning_returns_validation_error` — Gate-Matrix.
  - `filters_deleted_sales_persons` +
    `service_render_does_not_leak_deleted_sales_persons` — D-49-05.
  - `service_forwards_caller_context_to_dependencies` — D-49-07
    (Auth-Passthrough).
  - `view_error_bubbles_up`, `sales_person_error_bubbles_up`,
    `week_status_error_bubbles_up` — Fehler-Propagation.
  - `content_disposition_filename_format_helper` — `filename_for`.
  - `now_local_fallback_to_utc_on_indeterminate_offset` — D-50-12
    Fallback-Verkabelung.
- **`service_impl/src/test/pdf_export_scheduler.rs`** (~840 LOC):
  - `disabled_config_skips_run` — enabled=false ⇒ noop.
  - `incomplete_config_records_error` — Pflichtfelder fehlen.
  - `happy_path_renders_horizon_and_uploads` — Horizon-Loop End-to-End.
  - `webdav_transient_fail_after_3_retries_records_error` +
    `permanent_401_records_error_immediately` — Retry-Verhalten.
  - `year_week_wraps_correctly` — ISO-Wochen-Übergang Jahresgrenze.
  - `scheduler_calls_pdf_shiftplan_service_with_full_auth` — Full-Auth
    im Cron-Path.
  - `scheduler_skips_week_on_validation_error` +
    `scheduler_continues_past_validation_error_for_later_weeks` —
    v2.3.1 per-Week-Skip.
  - `boot_trigger_reload_flow` — Boot-Sequenz.
- **`service_impl/src/webdav_client.rs`** eingebettete Tests
  (Zeile 341 ff., wiremock-basiert):
  - Happy Path, MKCOL 405 = Success, MKCOL 201 + PUT 201,
    Transient-Retry-Erfolg im 3. Versuch, Permanent 401 ohne Retry,
    Transient exhausted, Debug-Impl-Leak-Guard (T-48-08),
    `classify(...)` Unit-Tests.
- **`rest/src/pdf_shiftplan.rs`** eingebettete Handler-Helper-Tests
  (Zeile 179 ff.):
  - `week_status_allows_download` Status-Matrix,
  - `not_releasable_returns_409_json_with_stable_error_code`,
  - `pdf_response_sets_pdf_content_type_and_filename` +
    Leading-Zero + KW52 Varianten.
- **`service_impl/src/test/block.rs`** deckt die iCal-Chain via
  `MockIcalService` ab (`service_impl/src/test/block.rs:200-233`).

### Integration

Full-Stack-Router-Coverage der PDF-Handler ist bewusst NICHT im
`rest`-Crate, weil `RestStateDef` 37 assoc-types hat; sie gehört in die
`shifty_bin`-Integration-Suite (siehe Kommentar in
`rest/src/pdf_shiftplan.rs:165-176`).

### Bekannte Lücken

- **iCal**: kein dedizierter Snapshot-Test der TZID-Ausgabe für
  konfigurierte Nicht-UTC-Zonen; nur der Konvertierungsvertrag ist
  gemockt.
- **PDF-Renderer**: Byte-Determinismus wurde in Phase 50 bewusst
  aufgegeben (`FIXED_METADATA_TIMESTAMP` bleibt für die
  Trailer/Metadaten, aber der sichtbare Timestamp-Header variiert).
  Snapshot-Tests des visuellen Layouts existieren nicht — Regressionen
  fängt UAT.
- **Multi-Shiftplan-Woche**: v1-Vereinfachung „erster non-deleted
  Shiftplan" ist getestet, aber nicht als Business-Rule dokumentiert
  außerhalb des Modul-Docs.

## 9. Historie & Kontext

- **Phase 48 (v2.2)** — Nextcloud-PDF-Export EXP-01/02/03:
  - EXP-01: Basis-Renderer (`printpdf` 0.7) + WebDAV-Client + Cron-Loop.
  - EXP-02: Admin-Config-Tabelle + REST + FE-Card.
  - EXP-03: Retry-Persistenz, Trigger-Endpoint,
    Reload-nach-Update-Flow.
  - Decisions: D-48-01 (Token Klartext), D-48-02 (Token-Response-Mask),
    D-48-ADMIN (admin-only), D-48-REST (Token-Merge), D-48-BASIC
    (Tier-Klassifikation), D-48-PDF (`printpdf`-Wahl),
    D-48-PDF-ACTIVE-ONLY (non-deleted-Filter).
- **v2.3.1** — Boot-Tolerance + per-Week-Skip + Cron-6-Feld-Fix:
  - Data-Fix-Migration `20260704000000_fix-pdf-export-cron-6-field.sql`.
  - `run_once_now`: ValidationError → `continue` statt Abbruch.
  - `record_success` nur bei `succeeded_count > 0`.
- **Phase 49 (v2.3)** — On-Demand-Wochen-Download PDF-03/04/05:
  - Neuer Business-Logic-Service `PdfShiftplanService` als DRY-Kern für
    REST-Handler und Scheduler.
  - Decisions: D-49-03 (409-Body-Format), D-49-05 (Aktiv-Filter im
    Assemble), D-49-06 (Status-Gate `Planned`/`Locked`), D-49-07
    (Kein Full-Auth-Bypass), D-49-08 (Scheduler-Delegation).
- **Phase 50 (v2.3)** — Browser-Look-Renderer:
  - Landscape-A4-Grid, Hybrid-Stack-Layout, Erstellt-am-Header,
    Row-Alignment über Tages-Spalten.
  - Decisions: D-50-01/02/…/17 (Layout), D-50-11 (kein implizites TZ-
    Conversion im Renderer), D-50-12 (`now_local` UTC-Fallback), D-50-13
    (Byte-Determinismus-Aufgabe).
- **iCal** — Pre-v2 Feature; keine dedizierte Phase. Sits am
  BlockService und dient dem persönlichen Kalender-Sync.
- Kontext-Reads (Planning-Artefakte):
  `.planning/phases-archive/…-48-*`, `…-49-*`, `…-50-*`.

---

**Fazit:** Der Export-Cluster teilt sich zwei Wege: den lokalen
On-Demand-Download (User-Auth, Status-Gate, kein Full-Auth-Bypass) und
den Scheduler-getriebenen Nextcloud-Push (Admin-Konfig, Full-Auth-
Interna, Retry mit Persistenz). Der iCal-Feed lebt als altes,
Middleware-bypasstes Pre-v2-Modul daneben und wartet auf TZID-Härtung.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
