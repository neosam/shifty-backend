# Phase 48: Nextcloud-PDF-Export via WebDAV (BE + Admin-UI) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous discuss (Textform, 8 Fragen + 2 Rückfragen zu Konfig-Ort)

<domain>
## Phase Boundary

Ein regelmäßiger Backend-Task rendert die Wochen-Schichtpläne als PDF und lädt sie per WebDAV nach Nextcloud hoch. Aktivierung, Zielkonfiguration und Zeitplan sind admin-gated und über eine Settings-Card editierbar. Alle Konfigurationswerte inkl. Nextcloud-App-Token liegen in der Datenbank (bewusste Abweichung vom ursprünglichen ROADMAP-Success-Criterion 2).

</domain>

<decisions>
## Implementation Decisions

### PDF-Renderer (Q1)
- **`printpdf`** (pure Rust, low-level). Kein System-Chrome, deterministisches Layout, keine System-Deps im Nix-Build.
- Layout wird programmatisch gebaut (Header „Schichtplan KW N (JJJJ)", Tagesspalten Mo–So, Sales-Person-Zeilen mit Zeitfenstern).
- **Alle aktiven Sales-Persons** im rendered PDF enthalten (Erfüllung Success Criterion 1).

### WebDAV-Client (Q2)
- **`reqwest_dav`** — dünner Wrapper um reqwest, WebDAV-Basics (PUT, MKCOL). Basic-Auth mit User + App-Token.

### Konfiguration (Q3 + Q3a rev)
- **Alles in DB** in neuer Tabelle `pdf_export_config` (Single-Row-Pattern analog `paid_limit_config` / `holiday_stichtag_config` aus v1.6/v1.7).
- Felder:
  - `enabled BOOLEAN NOT NULL DEFAULT FALSE`
  - `nextcloud_url TEXT` (Basis-URL, z.B. `https://cloud.example.com/remote.php/dav/files/user`)
  - `webdav_user TEXT`
  - `webdav_app_token TEXT` (**Klartext**, bewusste Entscheidung — Q3a-i; DB ist auf Filesystem-Ebene geschützt, andere DAO-Felder sind auch Klartext)
  - `target_folder TEXT` (Zielordner unter dem User-Root, z.B. `Schichtplaene/`)
  - `weeks_horizon INTEGER NOT NULL DEFAULT 8` (Wochen-Horizont ab aktueller KW; Q6a-ii)
  - `cron_schedule TEXT NOT NULL DEFAULT '0 6 * * 1'` (Cron-Ausdruck, Default „Montags 06:00")
  - `last_success_at TIMESTAMP NULL` (Status für Admin-UI)
  - `last_error_at TIMESTAMP NULL`
  - `last_error_message TEXT NULL`
- **Kein Env-Var-Fallback** — Feature-Flag off bei leerer/unvollständiger Konfig.
- **Migration**: `pdf_export_config`-Tabelle + Seed-Row mit `enabled=false`.

### Cron / Scheduling (Q4)
- **`tokio-cron-scheduler`** Crate. Cron-Ausdruck aus DB, Reload bei Config-Update via Admin-UI (Restart-frei oder mit Restart-Hinweis — Claude's Discretion).

### Filename & Kollision (Q5)
- **Overwrite**, deterministisch: `schichtplan-{JJJJ}-KW{NN}.pdf` (z.B. `schichtplan-2026-KW27.pdf`, KW zwei-stellig zero-padded).
- Nextcloud versioniert clientseitig ohnehin.

### Wochen-Horizont (Q6 + Q6a-ii)
- **DB-Feld `weeks_horizon`, Default 8** — im Admin-UI editierbar (Integer, sinnvolle Range z.B. 1–52).
- **Semantik**: ab aktueller ISO-Woche + `weeks_horizon - 1` Folgewochen. Bei `weeks_horizon = 8` also die aktuelle KW + 7 folgende, insgesamt 8 PDFs pro Lauf.

### Retry / Fehler-Toleranz (Q7)
- **In-Run-Retry mit Exponential-Backoff**: 3 Versuche, Delays 2s → 4s → 8s.
- Nach 3 gescheiterten Versuchen: `last_error_at` + `last_error_message` in DB persistieren, Log-Warnung, Cron-Loop läuft unbeeinträchtigt weiter (nächster geplanter Trigger versucht wieder).
- Erfolg: `last_success_at` setzen, `last_error_*` clearen.

### Admin-UI-Platzierung (Q8)
- **Neuer Card/Tab in Settings-Seite** (`shifty-dioxus/src/page/settings.rs`), analog Special-Days-Card oder Paid-Limit-Card.
- Rolle: **admin-gated** (Feature-Flag `pdf_export` ist admin-Privilege, nicht shiftplanner).
- UI-Elemente:
  - Toggle „PDF-Export aktiviert" (bindet an `enabled`)
  - Text-Inputs: URL, User, App-Token, Zielordner
  - Integer-Input: Wochen-Horizont (Default 8)
  - Text-Input: Cron-Ausdruck (Default „0 6 * * 1")
  - Save-Button
  - Read-only Status-Anzeige: „Letzter Erfolg: {timestamp}" · „Letzter Fehler: {timestamp} — {message}"

### Claude's Discretion
- Exakte Schema-Version-Prüfung: kein Snapshot-Bump nötig (RPT/EXP sind nicht Teil des `BillingPeriodValueType`-Sets — grep-verifizieren).
- REST-Endpoints für Admin-UI: `GET /pdf-export-config` + `PUT /pdf-export-config` (Body = alle editierbaren Felder). Response enthält Status-Felder (last_success_at / last_error_*).
- Basic Service oder Business-Logic-Service: `PdfExportConfigService` = Basic (nur DAO/Permission/Transaction); `PdfExportScheduler` (Cron-Task) = Business-Logic (konsumiert Config-Service + Shiftplan-View-Service + WebDAV-Client).
- i18n de/en/cs für die neue Settings-Card (Labels, Help-Texte, Status-Prefixes).
- Testing: Integration-Test mit einem in-memory- oder Mock-WebDAV-Server (`wiremock` oder ein leichter axum-Test-Server, der PUT anerkennt und Body verifiziert).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Single-Row-Config-Pattern: `paid_limit_config` (v1.6), `holiday_stichtag_config` (v1.7) — DAO + Service + REST + Admin-UI-Card. **Muster kopieren**.
- Admin-Privilegien-Gate: `PermissionService::is_admin` (Muster aus v1.6/v1.9 admin-gated Endpoints).
- Shiftplan-View: existierender Read-Aggregat-Service liefert die Wochen-Daten pro KW.
- i18n-Pattern: `shifty-dioxus/src/i18n/{en,de,cs}.rs` + Keys in `mod.rs::Key`-Enum.
- Settings-Card-Struktur: `shifty-dioxus/src/page/settings.rs` (Special-Days-Card / Paid-Limit-Card / Holiday-Stichtag-Card).

### Established Patterns
- Basic-Service-Tier: nur DAO/Permission/Transaction als Dependencies (kein Domain-Service). `PdfExportConfigService` = Basic.
- Business-Logic-Service-Tier: konsumiert andere Services. `PdfExportScheduler` = Business-Logic.
- DI-Wiring in `shifty_bin/src/main.rs`: erst Basic, dann Business-Logic (deterministische Reihenfolge).
- REST-Handler: Axum + `#[utoipa::path]` + DTO in `rest-types` mit `ToSchema`.
- Transaction-Muster: `service_impl` verwenden `Option<Transaction>`, `transaction_dao.use_transaction(tx).await?`, dann `.commit(tx).await?`.
- Migrations: `migrations/sqlite/`-Sequenz mit klarer up-Migration.
- Tests: `service_impl/src/test/` Integration mit in-memory SQLite.

### Integration Points
- **Neue Migration**: `migrations/sqlite/NNNN_pdf_export_config.sql` (nächste freie NNNN).
- **Neue DAO**: `dao_impl_sqlite/src/pdf_export_config.rs` + Trait in `dao/`.
- **Neue Services**: `service_impl/src/pdf_export_config.rs` (Basic), `service_impl/src/pdf_export_scheduler.rs` (Business-Logic), `service_impl/src/pdf_render.rs` (pure Rendering-Modul), `service_impl/src/webdav_client.rs` (pure Client-Wrapper).
- **Neue REST-Endpoints**: `rest/src/pdf_export_config.rs` (GET/PUT + `#[utoipa::path]`).
- **Neue DTO**: `rest-types/src/pdf_export_config.rs`.
- **Scheduler-Start**: in `shifty_bin/src/main.rs` beim App-Boot — konditional auf `enabled=true`.
- **Admin-UI-Card**: `shifty-dioxus/src/page/settings.rs` (neuer Card-Block, admin-gated).
- **Wochen-Daten**: Wiederverwendung Shiftplan-View-Service (bestehender Read-Aggregat).
- **Neue Dependencies** (Cargo.toml, backend): `printpdf`, `reqwest_dav`, `tokio-cron-scheduler`, plus test-only `wiremock` (falls für Mock-WebDAV genutzt).

</code_context>

<specifics>
## Specific Ideas

- **PDF-Layout** (Claude's Discretion für v1): einfaches Header + tabellarisches Layout, Landscape A4, ausreichend Platz für 7 Tagesspalten + Sales-Person-Zeilen. Fokus: lesbar, deterministisch, keine Fonts vom System (embedded font einbinden oder printpdf-Default).
- **App-Token in Admin-UI**: Text-Input mit `type=password` (Frontend-side masking); DB-Speicher Klartext. Admin sieht bei Bearbeitung leeres Feld mit Placeholder „(unverändert, hier neues Token eintippen)"; leer speichern lässt den existierenden Wert stehen.
- **Erst-Setup-Flow**: Admin öffnet Settings-Card → Toggle ist off → Feld füllt aus → Save → Toggle on → Scheduler übernimmt neue Config.
- **Erste manuelle Test-Ausführung**: „Jetzt exportieren"-Button in der Admin-Card (löst einmalige asynchrone Ausführung außerhalb des Cron aus). **Optional**, gerne in Plan 48-04 platziert; wenn zu viel, deferred.
- **ROADMAP-Success-Criterion 2 wird bewusst ersetzt** — siehe „ROADMAP-Anpassung" unter Deferred.

</specifics>

<deferred>
## Deferred Ideas

- **ROADMAP-Anpassung** (nicht deferred, sondern **eine begleitende Aktion**): ROADMAP Phase 48 Success Criterion 2 wird umformuliert von „Konfiguration über Env-Variablen … kein Klartext-Passwort im Repo/DB" zu „Konfiguration über die neue Admin-UI-Card + DB-Persistenz; App-Token Klartext in DB (bewusste Ops-Entscheidung, DB-File auf Filesystem-Ebene geschützt)".
- **Encrypted-at-rest für Token**: nicht in v2.2. Wenn später gewünscht, separater Follow-up mit Master-Key-Env-Var + AES-GCM.
- **Multi-Nextcloud-Ziele / Multi-Cron**: aktuell Single-Row-Config; Multi-Instance kommt (wenn) später mit eigenem Schema.
- **PDF-Layout-Feintuning** (Farben, Icons, Corporate-Design): erst nach v1-Deployment.
- **Retry-Persistenz über Restart**: aktuell in-run-only; nach Prozess-Restart läuft der nächste geplante Cron-Slot.

</deferred>
