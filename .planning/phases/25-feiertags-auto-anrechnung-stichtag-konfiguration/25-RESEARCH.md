# Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration - Research

**Researched:** 2026-06-28
**Domain:** Rust backend service layer (reporting injection, toggle infra extension) + Dioxus frontend settings page
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-25-01 (Derive-on-read, KEINE materialisierten Rows):** Feiertagsstunden werden beim
  Report-Erstellen direkt aus `special_day` + Vertrag berechnet und in die
  `holiday_hours`-Aggregation injiziert (reporting.rs:402-406). Keine echten ExtraHours-DB-Zeilen.
- **D-25-02 (Bestehenden Helper wiederverwenden):** Anrechnungsbetrag = `EmployeeWorkDetails::holiday_hours()`
  (= expected_hours / potential_days_per_week()), nur wenn `has_day_of_week(holiday.day_of_week)` true.
- **D-25-03 (Manuell gewinnt):** Existiert manueller ExtraHours(Holiday) für denselben
  Mitarbeiter+Tag → Automatik überspringen (keine Doppelzählung).
- **D-25-04 (Toggle-Tabelle um `value`-Spalte erweitern):** Stichtag als ISO-Datum-String im `value`.
  Reuse Toggle-Infrastruktur aus Phase 24.
- **D-25-05 (Toggle-Semantik & Default = aus):** `value` (ISO-Datum) ist der autoritative Gate.
  Default (kein value) = Automatik aus. Kein Regression auf Bestandsdaten.
- **D-25-07 (Bump 10 → 11):** `CURRENT_SNAPSHOT_SCHEMA_VERSION` in
  `service_impl/src/billing_period_report.rs:101`.
- **D-25-08 (Year-View unangetastet):** Keine Änderung an `dynamic_hours`/`paid_hours`/
  `committed_voluntary_hours`/`volunteer_hours` in `service_impl/src/booking_information.rs`.

### Claude's Discretion
- Exakter Toggle-Key-Name (Empfehlung: `holiday_auto_credit`)
- Ob `enabled` als separater Master-Schalter dient oder aus `value`-Präsenz abgeleitet wird
- Ableitung des absoluten Feiertags-Datums aus `(year, calendar_week, day_of_week)` für Stichtag-Vergleich
- Genaue Detektion „manueller Holiday-Eintrag deckt diesen Feiertag ab"
- Alle i18n-Labels/Texte (de/en/cs) — festgelegt in 25-UI-SPEC.md

### Deferred Ideas (OUT OF SCOPE)
- ShortDay/Kurztage automatisch anrechnen
- Volle Urlaubsverwaltung/-Balance für Freiwillige
- Ob Stichtag auch VFA-01 gated (Phase-26-Entscheidung)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HOL-01 | Pro Holiday aus special_day für jeden Mitarbeiter, der am Wochentag laut Vertrag arbeitet, Feiertagsstunden automatisch anrechnen | Injection in hours_per_week + get_reports_for_all_employees + EmployeeReport.holiday_hours; pre-load special_days |
| HOL-02 | Wirkung identisch zu manuellem ExtraHours(Holiday): expected↓, balance↑, holiday_hours-Spalte | Injection in exakt dieselbe holiday_hours-Aggregation; Vergleichstest |
| HOL-03 | Jahresansicht (paid_hours, committed_voluntary_hours, volunteer_hours) unverändert | booking_information.rs wird nicht angefasst; Regressionstest |
| HCFG-01 | Stichtag-Datum steuert ab wann Automatik greift; davor kein Auto-Eintrag | `holiday_date >= cutoff_date` Filter vor Injection |
| HCFG-02 | Admin-gated Settings-UI für Stichtag, persistiert, i18n de/en/cs | Toggle-Tabelle value-Spalte; neue GET/PUT REST-Endpoints; Dioxus date-input-Card |
| HCFG-03 | Keine Doppelzählung: manuell ODER automatisch, nicht beides | Conflict-check: manueller ExtraHours(Holiday) für denselben Tag → skip |
| HSNAP-01 | CURRENT_SNAPSHOT_SCHEMA_VERSION bump 10 → 11 + Locking-Test update | billing_period_report.rs:101 + billing_period_snapshot_locking.rs:29 |
</phase_requirements>

---

## Summary

Phase 25 implementiert die automatische Feiertags-Anrechnung als "derive-on-read" — keine neuen
DB-Zeilen, stattdessen werden Feiertagsstunden direkt bei der Report-Berechnung aus `special_day`
plus dem am Tag gültigen Vertrag abgeleitet und in die bestehende `holiday_hours`-Aggregation
injiziert. Ein konfigurierbarer Stichtag (ISO-Datum in der erweiterten `toggle`-Tabelle) gated die
Automatik; Feiertage vor dem Stichtag bleiben unberührt.

Die Implementierung berührt drei Schichten: (1) Backend-Service `reporting.rs` — drei separate
holiday_hours-Berechnungsstellen müssen um derive-on-read-Logik erweitert werden; (2) Toggle-Infra
— Migration + Entity + DAO + Service + REST müssen um `value TEXT`-Spalte erweitert werden;
(3) Dioxus-Frontend — neue Date-Input-Card in `settings.rs` plus zwei neue API-Funktionen.

**Primary recommendation:** Derived holiday hours als `HashMap<time::Date, f32>` pre-computieren
(vor den sync-only-Schleifen) und als Zusatzparameter in `hours_per_week` übergeben. Dieselbe
Vorberechnung im `get_reports_for_all_employees`-Fold inline verarbeiten.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Holiday derive-on-read | API / Backend (ReportingService) | — | Reporting ist Business-Logic-Tier; darf SpecialDayService + ToggleService konsumieren |
| Stichtag-Speicherung | Database / Storage (Toggle-Tabelle) | — | Bestehende toggle-Infra erweitert |
| Stichtag-API | API / Backend (REST toggle.rs) | — | Neue GET/PUT/DELETE Endpoints auf /toggle/{name}/value |
| Settings-UI Date-Input | Frontend Server / Browser (Dioxus WASM) | — | Admin-gated settings.rs, bestehende Seite erweitert |
| Snapshot-Versionsbump | API / Backend (billing_period_report.rs) | — | Pflicht-Bump wegen Computation-Änderung |

---

## Standard Stack

### Core (alle bereits im Projekt, keine neuen Abhängigkeiten)

| Library/Crate | Version | Purpose | Status |
|---------------|---------|---------|--------|
| `time` crate | (project version) | `time::Date::from_iso_week_date` für week→date-Konvertierung | [VERIFIED: codebase] |
| `sqlx` | (project version) | SQLite-Migration + compile-time query checking | [VERIFIED: codebase] |
| `mockall` | (project version) | Mock-basierte Unit-Tests | [VERIFIED: codebase] |
| `reqwest` | (project version) | Frontend HTTP-Client für neue Toggle-value-Endpoints | [VERIFIED: codebase] |
| `dioxus` | 0.6.3 | Frontend WASM | [VERIFIED: codebase] |

**Keine neuen externen Abhängigkeiten.** Alle benötigten Typen und Helper existieren bereits.

### Package Legitimacy Audit

> Kein neues Paket wird installiert. Diese Phase verwendet ausschließlich bereits im Projekt
> vorhandene Crates. Das Audit entfällt.

---

## Architecture Patterns

### System Architecture Diagram

```
Admin-Browser
    │ PUT /toggle/holiday_auto_credit/value  (ISO-Datum)
    ▼
REST Layer (rest/src/toggle.rs)
    │ new endpoints: GET/{name}/value, PUT/{name}/value, DELETE/{name}/value
    ▼
ToggleService (service_impl/src/toggle.rs)
    │ new methods: get_toggle_value, set_toggle_value
    ▼
ToggleDao (dao_impl_sqlite/src/toggle.rs)
    │ extended: value column in SELECT/UPDATE queries
    ▼
SQLite toggle table (+ value TEXT column via migration)

                        ┌────────────────────────────────┐
Report-Request          │ ReportingServiceImpl            │
GET /report/...    ───► │  (service_impl/src/reporting.rs)│
                        │                                  │
                        │  1. toggle_service.get_toggle_value("holiday_auto_credit")
                        │     → Option<time::Date> cutoff
                        │                                  │
                        │  2. special_day_service.get_by_week(year, wk)
                        │     for each week in range      │
                        │     → collect Holiday entries   │
                        │                                  │
                        │  3. Per holiday: cutoff-check + conflict-check
                        │     → derived_holiday_map: HashMap<time::Date, f32>
                        │                                  │
                        │  4. hours_per_week(... derived_holiday_map)
                        │     → GroupedReportHours.holiday_hours += derived
                        │                                  │
                        │  5. EmployeeReport.holiday_hours += derived_total
                        └────────────────────────────────┘
```

### Recommended Project Structure (Changes Only)

```
migrations/sqlite/
└── 20260628000000_toggle-value-column.sql    # ALTER TABLE toggle ADD COLUMN value TEXT
    20260628000001_seed-holiday-auto-credit-toggle.sql  # INSERT OR IGNORE

dao/src/toggle.rs               # ToggleEntity += value: Option<String>; new DAO method
service/src/toggle.rs           # Toggle += value: Option<Arc<str>>; new service methods
dao_impl_sqlite/src/toggle.rs   # ToggleDb += value; updated SELECT/UPDATE queries
service_impl/src/toggle.rs      # impl get_toggle_value, set_toggle_value
rest/src/toggle.rs              # new endpoints + OpenAPI annotations
rest-types/src/lib.rs           # ToggleTO += value: Option<Arc<str>>

service_impl/src/reporting.rs   # 3 injection points + SpecialDayService/ToggleService deps
shifty_bin/src/main.rs          # ReportingServiceDependencies += 2 types; constructor update

service_impl/src/test/
└── reporting_holiday_auto_credit.rs    # HOL-01/02/03 tests (new file)
    billing_period_snapshot_locking.rs  # update pinned version: 10 → 11

shifty-dioxus/src/
├── i18n/mod.rs             # 5 new Key variants
├── i18n/en.rs              # 5 new English strings
├── i18n/de.rs              # 5 new German strings
├── i18n/cs.rs              # 5 new Czech strings
├── api.rs                  # get_toggle_value, set_toggle_value
├── loader.rs               # get_holiday_cutoff_date, set_holiday_cutoff_date
└── page/settings.rs        # Card 2: holiday auto-credit date-input
```

---

## Detailed Findings by Research Focus

### 1. Exact Reporting Injection Points

There are **three** separate places where `holiday_hours` is computed. All three need derive-on-read injection:

#### 1a. `hours_per_week` free function — lines 1269-1273

```rust
// service_impl/src/reporting.rs:1269-1273 (current)
holiday_hours: filtered_extra_hours_list
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum(),
```

This function builds `GroupedReportHours` per week. Its result is consumed by `get_report_for_employee_range` (line 627: `let by_week = hours_per_week(...)`). The function is **sync** — no async I/O possible inside.

**Injection strategy:** Add a `derived_holiday_hours: &HashMap<time::Date, f32>` parameter. Inside, sum values for dates falling in the current week and add to `holiday_hours`:

```rust
// After the existing ExtraHours sum:
let derived_for_week: f32 = derived_holiday_hours
    .iter()
    .filter(|(date, _)| ShiftyDate::from(**date).as_shifty_week() == week)
    .map(|(_, h)| h)
    .sum();
holiday_hours: <existing_sum> + derived_for_week,
```

Note: this is exactly the same pattern used for `derived_absence_hours` (lines 1149-1161). [VERIFIED: codebase]

#### 1b. `get_reports_for_all_employees` — lines 402-406

```rust
// service_impl/src/reporting.rs:402-406 (current)
let holiday_hours = week_extra_hours
    .iter()
    .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
    .map(|eh| eh.amount)
    .sum::<f32>();
```

This is in the per-week inline fold starting at line 245. Here, `year` and `week` are in scope.
Inject: add derived holiday hours for this `(year, week)` from a pre-computed map.

#### 1c. `get_report_for_employee_range` — lines 717-720

```rust
// service_impl/src/reporting.rs:717-720 (current)
holiday_hours: extra_hours
    .iter()
    .filter(|extra_hours| extra_hours.category == ExtraHoursCategory::Holiday)
    .map(|extra_hours| extra_hours.amount)
    .sum(),
```

This final `EmployeeReport.holiday_hours` is computed DIRECTLY from `extra_hours` — NOT from `by_week`. It therefore misses derived holiday hours if only `hours_per_week` is updated.

**Fix options:**
- Option A (recommended): Switch to `by_week.iter().map(|w| w.holiday_hours).sum()` — aligns with how `vacation_hours`/`sick_leave_hours` are computed (lines 715-716). This eliminates the direct `extra_hours` filter entirely for holiday_hours.
- Option B: Add `+ derived_holiday_total` to the existing sum.

Option A is cleaner and keeps a single source of truth. [ASSUMED — either option is valid; planner chooses]

**Does this path feed the snapshot?** Yes — `BillingPeriodReportService` calls `reporting_service.get_report_for_employee_range(...)` at lines 130-168. The returned `EmployeeReport.holiday_hours` flows into `BillingPeriodValueType::Holiday` at billing_period_report.rs:241-248. The bump 10→11 is therefore necessary and sufficient. [VERIFIED: codebase]

**Absence of `absense_hours` double-count risk:** The existing code already has `absense_hours` (line 387-391) which reduces `expected_hours`. Holiday hours are absence hours (ExtraHoursCategory::Holiday.as_report_type() == ReportType::AbsenceHours). The derived holiday hours must flow through the same `absense_hours` path to correctly reduce `expected_hours`/`balance`. In the `get_reports_for_all_employees` fold, `absense_hours` (line 387) sums ALL AbsenceHours categories including Holiday — so adding holiday to `holiday_hours` alone will NOT automatically update `absense_hours`. The derived holiday amount must also be added to `absense_hours` (or the equivalent total absence sum). This is the critical balance-correctness point. [VERIFIED: codebase analysis]

Confirmed: `ExtraHoursCategory::Holiday.as_report_type()` returns `ReportType::AbsenceHours` — verify this in `service/src/extra_hours.rs`.

#### Expected_hours / Balance path verification

In `get_reports_for_all_employees` (line 495):
```rust
let expected_hours = weekly_hours.planned_hours - weekly_hours.absense_hours - weekly_hours.absence_derived_balance_hours;
```
And in `hours_per_week` (line 1243):
```rust
expected_hours: expected_hours - absence_hours,
```

The `absence_hours` variable in `hours_per_week` includes all AbsenceHours-typed extra_hours. Derived holiday hours must be included in this sum for correct balance reduction. [VERIFIED: codebase]

---

### 2. SpecialDayService Access in ReportingService

#### Current situation: NOT injected

The `gen_service_impl!` macro at `service_impl/src/reporting.rs:59-75` defines 10 deps. SpecialDayService is **not** among them. [VERIFIED: codebase]

#### Adding SpecialDayService as dep

Add to `gen_service_impl!`:
```rust
SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
```

Note: `SpecialDayService` trait (service/src/special_days.rs:82) has only `type Context`, **no Transaction type**. The `get_by_week` method takes no `tx` parameter — reads directly from pool. This is acceptable for reads. [VERIFIED: codebase]

#### Adding ToggleService as dep

Add to `gen_service_impl!`:
```rust
ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
```

**Service-tier check:** ReportingService is Business-Logic tier; ToggleService is Basic-tier. No cycle. [VERIFIED: CLAUDE.md conventions]

#### main.rs construction order issue

Currently `toggle_service` is constructed AFTER `reporting_service` (toggle: line 911, reporting: line 878). The toggle service has no deps on reporting — it can be moved to before line 878 without issue. `special_day_service` is already constructed at line 753, before `reporting_service`. [VERIFIED: codebase]

#### week→date conversion for the cutoff comparison

`SpecialDayEntity` fields: `year: u32, calendar_week: u8, day_of_week: DayOfWeek`.

To get a `time::Date`:
```rust
// Already used in codebase (ShiftyDate::new implementation)
time::Date::from_iso_week_date(year as i32, calendar_week, day_of_week.into())
    .expect("valid iso week date from DB")
```

Alternatively use `ShiftyDate::new(year, calendar_week, day_of_week)?.to_date()`.

**Year-boundary pitfall:** ISO week 53 of year Y may have dates in Y+1; ISO week 1 of year Y may have dates in Y-1. The `from_iso_week_date` call is authoritative — do NOT use calendar year + week + weekday arithmetic manually. The `time` crate handles this correctly. [VERIFIED: codebase — ShiftyDate::new uses from_iso_week_date]

**Cutoff comparison:** Parse the stored ISO string (`value`) to `time::Date` via `time::Date::parse` with format `[year]-[month]-[day]`. Compare `holiday_date >= cutoff_date`.

#### Pre-loading strategy

`SpecialDayService::get_by_week` fetches one week at a time. For a full-year report (up to 53 weeks), calling it in a loop is 53 async calls. Two options:

- Option A (simpler): call `get_by_week` for each week inside the existing per-week iteration (async context available in `get_reports_for_all_employees` and `get_report_for_employee_range`).
- Option B (efficient): Add `find_by_year` to SpecialDayDao/Service (single query `WHERE year = ?`). Then pre-load once, filter per-week in memory.

**Recommendation:** Option B for large date ranges (billing period reports spanning full year). Option A acceptable for the MVP. The planner decides. [ASSUMED]

---

### 3. Toggle `value` Column — Complete Change List

#### Migration (new file)

```sql
-- migrations/sqlite/20260628000000_toggle-value-column.sql
ALTER TABLE toggle ADD COLUMN value TEXT;
```

SQLite supports `ALTER TABLE ADD COLUMN` for nullable columns without DEFAULT. [ASSUMED — standard SQLite behavior]

Seeding migration (separate file):
```sql
-- migrations/sqlite/20260628000001_seed-holiday-auto-credit-toggle.sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'holiday_auto_credit',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), holidays on or after that date are auto-credited in reports. Leave value NULL to disable.',
    'phase-25-migration'
);
```

Pattern: exactly mirrors `20260627000000_seed-paid-limit-toggle.sql`. [VERIFIED: codebase]

#### dao/src/toggle.rs — ToggleEntity

```rust
pub struct ToggleEntity {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub value: Option<String>,  // NEW: ISO date string or NULL
}
```

New DAO method:
```rust
async fn get_toggle_value(&self, name: &str, tx: Self::Transaction) -> Result<Option<String>, DaoError>;
async fn set_toggle_value(&self, name: &str, value: Option<&str>, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
```

[ASSUMED — method names; shape follows existing `is_enabled` pattern]

#### dao_impl_sqlite/src/toggle.rs — ToggleDb

```rust
struct ToggleDb {
    name: String,
    enabled: i64,
    description: Option<String>,
    value: Option<String>,   // NEW
}
```

**All existing SELECT queries must include `value`** — SQLx compile-time checking requires the struct fields to match exactly:

```sql
-- get_toggle
SELECT name, enabled, description, value FROM toggle WHERE name = ?

-- get_all_toggles
SELECT name, enabled, description, value FROM toggle ORDER BY name

-- is_enabled: unchanged (only selects enabled column)
SELECT enabled FROM toggle WHERE name = ?

-- get_toggles_in_group: updated join query
SELECT t.name, t.enabled, t.description, t.value FROM toggle t ...

-- update_toggle: add value to SET clause
UPDATE toggle SET enabled = ?, description = ?, value = ?, update_process = ? WHERE name = ?
```

**Critical:** After migration, `sqlx prepare` must be re-run (or `SQLX_OFFLINE=false` for the build). The NixOS nix-shell provides sqlx. [VERIFIED: CLAUDE.local.md — use `nix develop`]

#### service/src/toggle.rs — Toggle struct

```rust
pub struct Toggle {
    pub name: Arc<str>,
    pub enabled: bool,
    pub description: Option<Arc<str>>,
    pub value: Option<Arc<str>>,  // NEW
}
```

`From<&ToggleEntity>` and `From<&Toggle>` impls updated accordingly.

New service trait methods:
```rust
async fn get_toggle_value(&self, name: &str, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Option<Arc<str>>, ServiceError>;
async fn set_toggle_value(&self, name: &str, value: Option<&str>, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<(), ServiceError>;
```

`set_toggle_value` requires `toggle_admin` privilege (same as `enable_toggle`/`disable_toggle`). When setting a non-None value, also set `enabled = true`. When setting None, set `enabled = false`. This keeps D-25-05 consistent. [ASSUMED — per D-25-05 "enabled kept consistent"]

#### rest/src/toggle.rs — New endpoints

```rust
// GET /toggle/{name}/value → Option<String> (200 with null body if None, or 200 with "YYYY-MM-DD")
// PUT /toggle/{name}/value, body = JSON string "YYYY-MM-DD" → 204
// DELETE /toggle/{name}/value → 204 (clears value, disables toggle)
```

Added to `generate_route`:
```rust
.route("/{name}/value", get(get_toggle_value::<RestState>))
.route("/{name}/value", put(set_toggle_value::<RestState>))
.route("/{name}/value", delete(clear_toggle_value::<RestState>))
```

OpenAPI annotations required on all three handlers (`#[utoipa::path(...)]`). [VERIFIED: CLAUDE.md "always add #[utoipa::path]"]

#### rest-types/src/lib.rs — ToggleTO

```rust
pub struct ToggleTO {
    pub name: Arc<str>,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<Arc<str>>,
    #[serde(default)]
    pub value: Option<Arc<str>>,  // NEW
}
```

`From<&Toggle> for ToggleTO` and inverse updated.

---

### 4. Conflict Detection (Manual Wins — D-25-03)

In the reporting loop, `extra_hours` for the employee+year are already loaded. A manual ExtraHours(Holiday) covering the same day as a derived holiday is identified by:

```rust
// holiday_date: time::Date from special_day (year, calendar_week, day_of_week)
let has_manual = extra_hours
    .iter()
    .any(|eh| {
        eh.category == ExtraHoursCategory::Holiday
        && eh.date_time.to_date() == holiday_date
    });
```

Where `eh.date_time` is a `ShiftyDate`. `ShiftyDate` has a `to_date()` method returning `time::Date`. [VERIFIED: codebase — date_utils.rs:210]

**Match granularity:** per employee + per concrete date (not week). This is unambiguous since `SpecialDayEntity` has a single day-of-week per record and one record per holiday+day. [VERIFIED: special_day schema]

**For `get_reports_for_all_employees`:** `extra_hours_array` is loaded per employee at line 182. The same conflict-check applies.

---

### 5. Snapshot Bump Mechanics

#### const location: `service_impl/src/billing_period_report.rs:101`

```rust
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 10;  // current
// Change to:
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 11;
```

#### Locking test — MUST UPDATE

`service_impl/src/test/billing_period_snapshot_locking.rs:29`:
```rust
assert_eq!(
    CURRENT_SNAPSHOT_SCHEMA_VERSION, 10,   // ← change to 11
    "..."                                   // ← update description
);
```

This test is a **hard gate** — it will fail immediately after bumping the const if not updated. The test is intentionally designed to fail so developers notice. [VERIFIED: codebase — billing_period_snapshot_locking.rs]

#### No other golden files

The tests in `billing_period_report.rs` reference `CURRENT_SNAPSHOT_SCHEMA_VERSION` via the const itself (not hardcoded 10), so they automatically pass after the bump. [VERIFIED: codebase — test/billing_period_report.rs:1045,1097]

The `snapshot_schema_version: 1` values in test fixtures (lines 215, 322) are for test-local mock objects, not pinned to the production const. [VERIFIED: codebase]

---

### 6. Frontend Changes

#### api.rs — two new functions (after line 1595)

```rust
// GET /toggle/{name}/value → Option<String>
pub async fn get_toggle_value(config: Config, name: &str) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    // 204 = not set; 200 with body = set
    if response.status() == 204 {
        Ok(None)
    } else {
        Ok(response.json::<Option<String>>().await?)
    }
}

// PUT /toggle/{name}/value, body = JSON "YYYY-MM-DD"
pub async fn set_toggle_value(config: Config, name: &str, value: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client.put(url).json(value).send().await?.error_for_status()?;
    Ok(())
}

// DELETE /toggle/{name}/value
pub async fn clear_toggle_value(config: Config, name: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/toggle/{}/value", config.backend, name);
    let client = reqwest::Client::new();
    client.delete(url).send().await?.error_for_status()?;
    Ok(())
}
```

[ASSUMED — exact HTTP status codes for None; planner adjusts based on actual REST implementation]

#### loader.rs — two new functions (after line 1024)

```rust
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

#### settings.rs — Card 2 addition

The existing `SettingsPage` component has one card (Phase 24 toggle). Card 2 is added below it, following the exact layout contract from `25-UI-SPEC.md`. Key signals:

```rust
let mut date_str: Signal<ImStr> = use_signal(|| ImStr::from(""));
let mut date_str_loaded_empty = use_signal(|| false);
let mut save_result: Signal<Option<bool>> = use_signal(|| None);
let mut saving = use_signal(|| false);

let cutoff_resource = use_resource(move || loader::get_holiday_cutoff_date(config_for_load.clone()));
use_effect(move || {
    match &*cutoff_resource.read_unchecked() {
        Some(Ok(Some(date))) => { date_str.set(ImStr::from(date.as_str())); date_str_loaded_empty.set(false); }
        Some(Ok(None)) => { date_str.set(ImStr::from("")); date_str_loaded_empty.set(true); }
        _ => {}
    }
});
```

**WASM caveat (from MEMORY):** Setting `<input type=date>` programmatically does NOT reliably trigger Dioxus signal updates. The Save button MUST be enabled whenever `date_str` is non-empty — do NOT gate on "value changed from loaded state". [VERIFIED: MEMORY reference_dioxus_browser_test_date_inputs.md]

**TextInput component:** Reuse `TextInput { input_type: ImStr::from("date"), value: date_str, on_change: update_date_signal }`. Pattern confirmed via `contract_modal.rs` and `extra_hours_modal.rs` usage. [VERIFIED: 25-UI-SPEC.md]

---

### 7. i18n

Three locale files in `shifty-dioxus/src/i18n/`:

**mod.rs** — add 5 new `Key` variants after the existing Settings keys (line ~596):
```rust
SettingsHolidayAutoCreditLabel,
SettingsHolidayAutoCreditDescription,
SettingsHolidayAutoCreditSave,
SettingsHolidayAutoCreditClear,
SettingsHolidayAutoCreditUnsetHint,
```

**en.rs** (add after line ~952):
```rust
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditLabel, "Holiday auto-credit activation date");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditDescription, "Holidays on or after this date are credited automatically. Leave empty to disable.");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditSave, "Save date");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditClear, "Clear (disable)");
i18n.add_text(Locale::En, Key::SettingsHolidayAutoCreditUnsetHint, "Not set — automation is off.");
```

**de.rs** (add after line ~1035):
```rust
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditLabel, "Feiertags-Automatik aktiv ab");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditDescription, "Feiertage ab diesem Datum werden automatisch angerechnet. Leer lassen = Automatik deaktiviert.");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditSave, "Datum speichern");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditClear, "Löschen (deaktivieren)");
i18n.add_text(Locale::De, Key::SettingsHolidayAutoCreditUnsetHint, "Nicht gesetzt — Automatik inaktiv.");
```

**cs.rs** (add after line ~1021):
```rust
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditLabel, "Automatické připisování svátků od");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditDescription, "Svátky od tohoto data jsou automaticky připisovány. Prázdné = automatika vypnuta.");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditSave, "Uložit datum");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditClear, "Smazat (deaktivovat)");
i18n.add_text(Locale::Cs, Key::SettingsHolidayAutoCreditUnsetHint, "Nenastaveno — automatika je vypnuta.");
```

All strings verbatim from `25-UI-SPEC.md`. [VERIFIED: 25-UI-SPEC.md Copywriting Contract]

There is an existing i18n completeness test (line ~1295 in mod.rs) that checks all Key variants are covered in all locales. New keys must be added to all three locales or this test fails. [VERIFIED: codebase grep]

---

### 8. Tests

#### Test file location

New test file: `service_impl/src/test/reporting_holiday_auto_credit.rs`

Register in `service_impl/src/test/mod.rs` (existing module file).

#### Test pattern (from reporting_additive_merge.rs)

Tests use:
- `MockExtraHoursService` / `MockShiftplanReportService` / `MockSpecialDayService` / `MockToggleService`
- `fixture_sales_person_id()`, `fixture_work_details_8h_mon_fri()` from `reporting_phase2_fixtures.rs`
- Direct call to `hours_per_week(...)` for unit-testing the free function (lines 1375, 1411, etc.)
- `service.get_report_for_employee_range(...)` for integration-style tests

#### HOL-02 equivalence test shape

```
// Setup:
// - Work details: Mon-Fri, 40h/week → holiday_hours() = 40/5 = 8h
// - SpecialDay: Holiday on Monday of week W
// - Toggle value: cutoff = date before holiday → automation ON
// - No manual ExtraHours(Holiday) for that day
//
// Call: get_report_for_employee_range(week W)
// Assert: report.holiday_hours == 8.0
//         report.expected_hours == 40.0 - 8.0 == 32.0
//         report.balance_hours == (shiftplan - 32.0)
//
// Compare: same result as with manual ExtraHours(Holiday, amount=8.0) + no special_day
```

#### HOL-03 regression test shape

```
// Setup: same work details; AbsenceService mock (for derive_hours); SpecialDay mock
// Call: (conceptually) get_weekly_summary for the same employee+week
// Assert: paid_hours / committed_voluntary_hours / volunteer_hours unchanged
// This requires booking_information_service, which doesn't touch special_days or toggle
// → Can be a simpler mock test asserting the deps are not called
```

#### HOL-03 implementation note

`booking_information.rs:203-273` is the `paid_hours`/`committed_voluntary_hours`/`volunteer_hours` path. The regression guard is that `SpecialDayService` and `ToggleService` are NOT in `BookingInformationServiceImpl`'s deps. If the planner does not add them, HOL-03 is mechanically satisfied. The test verifies this by calling `get_weekly_summary` with a special_day + toggle present and asserting the output is unchanged. [VERIFIED: codebase — booking_information.rs not touched]

#### HCFG-01 stichtag guard test shape

```
// Setup: special_day Holiday on 2024-03-18 (week 12)
// Toggle value: "2024-03-25" (one week after)
// Assert: report.holiday_hours == 0.0 (cutoff not reached)
//
// Toggle value: "2024-03-18" (exact cutoff day)
// Assert: report.holiday_hours == 8.0 (holiday >= cutoff)
```

#### HCFG-03 no-double-count test shape

```
// Setup: special_day Holiday on Mon week W
// Manual ExtraHours(Holiday, 8.0) on same date
// Toggle value: cutoff before this date
// Assert: report.holiday_hours == 8.0 (not 16.0)
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Week→date conversion | Custom arithmetic | `time::Date::from_iso_week_date` or `ShiftyDate::new` | ISO week edge cases at year boundaries |
| ISO date parsing | Manual string split | `time::Date::parse` with format `[year]-[month]-[day]` | Handle invalid inputs cleanly |
| Toggle CRUD | New table/service | Extend existing `toggle` table and ToggleServiceImpl | Phase 24 established the pattern; reuse avoids duplication |
| Snapshot version management | Conditionals | Bump const + update locking test | The const IS the version; any other approach diverges |

---

## Common Pitfalls

### Pitfall 1: ISO Week Year Boundary

**What goes wrong:** A holiday with `year=2024, calendar_week=1, day_of_week=Monday` may have `time::Date` in year 2023. Conversely, `year=2024, calendar_week=52` Monday may be in 2024 or 2025.

**Why it happens:** ISO week year ≠ calendar year. Week 1 of ISO year 2024 starts on 2024-01-01 (Monday).

**How to avoid:** Always use `time::Date::from_iso_week_date(year, calendar_week, weekday)` for the concrete date. Never compute `year * 365 + week * 7 + weekday_offset`.

**Warning signs:** Holidays near Jan 1 or Dec 31 not appearing in reports. [VERIFIED: existing ShiftyDate pattern]

---

### Pitfall 2: Three Separate holiday_hours Injection Points

**What goes wrong:** Only injecting into `hours_per_week` (point 1c) but not into `EmployeeReport.holiday_hours` (point 1c, line 717-720) means the snapshot gets wrong data.

**Why it happens:** `EmployeeReport.holiday_hours` at line 717 is computed INDEPENDENTLY from `extra_hours`, not from `by_week`. The billing period snapshot reads `EmployeeReport.holiday_hours`.

**How to avoid:** Either switch line 717 to `by_week.iter().map(|w| w.holiday_hours).sum()`, or add the derived total explicitly.

**Warning signs:** Per-week display shows correct holiday_hours but `EmployeeReport.holiday_hours` (and thus the snapshot) is still 0.0. [VERIFIED: codebase analysis]

---

### Pitfall 3: absense_hours Not Updated

**What goes wrong:** Adding derived holiday hours ONLY to `holiday_hours` but NOT to `absense_hours` means `expected_hours` and `balance` don't reflect the holiday.

**Why it happens:** In `get_reports_for_all_employees`, `absense_hours` (line 387-391) is summed separately from `holiday_hours` (line 402-406). Same in `hours_per_week` (lines 1140-1148 vs 1269-1273).

**How to avoid:** Derived holiday hours must be added to BOTH `holiday_hours` AND `absense_hours` in both functions. Pattern: same as how absence_period-derived hours add to both their category field AND `derived_absence_hours`. [VERIFIED: codebase analysis]

---

### Pitfall 4: Snapshot Locking Test Fails Without Locking Test Update

**What goes wrong:** After bumping `CURRENT_SNAPSHOT_SCHEMA_VERSION` to 11, the test at `billing_period_snapshot_locking.rs:29` asserts `== 10` and fails.

**Why it happens:** The locking test is intentionally strict — any bump requires explicit human acknowledgment via test update.

**How to avoid:** Always update the locking test in the same task/wave as the const bump. Update both the assert value (10→11) and the doc string.

**Warning signs:** `cargo test` fails with "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 10 sein...". [VERIFIED: codebase]

---

### Pitfall 5: SQLx Compile-Time Query Failure After Migration

**What goes wrong:** After adding `value TEXT` column to `toggle` table, existing `query_as!(ToggleDb, ...)` calls that don't include `value` in SELECT fail at compile time (or at runtime with `query!` if SQLX_OFFLINE=true).

**Why it happens:** SQLx's `query_as!` maps column by position or name; the struct fields must exactly match.

**How to avoid:** Update ALL SELECT queries that return `ToggleDb` to include `value`. Run `sqlx database reset` (destructive — ask user first!) and `cargo sqlx prepare` after migration. Use `nix develop` for sqlx. [VERIFIED: CLAUDE.local.md, MEMORY feedback_destructive_db_ops]

---

### Pitfall 6: main.rs Construction Order

**What goes wrong:** `toggle_service` is currently constructed at line 911, AFTER `reporting_service` at line 878. Adding `toggle_service` as a dep of `ReportingServiceImpl` causes a compile error (use before define).

**How to avoid:** Move `toggle_service` construction block (lines 910-914) to BEFORE `reporting_service` (line 878). ToggleService is Basic-Tier — it has no deps on any service that would be built after it. [VERIFIED: codebase]

---

### Pitfall 7: SpecialDayService `get_by_week` — No Transaction Parameter

**What goes wrong:** Attempting to pass `tx.clone()` to `special_day_service.get_by_week(...)` causes a compile error — the method signature doesn't accept a transaction.

**Why it happens:** `SpecialDayService::get_by_week` reads from the DB pool directly (no tx support in the trait). This is pre-existing design.

**How to avoid:** Call `get_by_week` without tx. This is consistent with how other services call it (e.g., in `AbsenceServiceImpl`). [VERIFIED: service/src/special_days.rs]

---

## Code Examples

### Week-to-Date Conversion

```rust
// Source: shifty-utils/src/date_utils.rs:180-188 (ShiftyDate::new pattern)
use time::Weekday;
use shifty_utils::DayOfWeek;

let holiday_date: time::Date = time::Date::from_iso_week_date(
    special_day.year as i32,
    special_day.calendar_week,
    Weekday::from(special_day.day_of_week),
).expect("valid ISO week date from SpecialDayEntity");
```

### Cutoff Gate Check

```rust
// Parse stored ISO string; compare
let cutoff_date: Option<time::Date> = toggle_value_str
    .as_deref()
    .and_then(|s| time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok());

let should_derive = match cutoff_date {
    None => false,                                    // no cutoff = automation off
    Some(cutoff) => holiday_date >= cutoff,          // only on/after cutoff
};
```

### Conflict Check

```rust
// extra_hours_for_employee: Arc<[ExtraHours]> already loaded
let has_manual_holiday = extra_hours_for_employee
    .iter()
    .any(|eh| {
        eh.category == ExtraHoursCategory::Holiday
            && eh.date_time.to_date() == holiday_date
    });

if has_manual_holiday {
    continue; // manual wins — skip auto-credit for this day
}
```

### holiday_hours() Formula

```rust
// Source: service/src/employee_work_details.rs:112-114 (VERIFIED)
// wh: &EmployeeWorkDetails active on holiday_date
let derived_hours = if wh.has_day_of_week(special_day.day_of_week.into()) {
    wh.holiday_hours()   // = expected_hours / potential_days_per_week()
} else {
    0.0
};
```

### hours_per_week Signature Extension

```rust
// Current signature:
fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportDay]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[EmployeeWorkDetails],
    derived_absence: &std::collections::BTreeMap<time::Date, service::absence::ResolvedAbsence>,
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> Result<Arc<[GroupedReportHours]>, ServiceError>

// Extended (proposed):
fn hours_per_week(
    shiftplan_hours_list: &Arc<[ShiftplanReportDay]>,
    extra_hours_list: &Arc<[ExtraHours]>,
    working_hours: &[EmployeeWorkDetails],
    derived_absence: &std::collections::BTreeMap<time::Date, service::absence::ResolvedAbsence>,
    derived_holiday: &std::collections::HashMap<time::Date, f32>,  // NEW
    from_date: ShiftyDate,
    to_date: ShiftyDate,
) -> Result<Arc<[GroupedReportHours]>, ServiceError>
```

---

## State of the Art

| Area | Current | Phase 25 Change |
|------|---------|-----------------|
| Holiday credit | Manual ExtraHours(Holiday) only | Derive-on-read from special_day + contract |
| Toggle table | `name, enabled, description` only | + `value TEXT` column |
| Snapshot version | 10 (UV-05 / D-18-07) | 11 (Phase 25 holiday computation change) |
| ReportingService deps | 10 deps (no SpecialDayService, no ToggleService) | 12 deps |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | (resolved) `ExtraHoursCategory::Holiday.as_report_type()` returns `ReportType::AbsenceHours` | Pitfall 3 / Injection | VERIFIED: service/src/extra_hours.rs:57 — derived holiday hours MUST be added to both holiday_hours AND absense_hours |
| A2 | Option A (switch EmployeeReport.holiday_hours to use by_week) is the right approach for point 1c | Section 1c | Planner may prefer Option B (add derived total separately) |
| A3 | `get_by_week`-per-week loop is acceptable for MVP; a year-range query is deferred | Section 2 | Performance concern for large billing periods |
| A4 | Toggle enabled field mirrors value presence (set → enabled=true, clear → enabled=false) | Section 3 toggle.rs | If enabled stays independent, the UI could show conflicting state |
| A5 | REST: GET /toggle/{name}/value returns 204 when not set | Section 6 api.rs | Actual status code depends on REST implementation; planner aligns api.rs to match |
| A6 | Toggle-key name = `holiday_auto_credit` | Throughout | Claude's Discretion — if changed, all references must be consistent |

---

## Open Questions

1. **`ExtraHoursCategory::Holiday.as_report_type()` — RESOLVED**
   - Answer: Returns `ReportType::AbsenceHours` (service/src/extra_hours.rs:57).
   - Impact: Derived holiday hours MUST flow into BOTH `holiday_hours` and `absense_hours` in all three injection points to correctly reduce `expected_hours` and `balance`.

2. **`hours_per_week` call sites beyond get_report_for_employee_range?**
   - What we know: Test calls in reporting.rs tests (lines 1375+) call it directly with mocks
   - What's unclear: Any other production call sites
   - Recommendation: `grep -n "hours_per_week" service_impl/src/reporting.rs` — update all production callsites with the new parameter

3. **special_day range query performance**
   - What we know: 52-53 async get_by_week calls per full-year report
   - What's unclear: Whether this is acceptable for production load
   - Recommendation: Acceptable for MVP; add `find_by_year` DAO method if profiling shows issue

---

## Environment Availability

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| `nix develop` | sqlx, cargo | ✓ | Use for sqlx database reset + cargo sqlx prepare |
| SQLite | toggle migration | ✓ | Project uses SQLite throughout |
| `cargo clippy --workspace -D warnings` | Hard gate (CLAUDE.md) | ✓ | Must run before committing |
| `cargo test` | All service changes | ✓ | Run in backend workspace |
| WASM build check | Frontend changes | ✓ | `cargo build --target wasm32-unknown-unknown` in shifty-dioxus |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | tokio-based unit tests with mockall |
| Config file | none (workspace-level cargo test) |
| Quick run command | `cargo test holiday_auto_credit` |
| Full suite command | `cargo test --workspace` (backend) + `cargo test` (shifty-dioxus) |
| Clippy gate | `cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HOL-01 | Auto-credit derives correct hours from special_day + contract | unit | `cargo test test_holiday_auto_credit_basic` | ❌ Wave 0 |
| HOL-02 | Auto == manual ExtraHours(Holiday) in effect | unit | `cargo test test_holiday_auto_credit_equivalence` | ❌ Wave 0 |
| HOL-03 | booking_information unchanged by holiday auto-credit | unit | `cargo test test_holiday_auto_credit_no_year_view_impact` | ❌ Wave 0 |
| HCFG-01 | Holiday before cutoff → no auto-credit | unit | `cargo test test_holiday_before_cutoff_skipped` | ❌ Wave 0 |
| HCFG-02 | Toggle value GET/PUT roundtrip | manual (browser) | N/A — date input WASM caveat applies | N/A |
| HCFG-03 | Manual ExtraHours(Holiday) → auto-credit skipped | unit | `cargo test test_holiday_manual_wins` | ❌ Wave 0 |
| HSNAP-01 | Snapshot version == 11 | unit | `cargo test test_snapshot_schema_version_pinned` | ✅ (must update) |

### Wave 0 Gaps

- [ ] `service_impl/src/test/reporting_holiday_auto_credit.rs` — covers HOL-01/02/03, HCFG-01, HCFG-03
- [ ] Register new test module in `service_impl/src/test/mod.rs`
- [ ] Update `billing_period_snapshot_locking.rs` pinned assert: 10 → 11

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes | `toggle_admin` privilege check on set_toggle_value (same as enable/disable) |
| V5 Input Validation | yes | ISO date parse validation in REST handler + on frontend before PUT |
| V6 Cryptography | no | — |

**Threat pattern:** Malformed ISO date string in PUT /toggle/{name}/value body. Mitigation: parse and reject non-ISO strings in the REST handler before passing to service. Return 400 Bad Request.

---

## Sources

### Primary (HIGH confidence — verified from codebase)
- `service_impl/src/reporting.rs` — actual injection points at lines 402-406, 717-720, 1054-1293
- `service_impl/src/billing_period_report.rs:101` — CURRENT_SNAPSHOT_SCHEMA_VERSION = 10
- `service_impl/src/test/billing_period_snapshot_locking.rs:29` — pinned assert to update
- `service/src/employee_work_details.rs:104-156` — holiday_hours(), has_day_of_week()
- `dao/src/special_day.rs`, `service/src/special_days.rs` — SpecialDay types + get_by_week
- `dao/src/toggle.rs`, `service/src/toggle.rs`, `service_impl/src/toggle.rs`, `rest/src/toggle.rs` — complete toggle infra
- `dao_impl_sqlite/src/toggle.rs` — all existing SELECT/UPDATE queries (must add value column)
- `migrations/sqlite/20260105000000_app-toggles.sql` — current schema (no value column)
- `migrations/sqlite/20260627000000_seed-paid-limit-toggle.sql` — seeding pattern
- `shifty_bin/src/main.rs:338-356,878-889,910-914` — ReportingServiceDeps + constructor order
- `shifty-dioxus/src/page/settings.rs` — existing Phase 24 card pattern
- `shifty-dioxus/src/api.rs:1574-1595` — existing toggle API functions
- `shifty-dioxus/src/loader.rs:1008-1024` — existing toggle loader functions
- `shifty-dioxus/src/i18n/mod.rs:319,585-596` — existing Settings keys
- `shifty-utils/src/date_utils.rs:180-228,296-328` — ShiftyDate, ShiftyWeek, DayOfWeek

### Secondary (HIGH confidence — from project spec documents)
- `.planning/phases/25-feiertags-auto-anrechnung-stichtag-konfiguration/25-CONTEXT.md` — all locked decisions
- `.planning/phases/25-feiertags-auto-anrechnung-stichtag-konfiguration/25-UI-SPEC.md` — all i18n strings, layout, state machine
- `.planning/REQUIREMENTS.md` — HOL-*/HCFG-*/HSNAP-* requirement text
- `shifty-backend/CLAUDE.md` — snapshot bump rule, clippy gate, service-tier convention

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; all existing crates verified in codebase
- Architecture / injection points: HIGH — read actual source lines
- Toggle infra changes: HIGH — complete schema + code read
- Test shape: MEDIUM — pattern inferred from existing similar tests
- Frontend state machine: HIGH — verbatim from 25-UI-SPEC.md

**Research date:** 2026-06-28
**Valid until:** 2026-07-28 (stable codebase; no fast-moving dependencies)
