# Phase 1: Absence Domain Foundation – Pattern Map

**Mapped:** 2026-05-01
**Files analyzed:** 14 neue Dateien + 9 Patches (in-place Module-Re-Exports / DI-Verdrahtung)
**Analogs found:** 14 / 14 (alle direkten Vorlagen im Repo lokalisiert)

> **Sprache:** Deutsche Prosa. Code, SQL, Identifier, Trait- und Modulnamen bleiben in Originalform.
>
> **Lese-Reihenfolge für den Planner:** Wave A (Migration → DateRange) → Wave B (DAO → Service) → Wave C (REST + DI) → Wave D (Tests). Diese Map folgt derselben Reihenfolge.

---

## File Classification

| Neue / geänderte Datei | Layer | Role | Data Flow | Closest Analog | Match Quality |
|------------------------|-------|------|-----------|----------------|---------------|
| `migrations/sqlite/<ts>_create-absence-period.sql` | Migration | DDL: CREATE TABLE + Indexes + CHECK | DDL / one-shot | `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql` (+ `…_add-table-billing-period.sql`-Schema-Style) | **exact** (Schema-Style 1:1 + Partial-Unique-Idiom) |
| `shifty-utils/src/date_range.rs` | Utility | Domain-Type / Pure (no I/O) | transform | `shifty-utils/src/date_utils.rs` (Module-Layout) + Research-Skizze §4 | **role-match** (kein DateRange-Bestand → Style folgt date_utils) |
| `dao/src/absence.rs` | DAO Trait | Trait + Entity + Enum-Spiegel | CRUD | `dao/src/extra_hours.rs` | **exact** |
| `dao_impl_sqlite/src/absence.rs` | DAO Impl | SQLx-Queries / Soft-Delete | CRUD + range query | `dao_impl_sqlite/src/extra_hours.rs` | **exact** |
| `service/src/absence.rs` | Service Trait | Trait + Domain-Modell + Enum + automock | request-response | `service/src/extra_hours.rs` | **exact** |
| `service_impl/src/absence.rs` | Service Impl | gen_service_impl! + logical_id-Update + Permission | CRUD + transaction | `service_impl/src/extra_hours.rs` | **exact** (Update-Flow ZZ. 220-301; Permission ZZ. 236-245) |
| `service_impl/src/test/absence.rs` | Test Unit | Mock-DI + `_success`/`_forbidden`/`_not_found` | request-response (mocked) | `service_impl/src/test/extra_hours.rs` | **exact** |
| `rest/src/absence.rs` | REST Handler | Router + `#[utoipa::path]` + `error_handler` | request-response | `rest/src/extra_hours.rs` (+ `rest/src/booking.rs` für GET-by-ID) | **exact** |
| `rest-types/src/absence_period_to.rs` (oder inline in `lib.rs`) | DTO | Transport-Object + ToSchema + bidir. From | transform | `rest-types/src/lib.rs` ZZ. 102-155 (`BookingTO`), ZZ. 741-789 (`ExtraHoursTO`) | **exact** (Repo-Konvention: alle DTOs inline in `lib.rs`) |
| `shifty_bin/src/integration_test/absence_period.rs` | Integration Test | echte In-Memory-SQLite + Migration-Run | request-response (full-stack) | `shifty_bin/src/integration_test/extra_hours_update.rs` | **exact** |
| `shifty_bin/src/main.rs` (PATCH) | DI | `*ServiceDependencies`-Block + `Arc::new(*ServiceImpl{…})` | wiring | ZZ. 223-236 (`ExtraHoursServiceDependencies`) + ZZ. 680-688 (Konstruktion) | **exact** |
| `rest/src/lib.rs` (PATCH) | REST Wiring | Module-Decl + `.nest("/absence-period",…)` + `ApiDoc`-nest + `RestStateDef`-Method | wiring | ZZ. 10 (mod), ZZ. 465 (ApiDoc-nest), ZZ. 539 (Router-nest), ZZ. 357 (RestStateDef-Method) | **exact** |
| `service/src/lib.rs` (PATCH) | Module Re-Export | `pub mod absence;` + ggf. neue `ValidationFailureItem`-Variante | wiring | ZZ. 20 (mod) + ZZ. 49-54 (`ValidationFailureItem` enum) | **exact** |
| `service_impl/src/lib.rs` (PATCH) | Module Re-Export | `pub mod absence;` | wiring | ZZ. 17 (`pub mod extra_hours;`) | **exact** |
| `dao/src/lib.rs` (PATCH) | Module Re-Export | `pub mod absence;` | wiring | ZZ. 14 (`pub mod extra_hours;`) | **exact** |
| `dao_impl_sqlite/src/lib.rs` (PATCH) | Module Re-Export | `pub mod absence;` | wiring | ZZ. 15 (`pub mod extra_hours;`) | **exact** |
| `rest-types/src/lib.rs` (PATCH oder neue Datei) | DTO Wiring | DTO-Definitionen ergänzen | wiring | bestehender DTO-Block ZZ. 741-789 | **exact** |
| `shifty-utils/src/lib.rs` (PATCH) | Module Re-Export | `pub mod date_range; pub use date_range::*;` | wiring | ZZ. 1-3 (`mod date_utils; pub use date_utils::*;`) | **exact** |
| `service_impl/src/test/mod.rs` (PATCH) | Test Wiring | `#[cfg(test)] pub mod absence;` | wiring | ZZ. 37-38 (`#[cfg(test)] pub mod extra_hours;`) | **exact** |

> **Notiz zur DTO-Datei:** RESEARCH.md erwähnt `rest-types/src/absence_period_to.rs` als möglichen Pfad, aber der Repo-Bestand zeigt **kein** einziges per-domain TO-File: alle Transport-Objects leben inline in `rest-types/src/lib.rs`. Plan-Phase sollte konservativ bleiben und ebenfalls inline anhängen — ein neues TO-File würde die Konvention brechen, ohne Vorteil. Der CONTEXT.md-Text (D-01) nennt zwar den Pfad, aber das ist ein Vorschlag, kein Muss.

---

## Pattern Assignments

### Layer A — Migration

#### `migrations/sqlite/<ts>_create-absence-period.sql` (Migration, DDL)

**Analog 1:** `migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql`

**Warum dieser Analog:** Liefert das **Partial-Unique-Index-Idiom auf `logical_id` mit Tombstone-Filter** — exakt das, was `absence_period` braucht (D-04, Pitfall-6).

**Code-Excerpt — Partial-Unique-Index** (`migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql:41-44`):

```sql
-- 4. Partial unique index: at most one active row per logical_id.
CREATE UNIQUE INDEX idx_extra_hours_logical_id_active
    ON extra_hours(logical_id)
    WHERE deleted IS NULL;
```

**Code-Excerpt — Audit-Spalten-Konvention** (`migrations/sqlite/20260428101456_add-logical-id-to-extra-hours.sql:8-26`):

```sql
CREATE TABLE extra_hours_new (
    id BLOB(16) NOT NULL PRIMARY KEY,
    sales_person_id BLOB(16) NOT NULL,
    amount FLOAT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    date_time TEXT NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB(16) NOT NULL,

    custom_extra_hours_id BLOB NOT NULL DEFAULT X'00000000000000000000000000000000',

    logical_id BLOB(16) NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);
```

**Adaptation Notes für den Executor:**

1. **Drei NEW-Tabellen-Spalten** statt `amount`/`date_time`: `from_date TEXT NOT NULL`, `to_date TEXT NOT NULL` (beide ISO-8601 `YYYY-MM-DD`).
2. **Inline `CHECK (to_date >= from_date)`** im `CREATE TABLE` — SQLite kann ADD CONSTRAINT nicht nachträglich (Pitfall 8 aus RESEARCH.md).
3. **Drei Indexes** (D-04 + RESEARCH.md §3.1): `idx_absence_period_logical_id_active` (partial unique auf `logical_id`), `idx_absence_period_sales_person_from` (composite auf `(sales_person_id, from_date)`), `idx_absence_period_self_overlap` (composite auf `(sales_person_id, category, from_date)`) — alle drei mit `WHERE deleted IS NULL`.
4. **Keine ALTER TABLE-Schritte** wie im Analog — `absence_period` ist eine Greenfield-Tabelle, kein Backfill nötig. Der Analog macht ALTER + Backfill, weil er `logical_id` *zu* einer bestehenden Tabelle hinzufügt; hier ist Migration ein einfaches `CREATE TABLE`.
5. **Kein `custom_extra_hours_id`-Feld** und kein `Custom`-Variant — `AbsenceCategory` hat 3 Werte (Vacation, SickLeave, UnpaidLeave) per D-02/D-03.
6. **`update_timestamp TEXT` beibehalten** (Konsistenz mit ExtraHours-Schema), auch wenn der DAO-Code es nicht aktiv beschreibt — siehe RESEARCH.md §3.1 Notiz.
7. **Generierung via `nix-shell -p sqlx-cli --run "sqlx migrate add create-absence-period --source migrations/sqlite"`** (CC-10).
8. **Migration MUSS vor `cargo build` laufen** — sonst scheitert SQLx-`query_as!` mit "no such table: absence_period" (Pitfall 5 RESEARCH.md).

---

### Layer B — Utility (shifty-utils)

#### `shifty-utils/src/date_range.rs` (Utility, Pure-Type)

**Analog:** `shifty-utils/src/date_utils.rs` (für Module-Style); für Implementation siehe RESEARCH.md §4 und Code-Skizze ZZ. 588-668.

**Warum dieser Analog:** Gibt die Konvention (kein I/O, `time`-Crate-Dependency, `thiserror`-Error-Type-Pattern, `pub use`-Re-Export aus `shifty-utils/src/lib.rs`). Es gibt **keinen DateRange-Bestand**; der nächste verwandte Type ist `ShiftyDate`/`ShiftyWeek`.

**Code-Excerpt — Error-Type-Konvention** (`shifty-utils/src/date_utils.rs:6-10`):

```rust
#[derive(Debug, Error)]
pub enum ShiftyDateUtilsError {
    #[error("Invalid date: {0}")]
    DateError(#[from] time::error::ComponentRange),
}
```

**Code-Excerpt — Re-Export-Pattern** (`shifty-utils/src/lib.rs:1-3`):

```rust
mod date_utils;

pub use date_utils::*;
```

**Adaptation Notes:**

1. **API-Surface 1:1 aus RESEARCH.md §4 Tabelle** übernehmen: `DateRange { from, to }`, `new(from, to) -> Result<Self, RangeError>`, `from()`, `to()`, `overlaps(&Self) -> bool`, `contains(Date) -> bool`, `iter_days() -> impl Iterator<Item = Date>`, `day_count() -> u32`.
2. **`RangeError::FromAfterTo { from: Date, to: Date }`** als einzige Variante in Phase 1 (genau so wie `ShiftyDateUtilsError` nur `DateError` hat).
3. **Inclusive Allen-Idiom** in `overlaps`: `self.from <= other.to && other.from <= self.to`. Pitfall 1 (RESEARCH.md): half-open Bounds bricht Single-Day-Range.
4. **`#[cfg(test)] mod tests`** mit den Mindest-Tests aus RESEARCH.md §4 (Touching-Boundary, Single-Day-Self-Overlap, Year-Boundary-Iter).
5. **`shifty-utils/src/lib.rs`-Patch:** `pub mod date_range; pub use date_range::*;` direkt nach den bestehenden Zeilen 1-3.
6. **Keine neuen Dependencies** — `time` und `thiserror` sind bereits in `shifty-utils/Cargo.toml`.

---

### Layer C — DAO (dao + dao_impl_sqlite)

#### `dao/src/absence.rs` (DAO Trait, CRUD)

**Analog:** `dao/src/extra_hours.rs`

**Warum dieser Analog:** Liefert das **komplette `[automock]`-Trait-Skelett mit `find_by_id`, `find_by_logical_id`, `create`, `update`** plus Entity-Struct + Enum-Spiegel — alles, was der `AbsenceDao`-Trait braucht.

**Code-Excerpt — Imports + Enum-Spiegel** (`dao/src/extra_hours.rs:1-18`):

```rust
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::ShiftyDate;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtraHoursCategoryEntity {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
    UnpaidLeave,
    VolunteerWork,
    Custom(Uuid),
}
```

**Code-Excerpt — Entity-Struct** (`dao/src/extra_hours.rs:20-32`):

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHoursEntity {
    pub id: Uuid,
    pub logical_id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategoryEntity,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
```

**Code-Excerpt — Trait mit `[automock]`** (`dao/src/extra_hours.rs:40-86`):

```rust
#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait ExtraHoursDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<ExtraHoursEntity>, crate::DaoError>;
    async fn find_by_logical_id(
        &self,
        logical_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<ExtraHoursEntity>, crate::DaoError>;
    // … find_by_sales_person_id_and_years, find_by_week, create, update …
}
```

**Adaptation Notes:**

1. **Enum-Spiegel `AbsenceCategoryEntity`** mit nur 3 Varianten (`Vacation`, `SickLeave`, `UnpaidLeave`) — D-02/D-03. Kein `Custom(Uuid)`.
2. **Felder von `AbsencePeriodEntity`** statt `amount`/`date_time`: `category: AbsenceCategoryEntity`, `from_date: time::Date`, `to_date: time::Date`, `description: Arc<str>`. Audit-Felder (`id`, `logical_id`, `sales_person_id`, `created`, `deleted`, `version`) bleiben identisch.
3. **`time::Date` statt `time::PrimitiveDateTime`** für `from_date`/`to_date` (D-05, STACK.md "whole-day, kein Timezone-Bezug").
4. **Trait-Methoden nach RESEARCH.md §6** — `find_by_id`, `find_by_logical_id`, `find_by_sales_person`, `find_all`, `find_overlapping(sales_person_id, category, range, exclude_logical_id, tx)`, `create`, `update`. **Kein `delete` im Trait** (RESEARCH.md §6 Notiz: ExtraHours hat `delete` als `unimplemented!()`; sauberer ist Weglassen).
5. **`#[automock(type Transaction = crate::MockTransaction;)]`** auf den Trait — generiert `MockAbsenceDao` für die Service-Unit-Tests.
6. **Imports:** `shifty_utils::DateRange` (für `find_overlapping`), `time::Date`, ansonsten identisch.
7. **`dao/src/lib.rs`-Patch:** `pub mod absence;` neben `pub mod extra_hours;` (ZZ. 14).

---

#### `dao_impl_sqlite/src/absence.rs` (DAO Impl, SQLx)

**Analog:** `dao_impl_sqlite/src/extra_hours.rs`

**Warum dieser Analog:** Liefert das **komplette SQLx-Compile-Time-Pattern mit `query_as!`/`query!`, Soft-Delete-`WHERE deleted IS NULL`, `TryFrom<&XxxDb> for XxxEntity` mit Enum-Mapping**, BLOB-UUID-Bind-Pfad und das `update`-Idiom (Soft-Delete only).

**Code-Excerpt — DB-Row-Struct + Imports** (`dao_impl_sqlite/src/extra_hours.rs:1-26`):

```rust
use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    extra_hours::{ExtraHoursCategoryEntity, ExtraHoursDao, ExtraHoursEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

struct ExtraHoursDb {
    id: Vec<u8>,
    logical_id: Vec<u8>,
    sales_person_id: Vec<u8>,
    amount: f64,

    category: String,
    description: Option<String>,
    date_time: String,
    custom_extra_hours_id: Vec<u8>,
    created: String,
    deleted: Option<String>,
    update_version: Vec<u8>,
}
```

**Code-Excerpt — TryFrom mit Enum-Mapping** (`dao_impl_sqlite/src/extra_hours.rs:27-68`):

```rust
impl TryFrom<&ExtraHoursDb> for ExtraHoursEntity {
    type Error = DaoError;

    fn try_from(extra_hours: &ExtraHoursDb) -> Result<Self, DaoError> {
        Ok(Self {
            id: Uuid::from_slice(extra_hours.id.as_ref())?,
            logical_id: Uuid::from_slice(extra_hours.logical_id.as_ref())?,
            sales_person_id: Uuid::from_slice(extra_hours.sales_person_id.as_ref())?,
            amount: extra_hours.amount as f32,
            category: match extra_hours.category.as_str() {
                "ExtraWork" => ExtraHoursCategoryEntity::ExtraWork,
                "Vacation" => ExtraHoursCategoryEntity::Vacation,
                // …
                value => return Err(DaoError::EnumValueNotFound(value.into())),
            },
            description: extra_hours
                .description
                .clone()
                .unwrap_or_default()
                .as_str()
                .into(),
            // … date_time, created, deleted parse …
            version: Uuid::from_slice(&extra_hours.update_version)?,
        })
    }
}
```

**Code-Excerpt — `find_by_logical_id` mit Soft-Delete-Filter** (`dao_impl_sqlite/src/extra_hours.rs:102-118`):

```rust
async fn find_by_logical_id(
    &self,
    logical_id: Uuid,
    tx: Self::Transaction,
) -> Result<Option<ExtraHoursEntity>, crate::DaoError> {
    let logical_id_vec = logical_id.as_bytes().to_vec();
    Ok(query_as!(
        ExtraHoursDb,
        "SELECT id, logical_id, sales_person_id, amount, category, description, custom_extra_hours_id, date_time, created, deleted, update_version FROM extra_hours WHERE logical_id = ? AND deleted IS NULL",
        logical_id_vec,
    ).fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(ExtraHoursEntity::try_from)
        .transpose()?)
}
```

**Code-Excerpt — `update` als Soft-Delete-only** (`dao_impl_sqlite/src/extra_hours.rs:218-241`):

```rust
async fn update(
    &self,
    entity: &ExtraHoursEntity,
    process: &str,
    tx: Self::Transaction,
) -> Result<(), crate::DaoError> {
    let id_vec = entity.id.as_bytes().to_vec();
    let version_vec = entity.version.as_bytes().to_vec();
    let delete = entity
        .deleted
        .map(|date_time| date_time.format(&Iso8601::DATE_TIME))
        .transpose()?;
    query!(
        "UPDATE extra_hours SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?",
        delete,
        version_vec,
        process,
        id_vec,
    )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
    Ok(())
}
```

**Adaptation Notes:**

1. **`AbsencePeriodDb`-Row-Struct** mit `from_date: String`, `to_date: String` (TEXT) und keine `amount`/`date_time`/`custom_extra_hours_id`-Spalten.
2. **`TryFrom<&AbsencePeriodDb> for AbsencePeriodEntity`** mit:
   - `from_date`/`to_date` parsen via `time::Date::parse(s, &Iso8601::DATE)?`
   - `category`-Match auf 3 Varianten + `value => return Err(DaoError::EnumValueNotFound(value.into()))` als Default
3. **`find_overlapping`-SQL** nach RESEARCH.md §7.1 mit **zwei Branches** (`Some(exclude)`/`None(exclude)`) — Pitfall 9 verbietet single-Query mit Sentinel-UUID. SQL-Form:
   ```sql
   SELECT … FROM absence_period
    WHERE sales_person_id = ?
      AND category = ?
      AND from_date <= ?      -- existing.from <= probe.to
      AND to_date   >= ?      -- existing.to   >= probe.from
      AND logical_id != ?     -- nur in Some(exclude)-Branch
      AND deleted IS NULL
   ```
4. **Inclusive Allen** (D-05 + Pitfall 1): `<=` und `>=`, niemals `<` oder `>`. Single-Day-Range muss gegen sich selbst überlappen.
5. **`update` als Soft-Delete-only** kopieren; kein DAO-`delete` (Service ruft `update(entity { deleted: Some(now) })` auf).
6. **Keine `find_by_week`-Methode** — Range-Domain hat keinen Wochen-Lookup. Stattdessen `find_by_sales_person` und `find_overlapping`.
7. **`dao_impl_sqlite/src/lib.rs`-Patch:** `pub mod absence;` neben `pub mod extra_hours;` (ZZ. 15).
8. **`category`-zu-String-Helper** kann lokal sein:
   ```rust
   fn category_to_str(c: &AbsenceCategoryEntity) -> &'static str {
       match c {
           AbsenceCategoryEntity::Vacation => "Vacation",
           AbsenceCategoryEntity::SickLeave => "SickLeave",
           AbsenceCategoryEntity::UnpaidLeave => "UnpaidLeave",
       }
   }
   ```

---

### Layer D — Service (service + service_impl)

#### `service/src/absence.rs` (Service Trait, request-response)

**Analog:** `service/src/extra_hours.rs`

**Warum dieser Analog:** Liefert das **komplette Service-Trait-Pattern mit `[automock]`, `Authentication<Self::Context>`, `Option<Self::Transaction>`, `From<&Entity>`/`TryFrom<&Domain>`-Conversions** — exakt was `AbsenceService` braucht.

**Code-Excerpt — Imports + Enum + From-Conversions** (`service/src/extra_hours.rs:14-128`, gekürzt):

```rust
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
// …
use uuid::Uuid;

use crate::{custom_extra_hours::CustomExtraHours, permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraHoursCategory {
    ExtraWork,
    Vacation,
    SickLeave,
    // …
}

impl From<&dao::extra_hours::ExtraHoursCategoryEntity> for ExtraHoursCategory {
    fn from(category: &dao::extra_hours::ExtraHoursCategoryEntity) -> Self {
        match category {
            dao::extra_hours::ExtraHoursCategoryEntity::Vacation => Self::Vacation,
            // …
        }
    }
}
```

**Code-Excerpt — Domain-Struct mit `id == logical_id`-Annotation** (`service/src/extra_hours.rs:130-158`):

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHours {
    /// Externally-stable id. Maps to the `logical_id` column on the persistence layer
    /// (which equals the physical row id for the first version of the entry).
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategory,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::extra_hours::ExtraHoursEntity> for ExtraHours {
    fn from(extra_hours: &dao::extra_hours::ExtraHoursEntity) -> Self {
        Self {
            id: extra_hours.logical_id,            // <-- KEY: domain id == logical_id
            // …
        }
    }
}
```

**Code-Excerpt — Trait-Surface** (`service/src/extra_hours.rs:185-235`):

```rust
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait ExtraHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;
    // …
    async fn create(
        &self,
        entity: &ExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ExtraHours, ServiceError>;
    async fn update(/* … */) -> Result<ExtraHours, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
```

**Adaptation Notes:**

1. **Enum `AbsenceCategory`** mit nur 3 Varianten und **kein `LazyLoad`** (Phase-1-Domain-Trennung, D-03).
2. **Domain-Struct `AbsencePeriod`** mit `from_date: time::Date`, `to_date: time::Date`, `category: AbsenceCategory`, `description: Arc<str>` (C-03), Audit-Felder identisch.
3. **`id == logical_id`-Doku** als `///`-Kommentar wörtlich übernehmen (RESEARCH.md §5 zitiert das).
4. **Service-Trait-Methoden nach RESEARCH.md §5**: `find_all`, `find_by_sales_person`, `find_by_id`, `create`, `update`, `delete` — 6 Methoden.
5. **Helper `AbsencePeriod::date_range(&self) -> Result<DateRange, ServiceError>`** für die Service-Validierung — siehe RESEARCH.md ZZ. 757-762.
6. **`#[automock(type Context=(); type Transaction=dao::MockTransaction;)]`** **wörtlich** übernehmen — bestimmt die Mock-Generation.
7. **`From<&dao::absence::AbsencePeriodEntity> for AbsencePeriod`** UND **`TryFrom<&AbsencePeriod> for dao::absence::AbsencePeriodEntity`** (mit `ServiceError::InternalError` falls `created.is_none()`).
8. **`service/src/lib.rs`-Patch:** `pub mod absence;` neben anderen Modulen (analog ZZ. 20). **Optional erweitere `ValidationFailureItem` um `OverlappingPeriod(Uuid)`** (D-13/A1, RESEARCH.md §8 Recommendation) — Plan-Phase entscheidet.

---

#### `service_impl/src/absence.rs` (Service Impl, CRUD + Transaction)

**Analog:** `service_impl/src/extra_hours.rs`

**Warum dieser Analog:** Liefert das **gen_service_impl!-DI-Block, das logical_id-Update-Idiom (Tombstone+Create) ZZ. 220-301, das Permission-Pattern (HR ∨ self via tokio::join!) ZZ. 236-245, das Transaction-Pattern (use_transaction → … → commit), und die Read-with-self-or-hr-Variante ZZ. 113-148** — alle 4 Patterns, die `AbsenceServiceImpl` 1:1 reproduzieren muss.

**Code-Excerpt — `gen_service_impl!`-Block** (`service_impl/src/extra_hours.rs:1-32`):

```rust
use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    extra_hours::{self, ExtraHoursDao},
    TransactionDao,
};
use service::{
    clock::ClockService,
    custom_extra_hours::CustomExtraHoursService,
    extra_hours::{ExtraHours, ExtraHoursService},
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct ExtraHoursServiceImpl: ExtraHoursService = ExtraHoursServiceDeps {
        ExtraHoursDao: ExtraHoursDao<Transaction = Self::Transaction> = extra_hours_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        CustomExtraHoursService: CustomExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = custom_extra_hours_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Code-Excerpt — `create` mit Permission + IdSetOnCreate-Validation** (`service_impl/src/extra_hours.rs:176-219`):

```rust
async fn create(
    &self,
    extra_hours: &ExtraHours,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<ExtraHours, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    let (hr_permission, sales_person_permission) = join!(
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            extra_hours.sales_person_id,
            context,
            tx.clone().into()
        ),
    );
    hr_permission.or(sales_person_permission)?;

    let mut extra_hours = extra_hours.to_owned();
    if !extra_hours.id.is_nil() {
        return Err(ServiceError::IdSetOnCreate);
    }
    if !extra_hours.version.is_nil() {
        return Err(ServiceError::VersionSetOnCreate);
    }

    extra_hours.id = self.uuid_service.new_uuid("extra_hours_service::create id");
    extra_hours.version = self
        .uuid_service
        .new_uuid("extra_hours_service::create version");
    extra_hours.created = Some(self.clock_service.date_time_now());

    let extra_hours_entity = extra_hours::ExtraHoursEntity::try_from(&extra_hours)?;
    self.extra_hours_dao
        .create(&extra_hours_entity, "extra_hours_service::create", tx.clone())
        .await?;

    self.transaction_dao.commit(tx).await?;
    Ok(extra_hours)
}
```

**Code-Excerpt — Update-Flow mit logical_id (kompletter Blueprint)** (`service_impl/src/extra_hours.rs:220-301`):

```rust
async fn update(
    &self,
    request: &ExtraHours,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<ExtraHours, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    let logical_id = request.id;

    let active = self
        .extra_hours_dao
        .find_by_logical_id(logical_id, tx.clone())
        .await?
        .ok_or(ServiceError::EntityNotFound(logical_id))?;

    let (hr_permission, sales_person_permission) = join!(
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            active.sales_person_id,
            context,
            tx.clone().into()
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
            logical_id,
            request.version,
            active.version,
        ));
    }

    let mut tombstone = active.clone();
    tombstone.deleted = Some(self.clock_service.date_time_now());
    self.extra_hours_dao
        .update(&tombstone, "extra_hours_service::update::soft_delete", tx.clone())
        .await?;

    let new_id = self.uuid_service.new_uuid("extra_hours_service::update::id");
    let new_version = self.uuid_service.new_uuid("extra_hours_service::update::version");
    let now = self.clock_service.date_time_now();

    let new_entity = extra_hours::ExtraHoursEntity {
        id: new_id,
        logical_id: active.logical_id,
        sales_person_id: active.sales_person_id,
        // … alle mutierbaren Felder aus `request` …
        category: (&request.category).into(),
        description: request.description.clone(),
        created: now,
        deleted: None,
        version: new_version,
    };
    self.extra_hours_dao
        .create(&new_entity, "extra_hours_service::update::insert", tx.clone())
        .await?;

    self.transaction_dao.commit(tx).await?;
    Ok(ExtraHours::from(&new_entity))
}
```

**Code-Excerpt — Read-with-HR-or-Self** (`service_impl/src/extra_hours.rs:113-148`):

```rust
async fn find_by_sales_person_id_and_year_range(
    &self,
    sales_person_id: Uuid,
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[ExtraHours]>, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    let (hr_permission, sales_person_permission) = join!(
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone()),
        self.sales_person_service.verify_user_is_sales_person(
            sales_person_id,
            context,
            tx.clone().into()
        ),
    );
    hr_permission.or(sales_person_permission)?;
    // … DAO-Lookup, transformation, commit …
}
```

**Adaptation Notes:**

1. **gen_service_impl!-Block für AbsenceServiceImpl** mit Deps: `AbsenceDao`, `PermissionService`, `SalesPersonService`, `ClockService`, `UuidService`, `TransactionDao`. **Kein `CustomExtraHoursService`** und (Phase-1-Empfehlung Option A, RESEARCH.md §9) auch **kein `SalesPersonShiftplanService`** — die D-10-Schichtplan-Kollege-Erweiterung wird auf Phase 3 verschoben.
2. **`create` exakt wie ExtraHours-Vorlage** kopieren, nur:
   - **Range-Validierung VOR `IdSetOnCreate`**: `let _ = DateRange::new(request.from_date, request.to_date).map_err(|_| ServiceError::DateOrderWrong(request.from_date, request.to_date))?;`
   - **Self-Overlap-Check NACH ID-Generation**: `find_overlapping(sales_person_id, category, range, None, tx.clone())` und bei nicht-leerem Ergebnis `ValidationError([OverlappingPeriod(conflict.logical_id)])` (D-12/D-13).
3. **`update`-Flow 1:1 aus ZZ. 220-301** kopieren, mit ZWEI Erweiterungen:
   - **Range-Validierung** auf `request.from_date`/`request.to_date` direkt nach Permission-Check.
   - **Self-Overlap-Check mit `exclude_logical_id: Some(logical_id)`** (D-15, RESEARCH.md Pitfall 2) — die alte Row muss exkludiert werden, sonst kollidiert das Update mit sich selbst.
4. **`delete` analog ExtraHours ZZ. 303-347** — `find_by_logical_id` → Permission → `update(entity { deleted: Some(now) })`. **Kein DAO-`delete`-Aufruf**.
5. **`find_all`** als HR-only-Pattern aus `service_impl/src/booking.rs:76-98` (`get_all`) — RESEARCH.md §9 Code-Skizze. `check_permission(HR_PRIVILEGE).await?` + `dao.find_all(tx)`.
6. **`find_by_sales_person` und `find_by_id`** als HR-or-Self-Pattern aus ZZ. 113-148. Bei `find_by_id` zuerst `find_by_logical_id` aufrufen, **dann** Permission auf `entity.sales_person_id` prüfen — Pattern aus `delete` ZZ. 318-333.
7. **Transaction-Pattern** in jeder Methode: `let tx = self.transaction_dao.use_transaction(tx).await?; … self.transaction_dao.commit(tx).await?;` (CC-03).
8. **Process-String-Konvention:** `"absence_service::create"`, `"absence_service::update::soft_delete"`, `"absence_service::update::insert"`, `"absence_service::delete"` — analog ExtraHours.
9. **`service_impl/src/lib.rs`-Patch:** `pub mod absence;` neben `pub mod extra_hours;` (ZZ. 17).

---

### Layer E — REST (rest + rest-types)

#### `rest-types/src/lib.rs` PATCH (oder neue Datei `absence_period_to.rs`) — DTO

**Analog:** `rest-types/src/lib.rs` ZZ. 102-155 (`BookingTO`) und ZZ. 741-789 (`ExtraHoursTO`).

**Warum diese beiden Analoga:** `BookingTO` zeigt das **kompakte DTO-Pattern mit `#[serde(default)]`-Audit-Feldern**, `ExtraHoursTO` zeigt das **DTO mit Domain-Enum (`ExtraHoursCategoryTO`) und bidirektionaler `From`-Conversion via `#[cfg(feature = "service-impl")]`** — exakt das, was `AbsencePeriodTO` braucht.

**Code-Excerpt — BookingTO mit Audit-Feldern** (`rest-types/src/lib.rs:102-155`):

```rust
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct BookingTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub created_by: Option<Arc<str>>,
    #[serde(default)]
    pub deleted_by: Option<Arc<str>>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&Booking> for BookingTO {
    fn from(booking: &Booking) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            // … alle Felder mappen …
            version: booking.version,
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self { /* gleiche Felder rückwärts */ }
    }
}
```

**Code-Excerpt — ExtraHoursTO mit eigenem DTO-Enum** (`rest-types/src/lib.rs:741-789`):

```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ExtraHoursTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategoryTO,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
#[cfg(feature = "service-impl")]
impl From<&service::extra_hours::ExtraHours> for ExtraHoursTO {
    fn from(extra_hours: &service::extra_hours::ExtraHours) -> Self {
        Self {
            id: extra_hours.id,
            // …
            description: extra_hours.description.clone(),
            // …
        }
    }
}
#[cfg(feature = "service-impl")]
impl From<&ExtraHoursTO> for service::extra_hours::ExtraHours {
    fn from(extra_hours: &ExtraHoursTO) -> Self { /* … */ }
}
```

**Adaptation Notes:**

1. **`AbsenceCategoryTO`-Enum** mit nur `Vacation`, `SickLeave`, `UnpaidLeave` und `#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]`.
2. **`AbsencePeriodTO`-Struct** mit `id` (`#[serde(default)]`), `sales_person_id`, `category: AbsenceCategoryTO`, `from_date: time::Date`, `to_date: time::Date`, `description: Arc<str>` (`#[serde(default)]` C-03), `created/deleted: Option<PrimitiveDateTime>` (`#[serde(default)]`), `version: Uuid` (`#[serde(rename = "$version", default)]`).
3. **`#[schema(value_type = String, format = "date")]`** auf `from_date` und `to_date` — RESEARCH.md ZZ. 1391-1394 zeigt das (utoipa braucht den Hint, weil `time::Date` keine direkte ToSchema-Impl hat).
4. **Bidirektionale `From`-Impls** `#[cfg(feature = "service-impl")]`-gated, exakt wie ExtraHoursTO ZZ. 758-789.
5. **Inline in `rest-types/src/lib.rs`** anhängen (Repo-Konvention: alle DTOs sind inline, kein per-domain-File). Plan-Phase darf alternativ ein neues `rest-types/src/absence_period_to.rs`-File anlegen, aber das bricht die Konvention.
6. **Keine `created_by`/`deleted_by`-Felder** — ExtraHours hat die auch nicht; Booking hat sie speziell, aber RESEARCH.md §10 Code-Skizze nimmt sie ebenfalls **nicht** auf.

---

#### `rest/src/absence.rs` (REST Handler, request-response)

**Analog:** `rest/src/extra_hours.rs` (Handler-Pattern + ApiDoc) und `rest/src/booking.rs` (GET-by-ID-Variante).

**Warum diese beiden Analoga:** ExtraHours liefert das **vollständige `[utoipa::path]`-+-`error_handler`-Pattern mit `update`-Handler ZZ. 121-162**, Booking liefert die **kompaktere GET-by-ID-Variante ZZ. 76-97 ohne utoipa-Annotations** — beide Pattern werden gemischt: Phase-1-Endpoints brauchen utoipa (CC-06).

**Code-Excerpt — Imports + `generate_route`** (`rest/src/extra_hours.rs:1-29`):

```rust
use std::rc::Rc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::ExtraHoursTO;

use serde::Deserialize;
use service::extra_hours::ExtraHoursService;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_extra_hours::<RestState>))
        .route("/{id}", put(update_extra_hours::<RestState>))
        .route("/{id}", delete(delete_extra_hours::<RestState>))
        .route(
            "/by-sales-person/{id}",
            get(get_extra_hours_for_sales_person::<RestState>),
        )
}
```

**Code-Excerpt — POST-Handler mit utoipa + error_handler** (`rest/src/extra_hours.rs:87-119`):

```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Extra Hours"],
    request_body = ExtraHoursTO,
    responses(
        (status = 201, description = "Extra hours created", body = ExtraHoursTO),
        (status = 400, description = "Invalid input"),
    ),
)]
pub async fn create_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person): Json<ExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let extra_hours = ExtraHoursTO::from(
                &rest_state
                    .extra_hours_service()
                    .create(&(&sales_person).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

**Code-Excerpt — PUT-Handler mit `path_id`-Override** (`rest/src/extra_hours.rs:121-162`):

```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Extra Hours"],
    params(
        ("id", description = "Extra hours id (logical id)", example = "1a2b3c4d-…"),
    ),
    request_body = ExtraHoursTO,
    responses(
        (status = 200, description = "Updated extra hours", body = ExtraHoursTO),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Extra hours not found"),
        (status = 409, description = "Version conflict"),
    ),
)]
pub async fn update_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
    Json(extra_hours_to): Json<ExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let mut entity: service::extra_hours::ExtraHours = (&extra_hours_to).into();
            entity.id = extra_hours_id;     // <-- path-id wins über body-id
            let updated = ExtraHoursTO::from(
                &rest_state
                    .extra_hours_service()
                    .update(&entity, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&updated).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

**Code-Excerpt — GET-by-ID-Variante** (`rest/src/booking.rs:76-97`):

```rust
#[instrument(skip(rest_state))]
pub async fn get_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let booking = rest_state
                .booking_service()
                .get(booking_id, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&BookingTO::from(&booking)).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}
```

**Code-Excerpt — ApiDoc-Struktur** (`rest/src/extra_hours.rs:194-207`):

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        get_extra_hours_for_sales_person,
        create_extra_hours,
        update_extra_hours,
        delete_extra_hours
    ),
    components(schemas(ExtraHoursTO)),
    tags(
        (name = "Extra Hours", description = "Extra hours management"),
    ),
)]
pub struct ExtraHoursApiDoc;
```

**Adaptation Notes:**

1. **6 Routen** nach RESEARCH.md §10:
   - `POST /` → `create_absence_period`
   - `GET /` → `get_all_absence_periods` (HR only)
   - `GET /{id}` → `get_absence_period`
   - `PUT /{id}` → `update_absence_period`
   - `DELETE /{id}` → `delete_absence_period`
   - `GET /by-sales-person/{sales_person_id}` → `get_absence_periods_for_sales_person`
2. **Alle Handler bekommen `#[utoipa::path]`** (CC-06) — auch das "GET all" und "GET by-id", die im Booking-Analog ohne utoipa sind. ExtraHours ist die richtige Vorlage hier.
3. **`#[instrument(skip(rest_state))]`** auf allen Handlern (Konvention).
4. **`path_id`-Override-Idiom** im PUT-Handler (`entity.id = absence_id;` nach dem `From<&AbsencePeriodTO>`-Parsing) — wörtlich aus ExtraHours-Vorlage.
5. **`AbsenceApiDoc`-Struct** analog `ExtraHoursApiDoc` mit allen 6 Pfaden in `paths(…)` und `components(schemas(AbsencePeriodTO, AbsenceCategoryTO))`.
6. **HTTP-Status-Codes** (RESEARCH.md §10 Tabelle): 201 (POST), 200 (GET/PUT), 204 (DELETE), 403 (Forbidden), 404 (NotFound), 409 (Conflict), 422 (ValidationError) — wird durch `error_handler` automatisch gemappt. Keine extra Logik im Handler nötig.
7. **`rest/src/lib.rs`-Patch (drei Stellen):**
   - ZZ. 3-26: `mod absence;`
   - ZZ. 296-298 (RestStateDef-Trait): `type AbsenceService: service::absence::AbsenceService<Context = Context> + Send + Sync + 'static;` und `fn absence_service(&self) -> Arc<Self::AbsenceService>;`
   - ZZ. 465 (im `nest(...)` der ApiDoc): `(path = "/absence-period", api = absence::AbsenceApiDoc),`
   - ZZ. 539 (im Router-`.nest`-Block): `.nest("/absence-period", absence::generate_route())`

---

### Layer F — Tests

#### `service_impl/src/test/absence.rs` (Test, Mock-DI)

**Analog:** `service_impl/src/test/extra_hours.rs` (komplett 1:1 Vorlage).

**Warum dieser Analog:** Liefert das **komplette Test-Setup mit `MockXxxDeps`-Struct, `build_dependencies()`-Helper, `_success`/`_forbidden`/`_not_found`/`_conflicts`/`_validation_error`-Pattern**, und die **expect_check_permission/expect_verify_user_is_sales_person-Mocks für die OR-Permission-Flow-Tests**.

**Code-Excerpt — Imports + Test-Konstanten** (`service_impl/src/test/extra_hours.rs:1-57`):

```rust
use std::sync::Arc;

use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::extra_hours::ExtraHoursEntity;
use dao::extra_hours::MockExtraHoursDao;
use dao::DaoError;
use dao::MockTransaction;
use dao::MockTransactionDao;
use mockall::predicate::always;
use mockall::predicate::eq;
use service::clock::MockClockService;
// …
use uuid::uuid;
use uuid::Uuid;

use crate::extra_hours::ExtraHoursServiceDeps;
use crate::extra_hours::ExtraHoursServiceImpl;
use crate::test::error_test::test_conflicts;
use crate::test::error_test::test_forbidden;
use crate::test::error_test::test_not_found;
use crate::test::error_test::test_validation_error;

pub fn default_logical_id() -> Uuid {
    uuid!("AA000000-0000-0000-0000-000000000001")
}
pub fn default_physical_id() -> Uuid {
    default_logical_id()  // first version: physical id == logical id
}
// … alternate_physical_id, default_sales_person_id, other_sales_person_id, default_version, …
```

**Code-Excerpt — Mock-Deps-Struct + Impl-Block** (`service_impl/src/test/extra_hours.rs:88-123`):

```rust
struct ExtraHoursDependencies {
    extra_hours_dao: MockExtraHoursDao,
    permission_service: MockPermissionService,
    sales_person_service: MockSalesPersonService,
    custom_extra_hours_service: MockCustomExtraHoursService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    transaction_dao: MockTransactionDao,
}

impl ExtraHoursServiceDeps for ExtraHoursDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ExtraHoursDao = MockExtraHoursDao;
    type PermissionService = MockPermissionService;
    // … alle Type-Aliases …
    type TransactionDao = MockTransactionDao;
}

impl ExtraHoursDependencies {
    fn build_service(self) -> ExtraHoursServiceImpl<ExtraHoursDependencies> {
        ExtraHoursServiceImpl {
            extra_hours_dao: self.extra_hours_dao.into(),
            permission_service: self.permission_service.into(),
            // … alle Felder mit .into() …
            transaction_dao: self.transaction_dao.into(),
        }
    }
}
```

**Code-Excerpt — `build_dependencies()`-Default-Mocks** (`service_impl/src/test/extra_hours.rs:124-156`):

```rust
fn build_dependencies() -> ExtraHoursDependencies {
    let extra_hours_dao = MockExtraHoursDao::new();
    let mut permission_service = MockPermissionService::new();
    let mut sales_person_service = MockSalesPersonService::new();
    let custom_extra_hours_service = MockCustomExtraHoursService::new();
    let mut clock_service = MockClockService::new();
    let uuid_service = MockUuidService::new();
    let mut transaction_dao = MockTransactionDao::new();

    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    clock_service
        .expect_date_time_now()
        .returning(|| datetime!(2026-04-28 12:00:00));
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    ExtraHoursDependencies { /* … */ }
}
```

**Code-Excerpt — `_forbidden`-Test** (`service_impl/src/test/extra_hours.rs:329-353`):

```rust
#[tokio::test]
async fn test_update_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}
```

**Adaptation Notes:**

1. **Test-Konstanten 1:1 kopieren** (`default_logical_id`, `default_physical_id`, `alternate_physical_id`, `default_sales_person_id`, `other_sales_person_id`, `default_version`, `alternate_version`, `unknown_logical_id`).
2. **`default_active_entity` und `default_update_request`** an `AbsencePeriodEntity`/`AbsencePeriod` mit Range-Feldern adaptieren — z.B. `from_date: date!(2026-04-12)`, `to_date: date!(2026-04-15)`, `category: AbsenceCategoryEntity::Vacation`.
3. **`AbsenceDependencies`-Struct** ohne `custom_extra_hours_service`-Feld (Phase-1-Empfehlung Option A — kein Schichtplan-Service-Dep).
4. **Test-Matrix nach RESEARCH.md §5** (~22 Tests). Mindestpflicht (D-11): pro public service method ein `_success` und ein `_forbidden` (= 12 Tests baseline).
5. **Spezielle Phase-1-Tests** (zusätzlich zur ExtraHours-Vorlage):
   - `test_create_inverted_range_returns_date_order_wrong` (D-14) → `test_date_order_wrong(&result)` aus `error_test`.
   - `test_create_self_overlap_same_category_returns_validation` (D-12, D-13) → `test_validation_error(&result, &ValidationFailureItem::OverlappingPeriod(other_logical_id), 1)`.
   - `test_create_self_overlap_different_category_succeeds` (D-12) — Vacation neben SickLeave erlaubt.
   - `test_update_self_overlap_excludes_self` (D-15) — Mock auf `find_overlapping(_, _, _, exclude_logical_id: Some(default_logical_id), _)`.
6. **`service_impl/src/test/mod.rs`-Patch:** Block `#[cfg(test)] pub mod absence;` neben den anderen (analog ZZ. 37-38).
7. **Helper aus `error_test.rs` benutzen:** `test_forbidden`, `test_not_found`, `test_conflicts`, `test_validation_error`, `test_date_order_wrong`. Keine eigenen Assertion-Helper schreiben.

---

#### `shifty_bin/src/integration_test/absence_period.rs` (Integration Test, full-stack)

**Analog:** `shifty_bin/src/integration_test/extra_hours_update.rs` (1:1 Vorlage; 271 Zeilen, deckt fast die komplette Test-Matrix der RESEARCH.md §11.2 ab).

**Warum dieser Analog:** Liefert **die direkte Vorlage für die Phase-1-Integration-Tests** — `TestSetup::new()`, `create_sales_person`-Helper, `create_extra_hours`-Helper, direkte SQLx-Queries für Schema-Constraint-Tests, `tombstone+active`-Verify.

**Code-Excerpt — Imports + Helper** (`shifty_bin/src/integration_test/extra_hours_update.rs:1-55`):

```rust
use std::sync::Arc;

use rest::RestStateDef;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService};
use service::permission::Authentication;
use service::sales_person::{SalesPerson, SalesPersonService};
use sqlx::Row;
use time::macros::datetime;
use uuid::Uuid;

use crate::integration_test::TestSetup;

async fn create_sales_person(test_setup: &TestSetup, name: &str) -> SalesPerson {
    test_setup
        .rest_state
        .sales_person_service()
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: name.into(),
                background_color: "#000000".into(),
                inactive: false,
                is_paid: Some(true),
                deleted: None,
            },
            Authentication::Full,
            None,
        )
        .await
        .unwrap()
}
```

**Code-Excerpt — Tombstone+Active-Test mit direkter SQLx-Query** (`shifty_bin/src/integration_test/extra_hours_update.rs:79-128`):

```rust
#[tokio::test]
async fn test_update_creates_tombstone_and_new_active_row() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Bob").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;

    let updated = test_setup
        .rest_state
        .extra_hours_service()
        .update(/* … */)
        .await
        .unwrap();
    // … assertions on logical_id, version rotation …

    let pool = test_setup.pool.as_ref();
    let logical_id_bytes = initial.id.as_bytes().to_vec();
    let rows = sqlx::query(
        "SELECT id, deleted, amount FROM extra_hours WHERE logical_id = ? ORDER BY created ASC",
    )
    .bind(&logical_id_bytes)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2, "tombstone + new active row");
    let tombstone_deleted: Option<String> = rows[0].get("deleted");
    let new_deleted: Option<String> = rows[1].get("deleted");
    assert!(tombstone_deleted.is_some(), "first row should be a tombstone");
    assert!(new_deleted.is_none(), "second row should be active");
}
```

**Code-Excerpt — Partial-Unique-Index-Constraint-Test** (`shifty_bin/src/integration_test/extra_hours_update.rs:192-231`):

```rust
#[tokio::test]
async fn test_partial_unique_index_rejects_two_active_rows_with_same_logical_id() {
    let test_setup = TestSetup::new().await;
    let sp = create_sales_person(&test_setup, "Eve").await;
    let initial = create_extra_hours(&test_setup, sp.id).await;
    // … direct INSERT with same logical_id and deleted=NULL …
    let result = sqlx::query(
        "INSERT INTO extra_hours \
         (id, logical_id, sales_person_id, amount, category, description, custom_extra_hours_id, \
          date_time, created, deleted, update_process, update_version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
    )
    .bind(/* … */)
    .execute(pool)
    .await;

    assert!(
        result.is_err(),
        "second active row with same logical_id should violate the partial unique index"
    );
}
```

**Code-Excerpt — `TestSetup::new()` mit Migrations-Run** (`shifty_bin/src/integration_test.rs:266-300`):

```rust
pub struct TestSetup {
    pub rest_state: RestStateImpl,
    pub pool: Arc<SqlitePool>,
    // …
}
impl TestSetup {
    pub async fn new() -> Self {
        let pool = Arc::new(
            SqlitePool::connect("sqlite:sqlite::memory:")
                .await
                .expect("Could not connect to database"),
        );
        sqlx::migrate!("./../migrations/sqlite")
            .run(pool.as_ref())
            .await
            .unwrap();

        let rest_state = RestStateImpl::new(pool.clone());
        create_admin_user(pool.clone(), "DEVUSER").await;
        // …
    }
}
```

**Adaptation Notes:**

1. **`create_sales_person`-Helper 1:1 kopieren** — funktioniert für Absence-Tests genauso.
2. **`create_absence_period`-Helper** statt `create_extra_hours`:
   ```rust
   async fn create_absence_period(test_setup: &TestSetup, sales_person_id: Uuid) -> AbsencePeriod {
       test_setup.rest_state.absence_service().create(
           &AbsencePeriod {
               id: Uuid::nil(),
               sales_person_id,
               category: AbsenceCategory::Vacation,
               from_date: date!(2026-04-12),
               to_date: date!(2026-04-15),
               description: "initial".into(),
               created: None,
               deleted: None,
               version: Uuid::nil(),
           },
           Authentication::Full,
           None,
       ).await.unwrap()
   }
   ```
3. **Mindest-Test-Set nach RESEARCH.md §11.2** (8 Tests):
   - `test_create_assigns_id_equal_to_logical_id` — analog ZZ. 58-75.
   - `test_update_creates_tombstone_and_new_active_row` — analog ZZ. 79-128.
   - `test_partial_unique_index_enforces_one_active_per_logical_id` — analog ZZ. 192-231 (direkte INSERT mit `deleted=NULL`).
   - `test_create_overlapping_same_category_returns_validation_error` — neue Self-Overlap-Detection.
   - `test_create_overlapping_different_category_succeeds` (D-12).
   - `test_update_can_extend_range_without_self_collision` (D-15) — Update-Range erweitern, alte Row darf nicht als Konflikt zählen.
   - `test_delete_softdeletes_row` — `find_by_id` → `None` nach delete.
   - `test_check_constraint_rejects_inverted_range` — direkter SQL-INSERT mit `to_date < from_date` muss DB-CHECK feuern.
4. **`create_admin_user(pool, "DEVUSER")`** wird automatisch durch `TestSetup::new()` aufgerufen — nichts zusätzlich nötig (siehe ZZ. 287).
5. **`shifty_bin/src/integration_test.rs`-Patch (NICHT integration_test/mod.rs!):** Es gibt im Repo keine `mod.rs` für `integration_test/` — die Module werden aus `integration_test.rs` heraus per `pub mod xxx;` registriert? Plan-Phase muss verifizieren: `grep "mod extra_hours_update" shifty_bin/src/integration_test.rs` zeigt, ob die Module dort hängen oder per `mod.rs`-Konvention. **Bei Unsicherheit: dem Pattern der bereits existierenden `integration_test/extra_hours_update.rs` folgen — wo die mod-Deklaration steht, dort kommt auch `pub mod absence_period;` hin.**
6. **`time::macros::date!`-Macro** für Date-Literale (siehe RESEARCH.md ZZ. 643).
7. **Imports analog ZZ. 1-11** — `use service::absence::{AbsencePeriod, AbsenceCategory, AbsenceService};` neu hinzu.

---

### Layer G — DI-Verdrahtung in `shifty_bin/src/main.rs`

#### `shifty_bin/src/main.rs` PATCH (DI, wiring)

**Analog:** ExtraHours-Block ZZ. 38, 223-236, 680-688.

**Warum dieser Analog:** Zeigt **alle drei Stellen, an denen ein neuer Service in `main.rs` verdrahtet wird**: (1) Type-Alias für DAO, (2) `*ServiceDependencies`-Struct mit `impl *ServiceDeps`, (3) konkrete `Arc::new(*ServiceImpl { … })`-Instanz. Außerdem zeigt das ExtraHoursService-Pattern, wie ein Service in den `RestStateImpl` integriert wird.

**Code-Excerpt — Type-Alias für DAO** (`shifty_bin/src/main.rs:10, 38`):

```rust
use dao_impl_sqlite::{
    /* … */
    extra_hours::ExtraHoursDaoImpl, sales_person::SalesPersonDaoImpl,
    /* … */
};
// …
type ExtraHoursDao = ExtraHoursDaoImpl;
```

**Code-Excerpt — ServiceDependencies-Block** (`shifty_bin/src/main.rs:223-236`):

```rust
pub struct ExtraHoursServiceDependencies;
impl service_impl::extra_hours::ExtraHoursServiceDeps for ExtraHoursServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursDao = ExtraHoursDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    type CustomExtraHoursService = CustomExtraHoursService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
type ExtraHoursService =
    service_impl::extra_hours::ExtraHoursServiceImpl<ExtraHoursServiceDependencies>;
```

**Code-Excerpt — Konkrete Arc-Instanz im `RestStateImpl::new`** (`shifty_bin/src/main.rs:680-688`):

```rust
let extra_hours_service = Arc::new(service_impl::extra_hours::ExtraHoursServiceImpl {
    extra_hours_dao,
    permission_service: permission_service.clone(),
    sales_person_service: sales_person_service.clone(),
    custom_extra_hours_service: custom_extra_hours_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```

**Adaptation Notes:**

1. **DAO-Type-Alias** (analog ZZ. 38): `type AbsenceDao = dao_impl_sqlite::absence::AbsenceDaoImpl;` und im `dao_impl_sqlite::{…}`-Use-Block `absence::AbsenceDaoImpl`.
2. **DAO-Instanz** im `RestStateImpl::new` (analog ZZ. 587 für ExtraHours): `let absence_dao = Arc::new(AbsenceDao::new(pool.clone()));`.
3. **`AbsenceServiceDependencies`-Block** mit Deps: `Context`, `Transaction`, `AbsenceDao`, `PermissionService`, `SalesPersonService`, `ClockService`, `UuidService`, `TransactionDao`. **Kein `CustomExtraHoursService`** (Phase-1-Empfehlung Option A).
4. **Konkrete `Arc::new(AbsenceServiceImpl { absence_dao, permission_service: …, sales_person_service: …, clock_service: …, uuid_service: …, transaction_dao: … })`** im `RestStateImpl::new` neben den anderen Service-Konstruktionen.
5. **`RestStateImpl`-Struct-Erweiterung** (analog ZZ. 447): `absence_service: Arc<AbsenceService>,`.
6. **`RestStateDef`-Impl** (analog ZZ. 476, 534): `type AbsenceService = AbsenceService;` und `fn absence_service(&self) -> Arc<Self::AbsenceService> { self.absence_service.clone() }`.
7. **`RestStateImpl`-Konstruktor-Return** (analog ZZ. 855): `absence_service,` zur Struct-Initialisierung hinzufügen.

**Plan-Phase-Reihenfolge in main.rs:** Pattern und Reihenfolge folgen **mechanisch** dem ExtraHours-Block — keine Discretion-Frage.

---

## Shared Patterns (cross-cutting)

### Pattern S-1: Authentication via `tokio::join!` (HR ∨ Self)

**Quelle:** `service_impl/src/extra_hours.rs:236-245` (Update-Variante mit ge-loadter aktiver Row).

**Anwenden auf:** Jede Schreib-Methode in `service_impl/src/absence.rs` (`create`, `update`, `delete`) und `find_by_sales_person`/`find_by_id` für Read-Permission (D-09, D-10 Option A).

```rust
let (hr_permission, sales_person_permission) = join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        active.sales_person_id,    // bei create: request.sales_person_id; bei update/delete: active.sales_person_id (vorher geladen)
        context,
        tx.clone().into()
    ),
);
hr_permission.or(sales_person_permission)?;
```

**Wichtig:** `tokio::join` aus `tokio::join` — Importe sind in `service_impl/src/extra_hours.rs:19` (`use tokio::join;`).

---

### Pattern S-2: HR-only Permission

**Quelle:** `service_impl/src/booking.rs:76-98` (`get_all` HR-only).

**Anwenden auf:** `find_all` in `service_impl/src/absence.rs`.

```rust
self.permission_service
    .check_permission(HR_PRIVILEGE, context)
    .await?;
```

---

### Pattern S-3: Transaction-Wrapping

**Quelle:** Jede Service-Methode in `service_impl/src/extra_hours.rs`.

**Anwenden auf:** Jede Service-Methode in `service_impl/src/absence.rs`.

```rust
async fn xxx(
    &self,
    /* … */,
    tx: Option<Self::Transaction>,
) -> Result<…, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // … business logic + DAO calls …
    self.transaction_dao.commit(tx).await?;
    Ok(/* … */)
}
```

---

### Pattern S-4: Soft-Delete im DAO-Read-SQL

**Quelle:** `dao_impl_sqlite/src/extra_hours.rs:91, 110, 130, 157` — jede Read-Query enthält `AND deleted IS NULL`.

**Anwenden auf:** Jede `query_as!`/`query!` in `dao_impl_sqlite/src/absence.rs`, **außer** der `update`-Methode (die schreibt das Tombstone).

```sql
SELECT … FROM absence_period WHERE … AND deleted IS NULL
```

CC-05 ist Pflicht. RESEARCH.md Pitfall 3 erklärt die Konsequenzen.

---

### Pattern S-5: Range-Validation (defense-in-depth)

**Quelle:** RESEARCH.md ZZ. 757-762 (Service-Helper) + Migration-CHECK (RESEARCH.md ZZ. 976).

**Anwenden auf:** Jeder `create`/`update`-Pfad im Service muss `DateRange::new(from, to)` bauen und bei Err `ServiceError::DateOrderWrong(from, to)` zurückgeben. DB-`CHECK (to_date >= from_date)` ist Backstop.

```rust
let new_range = DateRange::new(request.from_date, request.to_date)
    .map_err(|_| ServiceError::DateOrderWrong(request.from_date, request.to_date))?;
```

---

### Pattern S-6: Self-Overlap-Detection

**Quelle:** RESEARCH.md §7.1 (Allen-Algebra-SQL) + RESEARCH.md Update-Stencil ZZ. 382-395.

**Anwenden auf:** `create` (mit `exclude_logical_id: None`) und `update` (mit `exclude_logical_id: Some(logical_id)`).

```rust
let conflicts = self.absence_dao
    .find_overlapping(
        sales_person_id,
        (&category).into(),
        new_range,
        exclude,    // None für create, Some(logical_id) für update (D-15)
        tx.clone(),
    ).await?;
if !conflicts.is_empty() {
    return Err(ServiceError::ValidationError(Arc::from([
        ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
    ])));
}
```

**Discretion:** Variante `OverlappingPeriod(Uuid)` ist RESEARCH.md-Empfehlung (A1, §8). Plan-Phase darf alternativ `Duplicate` mit Kontext-String benutzen.

---

### Pattern S-7: utoipa-Annotation auf jedem Handler

**Quelle:** `rest/src/extra_hours.rs:42-55, 87-97, 121-137, 164-176`.

**Anwenden auf:** Alle 6 Handler in `rest/src/absence.rs`. CC-06 ist Pflicht.

```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    <method>,
    path = "<path>",
    tags = ["Absence"],
    params(/* … */),
    request_body = AbsencePeriodTO,    // nur bei POST/PUT
    responses(
        (status = 200, description = "…", body = AbsencePeriodTO),
        // … alle erwarteten Codes …
    ),
)]
```

---

### Pattern S-8: error_handler-Wrapper

**Quelle:** `rest/src/lib.rs:120-251`. Mappt `ServiceError` zu HTTP-Codes.

**Anwenden auf:** **Jeder** Handler-Body in `rest/src/absence.rs`.

```rust
error_handler(
    (async {
        // … Service-Call und Response-Build …
        Ok(Response::builder().status(<code>).header(…).body(…).unwrap())
    })
    .await,
)
```

**Mapping (für Phase-1-relevante Errors):**

| ServiceError | HTTP Status | Body |
|--------------|-------------|------|
| `Forbidden` | 403 | empty |
| `Unauthorized` | 401 | empty |
| `EntityNotFound(uuid)` | 404 | uuid string |
| `EntityConflicts(_, _, _)` | 409 | error string |
| `ValidationError(items)` | 422 | debug-format string |
| `IdSetOnCreate`/`VersionSetOnCreate`/`CreatedSetOnCreate`/`DeletedSetOnCreate` | 422 | error string |
| `DateOrderWrong(_, _)` | 422 | error string |
| `DatabaseQueryError` | 500 | error string |

---

## No Analog Found

Keine. Alle Phase-1-Dateien haben einen **starken Analog im Repo**. Die `DateRange`-Utility ist der schwächste Match (es gibt keinen direkten DateRange-Bestand), aber RESEARCH.md §4 liefert eine vollständige Implementierungs-Skizze, und `shifty-utils/src/date_utils.rs` gibt das Modul-Layout vor.

---

## Metadata

**Analog search scope:**
- `migrations/sqlite/` (alle 30 Migrations gelistet, Treffer: `20260428101456_…`)
- `dao/src/`, `dao_impl_sqlite/src/`, `service/src/`, `service_impl/src/`, `rest/src/` (per-domain Dateien gefiltert nach gleicher CRUD-Form)
- `service_impl/src/test/`, `shifty_bin/src/integration_test/` (Mock-DI + In-Memory-SQLite-Pattern)
- `rest-types/src/lib.rs` (DTO-Block-Suche)
- `shifty-utils/src/` (Utility-Layout-Suche)

**Files scanned:** 14 Analog-Dateien gelesen (insgesamt ~3.2k Zeilen). Keine Datei zweimal gelesen; bei großen Dateien (`shifty_bin/src/main.rs` 952 Zeilen, `rest/src/lib.rs` 619 Zeilen) wurden nur die relevanten Bereiche gelesen.

**Pattern extraction date:** 2026-05-01

**Pattern-Quality-Note:** Phase 1 ist (per RESEARCH.md "Don't Hand-Roll" Insight) **fast ausschließlich Pattern-Application**. Die einzigen wirklich neuen Konstrukte sind die `DateRange`-Utility (klein, vollständig im Research vorgegeben), die `find_overlapping`-Allen-SQL-Query (Pitfall-9-Two-Branch-Pattern) und ggf. die `ValidationFailureItem::OverlappingPeriod(Uuid)`-Variante (D-13/A1 — Plan-Phase-Discretion). Alles andere ist 1:1 von ExtraHours/Booking abgekupfert. Der Planner sollte deshalb keine eigenen Architektur-Entscheidungen treffen, sondern die hier zitierten Code-Stellen direkt referenzieren.

---

## PATTERN MAPPING COMPLETE

**Phase:** 1 — Absence Domain Foundation
**Files classified:** 14 neue + 9 Patches
**Analogs found:** 14 / 14 (alle exakt oder role-match mit klarem Code-Excerpt)

### Coverage
- Files mit exaktem Analog: 13 (alle bis auf `DateRange`-Utility)
- Files mit role-match Analog: 1 (`DateRange` — kein direkter Bestand, aber `date_utils.rs` als Style-Vorlage + RESEARCH.md §4 als vollständige Implementierungs-Skizze)
- Files ohne Analog: 0

### Key Patterns Identified
- Alle Service-Methoden folgen `tx = use_transaction → permission(HR ∨ self) → DAO-Calls → commit`-Idiom (S-1, S-3).
- DAO-Reads sind durchgängig partial (`WHERE deleted IS NULL`), DAO-`update` schreibt nur Tombstone (S-4).
- Update-Identität läuft IMMER über `find_by_logical_id → tombstone alte Row (UPDATE deleted) → INSERT neue Row mit gleicher logical_id, neuem id+version` — nie In-place-Mutation des Domain-Bodys.
- REST-Handler bestehen aus `#[instrument] + #[utoipa::path] + error_handler((async { … })`-Wrapper, mit `path-id wins`-Override im PUT (S-7, S-8).
- Tests benutzen `MockXxxDeps`-Struct + `build_dependencies()`-Default-Mocks + `error_test`-Helper für `_forbidden`/`_not_found`/`_conflicts`/`_validation_error`-Assertions.

### File Created
`/home/neosam/programming/rust/projects/shifty/shifty-backend/.planning/phases/01-absence-domain-foundation/01-PATTERNS.md`

### Ready for Planning
Pattern-Mapping vollständig. Der Planner kann nun für jede der 14 neuen Dateien direkt die Analog-Datei + Zeilenbereich + Adaptation-Notes referenzieren. Discretion-Punkte (`OverlappingPeriod(Uuid)` vs. `Duplicate`-Reuse, D-10 Read-Sicht Option A vs. B) sind in den Adaptation-Notes als "Plan-Phase entscheidet" markiert.
