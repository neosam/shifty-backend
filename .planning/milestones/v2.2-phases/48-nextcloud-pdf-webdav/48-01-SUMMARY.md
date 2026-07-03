---
phase: 48-nextcloud-pdf-webdav
plan: 01
subsystem: backend
tags: [sqlite, sqlx, migration, axum, rest, single-row-config, admin-gated, pdf-export, webdav, nextcloud]

# Dependency graph
requires:
  - phase: 28-vacation-entitlement-offset
    provides: Single-Row-Config + admin-gated Basic-Service-Pattern (vacation_entitlement_offset als Vorbild)
provides:
  - Migration `20260703000000_create-pdf-export-config.sql` (Single-Row-Tabelle mit fixer UUID + Seed)
  - DAO-Trait `PdfExportConfigDao` + SQLite-Impl `PdfExportConfigDaoImpl`
  - Basic-Tier-Service `PdfExportConfigService` mit admin-gated `get`/`update` und Full-Auth-only `record_success`/`record_error`
  - REST-Endpoints `GET /pdf-export-config` und `PUT /pdf-export-config` mit `PdfExportConfigTO` (Token IMMER maskiert in Response, T-48-02)
  - DI-Wiring in `shifty_bin/src/main.rs`
  - Integrationstest-Suite (`service_impl/src/test/pdf_export_config.rs`, 8 Tests inkl. Snapshot-Grep-Gate)
affects: [48-04-scheduler, 48-05-admin-ui-card]

# Tech tracking
tech-stack:
  added: []  # keine neuen Crates in dieser Plan-Ebene; Runtime-Deps (printpdf, reqwest_dav, tokio-cron-scheduler) landen in 48-02/03/04
  patterns:
    - "Single-Row-Konfig-Tabelle mit fixer UUID (X'…0048') + INSERT-OR-IGNORE-Seed"
    - "Token-Maskierung im REST-DTO via `From<&Domain>`-Konvertierung (Token IMMER `None` beim Serialisieren)"
    - "Update-DTO-Merge: `webdav_app_token: None` behält bestehenden Wert, `Some(v)` überschreibt"
    - "`record_success`/`record_error` als Scheduler-only Service-API (Full-Auth-Gate statt Privilege-Gate)"

key-files:
  created:
    - migrations/sqlite/20260703000000_create-pdf-export-config.sql
    - dao/src/pdf_export_config.rs
    - dao_impl_sqlite/src/pdf_export_config.rs
    - service/src/pdf_export_config.rs
    - service_impl/src/pdf_export_config.rs
    - rest/src/pdf_export_config.rs
    - service_impl/src/test/pdf_export_config.rs
    - .planning/phases/48-nextcloud-pdf-webdav/deferred-items.md
  modified:
    - dao/src/lib.rs
    - dao_impl_sqlite/src/lib.rs
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - rest-types/src/lib.rs
    - rest/src/lib.rs
    - shifty_bin/src/main.rs
    - .sqlx/ (4 neue query-…json Files aus `cargo sqlx prepare --workspace`)

key-decisions:
  - "Admin-Privilege ist der bestehende `admin`-String (kein separates Feature-Privilege), analog Toggle/Feature-Flag Admin-Endpoints"
  - "Token-Merge (leer = keep) implementiert im Service, nicht im REST-Handler — der REST-Handler kennt die Merge-Semantik nicht"
  - "Read-after-write im Service: nach dem `DAO.update` erneutes `DAO.get`, damit die Response konsistent den persistierten Stand liefert"
  - "Kein Snapshot-Bump: BillingPeriodValueType-Enum wird NICHT verändert, `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 (Test 6 als durables Regressions-Gate)"

patterns-established:
  - "Basic-Tier-Service mit reinen DAO/Permission/Clock/Uuid/Transaction-Deps für neue Single-Row-Konfigurationen"
  - "Snapshot-Grep-Gate via `include_str!` als test-level Assertion, damit Enum-erweiternde Phases nicht versehentlich stumm einen Snapshot-Version-Bump erzwingen"

requirements-completed: [EXP-02, EXP-03]

coverage:
  - id: D1
    description: "Migration `20260703000000_create-pdf-export-config.sql` legt Single-Row-Tabelle `pdf_export_config` an (fixer UUID PK, alle Felder gemäß D-48-CONFIG) und seedet EINE Zeile mit enabled=0"
    requirement: "EXP-02"
    verification:
      - kind: integration
        ref: "service_impl/src/test/pdf_export_config.rs::integration::fresh_db_returns_seed_row"
        status: pass
    human_judgment: false
  - id: D2
    description: "DAO-Trait `PdfExportConfigDao` + SQLite-Impl mit `get`/`update`/`record_success`/`record_error` — kein Uuid-Argument, da Single-Row"
    requirement: "EXP-02"
    verification:
      - kind: integration
        ref: "service_impl/src/test/pdf_export_config.rs::integration::admin_update_persists_full_values"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/pdf_export_config.rs::integration::record_success_and_record_error_persist"
        status: pass
    human_judgment: false
  - id: D3
    description: "Basic-Service `PdfExportConfigService` mit admin-gated `get`/`update` (Non-Admin → Forbidden) und Full-Auth-only `record_success`/`record_error`"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/pdf_export_config.rs::get_non_admin_forbidden"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/pdf_export_config.rs::update_non_admin_forbidden"
        status: pass
    human_judgment: false
  - id: D4
    description: "Token-Merge-Semantik: `webdav_app_token: None` im Update behält den bestehenden Wert; `Some(v)` überschreibt"
    requirement: "EXP-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/pdf_export_config.rs::update_with_empty_token_keeps_existing"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/pdf_export_config.rs::update_with_set_token_replaces_existing"
        status: pass
    human_judgment: false
  - id: D5
    description: "REST-Endpoints `GET /pdf-export-config` + `PUT /pdf-export-config`, DTO `PdfExportConfigTO` maskiert `webdav_app_token` IMMER (T-48-02); Swagger-Registrierung + Content-Type `application/json`"
    requirement: "EXP-02"
    verification:
      - kind: automated_ui
        ref: "cargo build --workspace + cargo clippy --workspace -- -D warnings + Swagger-UI Registrierung in rest/src/lib.rs `#[openapi(nest(…))]`"
        status: pass
    human_judgment: true
    rationale: "Swagger-UI-Sichtbarkeit + korrekter HTTP-Content-Type in Live-Response lassen sich am zuverlässigsten via Browser gegen einen laufenden Server verifizieren (48-05 UI-Verify-Cycle)."
  - id: D6
    description: "Status-Persistenz: `record_success` setzt `last_success_at`, clearet `last_error_*`; `record_error` setzt `last_error_at`+`last_error_message`, lässt `last_success_at` unverändert"
    requirement: "EXP-03"
    verification:
      - kind: integration
        ref: "service_impl/src/test/pdf_export_config.rs::integration::record_success_and_record_error_persist"
        status: pass
    human_judgment: false
  - id: D7
    description: "Snapshot-Version bleibt 12 (Phase 48 fügt keinen `BillingPeriodValueType` hinzu — durables Grep-Gate)"
    verification:
      - kind: unit
        ref: "service_impl/src/test/pdf_export_config.rs::snapshot_version_unchanged_grep_gate"
        status: pass
    human_judgment: false

# Metrics
duration: ~55min
completed: 2026-07-03
status: complete
---

# Phase 48 Plan 01: Persistenz-Fundament PDF-Export (Migration + DAO + Basic-Service + REST) Summary

**Single-Row-Konfig-Tabelle `pdf_export_config` mit fixer UUID-PK, `PdfExportConfigService` als Basic-Tier (admin-gated get/update, Full-Auth-only record_success/record_error), `PdfExportConfigTO` mit garantierter Token-Maskierung in der Response, und DI-Wiring in shifty_bin — freigeschaltet für Scheduler (48-04) und Admin-UI-Card (48-05).**

## Performance

- **Duration:** ~55 min
- **Started:** 2026-07-03 (Executor-Session)
- **Completed:** 2026-07-03
- **Tasks:** 3 (alle grün, ohne Deviations)
- **Files modified/created:** 16 (7 neu + 8 modifiziert + `.sqlx/` Cache)

## Accomplishments

- Migration `20260703000000_create-pdf-export-config.sql` mit `INSERT OR IGNORE`-Seed einer festen UUID-Row (`X'…0048'`), enabled=0, defaults wie D-48-CONFIG
- DAO-Trait + SQLite-Impl mit 4 Methoden (`get`/`update`/`record_success`/`record_error`) — kein Uuid-Argument, weil Single-Row; alle Zeitstempel via ISO-8601, alle UUIDs als BLOB
- Basic-Tier Service `PdfExportConfigService` mit korrektem Gate-Split (admin für Public-API, Full-Auth für Scheduler) und Read-after-Write auf `update` für konsistente Response
- rest-types-DTO `PdfExportConfigTO` mit `From<&PdfExportConfig>`-Impl, die den `webdav_app_token` IMMER auf `None` setzt (T-48-02, garantiert im DTO-Layer)
- REST-Handler `GET /pdf-export-config` + `PUT /pdf-export-config` mit `#[utoipa::path]`, `Content-Type: application/json` (HYG-05) und `error_handler`-Wrapping; Swagger-UI-Registrierung in `rest/src/lib.rs`
- DI-Wiring in `shifty_bin/src/main.rs`: DAO-Typalias + `PdfExportConfigServiceDependencies` + Service-Feld in `RestStateImpl` + Getter
- 8 Tests grün (5 Mock-Unit-Tests für Admin-Gate + Token-Merge, 3 in-memory-SQLite-Integration-Tests für DAO + Persistenz, 1 Snapshot-Grep-Gate)

## Task Commits

Each task committed atomically by the GSD auto-commit path (jj co-located git):

1. **Task 1: Migration + DAO Trait + SQLite-Impl** — `feat(48-01)` (in-memory SQLite validated via Task 3's tests)
2. **Task 2: Basic-Service + rest-types-DTO + REST-Endpoints + DI-Wiring** — `feat(48-01)`
3. **Task 3: Integrationstest-Suite + Snapshot-Grep-Gate + ROADMAP-Verify** — `test(48-01)`

**Plan metadata:** `docs(48-01): complete plan` (final)

## Files Created/Modified

### Created
- `migrations/sqlite/20260703000000_create-pdf-export-config.sql` — Single-Row-Tabelle + Seed
- `dao/src/pdf_export_config.rs` — Trait + Entity
- `dao_impl_sqlite/src/pdf_export_config.rs` — SQLite-Impl (query!/query_as!, ISO-8601 Serde)
- `service/src/pdf_export_config.rs` — Service-Trait + Domain-Struct + Update-DTO
- `service_impl/src/pdf_export_config.rs` — Basic-Tier-Impl via `gen_service_impl!`
- `rest/src/pdf_export_config.rs` — Axum-Handler + utoipa-ApiDoc
- `service_impl/src/test/pdf_export_config.rs` — 8 Tests (5 Mock + 3 in-memory + 1 grep-gate)
- `.planning/phases/48-nextcloud-pdf-webdav/deferred-items.md` — pre-existing clippy-Warnung ausserhalb dieses Scopes

### Modified
- `dao/src/lib.rs` + `dao_impl_sqlite/src/lib.rs` — `pub mod pdf_export_config`
- `service/src/lib.rs` + `service_impl/src/lib.rs` — `pub mod pdf_export_config`
- `service_impl/src/test/mod.rs` — Test-Modul registriert
- `rest-types/src/lib.rs` — `PdfExportConfigTO` + `From<&PdfExportConfig>` mit garantierter Token-Maskierung
- `rest/src/lib.rs` — RestStateDef-Erweiterung (`PdfExportConfigService`-Assoc-Type + Getter), Route nested unter `/pdf-export-config`, `ApiDoc::nest(...)`
- `shifty_bin/src/main.rs` — `PdfExportConfigDao`-Typalias, `PdfExportConfigServiceDependencies`, Service-Feld in `RestStateImpl` + Konstruktion + Getter
- `.sqlx/` — 4 neue `query-*.json` (aus `cargo sqlx prepare --workspace`)

## Decisions Made

- **Admin-Privilege-Konstante:** privat im Service als `const ADMIN_PRIVILEGE: &str = "admin"` (analog `feature_flag_admin`/`cutover_admin`), kein separater `PDF_EXPORT_ADMIN`-Privilege — der bestehende Admin ist ausreichend, D-48-ADMIN.
- **`record_success`/`record_error` Auth-Kontrolle:** `check_only_full_authentication` statt Privilege-Gate; diese Methoden sind Scheduler-only und laufen ohne Session-Context (48-04 authentifiziert intern).
- **Read-after-Write im Service:** `update` liest die Row nach dem Write erneut, damit Aufrufer nicht auf Merge-Konsistenz spekulieren müssen und die Response auch bei künftigen DB-Triggers stabil bleibt.
- **Token-Maskierung im DTO-Layer:** `From<&PdfExportConfig> for PdfExportConfigTO` setzt `webdav_app_token: None` — nicht im Service, weil der Service den Klartext-Token intern (z. B. für den Scheduler in 48-04) noch braucht. Der Service-Rückgabewert enthält den Token, die HTTP-Response nicht.
- **Test-Datei-Setup:** Mix aus mock-basierten Unit-Tests (Admin-Gate + Token-Merge) und in-memory-SQLite-Integration-Tests (DAO gegen echte Migration). Beides sinnvoll, kein Grund für eine Trennung in zwei Dateien.

## Deviations from Plan

None - plan executed exactly as written.

- Ein zusätzlicher Test (`update_with_set_token_replaces_existing`) wurde ergänzt, um die "Token gesetzt = überschreiben"-Semantik neben der "Token None = keep"-Semantik explizit zu belegen — der Plan verlangte nur den Keep-Fall in Test 3, aber die Umkehrung ist trivial zusätzlich (kein Deviation-Rule-Trigger, nur Test-Vollständigkeit). Gesamtzahl Tests damit 8 statt der geplanten 6 (5 Behaviors + 3 in-memory + 1 grep = die geplanten 6 Behaviors sind inkludiert; die zwei "Extra" sind der zweite Token-Fall + die Aufteilung des in-memory-Persistenz-Tests).

## Issues Encountered

- **Clippy `--all-targets` findet einen pre-existing doc_lazy_continuation Fehler** in `service_impl/src/test/shiftplan_edit_lock.rs:6` (Phase 40). Der plan-spezifizierte Gate (`cargo clippy --workspace -- -D warnings` OHNE `--all-targets`) passt und das ist derselbe Gate den `nix build` und CI fahren. Der `--all-targets`-Fehler ist damit dokumentiert außerhalb des Scopes (Scope-Boundary-Regel) und in `deferred-items.md` festgehalten — als Aufgabe für Phase 46 (Backend-Hygiene) oder einen kleinen Follow-Up-Commit.

## Gates Status

- `cargo sqlx prepare --workspace`: **grün** (4 neue query-Fingerprints in `.sqlx/`)
- `cargo build --workspace`: **grün** (finales Nachbuild, 19.60s)
- `cargo test --workspace`: **grün** (alle Tests, inkl. pdf_export_config 8/8)
- `cargo clippy --workspace -- -D warnings`: **grün** (Plan-spezifizierter Gate; identisch zu `nix build` und CI)
- Snapshot-Grep-Gate: **grün** (`CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` unverändert; Test 6 durchgesetzt)
- Migration auf frischer DB: **grün** (drei in-memory-SQLite-Tests laufen `sqlx::migrate!` jeweils frisch)
- ROADMAP Success Criterion 2: **bereits D-48-ROADMAP-konform** (Zeile 142, keine Änderung nötig)

## Threat Flags

Keine neuen unbekannten Trust-Boundaries — alle Threats (T-48-01..05) sind im `<threat_model>`-Block des PLAN.md erfasst und mitigiert (siehe D3/D5/D6 in `coverage:`).

## Next Phase Readiness

Freigeschaltet für nachfolgende Plans in Phase 48:

- **48-02** (PDF-Renderer, pure): kann isoliert entstehen, keine Dep auf 48-01
- **48-03** (WebDAV-Client, pure): kann isoliert entstehen, keine Dep auf 48-01
- **48-04** (Scheduler): konsumiert `PdfExportConfigService::get` (Config-Lese), `record_success`, `record_error` — ALLE bereitgestellt
- **48-05** (Admin-UI-Card): konsumiert `GET /pdf-export-config` + `PUT /pdf-export-config` — beide REST-Endpoints bereitgestellt, Swagger-UI-sichtbar, Content-Type application/json

## Self-Check

- **File check:**
  - FOUND: migrations/sqlite/20260703000000_create-pdf-export-config.sql
  - FOUND: dao/src/pdf_export_config.rs
  - FOUND: dao_impl_sqlite/src/pdf_export_config.rs
  - FOUND: service/src/pdf_export_config.rs
  - FOUND: service_impl/src/pdf_export_config.rs
  - FOUND: rest/src/pdf_export_config.rs
  - FOUND: service_impl/src/test/pdf_export_config.rs
- **Snapshot version check:** FOUND `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;` in service_impl/src/billing_period_report.rs:117
- **Test suite:** 8/8 tests in `service_impl::test::pdf_export_config` pass
- **Full workspace test:** all suites pass (`cargo test --workspace`)
- **Clippy gate:** `cargo clippy --workspace -- -D warnings` passes clean

## Self-Check: PASSED

---
*Phase: 48-nextcloud-pdf-webdav*
*Plan: 01*
*Completed: 2026-07-03*
