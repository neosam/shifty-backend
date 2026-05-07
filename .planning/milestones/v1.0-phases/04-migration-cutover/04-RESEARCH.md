# Phase 4: Migration & Cutover - Research

**Researched:** 2026-05-03
**Domain:** Rust / Axum / SQLx — Atomic Multi-Service-Tx, Heuristik-Migration, OpenAPI-Snapshot-Locking
**Confidence:** HIGH (alle 8 Recherche-Schwerpunkte direkt im Repo-Code verifiziert; Insta + utoipa Defaults via Docs+crates.io API bestätigt; SQLite-Tx-Modus via SQLite-WAL-Doku verifiziert)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Migrations-Heuristik (Area A):**
- **D-Phase4-01:** Werktage = per-Vertrag aus `EmployeeWorkDetails.workdays` (Mo..So Bool-Maske, am Tag gültiger Vertrag).
- **D-Phase4-02:** Strict-Cluster-Heuristik: `extra_hours.amount == contract_hours_at(day)` sonst Quarantäne. Konsekutive Werktage gleicher (sp, kategorie) zu einer Range gemerged. Bruchstunden, Wochenend bei Mo-Fr, Vertragswechsel-mid-cluster mit unpassenden Stunden, ISO-53 → Quarantäne.
- **D-Phase4-03:** Quarantäne in eigener Tabelle `absence_migration_quarantine` (PK: `extra_hours_id`). `extra_hours` bleibt während MIG-01 unangetastet.
- **D-Phase4-04:** Idempotenz via Mapping-Tabelle `absence_period_migration_source` (PK: `extra_hours_id`). Re-Run skippt bereits gemappte Rows. **`extra_hours.id` ist der Idempotenz-Key (KEIN logical_id auf extra_hours).**

**Cutover-Gate-Mechanik (Area B):**
- **D-Phase4-05:** Gate-Granularität = pro `(sales_person_id, kategorie, jahr)`. Toleranz absolut < 0.01h.
- **D-Phase4-06:** Diff-Report = JSON-Datei in `.planning/migration-backup/cutover-gate-{ISO_TIMESTAMP_UTC}.json` + `tracing::error!` pro Drift-Zeile.
- **D-Phase4-07:** Zwei separate REST-Endpunkte: `POST /admin/cutover/gate-dry-run` (HR) + `POST /admin/cutover/commit` (`cutover_admin`).
- **D-Phase4-08:** Drift-Schutz = Commit fährt Migration + Gate erneut auf aktuellem extra_hours-State zum Tx-Zeitpunkt. Identischer Code-Pfad `CutoverService::run(dry_run: bool)`.

**REST-Strategie + Schicksal alter ExtraHours (Area C):**
- **D-Phase4-09:** `/extra-hours` POST-Block ist **flag-gated**. Vor Cutover: 100% unverändert. Nach Cutover: 403 `ExtraHoursCategoryDeprecated` für Vacation/SickLeave/UnpaidLeave. ExtraWork/Holiday/Custom/Volunteer immer akzeptiert. DELETE/GET unverändert.
- **D-Phase4-10:** Soft-Delete der migrierten extra_hours-Rows INNERHALB der atomaren Cutover-Tx. `update_process = 'phase-4-cutover-migration'`. Quarantänierte Rows bleiben aktiv. Reverse-Migration via `update_process`-Suche möglich.
- **D-Phase4-11:** OpenAPI-Snapshot-Test via `insta` (SC-6). Neue Test-Datei `rest/tests/openapi_snapshot.rs` + dev-dependency `insta = { version = "1", features = ["json"] }` in `rest/Cargo.toml`. Updates per `cargo insta review` (Mensch bestätigt, dann jj-Commit). Snapshot lockt Vertrag, NICHT Flag-State.

**Carryover-Refresh + Atomic-Tx-Boundaries (Area D):**
- **D-Phase4-12:** Carryover-Refresh-Scope = alle `(sp, year)`-Tupel mit non-zero Vacation/Sick/UnpaidLeave-Stunden im Gate. FeatureFlag ist innerhalb der Tx schon `true` → ReportingService liest automatisch über `derive_hours_for_range`.
- **D-Phase4-13:** Pre-Cutover-Backup via separate Tabelle `employee_yearly_carryover_pre_cutover_backup`. Schema = employee_yearly_carryover-Spalten + `cutover_run_id` BLOB(16) NOT NULL + `backed_up_at` TEXT NOT NULL. PK auf `(cutover_run_id, sales_person_id, year)`. INSERT INTO ... SELECT * vor UPDATE.
- **D-Phase4-14:** **Eine einzige SQLite-Tx über alles** via `TransactionDao::use_transaction`. Atomar by definition. Alle inneren Service-Calls bekommen die Tx als `Some(tx)` und committen NICHT.

**Hygiene (Phase-Carry-Over):**
- **D-Phase4-15:** Wave-0 Mini-Plan: `dao/Cargo.toml` + `dao_impl_sqlite/Cargo.toml` `features = ["v4"]` ergänzen. Plus deferred-items.md neu für Phase-4 mit `localdb.sqlite3`-Drift-Hinweis.

### Claude's Discretion

- **C-Phase4-01:** Migrations-Datei-Anzahl + Reihenfolge. Vorgabe: 4 separate Files (`<TS>_create-absence-migration-quarantine.sql`, `<TS+1>_create-absence-period-migration-source.sql`, `<TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql`, `<TS+3>_add-cutover-admin-privilege.sql`).
- **C-Phase4-02:** `CarryoverService::rebuild_for_year` Surface. Vorgabe: neuer Helper auf `CarryoverService` (Cross-Entity → Service-Tier-Wechsel zu Business-Logic). Alternative: separater `CarryoverRebuildService`. **WICHTIG: explizit in Plan-Phase entscheiden.** Siehe Sektion #6 dieser Research für ein konkretes Cycle-Risiko.
- **C-Phase4-03:** Heuristik-Cluster-Algorithmus iterativ in Rust (einfacher zu testen, SQLx-portabel).
- **C-Phase4-04:** Soft-Delete-Modus für migrierte extra_hours: Vorgabe: neue `ExtraHoursService::soft_delete_bulk(ids, tx)`-Methode (saubere API).
- **C-Phase4-05:** Production-Data-Profile-Format-Detail. Vorgabe: pro `(sp, category, year)` Counts + Sum + Bruchstunden-Quote + Wochenend-Einträge-Count + ISO-53-Indicator. JSON konsistent mit Diff-Report-Format.
- **C-Phase4-06:** Migrations-Heuristik-Vertragslookup-Performance. Pre-fetch-Optimierung erlaubt (alle Verträge pro sp einmal laden, Map-Lookup pro Tag) — Vorgabe: erst messen, dann optimieren.
- **C-Phase4-07:** REST-Routen-Schnitt: neue Route-Gruppe `/admin/cutover/`.
- **C-Phase4-08:** Privileg-Surface: **neues** `cutover_admin`-Privileg (semantisch stärker als Reuse `feature_flag_admin`). Migration `<TS>_add-cutover-admin-privilege.sql` mit `INSERT INTO privilege (name, update_process) VALUES ('cutover_admin', 'phase-4-migration')`.

### Deferred Ideas (OUT OF SCOPE)

- REST-Endpunkt zum Auflisten der Quarantäne-Rows (`GET /admin/cutover/quarantine`) — Folgephase als HR-Admin-Surface.
- Auto-Cleanup der Quarantäne-Rows — manuell durch HR.
- Restore-Endpunkt aus `employee_yearly_carryover_pre_cutover_backup` — Folgephase.
- Frontend-Migration der `/extra-hours`-POST-Calls auf `/absence-period` — Frontend-Workstream.
- Bulk-Carryover-Rebuild-Endpoint — Folgephase.
- Read-Compat-Shim für `/extra-hours`-Vacation-POSTs nach Cutover — D-Phase4-09 wählt Hard-403.
- Audit-Trail für `feature_flag`-Flips — Phase-2 deferred, Phase-4 nicht.
- REST-Endpunkte für `feature_flag` mit OpenAPI — falls Frontend-Admin-Screen.
- Migration weiterer ExtraHours-Kategorien zu range-basiert — out of scope v1.
- CarryoverService-Tier-Wechsel als separater `CarryoverRebuildService` (Alternative zu C-Phase4-02 Vorgabe).
- Quarantäne-Reason-i18n — Frontend-Workstream.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MIG-01 | Heuristik-basierte Migration: konsekutive Werktage gleicher (sp, kategorie) mit `amount == contract_hours_at(day)` → eine `absence_period`. Mehrdeutige Bestände → `absence_migration_quarantine`. Re-Run idempotent über `absence_period_migration_source`. | Sektionen #3 (Cluster-Algorithmus, Pre-fetch) + Code-Examples Operation 1 (cluster sketch); reuse `EmployeeWorkDetails.has_day_of_week()` + `hours_per_day()` von Phase 2 |
| MIG-02 | Cutover-Gate pro `(sales_person_id, kategorie, jahr)`: `sum(extra_hours_legacy) ≈ sum(derive_hours_for_range)` mit |drift| < 0.01h. Single-Drift → Gate-Fail → Tx-Rollback. JSON-Diff-Report. | Sektion #2 (SQLite-Tx-Isolation) + Sektion #4 (Multi-Service-Tx-Pattern); reuse `AbsenceService::derive_hours_for_range` (Phase 2) als single source of truth |
| MIG-03 | Cutover-Surface: REST `POST /admin/cutover/gate-dry-run` (HR) + `POST /admin/cutover/commit` (`cutover_admin`). `CutoverService::run(dry_run: bool)` mit identischem Code-Pfad. utoipa-Annotations. | Sektion #5 (utoipa-Erweiterungen) + Sektion #4 (Multi-Service-Tx); inline DTOs `CutoverRunResultTO`, `CutoverGateDriftRowTO` analog Phase-3 WrapperDTO-Pattern in `rest-types/src/lib.rs` |
| MIG-04 | Atomarer Tx-Flip: `feature_flag.absence_range_source_active = 1` + Carryover-Refresh + Pre-Backup + Soft-Delete legacy-Rows + alle Migrations-Inserts in **einer** SQLite-Tx. Schlägt ein Schritt fehl → komplette Rollback. | Sektion #2 (SQLite + WAL nicht aktiv → Default-DEFERRED-Tx + Reader-Snapshot-Isolation) + Sektion #4 (Tx-Forwarding-Pattern) + Sektion #6 (Carryover-Service-Tier) |
| MIG-05 | REST-Deprecation: `/extra-hours` POST mit Vacation/Sick/UnpaidLeave gibt nach Cutover 403. ServiceError-Variante `ExtraHoursCategoryDeprecated { category }`. OpenAPI-Snapshot-Test via insta lockt API-Surface. | Sektion #1 (Insta + utoipa-Determinismus, sort_maps) + Sektion #5 (error_handler-Erweiterung); reuse Reporting-Switch-Pattern aus `service_impl/src/reporting.rs:475` |

</phase_requirements>

## Summary

Phase 4 ist eine **operations-getriggerte atomare Cutover-Phase mit hoher Sub-System-Kopplung**: ein neuer `CutoverService` (Business-Logic-Tier) hält eine einzige SQLite-Tx über fünf Operationen (Migration-Inserts → Gate-Berechnung → Soft-Delete legacy → Carryover-Backup+Refresh → Feature-Flag-Flip). Alle Sub-Service-Aufrufe bekommen `Some(tx)` und committen NICHT — Atomarität ergibt sich automatisch aus dem etablierten `TransactionDao::use_transaction`-Pattern (verifiziert in `dao_impl_sqlite/src/lib.rs:303-338` + `service_impl/src/absence.rs` als ≥3-Sub-Service-Präzedenz).

Drei Architektur-Entscheidungen brauchen explizite Plan-Phase-Aufmerksamkeit:

1. **Carryover-Refresh-Tier-Konflikt (kritisch).** `CarryoverService` ist heute Basic (`service_impl/src/carryover.rs`: nur DAO+Tx). `ReportingService` (Business-Logic) konsumiert bereits `CarryoverService` (`service_impl/src/reporting.rs:66`). Würde C-Phase4-02-Vorgabe (Helper `CarryoverService::rebuild_for_year` der `ReportingService` aufruft) umgesetzt, **entstünde Cycle Reporting → Carryover → Reporting**. Plan-Phase MUSS einen der zwei tier-konformen Pfade wählen: (a) neuer `CarryoverRebuildService` (Business-Logic, konsumiert ReportingService + CarryoverService einseitig), oder (b) Inline-Refresh im `CutoverService` mit direktem ReportingService-Call. Empfehlung: (a) — sauberer, reuseable, klare Trennung.

2. **`/extra-hours`-POST Flag-Check Lokation.** D-Phase4-09 lässt Service-Layer vs REST-Handler offen. Der etablierte Phase-2-Switch lebt in `service_impl/src/reporting.rs:475` (Service-Layer). Empfehlung: Service-Layer-Check in `service_impl/src/extra_hours.rs::create` — analog Phase 2, bessere Test-Isolation, einheitliche `ServiceError::ExtraHoursCategoryDeprecated`-Quelle für alle Caller (REST + ggf. interne Service-Calls).

3. **OpenAPI-Snapshot-Stabilität.** utoipa 5 ist deterministisch by default: `Components`-Felder (schemas/responses/security_schemes) sind `BTreeMap` (alphabetisch); Path-Order ist alphabetisch sofern `preserve_path_order`-Feature nicht aktiviert ist (im Repo NICHT aktiviert — `rest/Cargo.toml:49-50` zeigt `utoipa = "5"` ohne Features). Plus `insta::Settings::set_sort_maps(true)` als Belt-and-Suspenders gegen versehentlich neu eingeführte HashMaps. Snapshot-File-Ablage: `rest/tests/snapshots/openapi_snapshot__<test_fn_name>.snap` (insta-Default für Tests in `tests/`-Ordner).

**Primary recommendation:** Plan-Phase folgt der Wave-Sequenz aus CONTEXT.md (Wave 0 → Wave 1 → Wave 2 → Wave 3) mit folgender Sequencing-Korrektur:
- **Wave 0 (Hygiene + Migrations-DDL + Insta-Setup)** — ZUERST `dao/Cargo.toml` + `dao_impl_sqlite/Cargo.toml` `features = ["v4"]`-Patch (D-Phase4-15), DANN die 4 Migration-SQL-Files (C-Phase4-01), DANN `insta`-dev-dep + leere `rest/tests/openapi_snapshot.rs`. Snapshot-File darf NOCH NICHT existieren — wird in Wave 2 zusammen mit den utoipa-Path-Ergänzungen über `cargo insta accept` finalisiert.
- **Wave 1 (CutoverService Skeleton + Heuristik + Quarantäne-Logik)** — neuer `service/src/cutover.rs` (Trait + DTOs `CutoverRunResult`), `service_impl/src/cutover.rs` (Impl mit `gen_service_impl!`-Macro, `run(dry_run, …)`-Method), neuer `dao/src/cutover.rs` (Read-only DAO für `find_all_legacy_extra_hours_for_migration` + Migration-Tabellen). Plus `CarryoverRebuildService` (Empfehlung 1 oben). Plus `ExtraHoursService::soft_delete_bulk` (C-Phase4-04). Tests: Unit-Mocks für Heuristik-Cluster + Quarantäne.
- **Wave 2 (Gate-Computation + REST + utoipa + Snapshot-Accept)** — `CutoverService::run` ergänzt um Gate-Berechnung + Diff-Report-File-IO + Soft-Delete + Carryover-Refresh + Flag-Set. REST-Layer: neue `rest/src/cutover.rs` mit 2 Handlers + `CutoverApiDoc`, ApiDoc-Nest in `rest/src/lib.rs:483`. Inline DTOs in `rest-types/src/lib.rs`. ServiceError-Variante + error_handler-Mapping → 403. `cargo insta accept` für `openapi_snapshot__*.snap`-File (Mensch reviewt + committet mit jj). MIG-05 Service-Layer-Check.
- **Wave 3 (E2E-Tests + Profile + Pflicht-Tests)** — `shifty_bin/src/integration_test/cutover.rs` mit allen Pflicht-Tests (forbidden, dry-run, commit, rollback, idempotence, SC-5-invariant, REST gate-dry-run/commit/extra_hours-pre-cutover/post-cutover). `CutoverService::profile()`-Method (SC-1, separat von `run()`).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cutover-Orchestrierung (Migration + Gate + Refresh + Flip in einer Tx) | Business-Logic Service (`CutoverService`) | — | Cross-Entity über 5+ Aggregate; klassische BL-Surface; konsumiert AbsenceService + ExtraHoursService + CarryoverRebuildService + FeatureFlagService + EmployeeWorkDetailsService einseitig |
| Heuristik-Cluster-Algorithmus | Business-Logic Service (`CutoverService` private Helper) | — | Domain-Logik, keine eigenständige Surface; Cluster-Funktion sieht alle 3 Kategorien gleichzeitig |
| Carryover-Rebuild-pro-(sp, year) | Business-Logic Service (NEU `CarryoverRebuildService`) | — | Konsumiert `ReportingService` (Read) + `CarryoverService` (Write); Cycle-Auflösung gegen aktuelle Reporting→Carryover-Direction |
| Cutover-Gate-Berechnung (Sum-Vergleich) | Business-Logic Service (`CutoverService` private Helper) | AbsenceService::derive_hours_for_range (Read) | Single source of truth aus Phase 2 wird 1:1 wiederverwendet |
| Diff-Report-Persistenz | File-IO im `CutoverService` | — | Sync-Write nach `.planning/migration-backup/`; KEIN DAO, KEIN DB-Trace |
| Soft-Delete legacy extra_hours-Rows | Basic Service (`ExtraHoursService::soft_delete_bulk`) | DAO | Bulk-Schreib-Operation auf eigenem Aggregat — tier-konform |
| Pre-Cutover-Carryover-Backup | DAO-Direct INSERT INTO ... SELECT | — | Schema-isomorph zu employee_yearly_carryover; eigene Tabelle ohne Service-Surface (write-once Audit) |
| Feature-Flag-Set | Basic Service (`FeatureFlagService::set`) | DAO | Bestehende Phase-2-API; im Cutover-Tx-Kontext ohne Permission-Check (`Authentication::Full`-Bypass) |
| REST-Surface (`/admin/cutover/*`) | REST-Layer | utoipa-DTO-Schemas in `rest-types` | Standard-Repo-Pattern; 2 Handler + 1 ApiDoc-Block |
| OpenAPI-Snapshot-Lock | Integration-Test in `rest/tests/` | insta-Crate | Test-only, kein Runtime-Code |
| `/extra-hours`-POST Flag-Gate | Basic Service (`ExtraHoursServiceImpl::create` Pre-Check) | FeatureFlagService::is_enabled | Service-Layer analog Phase-2-Reporting-Switch (`reporting.rs:475`) — bessere Test-Isolation als REST-Layer-Check |

## Standard Stack

### Core (bereits im Repo)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | 0.8.2 (sqlite, runtime-tokio) | DB-Layer + Tx-Management | Compile-time-checked queries; `pool.begin()` liefert TransactionImpl |
| `axum` | 0.8.7 | HTTP-Framework | Repo-Standard |
| `utoipa` | 5 | OpenAPI-Schema-Generierung | Repo-Standard; Components/Schemas → BTreeMap (alphabetisch deterministisch) |
| `tokio` | 1.48 (full) | Async-Runtime | Repo-Standard |
| `tracing` | 0.1.41 | Strukturiertes Logging | bereits für `tracing::error!`/`info!` im Repo verwendet |
| `time` | 0.3.36 | Date/PrimitiveDateTime | Repo-Standard |
| `uuid` | 1.8.0 (v4, serde) | UUIDs | bereits im rest-Crate; Phase-4 Wave-0 ergänzt v4 in dao+dao_impl_sqlite |
| `serde_json` | 1.0.145 | JSON-Persistenz für Diff-Report | Repo-Standard |

### Supporting (NEU für Phase 4)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `insta` | 1.47.2 (json) | OpenAPI-Snapshot-Test | dev-dependency in `rest/Cargo.toml`; Pin-File `rest/tests/snapshots/openapi_snapshot__<test_fn>.snap` |

**Version verification:**
- `insta`: latest stable `1.47.2` (Updated 2026-03-30, verified via `crates.io/api/v1/crates/insta`). `features = ["json"]` (verified via API features endpoint: `json: ['serde']`). [VERIFIED: crates.io API]

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `insta::assert_json_snapshot` | `assert_yaml_snapshot` (default insta surface) | YAML kompakter als JSON für große OpenAPI-Specs, ABER D-Phase4-11 fix für JSON. JSON ist auch das native OpenAPI-Format — Diff-Reading ist intuitiver für API-Reviewer. Behalten. |
| neuer `CarryoverRebuildService` | Inline-Refresh im `CutoverService` mit direktem `reporting_service`-Call | Inline ist schmaler, aber `CutoverService` hält dann 6 statt 5 Sub-Services. Separater Service ist Reuseable für spätere Bulk-Rebuild-Folgephase (Deferred). Empfehlung: **separater Service** — Reuse-Wert > Surface-Slim-Wert. |
| neue `find_all_legacy_extra_hours` DAO-Methode | per-sales-person-Loop mit existing `find_by_sales_person_id_and_years` | per-sp-Loop ist N-Service-Calls; bei N=200 sales_persons × M Jahre = O(NM) DAO-Calls. Eine `find_all_for_categories(categories, tx)`-DAO-Method ist O(1) Query mit IN-Clause. Empfehlung: neue DAO-Method (Cluster-Algorithmus braucht ohnehin alle Rows global sortiert nach sp+category+date_time). |

**Installation:**
```toml
# rest/Cargo.toml — NEU dev-dependencies block
[dev-dependencies]
insta = { version = "1.47.2", features = ["json"] }

# dao/Cargo.toml + dao_impl_sqlite/Cargo.toml — Wave-0-Hygiene D-Phase4-15
uuid = { version = "1.8", features = ["v4"] }   # dao
uuid = { version = "1.8.0", features = ["v4"] } # dao_impl_sqlite (existing version-pin behält)
```

## Architecture Patterns

### System Architecture Diagram

```
                    ┌──────────────────────────────────────────────────────┐
                    │  HR Operator (REST Client)                           │
                    └────┬─────────────────────────────────────────────┬───┘
                         │ POST /admin/cutover/gate-dry-run            │ POST /admin/cutover/commit
                         │   (Permission: HR)                          │   (Permission: cutover_admin)
                         ▼                                             ▼
                    ┌────────────────────────────────────────────────────────┐
                    │  rest::cutover (Axum handlers + utoipa)                │
                    │  CutoverGateDryRunHandler, CutoverCommitHandler        │
                    │  → calls rest_state.cutover_service().run(...)         │
                    └────┬───────────────────────────────────────────────┬───┘
                         │ run(dry_run=true)                             │ run(dry_run=false)
                         ▼                                               ▼
                    ┌────────────────────────────────────────────────────────┐
                    │  service_impl::cutover::CutoverServiceImpl             │
                    │  (Business-Logic-Tier)                                 │
                    │                                                        │
                    │  1. permission_service.check_permission(HR or          │
                    │     CUTOVER_ADMIN, ctx)                                │
                    │  2. let tx = transaction_dao.use_transaction(None)     │
                    │  3. ── MIGRATION-PHASE ──                              │
                    │     a. read_legacy_extra_hours(tx)                     │
                    │     b. read_existing_mappings(tx) — Idempotenz-Skip    │
                    │     c. for each sp+category cluster:                   │
                    │          - lookup contract_hours_at(day) via           │
                    │            employee_work_details_service               │
                    │          - quarantine OR cluster-merge                 │
                    │     d. INSERT absence_period rows + mapping rows       │
                    │     e. INSERT quarantine rows                          │
                    │  4. ── GATE-PHASE ──                                   │
                    │     for each (sp, kat, year) in scope:                 │
                    │       legacy_sum  = sum extra_hours WHERE …            │
                    │       derived_sum = sum absence_service                │
                    │                       .derive_hours_for_range(…)       │
                    │       if |drift| > 0.01: gate_drift_rows.push(…)       │
                    │     write JSON diff report to                          │
                    │       .planning/migration-backup/cutover-gate-{ts}.json│
                    │     tracing::error! per drift row                      │
                    │  5. ── BRANCH ──                                       │
                    │     IF dry_run OR !gate_passed:                        │
                    │       transaction_dao.rollback(tx)                     │
                    │       return CutoverRunResult { gate_passed:false }    │
                    │     ELSE:                                              │
                    │  6. ── COMMIT-PHASE (only if gate_passed && !dry_run) ─│
                    │     a. INSERT employee_yearly_carryover_pre_cutover_   │
                    │          backup SELECT * FROM employee_yearly_         │
                    │          carryover WHERE (sp, year) IN scope_set       │
                    │     b. carryover_rebuild_service.rebuild_for_year(     │
                    │          sp, year, Authentication::Full, Some(tx))     │
                    │          for each scope tuple                          │
                    │     c. extra_hours_service.soft_delete_bulk(           │
                    │          migrated_ids, "phase-4-cutover-migration",    │
                    │          Authentication::Full, Some(tx))               │
                    │     d. feature_flag_service.set(                       │
                    │          "absence_range_source_active", true,          │
                    │          Authentication::Full, Some(tx))               │
                    │  7. transaction_dao.commit(tx)  ← ATOMIC FLIP          │
                    │  8. return CutoverRunResult { gate_passed:true,        │
                    │       migrated_clusters, quarantined_rows, ... }       │
                    └────┬───────────────────────────────────────────────┬───┘
                         │ Read-only-Reads                               │ Writes (within tx)
                         ▼                                               ▼
        ┌────────────────────────────┐           ┌─────────────────────────────────┐
        │ AbsenceService             │           │ ExtraHoursService.soft_delete   │
        │  .derive_hours_for_range   │           │   _bulk (NEW C-Phase4-04)       │
        │ EmployeeWorkDetailsService │           │ FeatureFlagService.set          │
        │  .find_by_sales_person_id  │           │ AbsenceDao.create               │
        │ ExtraHoursDao              │           │ MigrationSourceDao.upsert       │
        │  .find_all_for_categories  │           │ QuarantineDao.upsert            │
        │  (NEW)                     │           │ CarryoverRebuildService         │
        └─────────────┬──────────────┘           │  .rebuild_for_year (NEW)        │
                      │                          │   ↓                              │
                      ▼                          │   ReportingService              │
                ┌─────────────────┐              │     .get_report_for_employee_   │
                │ SQLite (sqlx)   │◄─────────────┤      range  (Flag is true       │
                │ ONE TX (DEFERRED│              │      already, reads via         │
                │  isolation)     │              │      derive_hours_for_range)    │
                │ • snapshot      │              └─────────────────────────────────┘
                │   isolation for │
                │   concurrent    │              ┌─────────────────────────────────┐
                │   readers (DELETE              │ rest::extra_hours POST handler  │
                │   journal mode) │              │  → ExtraHoursService.create     │
                └────────────┬────┘              │     pre-checks                  │
                             │ (other readers see│     feature_flag_service.is_    │
                             │  pre-cutover state│     enabled("absence_range_     │
                             │  during the tx,   │     source_active", ctx, tx)    │
                             │  see post-cutover │     → 403 ExtraHoursCategory    │
                             │  after commit —   │       Deprecated for Vac/Sick/  │
                             │  see Sektion #2)  │       UnpaidLeave (D-Phase4-09) │
                             │                   └─────────────────────────────────┘
                             ▼
                    ┌───────────────────────────────┐
                    │ Test Surface                  │
                    │ rest/tests/openapi_snapshot.rs│ → insta::assert_json_snapshot!(ApiDoc::openapi())
                    │ shifty_bin/src/integration_   │ → CutoverService E2E (dry-run, commit, rollback,
                    │ test/cutover.rs               │   idempotence, SC-5-invariant)
                    └───────────────────────────────┘
```

### Recommended Project Structure

```
shifty-backend/
├── dao/Cargo.toml                                       # ⊕ Wave 0: features=["v4"]
├── dao/src/cutover.rs                                   # ⊕ NEU Wave 1: Migration-Tabellen-Trait
│
├── dao_impl_sqlite/Cargo.toml                           # ⊕ Wave 0: features=["v4"]
├── dao_impl_sqlite/src/cutover.rs                       # ⊕ NEU Wave 1: Impl der Migration-DAOs
├── dao_impl_sqlite/src/lib.rs                           # patched: pub mod cutover;
│
├── service/src/cutover.rs                               # ⊕ NEU Wave 1: CutoverService Trait + DTOs
├── service/src/carryover_rebuild.rs                     # ⊕ NEU Wave 1: CarryoverRebuildService Trait
├── service/src/lib.rs                                   # patched: pub mod cutover; carryover_rebuild;
│                                                        #          + ExtraHoursCategoryDeprecated variant
├── service/src/extra_hours.rs                           # patched: + soft_delete_bulk method (C-Phase4-04)
│
├── service_impl/src/cutover.rs                          # ⊕ NEU Wave 1+2: Impl
├── service_impl/src/carryover_rebuild.rs                # ⊕ NEU Wave 1: Impl
├── service_impl/src/extra_hours.rs                      # patched: + soft_delete_bulk impl,
│                                                        #          + flag-gated create-pre-check
├── service_impl/src/test/cutover.rs                     # ⊕ NEU Wave 1+2: Service-Mock-Tests
│                                                        #   Heuristik, Cluster-Edge-Cases, Forbidden
├── service_impl/src/lib.rs                              # patched: mod cutover/carryover_rebuild
│
├── rest-types/src/lib.rs                                # patched: + CutoverRunResultTO,
│                                                        #            CutoverGateDriftRowTO,
│                                                        #            CutoverGateDriftReportTO,
│                                                        #            ExtraHoursCategoryDeprecatedErrorTO
│
├── rest/Cargo.toml                                      # ⊕ Wave 0: dev-dep insta + json
├── rest/src/cutover.rs                                  # ⊕ NEU Wave 2: 2 handlers + CutoverApiDoc
├── rest/src/extra_hours.rs                              # touched if Plan-Phase chooses REST-layer-check
├── rest/src/lib.rs                                      # patched: ApiDoc nest "/admin/cutover",
│                                                        #          router nest, error_handler
│                                                        #          + ExtraHoursCategoryDeprecated → 403
├── rest/tests/openapi_snapshot.rs                       # ⊕ NEU Wave 0 (skeleton) + Wave 2 (accept)
├── rest/tests/snapshots/                                # ⊕ NEU Wave 2: insta-Default-Verzeichnis
│   └── openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap
│
├── shifty_bin/src/main.rs                               # patched: CutoverServiceDependencies block,
│                                                        #          DI: ExtraHoursDao + 4 new daos
│                                                        #          + CutoverService construction
│                                                        #          (NACH AbsenceService + Reporting)
├── shifty_bin/src/integration_test/cutover.rs           # ⊕ NEU Wave 3: E2E-Tests
│
├── migrations/sqlite/                                   # ⊕ Wave 0: 4 neue Files (C-Phase4-01)
│   ├── <TS>_create-absence-migration-quarantine.sql
│   ├── <TS+1>_create-absence-period-migration-source.sql
│   ├── <TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql
│   └── <TS+3>_add-cutover-admin-privilege.sql
│
└── .planning/phases/04-migration-cutover/
    └── deferred-items.md                                # ⊕ Wave 0: localdb.sqlite3-Drift-Hinweis
```

### Pattern 1: Multi-Service-Tx-Atomicity (`use_transaction(Some(tx))`-Forwarding)

**What:** Eine Tx wird im Outer-Service via `transaction_dao.use_transaction(None)` geöffnet; alle Sub-Service-Calls bekommen `Some(tx.clone())` und committen NICHT — sie nutzen sie nur. Nur der Outer-Service ruft `transaction_dao.commit(tx)` (oder lässt sie via Drop rollback'n).

**When to use:** Sobald eine Operation über mehrere Aggregate atomar laufen muss. Phase 4 ist der größte Anwendungsfall: 5+ Sub-Operations in einer Tx.

**Example (REPO-präzedent in `service_impl/src/absence.rs:586-628`):**
```rust
// Source: /home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/absence.rs:585-628
// AbsenceService::create ruft drei verschiedene Sub-Services in derselben Tx
let bookings = self
    .booking_service
    .get_for_week(iso_week, iso_year as u32, Authentication::Full, tx.clone().into())
    .await?;
// ... in derselben Tx ...
let slot = self
    .slot_service
    .get_slot(&b.slot_id, Authentication::Full, tx.clone().into())
    .await?;
// ... und ...
let manual_all = self
    .sales_person_unavailable_service
    .get_all_for_sales_person(sales_person_id, Authentication::Full, tx.clone().into())
    .await?;
```

Die `Authentication::Full`-Bypass-Pattern für interne Service-zu-Service-Calls ist eingerichtet — das `FeatureFlagService::is_enabled` enthält dafür einen expliziten `Authentication::Full`-Bypass-Pfad in `service_impl/src/feature_flag.rs:31-41` (Phase-2-Plan-04 Auto-Fix).

### Pattern 2: REST-Wrapper-DTO-Inline-Pattern (Phase-3-präzedent)

**What:** Wrapper-Result-DTOs für neue REST-Endpunkte leben **inline in `rest-types/src/lib.rs`** (nicht in eigenen Modulen). `From<&ServiceType>` Impls sind `#[cfg(feature = "service-impl")]`-gated, damit das Crate auch ohne service-Dep gebaut werden kann.

**When to use:** Für jede neue REST-Surface mit Cross-Domain-Result-Strukturen. Phase 4 ergänzt: `CutoverRunResultTO`, `CutoverGateDriftRowTO`, `CutoverGateDriftReportTO`, `ExtraHoursCategoryDeprecatedErrorTO`.

**Example (REPO-präzedent in `rest-types/src/lib.rs:1779-1794`):**
```rust
// Source: /home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs:1779-1794
/// Wrapper für `POST /shiftplan-edit/booking` (BOOK-02 Reverse-Warning).
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BookingCreateResultTO {
    pub booking: BookingTO,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_edit::BookingCreateResult> for BookingCreateResultTO {
    fn from(r: &service::shiftplan_edit::BookingCreateResult) -> Self {
        Self {
            booking: BookingTO::from(&r.booking),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}
```

### Pattern 3: Permission-Branch-Innerhalb-`run`-Method (analog Phase 2 D-Phase2-07)

**What:** Wenn ein Service-Method dual-permission hat (HR vs neuere stärkere Privilege), Branch zur Auswahl des Privileg-Strings im Method-Body, dann ein einzelner `check_permission`-Call.

**When to use:** Phase 4 `CutoverService::run(dry_run, …)` mit HR (dry_run) vs `cutover_admin` (commit).

**Example (verbatim aus CONTEXT.md `<specifics>` — Plan-Phase übernimmt 1:1):**
```rust
async fn run(&self, dry_run: bool, ctx: Authentication<Context>, tx: Option<Tx>) -> Result<CutoverRunResult, ServiceError> {
    self.permission_service
        .check_permission(if dry_run { HR_PRIVILEGE } else { CUTOVER_ADMIN_PRIVILEGE }, ctx.clone())
        .await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ... Migration + Gate + (if !dry_run && gate_passed) Cleanup + Carryover + Flag-Flip ...
    if dry_run || !gate_passed {
        self.transaction_dao.rollback(tx).await?;
    } else {
        self.transaction_dao.commit(tx).await?;
    }
    Ok(result)
}
```

### Pattern 4: Insta-OpenAPI-Snapshot mit Sort-Maps-Belt-and-Suspenders

**What:** OpenAPI-Snapshot-Test in `rest/tests/openapi_snapshot.rs` schreibt mit `insta::assert_json_snapshot!` ein Pin-File in `rest/tests/snapshots/openapi_snapshot__<test_fn>.snap`. utoipa 5 Defaults sind bereits deterministisch (BTreeMap-backed Components, alphabetische Path-Sort sofern `preserve_path_order` nicht aktiviert ist). `set_sort_maps(true)` ist Belt-and-Suspenders für versehentliche HashMap-Felder in eigenen DTOs.

**When to use:** Eine Test-Datei, eine Test-Funktion, ein Pin-File. Updates per `cargo insta review`.

**Example (Plan-Phase übernimmt verbatim aus CONTEXT.md `<specifics>`):**
```rust
// Source: rest/tests/openapi_snapshot.rs (NEU)
use rest::ApiDoc;
use utoipa::OpenApi;

#[test]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
```

`sort_maps`-Setting ist via `with_settings!`-Macro setzbar (verified in [Settings docs](https://docs.rs/insta/latest/insta/struct.Settings.html)). Snapshot-File-Pfad-Default für Tests in `tests/`-Ordner: `tests/snapshots/<file_module>__<test_fn>.snap` (per Insta-Doku: "snapshots stored in the snapshots folder right next to the test file").

### Anti-Patterns to Avoid

- **CarryoverService::rebuild_for_year als Helper auf bestehendem CarryoverService:** Würde Cycle `Reporting → Carryover → Reporting` erzeugen (siehe Sektion #6). Stattdessen: separater `CarryoverRebuildService` (Business-Logic) ODER Inline-Logik im `CutoverService`.
- **Tx via separater `pool.begin()`-Call statt `TransactionDao::use_transaction(Some(tx))`:** Würde eine zweite parallele Tx öffnen → Lock-Conflict bei SQLite-Default-Mode. Alle Sub-Service-Calls MÜSSEN `Some(tx.clone())` bekommen.
- **`AbsenceService::create` für jeden gemergten Cluster aufrufen statt direkter DAO-Insert:** Würde Forward-Warning-Loop pro Cluster fahren (Phase-3-Wrapper) — sinnlos, weil Migration kein Booking-Konflikt produziert. Vorgabe (CONTEXT.md `<canonical_refs>` `absence_period`): direkter DAO-Insert via `AbsenceDao::create` mit `Authentication::Full`.
- **`ExtraHoursService::create` per-row-Soft-Delete der migrierten Rows:** O(N) statt O(1) Service-Calls. Stattdessen: neue `soft_delete_bulk(ids, …)`-Method (C-Phase4-04).
- **REST-Layer-Flag-Check in `rest/src/extra_hours.rs::create_handler`:** Bricht Service-Layer-Test-Isolation. Service-Layer-Check ist der Phase-2-Präzedenz-Pfad (`reporting.rs:475`). Empfehlung: in `service_impl/src/extra_hours.rs::create` direkt nach Permission-Check.
- **Snapshot-Update durch `INSTA_UPDATE=always` in CI:** würde stille Vertragsbrüche maskieren. CI muss `CI=true` setzen (insta-Default verhindert dann Auto-Update — `INSTA_UPDATE=auto` schreibt nur `.snap.new` wenn KEIN CI detektiert wird; mit `CI=true` werden gar keine Updates geschrieben). Updates LOKAL via `cargo insta review` (Mensch-Confirm + jj-Commit).
- **Quarantäne-Reason als untyped String:** wäre fehleranfällig + i18n-feindlich. Empfehlung: `enum QuarantineReason { AmountBelow, AmountAbove, ContractHoursZero, ContractNotActive, Iso53Gap }` + `as_str()` für Persistenz; finalisierte Liste aus CONTEXT.md `<specifics>` Quarantäne-Reason-Strings.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OpenAPI-Schema-Diff-Detection | Custom JSON-Diff-Logic auf `serde_json::Value` | `insta::assert_json_snapshot!` mit Pin-File | Battle-tested, integriert mit `cargo insta review`, korrekte Diff-Anzeige |
| Date-Iteration über Range | Eigene `from..=to`-Loop mit `Date::next_day()` | `shifty_utils::DateRange::iter_days()` (Phase 1 D-16) | Bereits etabliert; AbsenceService benutzt es in `derive_hours_for_range` |
| Per-Tag-Vertragsstunden-Lookup | Eigene Iteration durch `EmployeeWorkDetails`-Liste | `EmployeeWorkDetails.has_day_of_week(weekday)` + `.hours_per_day()` | Bereits in `service_impl/src/absence.rs:480-486` etabliert; identische Logik garantiert Gate-Identität |
| Cross-Category-Stunden-Resolver | Eigene Reimplementation für Vacation/Sick/UnpaidLeave-Konflikt | `AbsenceService::derive_hours_for_range` (Phase 2) | Single source of truth — Plan-Phase 4 Gate fährt EXAKT die Logik die nach dem Flip live ist |
| SQLite-Tx-Atomarität | Manuelle Save-Points / Multi-Statement-Wrapping | `TransactionDao::use_transaction(Some(tx))` + Sub-Service-Forwarding | etabliert; keine Sub-Service committet, nur Outer-Service |
| Diff-Report-Persistenz | DB-Tabelle `cutover_run_log` | File-IO `serde_json::to_writer(File::create(path))` | OUT OF SCOPE — D-Phase4-06 wählt JSON-Datei + tracing-Logs; DB-Audit deferred. CI-friendly, jj-committable |
| File-Path-Construction für ISO-Timestamp | Eigene Format-String-Logik | `time::PrimitiveDateTime::format(&time::format_description::well_known::Iso8601::DATE_TIME)` mit Replace `:`→`-` falls Filesystem-Restriction | Plus `.planning/migration-backup/` muss vor erstem Run existieren — Plan-Phase Wave-0 task |
| `Uuid`-Generation | Custom rand+format-Logik | `uuid::Uuid::new_v4()` (mit `v4`-Feature) | Wave-0 D-Phase4-15 ergänzt das Feature in dao+dao_impl_sqlite |

**Key insight:** Phase 4 ist eine **Pure-Composition-Phase** — fast alles ist bereits gebaut. Die Heuristik ist die einzige neue Domain-Logik. Alles andere ist Re-Use von Phase-1+2+3-Surfaces in einer atomaren Tx.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `extra_hours` Bestand (Vacation/SickLeave/UnpaidLeave) — wird heuristisch zu `absence_period` migriert + soft-deleted innerhalb der Cutover-Tx. `feature_flag` Tabelle hat `absence_range_source_active` Row (von Phase-2-Migration geseedet, `enabled=0`); Flip auf `1` in Cutover-Tx. `employee_yearly_carryover` wird selektiv überschrieben (nur Tupel im Gate-Scope) — Pre-Backup in Schwester-Tabelle. | Code: Migration-Heuristik + Gate + Tx-Flip. Daten-Migration: heuristisch erzeugt; Quarantäne für Mehrdeutige (manueller HR-Resolve danach). |
| Live service config | None — Cutover läuft im laufenden Server (D-Phase4-09 User-Korrektur "kein Bin-Restart"). Kein externes Service hat eine Cutover-Konfiguration. | None |
| OS-registered state | None — Backend ist ein einzelner Rust-Binary, kein systemd-Service-Rename, kein Cron, kein pm2. Cutover ist eine HTTP-Request-getriggerte Operation. | None |
| Secrets/env vars | None — keine neuen Secrets. `BASE_PATH` env var (existing, in `rest/src/lib.rs:513`) bleibt unverändert. SOPS/dotenv unangetastet. | None |
| Build artifacts / installed packages | `dao/Cargo.toml` + `dao_impl_sqlite/Cargo.toml` brauchen `uuid features=["v4"]` ergänzt (Wave-0 D-Phase4-15) — sonst standalone `cargo test -p dao` rot. Der workspace-Build ist heute grün, weil andere Crates `v4` transitiv via Feature-Unification aktivieren — pre-existing Phase-1-Drift, dokumentiert in `.planning/phases/03-.../deferred-items.md`. | Code-Edit (Cargo.toml-Patch in beiden Files); kein Reinstall nötig (Cargo löst neu auf bei nächstem Build). |

**Frontend-Workstream (separat — nicht Phase-4-Scope):** Nach Cutover wird das Frontend (`shifty-dioxus`) den `/extra-hours`-POST für die 3 Kategorien gegen 403 fahren — bis es seinerseits auf `/absence-period` migriert. Operations-Verantwortung: PR-Description-Hinweis + README-Update + Issue im Frontend-Repo (CONTEXT.md `<specifics>` Frontend-Migration-Hinweis).

## Common Pitfalls

### Pitfall 1: Cycle Reporting → Carryover → Reporting durch C-Phase4-02-Vorgabe

**What goes wrong:** CONTEXT.md C-Phase4-02 Vorgabe "neuer Helper auf bestehendem CarryoverService" würde — wenn der Helper `ReportingService` aufruft — einen Service-Cycle erzeugen, der mit der Service-Tier-Konvention bricht und in `gen_service_impl!`-DI mit `OnceLock`-Hacks oder Forward-Decl-Tricks gelöst werden müsste.

**Why it happens:** `ReportingService` (Business-Logic) konsumiert bereits `CarryoverService` (Basic, `service_impl/src/reporting.rs:66`). Würde `CarryoverService::rebuild_for_year` `ReportingService` aufrufen, ist die Kante umgekehrt — Cycle. Der `service_impl/src/carryover.rs`-File hat heute NUR `dao::carryover::CarryoverDao` + `TransactionDao` als Deps — Hinzufügen einer `ReportingService`-Dep ist genau das verbotene Cross-Tier-Coupling.

**How to avoid:** Plan-Phase MUSS einen der drei Pfade explizit wählen:
- (a) **Empfehlung:** neuer Service `CarryoverRebuildService` (Business-Logic-Tier) im neuen File `service/src/carryover_rebuild.rs` + `service_impl/src/carryover_rebuild.rs`. Konsumiert `ReportingService` (BL) + `CarryoverService` (Basic) einseitig. `CutoverService` hält dann `CarryoverRebuildService` (BL) als Dep (BL → BL ist tier-konform; einseitige Direction).
- (b) Inline-Refresh im `CutoverService` selbst (CutoverService ist BL und darf ReportingService konsumieren). Code-Duplikat-Risiko, aber schmaler. Empfehlung NEIN — schwerer testbar isoliert.
- (c) `CarryoverService` zu Business-Logic promoten (Tier-Wechsel) und `ReportingService` als Dep hinzufügen. Bricht den Tier-Snapshot massiv und ist semantisch schlecht (Carryover-Aggregat-Manager wird Cross-Entity-Service).

**Warning signs:** Wenn ein Plan-Task vorschlägt, `ReportingService` als neue Dep des `CarryoverService` einzuziehen — STOP, Service-Tier-Convention ist verletzt. Plan-Plan-Checker-Gate.

### Pitfall 2: Bit-Identitäts-Drift zwischen Gate und Live-Reporting nach Flip

**What goes wrong:** Gate sagt OK, aber nach Flag-Flip zeigt das Live-Reporting andere Werte → Cutover ist "successful" laut Gate, aber Production-Daten driften.

**Why it happens:** Wenn die Gate-Berechnung NICHT `AbsenceService::derive_hours_for_range` aufruft, sondern eine eigene Reimplementation (z.B. SQL-Window-Function), entkoppelt sich die Gate-Logik von der Live-Logik. Beim nächsten Phase-2-Refactor an `derive_hours_for_range` driften sie auseinander.

**How to avoid:** Gate-Berechnung MUSS `derive_hours_for_range` 1:1 aufrufen — keine Re-Implementation. Code-Beweis-Pattern: ein einziger gemeinsamer Code-Pfad. SC-5-Test (`shifty_bin/src/integration_test/cutover.rs::per_sales_person_per_year_invariant`) als Regressions-Lock.

**Warning signs:** Wenn ein Plan-Task einen "performance-optimierten Gate-Path" vorschlägt, der nicht `derive_hours_for_range` ruft — STOP. Performance-Optimierung gehört in `derive_hours_for_range` selbst (alle Caller profitieren).

### Pitfall 3: Quarantäne zu permissiv → Cutover unmöglich

**What goes wrong:** Heuristik ist zu großzügig (z.B. konvertiert 7-Tage-Bestände ohne Werktage-Check) — Gate fail't massiv in Production, Cutover nicht aufrufbar.

**Why it happens:** Die Strict-Cluster-Heuristik (D-Phase4-02: `amount == contract_hours_at(day)` exakt) ist konservativ by design. Pre-Phase-4 Versuche, "smart" zu sein (z.B. partial Cluster bei Wochenend-Lücke) erodieren die Identitäts-Garantie.

**How to avoid:** Plan-Phase fixt: Heuristik-Code akzeptiert NUR exakte Match. Jede Abweichung → Quarantäne mit klarem Reason-String. SC-1-Production-Data-Profile zeigt vor Cutover-Start, wie viel Quarantäne-Volumen entsteht — HR weiß was kommt.

**Warning signs:** Quote `quarantäne_count / total_extra_hours_count > 0.5` in Production-Profile → manuelle HR-Vorarbeit pflichtmäßig vor Cutover-Start.

### Pitfall 4: Flag-Race im Reporting während Cutover-Tx

**What goes wrong:** Ein Live-Reporting-Request (`feature_flag.is_enabled("absence_range_source_active")`) läuft, während die Cutover-Tx aktuell `UPDATE feature_flag SET enabled = 1` hält. Was sieht der Reader?

**Why it happens:** Das Repo benutzt **SQLite-Default journal_mode (DELETE) und BEGIN DEFERRED Tx-Mode** — verifiziert via `grep -rn "journal_mode\|PRAGMA"` (no hits) und `dao_impl_sqlite/src/lib.rs:316` (`pool.begin()` = DEFERRED). In SQLite mit DELETE-journal: ein konkurrierender Reader BLOCKIERT, sobald der Writer eine RESERVED-Lock hält (was beim ersten UPDATE passiert). Wenn der Reader vor der Cutover-Tx mit BEGIN startet, sieht er den **alten** Wert (snapshot isolation per file lock); wenn er während der Tx neu startet, **wartet** er bis die Cutover-Tx commit/rollback.

**How to avoid:**
- **Korrektes Verhalten ist garantiert:** SQLite-DELETE-Mode + DEFERRED-Tx liefert serialisierbare Isolation für konkurrierende Reader (sie sehen entweder pre- oder post-Cutover-State, niemals mixed). Plan-Phase muss keine zusätzliche Lock-Logik einbauen.
- **Cutover-Tx-Länge minimieren:** Bei N=10000+ Bestand-Rows könnte die Tx mehrere Sekunden halten — alle anderen Writer sind während dieser Zeit `SQLITE_BUSY`. Plan-Phase Wave-3 sollte einen Smoke-Test mit großer Fixture (z.B. 5000 extra_hours-Rows) durchführen, um Worst-Case-Tx-Länge zu messen.
- **Empfehlung Plan-Phase:** Cutover ist HR-getriggert und HR weiß "andere Schreib-Operationen kurz pausieren". Kein automatisches Maintenance-Window-Lock nötig. Pitfall-Test (Mock-Konkurrent-Reader) optional.
- **NICHT empfohlen:** WAL-Mode aktivieren ad-hoc für Phase 4. Das wäre eine Repo-weite Änderung mit eigenen Konsequenzen (separate WAL+SHM-Files, fsync-Verhalten etc.). Out of Scope.

**Warning signs:** Wenn ein Plan-Task `BEGIN IMMEDIATE` explizit vorschreibt — wahrscheinlich nicht nötig (DEFERRED reicht), aber harmlos. Wenn ein Plan-Task `PRAGMA journal_mode=WAL` setzen will — STOP, Repo-weiter Mode-Switch ist out of Phase-4-Scope.

### Pitfall 5: HR-Verwirrung über `GET /extra-hours` nach Cutover

**What goes wrong:** Migrierte Rows sind `deleted IS NOT NULL` → unsichtbar für GET. Quarantäne-Rows bleiben sichtbar. HR fragt: "Wo sind meine Vacation-Einträge geblieben?"

**Why it happens:** `extra_hours.find_*` benutzt `WHERE deleted IS NULL`-Konvention (Repo-weit). Reverse-Migration via `update_process = 'phase-4-cutover-migration'`-Suche ist möglich, aber nicht UI-exponiert.

**How to avoid:**
- Phase-4-Scope: NICHTS ändern an GET-Endpunkten (D-Phase4-09 sagt explizit "DELETE und GET bleiben unverändert").
- Operations-Briefing für HR (PR-Description, README-Update): "Nach Cutover sind migrierte Vacation/Sick/UnpaidLeave-Einträge in `/absence-period` zu finden, nicht mehr in `/extra-hours`."
- Folgephase: optionales `GET /admin/cutover/migrated-extra-hours` für Audit-Surface (deferred — CONTEXT.md `<deferred>`).

**Warning signs:** Wenn ein Plan-Task einen GET-Filter "show_migrated=true" einbauen will → STOP, deferred.

### Pitfall 6: Idempotenz vs. Quarantäne — Stale-Quarantäne-Rows

**What goes wrong:** HR löst eine Quarantäne-Row manuell auf (legt eine `/absence-period` an + soft-deleted die ursprüngliche `extra_hours`-Row). Beim nächsten Cutover-Re-Run hat `absence_migration_quarantine` einen Eintrag mit verschwundener `extra_hours_id` (soft-deleted, nicht mehr in der Read-Query sichtbar).

**Why it happens:** Quarantäne-Tabelle ist write-once mit FK-frei (per CONTEXT.md `<domain>` Tabellen-Schema; kein FK auf `extra_hours.id`). Re-Run-Idempotenz funktioniert über `absence_period_migration_source` (PK auf `extra_hours_id`), nicht über Quarantäne.

**How to avoid:** Plan-Phase entscheidet (per CONTEXT.md `<code_context>` Pitfall-Idempotenz-vs-Quarantäne): Vorgabe ist "erstes Re-Run löscht stale Quarantäne-Rows automatisch". Implementation: vor dem neuen Quarantäne-INSERT prüft Cutover-Service `WHERE NOT EXISTS (SELECT 1 FROM extra_hours WHERE id = quarantine.extra_hours_id AND deleted IS NULL)` und DELETE'd diese Quarantäne-Rows. Idempotent + selbstheilend.

**Warning signs:** Wenn ein Plan-Task die Quarantäne-Tabelle mit FK-CASCADE auf extra_hours-DELETE definieren will — STOP, soft-delete (deleted IS NOT NULL) kaskadiert nicht; manueller Cleanup-Pfad ist sauberer.

### Pitfall 7: OpenAPI-Snapshot wird Test-Noise statt Vertrags-Lock

**What goes wrong:** Jede Phase-Plan-Änderung (Field-Rename, neuer Endpoint, Status-Code-Wechsel) macht den Snapshot rot. Entwickler greifen reflexiv zu `cargo insta accept` ohne Review → der Snapshot verliert seinen Lock-Wert.

**Why it happens:** `cargo insta accept` ist trivial; der Mensch-Review-Schritt wird vergessen.

**How to avoid:**
- **Updates IMMER per `cargo insta review`** (nicht `accept`) — interaktiver Diff, Mensch-Confirm pro Snapshot.
- **CI-Mode:** `CI=true` env (in CI-Pipeline gesetzt) verhindert Auto-Update; `INSTA_UPDATE=no` als Belt-and-Suspenders. CI-Run rot bei Snapshot-Drift → Mensch muss lokal `review`.
- **PR-Description-Pflicht:** wenn der `.snap`-File im PR geändert ist, MUSS die PR-Description erklären welche API-Änderung das ist. (Convention nicht-tooling-enforceable, aber per Phase-4-Doku-Brief im SUMMARY.md festhalten.)

**Warning signs:** PR-Diff zeigt `.snap`-File-Änderung ohne entsprechende `service::*`/`rest::*`-Änderung — Plan-Phase oder Reviewer flaggt als Verdacht.

## Code Examples

Verifizierte Patterns aus dem Repo bzw. aus offiziellen Quellen:

### Operation 1: Heuristik-Cluster-Algorithmus mit Pre-fetched Verträgen (C-Phase4-03 + C-Phase4-06)

```rust
// Source: synthesized aus service_impl/src/absence.rs:418-510 (derive_hours_for_range Pre-fetch + Per-Tag-Logik)
// und CONTEXT.md <specifics> Cluster-Algorithmus-Skelett
async fn migrate_legacy_extra_hours_to_clusters(
    &self,
    cutover_run_id: Uuid,
    tx: <Deps as CutoverServiceDeps>::Transaction,
) -> Result<MigrationResult, ServiceError> {
    // 1. Read alle nicht-migrierten extra_hours für Vacation/Sick/UnpaidLeave global
    let all_legacy = self
        .cutover_dao
        .find_legacy_extra_hours_not_yet_migrated(tx.clone())
        .await?;
    // Sortiert by (sales_person_id, category, date_time) ASC

    // 2. Pre-fetch alle Verträge pro sp einmal (C-Phase4-06 Optimierung)
    let mut work_details_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = HashMap::new();
    for sp_id in all_legacy.iter().map(|eh| eh.sales_person_id).collect::<BTreeSet<_>>() {
        let wd = self
            .employee_work_details_service
            .find_by_sales_person_id(sp_id, Authentication::Full, Some(tx.clone()))
            .await?;
        work_details_by_sp.insert(sp_id, wd);
    }

    // 3. Cluster greedy
    let mut current_cluster: Vec<&ExtraHoursEntity> = Vec::new();
    let mut migrations: Vec<(AbsencePeriod, Vec<Uuid>)> = Vec::new();
    let mut quarantine: Vec<(Uuid, &'static str)> = Vec::new();

    for eh in all_legacy.iter() {
        let day = eh.date_time.date();
        let work_details = work_details_by_sp.get(&eh.sales_person_id).expect("pre-fetched");

        // Aktiven Vertrag am Tag finden (analog absence.rs:463-476)
        let active_contract = work_details.iter().find(|wh| {
            wh.deleted.is_none()
                && wh.from_date().map(|d| d.to_date() <= day).unwrap_or(false)
                && wh.to_date().map(|d| day <= d.to_date()).unwrap_or(false)
        });
        let Some(contract) = active_contract else {
            quarantine.push((eh.id, "contract_not_active_at_date"));
            current_cluster.clear();
            continue;
        };

        // Werktag check (D-Phase4-01)
        if !contract.has_day_of_week(day.weekday()) {
            quarantine.push((eh.id, "contract_hours_zero_for_day"));
            current_cluster.clear();
            continue;
        }

        // Strict-Match check (D-Phase4-02)
        let expected = contract.hours_per_day();
        if (eh.amount - expected).abs() > 0.001 {
            let reason = if eh.amount < expected { "amount_below_contract_hours" }
                         else { "amount_above_contract_hours" };
            quarantine.push((eh.id, reason));
            current_cluster.clear();
            continue;
        }

        // Cluster fortsetzen oder neu starten
        let extends_cluster = current_cluster.last().map_or(false, |last| {
            last.sales_person_id == eh.sales_person_id
                && last.category == eh.category
                && is_consecutive_workday(last.date_time.date(), day, contract)
        });
        if !extends_cluster && !current_cluster.is_empty() {
            migrations.push(close_cluster(&current_cluster));
            current_cluster.clear();
        }
        current_cluster.push(eh);
    }
    if !current_cluster.is_empty() {
        migrations.push(close_cluster(&current_cluster));
    }

    // 4. Persist: AbsencePeriod-Inserts + Mapping-Inserts + Quarantine-Inserts
    for (period, source_ids) in &migrations {
        self.absence_dao.create(period.try_into()?, "phase-4-cutover-migration", tx.clone()).await?;
        for src_id in source_ids {
            self.cutover_dao.upsert_migration_source(*src_id, period.id, cutover_run_id, tx.clone()).await?;
        }
    }
    for (eh_id, reason) in &quarantine {
        self.cutover_dao.upsert_quarantine(*eh_id, reason, cutover_run_id, tx.clone()).await?;
    }

    Ok(MigrationResult { migrations: migrations.len() as u32, quarantined: quarantine.len() as u32 })
}
```

### Operation 2: Gate-Berechnung mit derive_hours_for_range-Reuse

```rust
// Source: synthesized aus service_impl/src/reporting.rs:475-538 (Pattern für derive_hours_for_range-Aufruf)
// und CONTEXT.md D-Phase4-05 Gate-Granularität
async fn compute_gate(
    &self,
    cutover_run_id: Uuid,
    tx: <Deps as CutoverServiceDeps>::Transaction,
) -> Result<GateResult, ServiceError> {
    // Scope-Set: alle (sp, year) mit Vacation/Sick/UnpaidLeave-Einträgen
    let scope = self
        .cutover_dao
        .find_legacy_scope_set(tx.clone())
        .await?;
    // Returns Arc<[(Uuid /*sp*/, u32 /*year*/)]>

    let mut drift_rows: Vec<DriftRow> = Vec::new();

    for &(sp_id, year) in scope.iter() {
        // Per-Kategorie-Loop (jeweils eigene Drift-Row)
        for category in [AbsenceCategory::Vacation, AbsenceCategory::SickLeave, AbsenceCategory::UnpaidLeave] {
            let legacy_sum = self
                .cutover_dao
                .sum_legacy_extra_hours(sp_id, &category, year, tx.clone())
                .await?;

            // derive_hours_for_range fährt EXAKT die Live-Logik (D-Phase2-08-A)
            let year_start = time::Date::from_calendar_date(year as i32, time::Month::January, 1)?;
            let year_end = time::Date::from_calendar_date(year as i32, time::Month::December, 31)?;
            let derived = self
                .absence_service
                .derive_hours_for_range(year_start, year_end, sp_id, Authentication::Full, Some(tx.clone()))
                .await?;
            let derived_sum: f32 = derived
                .values()
                .filter(|r| matches!(r.category, c if c == category))
                .map(|r| r.hours)
                .sum();

            let drift = (legacy_sum - derived_sum).abs();
            if drift > 0.01 {
                drift_rows.push(DriftRow {
                    sales_person_id: sp_id,
                    year,
                    category,
                    legacy_sum,
                    derived_sum,
                    drift,
                });
                tracing::error!("[cutover-gate] drift sp={} {:?}/{}: legacy={} derived={} drift={}",
                    sp_id, category, year, legacy_sum, derived_sum, drift);
            }
        }
    }

    // Diff-Report-File schreiben (D-Phase4-06)
    let report_path = format!(".planning/migration-backup/cutover-gate-{}.json",
        time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Iso8601::DATE_TIME)?);
    let diff_report = build_diff_report(cutover_run_id, &drift_rows);
    let file = std::fs::File::create(&report_path).map_err(|e| ServiceError::InternalError /* extend */)?;
    serde_json::to_writer_pretty(file, &diff_report).map_err(|_| ServiceError::InternalError)?;

    Ok(GateResult {
        passed: drift_rows.is_empty(),
        drift_rows: drift_rows.len() as u32,
        diff_report_path: report_path.into(),
        scope_set: scope,
    })
}
```

### Operation 3: Insta OpenAPI Snapshot Test

```rust
// Source: rest/tests/openapi_snapshot.rs (NEU für Phase 4 D-Phase4-11)
// Verbatim aus CONTEXT.md <specifics> + Pattern-4 (sort_maps belt-and-suspenders)
use rest::ApiDoc;
use utoipa::OpenApi;

#[test]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
```

Snapshot-File: `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap` (insta-Default-Naming `<file_module>__<test_fn>.snap`).

### Operation 4: ServiceError-Variante + error_handler-Mapping → 403

```rust
// service/src/lib.rs — ergänzt nach line 119
#[error("ExtraHours category {0:?} is deprecated; use POST /absence-period for this category")]
ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory),

// rest/src/lib.rs — ergänzt im error_handler match (nach `NotLatestBillingPeriod`)
Err(RestError::ServiceError(err @ ServiceError::ExtraHoursCategoryDeprecated(_))) => {
    Response::builder()
        .status(403)
        .body(Body::new(serde_json::to_string(&serde_json::json!({
            "error": "extra_hours_category_deprecated",
            "category": format!("{:?}", /* extract from err */).to_lowercase(),
            "message": "Use POST /absence-period for this category"
        })).unwrap()))
        .unwrap()
}
```

### Operation 5: Service-Layer Flag-Check in `ExtraHoursServiceImpl::create`

```rust
// Source: synthesized analog service_impl/src/reporting.rs:475-505 (Phase-2-Präzedenz)
// service_impl/src/extra_hours.rs::create — pre-Insert-Check
async fn create(
    &self,
    entity: &ExtraHours,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<ExtraHours, ServiceError> {
    self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // NEU Phase-4 D-Phase4-09: Flag-Gated Block für 3 Kategorien
    if matches!(entity.category,
        ExtraHoursCategory::Vacation | ExtraHoursCategory::SickLeave | ExtraHoursCategory::UnpaidLeave)
    {
        let flag_active = self
            .feature_flag_service
            .is_enabled("absence_range_source_active", Authentication::Full, Some(tx.clone()))
            .await?;
        if flag_active {
            self.transaction_dao.commit(tx).await?; // Tx schließen vor Error
            return Err(ServiceError::ExtraHoursCategoryDeprecated(entity.category.clone()));
        }
    }

    // ... existing create logic ...
}
```

**Hinweis:** Diese Änderung erfordert `FeatureFlagService` als neue Dep im `ExtraHoursServiceImpl` — Plan-Phase prüft DI-Konstruktionsreihenfolge in `shifty_bin/src/main.rs:770`. FeatureFlagService wird heute in Z. 794 NACH ExtraHoursService konstruiert — Plan-Phase muss FeatureFlagService VOR ExtraHoursService bauen.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase-2 Snapshot-Bump 2→3 separat von Reporting-Switch | Atomic-Commit (D-Phase2-10): Bump + Switch + Pin-Map im selben jj-Commit | Phase 2 (2026-05-02, jj-Change `39be1b73`) | Phase 4 erbt Pattern: alle MIG-01..05 in EINER atomaren SQLite-Tx |
| Booking ⇄ Absence Cycle (Phase-3 Initial-Discuss) | Service-Tier-Konvention: Basic ↛ BL, BL → Basic einseitig | Phase 3 Re-Discuss 2026-05-02 (in CLAUDE.md dokumentiert) | Phase 4: Cycle-Vermeidung CarryoverService → CarryoverRebuildService (Pitfall 1) |
| Migration via SQL-Window-Functions | Iterativ in Rust (C-Phase4-03) | Phase 4 Discuss 2026-05-03 | Bessere Test-Coverage, SQLite-Window-Limitations vermieden |
| ExtraHours-POST-Frontend-Migration before Cutover | Atomar via Flag-Gate (D-Phase4-09) | Phase 4 Discuss 2026-05-03 | Frontend-Migration kann separat-asynchron laufen; Backend-Flag ist single-source-of-truth-Switch |
| OpenAPI als Doku-only (utoipa-Generator) | OpenAPI als Pin-File-Vertrag (insta-Snapshot) | Phase 4 (D-Phase4-11) | Stille Breaking Changes catched at `cargo test`-Time |

**Deprecated/outdated nach Phase 4:**
- `extra_hours.category IN (Vacation, SickLeave, UnpaidLeave)` als POST-Surface → 403 nach Cutover. (DELETE/GET bleiben funktional.)
- Reporting-ExtraHours-Quelle für die 3 Kategorien (war schon Phase-2-deprecated hinter Flag; Phase-4 flippt den Flag).
- `localdb.sqlite3` mit Pre-Phase-1-Migrations-State (recovered in Phase 3.06; Phase-4-deferred-items.md dokumentiert lokale-DB-Provisionierung-Hinweis).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Insta `set_sort_maps` ist über `with_settings!`-Macro setzbar (vs nur über `Settings::clone_current()`-Pfad) | Pattern 4 / Operation 3 | Niedrig — Doku referenziert `set_sort_maps` als Settings-Method; `with_settings!` ist die idiomatische Macro-Wrapper. Falls falsch, Plan-Phase nutzt Settings-Pfad direkt; Snapshot-Inhalt bleibt deterministisch. |
| A2 | Insta-Snapshot-File-Naming für Tests in `tests/`-Ordner ist `tests/snapshots/<file_module>__<test_fn>.snap` | Pattern 4 / Operation 3 | Mittel — Insta-Doku bestätigt "in einem `snapshots`-Ordner neben dem Test-File" und "`<module>__<n>.snap`". Falls Naming abweicht, erste `cargo test`-Run nach Wave-2-Insta-Accept generiert das tatsächliche File-Path; Plan-Phase verifiziert in Wave-2-Verify-Step. |
| A3 | utoipa 5 Default-Path-Order ist alphabetisch (ohne `preserve_path_order`) | Sektion #1 / Pitfall 7 | Niedrig — verified über `utoipa`-Repo-Source `pub paths: Paths` + Doku "If disabled the paths will be ordered in alphabetical order". `rest/Cargo.toml` zeigt `utoipa = "5"` ohne `preserve_path_order`. |
| A4 | `pool.begin()` in `dao_impl_sqlite::TransactionDaoImpl::new_transaction` benutzt SQLite-Default `BEGIN DEFERRED` | Sektion #2 / Pitfall 4 | Niedrig — sqlx-Doku bestätigt `Pool::begin` ruft `BEGIN` ohne Modifier (= DEFERRED). Plus kein `PRAGMA journal_mode` im Repo — DELETE-Mode ist Default. |
| A5 | Konkurrierende Reader während Cutover-Tx in DELETE-mode + DEFERRED-tx blockieren ODER sehen pre-Cutover-State (snapshot isolation) — niemals mixed | Pitfall 4 | Mittel — SQLite-WAL-Doku bestätigt Snapshot-Isolation für WAL; für DELETE-mode ist es Lock-basierte Serialization (Reader blockiert auf Writer-Lock). Verhalten ist korrekt für beide Modi — Cutover ist atomar by definition. Plan-Phase Wave-3 könnte einen Race-Test ergänzen, ist aber NICHT Pflicht (Garantie ist DB-engine-level). |
| A6 | `find_legacy_extra_hours_not_yet_migrated` als globale Read-Query mit IN-Clause für 3 Kategorien ist performance-akzeptabel für N=10000+ Bestand-Rows | Operation 1 / Sektion #3 | Niedrig — moderne SQLite-Versionen handhaben 10k-Row-Reads in ms; ein einzelner Indexed-Scan über extra_hours mit category + deleted IS NULL Filter ist O(N). Plan-Phase Wave-3 Smoke-Test mit großer Fixture verifiziert. |
| A7 | `time::OffsetDateTime::now_utc().format(...)` als File-Path-Component ist filesystem-safe (Linux + Windows kompatibel) | Operation 2 | Mittel — ISO8601 mit `:` ist auf Windows NICHT filesystem-safe. Plan-Phase muss `:` durch `-` ersetzen oder `time::format_description::parse("[year][month][day]T[hour][minute][second]Z")` mit kompaktem Format verwenden. Alternativ: separate `time::OffsetDateTime::now_utc().unix_timestamp()` als Filename (i64 ist immer safe). |

**Wenn diese Tabelle leer wäre:** Alle Claims wären verifiziert. Diese 7 sind alle LOW-MEDIUM Risiko und werden in Plan-Phase Wave-Verify-Steps abgedeckt — keine User-Confirm-Pflicht vor Plan-Phase-Start.

## Open Questions (RESOLVED)

> Alle 3 ehemals offenen Fragen sind durch Plan-Phase-Decisions adressiert (siehe RESOLVED-Marker pro Frage). Open-Questions-Block bleibt für Audit-Trail erhalten.

1. **RESOLVED — Wie groß ist der reale `extra_hours`-Bestand in Production?**
   - Was wir wissen: SC-1 fordert ein Production-Data-Profile vor jedem Migrations-Run. Format-Detail in C-Phase4-05.
   - Was unklar war: Konkret die Volumen-Größenordnung. 1k? 10k? 100k? Bestimmt Tx-Länge und damit Pitfall-4-Worst-Case.
   - **Resolution:** Volumen wird durch `CutoverService::profile()` (Wave 3, Plan 04-07) zur Cutover-Vorbereitung gemessen. Eine Fixture-basierte Smoke-Test-Pflicht (Größen 100/1000/5000) wurde in Plan-Phase **nicht** erzwungen — Operations-Briefing dokumentiert die Empfehlung. Fall > 5s Tx-Dauer in Production beobachtet wird, ist es ein Operations-Concern, kein Code-Bug.

2. **RESOLVED — ISO-Woche-53-Edge-Case: tritt das überhaupt auf?**
   - Was wir wissen: CONTEXT.md `<specifics>` Quarantäne-Reason `"iso_53_week_gap"` ist eine Zeile.
   - Was unklar war: ISO-53 entsteht nur in bestimmten Jahren (2020, 2026, 2032 usw.). Hat das Repo Bestand mit 2020-Vacation-Einträgen die Woche 53 berühren?
   - **Resolution:** Plan-Phase 04-02 fixiert die Heuristik als "Cluster bricht auf bei jeglichem Year-Boundary" — kein expliziter ISO-53-Sonderfall, Year-Boundary-Regel deckt es ab. Production-Data-Profile (SC-1) zeigt im Live-Run, ob ISO-53-Einträge existieren.

3. **RESOLVED — Diff-Report-File-Persistenz: was passiert bei Filesystem-Fehlern (disk full, permission denied)?**
   - Was wir wissen: `serde_json::to_writer(File::create(path), &report)?` returnt `io::Error`.
   - Was unklar war: Soll der Cutover dann auch fail'n (rollback) oder soll der Report nur best-effort sein?
   - **Resolution:** Plan-Phase 04-05 lockt: Filesystem-Fehler beim Diff-Report-Write = `ServiceError::InternalError` mit `tracing::error!`, Tx rollback. HR muss Disk-Space vor Cutover-Start verifizieren (Operations-Briefing).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` (Rust) | Build/Test | ✓ | (assumed stable) | — |
| `sqlx` migrations | Wave-0 | ✓ (workspace dep) | 0.8.2 | `sqlx-cli` via `nix-shell` falls neue Migration-Generation gebraucht wird (CLAUDE.local.md) |
| `insta` crate | OpenAPI-Snapshot-Test | ✗ (NEU dev-dep) | 1.47.2 (latest stable) | Plan-Phase Wave-0 ergänzt in `rest/Cargo.toml` |
| `cargo insta` CLI | Snapshot-Review | ✗ (optional, only for review) | 1.47.x | Lokal installierbar via `cargo install cargo-insta` (NICHT global ohne User-Erlaubnis — siehe MEMORY.md "Keine unauthorisierten Installs"). Manueller Diff via `git diff` der `.snap`-Files ist Fallback. |
| `.planning/migration-backup/` Verzeichnis | Diff-Report-Persistenz | ✗ (NEU) | — | Plan-Phase Wave-0 task: `mkdir -p .planning/migration-backup/` + `.gitkeep` (oder erste Cutover-Run erzeugt es per `std::fs::create_dir_all`). Empfehlung: Wave-0-Anlegen mit `.gitkeep` für Audit-Trail-Sichtbarkeit. |
| `jj` (Jujutsu) VCS | Commits (User-driven) | ✓ | (existing) | git als Fallback ist VERBOTEN (MEMORY.md "VCS-Konsistenz") |
| NixOS shell | sqlx-cli wenn verwendet | ✓ | (existing) | `nix-shell` für `sqlx-cli`-Aufrufe |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:**
- `insta` crate: Plan-Phase Wave-0 fügt es zu `rest/Cargo.toml` `[dev-dependencies]` hinzu. Kein Code-Schreiben verboten — Cargo lädt es beim nächsten Build automatisch.
- `cargo insta` CLI: optional, nur für interaktiven Review. Lokal vom Entwickler installierbar wenn benötigt — kein Phase-4-Build-Blocker.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in Rust test runner) + `tokio::test` für async + `mockall` für Service-Mocks + `proptest` für Property-Tests + `insta` (NEU für OpenAPI-Snapshot) |
| Config files | `Cargo.toml` per Crate (existing); kein zentrales test-config |
| Quick run command | `cargo test -p service_impl cutover` (Service-Layer-Mock-Tests, < 5s) |
| Full suite command | `cargo test --workspace` (alle Crates inkl. integration_test, < 60s) |
| OpenAPI-Snapshot | `cargo test -p rest --test openapi_snapshot` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MIG-01 | Heuristik-Cluster: konsekutive Werktage gleicher (sp,kat) mit `amount==contract_hours` → 1 absence_period | unit (mockall) | `cargo test -p service_impl test::cutover::cluster_merges_consecutive_workdays_with_exact_match` | ❌ Wave 1 |
| MIG-01 | Quarantäne-Pfade: amount-below, amount-above, weekend-on-workday-only-contract, contract-not-active, iso-53-gap | unit | `cargo test -p service_impl test::cutover::quarantine_*` (5 Tests) | ❌ Wave 1 |
| MIG-01 | Re-Run-Idempotenz: zweiter Cutover-Run skippt bereits gemappte extra_hours_id | unit + integration | `cargo test -p service_impl test::cutover::idempotent_rerun_skips_mapped` und `cargo test -p shifty_bin --test integration_test cutover::test_idempotence_rerun_no_op` | ❌ Wave 1 + Wave 3 |
| MIG-02 | Gate-Berechnung mit `derive_hours_for_range`-Reuse (kein Re-Implementation) | integration | `cargo test -p shifty_bin --test integration_test cutover::test_gate_uses_derive_hours_for_range_path` (Behavior: gate result == reporting result) | ❌ Wave 3 |
| MIG-02 | Gate-Drift Toleranz < 0.01h: konstruierte 0.005h-Drift = pass; 0.02h-Drift = fail | unit | `cargo test -p service_impl test::cutover::gate_tolerance_*` (2 Tests) | ❌ Wave 2 |
| MIG-02 | Diff-Report-JSON-File-Schema: Pflichtfelder vorhanden, drift-Liste korrekt | integration | `cargo test -p shifty_bin --test integration_test cutover::test_diff_report_json_schema` (öffnet generierten File, parsed mit serde_json::from_str) | ❌ Wave 3 |
| MIG-03 | REST `POST /admin/cutover/gate-dry-run` HR-permission, dry-run-mode | integration (full HTTP via Tower-test) | `cargo test -p shifty_bin --test integration_test cutover::test_gate_dry_run_endpoint_*` (success, forbidden für non-HR, 200 mit gate_passed:false bei Fixture mit Quarantäne) | ❌ Wave 3 |
| MIG-03 | REST `POST /admin/cutover/commit` cutover_admin-permission | integration | `cargo test -p shifty_bin --test integration_test cutover::test_commit_endpoint_*` (forbidden für HR-only-User, success für cutover_admin-User, gate_pass + state-change) | ❌ Wave 3 |
| MIG-04 | Atomic-Tx: alle 5 Operationen in einer Tx; Sub-Service-Error → komplette Rollback (Flag bleibt false) | integration | `cargo test -p shifty_bin --test integration_test cutover::test_atomic_rollback_on_subservice_error` (Mock CarryoverRebuildService::rebuild_for_year returnt Err; verify feature_flag = false, extra_hours unverändert, kein Backup-Row) | ❌ Wave 3 |
| MIG-04 | Carryover-Refresh-Scope: nur (sp, year)-Tupel mit non-zero Vac/Sick/UnpaidLeave-Sum | integration | `cargo test -p shifty_bin --test integration_test cutover::test_carryover_refresh_scope_only_affected_tuples` | ❌ Wave 3 |
| MIG-04 | Pre-Cutover-Backup: alle gateskopierten Tupel vor UPDATE in Backup-Tabelle | integration | `cargo test -p shifty_bin --test integration_test cutover::test_pre_cutover_backup_populated_before_update` | ❌ Wave 3 |
| MIG-04 | Soft-Delete legacy: migrierte Rows haben `deleted IS NOT NULL` + `update_process='phase-4-cutover-migration'`; Quarantäne-Rows aktiv | integration | `cargo test -p shifty_bin --test integration_test cutover::test_soft_delete_migrated_rows_only` | ❌ Wave 3 |
| MIG-04 | Flag-Flip: feature_flag.absence_range_source_active = 1 nach erfolgreichem Commit | integration | `cargo test -p shifty_bin --test integration_test cutover::test_feature_flag_set_to_true_on_commit` | ❌ Wave 3 |
| MIG-05 | `/extra-hours` POST flag-gated: vor Cutover funktional, nach Cutover 403 für Vac/Sick/UnpaidLeave | integration | `cargo test -p shifty_bin --test integration_test cutover::test_extra_hours_post_flag_gated_*` (vor: 200; nach: 403; ExtraWork bleibt 200) | ❌ Wave 3 |
| MIG-05 | OpenAPI-Snapshot lockt API-Surface (D-Phase4-11) | snapshot | `cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface` | ❌ Wave 0 (skeleton) + Wave 2 (accept) |
| MIG-05 | ServiceError::ExtraHoursCategoryDeprecated → 403 mit korrektem JSON-Body | integration | `cargo test -p shifty_bin --test integration_test cutover::test_403_body_format` (status 403, body matches `{"error":"extra_hours_category_deprecated","category":...}`) | ❌ Wave 3 |
| SC-1 | Production-Data-Profile via `CutoverService::profile()` (separater Read-Path) | integration | `cargo test -p shifty_bin --test integration_test cutover::test_profile_generates_json_with_histograms` | ❌ Wave 3 |
| SC-5 | Per-(sales_person, kategorie, jahr)-Invariant: Pre-Migration-Sum == Post-Migration-derived-Sum | integration | `cargo test -p shifty_bin --test integration_test cutover::per_sales_person_per_year_per_category_invariant` (Fixture mit allen 3 Kategorien × 2 sp × 2 Jahre, vergleicht Sums) | ❌ Wave 3 |
| Forbidden | `_forbidden`-Tests pro public service method (HR ∨ cutover_admin) | unit | `cargo test -p service_impl test::cutover::run_forbidden_for_unprivileged_user` und `_for_hr_only_when_committing` | ❌ Wave 1 |
| Wave-0 Hygiene | Standalone `cargo test -p dao` und `cargo test -p dao_impl_sqlite` grün (D-Phase4-15) | unit | `cargo test -p dao && cargo test -p dao_impl_sqlite` | ✅ existing tests; Wave-0 Cargo.toml-Patch macht sie grün |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl test::cutover` (< 5s) für Wave-1/2 Mock-Tests; `cargo test -p rest --test openapi_snapshot` (< 2s) für OpenAPI-Lock
- **Per wave merge:** `cargo build --workspace && cargo test --workspace` (< 60s)
- **Phase gate:** `cargo build --workspace && cargo test --workspace && cargo run` mit Timeout 30s (verifiziert dass Bin bootet) — vor `/gsd:verify-phase 04`

### Wave 0 Gaps

- [ ] `dao/Cargo.toml` `features = ["v4"]`-Patch — D-Phase4-15
- [ ] `dao_impl_sqlite/Cargo.toml` `features = ["v4"]`-Patch — D-Phase4-15
- [ ] `migrations/sqlite/<TS>_create-absence-migration-quarantine.sql` — D-Phase4-03 + C-Phase4-01
- [ ] `migrations/sqlite/<TS+1>_create-absence-period-migration-source.sql` — D-Phase4-04 + C-Phase4-01
- [ ] `migrations/sqlite/<TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql` — D-Phase4-13 + C-Phase4-01
- [ ] `migrations/sqlite/<TS+3>_add-cutover-admin-privilege.sql` — D-Phase4-07 + C-Phase4-08
- [ ] `rest/Cargo.toml` neue `[dev-dependencies]` block + `insta = { version = "1.47.2", features = ["json"] }`
- [ ] `rest/tests/openapi_snapshot.rs` (skeleton mit `#[ignore]` falls Snapshot-File noch nicht existiert) — D-Phase4-11
- [ ] `.planning/migration-backup/.gitkeep` — Verzeichnis-Anlegung für Diff-Reports
- [ ] `.planning/phases/04-migration-cutover/deferred-items.md` — `localdb.sqlite3`-Drift-Hinweis (D-Phase4-15)

### Wave 1 Gaps (Service-Layer)

- [ ] `service/src/cutover.rs` — Trait + DTOs (`CutoverRunResult`, `MigrationResult`, `GateResult`, `DriftRow`, `QuarantineReason`-Enum)
- [ ] `service/src/carryover_rebuild.rs` — Trait `CarryoverRebuildService` mit `rebuild_for_year`-Method
- [ ] `service/src/lib.rs` patch: `pub mod cutover;` + `pub mod carryover_rebuild;` + `ExtraHoursCategoryDeprecated`-Variante in `ServiceError`-Enum
- [ ] `service/src/extra_hours.rs` patch: `soft_delete_bulk(ids: Arc<[Uuid]>, ctx, tx)`-Method-Signatur
- [ ] `dao/src/cutover.rs` — Trait für 3 neue Tabellen + Read-Path `find_legacy_extra_hours_not_yet_migrated`
- [ ] `dao_impl_sqlite/src/cutover.rs` — Impl
- [ ] `service_impl/src/cutover.rs` — `CutoverServiceImpl` mit `gen_service_impl!` (Wave 1: Heuristik + Quarantäne; Wave 2: Gate + Refresh + Flip)
- [ ] `service_impl/src/carryover_rebuild.rs` — Impl konsumiert `ReportingService` + `CarryoverService`
- [ ] `service_impl/src/extra_hours.rs` patch: `soft_delete_bulk`-Impl + `create`-Pre-Check (Flag-Gated, D-Phase4-09)
- [ ] `service_impl/src/test/cutover.rs` — Service-Mock-Tests (Heuristik, Quarantäne, Forbidden)

### Wave 2 Gaps (Gate + REST + utoipa + Snapshot-Accept)

- [ ] `service_impl/src/cutover.rs` — Gate-Berechnung-Method, Diff-Report-File-IO, Soft-Delete-Bulk, Carryover-Refresh-Loop, Flag-Set
- [ ] `rest-types/src/lib.rs` patch: `CutoverRunResultTO`, `CutoverGateDriftRowTO`, `CutoverGateDriftReportTO`, `ExtraHoursCategoryDeprecatedErrorTO` inline + `From`-Impls
- [ ] `rest/src/cutover.rs` — 2 Handlers + `CutoverApiDoc` + `generate_route()` + `RestStateDef::cutover_service()`
- [ ] `rest/src/lib.rs` patch: `mod cutover;` + ApiDoc nest `(path = "/admin/cutover", api = cutover::CutoverApiDoc)` + Router nest + `error_handler` Mapping für `ExtraHoursCategoryDeprecated`
- [ ] `shifty_bin/src/main.rs` patch: `CutoverServiceDependencies`, `CarryoverRebuildServiceDependencies`, DI für 4 neue DAOs, Konstruktion in tier-konformer Reihenfolge (Basic → BL), `cutover_service()`-Method auf `RestStateImpl`
- [ ] `cargo insta accept` für `rest/tests/snapshots/openapi_snapshot__*.snap` — Wave-2-Verify-Step (Mensch reviewt + jj-committet)

### Wave 3 Gaps (E2E + Pflicht-Tests + Profile)

- [ ] `service_impl/src/cutover.rs` — `profile()`-Method (SC-1, separat von `run()`)
- [ ] `shifty_bin/src/integration_test/cutover.rs` — alle in der "Phase Requirements → Test Map"-Tabelle gelisteten Integration-Tests + SC-5-Invariant + Idempotence-Test + Atomic-Rollback-Test
- [ ] `shifty_bin/src/integration_test.rs` patch: `mod cutover;`

*(Falls Wave 0/1/2/3 Gaps reduziert werden während Plan-Phase: Plan-Phase darf Tests aus Wave 3 in Wave 1/2 vorziehen, sofern die unterliegende Surface da ist. Empfehlung: Atomic-Rollback-Test in Wave 2 wenn `CutoverService::run` final, sonst Wave 3.)*

### Sampling-Detail-Achsen für Plan-Phase VALIDATION.md (per Research-Brief)

- **Pre-/Post-Migration Stunden-Invariant (SC-5):** `per_sales_person_per_year_per_category_invariant`-Test in `shifty_bin/src/integration_test/cutover.rs`. Fixture: 2 sp × 2 Jahre × 3 Kategorien × ≥3 extra_hours-Cluster. Pre-Cutover: `sum(extra_hours.amount)` per Tupel. Post-Cutover (innerhalb derselben Tx, vor commit): `sum(absence_service.derive_hours_for_range(year_start..year_end, sp).where_category)`. Assert ≤ 0.001h Drift.
- **Tx-Atomicity bei Sub-Service-Fail:** `test_atomic_rollback_on_subservice_error`. Mock `CarryoverRebuildServiceImpl` mit `expect_rebuild_for_year().returning(|_| Err(ServiceError::InternalError))`. Run Cutover. Assert: feature_flag = false, extra_hours unverändert (kein soft-delete), backup-Tabelle leer, absence_period leer. Beweis: alle Sub-Service-Calls in derselben Tx, Outer-Service hat `transaction_dao.commit` NICHT erreicht.
- **Idempotenz (Re-Run-skip):** `test_idempotence_rerun_no_op`. Erste Cutover-Run committet. Zweiter Cutover-Run sollte 0 neue absence_period-Rows erzeugen (alle extra_hours_id sind in `absence_period_migration_source` gemappt) und 0 neue Quarantäne-Rows (alle quarantänierten haben deleted-Rows in extra_hours, werden gemäß Pitfall-6 stale-cleaned).
- **Flag-Race während Live-Reporting:** Optional in Wave-3. Mock-Konkurrent-Reader: spawn'e tokio-task der `feature_flag_service.is_enabled` in Loop ruft, während `CutoverService::run(dry_run=false)` läuft. Assert: alle Reader-Reads liefern entweder konsequent `false` (vor commit) oder konsequent `true` (nach commit), niemals oszillierend. Da SQLite-DELETE-Mode + DEFERRED Serialization garantiert, ist der Test eher Doku als Verteidigung — Plan-Phase kann es als deferred markieren wenn Wave-3-Zeit knapp.
- **OpenAPI-Snapshot-Determinismus:** `cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface` sollte 100x in Folge identische Output produzieren. Plan-Phase Wave-2-Verify ergänzt einen `for i in 1..=3 { cargo test }`-Smoke-Run und prüft dass kein `.snap.new`-File entsteht (Beweis: 3 identische Outputs).

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (existing) | Reuse mock_auth (dev) / OIDC (prod) — keine neue Auth-Surface |
| V3 Session Management | yes (existing) | Reuse `tower-sessions` (existing in `rest/Cargo.toml:65`) — keine neue Session-Surface |
| V4 Access Control | yes — NEUER Privileg | `cutover_admin`-Privileg via Permission-Check-Pattern (`PermissionService::check_permission` in `CutoverService::run`); HR-Branch für Dry-Run; analog FEATURE_FLAG_ADMIN_PRIVILEGE-Pattern aus Phase 2 |
| V5 Input Validation | yes (existing) | utoipa-Schema-validierung für Request-Bodies (existing); Cutover-Endpunkte haben keine User-Input außer `dry_run`-Bool — risk-low |
| V6 Cryptography | no | Keine neue Crypto-Surface in Phase 4 |

### Known Threat Patterns für SQLite/Axum/Cutover

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unprivilegierter User triggert irreversible Operation | Elevation of Privilege | `cutover_admin`-Privileg-Gate auf `POST /admin/cutover/commit`; `_forbidden`-Tests Pflicht |
| Doppel-Cutover (zwei concurrent Commit-Requests) | Tampering | SQLite-Serialization auf Write-Lock — ein Cutover blockiert den zweiten bis zum Commit/Rollback. Nach Commit: `feature_flag.absence_range_source_active = true`, zweiter Run sieht alle migrierten Rows in `absence_period_migration_source` und produziert leere migrations + leeres gate (Pitfall-6 Stale-Cleanup). Effektiv: keine doppelte Migration möglich. |
| Diff-Report mit PII (sales_person_name) auf Filesystem | Information Disclosure | `.planning/migration-backup/` ist in `.gitignore` ODER read-restricted Verzeichnis (Operations-Concern). Plan-Phase Wave-0 prüft `.gitignore` und ergänzt Eintrag falls nötig. |
| Migration-Heuristik-Bug → falsche absence_period erzeugt → falsche Stunden-Bilanzen | Tampering / Repudiation | Cutover-Gate (D-Phase4-05) MIT Toleranz < 0.01h ist die single line of defense. Pre-Cutover-Backup (D-Phase4-13) erlaubt Restore. Audit-Trail via `update_process = 'phase-4-cutover-migration'` in extra_hours + `cutover_run_id` in Migration-Tabellen. |
| `Authentication::Full`-Bypass im Sub-Service-Loop | Elevation of Privilege | OK by design — `CutoverService::run` hat outer Permission-Check (HR ∨ cutover_admin). Innere Sub-Service-Calls bekommen `Authentication::Full` (Service-internal trust, etabliert in `service_impl/src/feature_flag.rs:31-41` und `service_impl/src/absence.rs:587`). Nicht REST-exposed. |

## Sources

### Primary (HIGH confidence)

- **Repo-Code direkt verifiziert (jeder Pfad mit Zeile):**
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/04-migration-cutover/04-CONTEXT.md` (378 lines, alle 15 Decisions)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/STATE.md`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/ROADMAP.md`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/CLAUDE.md` (Service-Tier, Tx-Pattern, Snapshot-Versioning)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/CLAUDE.local.md` (jj VCS, NixOS)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/feature_flag.rs`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/absence.rs`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/carryover.rs`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/extra_hours.rs`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/lib.rs` (ServiceError-Enum, Z. 62-123)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/employee_work_details.rs` (Z. 220-278: Trait API)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/feature_flag.rs` (Z. 31-67: Auth::Full-Bypass + check_permission-Pattern)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/reporting.rs` (Z. 460-540: Reporting-Switch-Pattern)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/absence.rs` (Z. 45-62 DI; Z. 395-510 derive_hours_for_range; Z. 563-653 compute_forward_warnings als 3-Sub-Service-Tx-Präzedenz)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/carryover.rs` (Z. 14-19: Basic-Tier DI)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao/src/extra_hours.rs` (komplettes Trait)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao_impl_sqlite/src/lib.rs` (Z. 295-338: TransactionImpl + TransactionDaoImpl)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao_impl_sqlite/src/carryover.rs` (Z. 1-115: Schema-Realität)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/lib.rs` (Z. 121-249 error_handler; Z. 460-486 ApiDoc; Z. 488-559 router setup)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/Cargo.toml` (utoipa = "5", uuid features = ["v4","serde"])
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs` (Z. 1620-1800: Phase-3-Wrapper-DTO-Pattern)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao/Cargo.toml` (uuid = "1.8" OHNE v4)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao_impl_sqlite/Cargo.toml` (uuid = "1.8.0" OHNE v4)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20260501000000_add-feature-flag-table.sql`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20240618125847_paid-sales-persons.sql`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20260502170000_create-absence-period.sql`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20260105000000_app-toggles.sql` (privilege-INSERT-Pattern)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20241231065409_add_employee-yearly-vacation-carryover.sql`
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/main.rs` (Z. 770-870: DI-Konstruktionsreihenfolge)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test.rs` (Z. 266-300: TestSetup-Pattern für In-Memory-SQLite)
  - `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test/absence_period.rs` (Z. 1-90: Pattern für E2E-Test)
- **crates.io API für insta-Version:** `curl https://crates.io/api/v1/crates/insta` → `1.47.2` Updated 2026-03-30, Features: `json: ['serde']`. [VERIFIED: crates.io API, 2026-05-03]
- **utoipa-Repo Source:** `juhaku/utoipa/utoipa/src/openapi.rs` Z. 92-100: `pub paths: Paths` und `pub components: Option<Components>` — Components is BTreeMap-backed (per docs.rs Components docs). [VERIFIED: GitHub raw source]

### Secondary (MEDIUM confidence)

- **insta Settings docs (`set_sort_maps`, `set_snapshot_path`):** [docs.rs/insta/latest/insta/struct.Settings.html](https://docs.rs/insta/latest/insta/struct.Settings.html). Confirms `with_settings!` Macro Surface. Confidence MEDIUM weil exakte Snapshot-File-Naming-Convention für `tests/`-Ordner-Tests nicht 1:1 in Doku gespiegelt — verified via Insta `<module>__<n>.snap`-Pattern aus README + Settings docs.
- **SQLite Transaction-Modi:** [sqlite.org/lang_transaction.html](https://www.sqlite.org/lang_transaction.html) — `BEGIN` = `BEGIN DEFERRED` Default. Reader sieht Snapshot pre-Tx in WAL; in DELETE-Mode blockiert Reader auf Writer-Lock.
- **SQLite WAL Modes / Snapshot Isolation:** [sqlite.org/wal.html](https://www.sqlite.org/wal.html) — bestätigt `journal_mode=WAL` ist NICHT Default; muss explizit aktiviert werden via `PRAGMA`. Repo benutzt Default (DELETE-Mode) — verified via `grep -rn "journal_mode\|PRAGMA"` (no hits).
- **utoipa OpenAPI Determinismus:** [utoipa README](https://github.com/juhaku/utoipa) — bestätigt `preserve_path_order` und `preserve_order` Features (beide NICHT in Repo aktiviert), Default ist alphabetisch. Components verwendet BTreeMap.

### Tertiary (LOW confidence — Plan-Phase verifiziert)

- **Insta-Snapshot-Naming-Convention für `tests/openapi_snapshot.rs`:** Plan-Phase Wave-2-Verify-Step (`cargo test`-Run, dann `ls rest/tests/snapshots/`) verifiziert das tatsächliche Filename. Erwartung: `openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap`. Falls anders, Pin-File-Pfad in 04-CONTEXT.md update'n.
- **Filesystem-safe ISO-Timestamp im Filename:** Plan-Phase Wave-3-Smoke-Test verifiziert auf Linux + (falls relevant) Windows. Empfehlung: Unix-Timestamp statt ISO-String wenn Filesystem-Safety unklar.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — alle Versionen via Cargo.toml und crates.io API verifiziert; insta 1.47.2 + utoipa 5 + sqlx 0.8.2 sind aktuell
- Architecture: HIGH — Multi-Service-Tx-Pattern direkt im Repo verifiziert (`service_impl/src/absence.rs:585-628` als 3-Sub-Service-Präzedenz); Service-Tier-Cycle-Risk explizit identifiziert (Pitfall 1)
- Pitfalls: HIGH — alle 7 Pitfalls aus CONTEXT.md durchgegangen + 1 zusätzlicher (CarryoverService-Cycle); SQLite-Tx-Verhalten durch SQLite-Doku verifiziert
- Insta + utoipa Determinismus: HIGH — utoipa 5 BTreeMap-default (Source-verified); Insta sort_maps verfügbar (Docs-verified); File-Naming-Convention als A2 Assumption (LOW-Risk, Wave-2 verifiziert)

**Research date:** 2026-05-03
**Valid until:** 2026-06-02 (30 Tage — stable Domain mit klarem CONTEXT.md; nur Insta + crates.io Versionen könnten in der Zwischenzeit minor-bumpen)
