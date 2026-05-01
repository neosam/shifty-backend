# Phase 2: Reporting Integration & Snapshot Versioning — Pattern Map

**Mapped:** 2026-05-01
**Files analyzed:** 17 Dateien (6 neu, 11 modifiziert)
**Analogs found:** 17 / 17 (alle Vorlagen im Repo lokalisiert)

> **Sprache:** Deutsche Prosa. Code, SQL, Identifier, Trait- und Modulnamen bleiben in Originalform.
>
> **Lese-Reihenfolge für den Planner:** Wave 0 (Feature-Flag-Infra + Migration) → Wave 1 (derive_hours_for_range) → Wave 2 (Reporting-Switch + UnpaidLeave-Snapshot + Version-Bump + Locking-Tests, alles ein Commit).

---

## File Classification

| Neue / geänderte Datei | Layer | Role | Data Flow | Closest Analog | Match Quality |
|------------------------|-------|------|-----------|----------------|---------------|
| `migrations/sqlite/<ts>_add-feature-flag-table.sql` | Migration | DDL: CREATE TABLE + Seed + Privileg-INSERT | DDL / one-shot | `migrations/sqlite/20260105000000_app-toggles.sql` | **exact** (1:1 Schema-Style) |
| `service/src/feature_flag.rs` | Service Trait | Trait + Domain-Modell + Privileg-Konstante + automock | request-response | `service/src/toggle.rs` | **exact** (schmaaler Subset) |
| `service_impl/src/feature_flag.rs` | Service Impl | gen_service_impl! + PermissionService-Check | request-response | `service_impl/src/toggle.rs` | **exact** |
| `service_impl/src/test/feature_flag.rs` | Test Unit | Mock-DI + is_enabled / set / forbidden Tests | request-response (mocked) | `service_impl/src/test/toggle.rs` | **exact** |
| `dao/src/feature_flag.rs` | DAO Trait | Trait + Entity + automock | CRUD | `dao/src/toggle.rs` | **exact** (schmaaler Subset: get + set) |
| `dao_impl_sqlite/src/feature_flag.rs` | DAO Impl | SQLx-Queries, is_enabled fail-safe false | CRUD | `dao_impl_sqlite/src/toggle.rs` | **exact** |
| `service/src/absence.rs` (PATCH) | Service Trait | Neue Trait-Methode `derive_hours_for_range` | transform / request-response | `service/src/absence.rs` (bestehend) | **self-extend** |
| `service_impl/src/absence.rs` (PATCH) | Service Impl | Implementation `derive_hours_for_range` + neue Deps | transform + transaction | `service_impl/src/absence.rs` (bestehend) + `find_working_hours_for_calendar_week` in `reporting.rs:72-81` | **role-match** |
| `service_impl/src/test/absence.rs` (PATCH) | Test Unit | Tests für derive_hours_for_range (3 Cases) | request-response (mocked) | `service_impl/src/test/absence.rs` (bestehend) | **self-extend** |
| `service/src/billing_period.rs` (PATCH) | Service Trait | UnpaidLeave-Variante + as_str + FromStr | transform | `service/src/billing_period.rs:34-90` | **self-extend** |
| `service_impl/src/billing_period_report.rs` (PATCH) | Service Impl | CURRENT_SNAPSHOT_SCHEMA_VERSION 2→3; UnpaidLeave-Insert | CRUD / transform | `service_impl/src/billing_period_report.rs:109-228` | **self-extend** |
| `service_impl/src/reporting.rs` (PATCH) | Service Impl | Feature-Flag-Switch + neue Deps (FeatureFlagService, AbsenceService) | request-response | `service_impl/src/reporting.rs:58-70` (gen_service_impl!) + Zeilen 558-583 | **self-extend** |
| `service_impl/src/test/billing_period_report.rs` (PATCH) | Test Unit | Locking-Tests (Pin-Map + Exhaustive-Match) | request-response (mocked) | `service_impl/src/test/billing_period_report.rs:1089-1149` | **self-extend** |
| `shifty_bin/src/main.rs` (PATCH) | DI | FeatureFlagServiceDependencies-Block; ReportingServiceDeps erweitern | wiring | `shifty_bin/src/main.rs:427-435` (ToggleServiceDependencies) + `295-309` (ReportingServiceDependencies) | **exact** |
| `service/src/lib.rs` (PATCH) | Module Re-Export | `pub mod feature_flag;` | wiring | `service/src/lib.rs` (bestehend: `pub mod toggle;`) | **exact** |
| `service_impl/src/lib.rs` (PATCH) | Module Re-Export | `pub mod feature_flag;` | wiring | `service_impl/src/lib.rs` (bestehend: `pub mod toggle;`) | **exact** |
| `dao/src/lib.rs` + `dao_impl_sqlite/src/lib.rs` (PATCH) | Module Re-Export | `pub mod feature_flag;` | wiring | analog zu `pub mod toggle;` in den jeweiligen lib.rs | **exact** |

---

## Pattern Assignments

### Wave 0 — Feature-Flag-Infrastruktur

---

#### `migrations/sqlite/<ts>_add-feature-flag-table.sql` (Migration, DDL)

**Analog:** `migrations/sqlite/20260105000000_app-toggles.sql` (vollständig gelesen)

**Schema-Pattern** (Zeilen 1-8):

```sql
-- migrations/sqlite/20260105000000_app-toggles.sql:1-8
CREATE TABLE toggle (
    name TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,  -- 0 = disabled, 1 = enabled
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
```

**Adaptation für `feature_flag`:** Tabelle heißt `feature_flag`, Spalte `key` statt `name` (TEXT PK). Kein Group-Management. Kein `toggle_group`-/`toggle_group_toggle`-Teil.

**Seed + Privileg-INSERT** (Zeilen 29-30):

```sql
-- migrations/sqlite/20260105000000_app-toggles.sql:29-30
INSERT INTO privilege (name, update_process) VALUES ('toggle_admin', 'initial');
```

**Komplettes Template für `feature_flag`:**

```sql
CREATE TABLE feature_flag (
    key TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

INSERT INTO feature_flag (key, enabled, description, update_process)
VALUES (
    'absence_range_source_active',
    0,
    'When ON, range-based AbsencePeriods are the source of truth for Vacation/Sick/UnpaidLeave hours. Flip atomically with Phase-4 migration; do NOT flip manually.',
    'phase-2-migration'
);

INSERT INTO privilege (name, update_process)
VALUES ('feature_flag_admin', 'initial');
```

**Adaptation Notes:**
- Kein `toggle_group`-Block.
- `key` statt `name` als PK (semantisch klarer für K/V-Flags).
- Kein Soft-Delete — `feature_flag`-Zeilen werden nicht gesoft-deletet (D-Phase2-06).
- Timestamp des Migrations-Namens: `sqlx migrate add add-feature-flag-table --source migrations/sqlite` (in `nix-shell -p sqlx-cli`).

---

#### `service/src/feature_flag.rs` (Service Trait)

**Analog:** `service/src/toggle.rs` (vollständig gelesen, Zeilen 1-183)

**Imports + Privilege-Konstante** (Zeilen 1-9):

```rust
// service/src/toggle.rs:1-9
use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;

pub const TOGGLE_ADMIN_PRIVILEGE: &str = "toggle_admin";
```

**Adaptation:** `TOGGLE_ADMIN_PRIVILEGE` → `FEATURE_FLAG_ADMIN_PRIVILEGE: &str = "feature_flag_admin"`.

**Domain-Model + automock-Trait** (Zeilen 11-74, schmaaler Subset):

```rust
// service/src/toggle.rs:11-26 (Domain-Struct)
#[derive(Clone, Debug, PartialEq)]
pub struct Toggle {
    pub name: Arc<str>,
    pub enabled: bool,
    pub description: Option<Arc<str>>,
}

// service/src/toggle.rs:62-87 (#[automock] + is_enabled + get_toggle)
#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait ToggleService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn is_enabled(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;
    // ... (Phase 2 braucht nur is_enabled + set)
}
```

**Adaptation für `FeatureFlagService`:**
- Domain-Struct: `FeatureFlag { key: Arc<str>, enabled: bool, description: Option<Arc<str>> }`.
- Trait-Methoden: nur `is_enabled(key, ctx, tx)` und `set(key, value, ctx, tx)` (schmaales API per D-Phase2-07).
- `#[automock]` bleibt identisch — erzeugt `MockFeatureFlagService`.

---

#### `service_impl/src/feature_flag.rs` (Service Impl)

**Analog:** `service_impl/src/toggle.rs` (vollständig gelesen, Zeilen 1-341)

**gen_service_impl! + Deps-Block** (Zeilen 1-19):

```rust
// service_impl/src/toggle.rs:1-19
use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{toggle::ToggleDao, TransactionDao};
use service::{
    permission::Authentication,
    toggle::{Toggle, ToggleGroup, ToggleService, TOGGLE_ADMIN_PRIVILEGE},
    PermissionService, ServiceError,
};
use std::sync::Arc;

const TOGGLE_SERVICE_PROCESS: &str = "toggle-service";

gen_service_impl! {
    struct ToggleServiceImpl: ToggleService = ToggleServiceDeps {
        ToggleDao: ToggleDao<Transaction = Self::Transaction> = toggle_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**is_enabled-Pattern mit Auth-Check** (Zeilen 26-42):

```rust
// service_impl/src/toggle.rs:26-42
async fn is_enabled(
    &self,
    name: &str,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<bool, ServiceError> {
    // Requires authentication (user must be logged in)
    let user_id = self.permission_service.current_user_id(context).await?;
    if user_id.is_none() {
        return Err(ServiceError::Unauthorized);
    }

    let tx = self.transaction_dao.use_transaction(tx).await?;
    let result = self.toggle_dao.is_enabled(name, tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

**Permission-Check für Admin-Methoden** (Zeilen 79-96):

```rust
// service_impl/src/toggle.rs:79-96 (create_toggle als Vorlage für set)
async fn create_toggle(
    &self,
    toggle: &Toggle,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError> {
    // Requires toggle_admin privilege
    self.permission_service
        .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
        .await?;

    // ...
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // DAO call...
    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```

**Adaptation Notes:**
- `FeatureFlagServiceImpl` hat identischen `gen_service_impl!`-Block, aber mit `FeatureFlagDao` statt `ToggleDao`.
- `is_enabled` für nicht-existenten Key: fail-safe `false` (identisch zu DAO — Zeile 169 in `dao_impl_sqlite/src/toggle.rs`).
- `set(key, value, ctx, tx)` braucht `check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, ctx)`.

---

#### `dao/src/feature_flag.rs` (DAO Trait)

**Analog:** `dao/src/toggle.rs` (vollständig gelesen, Zeilen 1-127)

**Trait-Struktur mit automock** (Zeilen 1-55, schmaaler Subset):

```rust
// dao/src/toggle.rs:1-55
use crate::DaoError;
use mockall::automock;

#[derive(Clone, Debug, PartialEq)]
pub struct ToggleEntity {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait ToggleDao {
    type Transaction: crate::Transaction;

    async fn get_toggle(
        &self,
        name: &str,
        tx: Self::Transaction,
    ) -> Result<Option<ToggleEntity>, DaoError>;

    async fn is_enabled(&self, name: &str, tx: Self::Transaction) -> Result<bool, DaoError>;

    async fn update_toggle(
        &self,
        toggle: &ToggleEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
```

**Adaptation für `FeatureFlagDao`:**
- Entity: `FeatureFlagEntity { key: String, enabled: bool, description: Option<String> }`.
- Trait-Methoden: `get(key, tx)`, `set(key, enabled, process, tx)`, `is_enabled(key, tx)` (schmaales API per C-Phase2-03).
- `#[automock]` bleibt identisch.

---

#### `dao_impl_sqlite/src/feature_flag.rs` (DAO Impl)

**Analog:** `dao_impl_sqlite/src/toggle.rs` (vollständig gelesen, Zeilen 1-373)

**Db-Struct + From-Impl** (Zeilen 12-26):

```rust
// dao_impl_sqlite/src/toggle.rs:12-26
#[derive(Debug)]
struct ToggleDb {
    name: String,
    enabled: i64,
    description: Option<String>,
}

impl From<&ToggleDb> for ToggleEntity {
    fn from(db: &ToggleDb) -> Self {
        ToggleEntity {
            name: db.name.clone(),
            enabled: db.enabled != 0,
            description: db.description.clone(),
        }
    }
}
```

**Pool-Struct + new()** (Zeilen 43-51):

```rust
// dao_impl_sqlite/src/toggle.rs:43-51
pub struct ToggleDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl ToggleDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}
```

**is_enabled mit fail-safe false** (Zeilen 159-170 — das entscheidende Pattern):

```rust
// dao_impl_sqlite/src/toggle.rs:159-170
async fn is_enabled(&self, name: &str, tx: Self::Transaction) -> Result<bool, DaoError> {
    let result = query!(
        r#"SELECT enabled FROM toggle WHERE name = ?"#,
        name,
    )
    .fetch_optional(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?;

    // Returns false for non-existent toggles (fail-safe default)
    Ok(result.map(|row| row.enabled != 0).unwrap_or(false))
}
```

**SQLx-execute-Pattern** (Zeilen 57-75):

```rust
// dao_impl_sqlite/src/toggle.rs:57-75
query!(
    r#"INSERT INTO toggle (name, enabled, description, update_process)
       VALUES (?, ?, ?, ?)"#,
    toggle.name,
    enabled,
    toggle.description,
    process,
)
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;
```

**Adaptation Notes:**
- `name` → `key` in allen Queries (Spaltenname in `feature_flag`).
- Kein Group-Management — nur `get`, `set`, `is_enabled`.
- `set` macht ein `UPDATE feature_flag SET enabled = ?, update_process = ? WHERE key = ?`.

---

#### `service_impl/src/test/feature_flag.rs` (Unit Tests)

**Analog:** `service_impl/src/test/toggle.rs` (vollständig gelesen, Zeilen 1-100+)

**Deps-Struct + Trait-Impl + build_service()** (Zeilen 12-46):

```rust
// service_impl/src/test/toggle.rs:12-46
pub struct ToggleServiceDependencies {
    pub toggle_dao: MockToggleDao,
    pub permission_service: MockPermissionService,
}

impl ToggleServiceDeps for ToggleServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ToggleDao = MockToggleDao;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl ToggleServiceDependencies {
    pub fn build_service(self) -> ToggleServiceImpl<ToggleServiceDependencies> {
        let mut transaction_dao = MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        ToggleServiceImpl {
            toggle_dao: self.toggle_dao.into(),
            permission_service: Arc::new(self.permission_service),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}
```

**NoneTypeExt + mock_authenticated helper** (Zeilen 71-96):

```rust
// service_impl/src/test/toggle.rs:71-96
trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }
}

fn mock_authenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("test_user".into())));
    permission_service
}

fn mock_unauthenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(None));
    permission_service
}
```

---

### Wave 1 — derive_hours_for_range

---

#### `service/src/absence.rs` (PATCH — neue Trait-Methode)

**Self-Extend:** Die bestehende `AbsenceService`-Trait-Datei wird um eine weitere Methode erweitert.

**Bestehende Trait-Signatur als Vorlage** (Zeilen 126-132):

```rust
// service/src/absence.rs:126-132
async fn find_by_sales_person(
    &self,
    sales_person_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[AbsencePeriod]>, ServiceError>;
```

**Neue Methode (anzufügen nach `delete`):**

```rust
/// Conflict-resolved per-day map for a sales person in a date range.
/// SickLeave > Vacation > UnpaidLeave (D-Phase2-03).
/// Days with 0 contract hours (weekend/holiday) produce 0.
/// HR ∨ self — gleiche Permission wie find_by_sales_person.
async fn derive_hours_for_range(
    &self,
    from: time::Date,
    to: time::Date,
    sales_person_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<std::collections::BTreeMap<time::Date, ResolvedAbsence>, ServiceError>;
```

**Neues Domain-Struct `ResolvedAbsence` (in derselben Datei, vor dem Trait):**

```rust
/// Output von derive_hours_for_range — pro Tag bereits conflict-resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedAbsence {
    pub category: AbsenceCategory,
    pub hours: f32,
}
```

**Adaptation Notes:**
- Return-Type `BTreeMap<time::Date, ResolvedAbsence>` per C-Phase2-01.
- `#[automock]` auf dem Trait erzeugt `MockAbsenceService::expect_derive_hours_for_range()` automatisch.

---

#### `service_impl/src/absence.rs` (PATCH — Implementation)

**Analog für Per-Tag-Vertragsstunden-Lookup:** `service_impl/src/reporting.rs:72-81` (`find_working_hours_for_calendar_week`):

```rust
// service_impl/src/reporting.rs:72-81
pub fn find_working_hours_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> impl Iterator<Item = &EmployeeWorkDetails> {
    working_hours.iter().filter(move |wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}
```

**Adaptation:** Der Planner implementiert analog `find_working_hours_for_date(working_hours, date)` — findet den aktiven Vertrag für einen Tag via `from_date()` / `to_date()` statt calendar-week-Vergleich. Batch-Fetch einmal vor der Iteration, dann per Tag filtern (kein per-Tag-DAO-Call).

**Neuer Dep `SpecialDayService` im gen_service_impl!-Block** — bestehend (Zeilen 35-44):

```rust
// service_impl/src/absence.rs:35-44
gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Adaptation:** `SpecialDayService` und `EmployeeWorkDetailsService` als neue Deps hinzufügen (analog zu `ShiftplanViewServiceDependencies` in `main.rs:268-278` als Beispiel für multi-Dep-Block).

**Cross-Category-Resolver-Pseudo-Code für den Executor:**

```rust
// Für jeden Tag in DateRange::iter_days():
// 1. Batch-Lookup: working_hours = find_by_sales_person_id(...) einmal vor Schleife
// 2. active_contract = working_hours.iter().find(|wh| wh.from_date() <= day && day <= wh.to_date())
// 3. hours_for_day = active_contract.map(|wh| wh.hours_per_day()).unwrap_or(0.0)
// 4. Wenn hours_for_day == 0.0: alle Kategorien = 0 (Wochenende / kein Vertrag)
// 5. is_holiday: SpecialDayService::get_by_week(year, week) für betroffene Wochen (batch),
//    dann filter by exact date
// 6. Wenn Feiertag: hours_for_day = 0.0
// 7. Unter aktiven AbsencePeriods: Priorität SickLeave > Vacation > UnpaidLeave
// 8. Dominante Kategorie bekommt hours_for_day, alle anderen 0
// 9. BTreeMap::insert(day, ResolvedAbsence { category, hours })
```

---

#### `service_impl/src/test/absence.rs` (PATCH — neue Tests)

**Self-Extend:** Bestehende `AbsenceDependencies`-Struct und `build_dependencies()` werden wiederverwendet.

**Existierendes Deps-Pattern** (Zeilen 119-150):

```rust
// service_impl/src/test/absence.rs:119-150
struct AbsenceDependencies {
    absence_dao: MockAbsenceDao,
    permission_service: MockPermissionService,
    sales_person_service: MockSalesPersonService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    transaction_dao: MockTransactionDao,
}

impl AbsenceServiceDeps for AbsenceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    // ... alle type-Aliase ...
}

impl AbsenceDependencies {
    fn build_service(self) -> AbsenceServiceImpl<AbsenceDependencies> {
        AbsenceServiceImpl {
            absence_dao: self.absence_dao.into(),
            permission_service: self.permission_service.into(),
            // ...
        }
    }
}
```

**Adaptation Notes:**
- `AbsenceDependencies` bekommt `special_day_service: MockSpecialDayService` und `employee_work_details_service: MockEmployeeWorkDetailsService` als neue Felder — analog zum bestehenden Pattern.
- 3 neue Tests: `test_derive_hours_for_range_basic`, `test_derive_hours_holiday_is_zero`, `test_derive_hours_contract_change`.

---

### Wave 2 — Reporting-Switch + Snapshot-Bump + Locking-Tests (ein atomarer Commit)

---

#### `service/src/billing_period.rs` (PATCH — UnpaidLeave-Variante)

**Self-Extend:** Enum + as_str + FromStr erweitern.

**Bestehender Enum-Block** (Zeilen 33-90):

```rust
// service/src/billing_period.rs:33-90
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BillingPeriodValueType {
    Balance,
    Overall,
    ExpectedHours,
    ExtraWork,
    VacationHours,
    SickLeave,
    Holiday,
    Volunteer,
    CustomExtraHours(Arc<str>),
    VacationDays,
    VacationEntitlement,
}
impl BillingPeriodValueType {
    pub fn as_str(&self) -> Arc<str> {
        match self {
            BillingPeriodValueType::Balance => "balance".into(),
            BillingPeriodValueType::SickLeave => "sick_leave".into(),
            // ...
        }
    }
}
impl FromStr for BillingPeriodValueType {
    type Err = BillingPeriodValueTypeParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sick_leave" => Ok(BillingPeriodValueType::SickLeave),
            // ... (fehlend: "unpaid_leave")
            _ => Err(...)
        }
    }
}
```

**Neue Variante (nach SickLeave einfügen):**

```rust
// Enum: nach SickLeave einfügen
UnpaidLeave,

// as_str(): nach SickLeave-Arm einfügen
BillingPeriodValueType::UnpaidLeave => "unpaid_leave".into(),

// FromStr: nach "sick_leave"-Arm einfügen
"unpaid_leave" => Ok(BillingPeriodValueType::UnpaidLeave),
```

---

#### `service_impl/src/billing_period_report.rs` (PATCH — Version-Bump + UnpaidLeave-Insert)

**Self-Extend:** Zwei Änderungen.

**Bump-Stelle** (Zeile 37):

```rust
// service_impl/src/billing_period_report.rs:37
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 2;  // → 3
```

**UnpaidLeave-Insert nach SickLeave** (nach Zeilen 155-163):

```rust
// service_impl/src/billing_period_report.rs:155-163 (bestehender SickLeave-Insert)
billing_period_values.insert(
    BillingPeriodValueType::SickLeave,
    BillingPeriodValue {
        value_delta: report_delta.sick_leave_hours,
        value_ytd_from: report_start.sick_leave_hours,
        value_ytd_to: report_end.sick_leave_hours,
        value_full_year: report_end_of_year.sick_leave_hours,
    },
);
// NEU: Direkt danach einfügen (1:1 analog):
billing_period_values.insert(
    BillingPeriodValueType::UnpaidLeave,
    BillingPeriodValue {
        value_delta: report_delta.unpaid_leave_hours,
        value_ytd_from: report_start.unpaid_leave_hours,
        value_ytd_to: report_end.unpaid_leave_hours,
        value_full_year: report_end_of_year.unpaid_leave_hours,
    },
);
```

---

#### `service_impl/src/reporting.rs` (PATCH — Feature-Flag-Switch)

**Self-Extend:** gen_service_impl!-Block erweitern + Switch in get_report_for_employee_range.

**Bestehender gen_service_impl!-Block** (Zeilen 58-70):

```rust
// service_impl/src/reporting.rs:58-70
gen_service_impl! {
    struct ReportingServiceImpl: ReportingService = ReportingServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction> = extra_hours_service,
        ShiftplanReportService: ShiftplanReportService<Transaction = Self::Transaction> = shiftplan_report_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        CarryoverService: CarryoverService<Transaction = Self::Transaction> = carryover_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Neue Deps (hinzufügen):**

```rust
FeatureFlagService: FeatureFlagService<Context = Self::Context, Transaction = Self::Transaction> = feature_flag_service,
AbsenceService: AbsenceService<Context = Self::Context, Transaction = Self::Transaction> = absence_service,
```

**Bestehende Switch-Stellen in `get_report_for_employee_range`** (Zeilen 558-583):

```rust
// service_impl/src/reporting.rs:558-583 — diese Felder werden von ExtraHours befüllt
vacation_hours: extra_hours
    .iter()
    .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::Vacation)
    .map(|extra_hours| extra_hours.amount)
    .sum(),
sick_leave_hours: extra_hours
    .iter()
    .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::SickLeave)
    .map(|extra_hours| extra_hours.amount)
    .sum(),
// ...
unpaid_leave_hours: extra_hours
    .iter()
    .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::UnpaidLeave)
    .map(|extra_hours| extra_hours.amount)
    .sum(),
```

**Switch-Mechanismus (einzufügen vor `let extra_hours = ...` auf Zeile 456):**

```rust
// Einmaliger Flag-Read am Anfang der Funktion — vor ExtraHours-Aggregation
let use_absence_range_source = self
    .feature_flag_service
    .is_enabled("absence_range_source_active", Authentication::Full, tx.clone().into())
    .await?;
```

**Conditional-Block als Vorlage (D-Phase2-08-A):**

```rust
// if use_absence_range_source:
//   derived = self.absence_service.derive_hours_for_range(from_date, to_date, sales_person_id, ...)
//   vacation_hours = derived.iter().filter(|(_d, r)| r.category == AbsenceCategory::Vacation).map(|(_d,r)| r.hours).sum()
//   sick_leave_hours = ...
//   unpaid_leave_hours = ...
// else:
//   (bestehende ExtraHours-Filter wie oben, unverändert)
```

---

#### `service_impl/src/test/billing_period_report.rs` (PATCH — Locking-Tests)

**Analog:** Bestehende Tests in derselben Datei (Zeilen 1089-1149 — vollständig gelesen).

**Bestehender Test-Aufbau** (Zeilen 1089-1117):

```rust
// service_impl/src/test/billing_period_report.rs:1089-1117
#[tokio::test]
async fn test_build_and_persist_writes_current_snapshot_schema_version() {
    let mut deps = setup_build_and_persist_mocks();

    deps.billing_period_service
        .expect_create_billing_period()
        .withf(|bp, _process, _ctx, _tx| {
            bp.snapshot_schema_version == CURRENT_SNAPSHOT_SCHEMA_VERSION
        })
        .times(1)
        .returning(|bp, _process, _ctx, _tx| Ok(bp.clone()));

    let service = deps.build_service();
    let result = service
        .build_and_persist_billing_period_report(
            shifty_utils::ShiftyDate::from_ymd(2024, 7, 31).unwrap(),
            Authentication::Full,
            None,
        )
        .await;

    assert!(result.is_ok(), "...");
}
```

**Locking-Test-1 (Pin-Map) — Skelett mit Header-Kommentar:**

```rust
/// LOCKING TEST — DO NOT NAIVELY UPDATE.
///
/// If this test fails after a code change:
///   - Did you intentionally change the snapshot computation?
///   - If yes, you MUST also bump CURRENT_SNAPSHOT_SCHEMA_VERSION
///     in service_impl/src/billing_period_report.rs.
///   - See CLAUDE.md § "Billing Period Snapshot Schema Versioning"
///     for the bump-trigger rules.
#[tokio::test]
async fn test_snapshot_v3_pinned_values() {
    // Fixture: 1 Sales-Person, 8h/Tag 5 Werktage Mo-Fr
    // Range: 2024-06-03 (Mo) bis 2024-06-09 (So)
    // - Vacation AbsencePeriod: 2024-06-03..2024-06-05 (Mo-Mi)
    // - SickLeave AbsencePeriod: 2024-06-04..2024-06-04 (Di — ueberlappt)
    // - ExtraHours ExtraWork: 2024-06-06 +2h
    // assert_eq! pro BillingPeriodValueType-Variante
}
```

**Locking-Test-2 (Compiler-Exhaustive-Match) — Skelett:**

```rust
/// LOCKING TEST — DO NOT NAIVELY UPDATE.
/// Wenn der Compiler hier eine fehlende Variante meldet:
/// bist du sicher, dass du nicht CURRENT_SNAPSHOT_SCHEMA_VERSION bumpen wolltest?
/// Siehe CLAUDE.md § "Billing Period Snapshot Schema Versioning".
#[test]
fn test_billing_period_value_type_surface_locked() {
    fn ensure_locked(value_type: &BillingPeriodValueType) {
        match value_type {
            BillingPeriodValueType::Overall => {}
            BillingPeriodValueType::Balance => {}
            BillingPeriodValueType::ExpectedHours => {}
            BillingPeriodValueType::ExtraWork => {}
            BillingPeriodValueType::VacationHours => {}
            BillingPeriodValueType::SickLeave => {}
            BillingPeriodValueType::UnpaidLeave => {}   // NEU in v3
            BillingPeriodValueType::Holiday => {}
            BillingPeriodValueType::Volunteer => {}
            BillingPeriodValueType::VacationDays => {}
            BillingPeriodValueType::VacationEntitlement => {}
            BillingPeriodValueType::CustomExtraHours(_) => {}
        }
    }
    // ensure_locked wird nie aufgerufen — nur Compiler-Check
    let _ = ensure_locked;
}
```

**MockDeps-Struct als Vorlage** (Zeilen 20-44 — bestehend):

```rust
// service_impl/src/test/billing_period_report.rs:20-44
struct MockDeps {
    billing_period_service: service::billing_period::MockBillingPeriodService,
    reporting_service: service::reporting::MockReportingService,
    // ... alle Mock-Typen
}
impl BillingPeriodReportServiceDeps for MockDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    // ... alle type-Aliase
}
impl MockDeps {
    fn build_service(self) -> BillingPeriodReportServiceImpl<MockDeps> {
        BillingPeriodReportServiceImpl {
            billing_period_service: self.billing_period_service.into(),
            // ...
        }
    }
}
```

---

#### `shifty_bin/src/main.rs` (PATCH — DI-Verdrahtung)

**Analog:** `ToggleServiceDependencies` (Zeilen 427-435):

```rust
// shifty_bin/src/main.rs:427-435
pub struct ToggleServiceDependencies;
impl service_impl::toggle::ToggleServiceDeps for ToggleServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ToggleDao = ToggleDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type ToggleService = service_impl::toggle::ToggleServiceImpl<ToggleServiceDependencies>;
```

**Neuer Block für `FeatureFlagServiceDependencies`:**

```rust
pub struct FeatureFlagServiceDependencies;
impl service_impl::feature_flag::FeatureFlagServiceDeps for FeatureFlagServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type FeatureFlagDao = FeatureFlagDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type FeatureFlagService =
    service_impl::feature_flag::FeatureFlagServiceImpl<FeatureFlagServiceDependencies>;
```

**Bestehender `ReportingServiceDependencies`-Block** (Zeilen 295-309):

```rust
// shifty_bin/src/main.rs:295-309
pub struct ReportingServiceDependencies;
impl service_impl::reporting::ReportingServiceDeps for ReportingServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursService = ExtraHoursService;
    type ShiftplanReportService = ShiftplanReportService;
    type EmployeeWorkDetailsService = WorkingHoursService;
    type SalesPersonService = SalesPersonService;
    type CarryoverService = CarryoverService;
    type PermissionService = PermissionService;
    type ClockService = ClockService;
    type UuidService = UuidService;
    type TransactionDao = TransactionDao;
}
```

**Adaptation:** Zwei neue Zeilen in den Impl-Block:

```rust
type FeatureFlagService = FeatureFlagService;
type AbsenceService = AbsenceService;
```

**Konstruktions-Pattern** (Zeilen 738-748):

```rust
// shifty_bin/src/main.rs:738-748
let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl {
    extra_hours_service: extra_hours_service.clone(),
    shiftplan_report_service: shiftplan_report_service.clone(),
    employee_work_details_service: working_hours_service.clone(),
    sales_person_service: sales_person_service.clone(),
    carryover_service: carryover_service.clone(),
    permission_service: permission_service.clone(),
    clock_service: clock_service.clone(),
    uuid_service: uuid_service.clone(),
    transaction_dao: transaction_dao.clone(),
});
```

**Adaptation:** Zwei neue Felder:

```rust
feature_flag_service: feature_flag_service.clone(),
absence_service: absence_service.clone(),
```

---

## Shared Patterns

### gen_service_impl! + Transaction-Boilerplate
**Source:** `service_impl/src/toggle.rs:13-19` und `service_impl/src/absence.rs:35-44`
**Apply to:** `service_impl/src/feature_flag.rs`, alle Patches auf bestehende Service-Impls

```rust
gen_service_impl! {
    struct FooServiceImpl: FooService = FooServiceDeps {
        FooDao: FooDao<Transaction = Self::Transaction> = foo_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

// Jede Methode:
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... Business-Logik ...
self.transaction_dao.commit(tx).await?;
```

### MockTransactionDao-Setup in Tests
**Source:** `service_impl/src/test/toggle.rs:27-32` und `service_impl/src/test/absence.rs:169-172`
**Apply to:** alle neuen Test-Dateien

```rust
let mut transaction_dao = MockTransactionDao::new();
transaction_dao
    .expect_use_transaction()
    .returning(|_| Ok(MockTransaction));
transaction_dao.expect_commit().returning(|_| Ok(()));
```

### Permission-Check (Admin vs. Auth-only)
**Source:** `service_impl/src/toggle.rs:32-41` (auth-only) und `service_impl/src/toggle.rs:85-88` (admin-only)
**Apply to:** `service_impl/src/feature_flag.rs` — `is_enabled` braucht nur `current_user_id != None`, `set` braucht `check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, ctx)`.

```rust
// Auth-only (is_enabled):
let user_id = self.permission_service.current_user_id(context).await?;
if user_id.is_none() {
    return Err(ServiceError::Unauthorized);
}

// Admin-only (set):
self.permission_service
    .check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, context)
    .await?;
```

### SQLx-Query mit tx.tx.lock().await.as_mut()
**Source:** `dao_impl_sqlite/src/toggle.rs:64-74`
**Apply to:** `dao_impl_sqlite/src/feature_flag.rs`

```rust
.execute(tx.tx.lock().await.as_mut())
.await
.map_db_error()?;
```

### BillingPeriodValueType-Insert-Pattern
**Source:** `service_impl/src/billing_period_report.rs:155-163`
**Apply to:** UnpaidLeave-Insert direkt nach SickLeave

```rust
billing_period_values.insert(
    BillingPeriodValueType::SickLeave,
    BillingPeriodValue {
        value_delta: report_delta.sick_leave_hours,
        // value_ytd_from, value_ytd_to, value_full_year analog
    },
);
// UnpaidLeave analog:
billing_period_values.insert(
    BillingPeriodValueType::UnpaidLeave,
    BillingPeriodValue {
        value_delta: report_delta.unpaid_leave_hours,
        // ...
    },
);
```

---

## Commit-Atomarität (Wave 2)

Folgende Dateien MÜSSEN in einem einzigen jj-Commit zusammengeführt werden (CLAUDE.md-Pflicht, D-Phase2-10):

1. `service/src/billing_period.rs` — `UnpaidLeave`-Variante
2. `service_impl/src/billing_period_report.rs` — Version 2 → 3 + UnpaidLeave-Insert
3. `service_impl/src/reporting.rs` — Feature-Flag-Switch + neue Deps
4. `service_impl/src/test/billing_period_report.rs` — Locking-Tests (Pin-Map + Exhaustive-Match)
5. `migrations/sqlite/<ts>_add-feature-flag-table.sql` — falls noch nicht in Wave-0-Commit

Wave-0-Commit (FeatureFlagService-Infra) und Wave-1-Commit (derive_hours_for_range) dürfen separate Commits sein — Wave-2 ist der Single-Commit-Constraint.

---

## No Analog Found

Kein File ohne Analog — alle Phase-2-Dateien haben einen direkten strukturellen Vorläufer im Repo.

---

## Metadata

**Analog-Suche-Scope:** `service/`, `service_impl/`, `dao/`, `dao_impl_sqlite/`, `shifty_bin/src/`, `migrations/sqlite/`
**Dateien gescannt:** 20 Quelldateien direkt gelesen
**Pattern-Extraktion:** 2026-05-01
