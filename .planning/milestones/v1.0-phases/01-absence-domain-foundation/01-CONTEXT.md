# Phase 1: Absence Domain Foundation - Context

**Gathered:** 2026-05-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Eine neue, parallele `absence`-Domain existiert end-to-end (Schema, DAO, Service, REST, DI), permission-gated, **strikt additiv** — keine Auswirkung auf bestehende Reporting-, Snapshot-, Booking- oder Shift-Plan-Pfade. Entwickler können `AbsencePeriod`-Einträge anlegen, lesen, ändern und (soft-)löschen; alle bestehenden Tests bleiben grün.

**In Scope (Phase 1):**
- `absence_period`-Schema, Migration, Indexe
- `AbsenceDao`-Trait + SQLite-Implementation (CRUD + `find_by_logical_id` + `find_overlapping`)
- `AbsenceService`-Trait + Implementation (Range-Validation, Self-Overlap-Detection, Permission-Gate)
- REST-Endpunkte mit `#[utoipa::path]` (`POST`, `GET`-list, `GET`-by-id, `PATCH`, `DELETE`)
- `AbsencePeriodTO` in `rest-types`
- `DateRange`-Utility in `shifty-utils`
- DI-Verdrahtung in `shifty_bin/src/main.rs`
- `_success`- und `_forbidden`-Tests pro public service method
- Integration-Test in `shifty_bin/src/integration_test/`

**Strikt nicht in Scope (Phase 1):**
- Reporting-Integration (Phase 2 — `derive_hours_for_range`, Snapshot-Bump)
- Booking-Konflikt-Detection / Forward-/Reverse-Warnings (Phase 3)
- Shift-Plan-Integration (Phase 3)
- Migration aus `ExtraHours` (Phase 4)
- Frontend (separater Workstream)

</domain>

<decisions>
## Implementation Decisions

### Naming & Schema-Surface

- **D-01:** Entity-Name ist `AbsencePeriod` (per REQUIREMENTS.md ABS-01). Modul `service/src/absence.rs` und `service_impl/src/absence.rs`. DAO-Modul `dao/src/absence.rs` und `dao_impl_sqlite/src/absence.rs`. REST-Modul `rest/src/absence.rs`. Tabellenname `absence_period`. REST-Pfad `/absence-period`. Transport-Object `AbsencePeriodTO` in `rest-types/src/absence_period_to.rs`.
  **Override 2026-05-01 (Plan-Phase 1, A4 Pinned Discretion):** Transport-Object **inline in `rest-types/src/lib.rs`** statt eigener Datei `absence_period_to.rs`. Begründung: empirische Repo-Konvention (`ls rest-types/src/` zeigt nur `lib.rs`; alle Bestand-DTOs inline). Re-Export-Strategie folgt unverändert (`pub use ... AbsencePeriodTO`). Siehe `01-VALIDATION.md` → Pinned Discretion Items.
- **D-02:** **3 Kategorien** in Phase 1: `Vacation`, `SickLeave`, `UnpaidLeave`. `Holiday`, `Unavailable`, `VolunteerWork`, `ExtraWork`, `CustomExtraHours` bleiben hour-based im bestehenden `ExtraHours`-Pfad (per PROJECT.md Out-of-Scope und v2-CAT-Requirements).
- **D-03:** **Eigene `AbsenceCategory`-Enum** in `service/src/absence.rs` (mit DAO-Spiegel `AbsenceCategoryEntity` in `dao/src/absence.rs`). Kein Reuse von `ExtraHoursCategory` — saubere Domain-Trennung, Compiler garantiert Kategorie-Validität (kein `ExtraWork` in `AbsencePeriod` möglich).
- **D-04:** **Schema-Constraints (Vollpaket per Research):**
  - `CHECK (to_date >= from_date)` als Datenbank-Constraint
  - Partial unique index `(logical_id) WHERE deleted IS NULL` (analog Migration `20260428101456_add-logical-id-to-extra-hours.sql`)
  - Composite index `(sales_person_id, from_date) WHERE deleted IS NULL` für Hot-Overlap-Queries
  - `NOT NULL` auf `from_date`, `to_date`, `category`, `sales_person_id`, `id`, `logical_id`, `created`, `update_version`
  - Soft-Delete via nullable `deleted`-Spalte (Konvention: `WHERE deleted IS NULL` in jeder Read-Query)
- **D-05:** Date-Storage als `TEXT` im ISO-8601 Format (analog `billing_period.from_date_time`/`to_date_time`-Pattern). Inclusive bounds beidseitig (`from_date` und `to_date` zählen beide als Absence-Tag).

### Update-Semantik & logical_id

- **D-06:** **Mutable Felder per `update`/PATCH:** `from_date`, `to_date`, `description`, `category`. **Immutable:** `sales_person_id`, `id` (== `logical_id`). `category`-Mutation erlaubt (Vacation→SickLeave-Umwidmung möglich), aber Self-Overlap-Detection wird beim Update auf die **neue** Kategorie angewendet.
- **D-07:** **logical_id-Pattern 1:1 ExtraHours:** Domain-`id` == DAO-`logical_id`. Update-Flow:
  1. `find_by_logical_id(logical_id, tx)` → erwartet aktive Row, sonst `EntityNotFound`
  2. Permission-Check (HR oder Sales-User-Mapping)
  3. `sales_person_id`-Match-Check (sonst `ValidationError(ModificationNotAllowed("sales_person_id"))`)
  4. `update_version`-Match (sonst `EntityConflicts(logical_id, expected, actual)`)
  5. Soft-delete der alten Row (UPDATE `deleted = now`)
  6. Neue Row mit selber `logical_id`, neuer physical `id`, neuer `update_version`
  7. Commit

  Vorlage: `service_impl/src/extra_hours.rs:220-300`.
- **D-08:** **Booking-Konflikt-Detection ist KEIN Phase-1-Thema.** `AbsenceService::create/update` liefert `Result<AbsencePeriod, ServiceError>` — keinen Warning-Wrapper, keinen `BookingService`-Dependency, keine Booking-Lookups. Wrapper-Type (`AbsencePeriodCreateResult`) und Forward-Warning kommen erst in Phase 3 (BOOK-01) — Plan-Phase 3 entscheidet, ob das ein API-Bruch oder ein additiver Wrapper wird.

### Permission-Modell (ABS-05)

- **D-09:** **Write-Operations (`create`, `update`, `delete`):** `tokio::join!`-Pattern analog `service_impl/src/extra_hours.rs:236-245`:
  ```rust
  let (hr_permission, sales_person_permission) = join!(
      self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
      self.sales_person_service.verify_user_is_sales_person(sales_person_id, context, tx.clone().into()),
  );
  hr_permission.or(sales_person_permission)?;
  ```
  HR oder eigene Sales-Person-Identität reicht. Fremder Mitarbeiter → `ServiceError::Forbidden`. Bestehende Privileg-Konstanten (`HR_PRIVILEGE`) — **kein** neues Privileg.
- **D-10:** **Read-Sicht:** HR sieht alle `AbsencePeriod`-Einträge (alle Mitarbeiter, alle Kategorien). Mitarbeiter ohne HR-Rechte sieht eigene Einträge plus die der Schichtplan-Kollegen (Detail-Ausarbeitung in Plan-Phase basierend auf existierenden Booking-Read-Konventionen). Nicht-eingeloggte Anfragen → `Unauthorized`.
- **D-11:** **`_forbidden`-Test pro public service method** ist Pflicht (Phase-1-Erfolgskriterium 2). Tests prüfen: Mitarbeiter ohne HR-Privileg, der eine Operation auf einer fremden `AbsencePeriod` versucht, erhält `Forbidden`.

### Self-Overlap & Validierung

- **D-12:** **Self-Overlap-Scope:** Per `(sales_person_id, category, range)` — Vacation und SickLeave dürfen denselben Tag überdecken (BUrlG §9-Spirit; Cross-Category-Auflösung ist Phase-2-Reporting-Thema). Service lehnt nur ab, wenn der gleiche Mitarbeiter in der gleichen Kategorie überlappt.
- **D-13:** **Overlap-Error-Variante:** `ServiceError::ValidationError(Arc<[ValidationFailureItem]>)` als Channel. Plan-Phase erweitert `ValidationFailureItem` (in `service/src/lib.rs`) um eine geeignete Variante (z.B. `OverlappingPeriod(Uuid)` mit der ID des konfliktierenden AbsencePeriods) — alternativ existierende `Duplicate`-Variante mit Kontext-String.
- **D-14:** **Range-Validierung** (`from_date <= to_date`) doppelt: einmal im Service (`ServiceError::DateOrderWrong(from, to)` aus `service/src/lib.rs:127`), einmal als DB-CHECK-Constraint (siehe D-04). Defense in depth.
- **D-15:** Self-Overlap wird sowohl bei `create` als auch bei `update` geprüft. Beim Update darf die neue Range mit der eigenen alten Row nicht als Konflikt zählen (Filter: `WHERE logical_id != ?`).

### DateRange-Utility

- **D-16:** **`shifty_utils::DateRange`** lebt in `shifty-utils/src/date_range.rs` (neu) ab Phase 1. API:
  - `DateRange { from: time::Date, to: time::Date }` — inclusive beidseitig
  - `new(from, to) -> Result<DateRange, RangeError>` — validiert `from <= to`
  - `overlaps(other: &DateRange) -> bool`
  - `contains(date: time::Date) -> bool`
  - `iter_days() -> impl Iterator<Item = time::Date>` (für Phase 2)
  - `day_count() -> u32` (für Phase 2)
- **D-17:** Phase 1 nutzt `overlaps()` und `contains()`; `iter_days()` und `day_count()` sind bereits da, wenn Phase 2 sie braucht (vermeidet späteren Refactor in shifty-utils).

### Claude's Discretion (Plan-Phase entscheidet)

- **C-01:** **`AbsenceDao::find_overlapping`-Signatur:** Plan-Phase entscheidet zwischen `(sales_person_id, category, range)` und `(sales_person_id, range)` basierend auf SQL-Index-Optimierung. Vorgabe: Self-Overlap-Detection braucht kategorie-scoped (D-12) — Phase 3 kann separate `find_overlapping_for_booking(sales_person_id, range)` für cross-kategorie Booking-Konflikt-Lookups nachziehen.
- **C-02:** **REST-Filter-Query-Params** für `GET /absence-period` (z.B. `?sales_person_id=X&from=Y&to=Z&category=Vacation`) — Plan-Phase übernimmt das Pattern von `rest/src/booking.rs` und `rest/src/extra_hours.rs`.
- **C-03:** **`description`-Pflichtigkeit:** Analog ExtraHours — `Arc<str>` im Domain-Modell mit leerem String als Default; im DTO `Option<String>` mit `#[serde(default)]`. Plan-Phase darf alternativ explizit `Required` machen, falls UX-Argument vorliegt.
- **C-04:** **OpenAPI-Annotationen:** Standard `#[utoipa::path]`-Pattern; `AbsencePeriodTO` mit `#[derive(ToSchema)]`. Hinzufügen zur `ApiDoc`-Struktur in `rest/src/lib.rs`.
- **C-05:** **DI-Reihenfolge in `main.rs`:** Mechanische Erweiterung um `AbsenceServiceDependencies`-Block analog `BookingServiceDependencies`. Nest `.nest("/absence-period", absence::generate_route())` im Router.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level Spezifikationen
- `.planning/PROJECT.md` — Active Requirements AB-01..AB-08, Out-of-Scope, Key Decisions.
- `.planning/REQUIREMENTS.md` — Locked v1-Requirements (ABS-01..ABS-05 für Phase 1; REP, SNAP, BOOK, PLAN, MIG für Folgephasen). Phase-Mapping in der Traceability-Tabelle.
- `.planning/ROADMAP.md` — 4-Phasen-Build-Order, Phase-1-Goal, Success-Criteria, Discuss-Carry-Overs.
- `.planning/STATE.md` — Aktuelle Position, gelogte Decisions.

### Research-Outputs
- `.planning/research/SUMMARY.md` — TL;DR für Stack-, Feature-, Architektur-, Pitfall-Entscheidungen.
- `.planning/research/ARCHITECTURE.md` — Hybrid materialize-on-snapshot / derive-on-read; Direction `BookingService → AbsenceService`.
- `.planning/research/PITFALLS.md` — Snapshot-Versioning, Carryover-Poisoning, Contract-Boundary, Booking-Warning-Direction (für Folgephasen relevanter, aber Pitfall-5 "indexes from day 1" greift hier).
- `.planning/research/STACK.md` — Stack-Begründung; `time::Date` + `sqlx`-TEXT-Storage; eigener `DateRange`-Wrapper.
- `.planning/research/FEATURES.md` — AB-01..AB-08 Mapping zu Industrie-Practices.

### Codebase-Maps
- `.planning/codebase/STRUCTURE.md` — "New Domain"-Recipe (`Where to Add New Code`-Sektion); `gen_service_impl!`-Pattern; Service-Method-Signature-Pattern.
- `.planning/codebase/CONVENTIONS.md` — Naming-Patterns (Service/DAO/TO/Entity-Suffixe); Soft-Delete-Konvention; ServiceError- und ValidationFailureItem-Enum-Surface; Logical-ID-Conversion-Patterns.
- `.planning/codebase/ARCHITECTURE.md` — Layered-Architecture-Übersicht.
- `.planning/codebase/TESTING.md` — Mock-basierte Unit-Tests + Integration-Test-Setup.
- `.planning/codebase/CONCERNS.md` — Reporting-Service-Brittleness (für Phase 2 relevant).

### Code-Templates für Phase 1
- `service/src/extra_hours.rs` — Direktes Trait-Template für `AbsenceService`; `ExtraHoursCategory`-Pattern als Vorlage für `AbsenceCategory`.
- `service_impl/src/extra_hours.rs` — Update-Flow mit logical_id (Zeilen 220-300); Permission-Pattern (HR ∨ self via `verify_user_is_sales_person`); Transaction-Pattern.
- `dao/src/extra_hours.rs` — DAO-Trait-Template (`find_by_logical_id`).
- `dao_impl_sqlite/src/extra_hours.rs` — SQLx-Compile-Time-Pattern; Soft-Delete-Konvention; ISO-8601-Date-Storage.
- `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql` — Partial-Unique-Index-Pattern als Vorlage.
- `rest/src/extra_hours.rs` und `rest/src/booking.rs` — REST-Handler-Pattern, `error_handler`-Wrapper, `#[utoipa::path]`-Annotationen.
- `rest-types/src/booking_to.rs` — Transport-Object-Template mit `ToSchema`-Derive und Bidirektional-Conversion-Pattern.
- `service/src/sales_person.rs:124-129` — `verify_user_is_sales_person`-Trait-Methode (für Permission-Gate).
- `service/src/lib.rs:121-128` — `ServiceError::OverlappingTimeRange` und `DateOrderWrong` (Validation-Channels).
- `shifty-utils/src/date_utils.rs` — Vorhandenes `ShiftyDate`-Pattern; `DateRange` lebt parallel in `shifty-utils/src/date_range.rs` (neu).
- `shifty_bin/src/main.rs` — DI-Verdrahtungs-Beispiel (`BookingServiceDependencies`-Block ist die Vorlage).
- `service_impl/src/test/extra_hours.rs` — `_forbidden`-Test-Pattern, Mock-Setup mit `expect_check_permission` und `expect_verify_user_is_sales_person`.

### Backend-Konventionen
- `shifty-backend/CLAUDE.md` — Snapshot-Schema-Versioning-Pflicht (Phase 1 nicht direkt betroffen, aber Phase 2 — wichtig für Phase-Boundary-Disziplin); Layered-Architecture; OpenAPI-Pflicht; Test-Pattern.
- `shifty-backend/CLAUDE.local.md` — VCS via `jj` (alle Commits manuell durch User; GSD `commit_docs: false`); NixOS-Hinweise (`nix-shell` für `sqlx-cli`).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`gen_service_impl!`-Macro** (`service_impl/src/macros.rs`) — generiert DI-Trait und -Struct für `AbsenceServiceImpl`. Pattern aus `service_impl/src/booking.rs:20` direkt übertragbar.
- **`verify_user_is_sales_person`** (`service/src/sales_person.rs:124`) — bestehende Methode für "ist der eingeloggte User dieser Sales-Person zugeordnet". Löst die ABS-05-"Mitarbeiter selbst"-Frage ohne Neu-Erfindung.
- **`HR_PRIVILEGE`-Konstante** (`service_impl/src/permission.rs`) — schon definiert, direkt nutzbar.
- **`ValidationFailureItem`-Enum** (`service/src/lib.rs`) — `ModificationNotAllowed`, `Duplicate`, `InvalidValue` existieren; Plan-Phase erweitert ggf. um `OverlappingPeriod(Uuid)`.
- **`ServiceError::DateOrderWrong(Date, Date)`** (`service/src/lib.rs:127`) — direkt nutzbar für `from_date > to_date`-Fall.
- **`time::Date`** + **`sqlx 0.8`** — `Date`-Encode/Decode für `TEXT`-Spalten ist out-of-the-box.
- **`MockSalesPersonService`** (über `#[automock]`) — direkter Mock für Permission-Tests.
- **REST `error_handler`-Wrapper** (`rest/src/lib.rs:120`) — mappt `ServiceError` zu HTTP-Responses; `ValidationError → 400`, `Forbidden → 403`, `EntityNotFound → 404`.

### Established Patterns

- **Soft-Delete:** `deleted: Option<PrimitiveDateTime>` in DAO-Entity, `WHERE deleted IS NULL` in jedem Read-SQL.
- **Logical-ID-Update:** Domain-`id` == DAO-`logical_id`; Update soft-deleted alte Row und schreibt neue mit selber `logical_id`, neuer physical `id`, neuer `update_version`. Optimistic-Lock via Version-UUID.
- **Transaction-Pattern:** `let tx = self.transaction_dao.use_transaction(tx).await?; ... self.transaction_dao.commit(tx).await?;` umrahmt jede Service-Methode.
- **Permission-Pattern:** `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(sales_person_id))` + `.or()`.
- **Test-Helpers:** `build_dependencies()` baut `MockXxxDeps`-Struct; jeder Test ruft `.expect_*()` und `.checkpoint()` auf den verwendeten Mocks.
- **Integration-Test-Pattern:** `shifty_bin/src/integration_test/` für End-to-End-Roundtrips mit echtem In-Memory-SQLite.
- **DTO-Conversion:** `From<&Domain> for DomainTO` und `From<&DomainTO> for Domain` (oder `TryFrom` bei Fallibility); Service-Domain-Modell nie direkt serialisiert.

### Integration Points

- **`shifty_bin/src/main.rs`:** Neuer `AbsenceServiceDependencies`-Block analog `BookingServiceDependencies` (line ~144); konkrete Instance-Erzeugung; `Arc::new(AbsenceServiceImpl { ... })`.
- **`rest/src/lib.rs`:** Neuer Module-Eintrag, `.nest("/absence-period", absence::generate_route())` im Router (line ~518); `AbsencePeriodTO` zur `ApiDoc`-Struktur hinzufügen.
- **`service/src/lib.rs`:** `pub mod absence;` und Re-Exports.
- **`service_impl/src/lib.rs`:** `pub mod absence;` Re-Export.
- **`dao/src/lib.rs`:** `pub mod absence;` und Re-Export.
- **`dao_impl_sqlite/src/lib.rs`:** `pub mod absence;` Re-Export.
- **`rest-types/src/lib.rs`:** `mod absence_period_to;` und `pub use absence_period_to::AbsencePeriodTO;`.
- **`shifty-utils/src/lib.rs`:** `pub mod date_range;` und `pub use date_range::DateRange;`.
- **Migrations:** Eine neue Migration `migrations/sqlite/<timestamp>_create-absence-period.sql` mit Schema, Indexes, CHECK-Constraint.

### Risiken / Pitfalls für Phase 1 (aus PITFALLS.md)

- **Pitfall-5 ("Indexes from day 1"):** Composite Index `(sales_person_id, from_date) WHERE deleted IS NULL` von Anfang an mitliefern — Phase 3 hängt vom Hot-Lookup-Pfad ab.
- **Pitfall-6 (Soft-Delete + Partial Unique):** Partial unique `(logical_id) WHERE deleted IS NULL` ist die einzige korrekte Form — sonst kollidieren tombstones beim Update.
- **Pitfall-9 (Permission-Contract):** Permission-Konvention für die neue Domain wird HIER festgelegt — Phase 2/3 erben sie. Bewusst auf das ExtraHours-Pattern alignen.

</code_context>

<specifics>
## Specific Ideas

- **`update_version` als UUID** (nicht Integer-Counter) — match mit existierendem ExtraHours/Booking-Pattern (`service/src/extra_hours.rs:142`); generiert via `UuidService::new_uuid()`.
- **`description` als `Arc<str>`** im Domain-Modell mit leerem String als Default (analog `ExtraHours.description: Arc<str>`); im DTO `Option<String>` mit `#[serde(default)]`.
- **`tracing::instrument`** auf REST-Handlern (Pattern aus `rest/src/booking.rs:29`).
- **`#[automock(type Context=(); type Transaction=dao::MockTransaction;)]`** auf `AbsenceService`-Trait — direkt 1:1 von ExtraHours übernommen.
- **Integration-Test deckt:** Create → Update (Range verlängern) → Update (Kategorie umwidmen) → Soft-Delete → re-create mit gleicher logical_id-Soft-Delete-Trail. Roundtrip-CRUD plus Permission-Edge-Cases.

</specifics>

<deferred>
## Deferred Ideas

- **Holiday/Unavailable/VolunteerWork als Range** — v2-CAT-Requirements (CAT-01..03 in REQUIREMENTS.md). Schema ist bewusst nicht vorbereitet; v2-Phase darf Enum erweitern oder zweite Tabelle einführen.
- **Approval-Workflow** — APRV-01 in v2; aktuell Vertrauensbasis.
- **Halbtage / Stundengenaue Granularität** — GRAN-01/02 in v2; Phase 1 Schema bewusst date-only.
- **Self-Service-Antrag mit Status-Tracking** — v2.
- **Booking-Konflikt-Wrapper-Type** (`AbsencePeriodCreateResult { absence, warnings }`) — Phase 3 (BOOK-01). Plan-Phase 3 entscheidet, ob das ein API-Bruch wird oder ein additiver Wrapper.
- **`find_overlapping_for_booking(sales_person_id, range)`** (cross-kategorie Lookup für Phase 3) — Plan-Phase 3.
- **Reverse-Booking-Warning aus AbsencePeriod-Quelle** — Phase 3 (BOOK-02).
- **Reporting-Integration / `derive_hours_for_range`** — Phase 2.
- **Frontend (Dioxus)** — separater Workstream, nicht in dieser Backend-Iteration.

</deferred>

---

*Phase: 1-Absence-Domain-Foundation*
*Context gathered: 2026-05-01*
