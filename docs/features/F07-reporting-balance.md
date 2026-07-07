# Feature: Reporting & Balance Calculation

> **In short:** Computes the hour account (balance) for every paid
> employee across arbitrary time slices — per week, per year, per range —
> aggregating bookings, extra hours, absences, Carryover, and public
> holidays into a single formula: *Balance = actual − expected + extras*.

**Cluster ID:** F07
**Status:** production (core feature)
**First introduced:** before v1.0 (hour-account origin feature); multiple
deep rewrites (phases 8, 15, 17, 25, 34, 47, 51 and v2.2 RPT-01)
**Responsible crates:**
- `service::reporting`, `service::block`, `service::block_report`,
  `service::my_block`
- `service_impl::reporting` (2205 lines, business-logic-tier),
  `service_impl::block`, `service_impl::block_report`
- `rest::report`, `rest::block_report`, `rest::my_block`
- `rest-types` (`ShortEmployeeReportTO`, `EmployeeReportTO`,
  `WorkingHoursReportTO`, `BlockTO`, `EmployeeWeeklyStatisticsTO`,
  `EmployeeAttendanceStatisticsTO`)
- Frontend: `shifty-dioxus/src/page/weekly_overview.rs`,
  `shifty-dioxus/src/page/my_shifts.rs`, `shifty-dioxus/src/page/report.rs`
  (via `Employee` aggregate)

**Related but documented separately:**
- **Carryover** (year rollover, previous-year balance) → see
  [F06 Vacation Management](./F06-vacation-management.md), section
  *Carryover*.
- **Billing Period Snapshots** (persisted period aggregates with
  schema versioning) → see [F08 Billing Period](./F08-billing-period.md).
- **Extra Hours** (legacy time recording + custom categories) → see
  [F04 Extra Hours](./F04-extra-hours.md).
- **Absence** (range-based vacation/sick system, v1.0+) →
  see [F05 Absence System](./F05-absence-system.md).

---

## 1. What is this? (Domain view)

The Reporting cluster is **the compute engine behind the hour account**.
For every paid employee it answers the core question:
*"How many hours did they work, how many were expected, how many hours
of vacation/sick leave/public holidays count in — and what is the bottom
line?"*

From the user perspective, it feeds three views:

1. **HR overview (Report page)** — list of all paid employees with
   yearly balance, actual hours, expected hours, vacation days, etc.
2. **Employee detail** — per employee one row per calendar week
   (`by_week`) with the category splits. Yearly sums are built up from
   the weeks.
3. **My Shifts / Weekly Overview** — the personal view: "What did I
   work this week, what is on my balance account, which blocks are
   coming up next?"

Additionally, the cluster provides **blocks** (`Block`, `MyBlock`,
`BlockReport`): merged, consecutive bookings per weekday — the
operational view on "my shift today" or "which blocks are not yet
sufficiently staffed".

**Example workflow from the user perspective:**

1. Employee logs in → sees on `my_shifts` their blocks for the coming
   weeks (from `MyBlockService` → `BlockService`).
2. HR opens the Report page for an employee → frontend calls
   `GET /report/{id}?year=…&until_week=…` → backend computes:
   Bookings (from `ShiftplanReportService`) + Extra Hours (from
   `ExtraHoursService`) + Absences (from `AbsenceService.derive_hours…`) +
   Carryover (from `CarryoverService`) + public holidays (from
   `SpecialDayService`, gated by the `holiday_auto_credit` toggle) →
   returns an `EmployeeReport`.
3. Frontend renders the balance in a single row and the split in
   `by_week`.

**Why is this the most complex feature?** Because it is the output
layer for *all* underlying aggregates: every special rule (cap,
volunteer-work-without-contract, dynamic contract, holiday auto credit,
absence merge, custom categories, cutover old/new) must combine
correctly here — and must not drift from the persisted snapshots
(`billing_period_sales_person`).

---

## 2. Domain rules

### 2.1 The core formula

The hour account is conceptually simple; the complexity is in the
sources and gates.

```
balance = worked_hours − expected_hours + carryover_prev_year
```

Where:

```
worked_hours   = shiftplan_hours (capped, per week)
               + extra_work_hours          # ExtraHoursCategory::ExtraWork
               + custom_working_hours      # modifies_balance == true

expected_hours = Σ (contract_expected_for_week
                     − absence_reducing_expected_for_week)

absence_reducing_expected
             = extra_hours(AbsenceHours)   # Vacation, SickLeave, Holiday, UnpaidLeave
             + derived_absence             # from AbsenceService (V/S/U)
             + derived_holiday             # from SpecialDayService, gated

# Volunteer/volunteer work does NOT count in worked (but is reported).
# Unavailable counts neither in worked nor reduces expected.
```

Reference (implemented): `service_impl/src/reporting.rs:635`
(`balance_hours = overall_hours − expected_hours + previous_year_carryover`
for the yearly overview) and `reporting.rs:1502` (`balance = shiftplan_paid
+ extra_work_hours − expected_hours + absence_hours` per week).

> **Important:** `expected_hours` is already carried *after* the Absence
> deduction (`planned_hours − absence_hours` per week); the term
> `+ absence_hours` in the weekly formula moves the Absence onto the
> actual side *only for the balance*, so it does not reduce twice.
> Concretely: the model is mathematically equivalent to
> `balance = worked_hours − (planned − absence) + carryover`.

### 2.2 `ExtraHoursCategory` — semantics per category

Defined in `service/src/extra_hours.rs:41-97` via two getters:
`as_report_type()` → `ReportType` and `availability()` → `Availability`.

| Category | `ReportType` | reduces `expected`? | increases `worked`? | reported in report as … |
| --- | --- | --- | --- | --- |
| `ExtraWork` | `WorkingHours` | no | **yes** (`overall_hours`) | `extra_work_hours` |
| `Vacation` | `AbsenceHours` | **yes** | no | `vacation_hours` (+ `vacation_days`) |
| `SickLeave` | `AbsenceHours` | **yes** | no | `sick_leave_hours` (+ `sick_leave_days`) |
| `Holiday` | `AbsenceHours` | **yes** | no | `holiday_hours` (+ `holiday_days`) |
| `UnpaidLeave` | `AbsenceHours` | **yes** | no | `unpaid_leave_hours` (NOT in `vacation_days`) |
| `Unavailable` | `None` | no | no | `unavailable_hours` (info only) |
| `VolunteerWork` | `Documented` | no | no | `volunteer_hours` |
| `CustomExtraHours(id)` — `modifies_balance=true` | `WorkingHours` | no | **yes** | in `custom_extra_hours` and implicitly in `overall_hours` |
| `CustomExtraHours(id)` — `modifies_balance=false` | `None` | no | no | only in `custom_extra_hours` |

This is the authoritative table. Whoever introduces a new category must
set both getters and add a row here (see
[edge-cases §1.5](../domain/edge-cases.md#15-balance-perimeter--was-zählt-zur-balance)).

**Special case `UnpaidLeave`:** Reduces expected, adds nothing.
This is the only category that does **not** affect the `vacation_days()`
calculation but flows into `absence_days()`. Related tests:
`reporting.rs:1753-1849` (`test_unpaid_leave_tracked_separately`,
`test_unpaid_leave_does_not_affect_vacation_days`,
`test_unpaid_leave_included_in_absence_days`,
`test_unpaid_leave_reduces_expected_hours`).

### 2.3 `ExtraHoursReportCategory` — reporting layer

Extends `ExtraHoursCategory` by exactly one variant: `Shiftplan` (daily
bookings derived from `ShiftplanReportService`). All eight Extra Hours
categories are mapped 1:1 (`service/src/reporting.rs:26-41`).

The TO (`ExtraHoursReportCategoryTO`) flattens `CustomExtraHours(LazyLoad)`
into `Custom(Uuid)` (`rest-types/src/lib.rs:437`), because no `LazyLoad`
can be transported over the wire.

### 2.4 Carryover as a pre-persisted previous-year balance

The `CarryoverService` returns for `(sales_person_id, year - 1)` a
snapshot with `carryover_hours` (f32) and `vacation` (i32).
`ReportingService` adds `carryover_hours` **once** to the yearly balance
and `vacation` to `vacation_entitlement`
(`reporting.rs:806-819, 844-853`).

- The value is **not recomputed** if something is retroactively changed
  in a closed year. The live report shows the new truth; the persisted
  Carryover value drifts.
  → see [edge-cases §1.1](../domain/edge-cases.md#11-carryover-grenze--jahresrollover).
- The Carryover read runs with `Authentication::Full` (internal aggregate),
  so it can be loaded even for HR calls without a Sales Person context
  (`reporting.rs:811`).
- If the consumer does not need the yearly aggregation (range report),
  `include_carryover: false` can be set — then `carryover=0.0`.

### 2.5 Special Days impact on expected

Public holidays reduce expected in two ways:

1. **Manual path (classic):** HR enters `ExtraHours(Holiday, 8h,
   2026-05-01, …)` → `Holiday` is `AbsenceHours` → reduces expected.
2. **Automatic path (phase 25, HOL-01/02, HCFG-01/03):**
   The `holiday_auto_credit` toggle stores an ISO cutover date. For
   every public holiday ≥ cutover date, `build_derived_holiday_map`
   (`reporting.rs:151-242`) builds an entry `(date, hours)` for the
   range/year — with *derived hours* = `EmployeeWorkDetails::holiday_hours()`
   (contract hours × 1/workdays). These hours go into `holiday_hours`
   AND into `absence_hours` (see *Pitfall 3* in the code:
   `reporting.rs:533`).

**Manual-wins rule (D-25-03 / HCFG-03):** If a manual
`ExtraHours(Holiday)` already exists for the same employee + same day,
the auto credit is skipped (`reporting.rs:218-224`). Conversely: auto
credit is only entered when `wh.has_day_of_week(dow) && wh.holiday_hours() > 0`
— i.e. the contract covers this weekday.

**Cutover gate:** If the toggle value is missing (automation off) or the
toggle read receives `Unauthorized` (mock/internal callers without user
context), the helper returns an empty map (`reporting.rs:169-179`).
This is the legacy off branch.

For the *dynamic* week (no/expected=0 contract), derived holiday is
gated to 0 — otherwise the expected would go negative and inflate the
balance (`reporting.rs:1097-1098, 1406-1414`, "dynamic-week guard").

### 2.6 Further rules that are not obvious

- **Weekly cap** (`cap_planned_hours_to_expected`, since HRPX-01): when
  set on the `EmployeeWorkDetails` record and `shiftplan_hours >
  expected_hours`, the excess is moved into `auto_volunteer_hours`
  (`apply_weekly_cap`, `reporting.rs:124-137`). The capped value is the
  **only source** for `overall_hours`/`balance_hours`/`shiftplan_hours`
  — the raw uncapped value only exists transiently
  (`reporting.rs:763-767, 785, 836-847`).
- **No-contract week** (user rule `quick-260624-ujk`): If an
  `EmployeeWorkDetails` row is missing entirely for the week →
  Shiftplan hours go as volunteer work (`volunteer_hours`), not into
  `overall_hours`. Distinguished from the dynamic contract: there the
  row *exists* but has `expected_hours == 0` → then `expected = actual`,
  no volunteer redirect. See the three-case distinction in
  `hours_per_week` (`reporting.rs:1444-1454`) and the parallel logic in
  `get_reports_for_all_employees` (`reporting.rs:388-500`) and
  `get_week` (`reporting.rs:1006-1156`).
- **Volunteer merge:** `volunteer_hours` of a week is
  `manual_volunteer + auto_volunteer (cap) + no_contract_volunteer`
  (`reporting.rs:1539-1545`).
- **Dynamic guard on Absence** (phase 8.4 / CR-01, WR-01):
  Absence hours (extra_hours + derived) reduce expected **only** when
  `working_hours_for_week > 0`. Otherwise, expected would go negative
  on a dynamic contract (`reporting.rs:1378-1386, 1390-1398`).
- **Additive merge Extra Hours + Absence-derived** (phase 8.4, D-01):
  Both sources are summed per week. Converted Extra Hours are
  previously marked as deleted via `soft_delete_bulk`, so no double
  counting (reference `reporting.rs:731-745`).
- **`by_week` as single source of truth** (UV-05, D-18-04):
  From phase 18 the top-level `vacation_hours`/`sick_leave_hours`/
  `holiday_hours`/`unpaid_leave_hours`/`volunteer_hours` are filled by
  summing over `by_week` — the old yearly lumps have been removed
  (`reporting.rs:861-874`). No double counting can sneak in this way.

---

## 3. Data model

The reporting layer itself writes **no** data. It is a pure aggregate
over other aggregates.

### Where does "actual" come from?

| Aggregate | From | Field |
| --- | --- | --- |
| `shiftplan_hours` (per weekday) | `shiftplan_report_service.extract_shiftplan_report` → aggregates `bookings` × `slots` into `ShiftplanReportDay` | per day / week / Sales Person |
| `extra_hours` (`ExtraWork` and custom with `modifies_balance=true`) | `extra_hours_service.find_by_sales_person_id_and_year_range` → table `extra_hours` | `amount`, `category`, `date_time` |

### Where does "expected" come from?

| Aggregate | From | Field |
| --- | --- | --- |
| Contract weekly hours | `employee_work_details_service.find_by_sales_person_id` → table `employee_work_details` | `expected_hours`, `workdays_per_week`, `is_dynamic`, `cap_planned_hours_to_expected`, `monday…sunday`, `from_(year|calendar_week|day_of_week)`, `to_(…)`, `vacation_days`, `holiday_hours()` (derived) |
| Absence reduction (range-based, v1.0+) | `absence_service.derive_hours_for_range` → table `absence_period` | `date → ResolvedAbsence { hours, category (Vacation/SickLeave/UnpaidLeave) }` |
| Absence reduction (legacy, single-day) | via `extra_hours` (categories with `ReportType::AbsenceHours`) | `amount`, `category`, `date_time` |
| Public-holiday reduction (manual) | via `extra_hours` with category `Holiday` | see above |
| Public-holiday reduction (derived) | `special_day_service.get_by_week` (table `special_day`) → `build_derived_holiday_map` with cutover date from `toggle_service.get_toggle_value("holiday_auto_credit")` | per week, filtered by contract weekday |

### Where does "Carryover" come from?

- `carryover_service.get_carryover(sales_person_id, year - 1)` →
  tables `employee_yearly_carryover` (`carryover_hours`) and
  `employee_yearly_vacation_carryover` (`vacation`, i32).

### Migrations that directly affect the reporting read

Chronologically:

- `20241020064536_add-special-day-table.sql` — Holiday/ShortDay table
  (basis for `SpecialDayService`).
- `20241215063132_add_employee-yearly-carryover.sql` — basis for
  `carryover_service.get_carryover(...).carryover_hours`.
- `20241231065409_add_employee-yearly-vacation-carryover.sql` — basis for
  `.vacation` (vacation-day Carryover).
- `20250413073750_add-custom-extra-hours-table.sql` — custom categories,
  integrated into reporting since v1.x.
- `20250418200122_insert-custom-column-to-extra-hours.sql` — links
  `extra_hours` with custom categories.
- `20260428101456_add-logical-id-to-extra-hours.sql` — Logical-ID for
  soft-delete/replace semantics.
- `20260502170000_create-absence-period.sql` — range-based Absence
  aggregate (v1.0), source for `derive_hours_for_range`.
- `20260517120000_add-day-fraction-to-absence-period.sql` — day
  fractions for Absence-derived hours.
- `20260628000001_seed-holiday-auto-credit-toggle.sql` — toggle row for
  phase 25 holiday auto credit.
- `20260707000001_add-source-column-to-extra-hours.sql` — Phase 54
  (milestone v2.6) marker column `extra_hours.source TEXT NOT NULL
  DEFAULT 'manual'` (values: `manual` \| `rebooking`). See feature
  [F14](./F14-rebooking.md) for the full rule set. **Reader impact:**
  Balance-chain aggregates in `service_impl/src/reporting.rs` and
  its downstream consumers will filter `source = 'manual'` from
  Phase 55 onward — the first live consumer is
  `voluntary_ist_total_for_year(..)` (Plan 54-03). In Phase 54 no
  writer sets `rebooking`, so every existing row continues to enter
  the Balance identically (backfill via column DEFAULT).

Reporting itself writes into **none** of these tables.

### Relationships

```
                       ┌───────────────────┐
                       │  employee_work_   │  (contract rows per range)
                       │  details          │
                       └────────┬──────────┘
                                │  expected_hours, is_dynamic, cap, workdays
                                ▼
sales_person ─────► ReportingService ◄───── extra_hours (WorkingHours + AbsenceHours + Custom)
                        │      ▲            └─ cutover split with …
                        │      │
                        │      ├───── absence_period (range Absence, derived hours)
                        │      │
                        │      ├───── booking + slot (actual via ShiftplanReportService)
                        │      │
                        │      ├───── special_day + holiday_auto_credit toggle
                        │      │      (derived public holidays)
                        │      │
                        │      └───── employee_yearly_carryover +
                        │             employee_yearly_vacation_carryover
                        ▼
                 EmployeeReport / ShortEmployeeReport
                        │
                        ▼
                    REST /report/…
                        │
                        ▼
                 Frontend (Employee, WeeklySummary)
```

The Block area (`Block`, `MyBlock`, `BlockReport`) is a *different*
compute path: it aggregates `Booking + Slot` into consecutive time slices
per weekday — completely without expected/Absence calculation. Both
areas only share the source data (bookings, slots, special days).

---

## 4. Service API

### 4.1 `ReportingService` — Business-Logic-Tier

Trait: `service::reporting::ReportingService`
(`service/src/reporting.rs:366-438`).

```rust
#[async_trait]
pub trait ReportingService {
    type Context: …;
    type Transaction: dao::Transaction;

    async fn get_reports_for_all_employees(
        &self, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_report_for_employee(
        &self, sales_person_id: &Uuid, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_report_for_employee_range(
        &self, sales_person_id: &Uuid,
        from_date: ShiftyDate, to_date: ShiftyDate,
        include_carryover: bool,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeReport, ServiceError>;

    async fn get_week(
        &self, year: u32, week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShortEmployeeReport]>, ServiceError>;

    async fn get_employee_weekly_statistics(
        &self, sales_person_id: &Uuid,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWeeklyStatistics, ServiceError>;

    async fn get_employee_attendance_statistics(
        &self, sales_person_id: &Uuid, year: u32, until_week: u8,
        context: Authentication<Self::Context>, tx: Option<Self::Transaction>,
    ) -> Result<Option<EmployeeAttendanceStatistics>, ServiceError>;
}
```

### 4.2 Auth gate table

Binding rules (`Authentication::Full` is the bypass used only by
internal aggregates):

| Method | Gate | Note |
| --- | --- | --- |
| `get_reports_for_all_employees` | `HR_PRIVILEGE` (early, before any read) | REST handler calls with user context. Iterates internally with `Authentication::Full`, so each sub-read does not need a user context. |
| `get_report_for_employee`, `get_report_for_employee_range` | **or-gate**: `HR_PRIVILEGE` OR `verify_user_is_sales_person(id, ctx)` (reporting `join!`, see `reporting.rs:691-700`) | An employee may pull their own report. |
| `get_week` | Auth check delegated to `employee_work_details_service.all_for_week(…, ctx, …)` | User context is passed through; internal reads afterwards use `Full`. |
| `get_employee_weekly_statistics` (A-22-1) | `HR_PRIVILEGE` as the **first** instruction (STAT-01/D-22-05) | No data fetch before auth. HR-only. |
| `get_employee_attendance_statistics` (RPT-01/v2.2) | `HR_PRIVILEGE` as the **first** await (D-AVG-05) | Also HR-only. From v2.2 post-ship the filter applies to **all** employees (is_dynamic filter removed, see `reporting.rs:1207-1210`). |

**`ToggleService` read for `holiday_auto_credit`:** performed internally
with the passed `context` (NOT `Full`!). `Unauthorized` is interpreted
as "automation off" — that is the legacy off branch and prevents
mock/internal callers from making the toggle fail
(`reporting.rs:163-172`). See also
[edge-cases §6.1 (Full bypass)](../domain/edge-cases.md#61-authenticationfull-bypass)
and the `ToggleService` Full-Context fix in `service_impl/src/toggle.rs`.

### 4.3 Transaction behaviour

`ReportingService` has `TransactionDao` as a dep, but **none of the
public methods opens or commits a transaction itself**. They accept
`Option<Transaction>` and forward it to all sub-aggregates (`tx.clone()`).
If the consumer passes `None`, each sub-service works in its own
implicit transaction (or the respective `use_transaction` call opens
one).

For snapshot creation (Billing Period Report, see F08) this is critical:
if a report is computed there under a running transaction, *that*
transaction runs through all sub-reads too (read-consistency set). The
reader **does not commit** anything itself.

**[To verify]** whether the `Authentication::Full` sub-reads also
respect the `tx` handover (spot check confirms `tx.clone()` everywhere);
in particular the parallel `join!` in `reporting.rs:691-700` runs inside
the same transaction bracket.

### 4.4 Dependencies

`ReportingServiceDeps` (`service_impl/src/reporting.rs:61-82`):

- Basic-Tier consumers: `ExtraHoursService`, `ShiftplanReportService`,
  `EmployeeWorkDetailsService`, `SalesPersonService`, `CarryoverService`,
  `PermissionService`, `ClockService`, `UuidService`, `SpecialDayService`,
  `ToggleService`, `TransactionDao`.
- Business-Logic consumer: `AbsenceService` (also business-logic, but
  in a disjoint sub-domain — no cycles). Absence-derived hours are
  consumed under the additive merge model (D-01, phase 8.4).

Classified per the CLAUDE.md convention:
`ReportingService` is **Business-Logic-Tier**, because it reads across
multiple aggregates and maintains cross-entity invariants (balance
formula).

### 4.5 `BlockService` — Basic-Tier for time slices

Trait: `service::block::BlockService` (`service/src/block.rs:70-123`).

The `Block` is **not persisted** — a pure read aggregate: consecutive
`Booking + Slot` pairs on the same weekday with `slot_prev.to ==
slot_next.from` are merged into a block
(`service_impl/src/block.rs:150-215`).

Methods:

| Method | Purpose | Auth |
| --- | --- | --- |
| `get_blocks_for_sales_person_week` | Blocks for a person in a calendar week | delegates to `SalesPersonService.get(…)` |
| `get_blocks_for_next_weeks_as_ical` | iCal string over the next 12 weeks (backwards −2 → +10) | internally `Authentication::Full` |
| `get_unsufficiently_booked_blocks` | Blocks whose summed `min_resources` is not covered by bookings | `context` passed through |
| `get_blocks_for_current_user` | For the currently logged-in user over a ShiftyWeek range | delegates to `sales_person_service.get_sales_person_current_user(ctx, …)` |

**Phase 51 (D-51-06 Chain A' + D-51-07 cutover gate):** Before the
merge loop, `clip_slot_for_week` (`shortday_gate.rs`) runs — a ShortDay
clip per slot, gated by the `shortday_gate_active_from` toggle
(via `shortday_gate::read_active_from`; `Unauthorized → None` = legacy
off). Order is critical: clip first, then the `slot.from == to` merge,
otherwise consecutive detection shifts.

### 4.6 `BlockReportService`

Trait: `service::block_report::BlockReportService`.

Takes a `template_id` (references a `TextTemplate`), loads three weeks
(`current`, `next`, `week_after_next`), filters only future blocks
(`is_block_in_future` against `clock_service.date_time_now()`), and
renders either with **Tera** or **MiniJinja** into an `Arc<str>`
(`service_impl/src/block_report.rs:178-228`).

Context variables in the template:
- `current_week_blocks`, `next_week_blocks`, `week_after_next_blocks`
  (as `SimpleBlock` list, see `service_impl/src/block_report.rs:19-42`),
- `unsufficiently_booked_blocks` (aggregated over the three weeks),
- `current_(week|year)`, `next_(week|year)`, `week_after_next_(week|year)`.

Auth: `HR_PRIVILEGE` as the first instruction.

### 4.7 `MyBlockService`

Trait: `service::my_block::MyBlockService`.

**[To verify]** Although the trait exists, no `MyBlockServiceImpl` is
found in the repo (`grep -rn "MyBlockService" service_impl/` returns 0
hits). The REST handler `rest/src/my_block.rs:52-53` instead calls
`rest_state.block_service().get_blocks_for_current_user(…)` directly.
The `MyBlockService` trait is therefore currently **unused** —
presumably a historical artefact from before the method moved into
`BlockService`, or not yet cleanly removed. Reference per grep result:
`service_impl/src/…` (no hit for `MyBlockService`).

---

## 5. REST endpoints

All reporting endpoints are mounted under `/report/`
(`rest/src/lib.rs:650`). Block-related ones under `/blocks/` and
`/block-report/`.

| Method | Path | Description | DTO in | DTO out | Key errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/report/?year=…&until_week=…` | Short report of all paid employees in the year up to week N | Query: `ReportRequest { year, until_week }` | `Vec<ShortEmployeeReportTO>` | 401, 403, 500 |
| `GET` | `/report/{id}?year=…&until_week=…` | Full employee report incl. `by_week` | Path: `Uuid`, query see above | `EmployeeReportTO` | 401, 403 (neither HR nor self), 500 |
| `GET` | `/report/week/{year}/{calendar_week}` | Short report of all persons in ONE calendar week | Path: `(year, week)` | `Vec<ShortEmployeeReportTO>` | 500 |
| `GET` | `/report/{id}/weekly-statistics` | Avg worked hours/week (current year up to current calendar week) | Path: `Uuid` | `EmployeeWeeklyStatisticsTO` | 403 HR-only, 500 |
| `GET` | `/report/{id}/attendance-statistics?year=…&until_week=…` | Per-weekday attendance distribution (7 entries Mo..Su) | Path + query | `Option<EmployeeAttendanceStatisticsTO>` (currently always `Some`) | 403 HR-only, 500 |
| `GET` | `/blocks/{from_year}/{from_week}/{until_year}/{until_week}` | Blocks of the current user in the range | Path 4× | `Vec<BlockTO>` | 401, 403, 500 |
| `GET` | `/block-report/{template_id}` | Renders template with the 3-week blocks | Path: `Uuid` | `text/plain` (String) | 401, 403 HR-only, 404, 500 |

DTOs (wire format):

- `ShortEmployeeReportTO` (`rest-types/src/lib.rs:371-393`): compact
  row for the HR overview.
- `EmployeeReportTO` (`rest-types/src/lib.rs:523-596`): full report;
  contains `by_week: Arc<[WorkingHoursReportTO]>` and
  `by_month: Arc<[…]>` (currently always empty, `reporting.rs:877` writes
  `Arc::new([])` — [To verify] whether ever used).
- `WorkingHoursReportTO` (`rest-types/src/lib.rs:459-520`): one weekly
  row with day split.
- `EmployeeWeeklyStatisticsTO`, `EmployeeAttendanceStatisticsTO`,
  `WeekdayAttendanceTO`: A-22-1 / RPT-01 aggregates.
- `BlockTO` (`rest-types/src/lib.rs:1603-…`).

---

## 6. Frontend integration

The frontend lives entirely in `shifty-backend/shifty-dioxus/`. The
important point: the **entire** balance computation runs in the backend.
The frontend is a pure view layer — it reads the DTOs and renders.

### 6.1 Pages

- **`page/report.rs`** — HR overview: `GET /report/?year=…&until_week=…`.
- **`page/employee_details.rs`** (via `Employee` aggregate, loader
  `loader.rs:294`): `GET /report/{id}?year=…&until_week=…`.
- **`page/weekly_overview.rs`** — personal weekly overview with diff
  colour (`text-good`/`text-warn`, see `diff_color_and_sign`). Uses the
  `WeeklySummary` state.
- **`page/my_shifts.rs`** — personal block view of the upcoming weeks:
  `GET /blocks/{from_year}/{from_week}/{until_year}/{until_week}`.
  Formats hours with `format_hours_norm(hours, 1)` to avoid `-0.0`
  display.

### 6.2 Services / loader

- `shifty-dioxus/src/api.rs` — wrappers `get_short_reports`,
  `get_employee_reports`, `get_working_hours_for_week`,
  `get_balance_until_week` (see `loader.rs:267,294,360,363`).
- `shifty-dioxus/src/state/weekly_overview.rs` — `WeeklySummary` with
  the derived numbers (already `f32` from the TO).

### 6.3 i18n keys (reporting-relevant, from `weekly_overview.rs`)

`WeekLabel`, `PaidCommittedVolunteer`, `AvailableRequiredHours`,
`MissingHours`, `HoursShort` — each to be maintained in all three
locales (En, De, Cs) (see CLAUDE.md rule).

### 6.4 Dioxus.toml proxy

The reporting endpoints have existed in the proxy since v1.0. New
sub-paths (e.g. `/report/{id}/weekly-statistics`,
`/report/{id}/attendance-statistics` in v2.2) are reachable via the
generic `/report/{**}` forward — **[To verify]** whether the concrete
wildcard rule in the proxy actually forwards the sub-segments. See
[edge-cases §11 frontend-backend coupling](../domain/edge-cases.md#11-frontend-backend-kopplung)
and the memory note "Dioxus.toml proxy for new backend endpoints".

---

## 7. Edge cases

**Central references — mandatory reading before any change to reporting:**

- [edge-cases §1 hour account](../domain/edge-cases.md#1-stundenkonto)
  — Carryover boundary, contract change, Sales Person time bounds,
  Special Days, balance perimeter.
- [edge-cases §2 Absence & Extra Hours](../domain/edge-cases.md#2-absence--extra-hours)
  — cutover split, range edge cases, legacy delete semantics.
- [edge-cases §5 rounding & precision](../domain/edge-cases.md#5-rundung--genauigkeit)
  — f32 precision, associativity, display-vs-persistence rounding.

### 7.1 Feature-specific edges of reporting

- **Rounding consistency between display and sum.**
  The frontend shows weekly values with one decimal place
  (`format_hours(hours, 1)`, `my_shifts.rs:44`). The backend computes
  in `f32` and aggregates weeks into yearly sums *in the backend*
  before rounding. If a client re-sums the rounded weekly values, the
  displayed sum deviates from the backend balance. → **Rule:** Always
  display the backend total, never re-add rounded weekly values in the
  client. See
  [edge-cases §5](../domain/edge-cases.md#5-rundung--genauigkeit).

- **Cross-period consistency (live vs snapshot).**
  `EmployeeReport` is **always** a live read. If a `billing_period`
  snapshot with a specific `snapshot_schema_version` exists in parallel
  (see F08), the live report and the snapshot for the same period can
  differ from each other — legitimate if rule/category changes have
  happened in between. **Always** read the version together with the
  snapshot (validator pattern). See
  [edge-cases §3.1/3.3](../domain/edge-cases.md#3-billing-period--snapshots).

- **`by_month` is empty.**
  The struct has `by_month: Arc<[GroupedReportHours]>`, but the reader
  currently writes `Arc::new([])` (`reporting.rs:877`). The frontend
  ignores it accordingly. **[To verify]** whether ever activated or
  should be removed.

- **Range report + `include_carryover=false`.**
  `get_report_for_employee_range` with `include_carryover=false` returns
  `carryover_hours=0.0` and `vacation_carryover=0` — the consumer must
  know that the balance then does *not* contain the previous-year
  Carryover. Used, among other things, for sub-periods in which the
  Carryover has already been accounted for externally.

- **Dynamic contract + Absence in the same week.**
  Tested (`reporting.rs:1615-1707`). Core statement: with
  `is_dynamic == true`, the balance always stays 0 (expected = actual),
  even with Vacation extras. The reason lies in the doubled
  `if working_hours_for_week <= 0.0`-guard chain (lines 1378-1414,
  1097-1122).

- **Cap + holiday auto credit at the same time (HSP-03 band guard).**
  If Cap is active and a derived holiday falls into the same week, the
  holiday MUST NOT enter the cap baseline. Otherwise the holiday delta
  would leak as `auto_volunteer_hours` into the volunteer bands of
  `booking_information` (violet D-25-08). The guard sits in
  `reporting.rs:1113-1122`: `expected_hours_for_cap = planned − absence −
  absence_derived_balance` (without holiday), then apply_cap, then
  `expected_hours = expected_hours_for_cap − holiday_derived_gated`.

- **`vacation_days()` divisor with `workdays_per_week == 0`.**
  The getters `hours_per_day()`, `hours_per_holiday()`, `vacation_days()`
  etc. on `GroupedReportHours` explicitly protect against div-by-0
  (`service/src/reporting.rs:105-145`). Important because dynamic
  contracts can deliver `workdays_per_week=0.0`.

- **Calendar week 53 in non-53 years.**
  `until_week.min(time::util::weeks_in_year(year as i32))` in
  `get_report_for_employee` (`reporting.rs:664`) — clamped to the
  actual week count. **Important:** the REST query `until_week=53` in
  a 52-week year is silently reduced to 52.

- **`get_reports_for_all_employees` iterates `additional_weeks=1` at
  year boundaries** (`reporting.rs:317-321`) to correctly capture weeks
  with `iso_week == 1` in the following year / `iso_week == 53` in the
  previous year. Historical reason: ISO 8601 weeks straddle year
  changes. If you touch this loop, check the A-22-1 tests for
  year-change boundary cases (`reporting_avg_weekly.rs`).

- **Blocks — empty bookings set.**
  If no bookings exist for a person + week,
  `get_blocks_for_sales_person_week` returns `Arc::from([])` (empty,
  no error). The `MyShifts` page must handle the empty state.

- **iCal — `sales_person_id == Uuid::nil()`.**
  Historical convention: `nil()` → *only* unstaffed/understaffed blocks
  in the next 12 weeks. See `block.rs:232-249`.

- **RPT-01 (`weekday_attendance_distribution`) — empty denominator.**
  `counted_calendar_weeks == 0` → all `share=0.0`, no NaN
  (`reporting.rs:322-328`). Rounding: two decimals via
  `(x * 100.0).round() / 100.0`. Note: `share_of_hours` sums exactly
  to 1.0 when `total_hours > 0`, otherwise 0.0
  (`reporting.rs:330-335`).

- **A-22-1 (`average_worked_hours_per_week`) — empty "included" set.**
  Fully-absent weeks (`worked==0 && absence>0`) fly out of the
  denominator (`reporting.rs:225-231`). If *all* weeks are fully absent
  → avg=0.0, `included_weeks=0` — the consumer must handle this on the
  UI side.

---

## 8. Tests

Most extensive test area in the codebase.

### 8.1 Reporting

- `service_impl/src/test/reporting_additive_merge.rs` (1456 lines) —
  core regression suite for the Absence-derived + Extra Hours merge.
  Covers no-contract, dynamic, cap, custom categories, cross-week
  Absence, and the gap fixes from phase 8.4 (M-02/CR-01/WR-01).
- `service_impl/src/test/reporting_holiday_auto_credit.rs` (929 lines) —
  phase 25: `holiday_auto_credit` toggle, cutover gate, manual-wins,
  4×injection points (`hours_per_week` / all_employees / range /
  `get_week`), HSP-03 band guard.
- `service_impl/src/test/reporting_no_contract_volunteer.rs` (447 lines)
  — user rule `quick-260624-ujk`: no-contract week → volunteer work.
- `service_impl/src/test/reporting_cap_overflow.rs` (297 lines) — weekly
  cap: excess → `auto_volunteer`, correct with Absence & Extra Work.
- `service_impl/src/test/reporting_avg_weekly.rs` (175 lines) — A-22-1
  pure formula + service wrapper, HR gate first.
- `service_impl/src/test/reporting_weekday_attendance.rs` (251 lines) —
  RPT-01 v2.2: distinct-date dedup, filter (Shiftplan/ExtraWork/VolunteerWork),
  `share_of_hours` sum = 1.0.
- `service_impl/src/test/reporting_attendance_gate.rs` (318 lines) —
  HR gate + is_dynamic filter (in v2.2 post-ship switched to "for all",
  see `reporting.rs:1207-1210`).
- `service_impl/src/test/reporting_phase2_fixtures.rs` (141 lines) —
  fixtures for the early phase-2 regression.
- Inline tests in the impl itself: `test_dynamic_vacation_days`,
  `test_unpaid_leave_*` (`reporting.rs:1553-1849`),
  `test_weekly_planned_hours_cap` (from `reporting.rs:1852`).

### 8.2 Blocks

- `service_impl/src/test/block.rs` (1045 lines) — merge logic, ShortDay
  clipping (phase 51), unavailable blocks, iCal export round-trip,
  insufficient-booking detection.
- `service_impl/src/test/block_report.rs` (259 lines) — Tera/MiniJinja
  template rendering, future-only filter, HR gate.

### 8.3 Known gaps

- **[To verify]** Retroactive contract change + live report vs
  Carryover drift — no explicit regression test.
- **[To verify]** `by_month` field (currently always empty) — no test
  because no behaviour.
- **[To verify]** `MyBlockService` trait without impl — dead code,
  should be removed or implemented.
- **[To verify]** DST switch (March/October) in blocks across the
  overnight hours — see [edge-cases §4](../domain/edge-cases.md#4-zeit--zeitzone).

---

## 9. History & context

Reporting is **the oldest continuously productive feature** of Shifty
and has correspondingly many rewrites behind it. The most important
milestones:

- **v0.x (before v1.0):** Original report with `extra_hours` as the only
  Absence source, balance in weekly mode. `EmployeeReport`,
  `ShortEmployeeReport`, `GroupedReportHours` were already defined here
  as the basic structure.
- **v1.0:** Cutover to the range-based `AbsenceService`
  (`.derive_hours_for_range`). The legacy path via `extra_hours`
  **coexists** — converted legacy rows are marked via `soft_delete_bulk`
  so no double counting occurs.
- **Phase 8.4 (D-01, CR-01, WR-01, M-02, M-03):** Additive merge —
  Absence-derived is unconditionally added, feature-flag switch removed.
  Symmetric dynamic guards on both Absence contributions. Fix for the
  dynamic-contract balance asymmetry.
- **Phase 15 (D-01 "report-ehrenamt-gesamtstunden"):** `overall_hours` /
  `balance_hours` / `shiftplan_hours` use exclusively the per-week
  capped `shiftplan_hours_by_week` — the raw value no longer leaks.
- **Phase 17 (D-06, CVC-10):** `is_paid=false` (unpaid volunteers)
  are excluded from `paid_hours` / `WorkingHoursPerSalesPerson` /
  year summary. Both `get_reports_for_all_employees` and `get_week`
  now filter `if !sales_person.is_paid.unwrap_or(false) { continue; }`.
- **Phase 18 (UV-05, D-18-03/04/05):** `by_week` becomes the single
  source for top-level category sums. Old yearly lumps removed.
  Display-vs-balance split (display ungated, balance gated).
- **Phase 25 (HOL-01/02, HCFG-01/03):** Holiday derive-on-read via the
  `holiday_auto_credit` toggle. Four injection points, manual-wins,
  cutover gate, HSP-03 band guard.
- **Phase 34 (HSP-01/02, D-34-01):** 4th injection point in `get_week`.
- **v2.2 (RPT-01):** Per-weekday attendance distribution — replaces the
  earlier scalar avg-hours metric with count + share per weekday with
  `share_of_hours` (v2.2.1).
- **A-22-1 (STAT-01, D-22-05, D-22-06):** Weekly statistics — HR-only,
  avg excl. fully-absent weeks. Reused per-week data from
  `get_report_for_employee`.
- **Phase 47 (D-47-BE):** `EmployeeAttendanceStatistics` reshape to a
  7-weekday array + `counted_calendar_weeks`.
- **Phase 51 (D-51-06/07):** ShortDay cutover per weekday + cutover
  gate in `BlockService`. `ToggleService` Full-Context bypass for
  internal aggregate callers (see memory note).
- **Phase 52 (WOP-01/02, D-52-06/D-52-08):** Additive batch trait
  `ReportingService::get_year(year)` and
  `ShiftplanReportService::extract_shiftplan_report_for_year(year)` /
  `ExtraHoursService::find_by_year(year)`. Balance formula, CVC-06 cap
  semantics, `is_paid` filter, and the `assemble_weeks` per-week
  aggregation body are unchanged. Byte-identity between batch and
  single-week paths is structurally guaranteed through the shared
  `pub(crate) assemble_weeks` helper. Consumer:
  `BookingInformationServiceImpl::get_weekly_summary` now uses these
  bulk loads to replace ~55 sequential service calls with 7 constant
  bulk loads (byte-identical, ~2× latency reduction on Dev-DB).

### Fat-backend principle

The entire balance computation sits in the backend. The frontend takes
the DTOs (`ShortEmployeeReportTO`, `EmployeeReportTO`,
`WorkingHoursReportTO`) and just renders — not a single category summand
is recomputed on the FE. Reason: second-client capability (mobile app
etc.) without duplication of domain rules. See
[fat-backend memory note](../../CLAUDE.md) and the recommendation to
anchor the principle in every discuss-phase as a default.

### Relationship to Billing Period (F08)

`EmployeeReport` is the **live computation**.
`billing_period_sales_person` persists the same categories as a
Snapshot with a `snapshot_schema_version` (currently **12**, see
`billing_period_report.rs:117`). The snapshot writer consumes
`EmployeeReport` and writes a series of `value_type` rows. Details see
[F08 Billing Period](./F08-billing-period.md).

**Rule for reporting changes:** Any change to the computation of a
snapshot-persisted `value_type` (formula, inputs, filter) MUST bump the
`CURRENT_SNAPSHOT_SCHEMA_VERSION` — otherwise the validator drifts
(see the CLAUDE.md section "Billing Period Snapshot Schema
Versioning").

---

*Last verification against code:* see git blame of this file.

---

**Summary:** Reporting is the output layer that pours all
balance-relevant aggregates (bookings, extra hours, Absence, Carryover,
Special Days, toggles) into a single formula — with many guards grown
over the years for cap overflow, dynamic contract, no-contract week,
and holiday auto credit. Whoever changes something here first checks
the snapshot version (F08) and the edge-case reference, otherwise the
live report drifts away from the persisted truth.
