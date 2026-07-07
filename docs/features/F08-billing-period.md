# Feature: Billing Period & Snapshot Versioning

> **In short:** A Billing Period freezes, for a defined range and per
> employee, Balance, expected hours, worked hours, vacation and sick
> leave values as a **snapshot** in the DB — stamped with a schema
> version so later formula changes cannot silently "break" old
> snapshots.

**Cluster ID:** F08
**Status:** production (active Snapshot schema version **12**, as of 2026-07)
**First introduced:** 2025-08 (migration `20250813051848_add-table-billing-period.sql`);
versioning column 2026-04 (`20260426000000_add-snapshot-schema-version-to-billing-period.sql`)
**Responsible crates:**
- `service::billing_period`, `service::billing_period_report`
- `service_impl::billing_period`, `service_impl::billing_period_report`
- `dao::billing_period`, `dao::billing_period_sales_person`
- `dao_impl_sqlite::billing_period`, `dao_impl_sqlite::billing_period_sales_person`
- `rest::billing_period`
- Frontend: `shifty-dioxus/src/page/billing_periods.rs`, `.../billing_period_details.rs`, `.../service/billing_period.rs`

---

## 1. What is this? (Domain view)

A **Billing Period** is a contiguous range — typically a month, quarter,
or half-year — at the end of which the HR department freezes, for **every
paid employee**, a snapshot of their time metrics:

- **Balance** (hour account) at period end,
- **Expected Hours** in the range,
- **Overall Hours** (total worked),
- **Extra Work** (overtime category),
- **Vacation Hours / Sick Leave / Unpaid Leave / Holiday / Volunteer**,
- **Vacation Days** (taken) and **Vacation Entitlement**,
- optionally arbitrary **CustomExtraHours** depending on configured
  categories.

Each of these metrics is stored in **four views** (see §3):
`value_delta` (period only), `value_ytd_from` (YTD until period start),
`value_ytd_to` (YTD until period end), and `value_full_year` (full
calendar year).

HR thus sees at a glance what an employee earned in the billing month
**and** how they stand in the year to date — even months later, when
bookings have been retroactively adjusted.

**Example workflow from the HR perspective:**

1. HR opens the **Billing Periods** page
   (`shifty-dioxus/src/page/billing_periods.rs`).
2. Click "Create new Billing Period", pick an end date (e.g.
   `2026-06-30`).
3. Backend computes for **every paid** employee the metrics for the
   range `[last period + 1 day … end date]` and persists **one row** per
   metric in `billing_period_sales_person`.
4. The persisted `billing_period` row carries a `snapshot_schema_version`
   — the current version of the compute rules.
5. Later, HR opens the detail page
   (`shifty-dioxus/src/page/billing_period_details.rs`) and sees the
   values frozen back then — even if bookings inside the range have
   been changed since.
6. Optionally HR generates a custom report from a `text_template` (Tera
   or MiniJinja) over the snapshot data.

## 2. Domain rules

- **Rule — write-once:** A snapshot is written **once**
  (`create_billing_period` → `insert_billing_period_sales_person` in
  `service_impl/src/billing_period.rs:181-190`). There is no "update
  snapshot" path. Whoever wants to change the snapshot deletes it and
  creates a new one — and **only** for the latest period.
- **Rule — only the latest period is deletable:** `delete_billing_period`
  throws `ServiceError::NotLatestBillingPeriod` if `id` is not the
  latest one (`billing_period.rs:242-246`). This keeps the time chain
  gap-free.
- **Rule — only HR may write/delete:** `delete_billing_period` and
  `clear_all_billing_periods` check `HR_PRIVILEGE`
  (`billing_period.rs:226,275`). `generate_custom_report` likewise
  (`billing_period_report.rs:437-439`).
  **[To verify]** `create_billing_period` and the opening REST handler
  `POST /billing-period` have **no explicit permission check** before
  the call — the check currently hangs on the `HR_PRIVILEGE` check that
  fires deeper inside `ReportingService` /
  `EmployeeWorkDetailsService` during reads. A direct gate at the
  entry point would be more robust.
- **Rule — only paid persons in the snapshot:**
  `build_new_billing_period` filters `!sales_person.is_paid.unwrap_or(false)`
  (`billing_period_report.rs:371-373`). Volunteer helpers do not appear
  in the accounting. The comment there explains that this is **not**
  a Snapshot schema-version bump (person-set change, no
  `value_type` change).
- **Rule — periods are seamlessly chained:** Start date of a new
  period = `last_period.end_date.next_day()`
  (`billing_period_report.rs:349-356`). The first period starts on
  the UNIX-epoch day `1970-01-01` (the comment in
  `service/…/billing_period_report.rs:23` talks about `2020-01-01`;
  **[To verify]** — the code uses the UNIX epoch).
- **Rule — end date must lie after the last period:**
  The doc comment on the trait
  (`service/src/billing_period_report.rs:22-24`) promises an error
  if `end_date < last end_date` — but the code itself has no explicit
  guard. If the end date is unrealistically in the past, `next_day()`
  produces a start after the end date → empty or negative range.
  **[To verify]** whether a guard is missing or whether
  `ShiftyDate`/reporting catches this elsewhere.
- **Invariant — write version = read version:** The
  `snapshot_schema_version` with which a row was written must be checked
  when interpreting it later. Details in §7.
- **Invariant — enum completeness:** Every arm of
  `BillingPeriodValueType` must round-trip through `as_str()` /
  `from_str()` (`service/src/billing_period.rs:52-97`).
  A locking test (`test_billing_period_value_type_surface_locked`,
  `service_impl/src/test/billing_period_snapshot_locking.rs:44-70`)
  forces at compile time that every new variant is deliberately
  handled.

## 3. Data model

### Tables

| Table | Purpose | Key columns |
| --- | --- | --- |
| `billing_period` | Header row per Billing Period | `id`, `from_date_time`, `to_date_time`, **`snapshot_schema_version`**, `created`, `created_by`, `deleted`, `deleted_by`, `update_version`, `update_process` |
| `billing_period_sales_person` | One row per (period × person × `value_type`) | `id`, `billing_period_id`, `sales_person_id`, **`value_type`**, `value_delta`, `value_ytd_from`, `value_ytd_to`, `value_full_year`, `created_at`, `deleted_at`, `update_version` |

`billing_period_sales_person` has the unique index
`(billing_period_id, sales_person_id, value_type)`
(`20250813051848_add-table-billing-period.sql:36`), so the same metric
does not land twice for the same person of the same period.

### Migrations

- `2025-08-13` **`20250813051848_add-table-billing-period.sql`** —
  base tables `billing_period` + `billing_period_sales_person` with
  FKs onto `sales_person(id)`.
- `2026-04-26` **`20260426000000_add-snapshot-schema-version-to-billing-period.sql`**
  — additive column `snapshot_schema_version INTEGER NOT NULL DEFAULT 1`.
  Existing rows receive the default `1`, so validators can recognise
  them as "very old snapshot, semantics not guaranteed".

### `value_type` enum

The textual representation in the DB column `value_type` (see
`service/src/billing_period.rs:52-97`):

| `value_type` (string) | Rust variant | Meaning |
| --- | --- | --- |
| `balance` | `Balance` | Hour account (actual − expected + counting extras) |
| `overall` | `Overall` | Total worked hours incl. extras |
| `expected_hours` | `ExpectedHours` | Contractual expected in the range |
| `extra_work` | `ExtraWork` | Persisted extra work hours |
| `vacation_hours` | `VacationHours` | Vacation hours (extra_hours + absence_period-derived) |
| `sick_leave` | `SickLeave` | Sick leave hours |
| `unpaid_leave` | `UnpaidLeave` | Unpaid leave (from v3) |
| `holiday` | `Holiday` | Public-holiday hours |
| `volunteer` | `Volunteer` | Volunteer hours (only if ≠ 0) |
| `vacation_days` | `VacationDays` | Vacation days taken |
| `vacation_entitlement` | `VacationEntitlement` | Calendar-aliquot entitlement |
| `custom_extra_hours:<name>` | `CustomExtraHours(name)` | Free categories per business |

### Relationships

```
billing_period (1) ────────< (N) billing_period_sales_person
    │                                     │
    │                                     └── sales_person_id ──> sales_person(id)
    └── snapshot_schema_version (u32, stamped at write time)
```

Per person, typically **10–12 rows** are created (one per persisted
`value_type`) — plus one row per `custom_extra_hours:<name>`.

## 4. Service API

The cluster services follow the **Basic vs. Business-Logic** convention
(see `shifty-backend/CLAUDE.md`, section "Service tier conventions"):

- `BillingPeriodService` is **Basic-Tier** — CRUD on the aggregate, no
  consumption of other domain services (except the read-only
  `SalesPersonService` for the person set on reads).
- `BillingPeriodReportService` is **Business-Logic-Tier** — orchestrates
  `ReportingService`, `EmployeeWorkDetailsService`, `SalesPersonService`
  and finally writes **via** `BillingPeriodService` to the DB.

### 4.1 `BillingPeriodService` (Basic)

Trait: `service::billing_period::BillingPeriodService`
(`service/src/billing_period.rs:219-270`).

```rust
#[async_trait]
pub trait BillingPeriodService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_billing_period_overview(
        &self, ctx: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BillingPeriod]>, ServiceError>;

    async fn get_billing_period_by_id(
        &self, id: Uuid, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn create_billing_period(
        &self, entity: &BillingPeriod, process: &str, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn get_latest_billing_period_end_date(
        &self, ctx: …, tx: …,
    ) -> Result<Option<ShiftyDate>, ServiceError>;

    async fn delete_billing_period(
        &self, id: Uuid, ctx: …, tx: …,
    ) -> Result<(), ServiceError>;

    async fn clear_all_billing_periods(
        &self, ctx: …, tx: …,
    ) -> Result<(), ServiceError>;
}
```

**Auth gates:**

| Method | Permission | Location |
| --- | --- | --- |
| `get_billing_period_overview` | none (ignores context) | `billing_period.rs:92-104` |
| `get_billing_period_by_id` | calls `SalesPersonService::get_all` — which carries the permission check | `billing_period.rs:107-146` |
| `create_billing_period` | **no direct check** — the caller (`BillingPeriodReportService`) relies on downstream checks | `billing_period.rs:149-198` |
| `get_latest_billing_period_end_date` | none | `billing_period.rs:201-216` |
| `delete_billing_period` | `HR_PRIVILEGE` | `billing_period.rs:225-227` |
| `clear_all_billing_periods` | `HR_PRIVILEGE` | `billing_period.rs:274-276` |

**[To verify]** Whether `create_billing_period` needs an explicit
`HR_PRIVILEGE` guard — currently it uses the auth subject only to
fill `created_by`.

**Transaction behaviour:**

- All methods open a transaction on `tx=None`
  (`use_transaction(tx)`) and commit at the end.
- `create_billing_period` first writes the header, then in a loop per
  person per `value_type` one row — all under **one** transaction
  (`billing_period.rs:156-192`). Rollback on error = consistent state.
- `delete_billing_period` cascades: first
  `billing_period_sales_person` rows, then header
  (`billing_period.rs:255-262`).

**Dependencies:**

- DAOs: `BillingPeriodDao`, `BillingPeriodSalesPersonDao`,
  `TransactionDao`.
- Services: `SalesPersonService` (Basic, for person enumeration on
  reads), `PermissionService` (for `HR_PRIVILEGE` checks +
  `current_user_id` for auditing).
- Utility: `UuidService`, `ClockService`.

Constructed via the `gen_service_impl!` macro
(`service_impl/src/billing_period.rs:26-36`).

### 4.2 `BillingPeriodReportService` (Business-Logic)

Trait: `service::billing_period_report::BillingPeriodReportService`
(`service/src/billing_period_report.rs:10-54`).

```rust
#[async_trait]
pub trait BillingPeriodReportService {
    type Context: …; type Transaction: dao::Transaction;

    async fn build_new_billing_period(
        &self, end_date: ShiftyDate, ctx: …, tx: …,
    ) -> Result<BillingPeriod, ServiceError>;

    async fn build_and_persist_billing_period_report(
        &self, end_date: ShiftyDate, ctx: …, tx: …,
    ) -> Result<Uuid, ServiceError>;

    async fn generate_custom_report(
        &self, template_id: Uuid, billing_period_id: Uuid, ctx: …, tx: …,
    ) -> Result<Arc<str>, ServiceError>;
}
```

**Core path — `build_billing_period_report_for_sales_person`**
(`billing_period_report.rs:134-331`): per person **four**
`ReportingService::get_report_for_employee_range` calls:

1. **Report-Start**: `[year_start(start_date.year) … start_date-1]`
   → basis for `value_ytd_from`.
2. **Report-End**: `[year_start(end_date.year) … end_date]`
   → basis for `value_ytd_to`.
3. **Report-Full-Year**: `[year_start(end_date.year) … year_end]`
   → basis for `value_full_year`.
4. **Report-Delta**: `[start_date … end_date]`, flag `false`
   (presumably "period only, no Carryover"; **[To verify]**
   semantics of the flag in `ReportingService::get_report_for_employee_range`)
   → basis for `value_delta`.

These four numbers are woven into a `BillingPeriodValue` per
`BillingPeriodValueType`. `Volunteer` is only inserted if
`report_delta.volunteer_hours != 0.0`
(`billing_period_report.rs:283`). `CustomExtraHours` are name-based;
the YTD values are collected via `.find(|ch| ch.name ==
custom_hours.name)` from the three other reports
(`billing_period_report.rs:294-320`).

**`build_new_billing_period`** (`billing_period_report.rs:341-399`):

- Determines `start_date` = `last_period.end_date.next_day()` or
  UNIX-epoch if no period exists.
- Iterates `SalesPersonService::get_all`, filters `is_paid == true`.
- Builds per person via `build_billing_period_report_for_sales_person`
  a `BillingPeriodSalesPerson`.
- Creates in-memory `BillingPeriod { id: Uuid::nil(), snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION, … }`
  — **not** yet persisted.

**`build_and_persist_billing_period_report`**
(`billing_period_report.rs:401-425`):

- Calls `build_new_billing_period`.
- Calls `BillingPeriodService::create_billing_period(&billing_period, "BillingPeriodReportService", ctx, tx)`.
- Commits transaction.
- Returns `Uuid::nil()` **[Attention: suspected bug]** —
  `billing_period_id` is read before `create_billing_period` which
  only generates the new `Uuid` there. **[To verify]** whether the
  returned UUID is the real one or `Uuid::nil()` — line
  `billing_period_report.rs:412` reads `billing_period.id`, which in
  `build_new_billing_period` is set as `Uuid::nil()`
  (`billing_period_report.rs:387`). The REST handler
  `create_billing_period` serialises this into the response body
  (`rest/src/billing_period.rs:124-133`); the FE ignores it and
  reloads the list (`shifty-dioxus/src/service/billing_period.rs:60-62`),
  so this does not break the user flow.

**`generate_custom_report`** (`billing_period_report.rs:427-550`):

- Checks `HR_PRIVILEGE` explicitly.
- Loads `TextTemplate` + `BillingPeriod`.
- Enriches the snapshot data with `sales_person.name` and
  `is_paid`/`is_dynamic` from `EmployeeWorkDetailsService::all`.
- Renders depending on `TemplateEngine`:
  - **Tera**: `Tera::default().add_raw_template().render()`.
  - **MiniJinja**: `minijinja::Environment::new().render_str()`.
- NaN/Inf are sanitised to `0.0` beforehand
  (`billing_period_report.rs:478-481`).

**Dependencies:**

- Services: `BillingPeriodService` (writes via it), `ReportingService`
  (computes the four report views), `SalesPersonService`,
  `EmployeeWorkDetailsService`, `TextTemplateService`,
  `PermissionService`.
- Utility: `UuidService`, `ClockService`, `TransactionDao`.

**Auth gates:**

| Method | Permission | Note |
| --- | --- | --- |
| `build_new_billing_period` | indirectly via `ReportingService` / `SalesPersonService` | No direct check at the entry point. |
| `build_and_persist_billing_period_report` | indirectly (as above) | The writing call would be opened from the REST layer without a gate. **[To verify]** whether an HR gate should be added. |
| `generate_custom_report` | `HR_PRIVILEGE` | `billing_period_report.rs:437-439` |

## 5. REST endpoints

All under prefix `/billing-period` (see `rest/src/lib.rs:642`).
Handlers in `rest/src/billing_period.rs`.

| Method | Path | Description | DTO in | DTO out | Key errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/billing-period` | List (headers only, no `sales_persons`) | — | `Vec<BillingPeriodTO>` | 401, 500 |
| `GET` | `/billing-period/{id}` | Detail with all `BillingPeriodSalesPersonTO` | — | `BillingPeriodTO` | 401, 404 |
| `POST` | `/billing-period` | Create new period (builds snapshot) | `CreateBillingPeriodRequestTO { end_date }` | `Uuid` | 400, 401, 403, 500 |
| `DELETE` | `/billing-period` | Soft-delete **all** periods (reset) | — | 204 | 401, 403 |
| `DELETE` | `/billing-period/{id}` | Delete single period (only if latest) | — | 204 | 403, 404, 409 (`NotLatestBillingPeriod`) |
| `POST` | `/billing-period/{id}/custom-report/{template_id}` | Render text report | — | `String` (`text/plain`) | 401, 403, 404, 500 |

DTOs see `rest-types::lib.rs:1401-1494`:

- `BillingPeriodTO` — contains `snapshot_schema_version: u32`
  (`rest-types/src/lib.rs:1460`); the frontend sees the version.
- `BillingPeriodSalesPersonTO` — `values: BTreeMap<String, BillingPeriodValueTO>`
  with the `value_type` string as key.
- `BillingPeriodValueTO` — flat: `value_delta`, `value_ytd_from`,
  `value_ytd_to`, `value_full_year`.
- `CreateBillingPeriodRequestTO { end_date: time::Date }`.

**OpenAPI:** `BillingPeriodApiDoc` (`rest/src/billing_period.rs:250-267`)
collects all handlers under tag `billing_period`.

## 6. Frontend integration

- **Pages:**
  - `shifty-dioxus/src/page/billing_periods.rs` — overview, create
    dialog, delete-with-confirm (see MEMORY feedback "warnings inline
    instead of dialog" — here a confirm dialog is used explicitly
    because deletion is permanent).
  - `shifty-dioxus/src/page/billing_period_details.rs` — detail view
    with filter (`show_paid`, `show_active`, `filter_text`), sorted
    value table, custom-report selection.
- **Service:** `shifty-dioxus/src/service/billing_period.rs` — coroutine
  consumes `BillingPeriodAction::{LoadBillingPeriods, LoadBillingPeriod, CreateBillingPeriod, DeleteBillingPeriod}`,
  keeps store `BILLING_PERIOD_STORE { billing_periods, selected_billing_period }`.
- **API:** `shifty-dioxus/src/api.rs` with `get_billing_periods`,
  `get_billing_period`, `post_billing_period`, `delete_billing_period`,
  `generate_custom_report`.
- **i18n keys** (`shifty-dioxus/src/i18n/mod.rs:257-379`):
  `BillingPeriods`, `BillingPeriodDetails`, `CreateNewBillingPeriod`,
  `BillingPeriod`, `LoadingBillingPeriods`, `LoadingBillingPeriodDetails`,
  `CreateBillingPeriod`, `NoSalesPersonsInBillingPeriod`,
  `InvalidBillingPeriodId`, `SelectEndDateForNewBillingPeriod`,
  `DeleteBillingPeriod`, `ConfirmDeleteBillingPeriod`,
  `DeleteBillingPeriodError` — present in `en.rs` / `de.rs` / `cs.rs`.
- **Proxy:** `shifty-dioxus/Dioxus.toml:46`
  `backend = "http://localhost:3000/billing-period"` — without this
  entry the dx-serve dev server returns 404 (see MEMORY feedback
  "Dioxus.toml proxy for new backend endpoints").

## 7. Snapshot Versioning — the hard core

This section is the **actual raison d'être** of the cluster and the
central reference for anyone changing the calculation.

### 7.1 The contract

**Field:** `billing_period.snapshot_schema_version INTEGER NOT NULL DEFAULT 1`
(migration `20260426000000_add-snapshot-schema-version-to-billing-period.sql`).
Round-trips as `u32` through `BillingPeriodEntity.snapshot_schema_version`
(`dao/src/billing_period.rs:11`) and `BillingPeriod.snapshot_schema_version`
(`service/src/billing_period.rs:23`) all the way to `BillingPeriodTO`
(`rest-types/src/lib.rs:1460`).

**Constant (single source of truth):**

```rust
// service_impl/src/billing_period_report.rs:117
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
```

**Writer:** `build_new_billing_period` stamps the version onto every
freshly built snapshot:

```rust
// service_impl/src/billing_period_report.rs:386-396
let billing_period = BillingPeriod {
    id: Uuid::nil(),
    start_date,
    end_date,
    snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,   // <── line 390
    sales_persons: sales_person_reports.into(),
    // …
};
```

From there, `BillingPeriodService::create_billing_period` persists the
value into the DB (`service_impl/src/billing_period.rs:169`,
`dao_impl_sqlite/src/billing_period.rs`).

### 7.2 Bump rules (binding)

**Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` by exactly 1 when you:**

1. **add a new persisted `value_type`.**
   Example: v3 (`UnpaidLeave` as the 12th enum variant) —
   `billing_period_report.rs:41-48`.
2. **remove or rename an existing `value_type`.**
   (Historically never happened — the enum history is purely additive.)
3. **change the computation of an existing `value_type`** — different
   formula, different inputs, different filtering.
   Example: v4 (`day_fraction::Half` halves the expected hour count in
   `derive_hours_for_range`, affects `VacationHours`/`SickLeave`/
   `UnpaidLeave` + transitively `Balance`/`ExpectedHours` — doc comment
   `billing_period_report.rs:46-49`).
4. **change the input set** the computation reads from.
   Example: v5 ("additive merge" — Vacation/SickLeave/UnpaidLeave now
   read **both** sources: live `extra_hours` **plus**
   `absence_period`-derived; doc comment `billing_period_report.rs:50-55`).

### 7.3 Do **NOT** bump when you:

- add new REST endpoints that only read,
- change frontend views,
- add new fields on **other** tables (that do not produce a
  snapshot),
- refactor the writer without the computed output per `value_type`
  changing (verification: all existing tests green + a diff comparison
  old/new for the same inputs = `0.0`),
- make a **person-set change** (e.g. adding the `is_paid` filter in
  `build_new_billing_period`). The comment
  `billing_period_report.rs:365-370` states this explicitly:
  *"NO value_type change → NO CURRENT_SNAPSHOT_SCHEMA_VERSION bump."*

### 7.4 History of bumps (verified in the code)

From the large doc comment `billing_period_report.rs:38-117`:

| Version | Trigger | Affected `value_type`s |
| --- | --- | --- |
| v1 | Baseline (initial snapshot model) | — |
| v2 | Intermediate bump (details lost from history) | — |
| **v3** | Phase 2 — new `value_type` `UnpaidLeave` + AbsencePeriod-derived Vacation/Sick/Unpaid | `UnpaidLeave`, `VacationHours`, `SickLeave` |
| **v4** | Phase 8.3 — `day_fraction::Half` halves expected per day | Vacation/Sick/Unpaid (hours + days) |
| **v5** | Phase 8.4 — additive merge: extra_hours + absence_period instead of flag branch | Vacation/Sick/Unpaid |
| **v6** | Phase 8.4 Gap 2 (WR-01) — absence_period-derived categories symmetrically reduce Balance/ExpectedHours | `Balance`, `ExpectedHours` (+ transitively) |
| **v7** | Bugfix vacation-hours-overcounted — weekly cap `workdays_per_week` | `Vacation*`, `Balance`, `ExpectedHours`. Never deployed. |
| **v8** | Bugfix report-ehrenamt-gesamtstunden — cap overflow leaked into `overall/balance`; now week-capped | `Overall`, `Balance`, `ExpectedHours` |
| **v9** | quick-260624-ujk — shiftplan hours without a contract row are counted as `volunteer` instead of neutralised | `Volunteer`, transitively `Balance`/`ExpectedHours` |
| **v10** | UV-05 / D-18-07 — converted hours-based absences flow into per-week category fields | `VacationDays` (+ Sick/Unpaid days) |
| **v11** | Phase 25 (HOL-01/02, HCFG-01) — derive-on-read holiday auto credit via toggle | `Holiday`, transitively `Balance`/`ExpectedHours` |
| **v12** | Phase 28 (VAC-OFFSET-01 / D-28-05) — off-by-one fix in `vacation_days_for_year`; Jan-1st start subtracts 0 days instead of ~1/365 | `VacationEntitlement` (**not** `VacationDays`) |

Phase 15 (committed_voluntary two-band) was explicitly **not** bumped
because axis-B only, no persisted `value_type` affected
(`billing_period_report.rs:74`).

Phase 17 (person-set filter `is_paid`) was explicitly **not** bumped
(`billing_period_report.rs:365-370`).

**Milestone v2.6 Phase 54 — non-bump confirmation.**
`CURRENT_SNAPSHOT_SCHEMA_VERSION` remains **12**. Rationale: Phase 54
(voluntary-stats data-model, see feature [F14](./F14-rebooking.md))
adds only the `extra_hours.source` marker column (values: `manual` \|
`rebooking`) and two new `rebooking_batch` / `rebooking_batch_entry`
tables — neither introduces a new persisted `BillingPeriodValueType`,
nor changes any existing computation. Voluntary-Stats itself is a
**live-computed HR-only read view**, not a persisted snapshot: no
`billing_period_sales_person` row, no versioning, no writer touches
`billing_period_report.rs`. The snapshot-bump decision **12 → 13** is
deferred to Phase 56 (`REB-AUTO-05`, F4-Cron) when the first
`Rebooking`-source writer begins to feed the balance chain and reader
filters (`source = 'manual'`) become semantically load-bearing — see
`REQUIREMENTS.md`.

### 7.5 Edge case — validator reads a v11 snapshot with v12 code

Concrete case from the v12 doc comment
(`billing_period_report.rs:108-116`):

- Old snapshot: `snapshot_schema_version = 11`. Its
  `VacationEntitlement` rows hold the value that the old formula (with
  off-by-one) produced for the Jan-1st contract start.
- Current code: `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`. For the same
  contract, the new formula produces a minimally higher value (~1/365
  less deducted).
- **A naive re-compute validator** would see a discrepancy and yell
  "data bug!".
- **Correct validator behaviour** (per comment rule):
  1. Read `bp.snapshot_schema_version`.
  2. If `< CURRENT_SNAPSHOT_SCHEMA_VERSION` → mark as "older schema" and
     **skip** the affected re-validation (here: `VacationEntitlement`).
  3. `VacationDays` **may** still be re-validated because it is not
     touched by the v12 change.

Analogously: v10 → v11 skips Holiday hours
(`billing_period_report.rs:101-107`); v9 → v10 skips Vacation/Sick/Unpaid
days (`billing_period_report.rs:93-100`); and so on.

**[To verify]** In which file the validator logic actually lives.
The doc comment references it, but the current cluster does not yet
contain a visible validator routine. Possibly part of a later phase
(SNAP-02+).

### 7.6 Reference to the central edge-case doc

See [`../domain/edge-cases.md#3-billing-period--snapshots`](../domain/edge-cases.md#3-billing-period--snapshots)
for further edge scenarios:

- Race between a booking change and snapshot creation
  (transaction consistency of
  `build_and_persist_billing_period_report`).
- Feature Toggle changes semantics: Toggle change **must** trigger a
  version bump (see §9 of edge-cases).
- Report without snapshot: live computation applies — the UI currently
  does not indicate that no frozen value exists
  (**[To verify]** DTO field `is_snapshot`).

### 7.7 Gate test (locking)

`service_impl/src/test/billing_period_snapshot_locking.rs` guards
against silent drift:

```rust
// line 27-38
#[test]
fn test_snapshot_schema_version_pinned() {
    assert_eq!(
        CURRENT_SNAPSHOT_SCHEMA_VERSION, 12,
        "CURRENT_SNAPSHOT_SCHEMA_VERSION muss 12 sein nach Phase 28 …"
    );
}
```

Whoever bumps the constant **must** also lift this assert and rewrite
the message to reflect the new reason. The second test
`test_billing_period_value_type_surface_locked` (`:44-70`) is a
`match`-locking on `BillingPeriodValueType`: a new enum variant
produces `non-exhaustive patterns` and forces the author to
deliberately decide whether a bump is needed.

## 8. Tests

- **Unit — service basic:**
  `service_impl/src/test/billing_period.rs` (427 LoC). Mocks DAOs +
  downstream services (`MockDeps`), covers CRUD paths incl.
  `NotLatestBillingPeriod` guard, HR gate on delete/clear-all,
  cascade delete `billing_period_sales_person` → `billing_period`.
- **Unit — report business logic:**
  `service_impl/src/test/billing_period_report.rs` (1368 LoC).
  Covers `build_new_billing_period` (person filter, period chains),
  `build_and_persist_billing_period_report` (persistence path,
  `snapshot_schema_version` = current constant), and
  `generate_custom_report` (Tera/MiniJinja + sanitisation of NaN/Inf).
- **Locking regressions:**
  `service_impl/src/test/billing_period_snapshot_locking.rs` (70 LoC) —
  see §7.7.
- **Round-trip:**
  `service::billing_period::tests` contains
  `volunteer_row_round_trips_through_from_entities`
  (`service/src/billing_period.rs:186-216`) to guarantee that a
  persisted `volunteer` row is not silently dropped.
- **DAO trait default methods:** `dao::billing_period::tests` covers
  `all_ordered_desc` incl. soft-delete filter
  (`dao/src/billing_period.rs:99-238`).
- **Known gaps:**
  - **[To verify]** There is currently no visible test that runs a
    validator against multiple snapshot versions (v11-vs-v12 skip
    behaviour). If the validator logic exists, a parametrised test
    belongs with it.
  - **[To verify]** No test for the `end_date < last end_date` guard,
    because the guard itself may be missing.
  - **[To verify]** No documented e2e backend round-trip in the browser
    — MEMORY feedback "verify backend round-trip e2e" would be
    explicitly applicable here for the delete-latest-only path.

## 9. History & context

- **2025-08 (Milestone v1.0 area):** Base feature via
  `20250813051848_add-table-billing-period.sql` — snapshot concept
  without versioning. Bump rules exist only conceptually.
- **2026-04:** Migration
  `20260426000000_add-snapshot-schema-version-to-billing-period.sql`
  introduces the `snapshot_schema_version` field. The associated
  OpenSpec change lives/lived under
  `openspec/changes/billing-period-snapshot-versioning/` (see CLAUDE.md
  reference; **[To verify]** current status — the directory was empty
  or archived at the time of verification).
- **Continuous bumps v3–v12** along the feature phases 2, 8.3, 8.4,
  ~debug/vacation-hours-overcounted, ~debug/report-ehrenamt-gesamtstunden,
  quick-260624-ujk, phase 18 UV-05/D-18-07, phase 25 HOL-01/02/HCFG-01,
  phase 28 VAC-OFFSET-01/D-28-05 — see §7.4 and the doc-comment history
  in `service_impl/src/billing_period_report.rs:38-117`.
- **Cross-cluster dependency:** Any change to `ReportingService`
  (`service/src/reporting.rs` and `service_impl/src/reporting.rs`),
  `EmployeeWorkDetailsService::vacation_days_for_year`,
  `derive_hours_for_range` in absence_period logic, or to a Feature
  Toggle that triggers reporting semantics — MUST be checked for
  snapshot impact and, if affected, bumped.
- **PR review pattern:** For any PR that touches files under
  `service_impl/src/reporting.rs`,
  `service_impl/src/booking_information.rs`, or
  `service_impl/src/absence_period.rs` (or their traits), an active
  grep for `CURRENT_SNAPSHOT_SCHEMA_VERSION` and the deliberate
  decision "bump or comment why not" belongs in the review checklist
  pattern.

---

**Summary:** The cluster freezes HR-relevant time metrics per employee
period-by-period immutably and stamps every snapshot with a schema
version, so later formula changes cannot silently devalue old
snapshots. Whoever changes the computation of a persisted `value_type`
bumps `CURRENT_SNAPSHOT_SCHEMA_VERSION` **by exactly 1** and lifts the
assert in `billing_period_snapshot_locking.rs` alongside — otherwise no
validator can ever distinguish schema drift from real data bugs.

---

*Last verification against code:* 2026-07-05 against
`CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` (see git blame of this file).
