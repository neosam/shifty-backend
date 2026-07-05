# Feature: Vacation Management — Balance, Offset & Carryover

> **In short:** The cluster computes how much Vacation an employee is
> entitled to in the calendar year, how much has been used/planned, what
> was carried over from the previous year, and forwards the open remainder
> at year end. HR can correct the contractual entitlement per (person, year)
> via a signed offset.

**Cluster ID:** F06
**Status:** production (v2.4+, since Milestone 8)
**First introduced:** Milestone 8 (Vacation-Balance endpoint), Milestone 28 (Vacation Entitlement Offset)
**Responsible crates:**
`service::vacation_balance`, `service::vacation_entitlement_offset`, `service::carryover`,
`service_impl::vacation_balance`, `service_impl::vacation_entitlement_offset`, `service_impl::carryover`,
`service_impl::scheduler` (cron trigger for Carryover),
`dao::vacation_entitlement_offset`, `dao::carryover`,
`rest::vacation_balance`, `rest::vacation_entitlement_offset`,
`rest-types::{VacationBalanceTO, VacationEntitlementOffsetTO}`

---

## 1. What is this? (Domain view)

Every employee's employment contract states a number of paid vacation days
per year (`employee_work_details.vacation_days`). This cluster answers
three core questions for every employee:

- **How much Vacation am I entitled to this year?** — Contract entitlement,
  aliquot on contract change, and optionally corrected by an HR offset.
- **How much have I used and how much is still planned?** — Sum over the
  Vacation Absence periods of the year, split into `used` (until today)
  and `planned` (from tomorrow onwards).
- **How much do I have left?** — `entitled + carryover − (used + planned)`.

The **Carryover** is the remaining balance at year end that an employee
takes into the next year (analogous to the hour Carryover for balance
hours). It is freshly computed and persisted nightly by the scheduler —
both for the previous year (retroactive effects from late-entered bookings)
and for the current year (continuous preview).

The **Vacation Entitlement Offset** is an HR-only correction per
(employee, year): an integer in days that is added to the contract
entitlement — e.g. `+2` as a bonus, `-3` as a one-off deduction. The
offset is deliberately *whole-day* and is added AFTER the `.round()`
integer conversion of the entitlement, so it can never "disappear" through
rounding.

**Example workflow from the user perspective (HR):**

1. HR opens the "Absences" tab, selects year 2026 and an employee.
2. The `VacationEntitlementCard` shows five tiles:
   `Contract`, `Carryover from previous year`, `Used`, `Planned`, `Remaining`.
3. In addition to the effective entitlement, HR sees the *raw* contract
   entitlement and the current offset. An inline editor lets HR set or
   delete the offset. The tile refreshes immediately.
4. On Jan 1st, the cron runs: the previous year is finally "closed"
   (last late entry), the new year is initialised with a Carryover entry.

**Example workflow (employee):**

1. Employee opens their self view → sees `entitled`, `carryover`,
   `used`, `planned`, `remaining`. The **raw** contract entitlement and
   the offset are NOT served to them by the backend (API hiding, D-28-03).

## 2. Domain rules

### Contract entitlement (`entitled_days`)

- Source: `EmployeeWorkDetails.vacation_days: u8` — one annual number of
  vacation days per contract segment (`service/src/employee_work_details.rs:37`).
- Pro-rating: `EmployeeWorkDetails::vacation_days_for_year(year)`
  (`service/src/employee_work_details.rs:158-194`) computes, for contracts
  that cover only part of the year, the fraction `ordinal_days /
  days_in_year` and returns an `f32`. **Phase-28 fix (D-28-04):** The
  deduction at contract *start* begins at day 1 (`ordinal - 1`), so a
  contract with start date `01.01.` does NOT deduct one-sixtieth.
- Aggregation over all non-deleted contracts of the year, then `.round()`
  to whole days (`vacation_balance.rs:195-200` — consistent with
  `reporting.rs`).
- **Offset added after rounding:** `entitled_effective = round(base) +
  offset_days` (`vacation_balance.rs:213-214`, D-28-02). The offset is a
  whole number in days — it can be negative.

### Used / Planned (used / planned days)

- Data source: `AbsenceService::derive_hours_for_range(year_start,
  year_end, sales_person_id, …)` — returns per day of the year a
  `ResolvedAbsence { category, hours, days }`.
- Only days with `category == AbsenceCategory::Vacation` count; other
  categories (Sick, UnpaidLeave, …) are skipped
  (`vacation_balance.rs:248-249`).
- Conflict resolution (Sick > Vacation > UnpaidLeave) already happens in
  `derive_hours_for_range`; here it is only filtered.
- Split at cutover date `today`: `date <= today` → `used_days`,
  `date > today` → `planned_days` (`vacation_balance.rs:255-262`).
- **Days come exactly from `ResolvedAbsence.days`** — half days (via
  `day_fraction`, e.g. on half-day public holidays) and weekly cap are
  already accounted for. A naive "calendar day count" would have
  miscounted weekends/public holidays.
- Hours are summed in parallel, but currently only kept defensively
  (`_ = (used_hours, planned_hours)` in `vacation_balance.rs:264`).

### Carryover (`carryover_days`)

- Read from `CarryoverService::get_carryover(sales_person_id, year - 1, …)`
  (`vacation_balance.rs:270-273`). **Year semantics:** A `Carryover`
  entry with `year = Y` stores the end-of-Y balance that flows into
  Y+1. For the Carryover *into* `year`, `year - 1` must therefore be
  queried — this is a historical bug fix that originally forwarded
  `year` directly (see module doc comment in
  `vacation_balance.rs:30-35`).
- Soft-deleted rows are ignored (`filter(|c| c.deleted.is_none())`,
  `vacation_balance.rs:275`).

### Remaining (`remaining_days`)

```
remaining_days = entitled_effective + carryover_days − (used_days + planned_days)
                                                                 ↑             ↑
                                                              incl. half days / conflict resolution
```

(`vacation_balance.rs:279-280`)

### Offset semantics

- Exactly **one active row per (sales_person_id, year)**, enforced by a
  partial unique index on `WHERE deleted IS NULL`
  (`migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql`).
- `set` is an upsert: if an active row already exists, `offset_days` and
  `version` are updated; otherwise a new row is created
  (`service_impl/src/vacation_entitlement_offset.rs:67-102`).
- `delete` is a soft delete (`deleted = now(), new version`,
  `vacation_entitlement_offset.rs:130-139`).
- `get` returns only the active row
  (`find_by_sales_person_id_and_year` with `WHERE deleted IS NULL`).
- **All CRUD ops are HR-gated** (D-28-06b). A non-HR caller gets
  `ServiceError::Forbidden`.

### API hiding for HR-only fields

- `VacationBalance` carries two additional fields — `offset_days` and
  `computed_entitled_days` — as `Option<..>`.
- For HR callers both are set to `Some(..)`, for self-only they are
  `None` (`vacation_balance.rs:127-128`, `vacation_balance.rs:292-298`).
- **Important:** `entitled_days` (the effective value) is identical for
  both roles — the offset is never hidden from the calculation, only the
  *breakdown* is protected (D-28-03).

### Permission model

- `VacationBalanceService::get`: **HR ∨ self**. Implemented via
  `tokio::join!` over `check_permission(HR)` and
  `verify_user_is_sales_person` (`vacation_balance.rs:114-128`).
- `VacationBalanceService::get_team`: **HR-only**
  (`vacation_balance.rs:147-149`). Aggregates only *paid* Sales Persons
  (`get_all_paid`, `vacation_balance.rs:151-154`).
- `VacationEntitlementOffsetService::{get,set,delete}`: **HR-only**
  (`vacation_entitlement_offset.rs:39-41, 63-65, 116-118`).
- `CarryoverService::{get,set}`: **no explicit gate** — the context is
  ignored (`_context`) and the ops are intended as an internal aggregate
  of the pipeline (cron scheduler calls with `Authentication::Full`,
  reporting with `Authentication::Full`). A direct REST endpoint on
  Carryover does **not** exist.

## 3. Data model

### Tables

| Table | Purpose | Key columns |
| --- | --- | --- |
| `employee_yearly_carryover` | Year-end balance per (person, year) — hours **and** vacation days in *one* row | `sales_person_id`, `year`, `carryover_hours REAL`, `vacation INTEGER`, `deleted`, `update_process`, `update_version` (PK: `(sales_person_id, year)`) |
| `vacation_entitlement_offset` | Signed HR correction per (person, year) | `id BLOB PK`, `sales_person_id`, `year`, `offset_days INTEGER`, `deleted`, `update_process`, `update_version` |
| `employee_yearly_carryover_pre_cutover_backup` | **Historical only (deleted in Milestone 8.6).** Cutover backup before Absence cutover. | — |

### Migrations

Chronologically:

- `20241215063132_add_employee-yearly-carryover.sql` — base table
  `employee_yearly_carryover(sales_person_id, year, carryover_hours,
  created, deleted, update_process, update_version)`. Primary key
  `(sales_person_id, year)`.
- `20241231065409_add_employee-yearly-vacation-carryover.sql` — additive
  column `vacation INTEGER NOT NULL DEFAULT 0`. **Both values share one
  row** — the hour Carryover and the vacation-day Carryover are upserted
  together (`CarryoverEntity`, `dao/src/carryover.rs:5-14`).
- `20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql`
  (Milestone 4 cutover) — snapshot table for rollback before the Absence
  cutover; never populated in production.
- `20260611000001_drop-employee-yearly-carryover-pre-cutover-backup.sql`
  (Milestone 8.6, D-04) — removal of the backup table. Forward-only.
- `20260629000000_create-vacation-entitlement-offset.sql` (Milestone 28,
  VAC-OFFSET-01, D-28-01) — new table `vacation_entitlement_offset` with
  its own `id`-PK and `UNIQUE INDEX WHERE deleted IS NULL` on
  `(sales_person_id, year)`.

### Relationships

- `employee_yearly_carryover.sales_person_id` → `sales_person(id)` (FK).
- `vacation_entitlement_offset.sales_person_id` → `sales_person(id)` (FK).
- Both tables are bound to `SalesPerson`, not to `EmployeeWorkDetails`
  (i.e. NOT to a specific contract segment).
- Two separate Carryover concepts share **one** row with **two** columns
  (`carryover_hours` for the hour balance, `vacation` for vacation days).
  This is deliberate — both are rewritten together by the same nightly
  update path (`shiftplan_edit.update_carryover`, see §4).

## 4. Service API

### Traits

**`service::vacation_balance::VacationBalanceService`** — Business-Logic-Tier
(combines cross-entity data from 4 other domain services).

```rust
#[async_trait]
pub trait VacationBalanceService {
    type Context: …; type Transaction: …;

    async fn get(&self, sales_person_id: Uuid, year: u32,
                 context: Authentication<Self::Context>,
                 tx: Option<Self::Transaction>)
        -> Result<VacationBalance, ServiceError>;

    async fn get_team(&self, year: u32,
                      context: Authentication<Self::Context>,
                      tx: Option<Self::Transaction>)
        -> Result<Arc<[VacationBalance]>, ServiceError>;
}
```

**`service::vacation_entitlement_offset::VacationEntitlementOffsetService`** —
Basic-Tier (entity manager, NO domain service as dep).

```rust
async fn get   (&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<Option<VacationEntitlementOffset>, ServiceError>;
async fn set   (&self, sales_person_id: Uuid, year: u32, offset_days: i32, ctx, tx)
    -> Result<VacationEntitlementOffset, ServiceError>;   // upsert
async fn delete(&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<(), ServiceError>;                          // soft-delete
```

**`service::carryover::CarryoverService`** — Basic-Tier
(`service/src/carryover.rs:49-69`).

```rust
async fn get_carryover(&self, sales_person_id: Uuid, year: u32, ctx, tx)
    -> Result<Option<Carryover>, ServiceError>;
async fn set_carryover(&self, carryover: &Carryover, ctx, tx)
    -> Result<(), ServiceError>;                          // upsert
```

### Auth gates

| Op | Gate | Ref |
| --- | --- | --- |
| `VacationBalanceService::get` | HR ∨ self | `vacation_balance.rs:114-128` |
| `VacationBalanceService::get_team` | HR-only | `vacation_balance.rs:147-149` |
| `VacationEntitlementOffsetService::get/set/delete` | HR-only | `vacation_entitlement_offset.rs:39,63,116` |
| `CarryoverService::get/set_carryover` | **no gate** — context ignored; may only be called internally (scheduler / reporting) | `carryover.rs:31,45` |

### Transaction behaviour

- Both services open a transaction via `transaction_dao.use_transaction(tx)`,
  work atomically, and commit at the end — standard pattern.
- `VacationBalanceService::get_team` iterates over all paid Sales Persons
  and calls `compute_balance` within *one* transaction
  (`vacation_balance.rs:157-161`). On a failure for *one* person, the
  entire team query aborts (no partial result).
- `VacationEntitlementOffsetService::set` is upsert-atomic (find →
  update/create → commit in one transaction).
- **Scheduler update** (`shiftplan_edit.update_carryover_all_employees`)
  runs in *one* master transaction over all employees
  (`shiftplan_edit.rs:414-440`). If a report is missing for one
  employee, the entire nightly run aborts. **[To verify]** whether this
  is intended behaviour or whether a per-person transaction would be
  more robust.

### Cron trigger for Carryover

Two cron jobs in `SchedulerServiceImpl`
(`service_impl/src/scheduler.rs:59-74`):

```rust
shiftplan_edit_service.update_carryover_all_employees(year - 1, Full, None)
shiftplan_edit_service.update_carryover_all_employees(year, Full, None)
```

- Cron expression: **`"0 * * * * *"`** (`scheduler.rs:45`) — **[To verify]**:
  the notation is 6-part; the cluster comments talk about "nightly", but
  the expression looks like "every minute at second-slot 0". The code
  path is idempotent anyway (upsert), so it is not harmful, but the
  intent should be clarified.
- Why two years? Retroactive changes to the previous year (e.g. newly
  recorded sick leave for December) should correct the previous-year
  Carryover without waiting for a manual trigger.

### `update_carryover(sales_person_id, year)` — the compute core

`service_impl/src/shiftplan_edit.rs:362-407` (not in
`service_impl/src/carryover.rs`; that is deliberate — the compute core
belongs to the business-logic layer of the Shiftplan aggregate, see D-04
in the service tier model):

1. Fetches `employee_report = reporting_service.get_report_for_employee(sp_id,
   year, weeks_in_year, Full, tx)`.
2. `new_carryover_hours = employee_report.balance_hours`
3. `new_vacation_entitlement = floor(vacation_entitlement − vacation_days) as
   i32` — the vacation-day Carryover is the **floored** remainder from
   reporting.
4. Persist both values together as `Carryover{ carryover_hours, vacation }`
   via `set_carryover` (upsert).

### Dependencies

- `VacationBalanceServiceImpl` (`vacation_balance.rs:58-69`):
  `AbsenceService`, `EmployeeWorkDetailsService`, `CarryoverService`,
  `SalesPersonService`, `VacationEntitlementOffsetService`,
  `PermissionService`, `ClockService`, `TransactionDao`.
- `VacationEntitlementOffsetServiceImpl` (`vacation_entitlement_offset.rs:14-22`):
  `VacationEntitlementOffsetDao`, `PermissionService`, `ClockService`,
  `UuidService`, `TransactionDao` — **no domain service dependency**
  (D-28-06, anti-cycle rule: `VacationBalance` consumes Offset, so
  Offset must not consume a business-logic service).
- `CarryoverServiceImpl` (`carryover.rs:14-19`): `CarryoverDao`,
  `TransactionDao` — minimal, no permission gate in the service.

## 5. REST endpoints

| Method | Path | Description | DTO in | DTO out | Key errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/vacation-balance/{sales_person_id}/{year}` | Remaining vacation for a person | — | `VacationBalanceTO` | 403 (no HR + not self), 404 |
| `GET` | `/vacation-balance/team/{year}` | Aggregate over all paid Sales Persons | — | `[VacationBalanceTO]` | 403 |
| `POST` | `/vacation-entitlement-offset` | Upsert the HR offset | `VacationEntitlementOffsetTO` | `VacationEntitlementOffsetTO` | 403, 500 |
| `DELETE` | `/vacation-entitlement-offset/{sales_person_id}/{year}` | Soft-delete the offset | — | 204 no content | 403, 404 |

Registration: `rest/src/lib.rs:36, 590-591, 657-660`.

**Route order:** `/vacation-balance/team/{year}` **must be registered
before** `/vacation-balance/{sales_person_id}/{year}`; otherwise Axum
matches `"team"` as a Uuid parse → 400 instead of the team route
(`rest/src/vacation_balance.rs:34-42`).

`VacationBalanceTO` (`rest-types/src/lib.rs:2158-2178`):

```
sales_person_id, year,
entitled_days: f32,          // effective (round(base) + offset)
carryover_days: i32,
used_days: f32,
planned_days: f32,
remaining_days: f32,
offset_days: Option<i32>,            // HR-only, otherwise None
computed_entitled_days: Option<f32>, // HR-only, otherwise None
```

`VacationEntitlementOffsetTO` (`rest-types/src/lib.rs:2224-2230`): plain
DTO without `id`/`version` — the endpoint is upsert-based and the client
identifies the row via `(sales_person_id, year)`.

**No REST endpoints for `CarryoverService`.** The service is only called
server-internally (scheduler + reporting); direct client access to
Carryover rows is not intended.

## 6. Frontend integration

- **Pages / components:**
  - `shifty-dioxus/src/page/absences.rs:394-472` — `VacationEntitlementCard`
    (5 stat tiles, self/team mode, HR toggle for inline offset editor).
  - `shifty-dioxus/src/page/absences.rs:750-…` — `VacationPerPersonList`
    (HR aggregate list).
  - `shifty-dioxus/src/component/employee_view.rs:186-442` — Carryover
    balance and Vacation Carryover are displayed as read-only fields in
    the employee details view (source: HR employee report, not the
    VacationBalance endpoint).
- **Service:** `shifty-dioxus/src/service/vacation_balance.rs` — coroutine
  with `VacationBalanceAction` channel, store for self and team.
- **State:** `shifty-dioxus/src/state/vacation_balance.rs` — frontend
  domain (`From<&VacationBalanceTO>`).
- **API client:** `shifty-dioxus/src/api.rs:638-694` — `get_vacation_balance`
  and `get_team_vacation_balance`.
- **Loader:** `shifty-dioxus/src/loader.rs:841-860` — thin wrappers.
- **i18n keys (`shifty-dioxus/src/i18n/mod.rs:491-508`):**
  `VacationCardSelfTitle`, `VacationCardSelfSubtitle`, `VacationCardTeamTitle`,
  `VacationCardTeamSubtitle`, `VacationStatContract`, `VacationStatCarryover`,
  `VacationStatUsed`, `VacationStatPending`, `VacationStatRemaining`,
  `VacationEntitlementHero`, `VacationDaysRemaining`, `VacationPerPersonHeader`,
  `VacationPerPersonShowAll/Less`, `VacationOffsetLabel`,
  `VacationOffsetComputedLabel`.
- **Proxy** (`shifty-dioxus/Dioxus.toml:99-102`):
  - `backend = "http://localhost:3000/vacation-balance"`
  - `backend = "http://localhost:3000/vacation-entitlement-offset"`
  Both are registered — the standard mistake for new endpoints ("forgot
  Dioxus.toml proxy") does NOT apply here.

## 7. Edge cases

For the central edge-case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), section
[Hour account](../domain/edge-cases.md#1-stundenkonto).

- **Retroactive change in a closed year drifts the Carryover.**
  If an `AbsencePeriod` is retroactively entered for last December
  (e.g. sick certificate for a past range), the previous-year Carryover
  no longer matches. The scheduler addresses this with the `(year - 1)`
  job (`scheduler.rs:60`), which recomputes both years per tick. As long
  as the backend process is running, the Carryover converges by itself
  — after a server restart, only with the next cron tick.
- **Mid-year new hire.** `vacation_days_for_year` returns an aliquot
  `f32` (e.g. 15.25). Only after aggregating over all contract segments
  is `.round()` applied (`vacation_balance.rs:195-200`). The HR offset is
  added AFTERWARDS (D-28-02) — a `-1` offset therefore cannot "evaporate"
  through rounding.
- **Year rollover race.** If the cron runs on Jan 1st just after the
  date switch, first `update_carryover_all_employees(prev_year, …)`
  (final snapshot) and then `(current_year, …)` (initial snapshot) is
  invoked. `set_carryover` is upsert, so idempotent — multiple runs are
  harmless. A manual `POST` on Absence during the cron tick can however
  lead to a "stale" snapshot for a few seconds (until the next tick).
- **`representative_hours_per_day` on contract change mid-year.**
  Model A (decision 2026-06-12) picks a *single* representative
  `hours_per_day` per year — the most recent contract segment that
  touches `year` (`vacation_balance.rs:78-97`). With two contracts of
  different `hours_per_day`, the hours→days conversion is an
  approximation. Currently `hours_per_day` is only computed defensively;
  the day numbers come exactly from `ResolvedAbsence.days`.
- **Carryover year off-by-one.** Historical bug: the old implementation
  read `carryover(sp, year)` instead of `carryover(sp, year - 1)`.
  Result: the Carryover *from* the current year (which did not exist
  yet) was read → always 0. The module doc comment in
  `vacation_balance.rs:30-35` describes the fix; the test
  `carryover_read_uses_prior_year` (`test/vacation_balance.rs:892`)
  locks in the regression.
- **Offset is whole-day, never fractional.** `offset_days: i32` — there
  is no way to enter a half day as an offset. This is deliberate to
  keep API hiding simple (`i32` serialises consistently even for
  self-only, where it is `None`).
- **No permission gate in `CarryoverService`.** The service accepts any
  `Authentication<Ctx>` and ignores it. Since there is no REST endpoint,
  this is currently acceptable — but should be tightened if an admin UI
  for Carryover overrides ever appears.
- **`get_report_for_employee` as dependency.** The compute core
  (`shiftplan_edit.update_carryover`) builds on the *full* employee
  report. Changes to the report formula immediately affect the written
  `carryover_hours`/`vacation` value. There is no snapshot schema
  versioning for the Carryover table. **[To verify]** whether this is
  intentional or whether, analogous to the Billing Period Snapshot
  (see `CLAUDE.md` — Snapshot Schema Versioning), Carryover also needs
  a version column.

## 8. Tests

- **Unit — VacationBalance** (`service_impl/src/test/vacation_balance.rs`,
  1 113 lines, mock-based):
  - Happy path self (`get_returns_entitlement_minus_used_minus_planned`,
    l. 238),
  - Happy path HR (`get_with_hr_succeeds`, l. 314),
  - AuthZ (`get_other_sales_person_without_hr_is_forbidden` l. 364,
    `get_team_without_hr_is_forbidden` l. 393),
  - Team aggregate (`get_team_aggregates_per_paid_sales_person`, l. 409),
  - Edge (`get_with_no_active_contract_returns_zero_entitlement` l. 489,
    `get_rounds_aliquot_entitlement_to_whole_number` l. 542,
    `get_year_without_carryover_returns_zero_carryover` l. 593),
  - Half day (`half_day_vacation_counts_as_half_day`, l. 667),
  - Days-field-direct (`part_time_contract_used_days_come_from_days_field`,
    l. 693),
  - Used/planned split (`active_period_splits_used_and_planned_at_today`,
    l. 714),
  - Category filter (`non_vacation_categories_are_ignored`, l. 741),
  - Carryover year semantics (`get_carryover_is_called_with_previous_year`
    l. 796, `carryover_from_previous_year_is_included_in_balance` l. 835,
    `carryover_read_uses_prior_year` l. 892),
  - Offset (`offset_calc` l. 975, `offset_delta` l. 1013,
    `offset_api_hiding` l. 1033).
- **Unit — VacationEntitlementOffset**
  (`service_impl/src/test/vacation_entitlement_offset.rs`, 331 lines):
  `get`/`set`/`delete` happy path, HR gate denial, upsert semantics,
  soft-delete semantics.
- **Unit — Carryover** (`service_impl/src/test/carryover.rs`, 187 lines):
  `get_carryover_found/not_found`, DAO-error propagation,
  `set_carryover_success`, DAO error on set.
- **Frontend** (`shifty-dioxus/src/page/absences.rs:3531+`): snapshot tests
  for `VacationEntitlementCard` with `selected_person` (HR detail view).
- **Known gaps:**
  - No integration test (in-memory SQLite round-trip) for the scheduler
    cycle `update_carryover_all_employees` — the interaction
    Reporting↔Carryover↔VacationBalance is only mock-covered.
  - No test for the race case "Absence change during a running cron
    tick" (hard to reproduce in practice; **[To verify]**).
  - **[To verify]** Regression test for the off-by-one in the
    contract-start deduction (D-28-04): `vacation_days_for_year` with
    `from_date = 01.01.` — the test evidence lives in
    `service/src/employee_work_details.rs:287+ (mod vacation_days_for_year_tests)`
    per grep, but the exact coverage should be checked whenever
    `vacation_days_for_year` changes.

## 9. History & context

- **End of 2024 — basic Carryover.**
  `20241215063132_add_employee-yearly-carryover.sql` creates the hour
  Carryover table (only `carryover_hours`).
  `20241231065409_add_employee-yearly-vacation-carryover.sql` (two weeks
  later) additively adds the `vacation` column. It was a conscious
  design decision to keep hour and vacation-day Carryover in **one**
  row, because both are written by the same nightly job from the same
  report.
- **Milestone 4 — cutover backup.** Shortly before the Absence cutover, a
  backup table (`…_pre_cutover_backup`) was created to allow rollback to
  the pre-cutover state in an emergency. It was never populated in
  production and was removed again in Milestone 8.6 (D-04) —
  forward-only.
- **Milestone 8 — Vacation-Balance endpoint.** The business-logic service
  `VacationBalanceService` (D-04 in `08-CONTEXT.md`) was introduced as
  an aggregation layer to serve the frontend `VacationEntitlementCard`
  and `VacationPerPersonList` with a single round-trip. Wave-4
  delivered the endpoint, wave-5 the frontend wire-up.
- **Milestone 28 — Vacation Entitlement Offset (VAC-OFFSET-01).**
  Domain requirement: HR must be able to correct the contract
  entitlement case by case without changing the contract itself (bonuses,
  special deductions, court orders). Design decisions:
  - D-28-01: Separate table `vacation_entitlement_offset` with `id`-PK
    (not as a column on `employee_yearly_carryover`, to preserve domain
    boundaries).
  - D-28-02: Add offset AFTER `.round()`.
  - D-28-03: API hiding for non-HR — only the effective value is visible.
  - D-28-04: Fix of the off-by-one in the contract-start deduction.
  - D-28-06/06b: Basic-Tier for the Offset service (no domain dep, else
    cycle with VacationBalance).
  - D-28-07: Frontend inline editor only on HR detail path, never in the
    employee self view.
- **Context reads:**
  - `.planning/phases/08-…` — Vacation-Balance foundation (business-logic
    service classification, test coverage requirements).
  - `.planning/phases/28-…` — VAC-OFFSET-01 design.
  - `.planning/phases/*-51*` — ToggleService Full-Context bypass (effect
    on `derive_hours_for_range`, which VacationBalance consumes
    internally).

---

**Summary:** F06 solves the triad entitlement → usage → carryover per
employee/year with an hours-based core (`derive_hours_for_range`) and an
HR integer offset without destroying the rounding. The cron-driven
Carryover rewrite keeps previous and current year continuously
consistent — a snapshot versioning like the Billing Period one is
currently deliberately absent.

*Last verification against code:* see git blame of this file.
