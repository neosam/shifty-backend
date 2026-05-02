# Phase 3: Booking & Shift-Plan Konflikt-Integration - Research

**Researched:** 2026-05-02
**Updated:** 2026-05-02 (Re-Research nach Re-Discuss: Service-Tier-Konvention etabliert; Sektion #1 zur Cycle-Vermeidung komplett neu — kein Cycle mehr; alle anderen Sektionen unverändert valide)
**Domain:** Rust / Axum / SQLx — Service-Tier-Aware Cross-Service Read-Coupling, Wrapper-Result-API, ToSchema-Tagged-Enums
**Confidence:** HIGH (alle Anker-Code-Pfade direkt im Repo verifiziert; Service-Tier-Konvention dokumentiert in `shifty-backend/CLAUDE.md`; utoipa-tag-Verhalten via Docs bestätigt)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Warning API-Migration (Area A):**
- **D-Phase3-01:** Wrapper-Result lebt nur im Business-Logic-Tier. `BookingService::create` bleibt unverändert (`Result<Booking, ServiceError>`). Neue Methode `ShiftplanEditService::book_slot_with_conflict_check(...) -> Result<BookingCreateResult, ServiceError>`. `AbsenceService::create -> Result<AbsencePeriodCreateResult, ServiceError>` (AbsenceService IST Business-Logic-Tier).
- **D-Phase3-02:** `copy_week_with_conflict_check` aggregiert Warnings im Business-Logic-Tier. `BookingService::copy_week` bleibt unverändert. Neue Methode `ShiftplanEditService::copy_week_with_conflict_check(...) -> Result<CopyWeekResult, ServiceError>`.
- **D-Phase3-03:** REST-Wrapper-DTOs am neuen Schreib-Pfad oben; alter Endpunkt unverändert. `POST /booking` bleibt unverändert. Neue Endpunkte `POST /shiftplan-edit/booking` + `POST /shiftplan-edit/copy-week` mit Wrapper-DTOs. `POST /absence-period` + `PATCH /absence-period/{id}` geben Wrapper-DTO zurück.
- **D-Phase3-04:** `AbsenceService::update` symmetrisch zu `create`. Warnings für ALLE Tage in NEUER Range, kein Diff-Modus.

**Cross-Source-Unavailability Lookup (Area B):**
- **D-Phase3-05:** Neue DAO-Methode `AbsenceDao::find_overlapping_for_booking(sales_person_id, range, tx)` als single SQL-Query mit IN-Clause / kategorie-frei.
- **D-Phase3-06:** `ShiftplanEditService` aggregiert beide Quellen direkt (kein neuer UnavailabilityService). DI-Deps: zusätzlich `AbsenceService` (Business-Logic) + `SalesPersonUnavailableService` (Basic) — `BookingService`-Dep ist schon da.
- **D-Phase3-07:** Date-Konversion inline im Business-Logic-Tier (im `ShiftplanEditService`-Body), VOR dem internen `BookingService::create`-Call.
- **D-Phase3-08:** `AbsenceService::create/update` lädt Bookings per `BookingService` (Business-Logic ↑ konsumiert Basic ↓). Erste Variante: Loop über Wochen; Optimierung später.

**Shift-Plan-Markierungs-Surface (Area C):**
- **D-Phase3-09:** `ShiftplanViewService` bekommt per-sales-person-Variante mit zwei neuen Methoden (`get_shiftplan_week_for_sales_person`, `get_shiftplan_day_for_sales_person`).
- **D-Phase3-10:** `ShiftplanDay` bekommt `unavailable: Option<UnavailabilityMarker>`. Enum mit 3 Varianten: `AbsencePeriod`, `ManualUnavailable`, `Both`.
- **D-Phase3-11:** Parallel-Helper `build_shiftplan_day_for_sales_person` (statt Optional-Parameter am bestehenden Helper).
- **D-Phase3-12:** Permission HR ∨ `verify_user_is_sales_person(sales_person_id)` für die per-sales-person-Methoden + die neuen `ShiftplanEditService`-Methoden.
- **D-Phase3-13:** Per-Woche-Scope. Frontend loopt für Multi-Wochen.

**Warning-Datenmodell + Doppel-Markierung (Area D):**
- **D-Phase3-14:** `Warning`-Enum mit 4 klaren Varianten: `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`.
- **D-Phase3-15:** Granularität: eine Warning pro betroffenem Booking-Tag. Keine Aggregation.
- **D-Phase3-16:** Doppel-Quelle = Warning + KEIN Auto-Cleanup von `sales_person_unavailable`-Einträgen.
- **D-Phase3-17:** `SalesPersonUnavailableService::create` bleibt unverändert (deferred). Variante `ManualUnavailableOnAbsenceDay` wird in Phase 3 NICHT eingeführt.

**Service-Tier-Korollar (Re-Discuss 2026-05-02):**
- **D-Phase3-18:** `BookingService` bleibt strikt Basic-Tier. Keine neuen Service-Deps, keine Signatur-Brüche, keine Warning-Produktion. Konsumiert ausschließlich `BookingDao`, `PermissionService`, `TransactionDao` (+ Slot/SalesPerson falls schon vorhanden, weil das primäre CRUD-Validation ist). Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen".

### Claude's Discretion

- **C-Phase3-01:** Modul-Lokation für `Warning` und Wrapper-Structs. Vorgabe: eigenes `service/src/warning.rs` für Warning-Enum (geteilt zwischen `AbsenceService` + `ShiftplanEditService`); Wrapper-Structs jeweils im produzierenden Service-Modul (`AbsencePeriodCreateResult` in `service/src/absence.rs`; `BookingCreateResult` + `CopyWeekResult` in `service/src/shiftplan_edit.rs`).
- **C-Phase3-02:** Range-Lookup-API für `AbsenceService::create`. Vorgabe: erst Loop über Kalenderwochen mit `BookingService::get_for_week`. Optimieren mit neuer `BookingService::get_for_range`-Methode falls Tests Performance-Druck zeigen. **Neue Read-Methode auf BookingService ist Service-Tier-konform** (Read-Surface auf eigenem Aggregat).
- **C-Phase3-03:** `build_shiftplan_day_for_sales_person`-Layout. Vorgabe: neuer Parallel-Helper.
- **C-Phase3-04:** `Warning`-zu-`WarningTO`-Conversion. Vorgabe: Tag-und-Daten mit `#[serde(tag = "kind", content = "data")]`.
- **C-Phase3-05:** Test-Fixture für Doppel-Quelle. Vorgabe: eigene Datei `shifty_bin/src/integration_test/booking_absence_conflict.rs`.
- **C-Phase3-06:** Performance-Caching für `copy_week_with_conflict_check`. Vorgabe: erst messen, dann optimieren.
- **C-Phase3-07:** Kategorie-Trigger-Differenzierung. Default: alle 3 Kategorien gleich.
- **C-Phase3-08:** Naming/Lokation des konflikt-aware-Schreib-Pfads. Vorgabe: `ShiftplanEditService` erweitern. Methoden-Naming `book_slot_with_conflict_check` / `copy_week_with_conflict_check` als Vorschlag.
- **C-Phase3-09:** REST-Routen-Schnitt. Vorgabe: neue Route-Gruppe `/shiftplan-edit/booking` parallel zu altem `/booking`.

### Deferred Ideas (OUT OF SCOPE)

- `SalesPersonUnavailableService::create` symmetrisieren (Folgephase).
- Auto-Cleanup von `sales_person_unavailable` beim AbsencePeriod-Anlegen (irreversibel; bricht Direction).
- Multi-Wochen-Range-Variante `get_shiftplan_for_sales_person(range)` (Future-Phase).
- Kategorie-Trigger-Differenzierung (Default: alle gleich).
- Performance-Caching für `copy_week_with_conflict_check` (erst messen).
- Eigene `BookingService::get_for_range`-Methode (erst messen — Read auf eigenes Aggregat ist tier-konform).
- Frontend-Migration vom alten `POST /booking` zum neuen Endpunkt (Frontend-Workstream).
- REST-Deprecation des alten `POST /booking`-Endpunkts (spätere Phase entscheidet).
- REST-Deprecation alter Reporting-Endpunkte (Phase 4 / MIG-05).
- Phase-4-Cutover-Gate (Phase 4 / MIG-02/03).
- Carryover-Refresh (Phase 4 / MIG-04).
- Phase-1-Hygiene-Drift (Phase 4 nachreichen).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BOOK-01 | Forward-Warning: Beim Anlegen/Update einer AbsencePeriod, die ein bestehendes Booking überlappt, gibt `AbsenceService::create/update` einen Wrapper mit `Vec<Warning>` zurück (Booking-IDs + Daten). Absence wird trotzdem persistiert; kein Auto-Löschen. | Patterns 1+2+5; Code-Examples Operation 2; AbsenceService ist Business-Logic-Tier und darf BookingService konsumieren (D-Phase3-08); Wrapper-Struct `AbsencePeriodCreateResult` lebt im Service-Modul; Forward-Loop über Range mit `BookingService::get_for_week` |
| BOOK-02 | Reverse-Warning: Beim Anlegen eines Bookings auf einem Tag, der durch AbsencePeriod oder sales_person_unavailable abgedeckt ist, gibt der **NEUE Schreib-Pfad oben** (`ShiftplanEditService::book_slot_with_conflict_check`) eine Warnung zurück. **Booking wird trotzdem angelegt** — über den darunter aufgerufenen `BookingService::create`. | Patterns 1+2+4+5; Code-Examples Operation 1+3; ShiftplanEditService ist Business-Logic und hält BookingService schon als Dep; neue DAO-Methode `find_overlapping_for_booking` (D-Phase3-05); BookingService bleibt unverändert (D-Phase3-18) |
| PLAN-01 | Shift-Plan-Markierung: per-sales-person-Variante markiert pro Tag, ob er durch AbsencePeriod oder sales_person_unavailable (oder beides) abgedeckt ist. Doppel-Eintragung (ExtraHours + sales_person_unavailable) entfällt für zeitraum-basierte Kategorien. | Code-Example Operation 4 (`build_shiftplan_day_for_sales_person`); ShiftplanViewService ist Business-Logic und konsumiert AbsenceService + SalesPersonUnavailableService (D-Phase3-09/11); UnavailabilityMarker mit De-Dup `Both`-Variante (D-Phase3-10) |
</phase_requirements>

## Summary

Phase 3 ist eine **API-Erweiterungs-Phase mit klarer Service-Tier-Schichtung**: keine neue Migration, kein Service-Cycle (durch die Service-Tier-Konvention von 2026-05-02 strukturell ausgeschlossen), keine neuen DAO-Surface außer einer einzigen kategorie-freien `find_overlapping_for_booking`-Variante. Drei Wirkungs-Pfade (BOOK-01 Forward-Warning, BOOK-02 Reverse-Warning, PLAN-01 Shift-Plan-Marker) werden in vier Services geschnitten — alle Cross-Entity-Logik landet im Business-Logic-Tier:

- **`AbsenceService`** (Business-Logic, vorhanden) — Forward-Warning: `create` + `update` bekommen Signatur-Bruch zu `AbsencePeriodCreateResult`. Konsumiert neu `BookingService` + `SalesPersonUnavailableService`.
- **`ShiftplanEditService`** (Business-Logic, vorhanden — hält `BookingService` schon als Dep) — Reverse-Warning: bekommt zwei neue Methoden `book_slot_with_conflict_check` + `copy_week_with_conflict_check`. Konsumiert neu `AbsenceService` + (vorhandenes) `SalesPersonUnavailableService`.
- **`ShiftplanViewService`** (Business-Logic, vorhanden) — Shift-Plan-Marker: zwei neue per-sales-person-Methoden + neues `unavailable: Option<UnavailabilityMarker>`-Feld auf `ShiftplanDay`. Konsumiert neu `AbsenceService` + `SalesPersonUnavailableService`.
- **`AbsenceDao`** — eine neue Methode `find_overlapping_for_booking` (kategorie-frei, single SQL-Query, nutzt vorhandenen Index aus Phase 1 D-04).
- **`BookingService`** (Basic, vorhanden) — **bleibt vollständig unangetastet** (D-Phase3-18). KEINE neuen Deps, KEINE Signatur-Brüche, KEINE Warning-Produktion. Optional darf eine neue Read-Methode `get_for_range` hinzukommen (Read auf eigenes Aggregat ist tier-konform).

Der wichtigste architektur-relevante Befund (Re-Discuss 2026-05-02): **Service-Tier-Konvention macht den vorher diskutierten Booking↔Absence-Cycle strukturell unmöglich.** Die Hierarchie ist eine reine Tree-Struktur: Basic-Services konsumieren nur DAO/Permission/Tx; Business-Logic-Services konsumieren Basic-Services einseitig; Cross-Entity-Logik (Reverse-Warning) lebt strukturell im Business-Logic-Tier. Konstruktionsreihenfolge in `shifty_bin/src/main.rs::RestStateImpl::new`: erst alle Basic-Services (`BookingService`, `SalesPersonUnavailableService`, ...), dann alle Business-Logic-Services (`AbsenceService`, `ShiftplanEditService`, `ShiftplanViewService`, `ReportingService`, ...). KEIN `OnceLock`, KEIN `Arc::new_cyclic`, KEIN Service-zu-DAO-Workaround mehr nötig.

**Primary recommendation:** Plan-Phase folgt der Service-Tier-aware-Erweiterung:
1. Neue `service/src/warning.rs`-Datei mit `Warning`-Enum (geteilt zwischen `AbsenceService` und `ShiftplanEditService`).
2. Wrapper-Structs leben im produzierenden Service-Modul: `AbsencePeriodCreateResult` in `service/src/absence.rs`; `BookingCreateResult` + `CopyWeekResult` in `service/src/shiftplan_edit.rs`.
3. Date-Konversion inline mit `time::Date::from_iso_week_date(year as i32, calendar_week as u8, slot.day_of_week.into())?` — der `?`-Operator funktioniert direkt, weil `ServiceError::TimeComponentRangeError(#[from] time::error::ComponentRange)` in `service/src/lib.rs:108` schon existiert.
4. `WarningTO` als `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]`-Adjacently-tagged Enum mit struct-Varianten — utoipa 5 unterstützt das nativ.
5. `book_slot_with_conflict_check` baut Warnings VOR oder NACH dem internen `BookingService::create`-Call (Plan-Phase entscheidet — Vorgabe: NACH, damit die echte persistierte Booking-ID in der Warning steht).
6. `copy_week_with_conflict_check`-Aggregation = `let mut warnings = Vec::new(); for ... { let r = self.book_slot_with_conflict_check(...).await?; warnings.extend(r.warnings.iter().cloned()); }` — kein Restrukturieren des alten `BookingService::copy_week`-Bodies.
7. Konstruktionsreihenfolge in `shifty_bin/src/main.rs`: Plan-Phase verifiziert die jetzige Reihenfolge (BookingService Z. ~699, AbsenceService Z. ~737) und prüft, dass die zwei NEUEN Deps (BookingService → AbsenceService Verbindung im AbsenceService) und die ShiftplanEditService-DI-Erweiterung (zusätzlich AbsenceService) IN dieser Reihenfolge liegen — Basic FÜR Business-Logic.

## Architectural Responsibility Map

| Capability | Primary Tier | Service | Secondary | Rationale |
|------------|-------------|---------|-----------|-----------|
| Forward-Warning bei Absence-Anlage (Range über Bookings + ManualUnavailables) | Business-Logic | `AbsenceService` | DAO (read-only, vorhandene Methoden) + `BookingService` (Basic, neu Dep) + `SalesPersonUnavailableService` (Basic, neu Dep) | Cross-Entity-Logik gehört ins Business-Logic-Tier; AbsenceService darf Basic-Services einseitig konsumieren (CLAUDE.md § "Service-Tier-Konventionen"; D-Phase3-08) |
| Reverse-Warning bei Booking-Anlage (1 Tag, 2 Quellen) | Business-Logic | `ShiftplanEditService` (NEUE Methode `book_slot_with_conflict_check`) | `BookingService` (Basic, schon Dep) + `AbsenceService` (Business-Logic, neu Dep) + `SalesPersonUnavailableService` (Basic, schon Dep) + neue DAO-Methode `find_overlapping_for_booking` | Reverse-Warning ist Cross-Entity-Aggregation = Business-Logic; `BookingService` bleibt Basic-Tier (D-Phase3-18); ShiftplanEditService ist natürlicher Andock-Punkt (hält `BookingService` schon) |
| Shift-Plan-Marker (per Tag, per-sales-person) | Business-Logic | `ShiftplanViewService` (NEUE Methoden) | `AbsenceService` (neu Dep) + `SalesPersonUnavailableService` (neu Dep) | View-Service aggregiert beide Quellen pro Tag; Business-Logic-Tier — darf alle drei konsumieren; `build_shiftplan_day_for_sales_person`-Helper |
| Wrapper-Result-Konstruktion + REST-DTO-Mapping | REST | `rest/src/shiftplan_edit.rs` (neue Endpunkte) + `rest/src/absence.rs` (AbsencePeriodCreateResultTO) + `rest/src/shiftplan.rs` (per-sales-person-Endpunkte) | rest-types | REST mappt Domain-Wrapper → `*ResultTO`; OpenAPI dokumentiert Bruch via utoipa; **`rest/src/booking.rs` bleibt unverändert** (alter Endpunkt = pure CRUD, neuer Endpunkt = mit Warnings) |
| Cross-kategorie-Lookup (alle 3 Categories für 1 Sales-Person + Range) | DAO | `AbsenceDao::find_overlapping_for_booking` | — | Single SQL-Query mit `(sales_person_id, range)`-Filter ohne Category — bestehender Index `(sales_person_id, from_date) WHERE deleted IS NULL` aus Phase-1-D-04 reicht |
| **NICHT-Verantwortung von `BookingService`** | Basic | `BookingService` | — | `BookingService::create` + `copy_week` bleiben unverändert; KEINE Cross-Entity-Logik in einem Basic-Service (D-Phase3-18 + CLAUDE.md) |

## Standard Stack

### Core (alle bereits in der Workspace, KEINE neuen Dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | (Workspace, etabliert) | REST-Handler + Router | Existiert bereits — alle REST-Module nutzen es |
| `utoipa` | 5 (mit `time`/`rc_schema`/`uuid`-Features in `rest-types/Cargo.toml`) | OpenAPI-Schema-Generation, ToSchema-Derive | Bereits Phase-1-Standard; tag/content für adjacently-tagged Enums **ab utoipa 5 nativ unterstützt** [VERIFIED: docs.rs/utoipa/latest/utoipa/derive.ToSchema.html] |
| `serde` (mit `derive`) | (etabliert) | Serialization mit `tag`/`content` | tag+content-Adjacently-Tagged Pattern ist Standard-Serde |
| `mockall` | 0.13 (`service/Cargo.toml:mockall`) | Trait-Mocks via `#[automock]` für Service-Tests | Phase-1-Pattern; alle existierenden Mock-Services nutzen es |
| `tokio` (mit `join!`) | 1.44 | Concurrent Permission-Checks via `tokio::join!` | Phase-1-Pattern (`service_impl/src/absence.rs:90`, `:144`) |
| `time` | 0.3.36 (`service/Cargo.toml`) | `Date::from_iso_week_date`, `Weekday`-Conversion | Existing Repo-Pattern (`service_impl/src/shiftplan.rs:138`, `service_impl/src/shiftplan_edit.rs:70`, `service_impl/src/reporting.rs:994`) |
| `uuid` | 1.8.0 | UUIDs für Booking-IDs / Absence-IDs / Warning-Daten | Etabliert |
| `proptest` | 1.5 (dev-dep `shifty_bin/Cargo.toml:64`) | Property-based-Tests in `shifty_bin/src/integration_test.rs` | Existing pattern; falls Phase 3 Property-Tests will (nicht Pflicht) |

### Supporting / Internal Crates

| Library | Purpose | When to Use |
|---------|---------|-------------|
| `shifty_utils::DateRange` | `iter_days`/`day_count`/`overlaps`/`contains` | Per-Tag-Loops in `ShiftplanEditService::book_slot_with_conflict_check`, `AbsenceService::create/update`-Forward-Lookup |
| `shifty_utils::DayOfWeek` | Domain-Enum mit `From<Weekday>` / `Into<Weekday>` | Slot.day_of_week → time::Weekday-Konversion für `from_iso_week_date` |
| `service::ServiceError::TimeComponentRangeError` | `#[from] time::error::ComponentRange`-Variante in `service/src/lib.rs:108` | Date-Konversion-Fehler-Mapping; `?` funktioniert direkt |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `#[serde(tag, content)]` mit ToSchema | Manuell geschriebenes utoipa-Schema-Impl | Manuell skaliert nicht über alle 4 Varianten; tag/content ist seit utoipa 5 stable [VERIFIED: utoipa Discussion #1124] |
| `Arc<[Warning]>` im Domain | `Vec<Warning>` | Repo-Konvention nutzt `Arc<[T]>` (siehe `BookingService::get_all` Line 93) — billige Klone, immutable; konsistent. REST-Conversion zu `Vec<WarningTO>` bleibt trivial |
| Eigener Aggregator-Service `UnavailabilityService` | 2 Lookups direkt im `ShiftplanEditService` | CONTEXT.md D-Phase3-06 lockt: 2 Quellen sind nicht genug für eigene Indirektion; KISS-Prinzip |
| Range-Query `BookingService::get_for_range` | Loop über Wochen mit `get_for_week` | C-Phase3-02-Default = Loop; Optimierung später wenn Tests Druck zeigen — neue Read-Methode auf BookingService ist tier-konform |
| **Reverse-Warning in `BookingService::create`** | **Reverse-Warning in `ShiftplanEditService::book_slot_with_conflict_check`** | **Erstere bricht Service-Tier-Konvention (Basic-Service mit Cross-Entity-Logik = verboten per CLAUDE.md). Letzteres ist tier-konform und der einzige zulässige Andock-Punkt.** |

**Installation:** Keine neue Dependency — alle Versionen sind aktuell und im Workspace etabliert.

**Version verification:** Nicht erforderlich, alle benötigten Crates sind seit Phase 1/2 installiert und unverändert.

## Architecture Patterns

### System Architecture Diagram

```
                ┌──────────────────────────────────────────────┐
                │ REST Layer (Axum + utoipa)                   │
                │                                              │
                │  POST /booking                  → BookingTO         (200/201)  *unchanged*
                │  POST /booking/copy             → ()                 (200)     *unchanged*
                │  POST /shiftplan-edit/booking   → BookingCreateResultTO       (200/201)  *NEW*
                │  POST /shiftplan-edit/copy-week → CopyWeekResultTO            (200)      *NEW*
                │  POST /absence-period           → AbsencePeriodCreateResultTO (201)
                │  PUT  /absence-period           → AbsencePeriodCreateResultTO (200)
                │  GET  /shiftplan/{}/year/{}/week/{}/sales-person/{} → ShiftplanWeek (mit unavailable-Feld)  *NEW*
                │  GET  /shiftplan/.../day/{}/sales-person/{}        → ShiftplanDayAggregate                  *NEW*
                └────────────┬─────────────────────────────────┘
                             │ DTO ↔ Domain (From-Impls in rest-types/src/lib.rs)
                             ▼
                ┌──────────────────────────────────────────────────────────────┐
                │ Business-Logic Service Layer                                 │
                │                                                              │
                │ ┌────────────────────────────────────┐                       │
                │ │ ShiftplanEditService               │                       │
                │ │ ::book_slot_with_conflict_check    │ ← NEW                 │
                │ │ ::copy_week_with_conflict_check    │ ← NEW                 │
                │ │ ::modify_slot, ::add_vacation, ... │ existing              │
                │ │                                    │                       │
                │ │ produces: BookingCreateResult,     │                       │
                │ │ CopyWeekResult                     │                       │
                │ └─────┬──────────────────────────────┘                       │
                │       │ reads (existing) BookingService, SalesPersonUnavailableService
                │       │ reads (NEW DEP)   AbsenceService                     │
                │       ▼                                                       │
                │ ┌────────────────────────────────────┐                       │
                │ │ AbsenceService (existing)          │                       │
                │ │ ::create  → AbsencePeriodCreateResult                      │
                │ │ ::update  → AbsencePeriodCreateResult                      │
                │ │ ::find_overlapping_for_booking     │ ← NEW                 │
                │ │                                    │                       │
                │ │ produces: AbsencePeriodCreateResult│                       │
                │ └─────┬──────────────────────────────┘                       │
                │       │ reads (NEW DEPS) BookingService, SalesPersonUnavailableService
                │       ▼                                                       │
                │ ┌────────────────────────────────────┐                       │
                │ │ ShiftplanViewService               │                       │
                │ │ ::get_shiftplan_week_for_sales_p.. │ ← NEW                 │
                │ │ ::get_shiftplan_day_for_sales_p..  │ ← NEW                 │
                │ │ ::get_shiftplan_week, day          │ existing              │
                │ │                                    │                       │
                │ │ produces: ShiftplanWeek mit        │                       │
                │ │ unavailable: Option<Marker>        │                       │
                │ └─────┬──────────────────────────────┘                       │
                │       │ reads (NEW DEPS) AbsenceService, SalesPersonUnavailableService
                │       ▼                                                       │
                ├──────────────────────────────────────────────────────────────┤
                │ Basic Service Layer                                          │
                │                                                              │
                │ ┌──────────────────┐  ┌──────────────────────┐  ┌─────────┐ │
                │ │ BookingService   │  │ SalesPersonUnavailable│  │ ...     │ │
                │ │ ::create         │  │ Service              │  │         │ │
                │ │ ::copy_week      │  │ ::get_by_week_for_..  │  │         │ │
                │ │ ::get_for_week   │  │ ::get_all_for_..      │  │         │ │
                │ │ (UNCHANGED)      │  │ (UNCHANGED)           │  │         │ │
                │ │ produces: Booking│  │ produces: ManualUnav. │  │         │ │
                │ └──────┬───────────┘  └──────┬───────────────┘  └─────────┘ │
                │        │                     │                                │
                │        │ reads BookingDao    │ reads SalesPersonUnavailableDao│
                │        ▼                     ▼                                │
                └──────────────────────────────┬─────────────────────────────────┘
                                               ▼
                ┌─────────────────────────────────────────────────────────────┐
                │ DAO Layer                                                   │
                │                                                             │
                │ AbsenceDao::find_overlapping_for_booking(sales_person_id,   │ ← NEW
                │     range, tx) — single SQL, cross-Kategorie                │
                │ Single SQL: SELECT ... WHERE sales_person_id = ? AND        │
                │     from_date <= ? AND to_date >= ? AND deleted IS NULL     │
                │ (Index: idx_absence_period_sales_person_from)               │
                │                                                             │
                │ AbsenceDao::find_by_sales_person   ─ bestehend              │
                │ AbsenceDao::find_overlapping       ─ bestehend (kategorie-  │
                │                                       scoped, Self-Overlap) │
                │ BookingDao::find_by_week           ─ bestehend              │
                │ SalesPersonUnavailableDao::find_*  ─ bestehend              │
                └─────────────────────────────────────────────────────────────┘
                                                  │
                                                  ▼
                ┌─────────────────────────────────────────────────────────────┐
                │ SQLite — KEINE neuen Migrations (Phase-3 ist add-API-only)  │
                │ Bestehender Composite-Index aus Phase-1 D-04:               │
                │   idx_absence_period_sales_person_from                      │
                │   ON absence_period(sales_person_id, from_date)             │
                │   WHERE deleted IS NULL                                     │
                └─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (Diff zu Bestand)

```
service/src/
├── absence.rs          # bestehend; +AbsencePeriodCreateResult-struct, +find_overlapping_for_booking-Trait-Method, +update Signatur-Bruch
├── booking.rs          # bestehend; UNVERÄNDERT (D-Phase3-18); optional +get_for_range falls C-Phase3-02 optimiert
├── shiftplan.rs        # bestehend; +UnavailabilityMarker-enum, +unavailable-Field auf ShiftplanDay, +get_shiftplan_*_for_sales_person-Methoden
├── shiftplan_edit.rs   # bestehend; +BookingCreateResult+CopyWeekResult-structs, +book_slot_with_conflict_check + copy_week_with_conflict_check Trait-Methods
├── warning.rs          # NEU — Warning-Enum (4 Varianten)
└── lib.rs              # bestehend; +pub mod warning; pub use warning::Warning;

service_impl/src/
├── absence.rs          # bestehend; create+update um BookingService+SalesPersonUnavailableService-Lookups erweitert
├── booking.rs          # bestehend; UNVERÄNDERT (D-Phase3-18) — Regression-Tests bleiben grün
├── shiftplan.rs        # bestehend; +build_shiftplan_day_for_sales_person-Helper (Parallel zu build_shiftplan_day)
├── shiftplan_edit.rs   # bestehend; +book_slot_with_conflict_check + copy_week_with_conflict_check; +AbsenceService-Dep im gen_service_impl!
└── test/
    ├── absence.rs      # bestehend; +Forward-Warning-Tests (BookingService-Mock + SalesPersonUnavailableService-Mock)
    ├── booking.rs      # bestehend; UNVERÄNDERT — alle Phase-1+2-Tests bleiben grün als Regression-Schutz
    ├── shiftplan.rs    # bestehend; +per-sales-person-Test (UnavailabilityMarker::Both-Test)
    └── shiftplan_edit.rs (oder Modul-Verzeichnis) # +Reverse-Warning-Tests für die neuen Methoden (AbsenceService-Mock + SalesPersonUnavailableService-Mock + BookingService-Mock + Pitfall-6-Test)

dao/src/
├── absence.rs          # bestehend; +find_overlapping_for_booking-Trait-Method
└── (keine andere Änderung)

dao_impl_sqlite/src/
└── absence.rs          # bestehend; +find_overlapping_for_booking-Impl mit query_as!-Pattern

rest/src/
├── absence.rs          # bestehend; create-Handler + update-Handler liefern AbsencePeriodCreateResultTO statt AbsencePeriodTO
├── booking.rs          # bestehend; UNVERÄNDERT
├── shiftplan.rs        # bestehend; +2 neue per-sales-person-Endpunkte (week + day)
├── shiftplan_edit.rs   # bestehend; +2 neue Endpunkte (POST /shiftplan-edit/booking, POST /shiftplan-edit/copy-week) ODER neue Datei rest/src/shiftplan_edit_booking.rs
└── lib.rs              # bestehend; ApiDoc-Erweiterung für neue Top-Level-Endpunkte

rest-types/src/
└── lib.rs              # bestehend; +WarningTO+UnavailabilityMarkerTO+BookingCreateResultTO+AbsencePeriodCreateResultTO+CopyWeekResultTO inline (Repo-Konvention)

shifty_bin/src/
├── main.rs             # bestehend; AbsenceServiceDependencies bekommt BookingService+SalesPersonUnavailableService (NEUE Deps); ShiftplanEditServiceDependencies bekommt zusätzlich AbsenceService (NEUE Dep — BookingService + SalesPersonUnavailableService schon da); ShiftplanViewServiceDependencies bekommt AbsenceService+SalesPersonUnavailableService (NEUE Deps); BookingServiceDependencies UNVERÄNDERT
└── integration_test/
    └── booking_absence_conflict.rs   # NEU (analog absence_period.rs aus Phase 1)
```

### Pattern 1: gen_service_impl!-DI-Erweiterung im Business-Logic-Tier

**What:** Mechanische Erweiterung des bestehenden `gen_service_impl!`-Macros um zusätzliche Service-Dependencies — **ausschließlich auf Business-Logic-Services**. Phase 3 fügt zu drei Business-Logic-ServiceDeps-Strukturen Felder hinzu (`AbsenceService`, `ShiftplanEditService`, `ShiftplanViewService`); `BookingService` (Basic) bleibt unverändert.

**When to use:** Service-Tier-Konvention prüfen: Basic-Services → keine Domain-Service-Deps; Business-Logic → darf Basic + andere Business-Logic einseitig konsumieren. Nur Business-Logic-DI bekommt neue Service-Deps.

**Example A (ShiftplanEditServiceDeps in `service_impl/src/shiftplan_edit.rs:22-36`, Diff für Phase 3):**

```rust
// VORHER (heute, verifiziert via repo read):
gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        SlotService: service::slot::SlotService<Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        CarryoverService: service::carryover::CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        ReportingService: service::reporting::ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
        EmployeeWorkDetailsService: service::employee_work_details::EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        ExtraHoursService: ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

// NACHHER (Phase 3) — fügt EIN Feld hinzu (BookingService und SalesPersonUnavailableService schon da):
gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        // ... bestehende 11 Felder unverändert ...
        AbsenceService: service::absence::AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
    }
}
```

**Example B (AbsenceServiceDeps in `service_impl/src/absence.rs`, Diff für Phase 3):**

```rust
// NACHHER (Phase 3) — ergänzt zwei Felder:
gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        // ... bestehende Felder unverändert ...
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonUnavailableService: service::sales_person_unavailable::SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
    }
}
```

**Example C (BookingServiceDeps — UNVERÄNDERT in Phase 3, D-Phase3-18):**

```rust
// Phase 3 fasst BookingServiceDeps NICHT an. Keine neuen Service-Deps.
// Dies ist die Service-Tier-Konvention in Aktion: BookingService ist Basic-Tier
// und darf nur DAO + Permission + Tx + ggf. Slot/SalesPerson (CRUD-Validation) konsumieren.
```

**Source:** Pattern aus `service_impl/src/shiftplan_edit.rs:22-36` (Business-Logic mit 8+ Domain-Service-Deps), `service_impl/src/absence.rs:39-50` (existing 8 Service-Deps), `service_impl/src/reporting.rs` (12+ Deps inklusive `extra_hours_service`, `absence_service`, `feature_flag_service` — beweist dass Multi-Service-Coupling im Business-Logic-Tier etabliert ist).

**Konstruktionsreihenfolge in `shifty_bin/src/main.rs::RestStateImpl::new`:**
1. **Basic-Layer:** `BookingService` (Z. ~699 heute), `SalesPersonUnavailableService`, `SlotService`, `SalesPersonService`, `PermissionService`, `TransactionDao`, ... (alle Basic-Services).
2. **Business-Logic-Layer Step 1:** `AbsenceService` (NEU mit `BookingService` + `SalesPersonUnavailableService` als Deps; Z. ~737 heute, NACH Basic-Layer).
3. **Business-Logic-Layer Step 2:** `ShiftplanEditService` (mit `BookingService` + `AbsenceService` + `SalesPersonUnavailableService` als Deps; ALLE schon konstruiert).
4. **Business-Logic-Layer Step 3:** `ShiftplanViewService` (mit `AbsenceService` + `SalesPersonUnavailableService` + ggf. `BookingService` als Deps).
5. `ReportingService` etc. — bleibt wie heute.

**Plan-Phase-Verifikation:** Bestehende Reihenfolge in `main.rs::RestStateImpl::new` prüfen. Heute werden `booking_service` (Z. ~699) UND `absence_service` (Z. ~737) bereits in dieser Reihenfolge konstruiert — die Konvention ist also schon da, Phase 3 erweitert sie nur um die zusätzlichen `clone()`-Pässe der Basic-Services in die Business-Logic-Deps-Strukturen.

### Pattern 2: ISO-Week-Date-Konversion mit `?`-Operator

**What:** Ein Booking trägt `(year, calendar_week)` und referenziert einen Slot mit `day_of_week`. Konversion auf konkretes `time::Date` für AbsencePeriod-Lookup.

**When to use:** Im **Business-Logic-Tier**, im `ShiftplanEditService::book_slot_with_conflict_check`-Body, direkt nach dem Slot-Lookup VOR dem internen `BookingService::create`-Call. **NICHT in `BookingService::create`** (D-Phase3-07 + D-Phase3-18).

**Example:**

```rust
// Source: service_impl/src/shiftplan.rs:138, service_impl/src/shiftplan_edit.rs:70, service_impl/src/reporting.rs:994 — etabliertes Pattern
// Existing in service/src/lib.rs:108-109:
//     #[error("Time component range error: {0}")]
//     TimeComponentRangeError(#[from] time::error::ComponentRange),
// → der `?`-Operator funktioniert direkt; kein Custom-Mapping nötig.

let booking_date: time::Date = time::Date::from_iso_week_date(
    booking.year as i32,
    booking.calendar_week as u8,
    slot.day_of_week.into(), // shifty_utils::DayOfWeek -> time::Weekday (Phase-1-Conversion)
)?;

// Was sollte NIE passieren (validation läuft vor): year=0, week>53, etc.
// `?` mappt auf TimeComponentRangeError; error_handler in rest/src/lib.rs liefert 500.
// Defense: Validation oben (booking.calendar_week > 53 → ValidationFailureItem::InvalidValue)
// fängt den häufigsten Fall vorab.
```

**Source:** [VERIFIED: existing repo usage]
- `service_impl/src/shiftplan.rs:138` — `time::Date::from_iso_week_date(year as i32, week, time::Weekday::Thursday)?;`
- `service_impl/src/shiftplan_edit.rs:69-70` — bestehender Pattern direkt im selben Service, in dem Phase 3 die neuen Methoden hinzufügt
- `service_impl/src/reporting.rs:994-999` — komplexere Variante mit `nth_next`
- `shifty_bin/src/integration_test.rs:487` — `from_iso_week_date(year as i32, week, slot.day_of_week.into()).unwrap()` (proves DayOfWeek::into() compiles to time::Weekday)

### Pattern 3: Permission HR ∨ self via tokio::join!

**What:** HR-Privileg ODER User ist die selbst-referenzierte Sales-Person. Geschwindigkeitsoptimiert via parallel-await.

**When to use:** Permission-Gate für die neuen `get_shiftplan_*_for_sales_person`-Methoden (D-Phase3-12), die neuen `ShiftplanEditService::book_slot_with_conflict_check` / `copy_week_with_conflict_check`-Methoden, und implizit in den neuen `find_overlapping_for_booking`-Service-Methoden (HR-or-self analog `find_by_sales_person`).

**Example (aus `service_impl/src/absence.rs:144-153` — direkt 1:1 wiederverwendbar):**

```rust
let (hr, sp) = join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        sales_person_id,
        context,
        tx.clone().into()
    ),
);
hr.or(sp)?; // wenn beide fehlschlagen → ServiceError::Forbidden propagiert
```

### Pattern 4: Adjacently-Tagged Enum mit ToSchema (utoipa 5)

**What:** `WarningTO` als Enum mit struct-Varianten und stabilem JSON-Format `{ "kind": "...", "data": {...} }` für Frontend-Generatoren.

**When to use:** REST-DTO-Mapping von `Warning` (Domain-Enum mit 4 struct-Varianten).

**Example:**

```rust
// rest-types/src/lib.rs (inline, Repo-Konvention)
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WarningTO {
    BookingOnAbsenceDay {
        booking_id: Uuid,
        #[schema(value_type = String, format = "date")]
        date: time::Date,
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
    BookingOnUnavailableDay {
        booking_id: Uuid,
        year: u32,
        week: u8,
        day_of_week: DayOfWeekTO,
    },
    AbsenceOverlapsBooking {
        absence_id: Uuid,
        booking_id: Uuid,
        #[schema(value_type = String, format = "date")]
        date: time::Date,
    },
    AbsenceOverlapsManualUnavailable {
        absence_id: Uuid,
        unavailable_id: Uuid,
    },
}
```

**JSON-Output-Beispiel:**
```json
{ "kind": "booking_on_absence_day", "data": { "booking_id": "...", "date": "2026-04-27", "absence_id": "...", "category": "Vacation" } }
```

**Source:** [VERIFIED: docs.rs/utoipa/latest/utoipa/derive.ToSchema.html, WebFetch 2026-05-02]
- utoipa 5 unterstützt `#[serde(tag, content)]` nativ für Adjacently-Tagged Enums.
- **Limitation:** "tag" attribute cannot be used with tuple types — wir haben nur struct-Varianten, keine Tuple-Varianten. Compatible.
- **utoipa 5 migration note (Discussion #1124):** "Since 5.0.0, `#[serde(tag = ...)]` will not be used as a discriminator for enum variants — utoipa generates a oneOf-Schema, kein discriminator-Field." Das ist für unseren Use-Case egal: wir wollen das tag/content-JSON-Format, kein OpenAPI-discriminator-Polymorphism.

### Pattern 5: Wrapper-Struct + From-Impl Service-Domain ↔ TO

**What:** Service-Layer (Business-Logic-Tier!) produziert Domain-Wrapper, REST mappt zu TO via `From`.

**Example:**

```rust
// service/src/shiftplan_edit.rs (Business-Logic Service — NEU für Phase 3)
#[derive(Debug, Clone)]
pub struct BookingCreateResult {
    pub booking: Booking,
    pub warnings: Arc<[Warning]>,
}

#[derive(Debug, Clone)]
pub struct CopyWeekResult {
    pub copied_bookings: Arc<[Booking]>,
    pub warnings: Arc<[Warning]>,
}

// service/src/absence.rs (Business-Logic Service — Phase 3 erweitert die bestehende API)
#[derive(Debug, Clone)]
pub struct AbsencePeriodCreateResult {
    pub absence: AbsencePeriod,
    pub warnings: Arc<[Warning]>,
}

// rest-types/src/lib.rs
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

**Note `Arc<[Warning]>` → `Vec<WarningTO>`:** REST-Layer reicht `Vec` an `serde_json::to_string`; aber `Arc<[T]>` würde auch direkt serialisieren (wir nutzen es schon in 9 Stellen in `rest-types/src/lib.rs`, z.B. `Arc<[ReportingCustomExtraHoursTO]>` Z. 474). Plan-Phase darf zwischen `Vec<WarningTO>` und `Arc<[WarningTO]>` wählen — der Repo-Standard ist gemischt; `Vec` ist einfacher für REST-Body-Bau, `Arc<[T]>` ist die Domain-Konvention.

### Anti-Patterns to Avoid

- **Reverse-Warning ODER Wrapper-Result in `BookingService::create`** — **explicit verboten per D-Phase3-18 + Service-Tier-Konvention**. `BookingService` ist Basic-Tier; Cross-Entity-Logik gehört in den Business-Logic-Tier (`ShiftplanEditService`). Plan-Phase muss aktiv prüfen, dass `BookingServiceDeps` in Phase 3 KEINE neuen Domain-Service-Deps bekommt.
- **`AbsenceService` als Dep auf `BookingService` hängen** — **explicit verboten**. Direction ist umgekehrt: `AbsenceService` (Business-Logic) konsumiert `BookingService` (Basic) einseitig. Eine umgekehrte Dep-Verbindung ist Service-Tier-Verstoß.
- **Auto-Cleanup von überlappenden `sales_person_unavailable`-Einträgen beim AbsencePeriod-Anlegen** — explicit verboten per D-Phase3-16. Bricht Phase-4-Re-Run-Idempotenz; ist irreversibel ohne Audit.
- **Reine `Vec<Warning>` ohne Wrapper-Struct** — dann müsste die Service-Methode `Result<(Booking, Vec<Warning>), ServiceError>` zurückgeben (Tuple). Wrapper-Struct ist self-documenting und macht Future-Erweiterungen (z.B. `metadata: HashMap<String, String>`) trivial.
- **`Vec<Warning>` direkt als Enum-Variante in `ServiceError`** — Warnings sind Erfolgs-Pfad, kein Fehler-Pfad. REST muss 200/201 liefern, nicht 422.
- **Naive `Warning`-De-Dup** — eine Warning pro betroffenem Booking-Tag (D-Phase3-15). KEINE Aggregation `"3 Bookings überlappen"` als 1 Warning. Frontend rendert eine Liste; Backend liefert alle einzeln.
- **`AbsenceService` ruft `BookingService`-**Schreibpfad** an (z.B. `delete`)** — strict verboten. Nur `get_for_week` oder `get_for_range` (read-only) ist erlaubt.
- **`SalesPersonUnavailableService::create` symmetrisieren in Phase 3** — explicit deferred per D-Phase3-17. Phase-3 erweitert NUR Booking + Absence-Schreibpfade. `Warning::ManualUnavailableOnAbsenceDay` ist KEINE Phase-3-Variante.
- **`BookingService::create` selbst um Date-Konversion + AbsenceLookup erweitern** — verboten; gehört in den `ShiftplanEditService::book_slot_with_conflict_check`-Body (D-Phase3-07).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ISO-Week→Date-Konversion | Eigene Year/Week/Weekday→Date-Logik | `time::Date::from_iso_week_date` | Standard-Crate; kennt ISO-8601-Edge-Cases (Week 53, Year-Boundary); existing pattern |
| Range-Iteration über Tage | Manuelle `while date <= end_date` Loops | `shifty_utils::DateRange::iter_days()` | Phase-1-D-16; getestet (8 Unit-Tests) |
| Range-Overlap-Check | Manuelle Allen-Algebra-Berechnung | `DateRange::overlaps(other)` | Phase-1-D-16; getestet |
| DayOfWeek↔Weekday-Konversion | Match-Statements in jedem Service | `shifty_utils::DayOfWeek::From/Into<Weekday>` | `shifty-utils/src/date_utils.rs:100-125` |
| Permission HR-or-self | If-Else mit erst-HR-dann-self-Sequence | `tokio::join!` mit `.or(...)` | Spart eine Roundtrip-Latenz; Phase-1-Pattern in `service_impl/src/absence.rs:90-99` |
| Adjacently-tagged Enum-JSON | Manueller `Serialize`/`Deserialize`-Impl | `#[serde(tag = "kind", content = "data")]` | utoipa 5 + serde generiert beides |
| OpenAPI-Schema-Manuell-Schreiben | Manuell-erstellte JSON-OpenAPI-Files | utoipa-Derives auf DTOs + `#[utoipa::path]` auf Handlern | Existing pattern; ApiDoc-Aggregation via nest in `rest/src/lib.rs:462-483` |
| Mock-Setup für Multi-Service-Test | Eigene Hand-rolled-Mocks | `MockAbsenceService` + `MockBookingService` + `MockSalesPersonUnavailableService` (über `#[automock]`) | Trait-Mocks bereits in den Service-Definitions auto-generiert |
| **Cycle-Vermeidung mit OnceCell / new_cyclic / DAO-Workarounds** | **OnceLock-Field, Two-Phase-Init, Service→DAO-Bypass** | **Service-Tier-Konvention (CLAUDE.md): Basic↔Basic, Business-Logic→Basic, Business-Logic→Business-Logic einseitig.** | **Strukturell tree-förmig — kein Cycle möglich. Konstruktionsreihenfolge: erst Basic, dann Business-Logic. Existing Pattern: `ReportingService` hält 8 Domain-Service-Deps; `ShiftplanEditService` hält heute 8.** |

**Key insight:** Phase 3 ist eine API-Erweiterung — alle Bausteine (DateRange, time-Crate, mockall, gen_service_impl!, Permission-Pattern, **Service-Tier-Konvention**) sind aus Phase 1/2 + CLAUDE.md vorhanden. KEIN neuer Stack-Layer. KEIN Cycle-Workaround mehr nötig.

## Common Pitfalls

### Pitfall 1: Soft-Delete-Filter im neuen DAO-Lookup

**What goes wrong:** `find_overlapping_for_booking` vergisst `WHERE deleted IS NULL` → soft-deleted AbsencePeriods triggern Warnings → SC4 verletzt.

**Why it happens:** Copy-paste aus einer Read-Query, die schon einen anderen Filter hat. Mental-Model "Allen-Algebra-Range-Match" überschreibt die Soft-Delete-Konvention.

**How to avoid:** Composite-Index ist genau so partial: `idx_absence_period_sales_person_from ... WHERE deleted IS NULL` (Migration `20260501162017_create-absence-period.sql`). Wenn die Query den Index nutzen will, MUSS sie das `WHERE deleted IS NULL`-Prädikat tragen — sonst skip der Index. Die Performance-Linting forciert die Korrektheit.

**Warning signs:** SQLite `EXPLAIN QUERY PLAN` zeigt `SCAN absence_period` statt `SEARCH USING INDEX`; oder Pitfall-6-Test schlägt fehl.

**Test:** Pitfall-6-Test (verbatim aus CONTEXT.md `<specifics>`): AbsencePeriod anlegen → soft-deleten → Booking auf demselben Tag via `ShiftplanEditService::book_slot_with_conflict_check` anlegen → `BookingCreateResult.warnings.is_empty()`.

### Pitfall 2: Date-Konversion-Panic bei invalid (year, week, day_of_week)

**What goes wrong:** `time::Date::from_iso_week_date` returnt `Err(ComponentRange)` für ungültige Inputs (z.B. Year 2026 hat KEINE Week 53). `unwrap()` würde panicen; `?` mappt auf `ServiceError::TimeComponentRangeError` → REST 500.

**Why it happens:** Validation läuft VOR der Konversion (`booking.calendar_week > 53` → `ValidationFailureItem::InvalidValue`), aber 53 ist legal in einigen Jahren — das Validation-Gate ist NICHT scharf genug, um alle invalid-Inputs zu fangen.

**How to avoid:** `?`-Operator nutzen (kein `unwrap`), und im Gate vorher `(year, week, day_of_week)` validieren — falls Plan-Phase einen aussagekräftigeren Error will, eine eigene Variante `ServiceError::IsoWeekInvalid(year, week)` einführen. Default: `?` reicht, weil das **bestehende Pattern** in `shiftplan.rs:138` und `shiftplan_edit.rs:70` exakt so läuft.

**Warning signs:** Test mit Year=2026 + Week=53 schlägt fehl mit 500 statt 422 — falls dem User saubere 422 wichtig ist, dann Variante hinzufügen.

### Pitfall 3: copy_week_with_conflict_check-Aggregation hat versehentlich N-fach gleiche Warning

**What goes wrong:** `copy_week_with_conflict_check` ruft intern `book_slot_with_conflict_check` für jedes Booking; jedes `book_slot_with_conflict_check` produziert eigene Warnings. Wenn die innere Schleife die Warnings naiv `extend`et, kann derselbe AbsencePeriod-Konflikt in mehreren Bookings resultieren — das ist KORREKT (eine Warning pro Booking-Tag), nicht zu de-dupen.

**Why it happens:** Mental-Model "1 Konflikt = 1 Warning" verleitet zu De-Dup-Logik. Aber D-Phase3-15 sagt explizit "eine Warning pro betroffenem Booking-Tag" — bei 3 Bookings auf 3 Absence-Tagen sollen 3 Warnings rauskommen.

**How to avoid:** Plan-Phase sieht klar im Test (CONTEXT.md): "3 Quell-Bookings, davon 2 auf Absence-Tagen → CopyWeekResult enthält 2 Warnings + 3 kopierte Bookings". KEINE De-Dup.

**Warning signs:** Test-Assertion `assert_eq!(result.warnings.len(), 2)` schlägt fehl, weil de-dup auf 1 reduziert hat.

### Pitfall 4: AbsenceService.update mit Diff-Modus statt voller Range-Warnings

**What goes wrong:** Plan-Phase implementiert "nur Tage, die sich geändert haben, produzieren Warnings" → asymmetrisch zu `create`, fehlerträchtig.

**Why it happens:** Mental-Model "User soll nur den Diff sehen". Aber Diff-Logik ist Frontend-Aufgabe; Backend liefert volle Liste.

**How to avoid:** D-Phase3-04 explizit: "Warnings für ALLE Tage in der NEUEN Range". Symmetrisch zu `create`. Die alte Range ist irrelevant — nur die neue zählt.

**Warning signs:** Test-Vector mit "Range verlängern um 2 Tage, davon 1 mit Booking" liefert nur 1 Warning für den 1 Tag, statt N Warnings für ALLE Bookings in der NEUEN Range — das ist falsch.

### Pitfall 5: Service-Tier-Verstoß durch Reflex zur Cycle-Vermeidung

**What goes wrong:** Mechanisches Hinzufügen von Deps zu `BookingServiceDeps` (Basic-Tier) — z.B. weil "es schneller geht" oder weil "BookingService ja sowieso schon mehrere Deps hat". Bricht Service-Tier-Konvention; Phase 4 wird darüber stolpern, weil Basic-Service plötzlich Cross-Entity-Logik kennt.

**Why it happens:** Der frühere Research-Stand (vor Re-Discuss 2026-05-02) schlug noch Cross-Service-Cycle-Workarounds vor (OnceLock, Service→DAO-Bypass, Arc::new_cyclic). Der **Reflex** "wir brauchen einen Workaround" ist obsolet — die Service-Tier-Konvention macht Cycles strukturell unmöglich, ohne dass irgendwo ein Workaround stehen müsste.

**How to avoid:** Service-Tier-Konvention strikt einhalten (CLAUDE.md § "Service-Tier-Konventionen"):
- **Basic-Service** (BookingService, SalesPersonUnavailableService, SlotService, ...): konsumiert NUR DAO + Permission + Transaction. KEINE Domain-Service-Deps. Reine Entity-Manager.
- **Business-Logic-Service** (AbsenceService, ShiftplanEditService, ShiftplanViewService, ReportingService, ...): darf Basic + andere Business-Logic einseitig konsumieren. Cross-Entity-Aggregate leben hier.
- **Direction:** Business-Logic ↑ konsumiert Basic ↓ und ggf. andere Business-Logic einseitig (Tree-Struktur). NIE umgekehrt.

**Konstruktionsreihenfolge in `shifty_bin/src/main.rs::RestStateImpl::new`:**
1. Erst alle Basic-Services (BookingService Z. ~699 heute, SalesPersonUnavailableService, SlotService, ...).
2. Dann alle Business-Logic-Services in Topological-Order: AbsenceService (Z. ~737 heute), ShiftplanEditService, ShiftplanViewService, ReportingService.
3. KEIN `OnceLock`, KEIN `Arc::new_cyclic`, KEIN Service→DAO-Workaround.

**Warning signs:**
- `BookingServiceDeps` bekommt einen neuen `AbsenceService`-Dep → STOPP, Service-Tier-Verstoß.
- `gen_service_impl!`-Block in `service_impl/src/booking.rs` wird angefasst → STOPP, D-Phase3-18 verletzt.
- Plan-File enthält "OnceLock" oder "Arc::new_cyclic" → STOPP, der Cycle-Reflex schlägt zu, obwohl er obsolet ist.

**Test:** Plan-Phase verifiziert mit `git diff service_impl/src/booking.rs` (oder `jj diff`), dass diese Datei in Phase 3 NICHT angefasst wird. Phase-1+2-Booking-Tests bleiben grün als Regression-Schutz.

### Pitfall 6: Wrapper-DTO-Bruch trifft alte Frontend-Builds (nur AbsencePeriod-Endpunkte)

**What goes wrong:** `POST /absence-period` und `PATCH /absence-period/{id}` returnen jetzt `AbsencePeriodCreateResultTO` mit `{ absence: ..., warnings: [...] }` statt `AbsencePeriodTO`. Alte Dioxus-Builds parsen das als AbsencePeriodTO-Schema → JSON-Decode-Error.

**Why it happens:** Wrapper-DTO-Bruch ist erwünscht (D-Phase3-03), aber Frontend-Workstream ist separat — Phase-3-Backend-Build kann grün sein, während Frontend rot ist. **Wichtig:** Der Bruch betrifft NUR die Absence-Endpunkte. `POST /booking` bleibt unverändert (D-Phase3-03), und der konflikt-aware-Endpunkt ist NEU (`POST /shiftplan-edit/booking`) — d.h. **kein Bruch** beim alten `POST /booking`.

**How to avoid:** Phase-3-Doku weist klar auf den OpenAPI-Bruch bei den Absence-Endpunkten hin. KEINE Backward-Compat-Variante. Wenn Frontend hinterher hinkt, ist das geplant.

**Warning signs:** Frontend-Tests rot mit "missing field 'id' in body" beim Absence-PATCH → Frontend muss Wrapper kennen lernen.

### Pitfall 7: Soft-Delete bei sales_person_unavailable

**What goes wrong:** Soft-deleted `sales_person_unavailable`-Einträge triggern Warnings — analoges Pitfall-1-Risiko aus dem Bestand.

**Why it happens:** Phase-3-Tests fokussieren auf Pitfall-1 für AbsencePeriod (SC4), aber `SalesPersonUnavailable` hat dieselbe Soft-Delete-Konvention.

**How to avoid:** Im `sales_person_unavailable_dao` ist `WHERE deleted IS NULL` schon im Bestand (DAO-Konvention). Wenn `ShiftplanEditService::book_slot_with_conflict_check` direkt `SalesPersonUnavailableService::get_by_week_for_sales_person` ruft, propagiert der Filter automatisch.

**Warning signs:** Cross-Source-Test mit soft-deletetem `sales_person_unavailable` triggert eine Warning.

## Code Examples

### Operation 1: ShiftplanEditService::book_slot_with_conflict_check (NEUE Methode im Business-Logic-Tier)

```rust
// service_impl/src/shiftplan_edit.rs::ShiftplanEditServiceImpl<Deps>::book_slot_with_conflict_check
// NEW METHOD — produces BookingCreateResult { booking, warnings }
//
// Direction: Business-Logic-Tier konsumiert Basic-Tier + Business-Logic-Tier:
//   - BookingService (Basic, schon Dep) — für die eigentliche Persistierung (UNVERÄNDERT)
//   - AbsenceService (Business-Logic, NEU Dep) — für Reverse-Lookup auf AbsencePeriod
//   - SalesPersonUnavailableService (Basic, schon Dep) — für Reverse-Lookup auf manuelle Unavailables
//   - SlotService (Basic, schon Dep) — für Slot-Read

async fn book_slot_with_conflict_check(
    &self,
    booking: Booking,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<BookingCreateResult, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // 1) Permission HR ∨ self (Pattern 3)
    let (hr, sp) = join!(
        self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            booking.sales_person_id, context.clone(), tx.clone().into()
        ),
    );
    hr.or(sp)?;

    // 2) Slot-Read (existing, basic)
    let slot = self.slot_service
        .get_slot(&booking.slot_id, Authentication::Full, tx.clone().into())
        .await?;

    // 3) Date-Konversion (Pattern 2 — direkt im Body, NICHT im BookingService)
    let booking_date: time::Date = time::Date::from_iso_week_date(
        booking.year as i32,
        booking.calendar_week as u8,
        slot.day_of_week.into(),
    )?;
    let single_day_range = shifty_utils::DateRange::new(booking_date, booking_date)
        .map_err(|_| ServiceError::DateOrderWrong(booking_date, booking_date))?;

    // 4) Lookup AbsencePeriod-Konflikte (cross-Kategorie via NEUER find_overlapping_for_booking)
    let absence_periods = self
        .absence_service
        .find_overlapping_for_booking(
            booking.sales_person_id,
            single_day_range,
            Authentication::Full, // bypass: we already permission-checked above
            tx.clone().into(),
        )
        .await?;

    // 5) Lookup SalesPersonUnavailable für die KW
    let manual_unavailables = self
        .sales_person_unavailable_service
        .get_by_week_for_sales_person(
            booking.sales_person_id,
            booking.year,
            booking.calendar_week as u8,
            Authentication::Full, // bypass
            tx.clone().into(),
        )
        .await?;

    // 6) Persist via Basic-Service (BookingService::create UNVERÄNDERT)
    let persisted_booking = self
        .booking_service
        .create(&booking, Authentication::Full, tx.clone().into())
        .await?;

    // 7) Warning-Konstruktion mit echter persistierter Booking-ID
    let mut warnings: Vec<Warning> = Vec::new();
    for ap in absence_periods.iter() {
        warnings.push(Warning::BookingOnAbsenceDay {
            booking_id: persisted_booking.id,
            date: booking_date,
            absence_id: ap.id,
            category: ap.category,
        });
    }
    for mu in manual_unavailables.iter()
        .filter(|mu| mu.day_of_week == slot.day_of_week)
    {
        warnings.push(Warning::BookingOnUnavailableDay {
            booking_id: persisted_booking.id,
            year: booking.year,
            week: booking.calendar_week as u8,
            day_of_week: slot.day_of_week,
        });
    }

    self.transaction_dao.commit(tx).await?;
    Ok(BookingCreateResult {
        booking: persisted_booking,
        warnings: Arc::from(warnings),
    })
}
```

**Key observations:**
- **`BookingService::create` wird ohne Änderung gerufen.** Der einzige Unterschied zum heutigen `POST /booking` ist die Wrapper-Schicht in `ShiftplanEditService` darüber, die Date-Konversion + Lookup + Warning-Konstruktion vorhält.
- **Reihenfolge `create` vor Warning-Konstruktion** stellt sicher, dass die echte persistierte Booking-ID in der Warning steht. Plan-Phase darf alternativ die UUID up-front generieren und dann in `BookingService::create` mitgeben — aber nur, wenn `BookingService::create` einen optionalen `id`-Parameter akzeptiert (heute nicht der Fall, also der `create-then-construct`-Pfad ist der einfachere).

### Operation 2: AbsenceService::create — Forward-Warning-Loop (Business-Logic-Tier)

```rust
// service_impl/src/absence.rs::AbsenceServiceImpl<Deps>::create
// Source: existing service_impl/src/absence.rs:137-202
//
// Insertion-Point: NACH dem find_overlapping-Self-Overlap-Check (bestehender Block Z. 173-187),
// VOR dem `entity.id = self.uuid_service.new_uuid(...)` (Z. 189).
//
// Direction: AbsenceService (Business-Logic) konsumiert BookingService (Basic) + SalesPersonUnavailableService (Basic) — beide einseitig.

// 1) Forward-Warnings: Bookings im Range
let mut warnings: Vec<Warning> = Vec::new();
let new_absence_id = self.uuid_service.new_uuid("absence_service::create::id"); // up-front

// Wir wissen: new_range = DateRange::new(entity.from_date, entity.to_date)
// Loop über Kalenderwochen der Range (Plan-Phase darf BookingService::get_for_range nutzen, falls C-Phase3-02 optimiert)
let mut weeks_seen: std::collections::BTreeSet<(u32, u8)> = std::collections::BTreeSet::new();
for day in new_range.iter_days() {
    let (iso_year, iso_week, _) = day.to_iso_week_date();
    if !weeks_seen.insert((iso_year as u32, iso_week)) {
        continue;
    }
    let bookings = self
        .booking_service
        .get_for_week(
            iso_week,
            iso_year as u32,
            Authentication::Full,
            tx.clone().into(),
        )
        .await?;
    for b in bookings.iter() {
        if b.sales_person_id != entity.sales_person_id {
            continue;
        }
        // booking_date aus iso_week + slot.day_of_week — slot muss geladen werden
        let slot = self.slot_service.get_slot(&b.slot_id, Authentication::Full, tx.clone().into()).await?;
        let booking_date = time::Date::from_iso_week_date(
            b.year as i32,
            b.calendar_week as u8,
            slot.day_of_week.into(),
        )?;
        if !new_range.contains(booking_date) {
            continue;
        }
        warnings.push(Warning::AbsenceOverlapsBooking {
            absence_id: new_absence_id,
            booking_id: b.id,
            date: booking_date,
        });
    }
}

// 2) Forward-Warnings: SalesPersonUnavailable im Range
let manual_all = self
    .sales_person_unavailable_service
    .get_all_for_sales_person(
        entity.sales_person_id,
        Authentication::Full,
        tx.clone().into(),
    )
    .await?;
for mu in manual_all.iter() {
    let mu_date = time::Date::from_iso_week_date(
        mu.year as i32,
        mu.calendar_week as u8,
        mu.day_of_week.into(),
    )?;
    if !new_range.contains(mu_date) {
        continue;
    }
    warnings.push(Warning::AbsenceOverlapsManualUnavailable {
        absence_id: new_absence_id,
        unavailable_id: mu.id,
    });
}

// 3) DAO-Create + Result
entity.id = new_absence_id;
entity.version = self.uuid_service.new_uuid("absence_service::create::version");
entity.created = Some(self.clock_service.date_time_now());
let dao_entity = absence::AbsencePeriodEntity::try_from(&entity)?;
self.absence_dao.create(&dao_entity, "absence_service::create", tx.clone()).await?;
let result = AbsencePeriodCreateResult {
    absence: entity,
    warnings: Arc::from(warnings),
};
self.transaction_dao.commit(tx).await?;
Ok(result)
```

### Operation 3: New DAO method find_overlapping_for_booking

```rust
// dao/src/absence.rs (Trait)
async fn find_overlapping_for_booking(
    &self,
    sales_person_id: Uuid,
    range: DateRange,
    tx: Self::Transaction,
) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

// dao_impl_sqlite/src/absence.rs (Impl)
async fn find_overlapping_for_booking(
    &self,
    sales_person_id: Uuid,
    range: DateRange,
    tx: Self::Transaction,
) -> Result<Arc<[AbsencePeriodEntity]>, DaoError> {
    let sp_vec = sales_person_id.as_bytes().to_vec();
    let from_str = range.from().format(&Iso8601::DATE)?;
    let to_str = range.to().format(&Iso8601::DATE)?;
    Ok(query_as!(
        AbsencePeriodDb,
        "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version \
         FROM absence_period \
         WHERE sales_person_id = ? AND from_date <= ? AND to_date >= ? AND deleted IS NULL \
         ORDER BY from_date",
        sp_vec, to_str, from_str,
    )
    .fetch_all(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?
    .iter()
    .map(AbsencePeriodEntity::try_from)
    .collect::<Result<Arc<[_]>, _>>()?)
}
```

**SQL note:** **KEIN Category-Filter** — alle 3 Kategorien werden zurückgegeben. Service-Layer entscheidet, wie zu behandeln (alle gleichermaßen Warnings per D-Phase3-15 + C-Phase3-07-Default).

### Operation 4: build_shiftplan_day_for_sales_person (NEUER Helper)

```rust
// service_impl/src/shiftplan.rs (zusätzlich zu bestehendem build_shiftplan_day Z. 24-108)
pub(crate) fn build_shiftplan_day_for_sales_person(
    day_of_week: DayOfWeek,
    day_date: time::Date,
    slots: &[Slot],
    bookings: &[Booking],
    sales_persons: &[SalesPerson],
    special_days: &[SpecialDay],
    user_assignments: Option<&HashMap<Uuid, Arc<str>>>,
    sales_person_id: Uuid,
    absence_periods: &[AbsencePeriod],
    manual_unavailables: &[SalesPersonUnavailable],
) -> Result<ShiftplanDay, ServiceError> {
    // Reuse build_shiftplan_day für Slots/Bookings/Holiday-Filter:
    let mut day = build_shiftplan_day(
        day_of_week, slots, bookings, sales_persons, special_days, user_assignments,
    )?;

    // 1) AbsencePeriod-Marker: aktive AbsencePeriod, die day_date abdeckt, für sales_person_id
    let absence_match = absence_periods.iter().find(|ap| {
        ap.deleted.is_none()
            && ap.sales_person_id == sales_person_id
            && ap.from_date <= day_date
            && day_date <= ap.to_date
    });

    // 2) ManualUnavailable-Marker: aktiver Eintrag für (sales_person, day_of_week, year, week)
    //    Caller stellt sicher, dass `manual_unavailables` schon auf year+week gefiltert ist.
    let manual_match = manual_unavailables.iter().any(|mu| {
        mu.deleted.is_none()
            && mu.sales_person_id == sales_person_id
            && mu.day_of_week == day_of_week
    });

    // 3) UnavailabilityMarker-De-Dup
    day.unavailable = match (absence_match, manual_match) {
        (Some(ap), false) => Some(UnavailabilityMarker::AbsencePeriod {
            absence_id: ap.id,
            category: ap.category,
        }),
        (None, true) => Some(UnavailabilityMarker::ManualUnavailable),
        (Some(ap), true) => Some(UnavailabilityMarker::Both {
            absence_id: ap.id,
            category: ap.category,
        }),
        (None, false) => None,
    };

    Ok(day)
}
```

**ShiftplanDay-Erweiterung in `service/src/shiftplan.rs`:**
```rust
#[derive(Debug, Clone)]
pub struct ShiftplanDay {
    pub day_of_week: DayOfWeek,
    pub slots: Vec<ShiftplanSlot>,
    pub unavailable: Option<UnavailabilityMarker>, // NEU
}
```

### Operation 5: Mockall predicate für find_overlapping_for_booking-Mock

```rust
// service_impl/src/test/shiftplan_edit.rs — pattern für Reverse-Warning-Test
// (NICHT in service_impl/src/test/booking.rs — BookingService bleibt unverändert)
use service::absence::MockAbsenceService;
use mockall::predicate::{always, eq};

let mut absence_service = MockAbsenceService::new();
absence_service
    .expect_find_overlapping_for_booking()
    .with(
        eq(default_sales_person_id()),
        always(), // DateRange — immer match (oder eq(specific_range))
        always(), // Authentication
        always(), // Option<Transaction>
    )
    .returning(|_, _, _, _| {
        Ok(Arc::from([AbsencePeriod {
            id: default_absence_id(),
            sales_person_id: default_sales_person_id(),
            category: AbsenceCategory::Vacation,
            from_date: date!(2026 - 04 - 27),
            to_date: date!(2026 - 04 - 30),
            description: Arc::from(""),
            created: Some(datetime!(2026 - 04 - 01 00:00:00)),
            deleted: None,
            version: default_absence_version(),
        }]))
    });

// MockBookingService für die persistierende Hälfte:
let mut booking_service = MockBookingService::new();
booking_service
    .expect_create()
    .returning(|booking, _, _| Ok(booking.clone()));
```

**Source:** [VERIFIED: existing pattern in `service_impl/src/test/absence.rs:204-224`, `service_impl/src/test/booking.rs:118-180`]

## Runtime State Inventory

> Phase 3 ist eine reine API-Erweiterungs-Phase ohne Rename, Refactor oder Migration.
> Trotzdem: kurze Inventur zur Sicherheit.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — keine bestehende Daten werden umbenannt oder verschoben. AbsencePeriod-Schema bleibt 1:1 wie aus Phase 1. | None |
| Live service config | None — keine externe Service-Configuration touchiert. | None |
| OS-registered state | None — keine Cron-Jobs, Task-Scheduler, systemd-Units. | None |
| Secrets/env vars | None — keine neuen Secrets, keine Env-Var-Renames. Mock-Auth-Feature-Flag bleibt unberührt. | None |
| Build artifacts | None — keine `egg-info`-equivalent in Rust; sqlx-cache (`.sqlx/*.json`) wird automatisch durch `cargo build` regeneriert. | Nach DAO-Änderung: `nix-shell -p sqlx-cli --run "cargo sqlx prepare --workspace -- --tests"` (Repo-Konvention für sqlx-Offline-Cache). |

**Nothing found in any category** — verifiziert via grep auf `AbsencePeriod`-bezogene Configurations und Schema-Entries.

## Validation Architecture

> Phase 3 ist nicht direkt unter `nyquist_validation` konfiguriert; das Repo hat KEIN
> `.planning/config.json` mit `workflow.nyquist_validation`-Key (nur `workflow.use_worktrees: false`).
> Trotzdem dokumentiere ich hier den Test-Plan, weil das CONTEXT.md im prompt explizit
> `Validation Architecture (Nyquist Dimension 8)` erfragt.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (nativ, kein extra Framework); `mockall` 0.13 für Trait-Mocks; `tokio::test` für async-Tests |
| Config file | `Cargo.toml` per Crate; keine separate Test-Config |
| Quick run command | `cargo test -p service_impl test::shiftplan_edit` (oder `test::absence`, `test::shiftplan`) |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BOOK-01 | AbsenceService::create gibt Warning bei Booking-Konflikt | unit | `cargo test -p service_impl test::absence::test_create_warning_for_booking_in_range` | ❌ Wave 0 (zu schreiben) |
| BOOK-01 | AbsenceService::create gibt Warning bei manueller Unavailable im Range | unit | `cargo test -p service_impl test::absence::test_create_warning_for_manual_unavailable_in_range` | ❌ Wave 0 |
| BOOK-01 | AbsenceService::update gibt Warning für ALLE Tage in NEUER Range | unit | `cargo test -p service_impl test::absence::test_update_returns_warnings_for_full_new_range` | ❌ Wave 0 |
| BOOK-02 | ShiftplanEditService::book_slot_with_conflict_check gibt Warning bei AbsencePeriod-Tag | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_absence_day` | ❌ Wave 0 |
| BOOK-02 | ShiftplanEditService::book_slot_with_conflict_check gibt Warning bei sales_person_unavailable | unit | `cargo test -p service_impl test::shiftplan_edit::test_book_slot_warning_on_manual_unavailable` | ❌ Wave 0 |
| BOOK-02 | ShiftplanEditService::copy_week_with_conflict_check aggregiert Warnings | unit | `cargo test -p service_impl test::shiftplan_edit::test_copy_week_aggregates_warnings` | ❌ Wave 0 |
| BOOK-02 | Cross-Source: 2 Warnings bei Doppel-Quelle | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_double_source_two_warnings` | ❌ Wave 0 |
| BOOK-02 | Pitfall-1: soft-deleted AbsencePeriod triggert KEINE Warning | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_softdeleted_absence_no_warning` | ❌ Wave 0 |
| BOOK-02 (Regression) | Klassisches `BookingService::create` und `copy_week` bleiben unverändert (kein Warning, alte Tests grün) | unit | `cargo test -p service_impl test::booking` (alle bestehenden Tests müssen grün bleiben) | ✅ existing |
| PLAN-01 | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::AbsencePeriod | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_marker_absence_only` | ❌ Wave 0 |
| PLAN-01 | get_shiftplan_week_for_sales_person liefert UnavailabilityMarker::Both | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_marker_both_sources` | ❌ Wave 0 |
| PLAN-01 | Permission HR ∨ self auf den per-sales-person-Methoden | unit | `cargo test -p service_impl test::shiftplan::test_per_sales_person_forbidden_other_user` | ❌ Wave 0 |
| ALL | _forbidden-Test pro neue public Service-Methode | unit | `cargo test -p service_impl test_*_forbidden` | ❌ Wave 0 (5 _forbidden-Tests: AbsenceService::find_overlapping_for_booking, ShiftplanEditService::{book_slot,copy_week}_with_conflict_check, ShiftplanViewService::{get_shiftplan_week,_day}_for_sales_person — die alten BookingService::create + copy_week behalten ihre _forbidden-Tests aus Phase 1+2) |
| SC4 | Pitfall-1: soft-deleted AbsencePeriod erzeugt KEINEN ShiftplanDay-Marker | integration | `cargo test -p shifty_bin integration_test::booking_absence_conflict::test_shiftplan_marker_softdeleted_absence_none` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl test::shiftplan_edit` (oder relevantes Modul) — < 30 Sekunden
- **Per wave merge:** `cargo test --workspace` — ~ 90 Sekunden (Phase 1+2 hatten 381 passing tests)
- **Phase gate:** Full suite green + `cargo build --workspace` + `cargo run` boot OK

### Wave 0 Gaps

(Wave-0 = Test-Scaffolding, falls die Plan-Phase Test-First wählt)

- [ ] `service_impl/src/test/shiftplan_edit.rs` (oder Modul-Verzeichnis) — Reverse-Warning-Tests für `book_slot_with_conflict_check` + `copy_week_with_conflict_check` + Pitfall-1-Test scaffolden
- [ ] `service_impl/src/test/absence.rs` — Forward-Warning-Tests scaffolden
- [ ] `service_impl/src/test/shiftplan.rs` — per-sales-person + UnavailabilityMarker::Both-Test scaffolden
- [ ] `shifty_bin/src/integration_test/booking_absence_conflict.rs` — NEUE Datei analog `absence_period.rs` aus Phase 1
- [ ] `shifty_bin/src/integration_test.rs` — `mod booking_absence_conflict;` ergänzen
- [ ] **Regression-Lock:** Plan-File markiert `service_impl/src/test/booking.rs` als "DO NOT MODIFY in Phase 3" — alle bestehenden BookingService-Tests müssen grün bleiben.

**Existing test infrastructure deckt:** mockall-Patterns, TestSetup für Integration, `_forbidden`-Helper (`crate::test::error_test::test_forbidden`).

## Security Domain

> `security_enforcement` ist im Repo nicht gesetzt; default-an angenommen.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Existing — `Authentication<Context>`-Type, `mock_auth`/`oidc`-Feature-Flags. Phase 3 fügt nichts hinzu. |
| V3 Session Management | yes | Existing — `SessionService`/`session_dao`. Phase 3 unverändert. |
| V4 Access Control | yes | HR ∨ self via `tokio::join!(check_permission(HR_PRIVILEGE), verify_user_is_sales_person(sales_person_id))` — D-Phase3-12 für die neuen per-sales-person-Methoden + die neuen `ShiftplanEditService::book_slot_with_conflict_check` / `copy_week_with_conflict_check`-Methoden. _forbidden-Tests Pflicht. |
| V5 Input Validation | yes | Per-Methode validation; Date-Order-Check im Service (DateRange::new), DB-CHECK-Constraint defense-in-depth. utoipa-DTOs typisieren Input. |
| V6 Cryptography | no | Keine eigene Krypto in Phase 3; UUIDs via `UuidService` (zentralisiert). |

### Known Threat Patterns for stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection | Tampering | sqlx `query_as!`/`query!` mit compile-time-checked Parameter-Binding; KEIN String-Concat in SQL |
| Cross-Sales-Person-Datenleck via per-sales-person-Endpoint | Information Disclosure | Permission-Gate HR ∨ verify_user_is_sales_person(sales_person_id); _forbidden-Test pro public method |
| Soft-Delete-Bypass via direkten DAO-Aufruf | Tampering | DAO-Konvention `WHERE deleted IS NULL` in JEDER Read-Query; Pitfall-1-Test als Regression-Guard |
| Race-Conditions zwischen create-Booking-Read-Absence und delete-Absence | Tampering | Beide Service-Methoden laufen in einer Transaction (`use_transaction(tx).await?`); SQLite-Single-Writer-Lock greift |
| Wrapper-DTO bricht alte Frontend-Cookie-Auth | DoS (Frontend-side) | Nicht Phase-3-Concern; Frontend-Workstream getrennt; OpenAPI-Diff dokumentiert den Bruch nur bei AbsencePeriod-Endpunkten |
| **Service-Tier-Bypass durch direkten DAO-Call aus Handler** | **Elevation of Privilege** | **REST-Handler dürfen NUR Service-Methoden rufen, nie DAO direkt; Service-Tier-Konvention forciert das durch klare Trennung der Crates** |

## Sources

### Primary (HIGH confidence — direct repo reads)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/booking.rs:1-114` — BookingService trait surface (UNVERÄNDERT in Phase 3)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/booking.rs:1-423` — full BookingServiceImpl (UNVERÄNDERT in Phase 3)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/shiftplan_edit.rs:1-57` — ShiftplanEditService trait surface (Erweiterungs-Anchor)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/shiftplan_edit.rs:1-392` — full ShiftplanEditServiceImpl mit `gen_service_impl!` (Z. 22-36 — hält BookingService + SalesPersonUnavailableService schon als Deps)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/absence.rs:1-267` — AbsenceService trait + ResolvedAbsence
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/absence.rs:1-462` — full AbsenceServiceImpl with derive_hours_for_range (Phase 2)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao/src/absence.rs:1-156` — AbsenceDao trait
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/dao_impl_sqlite/src/absence.rs:1-269` — full SQLite impl with two-branch find_overlapping
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/shiftplan.rs:1-71` — ShiftplanViewService surface
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/shiftplan.rs:1-287` — full ShiftplanViewServiceImpl with build_shiftplan_day
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/sales_person_unavailable.rs:1-94` — SalesPersonUnavailableService surface
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/sales_person_unavailable.rs:1-100` — Implementation with SHIFTPLANNER_PRIVILEGE check
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/lib.rs:1-122` — ServiceError enum + ValidationFailureItem incl. OverlappingPeriod
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/main.rs:1-300, 600-900` — DI setup, RestStateImpl::new construction order
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/lib.rs:1-200, 460-545` — error_handler + ApiDoc + Router-Nest pattern
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/absence.rs:1-252` — Phase 1 REST handlers as template
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/booking.rs:1-173` — current Booking REST (UNVERÄNDERT in Phase 3)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest/src/shiftplan_edit.rs` — bestehender REST-Handler für ShiftplanEditService — Erweiterungs-Anchor für die neuen konflikt-aware-Endpunkte
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs:697-1623` — DTO patterns + AbsencePeriodTO + ExtraHoursCategoryTO
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/absence.rs:1-200` — mockall pattern + build_dependencies + _forbidden tests
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/test/booking.rs:1-280` — booking mock test pattern (UNVERÄNDERT in Phase 3 als Regression-Schutz)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test/absence_period.rs:1-353` — integration test pattern with TestSetup
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty_bin/src/integration_test.rs:1-520` — TestSetup + create_admin_user + In-Memory-SQLite setup
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/migrations/sqlite/20260501162017_create-absence-period.sql` — Phase 1 schema (composite index in place)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" — **Authoritative source** für die Service-Tier-Schichtung; macht den Cycle strukturell unmöglich
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — Phase 1 decisions D-08, D-09, D-12, D-15, D-16/17
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/01-absence-domain-foundation/01-VERIFICATION.md` — confirms Phase 1 stable
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md` — Phase 2 decisions (Phase 3 is flag-independent)
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/STATE.md` — current position, architecture decisions; Service-Tier-Konvention etabliert 2026-05-02
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/ROADMAP.md` — Phase 3 SC1-SC4
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/03-booking-shift-plan-konflikt-integration/03-CONTEXT.md` — locked decisions D-Phase3-01..18 (Re-Discuss 2026-05-02)

### Secondary (MEDIUM confidence — verified via official docs)
- `https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html` — confirmed `#[serde(tag, content)]` works on ToSchema-derived enums in utoipa 5; tuple variants are excluded
- `https://github.com/juhaku/utoipa/discussions/1124` — utoipa 5 Migration Guide; confirmed tag-as-discriminator change (irrelevant for our use-case — we don't use OpenAPI discriminator polymorphism)

### Tertiary (LOW confidence)
- (none — all claims grounded in repo or official docs)

## Sources (External References)

- [utoipa ToSchema Reference](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html)
- [utoipa 5.0.0 Migration Guide](https://github.com/juhaku/utoipa/discussions/1124)
- [utoipa GitHub repo](https://github.com/juhaku/utoipa)

---

# Pflicht-Recherche-Antworten (10 Fragen aus Prompt)

## #1 Service-Tier-Konvention als strukturelle Cycle-Vermeidung

**Status:** [VERIFIED via repo grep + main.rs read + CLAUDE.md read + Re-Discuss 2026-05-02]

**Hintergrund:** Die ältere Version dieser Sektion diskutierte Cycle-Vermeidung Booking ↔ Absence in `gen_service_impl!`-DI mit drei Optionen (klassisch, OnceLock, Service→DAO). Diese ganze Diskussion ist **obsolet** seit der Re-Discuss am 2026-05-02 und der Etablierung der Service-Tier-Konvention in `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services".

**Die Antwort: Es gibt keinen Cycle.** Service-Tier-Konvention macht ihn strukturell unmöglich.

### Service-Tier-Konvention (kondensiert aus CLAUDE.md)

**Basic Services (Entity-Manager):** verwalten genau ein Fach-Objekt.
- CRUD + Validation + Permission-Gates für ihr Aggregat.
- Konsumieren NUR DAOs, `PermissionService`, `TransactionDao`.
- Konsumieren KEINE anderen Domain-Services.
- **Beispiele:** `BookingService`, `SalesPersonService`, `SalesPersonUnavailableService`, `SlotService`, `ShiftplanService` (Stamm-Daten), `SpecialDayService`.

**Business-Logic Services:** kombinieren mehrere Aggregate ODER pflegen Cross-Entity-Invarianten.
- Dürfen Basic Services UND andere Business-Logic Services konsumieren — solange kein zyklisches Coupling entsteht.
- **Beispiele:** `AbsenceService` (Multi-Tag-Range, Konflikt-Lookups), `ShiftplanViewService` (Read-Aggregat), `ShiftplanEditService` (Write-Aggregat), `ReportingService`, `BookingInformationService`, `CarryoverService`, `WorkingHoursService`.

**Regeln:**
1. Wenn zwei Services sich gegenseitig brauchen → **einer ist Basic, einer ist Business-Logic**; der Basic kennt den Business-Logic-Service nicht. Bei Bedarf wandert die Cross-Entity-Operation in einen dritten Service eine Schicht höher.
2. **DI-Konstruktion in `shifty_bin/src/main.rs`:** erst alle Basic Services, dann die Business-Logic-Schicht — keine `OnceLock`/Forward-Decl-Tricks.
3. Faustregel zur Klassifizierung: Dependencies zählen. Nur DAOs + Permission + Transaction → basic. Sobald ein anderer Domain-Service als Dep auftaucht → business-logic.

### Anwendung auf Phase 3

**Booking ↔ Absence-"Cycle"-Auflösung:**
- `BookingService` ist **Basic** — bleibt strikt ohne Domain-Service-Deps.
- `AbsenceService` ist **Business-Logic** — darf `BookingService` (Basic) und `SalesPersonUnavailableService` (Basic) als Deps halten. Direction: `AbsenceService → BookingService` (einseitig).
- `ShiftplanEditService` ist **Business-Logic** — darf `BookingService` (Basic, schon Dep), `SalesPersonUnavailableService` (Basic, schon Dep) UND `AbsenceService` (Business-Logic, NEU Dep) als Deps halten. Direction: `ShiftplanEditService → BookingService + AbsenceService + SalesPersonUnavailableService` (alle einseitig).
- `ShiftplanViewService` ist **Business-Logic** — darf `AbsenceService` + `SalesPersonUnavailableService` als Deps halten. Direction: `ShiftplanViewService → AbsenceService + SalesPersonUnavailableService` (einseitig).

**Konsequenz:** Reverse-Warning ist Cross-Entity-Logik und gehört damit ins Business-Logic-Tier (`ShiftplanEditService`), NICHT in den Basic-`BookingService`. Der Basic-`BookingService` bleibt vollständig unangetastet (D-Phase3-18).

### Konstruktionsreihenfolge in `shifty_bin/src/main.rs::RestStateImpl::new`

```
Step 1 — alle Basic-Services (in beliebiger Reihenfolge untereinander):
  - SlotService
  - SalesPersonService
  - SalesPersonUnavailableService
  - BookingService    ← Z. ~699 heute (UNVERÄNDERT in Phase 3)
  - SpecialDayService
  - ShiftplanService (Stamm-Daten)
  - ... etc.

Step 2 — Business-Logic-Services in topologischer Reihenfolge:
  - AbsenceService       ← Z. ~737 heute, NEU mit BookingService + SalesPersonUnavailableService Deps
  - WorkingHoursService
  - CarryoverService
  - EmployeeWorkDetailsService
  - ShiftplanReportService
  - ReportingService
  - ShiftplanViewService  ← NEU mit AbsenceService + SalesPersonUnavailableService Deps
  - ShiftplanEditService  ← NEU mit AbsenceService Dep (BookingService + SalesPersonUnavailableService schon da)
  - ... etc.
```

**KEIN Cycle.** KEIN `Arc::new_cyclic`. KEIN `OnceLock`. KEIN Service→DAO-Workaround.

**Beweis im Repo:** `service_impl/src/shiftplan_edit.rs:22-36` zeigt heute schon, dass `ShiftplanEditServiceImpl` 8 Domain-Service-Deps hält (`BookingService`, `CarryoverService`, `ReportingService`, `SalesPersonService`, `SalesPersonUnavailableService`, `EmployeeWorkDetailsService`, `ExtraHoursService`, `SlotService`). Multi-Service-Coupling im Business-Logic-Tier ist **etabliertes Repo-Pattern** — Phase 3 fügt nur EINEN weiteren Dep (`AbsenceService`) hinzu.

### Was die Plan-Phase verifizieren muss

1. **`BookingServiceDeps`** in `service_impl/src/booking.rs` — keine neuen Domain-Service-Deps. Plan-Phase prüft mit `git diff service_impl/src/booking.rs` (Diff bleibt leer in Phase 3, abgesehen von einer optionalen `get_for_range`-Read-Methode pro C-Phase3-02).
2. **`AbsenceServiceDeps`** — bekommt `BookingService` + `SalesPersonUnavailableService` als zwei neue Felder.
3. **`ShiftplanEditServiceDeps`** — bekommt `AbsenceService` als ein neues Feld.
4. **`ShiftplanViewServiceDeps`** — bekommt `AbsenceService` + `SalesPersonUnavailableService` als zwei neue Felder.
5. **Konstruktionsreihenfolge** in `shifty_bin/src/main.rs::RestStateImpl::new` — Plan-Phase verifiziert die Reihenfolge: Basic-Services VOR Business-Logic-Services. Heutige Reihenfolge (BookingService Z. ~699, AbsenceService Z. ~737) erfüllt das schon.

**Status:** [VERIFIED via CLAUDE.md + service_impl/src/shiftplan_edit.rs:22-36 read 2026-05-02]

## #2 time::Date::from_iso_week_date-Fehlerbehandlung

**Status:** [VERIFIED via repo + service/src/lib.rs read]

**Antwort:** **`?`-Operator funktioniert direkt.** Begründung:

```rust
// service/src/lib.rs:108-109 (existing)
#[error("Time component range error: {0}")]
TimeComponentRangeError(#[from] time::error::ComponentRange),
```

Der `#[from]`-Attribut macht `From<time::error::ComponentRange> for ServiceError` automatisch verfügbar. `from_iso_week_date` returnt `Result<time::Date, ComponentRange>`. Der `?`-Operator nutzt die From-Impl.

**REST-Layer-Mapping:** In `rest/src/lib.rs:121-200` mappt `error_handler` `ServiceError::TimeComponentRangeError` NICHT explizit → fällt in den Catch-All (Default 500). Wenn Plan-Phase saubere 422 will, eine eigene Variante einführen oder den `error_handler` um einen Match-Arm ergänzen.

**Aber:** Validation oben fängt den häufigsten Fall (`booking.calendar_week > 53` → ValidationError 422). Year-53-Edge-Case (Year 2026 hat keine Week 53) ist selten, und 500 ist akzeptabel als Defense-in-Depth.

**Existing pattern:**
- `service_impl/src/shiftplan.rs:138`: `time::Date::from_iso_week_date(year as i32, week, time::Weekday::Thursday)?;` (genau dieser Pfad)
- `service_impl/src/shiftplan_edit.rs:69-70`: bestehender Pattern direkt im selben Service, in dem Phase 3 die neuen Methoden hinzufügt
- `service_impl/src/reporting.rs:994-999`: gleicher Pattern mit komplexerer day-of-week-Berechnung
- `shifty_bin/src/integration_test.rs:487`: `.unwrap()` (Tests dürfen)

## #3 utoipa Tag-Enum-Schema mit ToSchema

**Status:** [VERIFIED via docs.rs + WebFetch]

**Antwort:** **Funktioniert.** `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]` zusammen mit `ToSchema` für struct-Varianten ist nativ ab utoipa 5 unterstützt.

**Limitation:** Tuple-Varianten (`Foo(u32, u32)`) sind NICHT unterstützt. Phase 3 hat nur struct-Varianten — kein Problem.

**utoipa-5-Caveat (Discussion #1124):** Seit utoipa 5 wird `tag` NICHT mehr automatisch als OpenAPI `discriminator` gesetzt — utoipa generiert ein einfaches `oneOf`-Schema. Das ist für JSON-Generation egal: Frontend bekommt das gewünschte `{ "kind": ..., "data": {...} }`-Format, der OpenAPI-Schema-Type ist `oneOf` ohne expliziten Discriminator. Wenn Frontend-Code-Generator (z.B. openapi-generator-cli) den Discriminator zwingend braucht, kann zusätzlich `#[schema(discriminator = "kind")]` auf das Enum gesetzt werden.

**Repo-Konformität:** Das Repo hat aktuell **keinen** tag-content-Enum. Phase 3 wäre der erste. KEIN Workaround-Pattern existing — Plan-Phase ist auf grünem Feld.

**Beispiel-Test für die Plan-Phase:** Schreibe eine kleine `WarningTO`-Variant, kompiliere, prüfe `cargo test rest_types::warning_round_trip` (TO ↔ Domain ↔ JSON ↔ Domain) und `cargo run` zeigt Swagger-UI mit `WarningTO` als oneOf.

## #4 Arc<[T]> in Wrapper-Result und JSON-Serialisierung

**Status:** [VERIFIED via repo grep — 9 existing usages of `Arc<[T]>` in rest-types]

**Antwort:** **Funktioniert problemlos.** `Arc<[T]>` serialisiert via serde-`rc`-Feature, das im Repo schon aktiv ist (alle DTOs nutzen `Arc<str>`/`Arc<[X]>` durchgängig).

**JSON-Output:** `Arc<[Warning]>` serialisiert als JSON-Array `[...]` — exakt wie `Vec<Warning>`. Kein extra `{"inner": [...]}`-Wrapper.

**REST-Layer-Conversion:**
- Domain: `Arc<[Warning]>`
- TO: kann entweder `Arc<[WarningTO]>` ODER `Vec<WarningTO>` sein. Beide kompilieren mit utoipa.

**Empfehlung:** `Vec<WarningTO>` im DTO (einfacher, klarer Lifetime). `Arc<[Warning]>` im Domain (Repo-Konvention für unveränderbare-shared-Listen). Conversion ist trivial: `r.warnings.iter().map(WarningTO::from).collect::<Vec<_>>()`.

**Performance:** `Arc<[T]>` Klone ist O(1) (refcount-bump); für `Vec<T>` Klone wäre O(N). In `copy_week_with_conflict_check` ist die Konkatenation aber via `Vec::extend_from_slice` schneller als `Arc::from(itertools::concat([...]))`. Plan-Phase darf Mid-Compute mit `Vec` arbeiten, am Ende `Arc::from(vec)`-Konvertierung.

## #5 In-Memory-SQLite Integration-Test-Setup

**Status:** [VERIFIED via repo read of integration_test.rs + absence_period.rs]

**Antwort:** Setup über `TestSetup::new()` in `shifty_bin/src/integration_test.rs:266-300`:

```rust
pub async fn new() -> Self {
    let pool = Arc::new(
        SqlitePool::connect("sqlite:sqlite::memory:").await.expect(...)
    );
    sqlx::migrate!("./../migrations/sqlite").run(pool.as_ref()).await.unwrap();
    let rest_state = RestStateImpl::new(pool.clone());
    create_admin_user(pool.clone(), "DEVUSER").await;
    let basic_dao = BasicDaoImpl::new(pool.clone());
    basic_dao.clear_all().await.unwrap();
    Self { rest_state, pool, ... }
}
```

**Helper für Fixtures (existing patterns aus `shifty_bin/src/integration_test/absence_period.rs`):**
- `create_sales_person(test_setup, name)` → erstellt SalesPerson via `rest_state.sales_person_service().create(...)`. Direct usable.
- `create_absence_period(test_setup, sales_person_id)` → analog für AbsencePeriod.

**Was Phase 3 NEU braucht:**
- `create_slot(test_setup, day_of_week, shiftplan_id)` — gibt es noch nicht; Plan-Phase erstellt einen Helper (analog absence_period.rs Z. 19-38)
- `create_booking(test_setup, sales_person_id, slot_id, year, calendar_week)` — analog. **Wichtig:** Tests nutzen den NEUEN `ShiftplanEditService::book_slot_with_conflict_check`-Pfad für die konflikt-aware-Variante (BOOK-02-Tests); der alte `BookingService::create`-Pfad ist nur für Regression-Tests.
- `create_sales_person_unavailable(test_setup, sales_person_id, year, week, day_of_week)` — analog

**Authentication:** `Authentication::Full` in allen Tests bypass-permission. Realistische Permission-Pfade (HR vs Sales-Person) → eigene Tests für die _forbidden-Pfade.

## #6 mockall::predicate für Service-Mock-Stubs

**Status:** [VERIFIED via repo read of test/booking.rs:118-180 + test/absence.rs:140-194]

**Antwort:** Pattern ist etabliert. Drei Stub-Stile:

**Stil 1 — `.returning(...)`** (catch-all):
```rust
mock_service.expect_find_overlapping_for_booking()
    .returning(|_, _, _, _| Ok(Arc::from([])));
```

**Stil 2 — `.with(predicate1, predicate2, ...).returning(...)`** (specific match):
```rust
use mockall::predicate::{always, eq};
mock_dao.expect_find_overlapping()
    .with(eq(sales_person_id), eq(category), always(), eq(Some(logical_id)), always())
    .returning(|_, _, _, _, _| Ok(Arc::from([entity])));
```

**Stil 3 — `.withf(closure).returning(...)`** (custom predicate):
```rust
mock_dao.expect_create()
    .withf(|entity, _process, _tx| entity.id == expected_id && entity.deleted.is_none())
    .returning(|_, _, _| Ok(()));
```

**Service-zu-Service-Mock (für die neuen `ShiftplanEditService`-Tests):**
- `MockBookingService` (auto-generated von `#[automock]` in `service/src/booking.rs`)
- `MockAbsenceService` (auto-generated von `#[automock]` in `service/src/absence.rs:124`)
- `MockSalesPersonUnavailableService` (auto-generated in `service/src/sales_person_unavailable.rs:55`)
- Einsetzen direkt in `ShiftplanEditServiceImpl<MockShiftplanEditDeps>` über das gen_service_impl!-erzeugte Generic.

## #7 REST-Snapshot-Pflicht / OpenAPI-Snapshot-Tests

**Status:** [VERIFIED via repo grep — keine existing snapshot tests]

**Antwort:** **KEINE OpenAPI-Snapshot-Tests existieren.** Weder `insta` noch `expect-test` ist als Dependency aktiv.

**Was es gibt:** utoipa-Generation zur Laufzeit via `ApiDoc::openapi()` in `rest/src/lib.rs:511`. SwaggerUI rendert es. Manuelle Visual-Inspection ist die einzige "Snapshot-Verification".

**Was Phase 3 NICHT machen sollte:** Insta einführen — nicht nötig, andere Repo-Konventions-Risiken (CI-Setup, snapshot-accept-Workflow mit jj). Plan-Phase darf auf utoipa-Schema-Validation verzichten, solange:
- Alle neuen DTOs `#[derive(ToSchema)]`
- Alle neuen Endpoints `#[utoipa::path(...)]`-annotiert
- ApiDoc-Aggregation in `rest/src/lib.rs:462-483` ergänzt
- `cargo build` grün → utoipa-Macros validieren das Schema zur Compile-Zeit

**Hinweis:** `rest/src/booking.rs` (current) hat KEINE utoipa-Annotationen (existing tech-debt). Phase 3 hat 2 Optionen:
- **A** — `rest/src/booking.rs` UNVERÄNDERT lassen (D-Phase3-03 → alter Endpunkt bleibt unangetastet); nur die NEUEN Endpunkte (`POST /shiftplan-edit/booking` und `/copy-week`) mit utoipa annotieren. Tech-Debt im alten Endpunkt bleibt.
- **B** — bei Gelegenheit alle Booking-Endpoints utoipa-isieren (1:1 Pattern aus `rest/src/absence.rs` übertragen). Saubererer Schritt; +30min Work; aber Plan-Phase muss sicherstellen, dass die Annotations KEINE semantischen Änderungen am Body bringen (DTO bleibt `BookingTO`, kein Wrapper).

Empfehlung Plan-Phase: **A** (minimaler Footprint). Phase 3 fasst `rest/src/booking.rs` nicht an; Tech-Debt bleibt — kann separater Phase nachgereicht werden. **Achtung:** Diese Empfehlung weicht von der älteren Research-Empfehlung ab, weil D-Phase3-03 explicit „alter Endpunkt unverändert" lockt.

## #8 copy_week_with_conflict_check vs. BookingService::copy_week

**Status:** [VERIFIED via repo read service_impl/src/booking.rs:305-363]

**Antwort:** Aktuelle `BookingService::copy_week`-Implementation ist schleifen-basiert mit innerem `self.create(...)`-Call und **bleibt UNVERÄNDERT in Phase 3 (D-Phase3-18)**:

```rust
// BookingService::copy_week (UNVERÄNDERT)
for booking in from_week.iter() {
    self.create(booking, Authentication::Full, tx.clone().into()).await?;
}
```

**Phase-3-Diff:** Die NEUE Methode `ShiftplanEditService::copy_week_with_conflict_check` baut die Aggregation-Logik im Business-Logic-Tier — sie ruft NICHT `BookingService::copy_week` (das hätte keine Warnings), sondern die eigene `book_slot_with_conflict_check`-Methode in der Schleife:

```rust
// service_impl/src/shiftplan_edit.rs::copy_week_with_conflict_check (NEW)
async fn copy_week_with_conflict_check(
    &self,
    from_week: ...,
    to_week: ...,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<CopyWeekResult, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // Permission HR ∨ self ...
    let source_bookings = self.booking_service.get_for_week(...).await?;

    let mut all_warnings: Vec<Warning> = Vec::new();
    let mut copied: Vec<Booking> = Vec::new();
    for booking in source_bookings.iter() {
        // Ziel-KW + ggf. Sales-Person-Filter aus from_week-Logik
        let target = make_target_booking(booking, to_week);
        let result = self.book_slot_with_conflict_check(target, context.clone(), tx.clone().into()).await?;
        copied.push(result.booking);
        all_warnings.extend(result.warnings.iter().cloned());
    }

    self.transaction_dao.commit(tx).await?;
    Ok(CopyWeekResult {
        copied_bookings: Arc::from(copied),
        warnings: Arc::from(all_warnings),
    })
}
```

**KEINE Restrukturierung der bestehenden `BookingService::copy_week`** (Z. 305-363). Sie bleibt vollständig unverändert. Plan-Phase darf alternativ direkt `BookingService::create` (statt `book_slot_with_conflict_check`) im inneren Loop rufen und die Warning-Konstruktion separat in der Schleife inlinen — das spart eine Indirektion, aber die Logik wird verdoppelt. **Vorgabe:** der `book_slot_with_conflict_check`-internal-call ist KISS-konformer.

**Edge-Case:** `from_week` enthält 50 Bookings, alle auf Absence-Tagen → 50 `find_overlapping_for_booking`-DAO-Calls + 50 `get_by_week_for_sales_person`-Calls. Wenn das gleichzeitig Performance-Druck zeigt, dann pre-fetch alle AbsencePeriods für den Range einmal vor der Schleife (C-Phase3-06). Default: erst messen.

## #9 AbsenceService::update und find_overlapping mit exclude_logical_id

**Status:** [VERIFIED via repo read service_impl/src/absence.rs:204-296]

**Antwort:** Aktueller Update-Pfad ruft `find_overlapping(.., Some(logical_id), ..)` für Self-Overlap-Check (kategorie-scoped, D-15). Zusätzliche Range-Booking-Lookups (Phase 3) sind **getrennte** Calls:

```rust
// Bestehender Self-Overlap-Check (D-15) bleibt unverändert
let conflicts = self.absence_dao.find_overlapping(
    active.sales_person_id,
    (&request.category).into(),
    new_range,
    Some(logical_id),
    tx.clone(),
).await?;
if !conflicts.is_empty() { return Err(...); }

// NEU für Phase 3 — Booking-Forward-Warning-Lookup, KEIN Doppel-Query:
let warnings = compute_forward_warnings(
    &self,
    new_absence_id, // == logical_id
    request.sales_person_id,
    new_range,
    tx.clone(),
).await?;
```

**Wichtig:** Die NEUE find_overlapping_for_booking-Methode ist **kategorie-frei** und **per-DAO** (nicht per-Service). Sie ist ORTHOGONAL zur bestehenden `find_overlapping`. Kein Doppel-Query, weil:
- `find_overlapping` (Phase 1) ist für Absence-Self-Overlap → liefert nur AbsencePeriods derselben Kategorie
- `find_overlapping_for_booking` (Phase 3) ist für Cross-Category-Booking-Konflikt → liefert ALLE 3 Kategorien (im Service-Layer für Reverse-Warning-Konstruktion benutzt)
- Die zwei Methoden bedienen verschiedene Use-Cases.

**`exclude_logical_id` für Forward-Warning beim AbsenceService::update?**
- Nein nicht nötig: das aktuelle Update soft-deletet die alte Row erst NACH dem Self-Overlap-Check (Z. 262-270). Der Forward-Warning-Lookup würde **NACH** dem Soft-Delete laufen, also würde der eigene old-row schon ausgeblendet werden. Aber: **der Forward-Lookup geht NICHT auf AbsencePeriods, sondern auf Bookings + SalesPersonUnavailable**. Booking-IDs sind orthogonal zu Absence-IDs. Kein Self-Match möglich. KEIN exclude-Filter erforderlich.

## #10 Validation Architecture für Phase 3 (Property-Tests, Snapshot-Tests, Pflicht-Tests)

**Status:** [Analysis based on repo state]

**Antwort:**

**Property-Tests:** Existing in `shifty_bin/src/integration_test.rs:105+, 128+, 534+` mit `proptest = "1.5.0"`. Phase 3 könnte propertizen, aber **nicht zwingend**. Konkrete Property-Test-Vorschläge (optional für Plan-Phase):

```rust
// Property: für JEDE Range R und JEDEN Booking-Date D mit D ∈ R,
//          AbsenceService::create(absence(R)) liefert eine Warning für D.
proptest! {
    #[test]
    fn forward_warning_iff_booking_in_range(
        range_from in date_range_strategy(),
        offset_days in 0u32..14,
        sales_person_id in any::<Uuid>(),
    ) {
        // ...
    }
}
```

**Snapshot-Tests:** **KEINE** (insta/expect-test nicht eingeführt). utoipa-Schema wird zur Laufzeit gerendert — Schema-Drift-Detection nur via manueller Swagger-UI-Inspection. Plan-Phase hat KEINE Pflicht, neue Snapshot-Infrastruktur einzuführen.

**Konkrete Pflicht-Tests (synthetisiert aus CONTEXT.md + Phase-1-Pattern):**

| # | Test-Name | Type | Datei |
|---|-----------|------|-------|
| 1 | `test_book_slot_warning_on_absence_day` | unit | `service_impl/src/test/shiftplan_edit.rs` |
| 2 | `test_book_slot_warning_on_manual_unavailable` | unit | `service_impl/src/test/shiftplan_edit.rs` |
| 3 | `test_book_slot_no_warning_when_softdeleted_absence` | unit | `service_impl/src/test/shiftplan_edit.rs` (Pitfall-1, mock returnt empty Vec — DAO würde soft-deleted filtern) |
| 4 | `test_create_warning_for_booking_in_absence_range` | unit | `service_impl/src/test/absence.rs` |
| 5 | `test_create_warning_for_manual_unavailable_in_range` | unit | `service_impl/src/test/absence.rs` |
| 6 | `test_update_warnings_for_full_new_range` | unit | `service_impl/src/test/absence.rs` (D-Phase3-04) |
| 7 | `test_copy_week_with_conflict_check_aggregates_warnings` | unit | `service_impl/src/test/shiftplan_edit.rs` (D-Phase3-02) |
| 8 | `test_per_sales_person_marker_absence_only` | unit | `service_impl/src/test/shiftplan.rs` |
| 9 | `test_per_sales_person_marker_manual_only` | unit | `service_impl/src/test/shiftplan.rs` |
| 10 | `test_per_sales_person_marker_both` | unit | `service_impl/src/test/shiftplan.rs` (D-Phase3-10) |
| 11 | `test_per_sales_person_marker_softdeleted_absence_none` | unit | `service_impl/src/test/shiftplan.rs` (Pitfall-1, SC4) |
| 12 | `test_find_overlapping_for_booking_forbidden` | unit | `service_impl/src/test/absence.rs` (D-11/ABS-05) |
| 13 | `test_book_slot_with_conflict_check_forbidden` | unit | `service_impl/src/test/shiftplan_edit.rs` |
| 14 | `test_copy_week_with_conflict_check_forbidden` | unit | `service_impl/src/test/shiftplan_edit.rs` |
| 15 | `test_get_shiftplan_week_for_sales_person_forbidden` | unit | `service_impl/src/test/shiftplan.rs` |
| 16 | `test_get_shiftplan_day_for_sales_person_forbidden` | unit | `service_impl/src/test/shiftplan.rs` |
| 17 | `test_double_source_two_warnings_one_booking` | integration | `shifty_bin/src/integration_test/booking_absence_conflict.rs` (Cross-Source) |
| 18 | `test_softdeleted_absence_no_warning_no_marker` | integration | `shifty_bin/src/integration_test/booking_absence_conflict.rs` (SC4) |
| 19 | `test_copy_week_three_bookings_two_warnings` | integration | `shifty_bin/src/integration_test/booking_absence_conflict.rs` |
| 20 | **REGRESSION:** `test_classic_post_booking_unchanged` (alle bestehenden Phase-1+2 BookingService-Tests bleiben grün) | unit | `service_impl/src/test/booking.rs` (UNVERÄNDERT in Phase 3) |

**Property-Test-Vorschlag (optional, nicht Pflicht):** Granularitätsproperty: für *jede* `(absence_range, list_of_bookings)`-Kombination ist `len(warnings) == count(bookings ∩ range)`. Das fängt Off-By-One-Errors in der Allen-Algebra-Range-Match.

---

# Risiken / Risks (zusätzlich zu CONTEXT.md)

## Risk 1: Service-Tier-Drift (Konvention NICHT eingehalten)

**Description:** Plan-Phase fügt — vielleicht aus alter Gewohnheit oder vom alten Research-Stand inspiriert — eine Cross-Entity-Logik in `BookingService::create` hinzu (z.B. "der Wrapper soll dort sitzen, wo das Booking entsteht"). Bricht die Service-Tier-Konvention; Folgephasen werden über das daraus resultierende Coupling stolpern.
**Impact:** D-Phase3-18 verletzt; CLAUDE.md verletzt; spätere Refactor-Schmerz.
**Mitigation:** Plan-Phase markiert `service/src/booking.rs`, `service_impl/src/booking.rs`, `service_impl/src/test/booking.rs` und `rest/src/booking.rs` explicit als "DO NOT MODIFY in Phase 3". Plan-Verifikation: `git diff` (oder `jj diff`) auf diesen 4 Files MUSS leer sein nach Phase 3 (mit der einzigen Ausnahme: optionale `get_for_range`-Read-Methode pro C-Phase3-02 darf hinzukommen).
**Severity:** HIGH (architektur-relevant; Reflex-Falle aus altem Research-Stand)

## Risk 2: mockall-Generation für `AbsenceService::find_overlapping_for_booking`

**Description:** mockall 0.13 generiert Mocks via `#[automock]`. Wenn die neue Trait-Methode einen generischen Type-Param hat, kann mockall manchmal Probleme haben. Aber `find_overlapping_for_booking(sales_person_id: Uuid, range: DateRange, ctx: Authentication<Self::Context>, tx: Option<Self::Transaction>)` ist parameter-stable (alles concrete oder bereits-bekannt-generic).
**Impact:** Niedrig — Test-Compile-Fehler, schnell zu fixen.
**Mitigation:** Plan-Phase prüft die Trait-Methode kompiliert mit `#[automock]`-derive (1-2 min Iteration).
**Severity:** LOW

## Risk 3: Test-Compile-Time-Explosion

**Description:** Phase 3 fügt 19+ neue Tests + 2-3 neue Mock-Helpers + 3 erweiterte build_dependencies() hinzu (für AbsenceService, ShiftplanEditService, ShiftplanViewService). Plus die existing 381 Tests. Compile-Zeit für `cargo test --workspace` könnte 90s → 150s steigen.
**Impact:** Developer-Loop-Pain.
**Mitigation:** Plan-Phase verwendet `cargo test -p service_impl test::shiftplan_edit` (modul-spezifisch) für Quick-Iteration. Full-Suite nur am Wave-Merge.
**Severity:** LOW (kein Blocker, nur Komfort)

## Risk 4: jj-VCS-Workflow

**Description:** Repo wird mit `jj` (Jujutsu) verwaltet (`CLAUDE.local.md`). GSD-Auto-Commit ist deaktiviert. Phase-3-Plan-Phase muss sich bewusst sein, dass jeder Commit manuell durch User passiert.
**Impact:** Plan-Phase hat KEINE auto-commit-Hooks; Executor muss `jj-commit`-Skill nutzen.
**Mitigation:** Plan-Phase gibt klare jj-Anchor-Punkte vor (welche Tasks atomic gehören, welche separate jj-changes).
**Severity:** LOW (organisatorisch, nicht technisch)

## Risk 5: Frontend-Bruch nur bei AbsencePeriod-Endpunkten

**Description:** Wrapper-DTO-Bruch betrifft NUR `POST /absence-period` und `PATCH /absence-period/{id}`. `POST /booking` bleibt unverändert (D-Phase3-03), und der konflikt-aware-Endpunkt ist NEU. Dadurch ist das Frontend-Bruch-Risiko stark reduziert verglichen mit der älteren Phase-3-Variante (in der `POST /booking` selbst gebrochen wäre).
**Impact:** Live-Frontend nur für AbsencePeriod-Anlage rot bis Frontend-Workstream nachzieht. `POST /booking` bleibt frontend-kompatibel; Frontend-Migration auf konflikt-aware-Endpunkt ist Frontend-Workstream.
**Mitigation:** Phase-3-Doku in OpenAPI-Diff klar markieren — der Bruch ist begrenzt auf AbsencePeriod-Endpunkte.
**Severity:** LOW (geplant; Bruch begrenzt)

## Risk 6: REST-Handler-Doppelung (alter vs. neuer Booking-Endpunkt)

**Description:** Der alte `POST /booking` und der neue `POST /shiftplan-edit/booking` existieren parallel. Beide rufen letztendlich `BookingService::create`. Risiko: inkonsistente Permission-Gates oder semantische Drift zwischen den beiden Handlers.
**Impact:** Verwirrung im Frontend-Team; potentielle Sicherheitslücke wenn der eine Permission-Check schwächer ist als der andere.
**Mitigation:** Plan-Phase dokumentiert: `POST /shiftplan-edit/booking` hat zusätzlich HR ∨ self-Permission-Gate (D-Phase3-12) ÜBER der `BookingService::create`-Permission. Der alte Endpunkt behält seine bestehende Permission. Diese asymmetrische Permission ist akzeptabel — der neue Endpunkt ist strenger gestaltet.
**Severity:** LOW (gut zu testen via _forbidden-Tests)

## Risk 7: TimeOrder vs. ComponentRange-Edge-Case

**Description:** `time::Date::from_iso_week_date` returnt `ComponentRange` für invalid `(year, week, weekday)`-Tupel, ABER Phase 3 nutzt das im neuen Schreib-Pfad oben (`ShiftplanEditService::book_slot_with_conflict_check`). Validation oben fängt `calendar_week > 53` UND `calendar_week <= 0`. Aber `calendar_week == 53` in einem Year ohne KW-53 ist legaler Input → `ComponentRange`-Error → 500-Statt-422.
**Impact:** UX-Edge-Case; sehr selten (Year ohne KW-53).
**Mitigation:** Plan-Phase darf optional eine eigene `ServiceError::IsoWeekInvalid(year, week)`-Variante einführen. Alternativ den `error_handler` um `TimeComponentRangeError → 422`-Map ergänzen. Default: keine Änderung, 500 ist akzeptabel.
**Severity:** LOW (UX-Politur)

## Risk 8: Pitfall-1-Test im Mock-Pfad

**Description:** Mock-Test mit "soft-deleted AbsencePeriod triggert keine Warning" muss den Service-Mock so verdrahten, dass `find_overlapping_for_booking` einen LEEREN Vec returnt — die Soft-Delete-Logik ist im DAO-SQL (`WHERE deleted IS NULL`). Mock auf Service-Level kann den DAO-SQL nicht testen, also muss das via Integration-Test (echte SQLite + Direct-Insert eines deleted-AbsencePeriod-Rows) abgedeckt werden.
**Impact:** Test-Author kann den Test falsch konstruieren.
**Mitigation:** Plan-Phase dokumentiert klar: "Mock returnt leeren Vec; Pitfall-1 ist DAO-Verantwortung. Integration-Test mit echter SQLite testet das Pitfall-1-Schema-Verhalten (per Direct-SQL-Insert mit deleted-Wert + Service-Call → leere warnings)."
**Severity:** MEDIUM (testdesign-relevant)

---

## Pflicht-Empfehlung Plan-Phase: 4-Wave-Aufbau

(Plan-Phase darf das überstimmen, aber dies ist die Default-Struktur.)

### Wave 0 — Test-Scaffolding (optional, nur wenn Test-First gewünscht)

- [ ] `service_impl/src/test/shiftplan_edit.rs` (oder Modul-Verzeichnis) — Stub `test_book_slot_warning_on_absence_day`-Test (compile-fail-erwartet)
- [ ] `service_impl/src/test/absence.rs` — Stub `test_create_warning_for_booking_in_range`-Test
- [ ] `shifty_bin/src/integration_test/booking_absence_conflict.rs` — neue Datei, leerer `#[cfg(test)] mod ...`-Marker
- [ ] **Regression-Lock dokumentieren:** Plan-File markiert `service_impl/src/test/booking.rs` als "DO NOT MODIFY in Phase 3" — bestehende BookingService-Tests bleiben grün als Regression-Schutz für D-Phase3-18.

### Wave 1 — Domain-Surface

- [ ] `service/src/warning.rs` — neue Datei mit `Warning`-Enum
- [ ] `service/src/lib.rs` — `pub mod warning; pub use warning::Warning;`
- [ ] `service/src/shiftplan_edit.rs` — `BookingCreateResult`, `CopyWeekResult` structs + 2 neue Trait-Methoden (`book_slot_with_conflict_check`, `copy_week_with_conflict_check`)
- [ ] `service/src/absence.rs` — `AbsencePeriodCreateResult` struct, `find_overlapping_for_booking`-Trait-Method, signature-Bruch auf `create`/`update`
- [ ] `service/src/shiftplan.rs` — `UnavailabilityMarker`-Enum, `unavailable: Option<...>`-Field, neue Trait-Methods
- [ ] `dao/src/absence.rs` — neue `find_overlapping_for_booking`-Trait-Method
- [ ] `dao_impl_sqlite/src/absence.rs` — Impl von `find_overlapping_for_booking`
- [ ] `dao_impl_sqlite/.sqlx/...` — neuer compile-time-cache via `cargo sqlx prepare` (NixShell)
- [ ] **`service/src/booking.rs` UNVERÄNDERT** (modulo optionale `get_for_range`-Read-Methode pro C-Phase3-02)

### Wave 2 — Service-Logik + DI

- [ ] `service_impl/src/shiftplan_edit.rs` — neue Methoden `book_slot_with_conflict_check` und `copy_week_with_conflict_check` mit Date-Konversion + 2 Lookups + Warning-Konstruktion + internem `BookingService::create`-Call
- [ ] `service_impl/src/shiftplan_edit.rs:22-36` — `gen_service_impl!`-Block: NEUE Dep `AbsenceService` hinzufügen (BookingService + SalesPersonUnavailableService schon da)
- [ ] `service_impl/src/absence.rs` — `create`/`update` mit Forward-Warning-Loop; `gen_service_impl!`-Block: NEUE Deps `BookingService` + `SalesPersonUnavailableService` hinzufügen
- [ ] `service_impl/src/shiftplan.rs` — neuer `build_shiftplan_day_for_sales_person`-Helper + 2 neue ServiceImpl-Methods; `gen_service_impl!`-Block: NEUE Deps `AbsenceService` + `SalesPersonUnavailableService` hinzufügen
- [ ] `shifty_bin/src/main.rs` — DI-Erweiterung in der Reihenfolge: erst alle Basic-Services (UNVERÄNDERT), dann Business-Logic-Services in topologischer Reihenfolge mit den neuen Deps
- [ ] **`service_impl/src/booking.rs` UNVERÄNDERT**

### Wave 3 — REST-Layer

- [ ] `rest-types/src/lib.rs` — `WarningTO`, `UnavailabilityMarkerTO`, `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO` inline
- [ ] `rest/src/shiftplan_edit.rs` (Erweiterung oder neue Datei `rest/src/shiftplan_edit_booking.rs`) — neue Endpunkte `POST /shiftplan-edit/booking` + `POST /shiftplan-edit/copy-week` mit Wrapper-DTO-Mapping; utoipa-Annotations Pflicht
- [ ] `rest/src/absence.rs` — Wrapper-DTO-Mapping für POST/PATCH /absence-period
- [ ] `rest/src/shiftplan.rs` — 2 neue per-sales-person-Endpoints
- [ ] `rest/src/lib.rs` — ApiDoc-Erweiterung
- [ ] **`rest/src/booking.rs` UNVERÄNDERT** (D-Phase3-03)

### Wave 4 — Tests (Mock + Integration)

- [ ] Alle 19+ Pflicht-Tests aus Q10
- [ ] **Regression-Verifikation:** Alle bestehenden Tests in `service_impl/src/test/booking.rs` bleiben grün (D-Phase3-18-Beweis)
- [ ] `cargo test --workspace` grün
- [ ] `cargo build --workspace` grün
- [ ] `timeout 12s cargo run` boots OK

---

# Open Questions

## Q-Open-1: Module-Lokation für Warning-Enum

**What we know:** D-Phase3-14 + C-Phase3-01 lassen offen, ob `Warning` in `service/src/warning.rs` oder inline in `service/src/lib.rs` lebt. CONTEXT.md `<specifics>` zeigt das Code-Beispiel mit `service/src/warning.rs`.

**What's unclear:** Repo-Konvention. Ein neues `warning.rs`-Modul ist sauber, aber Inline in `lib.rs` neben `ValidationFailureItem` ist konsistenter mit Phase 1 D-13 (OverlappingPeriod als ValidationFailureItem-Variante in lib.rs).

**Recommendation:** Eigenes `service/src/warning.rs` — Warnings sind funktional anders als `ValidationFailureItem` (Erfolgs-Pfad statt Fehler), trennen ist klarer. Plan-Phase darf abweichen.

## Q-Open-2: Range-Lookup-API für AbsenceService::create

**What we know:** C-Phase3-02 lockt: erst Loop-Variante (mehrere `BookingService::get_for_week`-Calls), Optimierung später.

**What's unclear:** Performance ab 60-Tage-Sabbatical-Absences. 60 / 7 ≈ 9 Wochen-Calls. Akzeptabel für initial.

**Recommendation:** Default Loop-Variante. Eine neue `BookingService::get_for_range`-Methode darf hinzukommen — sie ist Read-Surface auf dem eigenen Aggregat und damit Service-Tier-konform (D-Phase3-18-konform).

## Q-Open-3: SalesPersonUnavailable-Range-Lookup

**What we know:** `SalesPersonUnavailableService` hat keine Range-Methode (nur `get_all_for_sales_person`, `get_by_week_for_sales_person`).

**What's unclear:** Soll `get_all_for_sales_person` clientside auf den Range gefiltert werden, oder eine neue Range-Methode hinzukommen?

**Recommendation:** Erst clientside-Filter (siehe Code-Example #2). Plan-Phase darf neue DAO/Service-Methode einführen, wenn Performance es nahelegt — aber nicht zwingend. Ist auch tier-konform (Read-Surface auf eigenem Aggregat).

## Q-Open-4: Naming der neuen `ShiftplanEditService`-Methoden

**What we know:** C-Phase3-08 schlägt `book_slot_with_conflict_check` und `copy_week_with_conflict_check` vor; Plan-Phase darf prägnantere Namen wählen (z.B. `book_slot_with_warnings` / `copy_week_with_warnings`).

**What's unclear:** Naming-Konvention im Repo. Bestehende Methoden im `ShiftplanEditService` heißen `modify_slot`, `remove_slot`, `update_carryover`, `add_vacation` — kurz, aktiv, ohne Suffix.

**Recommendation:** Kürzere Namen `book_slot` / `copy_week` würden mit existierenden `BookingService::create` (singular) / `BookingService::copy_week` (gleich!) kollidieren. Längere Namen mit Suffix sind expliziter. Vorgabe: `book_slot_with_warnings` / `copy_week_with_warnings` — wirft "warnings" als zentrales Feature heraus, ist kürzer als "_with_conflict_check". Plan-Phase darf entscheiden.

## Q-Open-5: REST-Routen-Schnitt

**What we know:** C-Phase3-09 schlägt neue Route-Gruppe `/shiftplan-edit/booking` vor; Plan-Phase wählt.

**What's unclear:** Welche Route-Konvention paßt zu der bestehenden `/shiftplan-edit/...`-Subroute (falls existing)? Fall ein REST-Handler `rest/src/shiftplan_edit.rs` heute schon existiert (verifiziert: existiert), prüft Plan-Phase die Route-Präfixe in `rest/src/lib.rs`-Router-Nest — und reiht die neuen Endpunkte konsistent ein.

**Recommendation:** Falls bestehende Route schon `/shiftplan-edit` heißt → neue Endpunkte als `POST /shiftplan-edit/booking` und `POST /shiftplan-edit/copy-week`. Falls Route anders heißt → den existierenden Präfix nutzen. Plan-Phase verifiziert via `grep -n "shiftplan_edit" rest/src/lib.rs` das aktuelle Routing.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` (Rust toolchain) | Build/Test | ✓ | (workspace-pinned, latest stable) | — |
| `sqlx-cli` | DAO-Cache regenerieren nach SQL-Schema-Change | ✓ via `nix-shell -p sqlx-cli` (CLAUDE.local.md) | (system) | — |
| `jj` (Jujutsu VCS) | Commits | ✓ | (system) | git-fallback verboten per CLAUDE.local.md |
| SQLite (in-memory) | Integration tests | ✓ via sqlx | (sqlx-bundled) | — |
| `mockall` 0.13 | Trait-Mocks | ✓ | 0.13 | — |
| `utoipa` 5 | OpenAPI generation | ✓ | 5 (rest-types/Cargo.toml) | — |
| `time` 0.3.36 | Date/Weekday | ✓ | 0.3.36 | — |
| `proptest` 1.5 | Property-Tests (optional) | ✓ (dev-dep) | 1.5.0 | — |

**Missing dependencies:** None.

---

## Project Constraints (from CLAUDE.md / CLAUDE.local.md)

| Source | Directive | Impact on Phase 3 |
|--------|-----------|------------------|
| `CLAUDE.md` (workspace) | Layered Architecture: REST → Service-Trait → DAO-Trait → SQLx | Phase 3 erweitert ALLE 4 Schichten konsistent |
| `CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" | **AUTHORITATIVE.** Basic-Services (BookingService, SalesPersonUnavailableService, ...) konsumieren NUR DAO + Permission + Tx. Business-Logic-Services (AbsenceService, ShiftplanEditService, ShiftplanViewService, ...) dürfen Domain-Services konsumieren. Konstruktionsreihenfolge: erst Basic, dann Business-Logic. | **Phase-3-Kern-Constraint.** D-Phase3-18 leitet sich direkt aus dieser Sektion ab. Plan-Phase MUSS prüfen, dass `BookingServiceDeps` keine neuen Domain-Service-Deps bekommt. |
| `CLAUDE.md` | OpenAPI-Pflicht für REST: `#[utoipa::path]`-Annotation auf jedem Handler | Neue per-sales-person-Endpoints + neue konflikt-aware-Endpunkte + Wrapper-DTOs MÜSSEN annotated sein. Alter `POST /booking` bleibt unverändert (Tech-Debt bleibt). |
| `CLAUDE.md` | Service-Method-Signature: `Option<Transaction>`-Pattern | Alle neuen Trait-Methods folgen dem |
| `CLAUDE.md` | `gen_service_impl!`-DI für ServiceImpl-Konstruktion | Phase 3 nutzt das Macro für die 3 erweiterten Business-Logic-ServiceDeps-Blöcke (NICHT für `BookingServiceDeps`) |
| `CLAUDE.md` | Soft-Delete-Konvention: `WHERE deleted IS NULL` in jeder Read-Query | Pitfall-1 / Pitfall-7 / SC4 — DAO-Pflicht für `find_overlapping_for_booking` |
| `CLAUDE.md` | Snapshot-Schema-Versioning Pflicht beim Bump-Trigger | **NICHT relevant** für Phase 3 — keine Änderung an Reporting-Inputs (Phase 3 ist flag-unabhängig, kein `derive_hours_for_range`-Pfad-Touch) |
| `CLAUDE.md` | i18n-Pflicht (en/de/cs) für benutzersichtbare Texte | **Backend-DTO-only** — Frontend-i18n separater Workstream |
| `CLAUDE.md` | Tests sind Pflicht für jede Änderung (User-Global) | 19+ neue Tests minimum (Q10-Tabelle); plus Regression-Lock auf bestehende Booking-Tests |
| `CLAUDE.local.md` | NixOS: `nix-shell -p sqlx-cli` für sqlx-cli | Wave 1 Task: nach DAO-SQL-Add `cargo sqlx prepare` in nix-shell |
| `CLAUDE.local.md` | jj-VCS, kein git: GSD-Auto-Commit ist `commit_docs: false` | Plan-Phase commits nur user-initiiert; Plan-Files dokumentieren atomare Commit-Grenzen |

---

## Assumptions Log

> Diese Sektion listet alle Claims, die mit `[ASSUMED]` getaggt werden müssten. Aktuell:
> KEINE Assumed-Claims im Research — alle Claims sind entweder via Repo-Code-Read oder
> via offizielle utoipa-Docs verifiziert oder via CLAUDE.md (authoritative project doc) referenziert.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | (keine) | — | — |

**This table is empty: All claims in this research were verified or cited — no user confirmation needed.**

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — alle Versionen direkt aus `Cargo.toml`-Reads
- Architecture (Service-Tier-Konvention als Cycle-Vermeidung): HIGH — direkt verifiziert via CLAUDE.md (authoritative project doc) + `service_impl/src/shiftplan_edit.rs:22-36` (8 Domain-Service-Deps schon im Repo etabliert) + STATE.md (Konvention dokumentiert 2026-05-02)
- Patterns (gen_service_impl!, ISO-Week-Date, mockall): HIGH — alle direkten Repo-Reads
- utoipa Tag/Content Support: HIGH — verified via official docs (docs.rs)
- Pitfalls: HIGH — 8 Pitfalls aus existing repo-state oder dokumentiert in CONTEXT.md

**Research date:** 2026-05-02
**Updated:** 2026-05-02 (Re-Research nach Re-Discuss: Service-Tier-Konvention etabliert; Cycle-Vermeidung-Sektion komplett neu)
**Valid until:** 2026-06-02 (30 days — repo state stable, dependencies pinned in Cargo.toml)

---

*Phase: 3-Booking-Shift-Plan-Konflikt-Integration*
*Research conducted: 2026-05-02*
*Re-Research applied: 2026-05-02 — Service-Tier-Konvention strukturell ersetzt die alte Cycle-Vermeidungs-Diskussion. BookingService bleibt strikt Basic-Tier; Reverse-Warning lebt in `ShiftplanEditService` (Business-Logic-Tier). Kein Cycle, kein OnceLock, kein Service→DAO-Workaround.*
