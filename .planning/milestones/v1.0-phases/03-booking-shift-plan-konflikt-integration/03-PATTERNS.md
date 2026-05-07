# Phase 3: Booking & Shift-Plan Konflikt-Integration — Pattern Map

**Mapped:** 2026-05-02
**Files analysed:** 14 neue / zu erweiternde Dateien + 4 read-only Reference-Files (Regression-Lock per D-Phase3-18)
**Analogs found:** 14 / 14 (alle direkten Vorlagen direkt im Repo verifiziert; v.a. aus Phase 1 + heutigem `ShiftplanEditServiceImpl`)
**Sprache:** Deutsche Prosa. Code, SQL, Trait- und Modulnamen unverändert.

> **Lese-Reihenfolge für den Planner (entspricht der Wave-Struktur aus 03-RESEARCH.md):**
> Wave 0 Test-Scaffolding → Wave 1 Domain-Surface (`Warning` + Trait-Erweiterungen + DAO) →
> Wave 2 Service-Logik + DI-Wiring → Wave 3 REST + ApiDoc → Wave 4 Tests/Verification.
>
> **Tier-Direction (siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen"):**
> Basic ↑ wird VON Business-Logic ↓ einseitig konsumiert. `BookingService` ist
> Basic — Phase 3 fasst ihn nicht an. Cross-Entity-Logik lebt in
> `AbsenceService` / `ShiftplanEditService` / `ShiftplanViewService` (alle
> Business-Logic).

---

## File Classification

| Datei | Action | Layer | Role | Data Flow | Closest Analog | Match Quality |
|-------|--------|-------|------|-----------|----------------|---------------|
| `service/src/warning.rs` | NEW | Service Trait | Domain-Enum (Warning, 4 Tag-Varianten) | transform | `service/src/absence.rs` (Domain-Enum-Layout `AbsenceCategory`) | exact |
| `service/src/lib.rs` | PATCH | Service Wiring | `pub mod warning; pub use warning::Warning;` | wiring | `service/src/lib.rs:7-42` (bestehende `pub mod`-Reihe) | exact |
| `service/src/absence.rs` | PATCH | Service Trait | `AbsencePeriodCreateResult` struct + `find_overlapping_for_booking` Trait-Method + Sig-Bruch `create`/`update` | request-response | `service/src/absence.rs:118-122` (`ResolvedAbsence` Wrapper-Struct schon im selben File) | exact |
| `service/src/shiftplan_edit.rs` | PATCH | Service Trait | `BookingCreateResult` + `CopyWeekResult` Wrapper-Structs + 2 neue Trait-Methoden | request-response | `service/src/absence.rs:118-122` (Wrapper-Struct) + `service/src/shiftplan_edit.rs:48-56` (Methode `add_vacation` als Pattern) | exact |
| `service/src/shiftplan.rs` | PATCH | Service Trait | `UnavailabilityMarker`-Enum + `unavailable: Option<...>`-Feld auf `ShiftplanDay` + 2 neue Trait-Methoden | request-response | `service/src/shiftplan.rs:14-18` (`ShiftplanDay`) + `service/src/absence.rs:27-32` (Enum-Layout) | exact |
| `dao/src/absence.rs` | PATCH | DAO Trait | `find_overlapping_for_booking`-Trait-Method (kategorie-frei) | CRUD + range-query | `dao/src/absence.rs:78-85` (`find_overlapping`, kategorie-scoped) | exact |
| `dao_impl_sqlite/src/absence.rs` | PATCH | DAO Impl | SQLx-Query mit Soft-Delete + Range-Filter, kein Category | CRUD + range-query | `dao_impl_sqlite/src/absence.rs:155-204` (`find_overlapping`, two-branch) | exact |
| `service_impl/src/absence.rs` | PATCH | Service Impl | gen_service_impl!-Erweiterung (BookingService + SalesPersonUnavailableService) + Forward-Warning-Loop in `create`/`update` | CRUD + transaction + cross-entity-read | `service_impl/src/absence.rs:39-50` (gen_service_impl!) + `service_impl/src/absence.rs:137-202` (`create`-Body) | exact |
| `service_impl/src/shiftplan_edit.rs` | PATCH | Service Impl | gen_service_impl!-Erweiterung (AbsenceService) + 2 neue Methoden mit Date-Konversion + Cross-Source-Lookup | request-response + transaction | `service_impl/src/shiftplan_edit.rs:22-36` (gen_service_impl! mit 11 Deps) + `:43-130` (`modify_slot`-Body als Date-Konversion-Vorlage) | exact |
| `service_impl/src/shiftplan.rs` | PATCH | Service Impl | gen_service_impl!-Erweiterung (AbsenceService + SalesPersonUnavailableService) + neuer Helper `build_shiftplan_day_for_sales_person` + 2 neue Methoden | request-response | `service_impl/src/shiftplan.rs:24-108` (`build_shiftplan_day`) + `:110-203` (gen_service_impl + `get_shiftplan_week`) | exact |
| `rest-types/src/lib.rs` | PATCH | REST DTO | inline DTOs `WarningTO` (Tag-Enum), `UnavailabilityMarkerTO`, `BookingCreateResultTO`, `AbsencePeriodCreateResultTO`, `CopyWeekResultTO` + From-Impls | transform | `rest-types/src/lib.rs:1543-1620` (`AbsenceCategoryTO` + `AbsencePeriodTO`) + `:1064` (Variant-Enum-Layout `SpecialDayTypeTO`) | exact |
| `rest/src/absence.rs` | PATCH | REST Handler | POST/PATCH /absence-period auf Wrapper-DTO umstellen | request-response | `rest/src/absence.rs:43-74` (heutiger `create_absence_period`) | exact |
| `rest/src/shiftplan_edit.rs` | PATCH (oder neue Datei `rest/src/shiftplan_edit_booking.rs`) | REST Handler | 2 neue Endpunkte `POST /shiftplan-edit/booking` + `POST /shiftplan-edit/copy-week` mit Wrapper-DTO | request-response | `rest/src/absence.rs:55-74` (POST mit Wrapper) + `rest/src/shiftplan_edit.rs:25-46` (existing `edit_slot`-Pattern) | exact |
| `rest/src/shiftplan.rs` | PATCH | REST Handler | 2 neue per-sales-person-Endpunkte (week + day) | request-response | `rest/src/shiftplan.rs:34-58` (existing `get_shiftplan_week`) | exact |
| `rest/src/lib.rs` | PATCH | REST Wiring | ApiDoc-Erweiterung; ShiftplanEditApiDoc neu (existing hat keine ApiDoc) | wiring | `rest/src/lib.rs:460-484` (ApiDoc-nest) + `rest/src/shiftplan.rs:101-117` (ShiftplanApiDoc) | exact |
| `service_impl/src/test/mod.rs` | PATCH | Test Wiring | `#[cfg(test)] pub mod shiftplan_edit;` ergänzen | wiring | `service_impl/src/test/mod.rs:1-50` (existing `pub mod`-Liste) | exact |
| `service_impl/src/test/shiftplan_edit.rs` | NEW | Test Unit | Mock-DI + Reverse-Warning-Tests + Pitfall-1-Test + _forbidden | request-response (mocked) | `service_impl/src/test/booking.rs:113-192` (build_dependencies) + `service_impl/src/test/shiftplan.rs:59-100` (Mock-DI-Struktur) + `service_impl/src/test/absence.rs:200-234` (Mock-Setup-Idiom) | exact |
| `service_impl/src/test/absence.rs` | PATCH | Test Unit | Forward-Warning-Tests + Cross-Source-Tests + AbsenceDependencies um BookingService + SalesPersonUnavailableService erweitern | request-response (mocked) | `service_impl/src/test/absence.rs:121-194` (`AbsenceDependencies` + `build_dependencies`) | exact |
| `service_impl/src/test/shiftplan.rs` | PATCH | Test Unit | per-sales-person + UnavailabilityMarker::Both Tests; Mock-DI um AbsenceService + SalesPersonUnavailableService erweitern | request-response (mocked) | `service_impl/src/test/shiftplan.rs:59-100` (`ShiftplanViewServiceDependencies`) | exact |
| `shifty_bin/src/integration_test/booking_absence_conflict.rs` | NEW | Integration Test | echte In-Memory-SQLite + neue Endpunkt-Aufrufe + Pitfall-1-Test | request-response (full-stack) | `shifty_bin/src/integration_test/absence_period.rs` (TestSetup-Pattern) | exact |
| `shifty_bin/src/integration_test.rs` | PATCH | Test Wiring | `#[cfg(test)] mod booking_absence_conflict;` ergänzen | wiring | `shifty_bin/src/integration_test.rs:1432-1437` (existing `mod absence_period;`) | exact |
| `shifty_bin/src/main.rs` | PATCH | DI Wiring | `AbsenceServiceImpl{...}` bekommt `booking_service.clone()` + `sales_person_unavailable_service.clone()`; `ShiftplanEditServiceImpl{...}` bekommt `absence_service.clone()`; `ShiftplanViewServiceImpl{...}` bekommt `absence_service.clone()` + `sales_person_unavailable_service.clone()` | wiring | `shifty_bin/src/main.rs:678-839` (existing Konstruktion) | exact |

### Read-only Reference (Regression-Lock per D-Phase3-18) — DO NOT MODIFY

| Datei | Begründung | Verifikation |
|-------|------------|--------------|
| `service/src/booking.rs` | BookingService bleibt strikt Basic-Tier (CLAUDE.md § Service-Tier-Konventionen). Phase 3 fasst NUR die optionale Read-Methode `get_for_range` (C-Phase3-02, deferred) an — alle anderen Methoden bleiben unverändert. | `jj diff service/src/booking.rs` MUSS leer bleiben (modulo optionale `get_for_range`-Read-Method). |
| `service_impl/src/booking.rs` | gen_service_impl!-Block + `create`/`copy_week`-Bodies bleiben unverändert. KEIN neuer Service-Dep. | `jj diff service_impl/src/booking.rs` MUSS leer bleiben. |
| `service_impl/src/test/booking.rs` | Alle bestehenden BookingService-Tests bleiben grün als Regression-Schutz für D-Phase3-18. | `cargo test -p service_impl test::booking` MUSS grün bleiben. |
| `rest/src/booking.rs` | `POST /booking` + `POST /booking/copy` bleiben unverändert. Frontend-Migration auf den neuen konflikt-aware-Endpunkt liegt im Frontend-Workstream. | `jj diff rest/src/booking.rs` MUSS leer bleiben. |

---

## Pattern Assignments

### Wave 1 — Domain-Surface

#### `service/src/warning.rs` (NEW — Domain-Enum)

**Analog:** `service/src/absence.rs:27-32` (Layout für Domain-Enum) + `03-RESEARCH.md` Pattern 4 (Tag-Enum-Skizze).

**Begründung der Modul-Wahl (C-Phase3-01):** eigenes `warning.rs`-Modul, weil
`Warning` semantisch funktional anders ist als `ServiceError` /
`ValidationFailureItem` (Erfolgs-Pfad statt Fehler-Pfad) UND zwischen mehreren
Services (`AbsenceService` + `ShiftplanEditService`) geteilt wird. Inline in
`service/src/lib.rs` würde den Re-Export wachsen lassen und den Fehler-Block
verwischen.

**Code-Excerpt — Modul-Header + Enum-Definition** (zu erstellen, Skizze aus 03-CONTEXT.md `<specifics>` + Pattern aus `service/src/absence.rs:27-32`):

```rust
//! Domain-Warnings für Phase-3 (Booking ⇄ Absence ⇄ ManualUnavailable Cross-Source-Konflikte).
//!
//! `Warning` ist Erfolgs-Pfad — sie wird in den Wrapper-Result-Structs
//! `BookingCreateResult` (in `service::shiftplan_edit`) und
//! `AbsencePeriodCreateResult` (in `service::absence`) propagiert. KEIN
//! `ServiceError`-Pfad. KEIN ValidationFailureItem (das wäre 422; Warnings
//! sind 200/201 mit Liste).
//!
//! Granularität (D-Phase3-15): eine Warning pro betroffenem Booking-Tag.

use std::sync::Arc;

use shifty_utils::DayOfWeek;
use time::Date;
use uuid::Uuid;

use crate::absence::AbsenceCategory;

/// Cross-Source-Konflikt-Warning. Vier Varianten, jede trägt nur die für die
/// jeweilige Quelle relevanten Felder. Frontend rendert eine Liste.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    BookingOnAbsenceDay {
        booking_id: Uuid,
        date: Date,
        absence_id: Uuid,
        category: AbsenceCategory,
    },
    BookingOnUnavailableDay {
        booking_id: Uuid,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
    },
    AbsenceOverlapsBooking {
        absence_id: Uuid,
        booking_id: Uuid,
        date: Date,
    },
    AbsenceOverlapsManualUnavailable {
        absence_id: Uuid,
        unavailable_id: Uuid,
    },
}
```

**Notes:** D-Phase3-14 fixes die 4 Varianten. KEINE 5. Variante in Phase 3
(D-Phase3-17, deferred). Hash/Eq nur falls dedupe nötig — Default ist nicht-Hash.

---

#### `service/src/lib.rs` (PATCH — Re-Export)

**Analog:** `service/src/lib.rs:7-42` (existing `pub mod`-Reihe).

**Code-Excerpt — Diff:**

```rust
// VORHER (Z. 28-32 ungefähr):
pub mod sales_person_unavailable;
pub mod scheduler;
pub mod session;
pub mod shiftplan;
pub mod shiftplan_catalog;

// NACHHER — Phase 3 ergänzt EINE Zeile (alphabetisch zwischen `user_invitation` und `user_service`,
// oder am Ende der Liste vor `pub use permission::*`):
pub mod warning;

// Optional zusätzlich:
// pub use warning::Warning;
```

**Source:** `service/src/lib.rs:7-47`

---

#### `service/src/shiftplan_edit.rs` (PATCH — Wrapper-Structs + 2 Trait-Methoden)

**Analog A (Wrapper-Struct):** `service/src/absence.rs:118-122` (`ResolvedAbsence`-Struct lebt im selben Service-Modul wie der Trait).

**Analog B (Trait-Methode-Signatur):** `service/src/shiftplan_edit.rs:48-56` (`add_vacation` — Methoden-Signatur-Pattern für ShiftplanEditService).

**Code-Excerpt — Wrapper-Structs + neue Trait-Methoden** (zu ergänzen NACH Z. 56):

```rust
// service/src/shiftplan_edit.rs — am Ende des Files NACH dem Trait

/// Wrapper-Result für `book_slot_with_conflict_check`. Enthält das
/// persistierte Booking + alle Cross-Source-Warnings, die für diesen Tag
/// detektiert wurden.
#[derive(Debug, Clone)]
pub struct BookingCreateResult {
    pub booking: crate::booking::Booking,
    pub warnings: Arc<[crate::warning::Warning]>,
}

/// Wrapper-Result für `copy_week_with_conflict_check`. Aggregiert pro
/// kopiertem Booking alle Warnings (D-Phase3-15: KEINE De-Dup).
#[derive(Debug, Clone)]
pub struct CopyWeekResult {
    pub copied_bookings: Arc<[crate::booking::Booking]>,
    pub warnings: Arc<[crate::warning::Warning]>,
}

// Innerhalb des Traits ShiftplanEditService — 2 neue Methoden ergänzen:
//
//     async fn book_slot_with_conflict_check(
//         &self,
//         booking: &crate::booking::Booking,
//         context: Authentication<Self::Context>,
//         tx: Option<Self::Transaction>,
//     ) -> Result<BookingCreateResult, ServiceError>;
//
//     async fn copy_week_with_conflict_check(
//         &self,
//         from_calendar_week: u8,
//         from_year: u32,
//         to_calendar_week: u8,
//         to_year: u32,
//         context: Authentication<Self::Context>,
//         tx: Option<Self::Transaction>,
//     ) -> Result<CopyWeekResult, ServiceError>;
```

**Notes:** Das `#[automock]` auf dem Trait (Z. 10) deckt automatisch die neuen
Methoden — `MockShiftplanEditService` wird die Methoden selbst-generieren.
Naming Plan-Phase darf abweichen (Q-Open-4: `book_slot_with_warnings` /
`copy_week_with_warnings` als kürzere Alternative). Die Methoden-Signatur
folgt dem Repo-Pattern `&self, ..., context: Authentication<Self::Context>, tx: Option<Self::Transaction>`.

---

#### `service/src/absence.rs` (PATCH — Wrapper-Struct + Trait-Method + Sig-Bruch)

**Analog A (Wrapper-Struct):** `service/src/absence.rs:118-122` (`ResolvedAbsence` direkt in dieser Datei).

**Analog B (Sig-Bruch + neue Trait-Method):** `service/src/absence.rs:154-166` (`create` + `update`-Signaturen).

**Code-Excerpt — neuer Wrapper-Struct + neue Trait-Method + Sig-Bruch:**

```rust
// service/src/absence.rs — NEU vor `pub trait AbsenceService` (Z. 124):

/// Output von `AbsenceService::create` und `AbsenceService::update`. Enthält
/// die persistierte AbsencePeriod + alle Forward-Warnings für die NEUE Range
/// (D-Phase3-04: kein Diff-Modus, alle Bookings + ManualUnavailables in der
/// neuen Range produzieren Warnings).
#[derive(Debug, Clone)]
pub struct AbsencePeriodCreateResult {
    pub absence: AbsencePeriod,
    pub warnings: Arc<[crate::warning::Warning]>,
}

// Innerhalb des Traits AbsenceService — Sig-Bruch und 1 neue Methode:

    // VORHER (Z. 154-159):
    // async fn create(...) -> Result<AbsencePeriod, ServiceError>;
    // NACHHER:
    async fn create(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriodCreateResult, ServiceError>;

    // VORHER (Z. 161-166):
    // async fn update(...) -> Result<AbsencePeriod, ServiceError>;
    // NACHHER:
    async fn update(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriodCreateResult, ServiceError>;

    // NEU — Cross-Kategorie-Range-Lookup für externe Konsumenten
    // (`ShiftplanEditService::book_slot_with_conflict_check` ruft das hier).
    // Permission HR ∨ verify_user_is_sales_person (D-Phase3-12).
    async fn find_overlapping_for_booking(
        &self,
        sales_person_id: Uuid,
        range: shifty_utils::DateRange,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;
```

**Notes:** Sig-Bruch ist Compiler-erzwungen — alle Konsumenten von
`AbsenceService::create`/`update` müssen migriert werden. Das ist die
gewünschte Detektion (REST-Handler in `rest/src/absence.rs`,
`rest/src/shiftplan_edit.rs::add_vacation`-Body in
`service_impl/src/shiftplan_edit.rs`). Plan-Phase identifiziert ALLE Call-Sites
und passt sie an.

---

#### `service/src/shiftplan.rs` (PATCH — UnavailabilityMarker + ShiftplanDay-Erweiterung + 2 Methoden)

**Analog A (Enum-Layout):** `service/src/absence.rs:27-32` (3-Variant-Enum).

**Analog B (Field-Add):** `service/src/shiftplan.rs:14-18` (heutiger `ShiftplanDay`).

**Analog C (neue Methoden-Signatur):** `service/src/shiftplan.rs:53-69` (`get_shiftplan_week` + `get_shiftplan_day`).

**Code-Excerpt — Diff:**

```rust
// service/src/shiftplan.rs

// NEU vor `ShiftplanDay`:
/// Per-Tag-Marker für die per-sales-person-Sicht (D-Phase3-10). Bei
/// Doppel-Quelle (AbsencePeriod UND ManualUnavailable am selben Tag) wird
/// `Both` gesetzt — die `absence_id`/`category` der AbsencePeriod werden
/// mitgeführt, weil sie semantisch reicher als der bloße ManualUnavailable-
/// Eintrag sind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnavailabilityMarker {
    AbsencePeriod {
        absence_id: uuid::Uuid,
        category: crate::absence::AbsenceCategory,
    },
    ManualUnavailable,
    Both {
        absence_id: uuid::Uuid,
        category: crate::absence::AbsenceCategory,
    },
}

// PATCH ShiftplanDay (Z. 14-18) — neues Feld `unavailable`:
#[derive(Debug, Clone)]
pub struct ShiftplanDay {
    pub day_of_week: DayOfWeek,
    pub slots: Vec<ShiftplanSlot>,
    pub unavailable: Option<UnavailabilityMarker>, // NEU für Phase 3
}

// Im Trait `ShiftplanViewService` — 2 neue Methoden ergänzen NACH Z. 69:

    /// Per-sales-person-Variante von `get_shiftplan_week`. Permission HR ∨
    /// `verify_user_is_sales_person(sales_person_id)` (D-Phase3-12).
    /// `unavailable: Option<UnavailabilityMarker>` ist pro Tag gesetzt, wenn
    /// für `sales_person_id` eine aktive AbsencePeriod und/oder ein aktiver
    /// `sales_person_unavailable`-Eintrag existiert.
    async fn get_shiftplan_week_for_sales_person(
        &self,
        shiftplan_id: uuid::Uuid,
        year: u32,
        week: u8,
        sales_person_id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError>;

    async fn get_shiftplan_day_for_sales_person(
        &self,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
        sales_person_id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanDayAggregate, ServiceError>;
```

**Notes:** Field-Add auf `ShiftplanDay` ist additiv — der existing
`ShiftplanDayTO::from(&ShiftplanDay)` in `rest-types/src/lib.rs:1008-1015`
muss um die `unavailable: day.unavailable.as_ref().map(...)`-Conversion
ergänzt werden. **Build-Bruch:** alle Construction-Sites von `ShiftplanDay`
(in `build_shiftplan_day` Z. 104-107 und ggf. Tests) müssen `unavailable: None`
explizit setzen — Compiler hilft.

---

#### `dao/src/absence.rs` (PATCH — neue Trait-Method)

**Analog:** `dao/src/absence.rs:78-85` (heutiger `find_overlapping`, kategorie-scoped).

**Code-Excerpt — neue Trait-Methode (zu ergänzen NACH Z. 85, vor `create`):**

```rust
    /// Findet aktive Absence-Periods derselben `sales_person_id`, die `range`
    /// inklusiv überlappen — **kategorie-frei** (alle 3 AbsenceCategory-Werte
    /// werden zurückgegeben). Verwendet vom `AbsenceService::find_overlapping_for_booking`-
    /// Pfad und vom `ShiftplanEditService::book_slot_with_conflict_check`-Pfad
    /// (Phase 3, D-Phase3-05).
    ///
    /// Nutzt den bestehenden Composite-Index
    /// `idx_absence_period_sales_person_from(sales_person_id, from_date)
    ///  WHERE deleted IS NULL` (Phase-1-D-04). Single-Roundtrip auch bei
    /// copy_week-Loops; Performance-skalierbar.
    async fn find_overlapping_for_booking(
        &self,
        sales_person_id: Uuid,
        range: DateRange,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;
```

**Notes:** `#[automock]` auf dem Trait (Z. 43) deckt die neue Methode
automatisch. KEIN `exclude_logical_id`-Parameter — Booking-IDs sind orthogonal
zu Absence-IDs (RESEARCH.md Q9), Self-Match unmöglich.

---

#### `dao_impl_sqlite/src/absence.rs` (PATCH — SQLx-Impl)

**Analog:** `dao_impl_sqlite/src/absence.rs:155-204` (`find_overlapping`, two-branch). Phase-3 nimmt die `None`-Branch (ohne `exclude_logical_id`) und entfernt zusätzlich den Category-Filter.

**Code-Excerpt — neue Impl-Methode (Single-Branch, kein Category-Filter):**

```rust
async fn find_overlapping_for_booking(
    &self,
    sales_person_id: Uuid,
    range: DateRange,
    tx: Self::Transaction,
) -> Result<Arc<[AbsencePeriodEntity]>, DaoError> {
    let sp_vec = sales_person_id.as_bytes().to_vec();
    // ISO-8601 YYYY-MM-DD; lex-sort == date-sort.
    let from_str = range.from().format(&Iso8601::DATE)?;
    let to_str = range.to().format(&Iso8601::DATE)?;

    Ok(query_as!(
        AbsencePeriodDb,
        "SELECT id, logical_id, sales_person_id, category, from_date, to_date, description, created, deleted, update_version \
         FROM absence_period \
         WHERE sales_person_id = ? \
           AND from_date <= ? \
           AND to_date >= ? \
           AND deleted IS NULL \
         ORDER BY from_date",
        sp_vec,
        to_str,   // range.to → from_date <= range.to
        from_str, // range.from → to_date >= range.from
    )
    .fetch_all(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?
    .iter()
    .map(AbsencePeriodEntity::try_from)
    .collect::<Result<Arc<[_]>, _>>()?)
}
```

**Notes:**
- **Pitfall 1 (SC4):** Das `WHERE deleted IS NULL`-Prädikat ist **Pflicht** — sonst
  triggern soft-deleted AbsencePeriods Warnings, und der composite index
  `idx_absence_period_sales_person_from ... WHERE deleted IS NULL` wird ohne
  das Prädikat nicht genutzt.
- **Allen-Algebra-Range-Match:** `from_date <= range.to AND to_date >= range.from`
  — exakt wie in `find_overlapping` (Z. 174 / 188-192).
- **sqlx-prepare:** Nach Hinzufügen MUSS `nix-shell -p sqlx-cli --run "cargo sqlx prepare --workspace -- --tests"` laufen (Phase-3-Wave-1-Step), sonst bleibt der `.sqlx/`-Cache veraltet.

---

### Wave 2 — Service-Logik + DI

#### `service_impl/src/absence.rs` (PATCH — gen_service_impl-Erweiterung + Forward-Warning)

**Analog A (gen_service_impl-Erweiterung):** `service_impl/src/absence.rs:39-50` (existing 8 Felder).

**Analog B (Forward-Warning-Body):** 03-RESEARCH.md "Operation 2" + `service_impl/src/absence.rs:137-202` (existing `create`-Body).

**Code-Excerpt A — gen_service_impl-Diff:**

```rust
// service_impl/src/absence.rs:39-50 — ERGÄNZT um zwei neue Felder:

gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,

        // NEU für Phase 3 (D-Phase3-08):
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonUnavailableService: service::sales_person_unavailable::SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
    }
}
```

**Code-Excerpt B — Forward-Warning-Insertion in `create`** (zu ergänzen ZWISCHEN Z. 197 (DAO-create) und Z. 200 (commit), nutzt die NEUE Range + `entity.id` (kürzlich oben gesetzt Z. 189)):

```rust
// service_impl/src/absence.rs::AbsenceServiceImpl<Deps>::create

// Bestehende Logik bis incl. Z. 198 (DAO-create) bleibt unverändert.

// NEU — Forward-Warning-Loop für die NEUE Range:
let mut warnings: Vec<crate::warning::Warning> = Vec::new();
let new_range = DateRange::new(entity.from_date, entity.to_date)
    .map_err(|_| ServiceError::DateOrderWrong(entity.from_date, entity.to_date))?;

// 1) Bookings im Range — Loop über betroffene Kalenderwochen (C-Phase3-02-Default).
let mut weeks_seen: BTreeSet<(u32, u8)> = BTreeSet::new();
for day in new_range.iter_days() {
    let (iso_year, iso_week, _) = day.to_iso_week_date();
    if !weeks_seen.insert((iso_year as u32, iso_week)) {
        continue;
    }
    let bookings = self
        .booking_service
        .get_for_week(iso_week, iso_year as u32, Authentication::Full, tx.clone().into())
        .await?;
    for b in bookings.iter() {
        if b.sales_person_id != entity.sales_person_id {
            continue;
        }
        // Slot lookup für day_of_week → konkretes Date
        let slot = self
            .slot_service
            .get_slot(&b.slot_id, Authentication::Full, tx.clone().into())
            .await?;
        let booking_date = time::Date::from_iso_week_date(
            b.year as i32,
            b.calendar_week as u8,
            slot.day_of_week.into(),
        )?;
        if !new_range.contains(booking_date) {
            continue;
        }
        warnings.push(crate::warning::Warning::AbsenceOverlapsBooking {
            absence_id: entity.id,
            booking_id: b.id,
            date: booking_date,
        });
    }
}

// 2) ManualUnavailables im Range — clientside-Filter
//    (C-Phase3-02-Default — Range-DAO-Methode kann später nachgereicht werden)
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
        mu.calendar_week,
        mu.day_of_week.into(),
    )?;
    if !new_range.contains(mu_date) {
        continue;
    }
    warnings.push(crate::warning::Warning::AbsenceOverlapsManualUnavailable {
        absence_id: entity.id,
        unavailable_id: mu.id,
    });
}

self.transaction_dao.commit(tx).await?;
Ok(AbsencePeriodCreateResult {
    absence: entity,
    warnings: Arc::from(warnings),
})
```

**Code-Excerpt C — `update` symmetrisch:** Loop läuft NACH der Tombstone+Insert-Logik
(Z. 262-292), VOR `commit` (Z. 294). Symmetrisch zu `create` mit `entity.id`
ersetzt durch `active.logical_id`. Plan-Phase darf den Loop in einen privaten
Helper `compute_forward_warnings(&self, absence_id, sales_person_id, new_range, tx)`
extrahieren (DRY). **D-Phase3-04:** Warnings für ALLE Tage in der NEUEN Range,
KEIN Diff-Modus.

**Code-Excerpt D — neue Service-Method `find_overlapping_for_booking`:**

```rust
async fn find_overlapping_for_booking(
    &self,
    sales_person_id: Uuid,
    range: DateRange,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // Permission HR ∨ self (Pattern 3 — gleiche Regel wie find_by_sales_person)
    let (hr, sp) = join!(
        self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            sales_person_id, context, tx.clone().into()
        ),
    );
    hr.or(sp)?;

    let entities = self
        .absence_dao
        .find_overlapping_for_booking(sales_person_id, range, tx.clone())
        .await?;
    let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

**Notes:**
- `entity.id` muss VOR dem Forward-Loop gesetzt sein (Z. 189) — ansonsten enthält
  die Warning Uuid::nil(). Bestehende `create`-Sequenz hat das schon korrekt.
- Plan-Phase darf in `update` die `entity.id` durch `active.logical_id` ersetzen
  (das ist im `update`-Body verfügbar Z. 211).
- BookingService-Calls nutzen `Authentication::Full` als Bypass — Permission ist
  oben in `create`/`update` bereits geprüft (Z. 144-153). Konsistent mit
  Phase-1-Pattern (`active.sales_person_id`-Lookup).

---

#### `service_impl/src/shiftplan_edit.rs` (PATCH — gen_service_impl-Erweiterung + 2 neue Methoden)

**Analog A (gen_service_impl-Diff):** `service_impl/src/shiftplan_edit.rs:22-36` (existing 11 Felder).

**Analog B (Date-Konversion-Pattern):** `service_impl/src/shiftplan_edit.rs:69-70` + `service_impl/src/shiftplan.rs:138`.

**Analog C (Methoden-Body-Layout):** `service_impl/src/shiftplan_edit.rs:43-130` (`modify_slot`-Body — Permission + Slot-Lookup + Date-Konversion + Booking-Operation).

**Code-Excerpt A — gen_service_impl-Diff:**

```rust
// service_impl/src/shiftplan_edit.rs:22-36 — ERGÄNZT um EIN neues Feld
// (BookingService + SalesPersonUnavailableService sind schon Z. 26 + 30):

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
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        // NEU für Phase 3 (D-Phase3-06):
        AbsenceService: service::absence::AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
    }
}
```

**Code-Excerpt B — `book_slot_with_conflict_check` (vollständig, NEU):** siehe 03-RESEARCH.md "Operation 1" (Z. 624-722). Kern-Pattern:

```rust
async fn book_slot_with_conflict_check(
    &self,
    booking: &Booking,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<BookingCreateResult, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // Permission HR ∨ self (Pattern 3, D-Phase3-12)
    let (hr, sp) = join!(
        self.permission_service.check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            booking.sales_person_id, context.clone(), tx.clone().into()
        ),
    );
    hr.or(sp)?;

    // Slot-Lookup — vorhandenes Pattern aus modify_slot Z. 56-59
    let slot = self.slot_service
        .get_slot(&booking.slot_id, Authentication::Full, tx.clone().into())
        .await?;

    // Date-Konversion (Pattern 2 — VOR der Persistierung, IM Business-Logic-Tier)
    let booking_date: time::Date = time::Date::from_iso_week_date(
        booking.year as i32,
        booking.calendar_week as u8,
        slot.day_of_week.into(),
    )?;
    let single_day_range = shifty_utils::DateRange::new(booking_date, booking_date)
        .map_err(|_| ServiceError::DateOrderWrong(booking_date, booking_date))?;

    // Lookup AbsencePeriod-Konflikte (cross-Kategorie via NEUE Service-Method)
    let absence_periods = self
        .absence_service
        .find_overlapping_for_booking(
            booking.sales_person_id,
            single_day_range,
            Authentication::Full, // Permission already verified above
            tx.clone().into(),
        )
        .await?;

    // Lookup ManualUnavailable für die KW
    let manual_unavailables = self
        .sales_person_unavailable_service
        .get_by_week_for_sales_person(
            booking.sales_person_id,
            booking.year,
            booking.calendar_week as u8,
            Authentication::Full,
            tx.clone().into(),
        )
        .await?;

    // Persist via Basic-Service — BookingService::create UNVERÄNDERT
    let persisted_booking = self
        .booking_service
        .create(booking, Authentication::Full, tx.clone().into())
        .await?;

    // Warning-Konstruktion mit echter persistierter Booking-ID
    let mut warnings: Vec<crate::warning::Warning> = Vec::new();
    for ap in absence_periods.iter() {
        warnings.push(crate::warning::Warning::BookingOnAbsenceDay {
            booking_id: persisted_booking.id,
            date: booking_date,
            absence_id: ap.id,
            category: ap.category,
        });
    }
    for mu in manual_unavailables.iter()
        .filter(|mu| mu.day_of_week == slot.day_of_week)
    {
        warnings.push(crate::warning::Warning::BookingOnUnavailableDay {
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

**Code-Excerpt C — `copy_week_with_conflict_check`:** siehe 03-RESEARCH.md
"Q-Open #8 / Pattern" — innerer Loop ruft `book_slot_with_conflict_check`
direkt; Warnings via `extend(result.warnings.iter().cloned())` aggregieren.
KEINE De-Dup (D-Phase3-15 + Pitfall 3). Permission HR ∨ self analog.

**Notes:**
- `BookingService::create` wird mit `&booking` aufgerufen — der existing
  Trait nimmt `&Booking` (siehe `service/src/booking.rs:93-97`).
- `Authentication::Full`-Bypass auf den inneren Service-Calls ist sicher,
  weil oben die strengere `HR ∨ self`-Permission läuft.
- Die `add_vacation`-Methode (Z. 48-56 im Trait) bleibt unverändert — sie ist
  ein anderer Use-Case (legacy ExtraHours-Vacation-Pfad, nicht `AbsencePeriod`).

---

#### `service_impl/src/shiftplan.rs` (PATCH — gen_service_impl + neuer Helper + 2 Methoden)

**Analog A (gen_service_impl-Diff):** `service_impl/src/shiftplan.rs:110-120` (existing 7 Felder).

**Analog B (Helper-Pattern):** `service_impl/src/shiftplan.rs:24-108` (`build_shiftplan_day` — Vorlage; Phase 3 ergänzt PARALLEL einen `build_shiftplan_day_for_sales_person` per C-Phase3-03).

**Analog C (Methoden-Body):** `service_impl/src/shiftplan.rs:127-203` (`get_shiftplan_week`-Body als Vorlage).

**Code-Excerpt A — gen_service_impl-Diff:**

```rust
// service_impl/src/shiftplan.rs:110-120 — ERGÄNZT um zwei neue Felder:

gen_service_impl! {
    struct ShiftplanViewServiceImpl: service::shiftplan::ShiftplanViewService = ShiftplanViewServiceDeps {
        SlotService: service::slot::SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SpecialDayService: service::special_days::SpecialDayService<Context = Self::Context> = special_day_service,
        ShiftplanService: service::shiftplan_catalog::ShiftplanService<Context = Self::Context, Transaction = Self::Transaction> = shiftplan_service,
        PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao,

        // NEU für Phase 3 (D-Phase3-09):
        AbsenceService: service::absence::AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
        SalesPersonUnavailableService: service::sales_person_unavailable::SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
    }
}
```

**Code-Excerpt B — `build_shiftplan_day_for_sales_person` (NEUER Parallel-Helper, NACH Z. 108):**

Siehe 03-RESEARCH.md "Operation 4" (Z. 861-913). Kern-Pattern:

```rust
pub(crate) fn build_shiftplan_day_for_sales_person(
    day_of_week: DayOfWeek,
    day_date: time::Date,
    slots: &[Slot],
    bookings: &[Booking],
    sales_persons: &[SalesPerson],
    special_days: &[SpecialDay],
    user_assignments: Option<&HashMap<Uuid, Arc<str>>>,
    sales_person_id: Uuid,
    absence_periods: &[service::absence::AbsencePeriod],
    manual_unavailables: &[service::sales_person_unavailable::SalesPersonUnavailable],
) -> Result<ShiftplanDay, ServiceError> {
    // Reuse für Slots/Bookings/Holiday-Filter:
    let mut day = build_shiftplan_day(
        day_of_week, slots, bookings, sales_persons, special_days, user_assignments,
    )?;

    // De-Dup-Match (D-Phase3-10): None / AbsencePeriod / ManualUnavailable / Both
    let absence_match = absence_periods.iter().find(|ap| {
        ap.deleted.is_none()
            && ap.sales_person_id == sales_person_id
            && ap.from_date <= day_date
            && day_date <= ap.to_date
    });
    let manual_match = manual_unavailables.iter().any(|mu| {
        mu.deleted.is_none()
            && mu.sales_person_id == sales_person_id
            && mu.day_of_week == day_of_week
    });

    day.unavailable = match (absence_match, manual_match) {
        (Some(ap), false) => Some(service::shiftplan::UnavailabilityMarker::AbsencePeriod {
            absence_id: ap.id,
            category: ap.category,
        }),
        (None, true) => Some(service::shiftplan::UnavailabilityMarker::ManualUnavailable),
        (Some(ap), true) => Some(service::shiftplan::UnavailabilityMarker::Both {
            absence_id: ap.id,
            category: ap.category,
        }),
        (None, false) => None,
    };

    Ok(day)
}
```

**Code-Excerpt C — `get_shiftplan_week_for_sales_person` (NEUE Method):**
Vorlage `get_shiftplan_week` Z. 127-203. Diff:
1. **Permission HR ∨ self** statt nur `SHIFTPLANNER_PRIVILEGE`-Check.
2. **Zusätzliche Reads:** `absence_service.find_by_sales_person(sales_person_id, ...)` (filterclient-side auf overlap) UND `sales_person_unavailable_service.get_by_week_for_sales_person(sales_person_id, year, week, ...)`.
3. **Tag-Loop nutzt `build_shiftplan_day_for_sales_person`** statt `build_shiftplan_day`. `day_date = time::Date::from_iso_week_date(year as i32, week, day_of_week.into())?` pro Tag inline.
4. **Bestehender `get_shiftplan_week` bleibt unangetastet** (additive Erweiterung).

**Notes:**
- **Bestehender `build_shiftplan_day` bleibt unverändert.** Phase 3 ergänzt
  einen Parallel-Helper (C-Phase3-03), damit die Globalsicht-Tests
  unverändert grün bleiben.
- `ShiftplanDay { ..., unavailable: None }` wird als Default in
  `build_shiftplan_day` gesetzt — der Helper läuft wie bisher und liefert
  immer `unavailable: None`. Plan-Phase ergänzt das eine Zeile in Z. 104-107.

---

#### `shifty_bin/src/main.rs` (PATCH — DI-Wiring)

**Analog:** `shifty_bin/src/main.rs:678-839` (existing Konstruktion). Konstruktionsreihenfolge ist heute schon Tier-konform: Basic vor Business-Logic. Phase 3 ergänzt nur `clone()`-Pässe in den DI-Structs.

**Code-Excerpt — Diff:**

```rust
// shifty_bin/src/main.rs:737-746 — ERGÄNZT um ZWEI neue clone-Pässe:
let absence_service = Arc::new(service_impl::absence::AbsenceServiceImpl {
    absence_dao,
    permission_service: permission_service.clone(),
    sales_person_service: sales_person_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    special_day_service: special_day_service.clone(),
    employee_work_details_service: working_hours_service.clone(),
    transaction_dao: transaction_dao.clone(),
    // NEU für Phase 3:
    booking_service: booking_service.clone(),
    sales_person_unavailable_service: sales_person_unavailable_service.clone(),
});

// shifty_bin/src/main.rs:808-821 — ERGÄNZT um EINEN neuen clone-Pass:
let shiftplan_edit_service =
    Arc::new(service_impl::shiftplan_edit::ShiftplanEditServiceImpl {
        permission_service: permission_service.clone(),
        slot_service: slot_service.clone(),
        booking_service: booking_service.clone(),
        sales_person_service: sales_person_service.clone(),
        employee_work_details_service: working_hours_service.clone(),
        carryover_service: carryover_service.clone(),
        reporting_service: reporting_service.clone(),
        uuid_service: uuid_service.clone(),
        transaction_dao: transaction_dao.clone(),
        extra_hours_service: extra_hours_service.clone(),
        sales_person_unavailable_service: sales_person_unavailable_service.clone(),
        // NEU für Phase 3:
        absence_service: absence_service.clone(),
    });

// shifty_bin/src/main.rs:831-839 — ERGÄNZT um ZWEI neue clone-Pässe:
let shiftplan_view_service = Arc::new(service_impl::shiftplan::ShiftplanViewServiceImpl {
    slot_service: slot_service.clone(),
    booking_service: booking_service.clone(),
    sales_person_service: sales_person_service.clone(),
    special_day_service: special_day_service.clone(),
    shiftplan_service: shiftplan_service.clone(),
    permission_service: permission_service.clone(),
    transaction_dao: transaction_dao.clone(),
    // NEU für Phase 3:
    absence_service: absence_service.clone(),
    sales_person_unavailable_service: sales_person_unavailable_service.clone(),
});
```

**Notes:**
- **`booking_service` ist Z. 699 konstruiert** — VOR `absence_service` (Z. 737)
  und `shiftplan_edit_service` (Z. 808). Topologische Reihenfolge erfüllt.
  Plan-Phase verifiziert das mit einem grep.
- **`sales_person_unavailable_service` ist Z. 678 konstruiert** — VOR allem.
- **`absence_service` ist Z. 737 konstruiert** — VOR `shiftplan_edit_service`
  (Z. 808) und `shiftplan_view_service` (Z. 831). Plan-Phase MUSS sicherstellen,
  dass `shiftplan_view_service` NACH `absence_service` konstruiert wird.

---

### Wave 3 — REST + ApiDoc

#### `rest-types/src/lib.rs` (PATCH — inline DTOs + From-Impls)

**Analog A (Tag-Enum-Layout):** RESEARCH.md Pattern 4 (Z. 411-447) + `rest-types/src/lib.rs:240-280` (`DayOfWeekTO`, allerdings als plain enum, nicht tag-content).

**Analog B (Wrapper-DTO-Layout):** `rest-types/src/lib.rs:1572-1620` (`AbsencePeriodTO` + From-Impls).

**Code-Excerpt — neue inline DTOs (zu ergänzen am Ende des Files vor letzter `}` oder NACH AbsencePeriod-Block Z. 1620):**

```rust
// rest-types/src/lib.rs — Phase-3 NEUE inline DTOs

// ──────────────────────────────────────────────────────────────────────
// Warning (Phase 3 — Cross-Source Konflikt-Hinweise; Erfolgs-Pfad)
// ──────────────────────────────────────────────────────────────────────

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

#[cfg(feature = "service-impl")]
impl From<&service::warning::Warning> for WarningTO {
    fn from(w: &service::warning::Warning) -> Self {
        match w {
            service::warning::Warning::BookingOnAbsenceDay { booking_id, date, absence_id, category } => Self::BookingOnAbsenceDay {
                booking_id: *booking_id,
                date: *date,
                absence_id: *absence_id,
                category: category.into(),
            },
            service::warning::Warning::BookingOnUnavailableDay { booking_id, year, week, day_of_week } => Self::BookingOnUnavailableDay {
                booking_id: *booking_id,
                year: *year,
                week: *week,
                day_of_week: (*day_of_week).into(),
            },
            service::warning::Warning::AbsenceOverlapsBooking { absence_id, booking_id, date } => Self::AbsenceOverlapsBooking {
                absence_id: *absence_id,
                booking_id: *booking_id,
                date: *date,
            },
            service::warning::Warning::AbsenceOverlapsManualUnavailable { absence_id, unavailable_id } => Self::AbsenceOverlapsManualUnavailable {
                absence_id: *absence_id,
                unavailable_id: *unavailable_id,
            },
        }
    }
}

// AbsenceCategoryTO::from(&AbsenceCategory) (Z. 1551 existing) —
// für die Warning-Conversion brauchen wir AbsenceCategoryTO::from(&AbsenceCategory)
// ohne Owned-Kopie. Existing impl nimmt schon `&service::absence::AbsenceCategory`,
// passt ohne Anpassung.

// ──────────────────────────────────────────────────────────────────────
// UnavailabilityMarker (Phase 3 — per-Tag-Marker für ShiftplanDay)
// ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum UnavailabilityMarkerTO {
    AbsencePeriod {
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
    ManualUnavailable,
    Both {
        absence_id: Uuid,
        category: AbsenceCategoryTO,
    },
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan::UnavailabilityMarker> for UnavailabilityMarkerTO {
    fn from(m: &service::shiftplan::UnavailabilityMarker) -> Self {
        match m {
            service::shiftplan::UnavailabilityMarker::AbsencePeriod { absence_id, category } => Self::AbsencePeriod {
                absence_id: *absence_id,
                category: category.into(),
            },
            service::shiftplan::UnavailabilityMarker::ManualUnavailable => Self::ManualUnavailable,
            service::shiftplan::UnavailabilityMarker::Both { absence_id, category } => Self::Both {
                absence_id: *absence_id,
                category: category.into(),
            },
        }
    }
}

// PATCH ShiftplanDayTO (Z. 973-977) — neues Feld:
// pub struct ShiftplanDayTO {
//     pub day_of_week: DayOfWeekTO,
//     pub slots: Vec<ShiftplanSlotTO>,
//     pub unavailable: Option<UnavailabilityMarkerTO>, // NEU
// }
//
// Und das From-Impl Z. 1008-1015 entsprechend ergänzen:
//   unavailable: day.unavailable.as_ref().map(UnavailabilityMarkerTO::from),

// ──────────────────────────────────────────────────────────────────────
// Wrapper-Result-DTOs (Phase 3)
// ──────────────────────────────────────────────────────────────────────

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

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CopyWeekResultTO {
    pub copied_bookings: Vec<BookingTO>,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::shiftplan_edit::CopyWeekResult> for CopyWeekResultTO {
    fn from(r: &service::shiftplan_edit::CopyWeekResult) -> Self {
        Self {
            copied_bookings: r.copied_bookings.iter().map(BookingTO::from).collect(),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AbsencePeriodCreateResultTO {
    pub absence: AbsencePeriodTO,
    pub warnings: Vec<WarningTO>,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsencePeriodCreateResult> for AbsencePeriodCreateResultTO {
    fn from(r: &service::absence::AbsencePeriodCreateResult) -> Self {
        Self {
            absence: AbsencePeriodTO::from(&r.absence),
            warnings: r.warnings.iter().map(WarningTO::from).collect(),
        }
    }
}
```

**Notes:**
- **Tag-Enum-Format:** `#[serde(tag = "kind", content = "data", rename_all = "snake_case")]`
  → JSON: `{ "kind": "booking_on_absence_day", "data": {...} }`. Pattern 4 / Q3
  bestätigt utoipa-5-Support.
- **Repo-Konvention:** Alle DTOs inline in `lib.rs` (Phase-1-Override). KEINE
  separate Datei.
- **`AbsenceCategoryTO::from(&AbsenceCategory)`** (`rest-types/src/lib.rs:1551`)
  ist schon vorhanden — Phase 3 nutzt es ohne Anpassung.
- **`ShiftplanDayTO`-Field-Add (Z. 974):** Compiler-Bruch erzwingt Update der
  From-Impl Z. 1008-1015. Alle Konsumenten müssen das `unavailable`-Feld
  bedienen.

---

#### `rest/src/absence.rs` (PATCH — Wrapper-DTO für POST + PATCH)

**Analog:** `rest/src/absence.rs:43-74` (heutiger `create_absence_period`).

**Code-Excerpt — Diff:**

```rust
// rest/src/absence.rs:43-74 — RETURN-TYPE-Diff: AbsencePeriodTO → AbsencePeriodCreateResultTO

#[utoipa::path(
    post,
    path = "",
    tags = ["Absence"],
    request_body = AbsencePeriodTO,
    responses(
        (status = 201, description = "Absence period created (with warnings if any)", body = AbsencePeriodCreateResultTO), // CHANGED
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn create_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            // Service liefert jetzt AbsencePeriodCreateResult (Wrapper), nicht AbsencePeriod
            let result = svc.create(&(&body).into(), context.into(), None).await?;
            let to = AbsencePeriodCreateResultTO::from(&result);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

`update_absence_period` (Z. 152-174) symmetrisch — Status 200, Body
`AbsencePeriodCreateResultTO`. ApiDoc-Schema-Liste (Z. 246) ergänzt:
`AbsencePeriodCreateResultTO`, `WarningTO`, `AbsenceCategoryTO` (existing).

**Notes:**
- Status-Code bleibt 201/200 — Frontend prüft `result.warnings.is_empty()`
  (D-Phase3-03).
- Pitfall 6: Bruch betrifft nur Absence-Endpunkte; `POST /booking` bleibt
  unverändert. Frontend-Migration im Frontend-Workstream.
- `delete_absence_period`, `find_*`-Handler bleiben unverändert.

---

#### `rest/src/shiftplan_edit.rs` (PATCH — 2 neue Endpunkte)

**Analog A (Wrapper-Result-Handler):** `rest/src/absence.rs:55-74` (POST mit Body-Wrapper).

**Analog B (Bestehender Service-Call-Pattern):** `rest/src/shiftplan_edit.rs:25-46` (existing `edit_slot`-Handler).

**Code-Excerpt — neue Endpunkte (zu ergänzen am Ende der Datei):**

```rust
// rest/src/shiftplan_edit.rs — Phase 3 NEUE Endpunkte

// 1) Route ergänzen in generate_route() (Z. 15-23):
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/slot/{year}/{week}", put(edit_slot::<RestState>))
        .route("/slot/{slot_id}/{year}/{week}", delete(delete_slot::<RestState>))
        .route("/vacation", put(add_vacation::<RestState>))
        // NEU für Phase 3:
        .route("/booking", post(book_slot_with_conflict_check::<RestState>))
        .route("/copy-week", post(copy_week_with_conflict_check::<RestState>))
}

// 2) Neuer Handler — POST /shiftplan-edit/booking
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/booking",
    tags = ["ShiftplanEdit"],
    request_body = BookingTO,
    responses(
        (status = 201, description = "Booking created (with cross-source warnings if any)", body = BookingCreateResultTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn book_slot_with_conflict_check<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(booking): Json<BookingTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.shiftplan_edit_service();
            let result = svc
                .book_slot_with_conflict_check(&Booking::from(&booking), context.into(), None)
                .await?;
            let to = BookingCreateResultTO::from(&result);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

// 3) Neuer Handler — POST /shiftplan-edit/copy-week
#[derive(Debug, Deserialize)]
pub struct CopyWeekRequest {
    pub from_year: u32,
    pub from_calendar_week: u8,
    pub to_year: u32,
    pub to_calendar_week: u8,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/copy-week",
    tags = ["ShiftplanEdit"],
    request_body = CopyWeekRequest,
    responses(
        (status = 200, description = "Bookings copied (with cross-source warnings if any)", body = CopyWeekResultTO),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn copy_week_with_conflict_check<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(req): Json<CopyWeekRequest>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.shiftplan_edit_service();
            let result = svc
                .copy_week_with_conflict_check(
                    req.from_calendar_week, req.from_year,
                    req.to_calendar_week, req.to_year,
                    context.into(), None,
                )
                .await?;
            let to = CopyWeekResultTO::from(&result);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

// 4) ApiDoc-Struct (NEU — existing rest/src/shiftplan_edit.rs hat keine ApiDoc):
#[derive(OpenApi)]
#[openapi(
    paths(
        edit_slot,
        delete_slot,
        add_vacation,
        book_slot_with_conflict_check,
        copy_week_with_conflict_check,
    ),
    components(schemas(
        SlotTO, VacationPayloadTO, BookingTO,
        BookingCreateResultTO, CopyWeekResultTO, WarningTO, AbsenceCategoryTO, DayOfWeekTO,
        CopyWeekRequest,
    )),
    tags(
        (name = "ShiftplanEdit", description = "Shiftplan edit operations (conflict-aware)"),
    ),
)]
pub struct ShiftplanEditApiDoc;
```

**Notes:**
- **Naming-Wahl C-Phase3-09:** Variante "Erweiterung des bestehenden
  `rest/src/shiftplan_edit.rs`" mit Routen `/booking` und `/copy-week`
  innerhalb der Route-Gruppe `/shiftplan-edit`. Plan-Phase darf
  `rest/src/shiftplan_edit_booking.rs` als separate Datei machen — beide
  Optionen sind tier-konform.
- **`use`-Imports** ergänzen: `axum::routing::post`, `service::booking::Booking`,
  `rest_types::{BookingTO, BookingCreateResultTO, CopyWeekResultTO, WarningTO,
  AbsenceCategoryTO, DayOfWeekTO}`, `axum::Json`, `serde::Deserialize`.
- **`add_vacation`** (Z. 67-90) bekommt nachträglich `#[utoipa::path(...)]` —
  empfohlen aber nicht zwingend (existing tech-debt).

---

#### `rest/src/shiftplan.rs` (PATCH — 2 neue per-sales-person-Endpunkte)

**Analog:** `rest/src/shiftplan.rs:34-58` (existing `get_shiftplan_week`).

**Code-Excerpt — neue Endpunkte:**

```rust
// rest/src/shiftplan.rs — Phase 3 ergänzt 2 neue Endpunkte

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/{shiftplan_id}/{year}/{week}", get(get_shiftplan_week::<RestState>))
        .route("/day/{year}/{week}/{day_of_week}", get(get_shiftplan_day::<RestState>))
        // NEU für Phase 3 (D-Phase3-12):
        .route(
            "/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}",
            get(get_shiftplan_week_for_sales_person::<RestState>),
        )
        .route(
            "/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}",
            get(get_shiftplan_day_for_sales_person::<RestState>),
        )
}

#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}",
    params(
        ("shiftplan_id" = Uuid, Path, description = "Shift plan ID"),
        ("year" = u32, Path, description = "Year"),
        ("week" = u8, Path, description = "Calendar week (1-53)"),
        ("sales_person_id" = Uuid, Path, description = "Sales person id (HR ∨ self)"),
    ),
    responses(
        (status = 200, description = "Shift plan week with per-day unavailable marker", body = ShiftplanWeekTO),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_week_for_sales_person<RestState: RestStateDef>(
    Path((shiftplan_id, year, week, sales_person_id)): Path<(Uuid, u32, u8, Uuid)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let week = rest_state
                .shiftplan_view_service()
                .get_shiftplan_week_for_sales_person(
                    shiftplan_id, year, week, sales_person_id,
                    Authentication::Context(context),
                    None,
                )
                .await?;
            let to = ShiftplanWeekTO::from(&week);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&to).unwrap()))
                .unwrap())
        }
        .await,
    )
}

// get_shiftplan_day_for_sales_person — symmetrisch
```

**ApiDoc-Schemas-Liste (Z. 108-110) ergänzen:** `UnavailabilityMarkerTO`, `AbsenceCategoryTO`.

**Notes:**
- Permission HR ∨ self läuft im Service-Layer (D-Phase3-12). REST passt
  nur `Authentication::Context(context)` weiter.
- Existing `/{shiftplan_id}/{year}/{week}` ist global → bleibt unverändert.

---

#### `rest/src/lib.rs` (PATCH — ApiDoc-Aggregation)

**Analog:** `rest/src/lib.rs:460-484` (existing `nest`-Block).

**Code-Excerpt — Diff:**

```rust
// rest/src/lib.rs:460-484 — ERGÄNZT um EINEN neuen nest-Eintrag:

#[derive(OpenApi)]
#[openapi(
    nest(
        // ... bestehende 19 Einträge ...
        // NEU für Phase 3:
        (path = "/shiftplan-edit", api = shiftplan_edit::ShiftplanEditApiDoc),
    )
)]
pub struct ApiDoc;
```

`absence::AbsenceApiDoc` ist schon nested (Z. 463) — Erweiterung der
`AbsenceApiDoc::components(schemas(...))` (Z. 246 in `rest/src/absence.rs`)
deckt automatisch `AbsencePeriodCreateResultTO`. Ähnlich für
`shiftplan::ShiftplanApiDoc::components`.

---

### Wave 4 — Tests

#### `service_impl/src/test/shiftplan_edit.rs` (NEW — Reverse-Warning-Tests)

**Analog A (Mock-DI-Struktur):** `service_impl/src/test/shiftplan.rs:59-100` (`ShiftplanViewServiceDependencies` — Mock-Pendant zu `gen_service_impl!`).

**Analog B (Mock-Setup-Stil):** `service_impl/src/test/booking.rs:113-192` (`build_dependencies` — slot/uuid/permission-Mocks).

**Analog C (Cross-Service-Mock-Stil):** `service_impl/src/test/absence.rs:200-234` (`expect_find_overlapping().returning(...)`-Pattern).

**Code-Excerpt — Datei-Struktur (zu erstellen):**

```rust
//! Service-Tests für `ShiftplanEditServiceImpl::book_slot_with_conflict_check`
//! und `copy_week_with_conflict_check` (Phase 3).
//!
//! Pflicht-Coverage (siehe 03-RESEARCH.md Q10):
//! - test_book_slot_warning_on_absence_day (Reverse-Warning, AbsencePeriod-Quelle)
//! - test_book_slot_warning_on_manual_unavailable (Reverse-Warning, ManualUnavailable-Quelle)
//! - test_book_slot_no_warning_when_softdeleted_absence (Pitfall-1, Mock returnt empty Vec)
//! - test_copy_week_aggregates_warnings (D-Phase3-02 + Pitfall-3 — KEINE De-Dup)
//! - test_book_slot_with_conflict_check_forbidden (Permission-Gate)
//! - test_copy_week_with_conflict_check_forbidden

use std::sync::Arc;

use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::{
    absence::{AbsenceCategory, AbsencePeriod, MockAbsenceService},
    booking::{Booking, MockBookingService},
    carryover::MockCarryoverService,
    employee_work_details::MockEmployeeWorkDetailsService,
    extra_hours::MockExtraHoursService,
    permission::Authentication,
    reporting::MockReportingService,
    sales_person::MockSalesPersonService,
    sales_person_unavailable::{MockSalesPersonUnavailableService, SalesPersonUnavailable},
    shiftplan_edit::{BookingCreateResult, CopyWeekResult, ShiftplanEditService},
    slot::{MockSlotService, Slot},
    uuid_service::MockUuidService,
    warning::Warning,
    MockPermissionService, ServiceError,
};
use shifty_utils::DayOfWeek;
use time::macros::date;
use uuid::{uuid, Uuid};

use crate::shiftplan_edit::{ShiftplanEditServiceDeps, ShiftplanEditServiceImpl};
use crate::test::error_test::test_forbidden;

// IDs
fn default_sales_person_id() -> Uuid { uuid!("BB000000-0000-0000-0000-000000000001") }
fn default_slot_id() -> Uuid { uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380") }
fn default_absence_id() -> Uuid { uuid!("AB000000-0000-0000-0000-000000000001") }
fn default_booking_id() -> Uuid { uuid!("CEA260A0-112B-4970-936C-F7E529955BD0") }
fn default_unavailable_id() -> Uuid { uuid!("DD000000-0000-0000-0000-000000000001") }

// Mock-DI-Struktur (Pattern aus shiftplan.rs:59-100)
pub struct ShiftplanEditDependencies {
    pub permission_service: MockPermissionService,
    pub slot_service: MockSlotService,
    pub booking_service: MockBookingService,
    pub carryover_service: MockCarryoverService,
    pub reporting_service: MockReportingService,
    pub sales_person_service: MockSalesPersonService,
    pub sales_person_unavailable_service: MockSalesPersonUnavailableService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub extra_hours_service: MockExtraHoursService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
    // NEU für Phase 3:
    pub absence_service: MockAbsenceService,
}

impl ShiftplanEditServiceDeps for ShiftplanEditDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type PermissionService = MockPermissionService;
    type SlotService = MockSlotService;
    type BookingService = MockBookingService;
    type CarryoverService = MockCarryoverService;
    type ReportingService = MockReportingService;
    type SalesPersonService = MockSalesPersonService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type ExtraHoursService = MockExtraHoursService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
    type AbsenceService = MockAbsenceService;
}

// build_service / build_dependencies analog shiftplan.rs:80-100 + booking.rs:99-110

#[tokio::test]
async fn test_book_slot_warning_on_absence_day() {
    let mut deps = build_dependencies();
    // Mock AbsenceService::find_overlapping_for_booking → returns 1 AbsencePeriod
    deps.absence_service
        .expect_find_overlapping_for_booking()
        .returning(|_, _, _, _| Ok(Arc::from([
            AbsencePeriod { /* fixture mit default_absence_id, AbsenceCategory::Vacation */ }
        ])));
    deps.sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| Ok((*b).clone()));
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), Authentication::Full, None)
        .await
        .expect("should succeed");
    assert_eq!(result.warnings.len(), 1);
    assert!(matches!(
        &result.warnings[0],
        Warning::BookingOnAbsenceDay { .. }
    ));
}

// test_book_slot_no_warning_when_softdeleted_absence:
//   Mock returnt empty Vec (DAO würde soft-deleted bereits filtern, siehe Pitfall 8)

// test_copy_week_aggregates_warnings:
//   3 Quell-Bookings via expect_get_for_week, 2 davon via find_overlapping_for_booking → 2 AbsencePeriods
//   Verifiziert assert_eq!(result.warnings.len(), 2) und result.copied_bookings.len() == 3

// test_book_slot_with_conflict_check_forbidden:
//   permission_service.expect_check_permission().returning(|_, _| Err(Forbidden))
//   sales_person_service.expect_verify_user_is_sales_person().returning(|_, _, _| Err(Forbidden))
//   → test_forbidden(&result)
```

**Notes:**
- `MockAbsenceService` wird automatisch von `#[automock]` auf `service::absence::AbsenceService`
  generiert (verifiziert: existing in `service/src/absence.rs:124`).
- Pattern für `expect_*_with_*().returning(|_,_,_,_| ...)` ist 1:1 aus
  `service_impl/src/test/absence.rs:204-216` übernehmbar.

---

#### `service_impl/src/test/absence.rs` (PATCH — Forward-Warning-Tests)

**Analog A (Erweiterung von `AbsenceDependencies`):** `service_impl/src/test/absence.rs:121-194`.

**Diff:** `AbsenceDependencies` bekommt 2 neue Mock-Felder (`MockBookingService`,
`MockSalesPersonUnavailableService`); `build_dependencies` ergänzt Default-
Returns für `expect_get_for_week` und `expect_get_all_for_sales_person`
(Default: leerer `Arc::from([])` — meiste Tests ohne Konflikt).

**Code-Excerpt — Diff:**

```rust
// service_impl/src/test/absence.rs:121-194 — ergänzt:

pub(crate) struct AbsenceDependencies {
    // ... bestehende 8 Felder ...
    // NEU für Phase 3:
    pub booking_service: MockBookingService,
    pub sales_person_unavailable_service: MockSalesPersonUnavailableService,
}

// build_dependencies fügt Default-Returns hinzu:
let mut booking_service = MockBookingService::new();
booking_service.expect_get_for_week().returning(|_, _, _, _| Ok(Arc::from([])));
let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
sales_person_unavailable_service
    .expect_get_all_for_sales_person()
    .returning(|_, _, _| Ok(Arc::from([])));
```

**Neue Tests (Q10-Liste):**

```rust
#[tokio::test]
async fn test_create_warning_for_booking_in_range() {
    let mut deps = build_dependencies();
    deps.absence_dao.expect_find_overlapping().returning(|_,_,_,_,_| Ok(Arc::from([])));
    deps.absence_dao.expect_create().returning(|_,_,_| Ok(()));
    // Setup: 1 Booking im Range
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::from([fixture_booking()])));
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(fixture_slot_monday()));
    // ... uuid + permission Mocks
    let service = deps.build_service();
    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await
        .unwrap();
    assert_eq!(result.warnings.len(), 1);
    assert!(matches!(&result.warnings[0], Warning::AbsenceOverlapsBooking { .. }));
}

#[tokio::test]
async fn test_create_warning_for_manual_unavailable_in_range() { /* analog */ }

#[tokio::test]
async fn test_update_warnings_for_full_new_range() { /* D-Phase3-04 */ }

#[tokio::test]
async fn test_find_overlapping_for_booking_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission().returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person().returning(|_, _, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();
    let result = service
        .find_overlapping_for_booking(default_sales_person_id(), fixture_range(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}
```

**Notes:** Das `AbsenceDependencies::build_service` (Z. 146-157) muss um die
zwei neuen `into()`-Felder ergänzt werden. Compile-Fehler hilft.

---

#### `service_impl/src/test/shiftplan.rs` (PATCH — per-sales-person-Tests)

**Analog:** `service_impl/src/test/shiftplan.rs:59-100`.

**Diff:** `ShiftplanViewServiceDependencies` bekommt 2 neue Mock-Felder
(`MockAbsenceService`, `MockSalesPersonUnavailableService`); neue Tests:

```rust
#[tokio::test]
async fn test_per_sales_person_marker_absence_only() {
    // absence_service.expect_find_by_sales_person → 1 active AbsencePeriod overlapping target_date
    // sales_person_unavailable_service.expect_get_by_week_for_sales_person → empty
    // → ShiftplanWeek.days[i].unavailable == Some(UnavailabilityMarker::AbsencePeriod{..})
}

#[tokio::test]
async fn test_per_sales_person_marker_manual_only() { /* analog */ }

#[tokio::test]
async fn test_per_sales_person_marker_both() { /* D-Phase3-10 — Both-Variante */ }

#[tokio::test]
async fn test_per_sales_person_marker_softdeleted_absence_none() {
    // absence_service mock returnt 1 AbsencePeriod mit deleted = Some(...)
    // build_shiftplan_day_for_sales_person filtert das raus → unavailable: None (SC4)
}

#[tokio::test]
async fn test_get_shiftplan_week_for_sales_person_forbidden() { /* HR ∨ self */ }

#[tokio::test]
async fn test_get_shiftplan_day_for_sales_person_forbidden() { /* analog */ }
```

---

#### `service_impl/src/test/mod.rs` (PATCH — Test-Wiring)

**Analog:** `service_impl/src/test/mod.rs:7-8` (`pub mod absence;` + `pub mod absence_derive_hours_range;`).

**Code-Excerpt — Diff (zwischen `pub mod sales_person_unavailable;` Z. 30 und `pub mod shiftplan;` Z. 32):**

```rust
#[cfg(test)]
pub mod sales_person_unavailable;
#[cfg(test)]
pub mod shiftplan;
// NEU für Phase 3:
#[cfg(test)]
pub mod shiftplan_edit;
```

---

#### `shifty_bin/src/integration_test/booking_absence_conflict.rs` (NEW — Cross-Source-Integration-Test)

**Analog:** `shifty_bin/src/integration_test/absence_period.rs:1-200` (TestSetup-Pattern, `create_sales_person`, `create_absence_period`-Helpers).

**Code-Excerpt — Datei-Struktur:**

```rust
//! End-to-End-Integrationstests für Phase 3 (Booking ⇄ Absence ⇄ ManualUnavailable).
//!
//! Pflicht-Coverage:
//! - test_double_source_two_warnings_one_booking (Cross-Source-Doppel-Quelle)
//! - test_softdeleted_absence_no_warning_no_marker (Pitfall 1 / SC4 — full-stack)
//! - test_copy_week_three_bookings_two_warnings (Aggregation, D-Phase3-02)

use rest::RestStateDef;
use service::{
    absence::{AbsenceCategory, AbsencePeriod, AbsenceService},
    booking::Booking,
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    shiftplan_edit::ShiftplanEditService,
    slot::{Slot, SlotService},
    warning::Warning,
};
use shifty_utils::DayOfWeek;
use sqlx::Row;
use time::macros::date;
use uuid::Uuid;

use crate::integration_test::TestSetup;

async fn create_sales_person(...) -> SalesPerson { /* analog absence_period.rs:19-38 */ }

async fn create_slot(test_setup: &TestSetup, day_of_week: DayOfWeek) -> Slot {
    // via rest_state.slot_service().create_slot(...)
}

async fn create_absence_period(...) -> AbsencePeriod { /* analog absence_period.rs:40-61 */ }

async fn create_sales_person_unavailable(...) -> SalesPersonUnavailable { ... }

#[tokio::test]
async fn test_double_source_two_warnings_one_booking() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "DoubleSource").await;
    let slot = create_slot(&test_setup, DayOfWeek::Monday).await;
    let _ap = create_absence_period_at(&test_setup, sp.id, date!(2026 - 04 - 27), date!(2026 - 04 - 30)).await;
    let _mu = create_sales_person_unavailable(&test_setup, sp.id, 2026, 18, DayOfWeek::Monday).await;

    // Booking auf Mo KW 18 2026 = 2026-04-27 → BEIDE Quellen treffen
    let booking = Booking { /* sp.id, slot.id, year=2026, calendar_week=18 */ };
    let result = test_setup
        .rest_state
        .shiftplan_edit_service()
        .book_slot_with_conflict_check(&booking, Authentication::Full, None)
        .await
        .unwrap();

    assert_eq!(result.warnings.len(), 2);
    assert!(result.warnings.iter().any(|w| matches!(w, Warning::BookingOnAbsenceDay { .. })));
    assert!(result.warnings.iter().any(|w| matches!(w, Warning::BookingOnUnavailableDay { .. })));

    // Booking wurde trotzdem persistiert
    let bookings = test_setup
        .rest_state
        .booking_service()
        .get_for_week(18, 2026, Authentication::Full, None)
        .await
        .unwrap();
    assert_eq!(bookings.len(), 1);
}

#[tokio::test]
async fn test_softdeleted_absence_no_warning_no_marker() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Pitfall1").await;
    let slot = create_slot(&test_setup, DayOfWeek::Monday).await;
    let ap = create_absence_period_at(&test_setup, sp.id, date!(2026 - 04 - 27), date!(2026 - 04 - 30)).await;
    test_setup.rest_state.absence_service().delete(ap.id, Authentication::Full, None).await.unwrap();

    let booking = Booking { /* gleicher Slot + KW 18 */ };
    let result = test_setup
        .rest_state
        .shiftplan_edit_service()
        .book_slot_with_conflict_check(&booking, Authentication::Full, None)
        .await
        .unwrap();
    assert!(result.warnings.is_empty(), "soft-deleted absence MUST NOT trigger warning");

    // Auch ShiftplanDay-Marker prüfen:
    let week = test_setup
        .rest_state
        .shiftplan_view_service()
        .get_shiftplan_week_for_sales_person(slot.shiftplan_id.unwrap(), 2026, 18, sp.id, Authentication::Full, None)
        .await
        .unwrap();
    let monday_marker = week.days.iter().find(|d| d.day_of_week == DayOfWeek::Monday).unwrap();
    assert!(monday_marker.unavailable.is_none(), "soft-deleted absence MUST NOT produce marker (SC4)");
}

#[tokio::test]
async fn test_copy_week_three_bookings_two_warnings() { /* D-Phase3-02 */ }
```

**Notes:**
- `Booking` braucht eine echte `id` (existing service generiert sie). Plan-Phase
  beachtet, dass `BookingService::create` über
  `shiftplan_edit_service.book_slot_with_conflict_check` aufgerufen wird —
  die ID wird intern gesetzt.
- Helper `create_slot` muss existieren — der Slot-Service ist über
  `rest_state.slot_service()` erreichbar (`rest/src/lib.rs:349`).

---

#### `shifty_bin/src/integration_test.rs` (PATCH — Test-Wiring)

**Analog:** `shifty_bin/src/integration_test.rs:1432-1437` (existing
`mod absence_period;`).

**Code-Excerpt — Diff (NACH Z. 1437):**

```rust
#[cfg(test)]
mod absence_period;
#[cfg(test)]
mod billing_period_custom_reports;
#[cfg(test)]
mod billing_period_snapshot_versioning;
// NEU für Phase 3:
#[cfg(test)]
mod booking_absence_conflict;
mod dev_seed;
```

---

## Shared Patterns

### Pattern S1 — `gen_service_impl!`-DI-Erweiterung im Business-Logic-Tier

**Source:** `service_impl/src/absence.rs:39-50` und `service_impl/src/shiftplan_edit.rs:22-36`.

**Apply to:** `service_impl/src/absence.rs` (+ 2 neue Felder), `service_impl/src/shiftplan_edit.rs` (+ 1 neues Feld), `service_impl/src/shiftplan.rs` (+ 2 neue Felder).

**Verboten:** Hinzufügen von Domain-Service-Deps zu `service_impl/src/booking.rs` (Basic-Tier, D-Phase3-18). Plan-Phase verifiziert mit `jj diff service_impl/src/booking.rs` (Diff bleibt leer).

```rust
// Pattern: Erweiterung der gen_service_impl!-DepStruct
gen_service_impl! {
    struct ServiceXImpl: ServiceX = ServiceXDeps {
        // ... bestehende Felder bleiben unverändert ...
        ServiceY: service::y::ServiceY<Context = Self::Context, Transaction = Self::Transaction> = service_y_field_name,
    }
}
```

### Pattern S2 — Permission HR ∨ self via tokio::join!

**Source:** `service_impl/src/absence.rs:90-99` (verbatim wiederverwendbar).

**Apply to:**
- `service_impl/src/absence.rs::AbsenceServiceImpl::find_overlapping_for_booking` (NEU)
- `service_impl/src/shiftplan_edit.rs::ShiftplanEditServiceImpl::book_slot_with_conflict_check` (NEU)
- `service_impl/src/shiftplan_edit.rs::ShiftplanEditServiceImpl::copy_week_with_conflict_check` (NEU)
- `service_impl/src/shiftplan.rs::ShiftplanViewServiceImpl::get_shiftplan_week_for_sales_person` (NEU)
- `service_impl/src/shiftplan.rs::ShiftplanViewServiceImpl::get_shiftplan_day_for_sales_person` (NEU)

```rust
let (hr, sp) = tokio::join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        sales_person_id,
        context,
        tx.clone().into()
    ),
);
hr.or(sp)?; // beide Err → ServiceError::Forbidden
```

### Pattern S3 — ISO-Week-Date-Konversion mit `?`-Operator

**Source:** `service_impl/src/shiftplan_edit.rs:69-70`, `service_impl/src/shiftplan.rs:138`.

**Apply to:**
- `service_impl/src/shiftplan_edit.rs::book_slot_with_conflict_check` (vor `BookingService::create`-Call)
- `service_impl/src/absence.rs::create`/`update` (Forward-Loop, je Booking + ManualUnavailable-Tag)
- `service_impl/src/shiftplan.rs::get_shiftplan_week_for_sales_person` (per-Tag im Loop)

```rust
let booking_date: time::Date = time::Date::from_iso_week_date(
    booking.year as i32,
    booking.calendar_week as u8,
    slot.day_of_week.into(),
)?; // ServiceError::TimeComponentRangeError via #[from] (service/src/lib.rs:108)
```

### Pattern S4 — Authentication::Full-Bypass für interne Service-Calls

**Source:** `service_impl/src/shiftplan_edit.rs:55-59` (existing `modify_slot`-Body), `service_impl/src/shiftplan.rs:142` (`get_by_week`).

**Apply to:** Alle internen `BookingService` / `AbsenceService` /
`SalesPersonUnavailableService`-Calls aus `book_slot_with_conflict_check`
und aus `AbsenceService::create`/`update`. Permission ist oben bereits
geprüft.

```rust
let absences = self
    .absence_service
    .find_overlapping_for_booking(
        sales_person_id,
        single_day_range,
        Authentication::Full, // bypass — outer permission already checked
        tx.clone().into(),
    )
    .await?;
```

### Pattern S5 — Wrapper-DTO + From-Impl für REST-Mapping

**Source:** `rest-types/src/lib.rs:1593-1606` (`AbsencePeriodTO::from(&AbsencePeriod)`-Pattern).

**Apply to:** `BookingCreateResultTO::from(&BookingCreateResult)`,
`CopyWeekResultTO::from(&CopyWeekResult)`,
`AbsencePeriodCreateResultTO::from(&AbsencePeriodCreateResult)`,
`WarningTO::from(&Warning)`, `UnavailabilityMarkerTO::from(&UnavailabilityMarker)` —
alle gated mit `#[cfg(feature = "service-impl")]`.

### Pattern S6 — `_forbidden`-Test pro public Service-Method

**Source:** `service_impl/src/test/error_test.rs:5-11` (`test_forbidden`-Helper) + `service_impl/src/test/absence.rs:200-234` (Mock-Setup mit `expect_check_permission` + `expect_verify_user_is_sales_person`).

**Apply to:** 5 neue _forbidden-Tests:
- `test_find_overlapping_for_booking_forbidden` (in `test/absence.rs`)
- `test_book_slot_with_conflict_check_forbidden` (in `test/shiftplan_edit.rs`)
- `test_copy_week_with_conflict_check_forbidden` (in `test/shiftplan_edit.rs`)
- `test_get_shiftplan_week_for_sales_person_forbidden` (in `test/shiftplan.rs`)
- `test_get_shiftplan_day_for_sales_person_forbidden` (in `test/shiftplan.rs`)

```rust
#[tokio::test]
async fn test_<method>_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission().returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person().returning(|_, _, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();
    let result = service.<method>(...).await;
    test_forbidden(&result);
}
```

### Pattern S7 — Soft-Delete-Filter im DAO

**Source:** `dao_impl_sqlite/src/absence.rs:90, 109, 128, 145, 174, 188` (alle existing Read-Queries haben `WHERE ... AND deleted IS NULL`).

**Apply to:** `dao_impl_sqlite/src/absence.rs::find_overlapping_for_booking` (Pflicht für Pitfall 1 / SC4).

```sql
WHERE sales_person_id = ?
  AND from_date <= ?
  AND to_date >= ?
  AND deleted IS NULL  -- Pflicht — sonst skip composite index + Pitfall-1-Test schlägt fehl
```

---

## No Analog Found

Alle Phase-3-Files haben einen klaren Analog im Repo. Es gibt **keine** Files
ohne Analog. Drei strukturelle Hinweise zu beachten:

| Aspekt | Begründung | Fallback |
|--------|------------|----------|
| Tag-Enum mit `#[serde(tag, content)]` und `ToSchema` | Repo hat bisher keinen Tag-Enum — `WarningTO` und `UnavailabilityMarkerTO` wären die ersten. | RESEARCH.md Pattern 4 (verifiziert via docs.rs/utoipa). Fallback: plain enum + extra `kind`-Field auf Struct-Variant. |
| `rest/src/shiftplan_edit.rs` ohne ApiDoc | Existing rest-shiftplan_edit hat keine `OpenApi`-Struct. | Phase 3 fügt `ShiftplanEditApiDoc` neu hinzu (Analog: `rest/src/shiftplan.rs:101-117`). |
| `service_impl/src/test/shiftplan_edit.rs` existiert nicht | Heute kein Test-Modul für ShiftplanEditService. | Analog `service_impl/src/test/shiftplan.rs` (Mock-DI-Pattern) + `test/booking.rs` (Mock-Setup-Idiom). Plan-Phase erstellt das Modul neu. |

---

## Metadata

**Analog search scope:**
- `service/src/`, `service_impl/src/`, `dao/src/`, `dao_impl_sqlite/src/`,
  `rest/src/`, `rest-types/src/`, `shifty_bin/src/`, `shifty-utils/src/`,
  `migrations/sqlite/`
- Phase-1 PATTERNS.md, Phase-1 CONTEXT.md (D-08, D-09, D-12, D-15, D-16/17),
  Phase-3 RESEARCH.md (Patterns 1-5, Operations 1-5, Pitfalls 1-7)

**Files scanned (verifiziert via direct read):**
- `service/src/{booking, shiftplan_edit, absence, shiftplan, sales_person_unavailable, lib}.rs`
- `service_impl/src/{absence, shiftplan_edit, shiftplan}.rs`
- `dao/src/absence.rs`
- `dao_impl_sqlite/src/absence.rs`
- `rest/src/{absence, booking, shiftplan_edit, shiftplan, lib}.rs`
- `rest-types/src/lib.rs` (1623 Zeilen — gezielte Sektionen)
- `service_impl/src/test/{absence, booking, shiftplan, mod}.rs`
- `shifty_bin/src/{main, integration_test}.rs`,
  `shifty_bin/src/integration_test/absence_period.rs`
- `service/src/lib.rs:90-122` (ServiceError-Surface)

**Pattern extraction date:** 2026-05-02

**Tier-Konsistenz-Verifikation (D-Phase3-18 + CLAUDE.md):**
- ✓ `BookingService` bleibt strikt Basic-Tier — nur `BookingDao`,
  `PermissionService`, `TransactionDao`, plus existing `SlotService`,
  `SalesPersonService`, `SalesPersonShiftplanService`, `ClockService`,
  `UuidService` (alles Tier-konform für CRUD-Validation).
- ✓ `AbsenceService`, `ShiftplanEditService`, `ShiftplanViewService` sind
  Business-Logic-Tier — dürfen Basic-Services konsumieren.
- ✓ Konstruktionsreihenfolge in `shifty_bin/src/main.rs::RestStateImpl::new`
  ist heute schon Tier-konform: `booking_service` Z. 699 vor `absence_service`
  Z. 737 vor `shiftplan_edit_service` Z. 808 / `shiftplan_view_service` Z. 831.
  Phase 3 ergänzt nur `clone()`-Pässe; kein Cycle, kein OnceLock.

---

*Phase: 3-Booking-Shift-Plan-Konflikt-Integration*
*Pattern-Mapping: 2026-05-02 — Service-Tier-konform (CLAUDE.md § "Service-Tier-Konventionen")*
