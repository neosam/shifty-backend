---
phase: 01-absence-domain-foundation
verified: 2026-05-01T18:11:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: n/a
  gaps_closed: []
  gaps_remaining: []
  regressions: []
---

# Phase 1: Absence Domain Foundation — Verification Report

**Phase Goal:** Eine neue, parallele `absence` Domain existiert end-to-end (Schema, DAO, Service, REST, DI), permission-gated, ohne Auswirkung auf Reporting/Snapshots/Booking-Flows. Entwickler können Absences anlegen, lesen, ändern und (soft-)löschen; alle Tests grün.
**Verified:** 2026-05-01T18:11:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | HR/Vorstand kann via REST `AbsencePeriod` (Vacation/Sick/UnpaidLeave) mit `from_date`/`to_date` anlegen, abrufen, ändern und soft-löschen — unabhängig vom ExtraHours-Pfad | VERIFIED | 6 REST-Handler in `rest/src/absence.rs` (POST/GET-list/GET-by-id/PUT/DELETE/by-sales-person), DI in `shifty_bin/src/main.rs:225-238` + `:703-710`, Router-Nest in `rest/src/lib.rs:543`, Integration-Test `test_create_assigns_id_equal_to_logical_id` + `test_update_creates_tombstone_and_new_active_row` + `test_delete_softdeletes_row` (alle grün); Domain-Trennung von ExtraHours: eigenes `AbsenceCategory`-Enum (`service/src/absence.rs:27`), keine Conversion zu/von `ExtraHoursCategory` |
| 2 | Mitarbeiter ohne HR-Rechte erhält bei jeder schreibenden Operation auf fremde `AbsencePeriod`s `403 Forbidden`; `_forbidden`-Tests pro public service method | VERIFIED | 6 `_forbidden`-Tests in `service_impl/src/test/absence.rs` (`test_create_*_forbidden`, `test_update_*_forbidden`, `test_delete_*_forbidden`, `test_find_by_id_*_forbidden`, `test_find_by_sales_person_*_forbidden`, `test_find_all_non_hr_is_forbidden`); Permission-Pattern `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(...))` mit `or` in `service_impl/src/absence.rs` für find_by_sales_person/find_by_id/create/update/delete; `find_all` ist HR-only |
| 3 | Self-Overlap (gleicher Mitarbeiter + gleiche Kategorie + überlappender Zeitraum) wird vom Service erkannt und als `ServiceError`-Variante zurückgewiesen | VERIFIED | `ValidationFailureItem::OverlappingPeriod(Uuid)` in `service/src/lib.rs:57` (D-13 A1-Pinning); `service_impl/src/absence.rs:166-170` und `:239-243` mappen Self-Overlap auf `ServiceError::ValidationError([OverlappingPeriod(conflicts[0].logical_id)])`; Two-Branch find_overlapping in `dao_impl_sqlite/src/absence.rs:155-204` mit Allen-Inclusive-Bounds (`from_date <= ?` x2, `to_date >= ?` x2) und `logical_id != ?`-Filter im Some-Branch (D-15); Mock-Test `test_create_self_overlap_same_category_returns_validation` + `test_update_self_overlap_excludes_self` (Predicate `eq(Some(default_logical_id()))`) + Integration-Test `test_create_overlapping_same_category_returns_validation_error` + D-12 `test_create_overlapping_different_category_succeeds` + D-15 `test_update_can_extend_range_without_self_collision` |
| 4 | `cargo test` und `cargo build` grün; Integration-Test in `shifty_bin/src/integration_test/` deckt CRUD-Round-Trip einschließlich Soft-Delete + `logical_id`-Update-Pfad | VERIFIED | `cargo build --workspace`: exit 0; `cargo test --workspace`: 381 passed / 0 failed (10 dao + 8 service + 316 service_impl + 13 shifty_utils + 34 shifty_bin = 381); `shifty_bin/src/integration_test/absence_period.rs` enthält 8 `#[tokio::test]`-Funktionen: CRUD (`test_create_assigns_id_equal_to_logical_id`), logical_id-Update (`test_update_creates_tombstone_and_new_active_row` — verifiziert 2 Rows mit gleicher logical_id, 1 Tombstone + 1 aktiv), Soft-Delete (`test_delete_softdeletes_row` — verifiziert via direktem SQL und EntityNotFound), Schema-Constraints (`test_check_constraint_rejects_inverted_range`, `test_partial_unique_index_enforces_one_active_per_logical_id`), Self-Overlap (`test_create_overlapping_same_category_returns_validation_error`), D-12 (`test_create_overlapping_different_category_succeeds`), D-15 (`test_update_can_extend_range_without_self_collision`); alle grün |
| 5 | Bestehende Reporting-/Booking-/Snapshot-Pfade liefern bit-identische Ergebnisse (Phase ist additiv — bestehende Tests unverändert grün) | VERIFIED | `git diff 6dea4b6..HEAD -- service_impl/src/billing_period_report.rs service_impl/src/reporting.rs service_impl/src/extra_hours.rs service_impl/src/booking.rs dao_impl_sqlite/src/extra_hours.rs dao_impl_sqlite/src/booking.rs rest/src/extra_hours.rs rest/src/booking.rs` ist leer (0 Zeilen); `CURRENT_SNAPSHOT_SCHEMA_VERSION = 2` unverändert in `service_impl/src/billing_period_report.rs`; Bestand-Tests `extra_hours_update::*` und `booking::*` und `billing_period_*` weiterhin grün im 381er-Lauf |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `migrations/sqlite/20260501162017_create-absence-period.sql` | Schema mit CHECK + 3 partial indexes | VERIFIED | 33 Zeilen; `CREATE TABLE absence_period` mit 12 Spalten (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_timestamp, update_process, update_version); `CHECK (to_date >= from_date)` (Z. 18); FK auf sales_person; 3 Partial-Indexes (`idx_absence_period_logical_id_active` UNIQUE, `idx_absence_period_sales_person_from`, `idx_absence_period_self_overlap`) alle mit `WHERE deleted IS NULL` |
| `shifty-utils/src/date_range.rs` | DateRange-Wrapper mit überlappungs-Logik | VERIFIED | 156 Zeilen; `pub struct DateRange { from: Date, to: Date }` (privat); `pub enum RangeError::FromAfterTo`; 7 Methoden (`new`, `from`, `to`, `overlaps`, `contains`, `iter_days`, `day_count`); `overlaps`: `self.from <= other.to && other.from <= self.to` (Inclusive Allen, Z. 41); 8 Unit-Tests grün; Re-Export in `shifty-utils/src/lib.rs:4` |
| `service/src/lib.rs` (`OverlappingPeriod`-Variante) | Erweiterung um `OverlappingPeriod(Uuid)` | VERIFIED | `service/src/lib.rs:57`: `OverlappingPeriod(Uuid)`; bestehende Varianten (`Duplicate`, `ModificationNotAllowed`, `InvalidValue`, `IdDoesNotExist`) unverändert |
| `dao/src/absence.rs` | AbsenceDao-Trait + Entities + Mock | VERIFIED | 156 Zeilen; `pub trait AbsenceDao` mit 7 Methoden (`find_by_id`, `find_by_logical_id`, `find_by_sales_person`, `find_all`, `find_overlapping(.., exclude_logical_id: Option<Uuid>, ..)`, `create`, `update`); `AbsencePeriodEntity` mit 10 Domain-Feldern; `AbsenceCategoryEntity` mit genau 3 Varianten (Vacation/SickLeave/UnpaidLeave); `#[automock]`-generierter `MockAbsenceDao`; 3 Smoke-Tests grün |
| `dao_impl_sqlite/src/absence.rs` | AbsenceDaoImpl mit 7 SQLx-Queries inkl. Two-Branch find_overlapping | VERIFIED | 270 Zeilen; `pub struct AbsenceDaoImpl` + `impl AbsenceDao` mit allen 7 Methoden; 8 SQL-Strings: 6 Reads alle mit `WHERE deleted IS NULL`/`AND deleted IS NULL`; Two-Branch find_overlapping mit Inclusive-Allen (`from_date <= ?` + `to_date >= ?`) und `logical_id != ?` im Some-Branch; `category_to_str`-Helper; 8 sqlx-cache-Files referenzieren `absence_period` |
| `service/src/absence.rs` | AbsenceService-Trait + AbsencePeriod + AbsenceCategory | VERIFIED | 242 Zeilen; `pub trait AbsenceService` mit 6 Methoden; `AbsencePeriod` Domain-Modell mit `id == logical_id`-Semantik (D-07); `AbsenceCategory` mit 3 Varianten + bidirektionalen `From`-Conversions zu `dao::absence::AbsenceCategoryEntity`; `date_range()`-Helper mappt Inversion auf `DateOrderWrong` (D-14); `MockAbsenceService` via `#[automock]`; 4 Smoke-Tests grün |
| `service_impl/src/absence.rs` | AbsenceServiceImpl mit gen_service_impl! + CRUD + Permission + Self-Overlap + logical_id-Update | VERIFIED | 314 Zeilen; `gen_service_impl!`-Block mit Option-A-Deps (6 Stück: AbsenceDao, PermissionService, SalesPersonService, ClockService, UuidService, TransactionDao — KEIN BookingService/SalesPersonShiftplanService/CustomExtraHoursService); 6 Methoden mit Permission-Pattern (HR-or-self via `tokio::join!` + `or`), Range-Validation (`DateRange::new` → `DateOrderWrong`), Self-Overlap-Check (None bei create, `Some(logical_id)` bei update — D-15), logical_id-Update via Tombstone+Insert (D-07), Soft-Delete via `update(tombstone)` |
| `service_impl/src/test/absence.rs` | Mock-basierte Service-Tests inkl. `_forbidden` pro public method | VERIFIED | 25 `#[tokio::test]`-Tests grün (Plan-Mindest: 13); davon 6 `_forbidden`-Tests (1 pro public method); D-15-Strukturtest `test_update_self_overlap_excludes_self` mit `eq(Some(default_logical_id()))`-Predicate |
| `rest-types/src/lib.rs` (AbsencePeriodTO + AbsenceCategoryTO) | Inline-DTOs mit ToSchema + From-Conversions | VERIFIED | Z. 1543-1623; `pub enum AbsenceCategoryTO` mit 3 Varianten (Vacation/SickLeave/UnpaidLeave); `pub struct AbsencePeriodTO` mit `#[schema(value_type = String, format = "date")]` für `from_date`/`to_date` (utoipa-Compat) und `#[serde(rename = "$version")]`; 4 `From`-Impls `#[cfg(feature = "service-impl")]`-gegated (bidirektional Domain ↔ TO) |
| `rest/src/absence.rs` | 6 REST-Handler mit OpenAPI + AbsenceApiDoc | VERIFIED | 252 Zeilen; `generate_route()` mit 6 Routen (POST `/`, GET `/`, GET `/{id}`, PUT `/{id}`, DELETE `/{id}`, GET `/by-sales-person/{sales_person_id}`); 6 `pub async fn`-Handler mit `#[utoipa::path]` (CC-06) + `#[instrument(skip(rest_state))]` + `error_handler`-Wrapper; PUT überschreibt `entity.id = absence_id` (path-id wins, Z. 162); POST → 201, DELETE → 204; `AbsenceApiDoc` mit `components(schemas(AbsencePeriodTO, AbsenceCategoryTO))` |
| `rest/src/lib.rs` (Wiring) | mod absence + RestStateDef-Erweiterung + ApiDoc-Nest + Router-Nest | VERIFIED | Z. 3: `mod absence;`; Z. 296: `type AbsenceService: service::absence::AbsenceService<Context = Context> + Send + Sync + 'static;`; Z. 354: `fn absence_service(&self) -> Arc<Self::AbsenceService>;`; Z. 463: ApiDoc-Nest `(path = "/absence-period", api = absence::AbsenceApiDoc)`; Z. 543: Router-Nest `.nest("/absence-period", absence::generate_route())` |
| `shifty_bin/src/main.rs` (DI) | AbsenceServiceDependencies-Block + RestStateImpl-Wiring | VERIFIED | Z. 7: `absence::AbsenceDaoImpl`-Import; Z. 39: `type AbsenceDao = AbsenceDaoImpl;`; Z. 225-238: `pub struct AbsenceServiceDependencies` + `impl AbsenceServiceDeps` (Option A — exakt 6 Type-Aliases ohne BookingService/SalesPersonShiftplanService/CustomExtraHoursService) + `type AbsenceService = AbsenceServiceImpl<...>`; Z. 464: `absence_service: Arc<AbsenceService>`-Feld; Z. 494: `type AbsenceService = AbsenceService;`; Z. 553-554: `fn absence_service` Methode; Z. 609: `let absence_dao = Arc::new(AbsenceDao::new(pool.clone()));`; Z. 703-710: `let absence_service = Arc::new(...AbsenceServiceImpl { absence_dao, permission_service, sales_person_service, clock_service, uuid_service, transaction_dao });` (6 Felder, KEIN custom_extra_hours_service oder sales_person_shiftplan_service); Z. 886: `absence_service,` in `Self {…}`-Init |
| `shifty_bin/src/integration_test/absence_period.rs` | 8 End-to-End Integration-Tests | VERIFIED | 353 Zeilen; 8 `#[tokio::test]`-Funktionen abdeckend: CRUD-Round-Trip, Schema-Constraints (DB-CHECK + Partial-Unique-Index per Direkt-SQL), Self-Overlap (Same-Category fail + Cross-Category D-12 succeed), D-15 Self-Range-Extension, Soft-Delete; alle grün; benutzen `TestSetup::new()` für In-Memory-SQLite + `Authentication::Full` |
| `shifty_bin/src/integration_test.rs` (Modul-Decl) | `mod absence_period;` | VERIFIED | Datei enthält `mod absence_period;` (verifiziert via grep) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `service::ValidationFailureItem` | `OverlappingPeriod(Uuid)` | enum-Variante in `service/src/lib.rs:57` | WIRED | Variante existiert; `Duplicate`-Variante unverändert; verwendet von `service_impl/src/absence.rs` für Self-Overlap-Reporting |
| `shifty-utils/src/lib.rs` | `DateRange` + `RangeError` | `pub mod date_range; pub use date_range::{DateRange, RangeError};` | WIRED | Re-Exports in Z. 2 + Z. 4; importiert von `dao::absence`, `dao_impl_sqlite::absence`, `service::absence`, `service_impl::absence` |
| `dao_impl_sqlite/src/absence.rs` | `dao::absence::AbsenceDao` | `impl AbsenceDao for AbsenceDaoImpl` | WIRED | Z. 79; alle 7 Methoden implementiert mit `query_as!` (compile-time-checked) |
| `dao_impl_sqlite::find_overlapping` | `absence_period`-Tabelle | `query_as!` Two-Branch (Some/None exclude) | WIRED | 2 Queries mit Allen-Inclusive (`from_date <= ?` + `to_date >= ?`); Some-Branch ergänzt `logical_id != ?` (D-15) |
| `service_impl::AbsenceServiceImpl::create` | `absence_dao.find_overlapping(.., None, ..)` | Self-Overlap-Check vor Insert | WIRED | `service_impl/src/absence.rs:156-170`; bei Konflikt → `ValidationError([OverlappingPeriod(conflicts[0].logical_id)])` |
| `service_impl::AbsenceServiceImpl::update` | `absence_dao.find_overlapping(.., Some(logical_id), ..)` | Self-Overlap mit Self-Exclude (D-15) | WIRED | `service_impl/src/absence.rs:235`; Strukturtest `test_update_self_overlap_excludes_self` enforced via Mock-Predicate |
| `rest/src/lib.rs` | `rest/src/absence.rs` | `mod absence;` + `.nest("/absence-period", absence::generate_route())` + ApiDoc-Nest | WIRED | 3 Anchor-Stellen verifiziert (Z. 3, Z. 463, Z. 543) |
| `rest/src/absence.rs` | `rest_state.absence_service()` | RestStateDef::absence_service() (6 Calls in den 6 Handlern) | WIRED | `let svc = rest_state.absence_service();` in jedem Handler |
| `rest_types::AbsencePeriodTO` | `service::absence::AbsencePeriod` | bidirektional `From<&AbsencePeriod>` + `From<&AbsencePeriodTO>` | WIRED | beide `#[cfg(feature = "service-impl")]`-gegated (rest-types/src/lib.rs:1593, :1609) |
| `shifty_bin::main::RestStateImpl` | `AbsenceServiceImpl` | `absence_service: Arc<AbsenceService>` Feld + Konstruktion mit 6 Deps | WIRED | DI-Block + Konstruktor-Aufruf + Self-Init (3 Stellen verifiziert) |
| `shifty_bin::main::RestStateImpl::absence_service` | `Arc<AbsenceService>` | `self.absence_service.clone()` (Trait-Impl) | WIRED | Methode in Z. 553-554 implementiert |
| `shifty_bin/src/integration_test/absence_period.rs` | `TestSetup::new()` | echte In-Memory-SQLite + `sqlx::migrate!`-Bootstrap | WIRED | 8 Tests rufen `TestSetup::new().await` und nutzen `test_setup.rest_state.absence_service()` für Service-Calls + `test_setup.pool` für direkte SQL-Probes |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `dao_impl_sqlite::AbsenceDaoImpl::find_*` | `absence_period`-Rows | sqlx `query_as!` mit compile-time-checked SQL gegen Migration `20260501162017` | Yes — DB-Query + TryFrom<&AbsencePeriodDb>-Mapping | FLOWING |
| `service_impl::AbsenceServiceImpl::find_by_sales_person` | `Arc<[AbsencePeriod]>` | DAO + `entities.iter().map(AbsencePeriod::from).collect()` | Yes | FLOWING |
| `rest::absence::get_absence_period` | `AbsencePeriodTO` | service `find_by_id` → DAO `find_by_logical_id` → DB | Yes | FLOWING |
| `rest::absence::create_absence_period` | `AbsencePeriodTO` | request body → service `create` → uuid-gen + clock + dao `create` → DB | Yes | FLOWING |
| Integration-Test `test_create_assigns_id_equal_to_logical_id` | physical_id + logical_id (Vec<u8>) | echte In-Memory-SQLite + Service-Layer + direkter SELECT | Yes — Round-Trip per echtem DB-Pool | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Workspace baut sauber | `cargo build --workspace` | exit 0, "Finished `dev` profile" | PASS |
| Alle Workspace-Tests grün | `cargo test --workspace` | 381 passed / 0 failed (10 dao + 8 service + 316 service_impl + 13 shifty_utils + 34 shifty_bin) | PASS |
| Phase-1 Service-Unit-Tests grün | `cargo test -p service_impl test::absence` | 25 passed / 0 failed | PASS |
| Phase-1 Integration-Tests grün | `cargo test -p shifty_bin integration_test::absence_period` | 8 passed / 0 failed (alle Pflicht-Tests aus Plan 01-04 ausgewiesen) | PASS |
| DateRange-Unit-Tests grün | `cargo test -p shifty-utils date_range` | 8 passed / 0 failed | PASS |
| Server bootet ohne Panic | `timeout 12s cargo run --bin shifty_bin` | Exit 0; Log enthält `INFO Running server at 127.0.0.1:3000`; `tokio_cron` aktiv | PASS |
| Snapshot-Schema-Version unverändert | `grep CURRENT_SNAPSHOT_SCHEMA_VERSION service_impl/src/billing_period_report.rs` | `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 2;` | PASS |
| Additivität: keine Änderung an Bestand-Reporting/Booking/Snapshot/ExtraHours | `git diff 6dea4b6..HEAD -- service_impl/src/{billing_period_report,reporting,extra_hours,booking}.rs dao_impl_sqlite/src/{extra_hours,booking}.rs rest/src/{extra_hours,booking}.rs` | leer (0 Zeilen) | PASS |
| sqlx-Offline-Cache enthält absence_period-Queries | `grep -l absence_period .sqlx/*.json | wc -l` | 8 Cache-Files | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| ABS-01 | 01-00, 01-01, 01-02, 01-03, 01-04 | Entity `AbsencePeriod` mit Feldern (id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted), Soft-Delete-Semantik | SATISFIED | `dao::absence::AbsencePeriodEntity` mit 10 Feldern, `service::absence::AbsencePeriod` mit `id == logical_id`-Mapping, Migration `20260501162017_create-absence-period.sql` schreibt Tabelle mit allen Spalten + 3 Partial-Indexes für Soft-Delete-Awareness; Soft-Delete via `update(tombstone)`-Pattern verifiziert in `test_delete_softdeletes_row` |
| ABS-02 | 01-01, 01-04 | DAO-Trait + SQLite-Impl mit CRUD, sqlx-compile-time-checked, `WHERE deleted IS NULL`-Konvention | SATISFIED | `dao::absence::AbsenceDao` (7 Methoden inkl. find_overlapping mit exclude_logical_id Option<Uuid>); `dao_impl_sqlite::absence::AbsenceDaoImpl` mit `query_as!`/`query!` (sqlx-compile-time-checked, 8 cache-Files); jede Read-Query enthält `WHERE deleted IS NULL` oder `AND deleted IS NULL` (verifiziert via grep) |
| ABS-03 | 01-00, 01-02, 01-04 | Service-Trait + Impl mit Range-Validierung (`from_date <= to_date`), Self-Overlap-Detection per (Mitarbeiter+Kategorie), Permission-Check, Transaction-Pattern via `Option<Transaction>` | SATISFIED | `service::absence::AbsenceService` Trait + `service_impl::absence::AbsenceServiceImpl`; Range-Validierung via `DateRange::new` → `DateOrderWrong` in create/update; Self-Overlap-Check via `find_overlapping` (None=create, Some(logical_id)=update D-15); Permission HR ∨ self via `tokio::join!`; alle 6 Methoden akzeptieren `Option<Self::Transaction>` |
| ABS-04 | 01-03, 01-04 | REST-Endpunkte (POST, GET-list, GET-by-id, PATCH/PUT, DELETE) mit OpenAPI-Annotation, ToSchema-Derive in rest-types | SATISFIED | 6 Routen unter `/absence-period` (POST, GET, GET-by-id, PUT, DELETE, GET-by-sales-person); jeder Handler mit `#[utoipa::path]` (CC-06) + `#[instrument]`; `AbsenceApiDoc` mit `components(schemas(AbsencePeriodTO, AbsenceCategoryTO))`; beide TOs mit `#[derive(... ToSchema)]` in `rest-types/src/lib.rs` |
| ABS-05 | 01-02, 01-04 | Permission-Check via `PermissionService` integriert; HR/Vorstand und Mitarbeiter selbst dürfen anlegen/ändern (Vertrauensbasis) | SATISFIED | HR-or-self-Pattern in `service_impl/src/absence.rs` für find_by_sales_person/find_by_id/create/update/delete via `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(...))` mit `or`; `find_all` HR-only; 6 `_forbidden`-Tests verifizieren Forbidden-Pfad strukturell |

**Coverage:** 5/5 ABS-Requirements satisfied. Keine ORPHANED Requirements für Phase 1 (REQUIREMENTS.md mappt ABS-01..05 auf Phase 1, und alle 5 sind durch ≥1 Plan in der `requirements`-Frontmatter abgedeckt).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (keine) | — | Phase 1 ist additiv; keine TODOs/FIXMEs/Stubs in den neuen Modulen | — | — |

Stichproben in `dao/src/absence.rs`, `dao_impl_sqlite/src/absence.rs`, `service/src/absence.rs`, `service_impl/src/absence.rs`, `rest/src/absence.rs`, `shifty_bin/src/integration_test/absence_period.rs` ergaben keine TODO-/FIXME-/Placeholder-Marker und keine `unimplemented!()`-Calls (DAO hat bewusst KEINE delete-Methode — Soft-Delete läuft über update(tombstone), Plan-explizit dokumentiert).

### Human Verification Required

Keine. Alle 5 Success-Criteria sind durch automatisierte Tests (`cargo test --workspace` 381/0, `cargo build --workspace` 0, `cargo run` boot OK) und Code-Inspektion (DAO-Trait, Service-Impl, REST-Layer, DI-Block, Integration-Tests) verifiziert. Die einzige potenziell visuelle Prüfung — Swagger-UI rendert die `/absence-period`-Endpoints — ist bereits durch die `#[utoipa::path]`-Annotationen + `AbsenceApiDoc` + ApiDoc-Nest in `rest/src/lib.rs:463` strukturell garantiert (Bestand-OpenAPI-Pfade rendern analog).

### Gaps Summary

Keine Gaps. Phase 1 erfüllt alle 5 ROADMAP-Success-Criteria:
- HR-CRUD per REST ist verdrahtet, getestet (Mock + Integration), und durch Compile-Time-Trait-Bounds garantiert.
- `_forbidden`-Tests existieren für jede der 6 public Service-Methoden (D-11/ABS-05 strukturell verifiziert).
- Self-Overlap wird durch Service-Layer (DateRange + find_overlapping) erkannt und mit der dedizierten `OverlappingPeriod(Uuid)`-Variante zurückgegeben — sowohl Mock-getestet (`test_create_self_overlap_same_category_returns_validation`, `test_update_self_overlap_excludes_self`) als auch Integration-getestet gegen echte SQLite (`test_create_overlapping_same_category_returns_validation_error` + `test_update_can_extend_range_without_self_collision`).
- `cargo test --workspace` und `cargo build --workspace` sind grün; Integration-Tests decken CRUD + Soft-Delete + logical_id-Update + Schema-Constraints (DB-CHECK, Partial-Unique-Index) ab.
- Additivität ist durch leeren `git diff` gegen alle 8 protected Bestand-Files bewiesen; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 2 (CC-07).

Phase 2 (Reporting Integration & Snapshot Versioning) kann starten — alle nötigen Vorlagen (`AbsenceService::find_by_sales_person`, `DateRange::iter_days/day_count`, `OverlappingPeriod`-Variante) sind in stable form vorhanden.

---

_Verified: 2026-05-01T18:11:00Z_
_Verifier: Claude (gsd-verifier)_
