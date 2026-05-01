# Phase 1: Absence Domain Foundation – Research

**Researched:** 2026-05-01
**Domain:** Neue parallele `absence_period`-Domain im bestehenden Rust/Axum/SQLite-Backend (REST → Service-Trait → DAO-Trait → SQLx).
**Confidence:** HIGH (Pattern, Stack und Codebase sind verifiziert; alle Empfehlungen aus existierendem Code im Repo abgeleitet)

> Dokument-Sprache: **Deutsch** (Prosa). Code, Identifier, SQL und Compiler-Symbole bleiben in der jeweiligen Originalform.

---

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Naming & Schema-Surface**

- **D-01:** Entity-Name `AbsencePeriod` (per ABS-01). Module: `service/src/absence.rs`, `service_impl/src/absence.rs`, `dao/src/absence.rs`, `dao_impl_sqlite/src/absence.rs`, `rest/src/absence.rs`. Tabelle `absence_period`. REST-Pfad `/absence-period`. Transport-Object `AbsencePeriodTO` in `rest-types/src/absence_period_to.rs`.
- **D-02:** **3 Kategorien** in Phase 1: `Vacation`, `SickLeave`, `UnpaidLeave`. `Holiday`, `Unavailable`, `VolunteerWork`, `ExtraWork`, `CustomExtraHours` bleiben hour-based im bestehenden `ExtraHours`-Pfad.
- **D-03:** **Eigene `AbsenceCategory`-Enum** in `service/src/absence.rs` (mit DAO-Spiegel `AbsenceCategoryEntity` in `dao/src/absence.rs`). Kein Reuse von `ExtraHoursCategory`.
- **D-04:** Schema-Constraints (Vollpaket): DB-`CHECK (to_date >= from_date)`, partial unique index `(logical_id) WHERE deleted IS NULL` (analog `20260428101456_add-logical-id-to-extra-hours.sql`), composite index `(sales_person_id, from_date) WHERE deleted IS NULL`, `NOT NULL` auf `from_date`/`to_date`/`category`/`sales_person_id`/`id`/`logical_id`/`created`/`update_version`, soft-delete via nullable `deleted`.
- **D-05:** Date-Storage als `TEXT` im ISO-8601 Format. Inclusive bounds beidseitig.

**Update-Semantik & logical_id**

- **D-06:** Mutable: `from_date`, `to_date`, `description`, `category`. Immutable: `sales_person_id`, `id` (== `logical_id`). Self-Overlap-Detection beim Update auf die **neue** Kategorie.
- **D-07:** logical_id-Pattern 1:1 ExtraHours: `find_by_logical_id` → Permission-Check → sales_person-Match → version-Match → soft-delete alte Row → neue Row mit selber `logical_id`/neuer physischer `id`/neuer `update_version` → commit. Vorlage: `service_impl/src/extra_hours.rs:220-300`.
- **D-08:** **Booking-Konflikt-Detection ist KEIN Phase-1-Thema.** `AbsenceService::create/update` liefert `Result<AbsencePeriod, ServiceError>` — keinen Warning-Wrapper, keinen `BookingService`-Dependency, keine Booking-Lookups.

**Permission-Modell (ABS-05)**

- **D-09:** Write-Operations: `tokio::join!`-Pattern `(check_permission(HR_PRIVILEGE) ∨ verify_user_is_sales_person(sales_person_id))`. Kein neues Privileg.
- **D-10:** Read-Sicht: HR sieht alle, Mitarbeiter ohne HR-Rechte sieht eigene Einträge plus die der Schichtplan-Kollegen. Detail-Ausarbeitung in Plan-Phase basierend auf existierenden Booking-Read-Konventionen.
- **D-11:** `_forbidden`-Test pro public service method ist Pflicht.

**Self-Overlap & Validierung**

- **D-12:** Self-Overlap-Scope per `(sales_person_id, category, range)`. Vacation und SickLeave dürfen denselben Tag überdecken (Cross-Category ist Phase-2-Thema).
- **D-13:** Overlap-Error-Variante: `ServiceError::ValidationError(Arc<[ValidationFailureItem]>)`. Plan-Phase erweitert `ValidationFailureItem` um eine geeignete Variante (z.B. `OverlappingPeriod(Uuid)`) oder benutzt `Duplicate` mit Kontext.
- **D-14:** Range-Validierung doppelt: Service (`ServiceError::DateOrderWrong`) + DB-CHECK-Constraint.
- **D-15:** Self-Overlap wird sowohl bei `create` als auch bei `update` geprüft. Beim Update Filter `WHERE logical_id != ?`.

**DateRange-Utility**

- **D-16:** `shifty_utils::DateRange` in `shifty-utils/src/date_range.rs` (neu). API: `DateRange { from, to }`, `new(from, to) -> Result<_, RangeError>`, `overlaps`, `contains`, `iter_days`, `day_count`. Inclusive beidseitig.
- **D-17:** Phase 1 nutzt `overlaps()` und `contains()`; `iter_days()` und `day_count()` sind bereits da für Phase 2.

### Claude's Discretion

- **C-01:** `AbsenceDao::find_overlapping`-Signatur — Plan-Phase entscheidet `(sales_person_id, category, range)` vs. `(sales_person_id, range)`. Vorgabe: kategorie-scoped (D-12).
- **C-02:** REST-Filter-Query-Params (`?sales_person_id=…&from=…&to=…&category=…`) — Plan-Phase übernimmt Pattern von `rest/src/booking.rs` und `rest/src/extra_hours.rs`.
- **C-03:** `description`-Pflichtigkeit: `Arc<str>` mit leerem String als Default; im DTO `Option<String>` mit `#[serde(default)]`. Plan-Phase darf `Required` machen.
- **C-04:** OpenAPI-Annotationen: Standard `#[utoipa::path]`-Pattern; `AbsencePeriodTO` mit `#[derive(ToSchema)]`. Eintrag in `ApiDoc` in `rest/src/lib.rs`.
- **C-05:** DI-Reihenfolge in `main.rs`: mechanische Erweiterung um `AbsenceServiceDependencies`-Block analog `BookingServiceDependencies`. `.nest("/absence-period", absence::generate_route())` im Router.

### Deferred Ideas (OUT OF SCOPE)

- Holiday/Unavailable/VolunteerWork als Range (v2-CAT)
- Approval-Workflow (v2-APRV)
- Halbtage / Stundengenau (v2-GRAN)
- Self-Service-Antrag mit Status-Tracking (v2)
- Booking-Konflikt-Wrapper-Type (`AbsencePeriodCreateResult`) — Phase 3 (BOOK-01)
- `find_overlapping_for_booking(sales_person_id, range)` — Phase 3
- Reverse-Booking-Warning aus AbsencePeriod-Quelle — Phase 3 (BOOK-02)
- Reporting-Integration / `derive_hours_for_range` — Phase 2
- Frontend (Dioxus) — separater Workstream

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **ABS-01** | Neue Entity `AbsencePeriod` (Felder: id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted). Soft-Delete-Semantik. | Schema-Migration (§3), Domain-Modell + Entity-Spiegel + DTO (§4–§6), TryFrom-Pattern aus `dao_impl_sqlite/src/extra_hours.rs:27-68`. |
| **ABS-02** | DAO-Trait + SQLite-Impl mit `create`, `find_by_id`, `find_by_logical_id`, `find_by_sales_person`, `find_overlapping`, `update`, `soft_delete`. SQLx-compile-time-checked, `WHERE deleted IS NULL`. | DAO-Trait-Shape (§6), SQLx-Patterns (§7), Trait-Vorlage `dao/src/extra_hours.rs:42-86`, Impl-Vorlage `dao_impl_sqlite/src/extra_hours.rs:79-249`. |
| **ABS-03** | Service-Trait + Impl: Range-Validierung (`from_date <= to_date`), Self-Overlap-Detection (`sales_person + category + Range`), Permission-Check, `Option<Transaction>`. | Service-Trait-Shape (§5), Self-Overlap-Channel (§8), Update-Flow-Vorlage `service_impl/src/extra_hours.rs:220-300`. |
| **ABS-04** | REST: `POST`, `GET`-list, `GET`-by-id, `PATCH`/`PUT`, `DELETE` mit `#[utoipa::path]`. Transport-Objects mit `ToSchema`. | REST-Endpoints (§10), Vorlage `rest/src/extra_hours.rs:1-207`, TO-Pattern aus `rest-types/src/lib.rs:741-789`. |
| **ABS-05** | Permission-Check via `PermissionService` integriert; HR + Sales-Person-Self dürfen anlegen/ändern. | Permission-Pattern (§9), Vorlage `service_impl/src/extra_hours.rs:236-245` (write) + `service_impl/src/sales_person_unavailable.rs:41-50` (read). |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

Diese Direktiven aus `shifty-backend/CLAUDE.md` und `shifty-backend/CLAUDE.local.md` müssen im Plan eingehalten werden — sie sind **nicht** verhandelbar:

| # | Constraint | Quelle |
|---|------------|--------|
| CC-01 | Layered Architecture: REST → Service-Trait → DAO-Trait → SQLite. Keine Schicht überspringen, keine SQL-Strings im Service-Layer. | `shifty-backend/CLAUDE.md` "Architecture Overview" |
| CC-02 | `gen_service_impl!`-Macro für DI in jedem `*ServiceImpl`. | `shifty-backend/CLAUDE.md` "Implementation Patterns" |
| CC-03 | Jede Service-Methode akzeptiert `Option<Transaction>` und folgt dem `use_transaction → … → commit`-Pattern. | `shifty-backend/CLAUDE.md` "Transaction Management" |
| CC-04 | `Authentication<Context>` wird durch jeden Service-Call gereicht. | `shifty-backend/CLAUDE.md` "Authentication & Authorization" |
| CC-05 | Jede DAO-Read-Query enthält `WHERE deleted IS NULL`. | `shifty-backend/CLAUDE.md` "DAO Implementation" + `.planning/codebase/CONVENTIONS.md` "Soft Delete" |
| CC-06 | Jeder REST-Handler trägt `#[utoipa::path(...)]` und das DTO `#[derive(ToSchema)]`. Endpunkte werden in `ApiDoc` registriert. | `shifty-backend/CLAUDE.md` "OpenAPI Documentation" |
| CC-07 | Snapshot-Schema-Versioning: `CURRENT_SNAPSHOT_SCHEMA_VERSION` darf in **Phase 1 NICHT** verändert werden. Phase 1 ist additiv, ohne Reporting-Wirkung. | `shifty-backend/CLAUDE.md` "Billing Period Snapshot Schema Versioning" + ROADMAP.md (Bump in Phase 2) |
| CC-08 | `cargo build`, `cargo test` und (mit Timeout) `cargo run` werden nach Implementation ausgeführt. | `shifty-backend/CLAUDE.md` letzte Zeile |
| CC-09 | Tests Pflicht für jede Änderung: `_success`- und `_forbidden`-Test pro public service method. | global `~/.claude/CLAUDE.md` + Phase-1-Erfolgskriterium (D-11) |
| CC-10 | NixOS: `sqlx-cli` ggf. via `nix-shell` (z.B. `nix-shell -p sqlx-cli --run "sqlx migrate add …"`). | `shifty-backend/CLAUDE.local.md` |
| CC-11 | VCS via `jj` co-located mit git. **Niemals `git commit`/`git add`** aufrufen. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`). | `shifty-backend/CLAUDE.local.md` + `.planning/config.json` |
| CC-12 | i18n: User-sichtbare DTO-Felder, die Texte transportieren, müssen Frontend-i18n-tauglich sein (für `description` faktisch transparent — reiner String). | `shifty-backend/CLAUDE.md` "Constraints In Force" |

---

## Summary

Phase 1 baut eine neue, **strikt additive** `absence_period`-Domain end-to-end: Migration (Schema + Indexe + CHECK), DAO mit SQLx-compile-time-checked Queries, Service mit Range-Validierung, Self-Overlap-Detection und Permission-Gate, REST-Endpunkte mit OpenAPI-Annotationen, Transport-Objects, eine `DateRange`-Utility in `shifty-utils` und die DI-Verdrahtung in `shifty_bin/src/main.rs`. **Alle architektonischen Entscheidungen sind in CONTEXT.md (D-01..D-17) gepinnt.** Das Research-Ergebnis legt deshalb die exakten Code-Konturen fest — Schema-SQL, Trait-Signaturen, das Range-Overlap-SQL-Idiom, die `ValidationFailureItem`-Erweiterung, die REST-Routen-Surface und die Test-Matrix — sodass die Plan-Phase nur noch Tasks und Reihenfolge spezifizieren muss.

Die Domain ist konzeptionell einfach (CRUD mit Range), aber die Korrektheits-Hotspots liegen an drei Stellen:
1. **Range-Overlap-SQL** muss inclusive-bounds-Allen-Form verwenden (`existing.from <= probe.to AND existing.to >= probe.from`), nicht half-open.
2. **logical_id-Update** muss Self-Overlap **mit Filter `logical_id != ?`** prüfen, sonst kollidiert die Row mit sich selbst.
3. **Permission-Gate** für Read-Operations (D-10) ist die einzige Discretion-Frage in der Schreib-Surface — die Empfehlung ist eine Dual-Endpoint-Strategie (HR-Vollsicht + per-sales-person-Self-Sicht), die das bestehende `extra_hours.rs:114-148`-Idiom 1:1 spiegelt.

**Primary recommendation:** Migration und DAO zuerst (Wave A), Service + DateRange (Wave B parallel), REST + DI (Wave C), Integration-Test (Wave D). Snapshot-Schema-Versioning bleibt **bewusst** unberührt — dies ist ein Phase-2-Thema und wird durch eine Verifikation am Phasenende abgesichert (kein Diff in `service_impl/src/billing_period_report.rs:CURRENT_SNAPSHOT_SCHEMA_VERSION`).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Range-Validierung (`from_date <= to_date`) | Service (Pre-DAO-Check) | Database (`CHECK`-Constraint) | Defense-in-depth; Service liefert klaren `ServiceError::DateOrderWrong`; DB schützt gegen direkte Schreibpfade (Migrations, manuelle Inserts). [VERIFIED: D-14 + STACK.md] |
| Self-Overlap-Detection | Service | DAO (`find_overlapping`-Query) | Business-Regel ist „selber `sales_person_id`+`category`+ Range-Overlap"; SQL liefert die Kandidaten, Service entscheidet (D-12). |
| Permission-Gate (Write) | Service (`tokio::join!`) | — | HR-Privileg + Self-Verifikation zusammen, vor jeder Schreiboperation. [VERIFIED: D-09; Vorlage `service_impl/src/extra_hours.rs:236-245`] |
| Permission-Gate (Read) | Service | — | HR sieht alle, Sales-Person-Self sieht eigene + Schichtplan-Kollegen (D-10); Detail in Plan-Phase. |
| Soft-Delete | DAO (`UPDATE deleted = ?`) | — | Konvention `WHERE deleted IS NULL` in jeder Read-Query. [VERIFIED: CONVENTIONS.md "Soft Delete Convention"] |
| logical_id-Update (write-once + tombstone) | Service | DAO | Service orchestriert (find_by_logical_id → soft-delete tombstone → insert neue Row); DAO liefert nur die zwei Primitiven. [VERIFIED: `service_impl/src/extra_hours.rs:220-300`] |
| OpenAPI-Schema-Generierung | rest-types (`#[derive(ToSchema)]`) + REST (`#[utoipa::path]`) | — | Doppelpunkt: Schema beim DTO, Pfad-Annotation beim Handler. [VERIFIED: CC-06] |
| DateRange-Wrapper | shifty-utils | — | Domain-übergreifender Pure-Type ohne sqlx/serde-Kopplung; in service/dao identisch verwendbar. [VERIFIED: D-16, STACK.md `Domain Types — Code You Will Write`] |
| DI-Verdrahtung | shifty_bin (main.rs) | — | Konkrete Instanzen + `*ServiceDependencies`-Block — die einzige Stelle, an der konkrete Typen treffen. |

## Standard Stack

### Core (Bestandsstack — KEEP, kein Versions-Drift)

| Library | Version (in Tree) | Purpose | Why Standard |
|---------|-------------------|---------|--------------|
| `time` | 0.3.36 (service, rest), 0.3.41 (utils) | `time::Date` für inclusive Date-Ranges, `PrimitiveDateTime` für Audit-Spalten (`created`, `deleted`) | Bestand. `time::Date` mappt natively auf SQLite-TEXT, hat `ToSchema` über utoipa-`time`-Feature. **Nicht** auf `chrono` oder `jiff` switchen. [VERIFIED: STACK.md Quelle] |
| `sqlx` | 0.8.2 (`runtime-tokio`, `sqlite`-Features) | Compile-time-checked Queries; native `time::Date ↔ TEXT`-Mapping | Bestand. `query_as!`-Macro hat in Phase 1 ohne Migrations-Run schon den neuen Schema-Bezug — d.h. **Migration MUSS vor `cargo check` laufen**. [VERIFIED: `dao_impl_sqlite/Cargo.toml:18-20`] |
| `utoipa` | 5 | OpenAPI-Schemas auf DTOs (`ToSchema`), Pfad-Annotationen (`#[utoipa::path]`) | Bestand. [VERIFIED: STACK.md] |
| `serde` / `serde_json` | 1.x | Serialisierung der DTOs | Bestand. `time::Date` rundet als ISO-8601-String (über `serde-human-readable` in `rest/Cargo.toml`). |
| `thiserror` | 2.0 | `ServiceError`, `DaoError` | Bestand; neuer `RangeError` für `DateRange` lässt sich plug-and-play hinzufügen. |
| `mockall` | 0.13 | `#[automock]` für Service- und DAO-Traits | Bestand. `MockAbsenceService`/`MockAbsenceDao` werden automatisch generiert. |
| `tokio` | 1.44 | `async`-Runtime, `tokio::join!` für parallele Permission-Checks | Bestand. |
| `async-trait` | 0.1.80 | `#[async_trait]` für trait-async-fns | Bestand. |
| `tracing` | 0.1.40 | `#[instrument]` auf REST-Handlern; `warn!`/`error!` | Bestand. |
| `uuid` | 1.8 | `Uuid::new_v4()` über `UuidService::new_uuid(process_label)` | Bestand. |

**Versions-Verifikation:** Versionen wurden 1:1 aus den existierenden `Cargo.toml`-Dateien gelesen (siehe `dao_impl_sqlite/Cargo.toml:18-26`, `shifty-utils/Cargo.toml:8`, `service/Cargo.toml:10`, `rest/Cargo.toml:20`). Phase 1 fügt **keine** neuen Dependencies hinzu — alles läuft auf dem bestehenden Stack. Ein optionaler `time`-Bump (0.3.36 → 0.3.47) wird in STACK.md erwähnt, ist aber **nicht** Phase-1-Scope. [VERIFIED: filesystem inspection]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (keine neuen Dependencies) | — | — | Phase 1 ist additiv — neuer Code, kein neuer Stack. [VERIFIED: STACK.md, alle Hand-Patterns sind bereits im Tree] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled `DateRange` in `shifty-utils` | `range-set-blaze`, `intervaltree` | Generic-Crate-Surface ist domain-agnostisch und zwingt zu ungeeigneter Modellierung. STACK.md lehnt das explizit ab. |
| `time::Date` | `chrono`, `jiff` | Workspace-weite Migration. Kein Feature-Payoff für Phase 1 (whole-day, kein Timezone-Bezug). STACK.md verbietet das ausdrücklich. |
| `validator` / `garde`-Derive-Macros | — | 4 Checks (range, self-overlap, IdSetOnCreate, VersionSetOnCreate). Inline-Service-Validation ist projektidiomatisch. |
| `PrimitiveDateTime` für `from_date`/`to_date` | — | Phantom-Sub-Day-Präzision; Self-Overlap-Queries werden auf `<` vs. `<=` zerschossen. STACK.md "Why date-only". |

**Installation:** Keine. Phase 1 fügt keine Workspace-Dependencies hinzu. Die einzige neue Datei in `shifty-utils/` (`date_range.rs`) verwendet nur `time` und `thiserror`, beide bereits in `shifty-utils/Cargo.toml`.

## Architecture Patterns

### System Architecture Diagram

```
HTTP Client (Dioxus / Swagger UI / curl)
   │
   │  JSON over HTTP
   ▼
┌───────────────────────────────────────────────────────────┐
│ rest/src/absence.rs                                       │
│   POST   /absence-period          → create_absence_period │
│   GET    /absence-period          → get_absence_periods   │
│   GET    /absence-period/{id}     → get_absence_period    │
│   PUT    /absence-period/{id}     → update_absence_period │
│   DELETE /absence-period/{id}     → delete_absence_period │
│                                                           │
│   #[utoipa::path], error_handler, AbsencePeriodTO         │
└───────────────────────────────────────────────────────────┘
   │
   │  &AbsencePeriod, Authentication<Context>, Option<Tx>
   ▼
┌───────────────────────────────────────────────────────────┐
│ service_impl/src/absence.rs (AbsenceServiceImpl)          │
│   ┌──────────────────────────────────────────────────┐    │
│   │ tx = transaction_dao.use_transaction(tx)         │    │
│   │ join!(check_permission(HR), verify_user_is_sp)   │    │
│   │ → permission OK?                                 │    │
│   │ → range valid? (DateRange::new ⇒ DateOrderWrong) │    │
│   │ → self_overlap? (find_overlapping + filter)      │    │
│   │ → DAO call (create | tombstone+insert | update)  │    │
│   │ → transaction_dao.commit(tx)                     │    │
│   └──────────────────────────────────────────────────┘    │
│                                                           │
│   Deps: AbsenceDao, PermissionService,                    │
│         SalesPersonService, SalesPersonShiftplanService   │
│         (für D-10), ClockService, UuidService,            │
│         TransactionDao                                    │
└───────────────────────────────────────────────────────────┘
   │
   │  AbsencePeriodEntity, &str process, Transaction
   ▼
┌───────────────────────────────────────────────────────────┐
│ dao_impl_sqlite/src/absence.rs (AbsenceDaoImpl)           │
│   AbsencePeriodDb (sqlx::query_as!)                       │
│   - find_by_id, find_by_logical_id, find_by_sales_person  │
│   - find_overlapping (Allen's algebra, inclusive bounds)  │
│   - create (INSERT), update (UPDATE für tombstone)        │
│   ALL: WHERE deleted IS NULL                              │
└───────────────────────────────────────────────────────────┘
   │
   │  SQLx-Connection (sqlite::memory in tests, file in prod)
   ▼
┌───────────────────────────────────────────────────────────┐
│ SQLite: absence_period                                    │
│   id BLOB PK, logical_id BLOB NOT NULL,                   │
│   sales_person_id BLOB NOT NULL → sales_person.id,        │
│   category TEXT NOT NULL,                                 │
│   from_date TEXT NOT NULL, to_date TEXT NOT NULL,         │
│   description TEXT, created TEXT NOT NULL, deleted TEXT,  │
│   update_process TEXT NOT NULL, update_version BLOB NOT NULL │
│                                                           │
│   CHECK (to_date >= from_date)                            │
│   UNIQUE(logical_id) WHERE deleted IS NULL                │
│   INDEX (sales_person_id, from_date) WHERE deleted IS NULL│
│   INDEX (sales_person_id, category, from_date)            │
│         WHERE deleted IS NULL                             │
└───────────────────────────────────────────────────────────┘
```

**Komponenten-Verantwortlichkeiten:**

| Komponente | Datei (neu in Phase 1) | Verantwortlich für |
|-----------|------------------------|---------------------|
| Migration | `migrations/sqlite/<ts>_create-absence-period.sql` | Schema, Indizes, CHECK |
| DateRange-Utility | `shifty-utils/src/date_range.rs` | Pure inclusive Range-Type, `overlaps`, `contains`, `iter_days`, `day_count`, `RangeError` |
| Domain-Modell + Enum | `service/src/absence.rs` | `AbsencePeriod`-Struct, `AbsenceCategory`-Enum, `From<&AbsencePeriodEntity>`/`TryFrom<&AbsencePeriod>` |
| DAO-Trait + Entity | `dao/src/absence.rs` | `AbsenceDao`-Trait (`#[automock]`), `AbsencePeriodEntity`, `AbsenceCategoryEntity` |
| DAO-Impl | `dao_impl_sqlite/src/absence.rs` | SQLx-Queries, `AbsencePeriodDb`-Row, `TryFrom<&AbsencePeriodDb>` |
| Service-Trait | `service/src/absence.rs` | `AbsenceService`-Trait (`#[automock]`) |
| Service-Impl | `service_impl/src/absence.rs` | Range-Validierung, Self-Overlap, Permission-Gate, Transaction, logical_id-Update |
| REST-Handler | `rest/src/absence.rs` | Routen, `#[utoipa::path]`, `error_handler` |
| TO | `rest-types/src/absence_period_to.rs` (oder inline in `lib.rs`) | `AbsencePeriodTO`, `AbsenceCategoryTO`, bidirektionale `From`-Impls |
| Test (Service-Unit) | `service_impl/src/test/absence.rs` | Mock-basierte Tests pro Methode (`_success`, `_forbidden`, `_overlap`, …) |
| Test (Integration) | `shifty_bin/src/integration_test/absence_period.rs` | CRUD-Round-Trip mit echtem In-Memory-SQLite + Migration-Run |
| DI-Verdrahtung | `shifty_bin/src/main.rs` (Patch) | `AbsenceServiceDependencies`-Block, konkrete `Arc::new(AbsenceServiceImpl { … })` |
| Module-Re-Exports | `service/src/lib.rs`, `service_impl/src/lib.rs`, `dao/src/lib.rs`, `dao_impl_sqlite/src/lib.rs`, `rest/src/lib.rs`, `rest-types/src/lib.rs`, `shifty-utils/src/lib.rs` (Patches) | `pub mod absence` (etc.), Re-Export, Router-Nesting, ApiDoc |

### Recommended Project Structure (Patches in Phase 1)

```
shifty-utils/src/
├── lib.rs                      [PATCH: pub mod date_range; pub use date_range::*;]
├── date_utils.rs
└── date_range.rs               [NEU]

service/src/
├── lib.rs                      [PATCH: pub mod absence;]
└── absence.rs                  [NEU]

service_impl/src/
├── lib.rs                      [PATCH: pub mod absence;]
├── absence.rs                  [NEU]
└── test/
    ├── mod.rs                  [PATCH: pub mod absence;]
    └── absence.rs              [NEU]

dao/src/
├── lib.rs                      [PATCH: pub mod absence;]
└── absence.rs                  [NEU]

dao_impl_sqlite/src/
├── lib.rs                      [PATCH: pub mod absence;]
└── absence.rs                  [NEU]

rest/src/
├── lib.rs                      [PATCH: mod absence; AbsenceApiDoc-nest; .nest("/absence-period", …)]
└── absence.rs                  [NEU]

rest-types/src/
├── lib.rs                      [PATCH: AbsencePeriodTO, AbsenceCategoryTO, From-Impls — entweder inline (Konvention für die meisten DTOs im Repo) oder via `mod absence_period_to;`]

migrations/sqlite/
└── <timestamp>_create-absence-period.sql   [NEU]

shifty_bin/src/
├── main.rs                     [PATCH: AbsenceServiceDependencies, Arc::new(AbsenceServiceImpl { … }), RestStateImpl-Erweiterung]
└── integration_test/
    └── absence_period.rs       [NEU]
```

### Pattern 1: gen_service_impl!-DI

**What:** Macro-generierte Dependency-Injection-Trait + Impl-Struct.
**When to use:** Jeder neue `*ServiceImpl` (CC-02 ist non-negotiable).
**Example (verifiziertes Repo-Idiom):**

```rust
// service_impl/src/absence.rs (NEU)
use crate::gen_service_impl;
use service::{
    absence::{AbsencePeriod, AbsenceService},
    permission::{Authentication, PermissionService, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    sales_person_shiftplan::SalesPersonShiftplanService,  // für D-10 Read-Sicht
    clock::ClockService,
    uuid_service::UuidService,
    ServiceError, ValidationFailureItem,
};
use dao::{absence::AbsenceDao, TransactionDao};

gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SalesPersonShiftplanService: SalesPersonShiftplanService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_shiftplan_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

[VERIFIED: `service_impl/src/extra_hours.rs:22-32` und `service_impl/src/booking.rs:20-31` sind die direkten Vorlagen]

### Pattern 2: logical_id-Update-Flow

**What:** Domain-`id` == DAO-`logical_id`. Update = soft-delete der alten Row + Insert einer neuen Row mit gleichem `logical_id`, neuer physischer `id`, neuer `update_version`.

**When to use:** Jede `update`-Methode, die mutationssicher und audit-fähig sein soll. Phase 1 erbt das 1:1 von ExtraHours (D-07).

**Example (1:1 Vorlage; ExtraHours-Code zeilenweise zitiert):**

```rust
// service_impl/src/absence.rs::update — Stencil aus extra_hours.rs:220-301
async fn update(
    &self,
    request: &AbsencePeriod,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<AbsencePeriod, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    let logical_id = request.id;

    let active = self
        .absence_dao
        .find_by_logical_id(logical_id, tx.clone())
        .await?
        .ok_or(ServiceError::EntityNotFound(logical_id))?;

    let (hr_permission, sales_person_permission) = join!(
        self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            active.sales_person_id, context.clone(), tx.clone().into(),
        ),
    );
    hr_permission.or(sales_person_permission)?;

    if request.sales_person_id != active.sales_person_id {
        return Err(ServiceError::ValidationError(Arc::from([
            ValidationFailureItem::ModificationNotAllowed("sales_person_id".into()),
        ])));
    }
    if request.version != active.version {
        return Err(ServiceError::EntityConflicts(
            logical_id, request.version, active.version,
        ));
    }

    // D-14: Range-Validierung (DateRange::new -> DateOrderWrong)
    let new_range = DateRange::new(request.from_date, request.to_date)
        .map_err(|_| ServiceError::DateOrderWrong(request.from_date, request.to_date))?;

    // D-15: Self-Overlap-Detection mit Filter "logical_id != ?"
    let conflicts = self.absence_dao
        .find_overlapping(
            active.sales_person_id,
            (&request.category).into(),
            new_range,
            Some(logical_id),     // exclude_logical_id
            tx.clone(),
        ).await?;
    if !conflicts.is_empty() {
        return Err(ServiceError::ValidationError(Arc::from([
            ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
        ])));
    }

    // Tombstone alte Row
    let mut tombstone = active.clone();
    tombstone.deleted = Some(self.clock_service.date_time_now());
    self.absence_dao.update(&tombstone, "absence_service::update::soft_delete", tx.clone()).await?;

    // Neue aktive Row
    let new_id = self.uuid_service.new_uuid("absence_service::update::id");
    let new_version = self.uuid_service.new_uuid("absence_service::update::version");
    let now = self.clock_service.date_time_now();

    let new_entity = absence::AbsencePeriodEntity {
        id: new_id,
        logical_id: active.logical_id,
        sales_person_id: active.sales_person_id,
        category: (&request.category).into(),
        from_date: request.from_date,
        to_date: request.to_date,
        description: request.description.clone(),
        created: now,
        deleted: None,
        version: new_version,
    };
    self.absence_dao.create(&new_entity, "absence_service::update::insert", tx.clone()).await?;

    self.transaction_dao.commit(tx).await?;
    Ok(AbsencePeriod::from(&new_entity))
}
```

[VERIFIED: `service_impl/src/extra_hours.rs:220-301` ist die direkte Vorlage]

### Pattern 3: REST-Handler mit utoipa und error_handler

**Example (1:1 aus `rest/src/extra_hours.rs:121-162`):**

```rust
// rest/src/absence.rs
#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    request_body = AbsencePeriodTO,
    responses(
        (status = 200, description = "Updated absence period", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence period not found"),
        (status = 409, description = "Version conflict"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn update_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(absence_id): Path<Uuid>,
    Json(absence_to): Json<AbsencePeriodTO>,
) -> Response {
    error_handler((async {
        let mut entity: service::absence::AbsencePeriod = (&absence_to).into();
        entity.id = absence_id;  // path-id wins über body-id (vergleiche update_extra_hours)
        let updated = AbsencePeriodTO::from(
            &rest_state.absence_service().update(&entity, context.into(), None).await?,
        );
        Ok(Response::builder().status(200).header("Content-Type", "application/json")
            .body(Body::new(serde_json::to_string(&updated).unwrap())).unwrap())
    }).await)
}
```

### Anti-Patterns to Avoid

- **PrimitiveDateTime für from_date/to_date:** STACK.md verbietet das. `time::Date` ist die korrekte Wahl. Phantom-Hour-Präzision macht Self-Overlap-Queries kaputt.
- **`SELECT … FROM absence_period WHERE …` ohne `AND deleted IS NULL`:** Tombstones leaken in Read-Pfade. PITFALLS.md Pitfall-6.
- **`find_overlapping` ohne `logical_id != ?`-Filter beim Update:** Die eigene Row kollidiert mit sich selbst, jedes Update schlägt mit Self-Overlap fehl. PITFALLS.md Pitfall-6 + D-15.
- **`AbsenceCategory::from(ExtraHoursCategory)`-Conversion:** D-03 verbietet das. Saubere Domain-Trennung.
- **Snapshot-Schema-Bump in Phase 1:** CC-07. Bestätigung am Phasenende: kein Diff in `service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`.
- **Booking-Konflikt-Lookup im AbsenceService:** D-08. Phase 1 hat **keine** Edge zu BookingService.
- **DAO-Methode `delete` als hard-delete:** ExtraHours-Vorlage liefert `unimplemented!()` für `delete` (vgl. `dao_impl_sqlite/src/extra_hours.rs:242-249`). Soft-Delete läuft über `update(entity { deleted: Some(now), … })`. Konsistenz halten.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Range-Overlap-Erkennung in Rust | Custom Set-Membership-Logik / Intervaltree | `DateRange::overlaps` + SQL-Allen-Algebra im DAO | Allen's algebra in einer `WHERE`-Klausel ist O(log n) mit dem composite index; jede Rust-Variante zieht alle Kandidaten in Memory. STACK.md "Pattern 1". |
| Self-Overlap-Validierung | Application-Lock + Re-Read | `find_overlapping(sales_person_id, category, range, exclude_logical_id)` in derselben Transaction | Transactional consistency garantiert kein Race; Composite-Index (D-04) deckt das ab. |
| Update-Identität (logical_id) | Counter, expand-by-day, neue Tabelle für History | Tombstone-Pattern mit `partial UNIQUE WHERE deleted IS NULL` (D-04) | Bestehender Standard im Repo (extra_hours, sales_person_unavailable Conv). PITFALLS.md Pitfall-6. |
| Permission-Check pro Methode | Hand-rolled Boolean-AND | `tokio::join!(check_permission(HR), verify_user_is_sales_person(sp_id))` + `.or()` | Bestehender Standard (D-09). |
| DateRange-Type | New-Type um `(time::Date, time::Date)` Tuple inline | `shifty_utils::DateRange` (zentral) | D-16; ein Refactor gegen einen Tuple-Type später kostet mehr als die einmalige Erstellung jetzt. |
| Mock-Erstellung für Tests | Hand-geschriebene Mock-Structs | `#[automock]` auf jedem Trait | mockall ist bereits im Stack; Repo-Konvention (CONVENTIONS.md). |
| Transaction-Boilerplate | Try/catch + manuelles Commit/Rollback | `let tx = self.transaction_dao.use_transaction(tx).await?; … self.transaction_dao.commit(tx).await?;` | Repo-Standard. CC-03. |

**Key insight:** Phase 1 ist **fast ausschließlich Pattern-Application**, nicht Pattern-Erfindung. Die einzigen "neuen" Konstrukte sind (a) die `DateRange`-Utility (sehr klein, STACK.md liefert den Code), (b) die `find_overlapping`-DAO-Methode (Allen-Algebra, STACK.md liefert das SQL), (c) ein neuer `ValidationFailureItem`-Enum-Variant (D-13). Alles andere ist 1:1 von ExtraHours/Booking abgekupfert.

## Common Pitfalls

### Pitfall 1: SQL-Range-Overlap mit half-open Bounds (statt inclusive)

**What goes wrong:** Schreibt man `from_date < probe.to AND to_date > probe.from` (half-open), verliert man Tage am Rand: ein 1-Tages-Range matcht nicht gegen sich selbst, und Same-Day-Bounds (Mo–Mo) werden nicht erkannt.

**Why it happens:** SQL-Tutorials behandeln häufig Datetime-Ranges (mit `[from, to)`-Konvention). Date-Ranges in diesem Repo sind **inclusive beidseitig** (D-05).

**How to avoid:** Inclusive Allen-Idiom benutzen: `existing.from_date <= ?(probe.to) AND existing.to_date >= ?(probe.from)`. STACK.md Pattern 1 ist die Vorlage. Test mit Single-Day-Range absichern: `from = to = 2026-05-15` muss gegen sich selbst überlappen.

**Warning signs:** Self-Overlap-Test schlägt fehl bei `from = to`. Self-Overlap-Test schlägt fehl bei genau aufeinander-grenzenden Ranges (`A: 2026-05-15..2026-05-20`, `B: 2026-05-20..2026-05-25` — die müssen überlappen).

### Pitfall 2: Update kollidiert mit sich selbst (Self-Overlap-Filter fehlt)

**What goes wrong:** Beim Update wird die alte Row noch gefunden (sie ist in der `find_overlapping`-Query noch `deleted IS NULL`), und der Service lehnt das Update ab — als ob es ein anderer Eintrag wäre.

**Why it happens:** D-15 wurde im Plan vergessen. Reine SQL-Logik findet die Self-Row.

**How to avoid:** `find_overlapping(sales_person_id, category, range, exclude_logical_id: Option<Uuid>)` — beim Update wird `Some(request.id)` übergeben, beim Create `None`. SQL: `WHERE … AND (? IS NULL OR logical_id != ?)`. (Achtung: SQLx-`Option<Uuid>` als bytes — Two-Parameter-Pattern ist sauberer: zwei Bind-Slots oder Conditional-Query.)

**Warning signs:** Updates schlagen mit "OverlappingPeriod"-Validation-Error fehl, wenn die Range gar nicht geändert wurde (nur Description). Test `test_update_description_only_succeeds` würde das catchen.

### Pitfall 3: `WHERE deleted IS NULL` in einer Read-Query vergessen

**What goes wrong:** Soft-deleted Tombstones tauchen in Listen auf. `find_overlapping` triggert auf historische soft-deleted Ranges. Update-Pfad findet zwei Rows (alte tombstone + neue active) für denselben `logical_id` und kollidiert.

**Why it happens:** SQL-Strings im DAO sind händisch; CC-05 ist Developer-Discipline. PITFALLS.md Pitfall-6.

**How to avoid:** Code-Review-Pflicht: Jede `query!`/`query_as!` mit `FROM absence_period` MUSS `AND deleted IS NULL` enthalten. Ausnahme: NUR die explizite `update`-DAO-Methode (sie schreibt Tombstones via `UPDATE`-Statement nach `id`). Test: `test_find_overlapping_excludes_soft_deleted` mit fixture, wo eine soft-deleted Row in der Range liegt.

**Warning signs:** Listen-Endpoints zeigen mehr Einträge als erwartet. Update-Pfad scheitert mit `EntityConflicts(version_a, version_b)` wegen zwei "active" Rows.

### Pitfall 4: Snapshot-Schema-Versioning vergessen / fälschlich gebumpt

**What goes wrong:** Phase 1 ist additiv (CC-07). Wird `CURRENT_SNAPSHOT_SCHEMA_VERSION` erhöht, signalisiert das den Validatoren einen Computation-Change, der gar nicht stattgefunden hat — bestehende Snapshots werden invalidiert. Wird er bei einer **echten** Computation-Änderung NICHT erhöht (Phase 2), drift entsteht silent.

**Why it happens:** Phasen-Boundary-Disziplin nicht eingehalten.

**How to avoid:** Verifikations-Schritt am Phase-Ende: `git diff main -- service_impl/src/billing_period_report.rs` muss leer sein bzgl. `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Phase 1 fasst `service_impl/src/billing_period_report.rs`, `service_impl/src/reporting.rs`, `service_impl/src/extra_hours.rs` **gar nicht** an (außer transitive Re-Compile).

**Warning signs:** Phase-1-Diff zeigt geänderte Files in `service_impl/src/billing_period_report.rs` oder `service_impl/src/reporting.rs`.

### Pitfall 5: SQLx compile-time-checked Queries scheitern, weil Migration nicht gelaufen ist

**What goes wrong:** `cargo build` failed mit "table absence_period not found", weil sqlx versucht, gegen die DB im `.env`-`DATABASE_URL` zu validieren — aber die neue Migration ist dort noch nicht angewandt.

**Why it happens:** SQLx-`query!`/`query_as!`-Macros validieren zur Compile-Zeit gegen die im `DATABASE_URL` referenzierte Live-DB.

**How to avoid:**
1. Zuerst Migration committen, dann `sqlx migrate run --source migrations/sqlite` (aus `nix-shell -p sqlx-cli` heraus, CC-10).
2. Alternativ: `cargo sqlx prepare --workspace` und committe `.sqlx/` für Offline-Build (Repo-Konvention zu prüfen).
3. Plan-Phase muss die Migration als **erste** Wave aufnehmen (vor jeder DAO-Implementation).

**Warning signs:** `cargo build` failed mit `error returned from database: (code: 1) no such table: absence_period`.

### Pitfall 6: D-10 Read-Sicht — falsches Privilegien-Modell

**What goes wrong:** Read-Endpoint exposiert entweder zu viel (alle Mitarbeiter sehen alle Absence-Periods) oder zu wenig (kein Mitarbeiter sieht sein eigenes Profil).

**Why it happens:** D-10 ist nur als "HR alle, Mitarbeiter eigene + Schichtplan-Kollegen" beschrieben. Die exakte Idiom-Wahl liegt in der Plan-Phase (siehe §9 Recommendation).

**How to avoid:** Section §9 dieses Dokuments dokumentiert die Recommendation und die zwei Optionen. Plan-Phase wählt explizit eine.

**Warning signs:** Read-Endpoint hat keinen `_forbidden`-Test für "andere Sales-Person". Read-Endpoint sieht keine Schichtplan-Kollegen-Range. Read-Endpoint hat keinen separaten HR-Endpoint.

### Pitfall 7: Reverse-Compatibility-Bruch durch versehentliches Editieren von ExtraHours

**What goes wrong:** Beim Refactoring fasst man `service_impl/src/extra_hours.rs` an (z.B. um den Update-Flow zu DRY-en) und ändert subtil die Semantik. Bestehende Tests bleiben grün, aber Reporting-Snapshot driftet.

**Why it happens:** Code-Templating-Versuchung. Phase 1 ist aber strikt **additiv** (Erfolgskriterium 5).

**How to avoid:** Plan-Tasks dürfen `extra_hours.rs` **nicht** patchen. DRY-Refactor ist out-of-scope. Verifikations-Step: `git diff main -- service_impl/src/extra_hours.rs dao_impl_sqlite/src/extra_hours.rs service/src/extra_hours.rs dao/src/extra_hours.rs rest/src/extra_hours.rs rest-types/src/lib.rs` (für ExtraHoursTO-Bereich) muss leer sein.

**Warning signs:** Diff über `extra_hours`-Files zeigt Änderungen.

### Pitfall 8: Migration-File für SQLite — `ALTER TABLE … ADD CONSTRAINT` nicht unterstützt

**What goes wrong:** `ALTER TABLE absence_period ADD CONSTRAINT chk_absence_period_dates CHECK (to_date >= from_date)` schlägt fehl — SQLite unterstützt ADD CONSTRAINT nicht; CHECKs müssen im `CREATE TABLE` direkt stehen.

**Why it happens:** Tutorials nehmen häufig PostgreSQL-Syntax an.

**How to avoid:** Ein-Statement-`CREATE TABLE` mit inline-`CHECK`-Klausel. Vorlage: SQL in §3.

**Warning signs:** `sqlx migrate run` failed mit "near 'CONSTRAINT': syntax error".

### Pitfall 9: `query_as!` und `Option<Uuid>` als optionaler Parameter

**What goes wrong:** Bei `find_overlapping(…, exclude_logical_id: Option<Uuid>)` will man eine Query schreiben mit `(? IS NULL OR logical_id != ?)`. SQLx-`query!`-Macro erwartet aber kompilierungs-zeit-bekannte Parameter-Bindings, was bei `Option<Vec<u8>>` zu komischen Fehlern führt.

**How to avoid:** Zwei separate SQL-Strings (mit/ohne Filter) oder ein Hilfsparameter — z.B. `nil_uuid` als Sentinel und `(logical_id != ?)` mit `id_vec` (immer gefüllt mit nil-UUID-Bytes wenn nicht zu excludieren). Im Repo-Bestand wird Sentinel-UUID nicht durchgängig benutzt — sauber sind zwei Branches in Rust mit zwei Queries.

**Warning signs:** sqlx-prepare-Schritt scheitert oder Compile-Time-Check des Macros klappt nicht.

## Code Examples

### `DateRange`-Utility (Stencil)

```rust
// shifty-utils/src/date_range.rs (NEU; STACK.md ist die Quelle des Designs)
use thiserror::Error;
use time::Date;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RangeError {
    #[error("Range invalid: from {from} is after to {to}")]
    FromAfterTo { from: Date, to: Date },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateRange {
    from: Date,
    to: Date,
}

impl DateRange {
    pub fn new(from: Date, to: Date) -> Result<Self, RangeError> {
        if from > to {
            return Err(RangeError::FromAfterTo { from, to });
        }
        Ok(Self { from, to })
    }
    pub fn from(&self) -> Date { self.from }
    pub fn to(&self) -> Date { self.to }

    /// Allen's interval algebra (inclusive bounds).
    pub fn overlaps(&self, other: &DateRange) -> bool {
        self.from <= other.to && other.from <= self.to
    }
    pub fn contains(&self, day: Date) -> bool {
        self.from <= day && day <= self.to
    }
    pub fn iter_days(&self) -> impl Iterator<Item = Date> {
        let mut current = Some(self.from);
        let to = self.to;
        std::iter::from_fn(move || {
            let c = current?;
            current = if c == to { None } else { c.next_day() };
            Some(c)
        })
    }
    pub fn day_count(&self) -> u32 {
        ((self.to - self.from).whole_days() + 1).max(0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn new_rejects_inverted() {
        assert!(DateRange::new(date!(2026-05-20), date!(2026-05-15)).is_err());
    }

    #[test]
    fn overlaps_inclusive_single_day() {
        let a = DateRange::new(date!(2026-05-15), date!(2026-05-15)).unwrap();
        assert!(a.overlaps(&a));
    }

    #[test]
    fn overlaps_touching_boundary_overlaps() {
        let a = DateRange::new(date!(2026-05-15), date!(2026-05-20)).unwrap();
        let b = DateRange::new(date!(2026-05-20), date!(2026-05-25)).unwrap();
        assert!(a.overlaps(&b));    // bei inclusive bounds: gemeinsamer Tag = Overlap
    }

    #[test]
    fn iter_days_emits_inclusive_count() {
        let r = DateRange::new(date!(2026-05-15), date!(2026-05-17)).unwrap();
        assert_eq!(r.iter_days().count(), 3);
        assert_eq!(r.day_count(), 3);
    }

    // … weitere Tests (Jahresgrenze, Schaltjahr, leerer Range = Single-Day) …
}
```

### Service-Trait (komplette Phase-1-Surface)

```rust
// service/src/absence.rs (NEU)
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DateRange;
use time::Date;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbsenceCategory {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

impl From<&dao::absence::AbsenceCategoryEntity> for AbsenceCategory {
    fn from(c: &dao::absence::AbsenceCategoryEntity) -> Self {
        match c {
            dao::absence::AbsenceCategoryEntity::Vacation => Self::Vacation,
            dao::absence::AbsenceCategoryEntity::SickLeave => Self::SickLeave,
            dao::absence::AbsenceCategoryEntity::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}
impl From<&AbsenceCategory> for dao::absence::AbsenceCategoryEntity {
    fn from(c: &AbsenceCategory) -> Self {
        match c {
            AbsenceCategory::Vacation => Self::Vacation,
            AbsenceCategory::SickLeave => Self::SickLeave,
            AbsenceCategory::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriod {
    /// Externally stable id == DAO logical_id. Equals the physical row id of the first version.
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategory,
    pub from_date: Date,
    pub to_date: Date,
    pub description: Arc<str>,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&dao::absence::AbsencePeriodEntity> for AbsencePeriod {
    fn from(e: &dao::absence::AbsencePeriodEntity) -> Self {
        Self {
            id: e.logical_id,                 // Domain-id == logical_id
            sales_person_id: e.sales_person_id,
            category: (&e.category).into(),
            from_date: e.from_date,
            to_date: e.to_date,
            description: e.description.clone(),
            created: Some(e.created),
            deleted: e.deleted,
            version: e.version,
        }
    }
}
impl TryFrom<&AbsencePeriod> for dao::absence::AbsencePeriodEntity {
    type Error = ServiceError;
    fn try_from(a: &AbsencePeriod) -> Result<Self, Self::Error> {
        Ok(Self {
            id: a.id,
            logical_id: a.id,                  // bei first version
            sales_person_id: a.sales_person_id,
            category: (&a.category).into(),
            from_date: a.from_date,
            to_date: a.to_date,
            description: a.description.clone(),
            created: a.created.ok_or(ServiceError::InternalError)?,
            deleted: a.deleted,
            version: a.version,
        })
    }
}

impl AbsencePeriod {
    pub fn date_range(&self) -> Result<DateRange, ServiceError> {
        DateRange::new(self.from_date, self.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(self.from_date, self.to_date))
    }
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait AbsenceService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// HR sees all (D-10).
    async fn find_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;

    /// HR + sales-person-self + Schichtplan-colleagues (D-10).
    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;

    /// Find by id (logical_id). HR + sales-person-self.
    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn create(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn update(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
```

### DAO-Trait (komplette Phase-1-Surface)

```rust
// dao/src/absence.rs (NEU)
use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DateRange;
use time::Date;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbsenceCategoryEntity {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriodEntity {
    pub id: Uuid,
    pub logical_id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategoryEntity,
    pub from_date: Date,
    pub to_date: Date,
    pub description: Arc<str>,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait AbsenceDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, crate::DaoError>;

    async fn find_by_logical_id(
        &self,
        logical_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, crate::DaoError>;

    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    async fn find_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    /// Find active rows for a sales person and category that overlap `range`.
    /// `exclude_logical_id` is used during update so the row being edited
    /// does not collide with itself (D-15).
    async fn find_overlapping(
        &self,
        sales_person_id: Uuid,
        category: AbsenceCategoryEntity,
        range: DateRange,
        exclude_logical_id: Option<Uuid>,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    async fn create(
        &self,
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;

    /// Soft-delete or audit-update (writes `deleted`, `update_version`, `update_process` by `id`).
    async fn update(
        &self,
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
}
```

### Schema-Migration (komplettes SQL)

Siehe §3.

### Self-Overlap Validation Channel

Siehe §8.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ExtraHours pro Tag (1 Row pro Datum mit Stundenzahl) | AbsencePeriod als Range (1 Row pro Zeitraum) | Phase 1 (additiv), Reporting-Switch in Phase 2 | Doppelte Eintragung (`sales_person_unavailable` + ExtraHours für Vacation/Sick) entfällt ab Phase 3. |
| ExtraHours-Update = direktes Mutieren | logical_id-Pattern (tombstone + neue Row) | Bestand seit Migration `20260428101456_…` (April 2026) | Audit-Trail per `logical_id`-Historie statt verlorener Update-Spuren. **Phase 1 erbt das 1:1.** |
| `WHERE deleted IS NULL` als Developer-Discipline | Partial unique indexes `WHERE deleted IS NULL` für `logical_id` | Bestand seit `20260428101456_…` | Schema-erzwungene Garantie "max. 1 active Row pro `logical_id`". |
| Snapshot-Schema-Versioning implizit | `CURRENT_SNAPSHOT_SCHEMA_VERSION u32`-Konstante in `service_impl::billing_period_report` (CC-07) | Bestand seit Migration `20260426000000_…` | **Phase 1 darf die Konstante NICHT bumpen** — Bump kommt in Phase 2. |

**Deprecated/outdated:**
- Keiner der eingesetzten Patterns ist deprecated. Phase 1 verwendet ausschließlich aktuelle Repo-Konventionen.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Die exakte Variante der `ValidationFailureItem`-Erweiterung (`OverlappingPeriod(Uuid)` vs. Reuse von `Duplicate`) ist nicht in CONTEXT festgenagelt. | §8 | Plan-Phase wählt explizit; falls Reuse, dann verliert die UI Kontext über die konfliktierende Logical-ID — sauberer ist eine eigene Variante. [ASSUMED, basierend auf D-13 "alternativ existierende Duplicate-Variante"] |
| A2 | `SalesPersonShiftplanService` hat eine Methode/Idiom, das "Schichtplan-Kollegen einer Sales-Person" ermitteln kann (D-10 Read-Sicht). | §9 | Plan-Phase muss das verifizieren; falls nicht, fällt der Implementierungsweg auf eine alternative DI-Komposition zurück (z.B. `BookingService::find_colleagues_of(sales_person_id)`). [ASSUMED] |
| A3 | `cargo sqlx prepare`-Workflow ist im Repo benutzt — d.h. nach Migrations-Add muss prepare laufen, sonst CI-Build im Offline-Modus failt. | §3, Pitfall 5 | Falls Repo den Live-DB-Check nutzt, ist nur `sqlx migrate run` nötig. Plan-Phase verifiziert via `ls .sqlx/` oder `find . -name "*.json"` im Wurzelverzeichnis. [ASSUMED] |
| A4 | `permission_service.check_permission` erwartet keine spezielle Behandlung für anonyme Requests; Anonymous-Context wird via Middleware vor dem Handler abgewiesen (RestState fordert `Authentication`). | §9 | Plan-Phase prüft `rest/src/lib.rs:forbid_unauthenticated`. [ASSUMED basierend auf `rest/src/lib.rs:564-567`] |
| A5 | Der bestehende Bookings-Pattern (`check_booking_permission` in `service_impl/src/booking.rs:34-68`) ist die geeignete Vorlage für D-10 Read-Operations, falls die Recommendation aus §9 nicht greift. | §9 | Geringes Risiko; das Pattern ist im Repo etabliert. [ASSUMED] |
| A6 | Der `description`-Spalten-Typ in der Migration ist `TEXT` (nullable) — analog ExtraHours. | §3 | Falls Plan-Phase `Required` macht (C-03), wird die Spalte `TEXT NOT NULL DEFAULT ''`. [ASSUMED basierend auf C-03 "C-03 erlaubt beides"] |

**Hinweis für Plan-Phase und discuss-phase:** A1, A2, A3 brauchen aktive Plan-Phase-Entscheidung. A4-A6 sind sichere Default-Annahmen aus dem Repo-Bestand.

---

## §3 Schema & Migration

### 3.1 Migrations-File (NEU)

**Pfad:** `migrations/sqlite/<timestamp>_create-absence-period.sql` — Timestamp-Format `YYYYMMDDHHMMSS` (Konvention; siehe `20260428101456_add-logical-id-to-extra-hours.sql`).

**Generierung:** `nix-shell -p sqlx-cli --run "sqlx migrate add create-absence-period --source migrations/sqlite"` (CC-10).

```sql
-- migrations/sqlite/<timestamp>_create-absence-period.sql

-- Schema for the new range-based absence domain (Phase 1).
-- Strikt additiv: keine Änderungen an extra_hours, billing_period o.ä.

CREATE TABLE absence_period (
    -- Identity
    id              BLOB(16) NOT NULL PRIMARY KEY,
    logical_id      BLOB(16) NOT NULL,

    -- Foreign key
    sales_person_id BLOB(16) NOT NULL,

    -- Domain fields
    category        TEXT NOT NULL,                 -- "Vacation" | "SickLeave" | "UnpaidLeave"
    from_date       TEXT NOT NULL,                 -- ISO-8601 YYYY-MM-DD
    to_date         TEXT NOT NULL,                 -- ISO-8601 YYYY-MM-DD; both inclusive
    description     TEXT,                          -- nullable; service treats NULL as ""

    -- Audit
    created         TEXT NOT NULL,                 -- ISO-8601 datetime
    deleted         TEXT,                          -- soft-delete tombstone marker
    update_timestamp TEXT,                         -- audit (matches extra_hours pattern)
    update_process  TEXT NOT NULL,                 -- audit (matches extra_hours pattern)
    update_version  BLOB(16) NOT NULL,             -- optimistic-concurrency UUID

    -- Defense-in-depth: range invariant at DB level (D-04, D-14).
    CHECK (to_date >= from_date),

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

-- Partial UNIQUE: at most one active row per logical_id (Pitfall 6).
CREATE UNIQUE INDEX idx_absence_period_logical_id_active
    ON absence_period(logical_id)
    WHERE deleted IS NULL;

-- Hot path: "absences for sales person, anchored on from_date".
-- Used by find_by_sales_person and the booking-conflict / shift-plan-marking
-- queries that arrive in Phase 3.
CREATE INDEX idx_absence_period_sales_person_from
    ON absence_period(sales_person_id, from_date)
    WHERE deleted IS NULL;

-- Hot path: "self-overlap detection (sales_person + category + range)".
-- find_overlapping(sales_person, category, …) prunes on this prefix, then
-- range-scans on from_date.
CREATE INDEX idx_absence_period_self_overlap
    ON absence_period(sales_person_id, category, from_date)
    WHERE deleted IS NULL;
```

**Verifikations-Notizen:**

- **CHECK-Constraint inline:** SQLite unterstützt `ADD CONSTRAINT` nicht nachträglich (Pitfall 8). Daher inline.
- **`BLOB(16)` für UUID:** Repo-Konvention (siehe `20260428101456_…`). UUIDs werden als 16-byte BLOBs gespeichert; `Uuid::as_bytes().to_vec()` ist der Bind-Pfad in SQLx-Queries.
- **`update_timestamp` nullable, `update_process` NOT NULL:** Kopiert aus dem ExtraHours-Schema (siehe `20260428101456_…` Zeile 18-19). `update_timestamp` wird im Repo-Bestand offenbar nicht aktiv geschrieben (siehe `dao_impl_sqlite/src/extra_hours.rs:230-240` — nur `deleted`, `update_version`, `update_process` werden mit der `update`-Methode geschrieben). Plan-Phase darf entscheiden, `update_timestamp` aus dem Schema wegzulassen, falls keine Code-Verwendung. Empfehlung: dabei behalten für Konsistenz.
- **Indexes:** Drei Stück (D-04 + Hot-Path für Phase 3). Composite mit `category` ist im D-04-Wortlaut nicht explizit, aber per C-01 (find_overlapping kategorie-scoped) der korrekte Index. Der reine `(sales_person_id, from_date)`-Index aus D-04 deckt die Read-Pfade `find_by_sales_person` ab.
- **Snapshot-Versioning:** Diese Migration berührt **nicht** `billing_period`-Tabellen — kein Phase-2-Bump nötig. CC-07 ist gewahrt.

### 3.2 Migration-Run-Sequenz (für die Plan-Phase)

1. Migration committen.
2. `nix-shell -p sqlx-cli --run "sqlx migrate run --source migrations/sqlite"` gegen die Dev-DB (aus `.env:DATABASE_URL`).
3. Falls Repo Offline-Mode benutzt (`.sqlx/`-Verzeichnis vorhanden): `cargo sqlx prepare --workspace` und `.sqlx/`-Diff committen.
4. **Erst dann** kann `cargo build` erfolgreich gegen DAO-Code mit `query_as!(AbsencePeriodDb, …)` validieren.

[CITED: `shifty-backend/CLAUDE.md` "Database Setup" + `shifty-backend/CLAUDE.local.md` "nix-shell für sqlx"]

---

## §4 DateRange API (Detail)

| Symbol | Signatur | Verhalten | Phase |
|--------|----------|-----------|-------|
| `DateRange` | `pub struct DateRange { from: time::Date, to: time::Date }` | Inclusive both bounds. Nicht-public Felder, Konstruktor erzwingt Invariante. | 1 |
| `DateRange::new` | `pub fn new(from: Date, to: Date) -> Result<Self, RangeError>` | `Err(FromAfterTo { from, to })` falls `from > to`. `from == to` ist erlaubt (Single-Day-Range). | 1 |
| `DateRange::from` | `pub fn from(&self) -> Date` | Getter. | 1 |
| `DateRange::to` | `pub fn to(&self) -> Date` | Getter. | 1 |
| `DateRange::overlaps` | `pub fn overlaps(&self, other: &DateRange) -> bool` | Allen-Algebra: `self.from <= other.to && other.from <= self.to`. **Touching boundaries gelten als overlap** (inclusive). | 1 |
| `DateRange::contains` | `pub fn contains(&self, day: Date) -> bool` | `self.from <= day && day <= self.to`. | 1 |
| `DateRange::iter_days` | `pub fn iter_days(&self) -> impl Iterator<Item = Date>` | Inclusive-Iterator: emittiert `from`, `from+1`, …, `to`. Endet **nach** `to`. | 2 (Vorhanden ab Phase 1, genutzt ab Phase 2) |
| `DateRange::day_count` | `pub fn day_count(&self) -> u32` | `to - from + 1` (inclusive). | 2 (Vorhanden ab Phase 1, genutzt ab Phase 2) |
| `RangeError::FromAfterTo` | enum-variant `{ from: Date, to: Date }` | Einzige Variante in Phase 1. | 1 |

**Kollisions-Check:** `shifty-utils/src/lib.rs` exportiert aktuell `date_utils::*` (`ShiftyDate`, `ShiftyWeek`, `DayOfWeek`). **Kein Konflikt** mit `DateRange`. Plan-Phase patcht `lib.rs` mit `pub mod date_range; pub use date_range::*;`.

**Tests (Mindest-Coverage in `date_range.rs::tests`):**
- `new_rejects_inverted_range`
- `new_accepts_single_day`
- `overlaps_single_day_with_self`
- `overlaps_touching_boundary` (inclusive!)
- `overlaps_disjoint_returns_false`
- `contains_endpoints_inclusive`
- `iter_days_emits_inclusive_count` (3 Tage = 3 Items)
- `iter_days_year_boundary` (Dec-30 → Jan-2 = 4 Tage; Schaltjahr-Variante)
- `day_count_inclusive`

[VERIFIED: API-Surface basiert exakt auf D-16; STACK.md liefert die Implementierungs-Skizze; Test-Set ist Standard-Pattern für inclusive Range-Types]

---

## §5 AbsenceService Trait Shape

Siehe Code-Beispiel oben unter "Service-Trait (komplette Phase-1-Surface)".

**Methoden-Surface (vollständig für Phase 1):**

| Method | Permission-Pfad | Verhalten | `_forbidden`-Test? (D-11) |
|--------|-----------------|-----------|---------------------------|
| `find_all` | HR only (vgl. `BookingService::get_all`) | Listet aktive Absence-Periods aller Mitarbeiter. | ✓ Sales-Person-User ohne HR → Forbidden |
| `find_by_sales_person` | D-10: HR ∨ self ∨ Schichtplan-Kollege | Listet aktive Absence-Periods eines Mitarbeiters. | ✓ Fremde Sales-Person ohne HR/Schichtplan-Beziehung → Forbidden |
| `find_by_id` | HR ∨ self (über `find_by_logical_id` → `sales_person_id` → `verify_user_is_sales_person`) | Liest eine bestimmte Absence-Period. | ✓ |
| `create` | HR ∨ verify_user_is_sales_person(entity.sales_person_id) | Range-Check, Self-Overlap-Check, Insert. | ✓ |
| `update` | HR ∨ verify_user_is_sales_person(active.sales_person_id) | logical_id-Update-Flow (D-07) inkl. Self-Overlap-Check mit `exclude_logical_id` | ✓ |
| `delete` | HR ∨ verify_user_is_sales_person(active.sales_person_id) | Soft-Delete via `update(entity { deleted: now, … })`. | ✓ |

**Validation-Pflichten (in jeder Schreibmethode):**

1. `entity.id == Uuid::nil()` (create) bzw. `!= Uuid::nil()` (update) — `IdSetOnCreate`/`EntityNotFound`.
2. `entity.version == Uuid::nil()` (create) — `VersionSetOnCreate`.
3. `entity.created.is_none()` (create) — Repo-Pattern; sonst `CreatedSetOnCreate`.
4. `entity.deleted.is_none()` (create) — `DeletedSetOnCreate`.
5. `DateRange::new(from, to)` — bei Err → `ServiceError::DateOrderWrong` (Repo-Idiom).
6. `find_overlapping(sales_person_id, category, range, exclude_logical_id)` empty? — sonst `ValidationError([OverlappingPeriod(conflict.logical_id)])`.

**Test-Matrix per Methode (`service_impl/src/test/absence.rs`):**

| Test | Method | Coverage |
|------|--------|----------|
| `test_create_success` | create | Happy-Path inkl. UUID/version/created-Generierung |
| `test_create_id_set_returns_error` | create | `IdSetOnCreate` |
| `test_create_version_set_returns_error` | create | `VersionSetOnCreate` |
| `test_create_inverted_range_returns_date_order_wrong` | create | `DateOrderWrong` |
| `test_create_self_overlap_same_category_returns_validation` | create | `OverlappingPeriod` |
| `test_create_self_overlap_different_category_succeeds` | create | D-12 (Vacation + SickLeave dürfen überlappen) |
| `test_create_other_sales_person_without_hr_is_forbidden` | create | `_forbidden` (CC-09 + D-11) |
| `test_update_success_soft_deletes_old_inserts_new` | update | Tombstone + neue Row |
| `test_update_changing_sales_person_id_is_rejected` | update | `ModificationNotAllowed("sales_person_id")` |
| `test_update_stale_version_returns_conflict` | update | `EntityConflicts` |
| `test_update_unknown_logical_id_returns_not_found` | update | `EntityNotFound` |
| `test_update_self_overlap_excludes_self` | update | D-15 (eigene alte Row darf nicht kollidieren) |
| `test_update_other_sales_person_without_hr_is_forbidden` | update | `_forbidden` |
| `test_update_can_change_category` | update | D-06 (Vacation→SickLeave erlaubt; neue Self-Overlap-Detection auf neue Kategorie) |
| `test_delete_success_soft_deletes` | delete | `deleted` ist gesetzt nach delete |
| `test_delete_other_sales_person_without_hr_is_forbidden` | delete | `_forbidden` |
| `test_find_by_id_returns_active_row` | find_by_id | Happy-Path |
| `test_find_by_id_unknown_returns_not_found` | find_by_id | `EntityNotFound` |
| `test_find_by_id_other_without_permission_is_forbidden` | find_by_id | `_forbidden` (D-10 enforcement) |
| `test_find_by_sales_person_self_succeeds` | find_by_sales_person | D-10 self-path |
| `test_find_by_sales_person_other_without_permission_forbidden` | find_by_sales_person | D-10 deny-path |
| `test_find_all_hr_succeeds` | find_all | HR-Vollsicht |
| `test_find_all_non_hr_forbidden` | find_all | `_forbidden` |

---

## §6 AbsenceDao Trait Shape

Siehe Code-Beispiel oben unter "DAO-Trait (komplette Phase-1-Surface)".

**Methoden-Surface (vollständig für Phase 1):**

| Method | Signature | SQL-Skelett |
|--------|-----------|-------------|
| `find_by_id` | `(id, tx) -> Option<Entity>` | `SELECT … WHERE id = ? AND deleted IS NULL` |
| `find_by_logical_id` | `(logical_id, tx) -> Option<Entity>` | `SELECT … WHERE logical_id = ? AND deleted IS NULL` |
| `find_by_sales_person` | `(sales_person_id, tx) -> Arc<[Entity]>` | `SELECT … WHERE sales_person_id = ? AND deleted IS NULL` |
| `find_all` | `(tx) -> Arc<[Entity]>` | `SELECT … WHERE deleted IS NULL` |
| `find_overlapping` | `(sales_person_id, category, range, exclude_logical_id, tx) -> Arc<[Entity]>` | siehe §7 |
| `create` | `(&Entity, &str process, tx) -> ()` | `INSERT INTO absence_period (…) VALUES (?, ?, …)` |
| `update` | `(&Entity, &str process, tx) -> ()` | `UPDATE absence_period SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?` |

**Notiz zu `update`:** Wie bei ExtraHours (`dao_impl_sqlite/src/extra_hours.rs:218-241`) wird `update` auf das Schreiben der Soft-Delete-Spalten reduziert. Reine Mutationen am Domain-Body werden nicht unterstützt — der Service-Layer rotiert immer via `update(tombstone) + create(neu)`. **D-07 erbt das.**

**Notiz zu `delete`:** Ich empfehle, **kein** `delete` im Trait zu haben. ExtraHours hat es deklariert aber `unimplemented!()` (siehe `dao_impl_sqlite/src/extra_hours.rs:242-249`). Sauberer: gar nicht im Trait. Der Service ruft `update(tombstone)` auf. Plan-Phase entscheidet final.

---

## §7 find_overlapping SQL Pattern

### 7.1 Korrekte SQL-Form (inclusive Allen)

**Two queries** (für `Some(exclude)` und `None(exclude)`-Pfad), oder eine Query mit Sentinel — siehe Pitfall 9.

**Variante A: Zwei Queries (empfohlen für SQLx-Compile-Time-Check):**

```rust
// dao_impl_sqlite/src/absence.rs (Skizze für find_overlapping)
async fn find_overlapping(
    &self,
    sales_person_id: Uuid,
    category: AbsenceCategoryEntity,
    range: DateRange,
    exclude_logical_id: Option<Uuid>,
    tx: Self::Transaction,
) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError> {
    let sp_vec = sales_person_id.as_bytes().to_vec();
    let category_str = category_to_str(&category);              // "Vacation"|"SickLeave"|"UnpaidLeave"
    // ISO-8601 YYYY-MM-DD; lex-sort == date-sort
    let from_str = range.from().format(&Iso8601::DATE)?;
    let to_str = range.to().format(&Iso8601::DATE)?;

    let rows = match exclude_logical_id {
        Some(exclude) => {
            let exclude_vec = exclude.as_bytes().to_vec();
            query_as!(
                AbsencePeriodDb,
                r#"
                SELECT id, logical_id, sales_person_id, category,
                       from_date, to_date, description, created, deleted, update_version
                  FROM absence_period
                 WHERE sales_person_id = ?
                   AND category = ?
                   AND from_date <= ?     -- existing.from <= probe.to
                   AND to_date   >= ?     -- existing.to   >= probe.from
                   AND logical_id != ?    -- D-15: exclude self during update
                   AND deleted IS NULL
                "#,
                sp_vec,
                category_str,
                to_str,
                from_str,
                exclude_vec,
            )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
        }
        None => {
            query_as!(
                AbsencePeriodDb,
                r#"
                SELECT id, logical_id, sales_person_id, category,
                       from_date, to_date, description, created, deleted, update_version
                  FROM absence_period
                 WHERE sales_person_id = ?
                   AND category = ?
                   AND from_date <= ?
                   AND to_date   >= ?
                   AND deleted IS NULL
                "#,
                sp_vec,
                category_str,
                to_str,
                from_str,
            )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
        }
    };

    rows.iter().map(AbsencePeriodEntity::try_from).collect::<Result<Arc<[_]>, _>>()
}
```

### 7.2 Korrektheits-Analyse (warum inclusive Allen)

- D-05 sagt **inclusive bounds beidseitig**.
- D-12 sagt: gleicher Mitarbeiter + gleiche Kategorie + überlappender Zeitraum = Konflikt.
- Inclusive Allen: A.from ≤ B.to ∧ B.from ≤ A.to. Das ergibt für `A = [2026-05-15, 2026-05-15]` (single day) und `B = [2026-05-15, 2026-05-15]` Overlap, was korrekt ist.
- Für `A = [2026-05-15, 2026-05-20]` und `B = [2026-05-20, 2026-05-25]` ergibt es Overlap am Tag 2026-05-20, was bei inclusive bounds erwartet ist.
- Die Frage "Vacation am Mo–Fr und SickLeave am Wed" ist dank D-12 (Filter `category = ?`) **kein** Konflikt.

### 7.3 Performance-Analyse

- Index `idx_absence_period_self_overlap (sales_person_id, category, from_date) WHERE deleted IS NULL` deckt:
  - Equality auf `sales_person_id`
  - Equality auf `category`
  - Range-Lookup `from_date <= probe.to`
- `to_date >= probe.from` ist Post-Filter — akzeptabel, weil die Vorfilterung über die ersten drei Spalten die Kandidatenmenge stark reduziert.
- `EXPLAIN QUERY PLAN` muss `SEARCH absence_period USING INDEX idx_absence_period_self_overlap` zeigen — wenn nicht, ist der Index falsch oder die Query-Reihenfolge passt nicht.

[VERIFIED: STACK.md "Pattern 1: Find absences overlapping a given range" + PITFALLS.md Pitfall 5]

---

## §8 Self-Overlap Validation Channel

### Recommendation: Neue `ValidationFailureItem`-Variante `OverlappingPeriod(Uuid)`

**Begründung:**

- `Duplicate` ist contextless — der Caller weiß nicht, **welche** Row kollidiert.
- `OverlappingPeriod(Uuid)` mit der `logical_id` des konfliktierenden AbsencePeriods erlaubt der UI (zukünftiges Dioxus-Frontend), den Konflikt direkt verlinkbar darzustellen.
- Phase 3 (BOOK-01) wird das Forward-Warning-Pattern auf Bookings ausweiten — die parallele Erweiterung könnte z.B. `OverlappingBooking(Uuid)` nutzen. Eine sprechende Variante reduziert spätere Refactor-Kosten.

**Patch-Skizze für `service/src/lib.rs`:**

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationFailureItem {
    ModificationNotAllowed(Arc<str>),
    InvalidValue(Arc<str>),
    IdDoesNotExist(Arc<str>, Uuid),
    Duplicate,
    OverlappingPeriod(Uuid),       // NEU in Phase 1 (D-13)
}
```

**REST-Mapping (kein Change nötig):**

`error_handler` (`rest/src/lib.rs:174-179`) mappt `ServiceError::ValidationError(_)` → HTTP 422. Der konflikthafte `Uuid` kommt im Error-Body (über `Debug`-Format des `ValidationFailureItem`-Vec). Falls UI strukturierten JSON-Body erwartet, muss Phase 3 das ggf. via custom-`Display`-Impl serialisieren — out of scope für Phase 1.

**Alternative: `Duplicate` mit Kontext-String** (CONTEXT.md D-13 erwähnt das):
- Pro: kein Schema-Bruch in `ValidationFailureItem`.
- Contra: Kontext geht durch String-Format verloren; UI muss parsen.

**Plan-Phase soll **explizit** entscheiden** — A1 in Assumptions Log.

[VERIFIED: `service/src/lib.rs:48-54` + `rest/src/lib.rs:174-179`]

---

## §9 Permission Patterns for Reads (D-10 Detail)

### Problem-Statement

D-10: "HR sieht alle, Mitarbeiter ohne HR-Rechte sieht eigene Einträge plus die der Schichtplan-Kollegen."

Die schwierige Frage: **Wie wird "Schichtplan-Kollege" formal definiert?** Der Begriff ist im Repo nicht direkt als Service-API vorhanden — Bookings und Slots haben Beziehungen zu Sales-Persons über den Shiftplan, aber keine "Sind-Person-A-und-Person-B-Kollegen?"-Abfrage.

### Recommendation: Dual-Endpoint-Strategie + verfeinerte Permission im find_by_sales_person

**Architektur:**

| Endpoint | Permission-Pfad | Wer? |
|----------|----------------|------|
| `GET /absence-period` (HR-Vollsicht) | `check_permission(HR_PRIVILEGE)` only | Vorstand/HR |
| `GET /absence-period/by-sales-person/{id}` | HR ∨ verify_user_is_sales_person(id) ∨ {Schichtplan-Kollege-Verifikation} | HR oder Self oder Kollege |
| `GET /absence-period/{id}` | HR ∨ verify_user_is_sales_person(active.sales_person_id) ∨ {Schichtplan-Kollege-Verifikation} | HR oder Self (nicht Kollege — Detail einzelner Period ist self/HR-only) |

**Schichtplan-Kollege-Verifikation (zwei Sub-Optionen für die Plan-Phase):**

1. **Option A (defensiv, Phase-1-konform):** "Schichtplan-Kollege"-Begriff wird in Phase 1 **bewusst noch nicht** umgesetzt; D-10-Read-Sicht reduziert sich auf "HR ∨ self". Phase 2 oder Phase 3 erweitert. Plan-Phase notiert das als Discuss-Phase-Carryover.

2. **Option B (Ausarbeitung in Plan-Phase):** Define "Schichtplan-Kollege von SP_X" als "Sales-Person mit mindestens einer Booking auf einem Slot, der auch von SP_X ein Booking hat im selben Zeitraum". Implementierung delegiert an `BookingService` oder `SalesPersonShiftplanService` — **A2 in Assumptions Log: Plan-Phase muss Service-Surface verifizieren**.

**Recommended:** **Option A für Phase 1**, Option B für Phase 3 (parallel zu BOOK-01/PLAN-01, wo das Konzept ohnehin gebraucht wird). Diese Empfehlung minimiert Phase-1-Scope-Creep und vermeidet, dass Phase 1 von einem nicht-existenten Service-API abhängt.

**Code-Skizze für Option A (`find_by_sales_person`):**

```rust
async fn find_by_sales_person(
    &self,
    sales_person_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    let (hr_permission, sales_person_permission) = join!(
        self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(sales_person_id, context, tx.clone().into()),
    );
    hr_permission.or(sales_person_permission)?;

    let entities = self.absence_dao.find_by_sales_person(sales_person_id, tx.clone()).await?;
    let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

**Code-Skizze für `find_all` (HR-Vollsicht):**

```rust
async fn find_all(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    self.permission_service.check_permission(HR_PRIVILEGE, context).await?;
    let entities = self.absence_dao.find_all(tx.clone()).await?;
    let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

[VERIFIED: Patterns abgeleitet aus `service_impl/src/extra_hours.rs:113-148` (find-with-self-or-hr) und `service_impl/src/booking.rs:76-98` (HR-only)]

---

## §10 REST Endpoints

### Routing

```rust
// rest/src/absence.rs (NEU)
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_absence_period::<RestState>))
        .route("/", get(get_all_absence_periods::<RestState>))                          // HR only
        .route("/{id}", get(get_absence_period::<RestState>))                           // HR ∨ self
        .route("/{id}", put(update_absence_period::<RestState>))
        .route("/{id}", delete(delete_absence_period::<RestState>))
        .route("/by-sales-person/{sales_person_id}", get(get_absence_periods_for_sales_person::<RestState>))
}
```

### Endpoint-Surface

| Method | Path | Service-Call | Status-Codes |
|--------|------|--------------|--------------|
| `POST` | `/absence-period` | `create(&AbsencePeriod, ctx, None)` | 201 (Created), 400 (range invalid), 403, 422 (validation incl. self-overlap) |
| `GET` | `/absence-period` | `find_all(ctx, None)` | 200, 403 |
| `GET` | `/absence-period/{id}` | `find_by_id(id, ctx, None)` | 200, 403, 404 |
| `PUT` | `/absence-period/{id}` | `update(&AbsencePeriod { id: path_id, … }, ctx, None)` | 200, 403, 404, 409, 422 |
| `DELETE` | `/absence-period/{id}` | `delete(id, ctx, None)` | 204, 403, 404 |
| `GET` | `/absence-period/by-sales-person/{sales_person_id}` | `find_by_sales_person(sp_id, ctx, None)` | 200, 403 |

**Query-Params (C-02):** Plan-Phase darf optionale Filter `?from=YYYY-MM-DD&to=YYYY-MM-DD&category=Vacation` zur Liste hinzufügen — analog `ExtraHoursForSalesPersonAttributes` aus `rest/src/extra_hours.rs:31-39`. Empfehlung: in Phase 1 nur das nötige Minimum (`/`, `/{id}`, `/by-sales-person/{sp_id}`), ohne zusätzliche Filter. Filter in Phase 3 nachziehen.

### Transport Object (Inline in `rest-types/src/lib.rs` oder eigene Datei)

```rust
// rest-types/src/absence_period_to.rs (NEU; alternativ inline in lib.rs gemäß Repo-Konvention)
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use time::Date;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AbsenceCategoryTO {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsenceCategory> for AbsenceCategoryTO { /* … */ }
#[cfg(feature = "service-impl")]
impl From<&AbsenceCategoryTO> for service::absence::AbsenceCategory { /* … */ }

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AbsencePeriodTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategoryTO,
    #[schema(value_type = String, format = "date")]
    pub from_date: Date,
    #[schema(value_type = String, format = "date")]
    pub to_date: Date,
    #[serde(default)]
    pub description: Arc<str>,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsencePeriod> for AbsencePeriodTO { /* … */ }
#[cfg(feature = "service-impl")]
impl From<&AbsencePeriodTO> for service::absence::AbsencePeriod { /* … */ }
```

### ApiDoc-Eintrag

```rust
// rest/src/absence.rs (NEU)
#[derive(OpenApi)]
#[openapi(
    paths(
        create_absence_period,
        get_all_absence_periods,
        get_absence_period,
        update_absence_period,
        delete_absence_period,
        get_absence_periods_for_sales_person,
    ),
    components(schemas(AbsencePeriodTO, AbsenceCategoryTO)),
    tags(
        (name = "Absence", description = "Absence period management (range-based)"),
    ),
)]
pub struct AbsenceApiDoc;

// rest/src/lib.rs PATCH (im nest-Block der ApiDoc):
//   (path = "/absence-period", api = absence::AbsenceApiDoc),
```

[VERIFIED: Pattern aus `rest/src/extra_hours.rs:194-207` und `rest/src/lib.rs:457-481`]

---

## §11 Test Architecture

### 11.1 Service-Unit-Tests (Mock-basiert)

**Pfad:** `service_impl/src/test/absence.rs` (NEU)

**Setup:** Adaptiert von `service_impl/src/test/extra_hours.rs:33-156`:

```rust
struct AbsenceDependencies {
    absence_dao: MockAbsenceDao,
    permission_service: MockPermissionService,
    sales_person_service: MockSalesPersonService,
    sales_person_shiftplan_service: MockSalesPersonShiftplanService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    transaction_dao: MockTransactionDao,
}

impl AbsenceServiceDeps for AbsenceDependencies { /* … */ }

impl AbsenceDependencies {
    fn build_service(self) -> AbsenceServiceImpl<AbsenceDependencies> { /* … */ }
}

fn build_dependencies() -> AbsenceDependencies {
    // Defaults: HR + verify_user → Ok(()), clock fixed, transaction trivial
}
```

**Mindest-Test-Umfang:** siehe Test-Matrix in §5. Total ~22 Tests.

**Konventions-Helper:** `error_test::test_forbidden`, `test_conflicts`, `test_not_found`, `test_validation_error`, `test_date_order_wrong` — alle verfügbar in `service_impl/src/test/error_test.rs`.

### 11.2 Integration-Test (echte In-Memory-SQLite)

**Pfad:** `shifty_bin/src/integration_test/absence_period.rs` (NEU)

**Vorlage:** `shifty_bin/src/integration_test/extra_hours_update.rs` (komplett, 100% adaptierbar).

**Mindest-Szenarien:**

1. `test_create_assigns_id_equal_to_logical_id` — analog ExtraHours.
2. `test_update_creates_tombstone_and_new_active_row` — analog ExtraHours.
3. `test_partial_unique_index_enforces_one_active_per_logical_id` — schreibe direkt zwei Rows mit gleichem `logical_id` und `deleted IS NULL` → SQLx-Error erwartet.
4. `test_create_overlapping_same_category_returns_validation_error` — Self-Overlap echte Detection in DB.
5. `test_create_overlapping_different_category_succeeds` — D-12.
6. `test_update_can_extend_range_without_self_collision` — D-15 in echter DB.
7. `test_delete_softdeletes_row` — `deleted IS NOT NULL` nach delete; `find_by_id` → None.
8. `test_check_constraint_rejects_inverted_range` — direkter SQL-INSERT mit `to_date < from_date` → DB-CHECK greift.

### 11.3 Unit-Tests in shifty-utils (DateRange)

**Pfad:** `shifty-utils/src/date_range.rs::tests`

Mindest-Tests siehe §4.

### 11.4 Bestehende-Test-Suite-Stabilität (Erfolgskriterium 5)

**Verifikation am Phase-Ende:**

```bash
cargo test --workspace 2>&1 | tee phase-1-test.log
# Erwartung: alle bestehenden Tests grün, neue Tests grün, kein Diff in Pass-Counts auf
# bestehende Tests (vergleiche mit cargo test --workspace VOR Phase 1).
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `tokio` 1.44 + `mockall` 0.13 + Rust-Standard-Test-Harness |
| Config file | none (Cargo defaults) |
| Quick run command | `cargo test -p service_impl test::absence` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ABS-01 | Entity persistiert, Soft-Delete-Spalte, logical_id-Spalte | integration | `cargo test -p shifty_bin integration_test::absence_period::test_create_assigns_id_equal_to_logical_id` | ❌ Wave 0 |
| ABS-01 | DB-CHECK lehnt invertierten Range ab | integration | `cargo test -p shifty_bin integration_test::absence_period::test_check_constraint_rejects_inverted_range` | ❌ Wave 0 |
| ABS-01 | Partial unique index erzwingt max. 1 aktive Row pro logical_id | integration | `cargo test -p shifty_bin integration_test::absence_period::test_partial_unique_index_enforces_one_active_per_logical_id` | ❌ Wave 0 |
| ABS-02 | DAO `find_by_logical_id` filtert `deleted IS NULL` | unit (mock) + integration | `cargo test -p service_impl test::absence::test_update_unknown_logical_id_returns_not_found` | ❌ Wave 0 |
| ABS-02 | DAO `find_overlapping` Allen-inclusive | integration | `cargo test -p shifty_bin integration_test::absence_period::test_create_overlapping_same_category_returns_validation_error` | ❌ Wave 0 |
| ABS-02 | DAO `find_overlapping` honors `exclude_logical_id` | integration | `cargo test -p shifty_bin integration_test::absence_period::test_update_can_extend_range_without_self_collision` | ❌ Wave 0 |
| ABS-03 | Service Range-Validierung (`from > to` → DateOrderWrong) | unit | `cargo test -p service_impl test::absence::test_create_inverted_range_returns_date_order_wrong` | ❌ Wave 0 |
| ABS-03 | Service Self-Overlap auf Same-Category | unit | `cargo test -p service_impl test::absence::test_create_self_overlap_same_category_returns_validation` | ❌ Wave 0 |
| ABS-03 | Service Self-Overlap exkludiert eigene Row beim Update | unit | `cargo test -p service_impl test::absence::test_update_self_overlap_excludes_self` | ❌ Wave 0 |
| ABS-03 | Service Cross-Category darf überlappen (D-12) | unit | `cargo test -p service_impl test::absence::test_create_self_overlap_different_category_succeeds` | ❌ Wave 0 |
| ABS-03 | Update logical_id-Pattern (tombstone + neue Row) | integration | `cargo test -p shifty_bin integration_test::absence_period::test_update_creates_tombstone_and_new_active_row` | ❌ Wave 0 |
| ABS-04 | REST Routes vorhanden, OpenAPI registriert | manual + integration (smoke) | `cargo build && cargo run -- --check-api` (Plan-Phase entscheidet, oder via Swagger-UI manuell) | ❌ Wave 0 |
| ABS-04 | DTO Round-Trip serializes/deserializes | unit | `cargo test -p rest-types absence_period_to` (falls Tests vorhanden, sonst Plan-Phase ergänzt) | ❌ Wave 0 |
| ABS-05 | `_forbidden`-Test pro Methode | unit | `cargo test -p service_impl test::absence::test_*_forbidden` | ❌ Wave 0 |
| ABS-05 | HR ∨ Self-Pattern beim Create | unit | `cargo test -p service_impl test::absence::test_create_other_sales_person_without_hr_is_forbidden` | ❌ Wave 0 |
| (additivity) | Bestehende ExtraHours-Tests bleiben grün | regression | `cargo test --workspace -- --skip absence` (oder kompletter Run mit Pre/Post-Vergleich) | ✅ existing |
| (additivity) | Snapshot-Schema-Versioning unverändert | manual | `git diff main -- service_impl/src/billing_period_report.rs \| grep -i version` MUSS leer sein | ✅ check via git |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl test::absence` (~22 Tests, < 5 s)
- **Per wave merge:** `cargo test -p service_impl && cargo test -p shifty_bin integration_test::absence_period`
- **Phase gate:** `cargo test --workspace` muss komplett grün vor `/gsd-verify-work` (CC-08).

### Wave 0 Gaps

- [ ] `migrations/sqlite/<timestamp>_create-absence-period.sql` — Schema-Vorlage für DB-Pflichten (CHECK + Indexes).
- [ ] `shifty-utils/src/date_range.rs` — Inkl. inline `#[cfg(test)] mod tests`.
- [ ] `dao/src/absence.rs` — Trait mit `#[automock]` (nötig für Service-Unit-Tests).
- [ ] `dao_impl_sqlite/src/absence.rs` — SQLx-Queries (nötig für Integration-Tests).
- [ ] `service/src/absence.rs` — Trait + Domain-Modell (nötig für Service-Tests).
- [ ] `service_impl/src/absence.rs` — Implementation (testet sich gegen Mock-DAO).
- [ ] `service_impl/src/test/absence.rs` — Test-Modul.
- [ ] `service_impl/src/test/mod.rs` — `pub mod absence;` Patch.
- [ ] `rest/src/absence.rs` — Handlers + ApiDoc.
- [ ] `rest-types/src/lib.rs` (oder `absence_period_to.rs`) — `AbsencePeriodTO`/`AbsenceCategoryTO`.
- [ ] `rest/src/lib.rs` — Router-Patch + ApiDoc-Nest.
- [ ] `shifty_bin/src/main.rs` — `AbsenceServiceDependencies`-Block + Arc-Init + RestStateImpl.
- [ ] `shifty_bin/src/integration_test/absence_period.rs` — End-to-End-Tests.
- [ ] `service/src/lib.rs` — `pub mod absence;` Patch + ggf. neue `ValidationFailureItem::OverlappingPeriod(Uuid)`-Variante.
- [ ] Framework-Install: keiner — alle Deps bereits da.

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (transitive) | OIDC / Mock-Auth durch bestehende Middleware (`rest/src/lib.rs:564-567`); Phase 1 fügt nur `Authentication<Context>`-Durchreichung hinzu. |
| V3 Session Management | yes (transitive) | Bestehende Session-Implementation (`service::session`); Phase 1 berührt das nicht. |
| V4 Access Control | yes | RBAC via `PermissionService::check_permission(HR_PRIVILEGE)` + `verify_user_is_sales_person`. **Pflicht-Pattern in jeder Phase-1-Methode.** Default-deny via `error_handler` → 401/403. |
| V5 Input Validation | yes | Range-Validation im Service (`DateRange::new`), DB-CHECK als Defense-in-Depth, `IdSetOnCreate`/`VersionSetOnCreate`/`CreatedSetOnCreate`/`DeletedSetOnCreate` Vorbedingungen. |
| V6 Cryptography | partial | UUID v4 wird per `UuidService` zentralisiert generiert; **kein** Hand-rolled Crypto. |

### Known Threat Patterns for Rust/Axum/SQLite/sqlx

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection auf `category` (TEXT-Spalte) | Tampering | sqlx parameterized binding (`query_as!` macro); enum-Mapping in `try_from` rejects unknown values via `DaoError::EnumValueNotFound`. |
| Privilege escalation via `sales_person_id`-Spoofing in Update-Body | Elevation | `update`-Flow lädt aktive Row aus DB und vergleicht `request.sales_person_id` gegen `active.sales_person_id` (D-06 explizit immutable). Mismatch → `ValidationError(ModificationNotAllowed("sales_person_id"))`. |
| Soft-deleted Row "Resurrection" via direkten DB-Zugriff | Tampering | Phase 1 hat keinen Resurrection-Pfad. Soft-deleted Tombstones bleiben tombstones (PITFALLS.md Pitfall 6). |
| Unauthenticated Request schreibt Absence | Spoofing | `forbid_unauthenticated`-Middleware (`rest/src/lib.rs:564-567`) wirft 401 vor jedem Handler. |
| GDPR-Risk: SickLeave-Daten leaken in logs | Information Disclosure | Plan-Phase sollte `tracing::debug!`-statt-`info!`-Konvention für Body-Inhalte erwägen (PITFALLS.md security mistakes). Phase 1 fügt **keinen** neuen Logging-Pfad hinzu, der `description` oder `category` ins Log schreibt. |
| Unbounded Range (`to_date = 9999-12-31`) | DoS / Logical Bug | Plan-Phase darf optional eine Range-Size-Sanity in `create`/`update` einbauen (z.B. `day_count() <= 365` ohne Override). [DEFERRED zu Plan-Phase entscheiden — nicht in CONTEXT.md, also Discretion] |

[VERIFIED: STACK.md "Why date-only" + PITFALLS.md "Security Mistakes" + bestehende Auth-Middleware in rest/src/lib.rs]

---

## Risks & Open Questions for Plan-Phase

### Risiken (geringer-zu-hoch sortiert)

| Risiko | Impact | Mitigation in Plan-Phase |
|--------|--------|--------------------------|
| Migration-Reihenfolge-Bruch (DAO-Compile vor migrate run) | High | Plan ordnet Migration als **erste** Wave 0-Task ein, vor jeder DAO-Datei. CC-10. |
| Self-Overlap-SQL ohne `exclude_logical_id`-Filter | Medium-High | DAO-Trait hat `exclude_logical_id: Option<Uuid>` als Pflicht-Param; D-15-Test (`test_update_self_overlap_excludes_self`) catcht Verstöße. |
| `ValidationFailureItem`-Erweiterung kollidiert mit anderen Phasen | Low | Neue Variante `OverlappingPeriod(Uuid)` ist additiv; kein Bestand-Konsument bricht. |
| Snapshot-Bump aus Versehen | Low | Phase-Ende-Verification: `git diff` prüft `service_impl/src/billing_period_report.rs`. |
| D-10 Read-Sicht missverstanden | Medium | Recommendation §9 Option A (HR ∨ self only in Phase 1) ist konservativ; Plan-Phase sollte das explizit ratifizieren. |
| `cargo sqlx prepare` vergessen → CI-Bruch | Medium | Plan-Phase fügt `cargo sqlx prepare --workspace` als Wave 1-End-Task hinzu, falls Repo `.sqlx/`-Cache benutzt. |

### Open Questions für Plan-Phase (RESOLVED)

1. **`ValidationFailureItem::OverlappingPeriod(Uuid)` vs. `Duplicate`-Reuse?** — Recommendation: neue Variante. (A1) **RESOLVED:** A1 in `01-VALIDATION.md` → neue Variante `ValidationFailureItem::OverlappingPeriod(Uuid)` (Plan 01-00 Task 0.3).
2. **D-10 Read-Sicht in Phase 1 vollständig oder als HR ∨ self only?** — Recommendation: HR ∨ self only; "Schichtplan-Kollege"-Erweiterung in Phase 3. (A2) **RESOLVED:** A2 in `01-VALIDATION.md` → Phase 1 = HR ∨ self only; Schichtplan-Kollegen-Erweiterung deferred to Phase 3 (Plan 01-02 Task 2.2, KEIN `SalesPersonShiftplanService`-Dependency).
3. **`description`-Pflichtigkeit?** (C-03) — Recommendation: `Arc<str>` mit leerem Default (analog ExtraHours). **RESOLVED:** C-03 in `01-CONTEXT.md` → Domain `Arc<str>` mit leerem String als Default; DTO `Arc<str>` mit `#[serde(default)]` (Plan 01-02 Task 2.1, Plan 01-03 Task 3.1).
4. **Filter-Query-Params auf `GET /absence-period` und `GET /by-sales-person/{id}`?** (C-02) — Recommendation: keine in Phase 1; Filter ist Phase 3. **RESOLVED:** C-02 in `01-CONTEXT.md` (Anti-Pattern in Plan 01-03 Task 3.2: KEIN Filter-Query-Param `?from=&to=&category=`); Phase-1-Carry-Over auf Phase 3.
5. **`cargo sqlx prepare`-Workflow in CI?** — Plan-Phase muss verifizieren; entweder `.sqlx/`-Diff als zusätzlicher Commit oder Live-DB-Check (DATABASE_URL). **RESOLVED:** A3 in `01-VALIDATION.md` → `cargo sqlx prepare --workspace` als Pflicht-Schritt nach Migration und nach DAO-Impl (Plan 01-00 Task 0.1, Plan 01-01 Task 1.2).
6. **`DateRange::new` API: Method oder freie Funktion?** — Recommendation: assoziierte Funktion `DateRange::new`, Konstruktor ist Standard. **RESOLVED:** D-16 in `01-CONTEXT.md` → `DateRange::new(from, to) -> Result<DateRange, RangeError>` als assoziierte Funktion (Plan 01-00 Task 0.2).
7. **`update_timestamp`-Spalte im Schema beibehalten oder weglassen?** — Recommendation: beibehalten für Konsistenz mit ExtraHours, auch wenn aktuell nicht aktiv geschrieben. **RESOLVED:** D-04/D-05 in `01-CONTEXT.md` → Schema folgt ExtraHours-Konvention; `update_process` und `update_version` werden geschrieben (Plan 01-00 Task 0.1).
8. **Range-Size-Sanity (z.B. `to - from <= 365 Tage`)?** — Discretion. Recommendation: nicht in Phase 1 (PITFALLS.md security #4); gegebenenfalls in Phase 3 nachziehen, wenn HR-UI darum bittet. **RESOLVED:** Deferred to Phase 3 per CONTEXT.md Deferred Ideas / Discuss-Phase-Carryovers; Phase 1 implementiert keine Range-Size-Sanity (Plan 01-02 Task 2.2 enthält keinen entsprechenden Check).

### Discuss-Phase-Carryovers (aus ROADMAP.md, für Phase 2/3 weiterführen)

- Sick-overlapping-Vacation Policy (BUrlG §9) → Phase 2 (REP).
- Liste der berührten `value_type`s im Snapshot → Phase 2 (SNAP).

---

## Sources

### Primary (HIGH confidence — verifiziert im Repo)

- `.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — alle D-Decisions D-01..D-17, C-01..C-05.
- `.planning/REQUIREMENTS.md` — ABS-01..ABS-05 (Wortlaut).
- `.planning/ROADMAP.md` — Phase-1-Goal, Success-Criteria, Discuss-Carry-Overs.
- `.planning/research/STACK.md` — Stack-Begründung, `time::Date`-Mapping, Allen-Algebra-Pattern, Schema-Skizze.
- `.planning/research/PITFALLS.md` — Pitfall 5 (Indexes), Pitfall 6 (Soft-Delete + logical_id), Pitfall 9 (Permission-Contract).
- `.planning/codebase/CONVENTIONS.md` — Naming, Soft-Delete-Konvention, ServiceError-Surface.
- `.planning/codebase/STRUCTURE.md` — `Where to Add New Code`-Sektion (New-Domain-Recipe), `gen_service_impl!`.
- `service/src/extra_hours.rs:185-235` — Trait-Surface-Vorlage.
- `service/src/lib.rs:48-117` — `ValidationFailureItem`, `ServiceError`-Varianten.
- `service/src/sales_person.rs:124-129` — `verify_user_is_sales_person`-Methode.
- `service/src/permission.rs:1-80` — `Authentication<Context>`, `HR_PRIVILEGE`, `SHIFTPLANNER_PRIVILEGE`, `SALES_PRIVILEGE`.
- `service_impl/src/extra_hours.rs:22-348` — DI-Macro-Block, Update-Flow (Zeilen 220-301), Permission-Pattern (Zeilen 236-245), Read-with-self-or-hr (Zeilen 113-148).
- `service_impl/src/booking.rs:20-180` — `gen_service_impl!` mit mehreren Deps; HR-only-`get_all` (Zeilen 76-98).
- `service_impl/src/sales_person_unavailable.rs:30-150` — Read-Sicht-Pattern (`SHIFTPLANNER ∨ verify_user`).
- `service_impl/src/test/extra_hours.rs:1-450` — Mock-Setup, `_forbidden`-Pattern.
- `service_impl/src/test/error_test.rs:1-138` — Test-Helper (`test_forbidden`, `test_validation_error`, …).
- `dao/src/extra_hours.rs:1-87` — DAO-Trait-Vorlage.
- `dao/src/lib.rs:1-89` — DaoError, Transaction-Trait.
- `dao_impl_sqlite/src/extra_hours.rs:1-251` — SQLx-Patterns, `TryFrom<&Db>` für Entity, Soft-Delete-Update.
- `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql` — Partial-Unique-Index-Vorlage, Schema-Form.
- `migrations/sqlite/20260330000000_add-shiftplan-table.sql` — Schema-Style (BLOB(16), update_process, update_version).
- `rest/src/extra_hours.rs:1-207` — REST-Handler-Vorlage, ApiDoc.
- `rest/src/lib.rs:120-251` — `error_handler`, HTTP-Status-Mapping.
- `rest/src/lib.rs:457-553` — `ApiDoc` nest, Router-Setup.
- `rest-types/src/lib.rs:741-789` — `ExtraHoursTO`-DTO-Pattern mit `ToSchema`.
- `shifty-utils/src/date_utils.rs:1-340` — bestehende `ShiftyDate`-Konvention; Bestätigung dass kein Konflikt mit `DateRange`.
- `shifty-utils/src/lib.rs:1-78` — Utility-Crate-Surface.
- `shifty_bin/src/main.rs:144-236` — `BookingServiceDependencies`-Block, `ExtraHoursServiceDependencies`-Block.
- `shifty_bin/src/main.rs:670-688` — Konkrete `Arc::new(ExtraHoursServiceImpl { … })`.
- `shifty_bin/src/integration_test.rs:266-300` — `TestSetup::new()` mit `sqlx::migrate!()`.
- `shifty_bin/src/integration_test/extra_hours_update.rs:1-100` — Integration-Test-Vorlage.

### Secondary (MEDIUM confidence)

- SQLite Optimizer Overview ([CITED: sqlite.org/optoverview.html] — composite index "left to right, no skipping"); kein direktes Test, aber STACK.md zitiert.
- Allen's interval algebra ([CITED: salman-w.blogspot.com] — applied to SQL); STACK.md verifiziert das gegen Repo-Bestand `dao_impl_sqlite/src/billing_period.rs:16-17`.

### Tertiary (LOW confidence)

- Keine. Alle Empfehlungen basieren auf Repo-Inspektion oder verifizierten Research-Outputs aus `.planning/research/`.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — alle Versionen aus `Cargo.toml` gelesen.
- Architecture: HIGH — gespiegelt 1:1 von ExtraHours / Booking.
- DAO-SQL: HIGH — Allen-inclusive ist Standard; Index-Strategie aus PITFALLS.md.
- Pitfalls: HIGH — alle aus PITFALLS.md verifiziert oder direkt aus Repo-Code.
- Permission-Pattern Read (D-10): MEDIUM — Recommendation §9 Option A ist konservativ; Option B wartet auf Plan-Phase-Discretion.
- ValidationFailureItem-Variant (D-13): MEDIUM — Plan-Phase entscheidet zwischen `OverlappingPeriod(Uuid)` und `Duplicate`-Reuse.

**Research date:** 2026-05-01
**Valid until:** 2026-06-01 (Phase 1 ist additiv; State-of-the-Art-Stack ist stabil; mögliche Drifts: `time` 0.3.x patch-bumps — nicht-kritisch.)

---

## RESEARCH COMPLETE

**Confirmed (alle High-Confidence):**
- Schema-Migration-SQL (CHECK + 3 partial indexes), DAO-Trait + SQLx-Queries (inkl. inclusive-Allen find_overlapping mit `exclude_logical_id`), Service-Trait mit 6 Methoden + 22 Mindest-Tests, REST-Surface mit 6 Routen + utoipa-ApiDoc-Patch, `DateRange`-Utility-API, DI-Verdrahtung in main.rs, Test-Vorlagen aus extra_hours für Mock-Unit + Integration mit echter In-Memory-SQLite.
- Phase 1 ist strikt additiv: keine Diffs in `service_impl/src/billing_period_report.rs`/`reporting.rs`/`extra_hours.rs`/`booking.rs`; CC-07 (Snapshot-Versioning) ist gewahrt.

**Open items für Plan-Phase (Discretion-Entscheidungen):**
- A1: `ValidationFailureItem::OverlappingPeriod(Uuid)` (empfohlen) vs. `Duplicate`-Reuse.
- A2: D-10 Read-Sicht — Option A (HR ∨ self only) für Phase 1, Schichtplan-Kollege auf Phase 3 verschieben (empfohlen) vs. Option B (volle D-10-Implementierung in Phase 1).
- A3: `cargo sqlx prepare`-Workflow im Repo verifizieren und Plan-Wave-Reihenfolge anpassen.
