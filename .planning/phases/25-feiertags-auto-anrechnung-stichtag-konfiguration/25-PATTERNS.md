# Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration - Pattern Map

**Mapped:** 2026-06-28
**Files analyzed:** 18 new/modified files
**Analogs found:** 17 / 18

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `migrations/sqlite/20260628000000_toggle-value-column.sql` | migration | transform | `migrations/sqlite/20260105000000_app-toggles.sql` | role-match |
| `migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql` | migration | transform | `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` | exact |
| `dao/src/toggle.rs` | model | CRUD | self (extend existing) | exact |
| `dao_impl_sqlite/src/toggle.rs` | dao | CRUD | self (extend existing) | exact |
| `service/src/toggle.rs` | service | CRUD | self (extend existing) | exact |
| `service_impl/src/toggle.rs` | service | CRUD | self (extend existing, read enable_toggle/disable_toggle impls) | exact |
| `rest/src/toggle.rs` | controller | request-response | `rest/src/toggle.rs` enable/disable handlers (lines 189-251) | exact |
| `rest-types/src/lib.rs` | model | transform | `rest-types/src/lib.rs` ToggleTO (lines 1529-1557) | exact |
| `service_impl/src/reporting.rs` | service | CRUD | self injection points (lines 402-406, 717-720, 1140-1161, 1269-1273) | exact |
| `service_impl/src/billing_period_report.rs` | service | CRUD | self line 101 const | exact |
| `service_impl/src/test/billing_period_snapshot_locking.rs` | test | — | self line 26-39 | exact |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | test | — | `service_impl/src/test/reporting_additive_merge.rs` | role-match |
| `service_impl/src/test/mod.rs` | config | — | self (add one line) | exact |
| `shifty_bin/src/main.rs` | config | — | self lines 878-915 (construction block) | exact |
| `shifty-dioxus/src/api.rs` | utility | request-response | `shifty-dioxus/src/api.rs` lines 1574-1595 | exact |
| `shifty-dioxus/src/loader.rs` | utility | request-response | `shifty-dioxus/src/loader.rs` lines 1008-1024 | exact |
| `shifty-dioxus/src/page/settings.rs` | component | request-response | self (lines 1-148, extend Card 1 pattern) | exact |
| `shifty-dioxus/src/i18n/{mod,en,de,cs}.rs` | utility | — | existing Settings keys in each file | exact |

---

## Pattern Assignments

### `migrations/sqlite/20260628000000_toggle-value-column.sql` (migration, transform)

**Analog:** `migrations/sqlite/20260105000000_app-toggles.sql`

**Schema pattern** (lines 1-8):
```sql
-- Individual toggles
CREATE TABLE toggle (
    name TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
```

**New migration file** — one ALTER TABLE statement:
```sql
ALTER TABLE toggle ADD COLUMN value TEXT;
```

SQLite supports `ALTER TABLE ADD COLUMN` for nullable columns without DEFAULT. No data migration needed.

---

### `migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql` (migration, transform)

**Analog:** `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` (lines 1-7):
```sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'paid_limit_hard_enforcement',
    0,
    'When ON, booking over a slot/week paid-employee limit is blocked...',
    'phase-24-migration'
);
```

**Copy pattern exactly**, changing name/description/process:
```sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'holiday_auto_credit',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), holidays on or after that date are auto-credited in reports. Leave value NULL to disable.',
    'phase-25-migration'
);
```

---

### `dao/src/toggle.rs` — ToggleEntity + new DAO methods (model, CRUD)

**Analog:** `dao/src/toggle.rs` lines 6-11 (current ToggleEntity):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleEntity {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
}
```

**Extend** by adding one field:
```rust
pub struct ToggleEntity {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub value: Option<String>,   // NEW — ISO date string or NULL
}
```

**New DAO method signatures** — copy shape of `is_enabled` (line 55) and `update_toggle` (line 41-46):
```rust
async fn get_toggle_value(
    &self,
    name: &str,
    tx: Self::Transaction,
) -> Result<Option<String>, DaoError>;

async fn set_toggle_value(
    &self,
    name: &str,
    value: Option<&str>,
    process: &str,
    tx: Self::Transaction,
) -> Result<(), DaoError>;
```

---

### `dao_impl_sqlite/src/toggle.rs` — ToggleDb + SELECT/UPDATE queries (dao, CRUD)

**Analog:** `dao_impl_sqlite/src/toggle.rs` lines 12-25 (current ToggleDb + From):
```rust
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

**Extend ToggleDb** with `value: Option<String>` and update From impl to pass `value: db.value.clone()`.

**Critical: ALL queries returning ToggleDb must include `value`** — SQLx compile-time checking enforces this. Affected queries:

`get_toggle` (lines 83-88):
```rust
query_as!(
    ToggleDb,
    r#"SELECT name, enabled, description
       FROM toggle
       WHERE name = ?"#,
    name,
)
// Change to:
r#"SELECT name, enabled, description, value FROM toggle WHERE name = ?"#
```

`get_all_toggles` (lines 101-105):
```rust
// Change to:
r#"SELECT name, enabled, description, value FROM toggle ORDER BY name"#
```

`get_toggles_in_group` (lines 294-300):
```rust
// Change to:
r#"SELECT t.name, t.enabled, t.description, t.value
   FROM toggle t
   INNER JOIN toggle_group_toggle tgt ON t.name = tgt.toggle_name
   WHERE tgt.toggle_group_name = ?
   ORDER BY t.name"#
```

`update_toggle` (lines 121-130) — add `value` to SET:
```rust
query!(
    r#"UPDATE toggle
       SET enabled = ?, description = ?, value = ?, update_process = ?
       WHERE name = ?"#,
    enabled,
    toggle.description,
    toggle.value,    // NEW
    process,
    toggle.name,
)
```

**New `get_toggle_value` impl** — copy shape of `is_enabled` (lines 159-170):
```rust
async fn get_toggle_value(&self, name: &str, tx: Self::Transaction) -> Result<Option<String>, DaoError> {
    let result = query!(
        r#"SELECT value FROM toggle WHERE name = ?"#,
        name,
    )
    .fetch_optional(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?;
    Ok(result.and_then(|row| row.value))
}
```

**New `set_toggle_value` impl** — copy shape of `update_toggle` (lines 114-134):
```rust
async fn set_toggle_value(
    &self,
    name: &str,
    value: Option<&str>,
    process: &str,
    tx: Self::Transaction,
) -> Result<(), DaoError> {
    let enabled: i64 = if value.is_some() { 1 } else { 0 };
    query!(
        r#"UPDATE toggle
           SET value = ?, enabled = ?, update_process = ?
           WHERE name = ?"#,
        value,
        enabled,
        process,
        name,
    )
    .execute(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?;
    Ok(())
}
```

---

### `service/src/toggle.rs` — Toggle struct + new trait methods (service, CRUD)

**Analog:** `service/src/toggle.rs` lines 11-36 (current Toggle + From impls):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Toggle {
    pub name: Arc<str>,
    pub enabled: bool,
    pub description: Option<Arc<str>>,
}

impl From<&dao::toggle::ToggleEntity> for Toggle {
    fn from(entity: &dao::toggle::ToggleEntity) -> Self {
        Self {
            name: entity.name.clone().into(),
            enabled: entity.enabled,
            description: entity.description.clone().map(Into::into),
        }
    }
}
```

**Extend Toggle** with `pub value: Option<Arc<str>>` and update both From impls to include `value`.

**New trait methods** — copy shape of `enable_toggle`/`disable_toggle` (lines 97-109):
```rust
async fn get_toggle_value(
    &self,
    name: &str,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Option<Arc<str>>, ServiceError>;

async fn set_toggle_value(
    &self,
    name: &str,
    value: Option<&str>,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError>;
```

`set_toggle_value` requires `toggle_admin` privilege (same as `enable_toggle`).

---

### `rest/src/toggle.rs` — three new value endpoints (controller, request-response)

**Analog:** `rest/src/toggle.rs` `enable_toggle` handler (lines 189-219):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{name}/enable",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 204, description = "Toggle enabled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
        (status = 404, description = "Toggle not found"),
    ),
)]
pub async fn enable_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .enable_toggle(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
```

**New endpoints to add** — three handlers + route registrations:

`GET /{name}/value` — returns value or 204 if unset (copy `is_toggle_enabled` shape, lines 126-158):
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{name}/value",
    tags = ["Toggles"],
    params(("name", description = "Toggle name", example = "holiday_auto_credit")),
    responses(
        (status = 200, description = "Value string", body = String),
        (status = 204, description = "Value not set"),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_toggle_value<RestState: RestStateDef>(...) -> Response { ... }
```

`PUT /{name}/value` — body = JSON string, requires toggle_admin (copy `enable_toggle` shape):
```rust
#[utoipa::path(put, path = "/{name}/value", ...)]
pub async fn set_toggle_value<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
    Json(value): Json<String>,
) -> Response { ... }
```

`DELETE /{name}/value` — clears value (copy `disable_toggle` shape, returns 204):
```rust
#[utoipa::path(delete, path = "/{name}/value", ...)]
pub async fn clear_toggle_value<RestState: RestStateDef>(...) -> Response { ... }
```

**Route registration** in `generate_route` (line 17-27):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        // existing routes...
        .route("/{name}/value", get(get_toggle_value::<RestState>))
        .route("/{name}/value", put(set_toggle_value::<RestState>))
        .route("/{name}/value", delete(clear_toggle_value::<RestState>))
}
```

Add new handlers to `ToggleApiDoc` `openapi(paths(...))` at lines 588-600.

---

### `rest-types/src/lib.rs` — ToggleTO value field (model, transform)

**Analog:** `rest-types/src/lib.rs` lines 1529-1557 (current ToggleTO):
```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ToggleTO {
    pub name: Arc<str>,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<Arc<str>>,
}

#[cfg(feature = "service-impl")]
impl From<&service::toggle::Toggle> for ToggleTO {
    fn from(toggle: &service::toggle::Toggle) -> Self {
        Self {
            name: toggle.name.clone(),
            enabled: toggle.enabled,
            description: toggle.description.clone(),
        }
    }
}
```

**Add one field** + update both From impls:
```rust
pub struct ToggleTO {
    pub name: Arc<str>,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<Arc<str>>,
    #[serde(default)]
    pub value: Option<Arc<str>>,   // NEW
}
```

---

### `service_impl/src/reporting.rs` — three injection points (service, CRUD)

**Analog:** Existing `derived_absence_hours` injection at lines 1149-1161 — **this is the model to copy** for holiday injection:

```rust
// service_impl/src/reporting.rs:1149-1161 (derived_absence_hours pattern)
let derived_absence_hours = if working_hours_for_week <= 0.0 {
    0.0f32
} else {
    derived_absence
        .iter()
        .filter(|(d, _)| ShiftyDate::from(**d).as_shifty_week() == week)
        .map(|(_, r)| r.hours)
        .sum::<f32>()
};
let absence_hours = absence_hours + derived_absence_hours;
```

**Injection point 1a — `hours_per_week` holiday_hours (lines 1269-1273):**

Current:
```rust
holiday_hours: filtered_extra_hours_list
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum(),
```

Add `derived_holiday: &HashMap<time::Date, f32>` parameter to function signature. Then after existing sum:
```rust
let manual_holiday: f32 = filtered_extra_hours_list
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum();
let derived_for_week: f32 = derived_holiday
    .iter()
    .filter(|(date, _)| ShiftyDate::from(**date).as_shifty_week() == week)
    .map(|(_, h)| h)
    .sum();
// holiday_hours field:
holiday_hours: manual_holiday + derived_for_week,
```

**Critical:** also add `derived_for_week` to `absence_hours` (same line as the `derived_absence_hours` merge at line 1161 pattern) to correctly reduce `expected_hours`/`balance`.

**Injection point 1b — `get_reports_for_all_employees` holiday_hours (lines 402-406):**

Current:
```rust
let holiday_hours = week_extra_hours
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum::<f32>();
```

After injection:
```rust
let manual_holiday_hours = week_extra_hours
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum::<f32>();
let derived_holiday_for_week: f32 = /* sum derived_holiday_map for this (year, week) */;
let holiday_hours = manual_holiday_hours + derived_holiday_for_week;
// Also add derived_holiday_for_week to absense_hours (lines 387-391 sum).
```

**Injection point 1c — `EmployeeReport.holiday_hours` (lines 717-720):**

Current (direct extra_hours filter):
```rust
holiday_hours: extra_hours
    .iter()
    .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::Holiday)
    .map(|extra_hours| extra_hours.amount)
    .sum(),
```

**Switch to `by_week` source** (Option A from RESEARCH — aligns with vacation_hours at line 715):
```rust
// UV-05 pattern: by_week is SINGLE SOURCE OF TRUTH
vacation_hours: by_week.iter().map(|w| w.vacation_hours).sum::<f32>(),  // already like this
sick_leave_hours: by_week.iter().map(|w| w.sick_leave_hours).sum::<f32>(),  // already like this
holiday_hours: by_week.iter().map(|w| w.holiday_hours).sum::<f32>(),  // CHANGE: was direct extra_hours
```

**New deps in `gen_service_impl!` block (lines 59-75):**
```rust
gen_service_impl! {
    struct ReportingServiceImpl: ReportingService = ReportingServiceDeps {
        // ... existing 10 deps ...
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,  // NEW
        ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,  // NEW
    }
}
```

**Pre-computation of derived holiday map** — async context available in `get_report_for_employee_range` and `get_reports_for_all_employees`. For each week in range:
1. Call `toggle_service.get_toggle("holiday_auto_credit", ...)` → parse `value` as `time::Date` cutoff
2. Call `special_day_service.get_by_week(year, week, context, None)` → filter by `day_type == Holiday`
3. For each holiday: compute date via `time::Date::from_iso_week_date(year, week, day_of_week.into())`
4. Gate: `holiday_date >= cutoff_date`
5. Conflict check: `extra_hours.iter().any(|eh| eh.category == Holiday && eh.date_time.to_date() == holiday_date)`
6. Amount: `find_working_hours_for_calendar_week(working_hours, year, week).next()` → `wh.has_day_of_week(dow)` → `wh.holiday_hours()`
7. Insert into `HashMap<time::Date, f32>` if no conflict

---

### `service_impl/src/billing_period_report.rs` — version bump (service, CRUD)

**Analog:** `service_impl/src/billing_period_report.rs` line 101:
```rust
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 10;
```

**Change to:**
```rust
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;
```

Add doc comment above explaining the Phase 25 reason (holiday derive-on-read changes the computation of `BillingPeriodValueType::Holiday`).

---

### `service_impl/src/test/billing_period_snapshot_locking.rs` — locking test update (test)

**Analog:** `service_impl/src/test/billing_period_snapshot_locking.rs` lines 26-39:
```rust
#[test]
fn test_snapshot_schema_version_pinned() {
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 10,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 10 sein nach UV-05 / D-18-07: ..."
    );
}
```

**Change assert value and update doc string:**
```rust
assert_eq!(
    CURRENT_SNAPSHOT_SCHEMA_VERSION, 11,
    "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 11 sein nach Phase 25: \
     holiday derive-on-read (SpecialDay + Vertrag) aendert die Computation von \
     BillingPeriodValueType::Holiday fuer Perioden ab Stichtag. \
     Laut CLAUDE.md (Snapshot Schema Versioning: 'Change the computation that produces \
     an existing value_type') ist ein Bump Pflicht. \
     Siehe service_impl/src/billing_period_report.rs § CURRENT_SNAPSHOT_SCHEMA_VERSION."
);
```

Also update the module doc comment at line 7 (currently says "erwartet 10 (UV-05 / D-18-07 ...)").

---

### `service_impl/src/test/reporting_holiday_auto_credit.rs` (new test file)

**Analog:** `service_impl/src/test/reporting_additive_merge.rs` — complete structure to copy.

**Imports pattern** (reporting_additive_merge.rs lines 20-43):
```rust
use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::{AbsenceCategory, MockAbsenceService, ResolvedAbsence};
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::ShiftyDate;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
};
```

**Additional imports for Phase 25:**
```rust
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
```

**ReportingMocks struct** — copy from reporting_additive_merge.rs lines 97-155, add two new fields:
```rust
struct ReportingMocks {
    // ... all existing fields ...
    special_day_service: MockSpecialDayService,
    toggle_service: MockToggleService,
}

struct TestDeps;
impl ReportingServiceDeps for TestDeps {
    // ... all existing assoc types ...
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}
```

**Test shapes** (see RESEARCH.md sections 8 for HOL-01/02/03, HCFG-01, HCFG-03 test patterns):

```rust
#[tokio::test]
async fn test_holiday_auto_credit_basic() {
    // HOL-01: Monday holiday, 40h/week Mon-Fri contract
    // toggle value = "2024-01-01" (before holiday date 2024-03-18)
    // Expected: report.holiday_hours == 8.0
}

#[tokio::test]
async fn test_holiday_auto_credit_equivalence() {
    // HOL-02: compare derived vs manual ExtraHours(Holiday, 8.0)
    // Same expected_hours and balance in both cases
}

#[tokio::test]
async fn test_holiday_before_cutoff_skipped() {
    // HCFG-01: holiday 2024-03-18, cutoff "2024-03-25" → holiday_hours == 0.0
    //          holiday 2024-03-18, cutoff "2024-03-18" → holiday_hours == 8.0 (>= boundary)
}

#[tokio::test]
async fn test_holiday_manual_wins() {
    // HCFG-03: manual ExtraHours(Holiday) on same day → no double-count
    // report.holiday_hours == 8.0, not 16.0
}

#[tokio::test]
async fn test_holiday_auto_credit_no_year_view_impact() {
    // HOL-03: booking_information_service not in ReportingServiceImpl — 
    // guard: SpecialDayService.get_by_week not called from BookingInformationService
    // (structural test: BookingInformationServiceImpl deps do not include SpecialDayService)
}
```

---

### `service_impl/src/test/mod.rs` — new module registration (config)

**Analog:** any existing two-line block in `service_impl/src/test/mod.rs` (e.g., lines 26-27):
```rust
#[cfg(test)]
pub mod reporting_additive_merge;
```

**Add after `reporting_no_contract_volunteer`:**
```rust
#[cfg(test)]
pub mod reporting_holiday_auto_credit;
```

---

### `shifty_bin/src/main.rs` — DI construction order fix (config)

**Analog:** `shifty_bin/src/main.rs` lines 878-915 (reporting + toggle construction blocks):

Current order:
- line 878: `reporting_service` constructed (10 deps, no toggle)
- line 910: `toggle_service` constructed

**Required change — move `toggle_service` block (lines 910-915) to BEFORE line 878**, then add two new fields to `ReportingServiceImpl` struct literal:

```rust
// BEFORE reporting_service:
let toggle_dao = Arc::new(ToggleDao::new(pool.clone()));
let toggle_service = Arc::new(service_impl::toggle::ToggleServiceImpl {
    toggle_dao,
    permission_service: permission_service.clone(),
    transaction_dao: transaction_dao.clone(),
});

// THEN reporting_service with 2 new deps:
let reporting_service = Arc::new(service_impl::reporting::ReportingServiceImpl {
    // ... existing 10 fields ...
    special_day_service: special_day_service.clone(),  // NEW (already constructed at ~line 753)
    toggle_service: toggle_service.clone(),             // NEW (just constructed above)
});
```

`special_day_service` is already constructed around line 753 (before reporting at 878) — no reorder needed for it.

---

### `shifty-dioxus/src/api.rs` — new toggle value functions (utility, request-response)

**Analog:** `shifty-dioxus/src/api.rs` lines 1574-1595:
```rust
// ─── Toggle REST client (Phase 24 D-24-06) ───────────────────────────────────

pub async fn set_toggle(
    config: Config,
    name: &str,
    enabled: bool,
) -> Result<(), reqwest::Error> {
    let verb = if enabled { "enable" } else { "disable" };
    let url = format!("{}/toggle/{}/{}", config.backend, name, verb);
    let client = reqwest::Client::new();
    client.put(url).send().await?.error_for_status()?;
    Ok(())
}

pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, reqwest::Error> {
    let url = format!("{}/toggle/{}/enabled", config.backend, name);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}
```

**Add after line 1595** — copy URL format and error pattern:
```rust
/// GET /toggle/{name}/value → Option<String>
pub async fn get_toggle_value(config: Config, name: &str) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let response = reqwest::get(url).await?;
    if response.status() == 204 {
        return Ok(None);
    }
    response.error_for_status_ref()?;
    Ok(response.json::<Option<String>>().await?)
}

/// PUT /toggle/{name}/value
pub async fn set_toggle_value(config: Config, name: &str, value: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client.put(url).json(value).send().await?.error_for_status()?;
    Ok(())
}

/// DELETE /toggle/{name}/value
pub async fn clear_toggle_value(config: Config, name: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client.delete(url).send().await?.error_for_status()?;
    Ok(())
}
```

---

### `shifty-dioxus/src/loader.rs` — cutoff date loaders (utility, request-response)

**Analog:** `shifty-dioxus/src/loader.rs` lines 1008-1024:
```rust
// ─── Toggle loaders (Phase 24 D-24-06) ──────────────────────────────────────

pub async fn set_toggle(config: Config, name: &str, enabled: bool) -> Result<(), ShiftyError> {
    api::set_toggle(config, name, enabled).await?;
    Ok(())
}

pub async fn get_toggle_enabled(config: Config, name: &str) -> Result<bool, ShiftyError> {
    let enabled = api::get_toggle_enabled(config, name).await?;
    Ok(enabled)
}
```

**Add after line 1024:**
```rust
// ─── Holiday auto-credit cutoff (Phase 25) ───────────────────────────────────

pub async fn get_holiday_cutoff_date(config: Config) -> Result<Option<String>, ShiftyError> {
    Ok(api::get_toggle_value(config, "holiday_auto_credit").await?)
}

pub async fn set_holiday_cutoff_date(config: Config, value: Option<&str>) -> Result<(), ShiftyError> {
    match value {
        Some(v) => api::set_toggle_value(config, "holiday_auto_credit", v).await?,
        None => api::clear_toggle_value(config, "holiday_auto_credit").await?,
    }
    Ok(())
}
```

---

### `shifty-dioxus/src/page/settings.rs` — Card 2 date-input (component, request-response)

**Analog:** `shifty-dioxus/src/page/settings.rs` lines 1-148 (complete Phase 24 card pattern):

**Admin guard pattern** (lines 23-35) — reuse verbatim:
```rust
let is_admin = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("admin"))
    .unwrap_or(false);
if !is_admin {
    return rsx! {
        TopBar {}
        div { class: "p-md text-ink-muted", "Not authorized." }
    };
}
```

**Resource + signal pattern** (lines 38-56) — copy for date input:
```rust
// Card 2 signals
let mut date_str: Signal<String> = use_signal(|| String::new());
let mut date_str_loaded_empty = use_signal(|| false);
let mut cutoff_save_result: Signal<Option<bool>> = use_signal(|| None);
let mut cutoff_saving = use_signal(|| false);

let config_for_cutoff = config.clone();
let cutoff_resource = use_resource(move || loader::get_holiday_cutoff_date(config_for_cutoff.clone()));
use_effect(move || {
    match &*cutoff_resource.read_unchecked() {
        Some(Ok(Some(date))) => { date_str.set(date.clone()); date_str_loaded_empty.set(false); }
        Some(Ok(None)) => { date_str.set(String::new()); date_str_loaded_empty.set(true); }
        _ => {}
    }
});
```

**WASM caveat (from MEMORY):** Save button MUST be enabled whenever `date_str` is non-empty — do NOT gate on "changed from loaded state". Programmatic date input changes do not reliably trigger Dioxus signals.

**On-save handler** (copy `on_toggle` pattern lines 59-82 for async spawn):
```rust
let on_save_cutoff = move |_| {
    if *cutoff_saving.read() { return; }
    let val = date_str.read().clone();
    if val.is_empty() { return; }
    cutoff_saving.set(true);
    cutoff_save_result.set(None);
    let cfg = config_for_save.clone();
    spawn(async move {
        match loader::set_holiday_cutoff_date(cfg, Some(&val)).await {
            Ok(()) => { cutoff_save_result.set(Some(true)); }
            Err(_) => { cutoff_save_result.set(Some(false)); }
        }
        cutoff_saving.set(false);
    });
};
```

**On-clear handler** — calls `set_holiday_cutoff_date(cfg, None)`.

**Card 2 RSX layout** — add second `div { class: "bg-surface border border-border rounded-md p-4 ..." }` block after the existing Card 1 block (line 109). Inside:
- `span` for `Key::SettingsHolidayAutoCreditLabel`
- `span` for `Key::SettingsHolidayAutoCreditDescription`
- `input { r#type: "date", value: "{date_str}", oninput: update_date_str, ... }`
- Save button calling `on_save_cutoff`
- Clear button calling `on_clear_cutoff`
- Unset hint when `date_str_loaded_empty`
- Inline feedback `save_result` (same `Some(true)/Some(false)/None` pattern as Card 1 lines 131-143)

---

### `shifty-dioxus/src/i18n/{mod,en,de,cs}.rs` — 5 new Keys (utility)

**Analog:** existing Settings keys in each file. Exact strings from RESEARCH.md section 7 (verbatim from 25-UI-SPEC.md):

**mod.rs** — add 5 new variants after existing Settings keys (~line 596):
```rust
SettingsHolidayAutoCreditLabel,
SettingsHolidayAutoCreditDescription,
SettingsHolidayAutoCreditSave,
SettingsHolidayAutoCreditClear,
SettingsHolidayAutoCreditUnsetHint,
```

**en.rs** — add after last Settings key (~line 952):
```rust
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditLabel, "Holiday auto-credit activation date");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditDescription, "Holidays on or after this date are credited automatically. Leave empty to disable.");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditSave, "Save date");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditClear, "Clear (disable)");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditUnsetHint, "Not set — automation is off.");
```

**de.rs** — add after last Settings key (~line 1035):
```rust
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditLabel, "Feiertags-Automatik aktiv ab");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditDescription, "Feiertage ab diesem Datum werden automatisch angerechnet. Leer lassen = Automatik deaktiviert.");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditSave, "Datum speichern");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditClear, "Löschen (deaktivieren)");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditUnsetHint, "Nicht gesetzt — Automatik inaktiv.");
```

**cs.rs** — add after last Settings key (~line 1021):
```rust
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditLabel, "Automatické připisování svátků od");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditDescription, "Svátky od tohoto data jsou automaticky připisovány. Prázdné = automatika vypnuta.");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditSave, "Uložit datum");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditClear, "Smazat (deaktivovat)");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditUnsetHint, "Nenastaveno — automatika je vypnuta.");
```

All three locales must be updated or the i18n completeness test (mod.rs ~line 1295) fails.

---

## Shared Patterns

### Transaction Pattern
**Source:** Any service_impl file (e.g., `service_impl/src/toggle.rs`)
**Apply to:** All new service methods in `ToggleServiceImpl`
```rust
async fn set_toggle_value(&self, name: &str, value: Option<&str>,
    context: Authentication<Self::Context>, tx: Option<Self::Transaction>
) -> Result<(), ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // privilege check
    self.permission_service.check_permission(TOGGLE_ADMIN_PRIVILEGE, context, tx.clone()).await?;
    // DAO call
    self.toggle_dao.set_toggle_value(name, value, "api", tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```

### OpenAPI Annotation Pattern
**Source:** `rest/src/toggle.rs` lines 51-82 (`get_all_toggles`)
**Apply to:** All three new REST endpoints (get/put/delete toggle value)
```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    METHOD,
    path = "/{name}/value",
    tags = ["Toggles"],
    params(("name", description = "Toggle name", example = "holiday_auto_credit")),
    responses(
        (status = ..., description = "...", body = TYPE),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
```

### Week-to-Date Conversion
**Source:** `shifty-utils/src/date_utils.rs` ShiftyDate::new pattern (referenced in RESEARCH.md)
**Apply to:** `service_impl/src/reporting.rs` holiday date derivation
```rust
let holiday_date: time::Date = time::Date::from_iso_week_date(
    special_day.year as i32,
    special_day.calendar_week,
    time::Weekday::from(special_day.day_of_week),
).expect("valid ISO week date from SpecialDayEntity");
```

### ISO Date Parse for Cutoff Gate
**Source:** RESEARCH.md Code Examples section
**Apply to:** `service_impl/src/reporting.rs` cutoff comparison
```rust
let cutoff_date: Option<time::Date> = toggle_value_str
    .as_deref()
    .and_then(|s| time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok());

let should_derive = match cutoff_date {
    None => false,
    Some(cutoff) => holiday_date >= cutoff,
};
```

### Error Handling Pattern
**Source:** `rest/src/toggle.rs` lines 64-81 (`error_handler` wrapper)
**Apply to:** All three new REST endpoints
```rust
error_handler(
    (async {
        // ... service call ...
        Ok(Response::builder().status(204).body(Body::empty()).unwrap())
    })
    .await,
)
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| (none) | — | — | All files have strong analogs in the existing codebase |

---

## Key Pitfalls (from RESEARCH.md — executor must read)

1. **Three injection points, not one** — `hours_per_week` (1269), `get_reports_for_all_employees` (402), and `EmployeeReport.holiday_hours` (717) all need updating. Missing point 1c means the snapshot gets wrong data.
2. **absense_hours must also get the derived hours** — holiday is AbsenceHours-typed; only updating `holiday_hours` without `absense_hours` breaks `expected_hours`/`balance`.
3. **Locking test update is mandatory** — after bumping const to 11, the assert at `billing_period_snapshot_locking.rs:29` fails until updated.
4. **All SELECT queries returning ToggleDb must include `value`** — SQLx compile-time check enforces this.
5. **toggle_service construction must move before reporting_service in main.rs** — currently it's 33 lines after.
6. **WASM date input caveat** — save button enabled on non-empty `date_str`, NOT on "changed from loaded state".

---

## Metadata

**Analog search scope:** `dao/`, `dao_impl_sqlite/`, `service/`, `service_impl/`, `rest/`, `rest-types/`, `shifty_bin/`, `migrations/sqlite/`, `shifty-dioxus/src/`
**Files scanned:** 18 source files read
**Pattern extraction date:** 2026-06-28
