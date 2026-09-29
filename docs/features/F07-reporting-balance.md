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
- `service_impl::reporting` (3261 lines, business-logic-tier),
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
  no volunteer redirect. **Exception since phase 57 (CR-03):**
  rebooking-tagged WorkingHours rows (`source == Rebooking`) are removed
  from this branch's expected — otherwise the `+h` ExtraWork leg of a
  rebooking pair raises `overall` and `expected` at the same time (balance
  stays neutral) while the `−h` VolunteerWork leg does lower
  `volunteer_hours`: `h` hours would vanish. The adjustment affects
  **only** expected — `overall_hours` and `balance` count the row
  normally (VOL-ACCT-04). Symmetric to the `availability()` exception in
  `assemble_weeks`. See the three-case distinction in
  `hours_per_week` (`reporting.rs:1444-1454`) and the parallel logic in
  `get_reports_for_all_employees` (`reporting.rs:388-500`) and
  `get_week` (`reporting.rs:1006-1156`).
- **Volunteer merge:** `volunteer_hours` of a week is
  `manual_volunteer + auto_volunteer (cap) + no_contract_volunteer`
  (`reporting.rs:1539-1545`). **Since phase 57 (WR-01) this holds in all
  three read paths.** The no-contract and dynamic branches of
  `get_reports_for_all_employees` used to build the value from shiftplan
  quantities alone and never summed manual `VolunteerWork` rows, while
  the `+h` ExtraWork leg of a rebooking pair *was* counted through the
  category filter — the same hours counted twice in the no-contract
  branch, and in the dynamic branch the pair disappeared entirely
  (`planned_hours = overall_hours`). Both branches now mirror
  `hours_per_week` / `assemble_weeks`, including the CR-03 expected
  adjustment for rebooking-tagged ExtraWork in the dynamic branch.
  Pinned by
  `rebooking_effectiveness.rs::rebooking_conserves_hours_in_summary_without_contract_row`
  and `::rebooking_effective_in_summary_on_dynamic_contract`.
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
- **Rebooking aggregation — category semantics, not the marker**
  (VOL-ACCT-04, D-57-06, since Phase 57, milestone v2.7): the
  `extra_hours` read aggregates do **not** filter by provenance. What a
  row contributes is decided by its category:
  `ExtraHoursCategory::as_report_type()` decides balance relevance
  (`ExtraWork` → `WorkingHours`, counts into the paid balance;
  `VolunteerWork` → `Documented`, does not), and
  `ExtraHoursCategory::availability()` decides presence. An
  equal-and-opposite rebooking pair `(−h VolunteerWork, +h ExtraWork)`,
  both stamped `source = 'rebooking'`, therefore **raises the balance by
  `h` and lowers the voluntary Ist by `h`** — that is the point of the
  transfer, not an accident. Absence/Holiday aggregates are unaffected
  (rebooking rows never land there — `rebook()` only writes
  `VolunteerWork`/`ExtraWork`).

  **The single remaining source-aware site** is `assemble_weeks`
  (`service_impl/src/reporting.rs`, the weekly-summary path): its
  balance-relevant `extra_working_hours` sum is built over
  `availability()`, and `VolunteerWork.availability()` is `Available`.
  There the conjunction `VolunteerWork` **and** `source = Rebooking` is
  excluded, because otherwise the `−h` leg would cancel the lift of the
  `+h` leg and the pair would net to zero at that one site. Ordinary
  (non-rebooking) `VolunteerWork` rows still count there unchanged —
  that is the pre-existing weekly-vs-annual inconsistency (RISK-01),
  deliberately left alone by Phase 57. In `manual_volunteer_hours` of
  the same function the rebooking row stays in on purpose: it is what
  lowers the voluntary Ist.

  **What is still neutral:** only the VOL-ACCT-01 **Soll**. It comes
  from `committed_voluntary` on the contract and reads no `extra_hours`
  at all, so it is bit-identical across a rebooking
  (`test_neutrality_f2_soll`). The delta promise — balance `+h`,
  voluntary Ist `−h` — is nailed by `rebooking_raises_balance` /
  `rebooking_lowers_volunteer_ist`
  (`service_impl/src/test/rebooking_effectiveness.rs`),
  `rebooking_effective_single_manual`
  (`service_impl/src/test/rebooking_reconciliation.rs`), the
  weekly-summary guard `rebooking_effective_in_weekly_summary` for the
  `assemble_weeks` site, and the end-to-end reference scenario in
  `service_impl/src/test/rebooking_reference_scenario.rs`
  (8 h paid / 12 h voluntary / balance −2 → 10 / 10 / 0). These are the
  standing CI-guards: they turn red as soon as a rebooking pair stops
  moving the balance.

  **Snapshot consequence:** because rebooking rows now reach the
  persisted values, `CURRENT_SNAPSHOT_SCHEMA_VERSION` was bumped
  **12 → 13** in Phase 57 (D-57-10) — the Phase-56 statement "stays at
  12" is void. See F08 §7.4.

  **Historical note.** Up to Phase 56 an unconditional marker filter
  (`source == ExtraHoursSource::Manual`, D-55-A1) sat at the aggregation
  entry of all four report paths (`get_week`,
  `get_reports_for_all_employees`, `hours_per_week`,
  `overall_extra_work_hours`) and was justified with "double counting".
  It derived from the original wording of VOL-ACCT-03, which demanded
  balance neutrality — and that requirement was itself the bug: the
  filter made the entire rebooking transfer ineffective for a whole
  milestone. Re-drafted in Phase 57 (VOL-ACCT-04). See feature
  [F14 §3 / §8 / §9](./F14-rebooking.md).

### 2.7 Planning target vs balance-relevant expected (Phase 64, PSOLL-01…04)

Since Phase 64 `ShortEmployeeReport` carries **two** target ("Soll")
figures. They look alike, they are not interchangeable, and confusing
them either re-introduces the defect this phase fixed or breaks the
availability computation.

| | `expected_hours` — *balance-relevant Soll* | `planning_target_hours` — *planning target* |
| --- | --- | --- |
| What it is for | the expected leg of `balance = worked − expected + carryover` | the figure a planner compares the assigned shifts against |
| Dynamic contract (`is_dynamic == true`) | deliberately **`0.0`** — *expected = actual*, the balance must stay `0` | deliberately **`> 0`** — the reduced commitment (contract hours **plus** committed volunteer hours) |
| Contract with `expected_hours == 0` and commitment > 0 | `0.0` | **`> 0`** — since plan 64-05; the original gate returned `0.0` here |
| Non-dynamic contract, **commitment 0** | contract hours minus absence and holiday reductions | **the same number**, structurally (see below) |
| Non-dynamic contract, **commitment > 0** | as above, without volunteer hours | **larger** — by exactly the absence- and holiday-reduced commitment |
| Includes the committed volunteer hours (`committed_voluntary`) | **no — never** | **yes** [D-64-05] |
| Enters balance / Carryover / Billing snapshot / `available_hours` | **yes** | **no — never** |
| Where it is visible | the hour-account surfaces (employee detail, weekly histogram, employee list) | the overview below the shiftplan only (§6.1) |

**What the field is.** `planning_target_hours` is the holiday- and
absence-reduced **commitment** of the week
(`service/src/reporting.rs:164`, mirrored on `ShortEmployeeReportTO`,
`rest-types/src/lib.rs:392`, `#[serde(default)]`). It answers *"how many
hours should this person be scheduled for"* — not *"how many hours does
the balance expect"*.

**It is explicitly NOT a contract figure** [D-64-05, plan 64-05]. The
purpose of the field is shift planning: the planner needs to see *what
the person commits to for the week*. Whether those hours are paid is
irrelevant for that — which is why the committed volunteer hours
(`employee_work_details.committed_voluntary`) flow in. The strict
separation of `expected_hours` and `committed_voluntary` holds for the
**balance** and stays untouched there; it does not hold for **planning**.
Anyone who pulls the commitment back out here "because the balance does it
that way too" breaks the purpose of the field.

Absences and holidays reduce the volunteer portion **as well** — someone
who is around less during that time also commits less voluntarily. The
reduction is *proportional*, not all-or-nothing (§ D-64-07 below).

**What it is explicitly NOT used for.** Balance, Carryover, the Billing
Period snapshot, and the availability computation all keep reading
`expected_hours` / `dynamic_hours` exclusively.
`WorkingHoursPerSalesPerson.available_hours` still reads
`report.expected_hours` at both of its sites
(`service_impl/src/booking_information.rs:644` and `:818`) and therefore
still evaluates to `0.0` for a dynamic contract — that is intended and
is pinned by `test_available_hours_unchanged_dynamic`.

**The formula (worded identically at both construction sites).**

```rust
// service_impl/src/reporting.rs:1162-1186 (assemble_weeks),
// mirrored verbatim at :1562-1577 (get_reports_for_all_employees)
let planning_base_for_target = dynamic_hours + committed_voluntary_for_target;
let target_reduction_scale = if dynamic_hours > 0.0 {
    planning_base_for_target / dynamic_hours
} else {
    1.0f32
};
let reduce_target = has_contract_row && planning_base_for_target > 0.0;
let planning_target_hours = if reduce_target {
    planning_base_for_target
        - (absence_manual_for_target
            + absence_derived_for_target
            + holiday_derived_for_target)
            * target_reduction_scale
} else {
    0.0f32
};
```

`committed_voluntary_for_target` is the sum of
`committed_voluntary_weight_for_week(year, week, wh)` over the contract
rows active in the week (`reporting.rs:2451`, SUM aggregation following
the D-OVERLAP-AGG precedent).

**Why the field has its own gate, and why plan 64-05 changed it.**
Originally (64-01) it read `has_contract_row && dynamic_hours > 0.0`. That
was correct as long as the base was `dynamic_hours` — with the commitment
in the base it *is* the defect. The three properties the current gate
carries:

- *It must not borrow the balance gate.* The balance branch keys off
  `planned_hours`, element 0 of the `weight_for_week` return tuple, which
  is **forced to `0.0`** whenever `is_dynamic` is true. Its condition
  `!has_contract_row || planned_hours <= 0.0` (`reporting.rs:1045`) is
  consequently true for *every* week of a dynamic contract and would skip
  the reduction exactly where it is needed. The paid portion of the
  planning base is therefore measured against element 1 of the same tuple
  (`dynamic_hours`) — the only value in the chain that stays unforced.
  **Do not "unify" the two conditions back into one.**
- *It fires for the `expected_hours == 0` with commitment > 0 class.*
  This is exactly where the old second half measured the **wrong value**:
  a person with 0 contract hours and a real commitment has
  `dynamic_hours == 0`, so the old gate returned `0.0` for them. Since
  plan 64-05 the measurement is against `planning_base_for_target`, and
  the commitment is part of it. Pinned by
  `planning_target_zero_expected_hours_with_commitment`.
- *It never subtracts from a zero base.* When contract hours AND
  commitment are 0 — or the week lies outside the contract boundaries
  (`relation == 0.0`) — `planning_base_for_target` is exactly `0`, the
  gate does not fire, and the planning target is byte-identical `0.0`
  just like `expected_hours`. Same guarantee as before, now measured
  against the right base. A contract row without a single weekday set
  yields `relation = 0/0 = NaN` (pre-existing in `weight_for_week` since
  v1); `NaN > 0.0` is `false`, so the gate does not fire there either and
  the planning target is `0.0` rather than `NaN`.

**Why byte-identity for non-dynamic contracts follows structurally — but
only at commitment 0.** With `committed_voluntary == 0`,
`committed_voluntary_for_target == 0`, hence
`planning_base_for_target == dynamic_hours`, hence
`target_reduction_scale == 1.0` — which leaves line for line the same
computation 64-01 wrote, and for a non-dynamic contract element 0 and
element 1 of the `weight_for_week` tuple carry the same value. So
`planning_target_hours == expected_hours` by construction, not by
coincidence. Pinned by
`test_planning_target_matches_expected_for_fixed_contract` and by block 3
of `planning_target_fixed_contract_differs_by_commitment`.

**With a commitment the difference is exactly the reduced commitment.**
Measured in `planning_target_fixed_contract_differs_by_commitment` (40 h
contract, commitment 10.0): in an undisturbed week `50.0` against `40.0`,
difference exactly **10.0**; in a week with three absent workdays (24 h)
`20.0` against `16.0`, difference exactly **4.0** (= `10 × (1 − 24/40)`).

**Deliberately no `.max(0.0)`.** The balance path does not clamp either.
The gate prevents the *structural* zero-base case; it does not prevent
reductions from exceeding the base. Since plan 64-05 that is **asserted**
behaviour: `planning_target_not_clamped_when_reductions_exceed_base`
measures a planning target of **−10.0** for a 0-hour contract with
commitment 10.0 and a manual `Vacation` row of 20.0 h. A `.max(0.0)`
returns `0.0` there and breaks that test. Whoever adds a clamp later must
clamp both figures or neither — and must first ask the product question
whether a negative planning target is readable for the planner.

**D-64-06 — the commitment scales with `relation`, not day-based
pro-rata.** Three candidates exist in the code for the weekly amount of
the commitment. At an edge week (contract Mon–Fri, ends Wednesday,
commitment 7.0) they yield three different numbers:

| Candidate | Weekly amount | Verdict |
| --- | --- | --- |
| `committed_voluntary_for_calendar_week` (`reporting.rs:130`) — SUM without scaling | **7.0** | rejected: reports the full weekly commitment for a week the contract only covers three fifths of |
| `committed_voluntary_prorata_for_week` (`reporting.rs:163`) — `committed_voluntary / 7.0` per day (D-F2-01) | **3.0** (3/7) | rejected: mixes calendar-day and workday scaling in one displayed number; the subtracted reduction terms are workday-based |
| **`committed_voluntary_weight_for_week` (`reporting.rs:2451`) — `committed_voluntary * relation`** | **4.2** (3/5) | **chosen**: one scaling for the whole number, same source of `relation` as the paid portion (`potential_workdays_for_week`) |

Pinned by `committed_voluntary_weight_pins_relation_scaling_d_64_06`,
which carries both rejected numbers as explicit counter-assertions.

**Why `committed_voluntary_target_in_range` is NOT reused.** Its absence
semantics are **whole-week-out** (D-54.5-01): an ISO week with *one*
absence day contributes **0** to the target. For the F2 target (volunteer
account) that is right — it is a **balance** figure and establishes
actual/target symmetry. For the planning target it would be wrong: one
vacation day would erase the *entire* weekly commitment while the paid
portion only loses one day. D-64-05 explicitly demands *proportional*,
not all-or-nothing. **Consequence: planning target and F2 target may
differ at edge weeks. That is intended and not drift for someone to "clean
up".**

**D-64-07 — the reduction applies proportionally, via
`target_reduction_scale`.** Plainly subtracting the reduction terms from
the enlarged base would shorten **only the paid portion**: the absence
hours are derived from `expected_hours` (hours per workday), not from
`expected_hours + committed_voluntary`. Example: 40 h contract + 10 h
commitment, one absence day out of five → reduction 8 h → `50 − 8 = 42`,
although the person is only around 4/5 of the week and commits 4/5 of 50 =
**40**. The reduction is therefore scaled up by exactly the factor the
base grew by. Pinned by
`planning_target_commitment_reduced_by_absence_parity` (measured: 50.0 in
the undisturbed week, 20.0 in the absence week; the plain subtraction
would yield 26.0).

**Pre-existing asymmetry, deliberately not touched by Phase 64.** The two
construction sites (§3) derive their *manual* absence raw value through
**different** filters: `assemble_weeks` sums over
`category.availability() == Availability::Unavailable`
(`reporting.rs:917`), whereas `get_reports_for_all_employees` sums over
`category.as_report_type() == ReportType::AbsenceHours`
(`reporting.rs:1450`). The asymmetry predates Phase 64 and already
affects today's **balance** paths; unifying it would have been a change
to the balance and was therefore out of scope. Recorded here so the next
reader does not mistake it for something Phase 64 introduced.

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

| Committed volunteer hours (planning target only) | `employee_work_details.committed_voluntary`, scaled via `committed_voluntary_weight_for_week` (`reporting.rs:2451`) | `committed_voluntary` |

The **planning target** (`planning_target_hours`, §2.7) is fed by **four**
sources: contract, absences, holidays **and** the committed volunteer
hours from `employee_work_details.committed_voluntary`. Compared to the
balance-relevant target, the **base** (contract + commitment instead of
contract only), the **gate** (`planning_base_for_target > 0.0` instead of
`planned_hours > 0.0`) **and** the **scaling of the reduction**
(`target_reduction_scale`, D-64-07) all differ. It is not "the same
computation with a different gate".

#### The two `ShortEmployeeReport` construction sites

`ShortEmployeeReport` is assembled at **two independent places** in
`service_impl/src/reporting.rs`, and neither of them delegates to the
other:

| Site | Function | Consumed by |
| --- | --- | --- |
| **A** | `assemble_weeks` (`reporting.rs:769`, per-week body around `:1121`) | `get_week` (`:2020`) **and** `get_year` (`:2101`) — they share this helper, which is what keeps them byte-identical (Phase 52, D-52-06) |
| **B** | `get_reports_for_all_employees` (`reporting.rs:1212`, per-week body around `:1476`) | `GET /report/?year=…&until_week=…` |

**The rule Phase 64 learned the hard way:** a new field on this struct
must be filled at **both** sites, and it needs a test that holds the two
*against each other* — not two tests that can each go green on their
own. Phase 64's `test_planning_target_parity_both_sites` is exactly that
cross-site parity test; in the RED run it would have passed on `0.0 ==
0.0` without its third assertion (`> 0.0`). Two isolated per-site tests
would not have caught that site B's dynamic branch did not even know the
holiday terms before this phase.

**`has_pending_rebooking` / `pending_rebooking_id` are the documented
exception, not the template.** Both are computed at site B only
(`reporting.rs:1712`) and hard-wired to `false`/`None` at site A
(`reporting.rs:1193`). The reason is stated in the code: the HR alert
banner (Phase 55, D-55-B) is driven exclusively by the
`/report`-list path and has no meaning in the per-week
`get_week`/`assemble_weeks` path. Any further single-site field needs
the same kind of explicit justification — the default is parity.

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
  [F14](./F14-rebooking.md) for the full rule set. **Reader impact
  (state since Phase 57):** the aggregation entries in
  `service_impl/src/reporting.rs` do **not** filter by provenance; the
  category semantics decide what a row contributes (see §2.6). The
  `source` value stays on the record as an origin marker and is
  evaluated for aggregation purposes at exactly one place — the
  `availability()`-based sum in `assemble_weeks`. The Voluntary-Stats
  Ist aggregate delegates to `EmployeeReport::volunteer_hours` (via
  `ReportingService::get_report_for_employee_range`) and therefore
  inherits the same behaviour. Rows without a `rebooking` writer still
  enter the Balance identically (backfill lands on `manual` via column
  DEFAULT).

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

#### `ShortEmployeeReport` — the two target fields

Excerpt from `service/src/reporting.rs:148-…` (Phase 64 added the last
of the four hour figures):

```rust
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
    pub dynamic_hours: f32,
    pub expected_hours: f32,          // balance-relevant Soll — 0.0 on a dynamic contract
    pub overall_hours: f32,
    pub planning_target_hours: f32,   // Phase 64 (PSOLL-01…04) — planning target: the
                                      // reduced COMMITMENT of the week incl. volunteer
                                      // hours [D-64-05]; > 0 on a dynamic contract and on
                                      // a 0-hour contract with a commitment; never enters
                                      // any balance (§2.7)
    …
}
```

Mirrored 1:1 on `ShortEmployeeReportTO`
(`rest-types/src/lib.rs:392`, `#[serde(default)]` so the addition is
wire-additive) plus the field mapping in the `From` impl (`:414`).

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

- `ShortEmployeeReportTO` (`rest-types/src/lib.rs:380-402`): compact
  row for the HR overview. **Since Phase 64** it carries the additional
  response field `planning_target_hours: f32` (§2.7) — the reduced
  **commitment** of the week including volunteer hours [D-64-05] —
  delivered by both
  short-report endpoints — `GET /report/?year=…&until_week=…` and
  `GET /report/week/{year}/{calendar_week}`. The addition is purely
  additive: **no new route, no new handler, and no new `Dioxus.toml`
  proxy entry** was created, and `#[serde(default)]` keeps an old
  payload without the field deserialisable (to `0.0`). **Plan 64-05 did
  NOT touch the wire form:** same field, same type, same endpoints — only
  the semantics of the value is now a commitment rather than a contract
  figure.
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
- **`page/shiftplan.rs`** — the shiftplan itself; below the plan it
  renders `component/working_hours_mini_overview.rs`
  (`WorkingHoursMiniOverview`, `shiftplan.rs:1724`), fed by
  `GET /report/week/{year}/{calendar_week}` via
  `loader::load_working_hours_minified_for_week` →
  `build_working_hours_mini` → `WorkingHoursMini`. **This is the only
  surface that shows the planning target** (§6.5).

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

### 6.5 Which surface shows which "Soll" (Phase 64)

`ShortEmployeeReportTO` now carries two target figures (§2.7) — a balance
figure and a commitment figure. The split
across the frontend is deliberate and follows the transport object each
surface reads — most hour-account surfaces *cannot* show the planning
target, because their DTO does not have it.

**Shows the planning target (`planning_target_hours`)** — exactly three
display sites, all in `component/working_hours_mini_overview.rs`, all
fed from `ShortEmployeeReportTO`:

| Site | File position | What it drives |
| --- | --- | --- |
| card layout, `target` | `working_hours_mini_overview.rs:123` | the Soll figure of the `actual / target` line, plus progress-bar fill and colour threshold (`progress_bar_class`, `hours_text_class`, `progress_bar_percent` read the same binding) |
| table layout, per-row `target` | `:238` | the Soll column of a row, its utilisation bar and percentage |
| table layout, footer `total_target` | `:186` | the Soll sum of the column above it |

A source-level invariant test
(`working_hours_mini_overview_reads_planning_target_only`) keeps this at
exactly three reads, forbids any fallback to `.expected_hours` /
`.dynamic_hours`, and forbids arithmetic directly on the field
(PSOLL-02: the frontend *reads* the value, it does not derive it).

**Still shows the balance-relevant Soll (`expected_hours`)** — these
read `WorkingHoursReportTO` or `EmployeeReportTO`, which Phase 64 did
not touch:

| Surface | Source |
| --- | --- |
| employee detail: year/week list, week detail card, difference column (`component/employee_view.rs:805,867,869`) | `WorkingHoursReportTO.expected_hours` |
| weekly histogram: reference line and bar colour (`component/employee_weekly_histogram.rs:119,126,167`) | `WorkingHoursReportTO.expected_hours` |
| employee list: `target_hours_for` (last week) (`component/employees_list.rs:51`) | `WorkingHoursReportTO.expected_hours` |
| `state/employee.rs:368` — `expected_working_hours` | `EmployeeReportTO.expected_hours` |

**Rule of thumb:** the planning target appears **only** in the weekly
overview below the shiftplan; every hour-account surface shows the
balance-relevant Soll, unchanged. The three display sites therefore show
a **commitment** figure including volunteer hours [D-64-05] — for the same
person it can be larger than any number on the hour-account surfaces, and
that is intended.

The second consumer of `ShortEmployeeReportTO` in the frontend,
`impl From<&ShortEmployeeReportTO> for Employee`
(`state/employee.rs:297`), sets `expected_working_hours: 0.0`
unconditionally and reads no Soll field of the report at all — so it
cannot silently drift apart from the mini overview.

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
  **The only exception since phase 57 (CR-03):** a rebooking pair
  (`source == Rebooking`) raises the balance by `h` and lowers
  `volunteer_hours` by `h` in a dynamic week too — the `+h` leg is
  removed from the expected-equals-actual expected. Otherwise the
  rebooked hours would leave the volunteer account without ever arriving
  in the paid one. Pinned by
  `rebooking_effectiveness.rs::rebooking_conserves_hours_on_dynamic_contract`,
  which additionally checks that `hours_per_week` and `assemble_weeks`
  report the same effect.

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

- `service_impl/src/test/reporting_additive_merge.rs` (1575 lines) —
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
- `service_impl/src/test/reporting_phase2_fixtures.rs` (199 lines) —
  fixtures for the early phase-2 regression; also the mirror pair
  (dynamic vs fixed 40 h contract + vacation/sick periods) that the
  Phase-64 planning-target suite builds on.
- `service_impl/src/test/reporting_planning_target.rs` (1087 lines) —
  Phase 64 (PSOLL-01/03/04), ten tests covering the planning target
  (§2.7). The first five come from 64-01 (contract semantics, all
  fixtures at commitment 0), the last five from plan 64-05 (commitment
  semantics):

  | Test | Purpose | mis-implementation it catches |
  | --- | --- | --- |
  | `test_planning_target_dynamic_get_week` | a dynamic contract reports `planning_target_hours = 40.0` in `get_week` while `expected_hours` stays `0.0` — construction site A | field left neutral (`0.0`) |
  | `test_planning_target_dynamic_all_employees` | the same contract reports `80.0` over two contract weeks in `get_reports_for_all_employees` — construction site B | site B not filled |
  | `test_planning_target_parity_both_sites` | **the cross-site parity test**: both sites yield the same value (`56.0`) for the same input, and absences demonstrably reduce it (`0 < 56 < 80`). The third assertion (`> 0.0`) is what makes it a driver instead of a `0.0 == 0.0` pass | reuse of the balance reduction terms forced to `0` (`80.0`) |
  | `test_planning_target_matches_expected_for_fixed_contract` | byte-identity with `expected_hours` for non-dynamic contracts **at commitment 0**, incl. the 0-hour-contract regression block | loss of the zero-base guarantee (negative value) |
  | `test_available_hours_unchanged_dynamic` | regression lock: `available_hours` stays `0.0` on a dynamic contract even though the report now carries a planning target of `40.0` | switching the `available_hours` assignments to the new field (T-64-02) |
  | `planning_target_includes_commitment_dynamic_both_sites` | commitment 10.0 ⇒ `50.0` (A) and `100.0` (B), `expected_hours` stays `0.0` | commitment ignored (`40.0` / `80.0` — the state before plan 64-05) |
  | `planning_target_commitment_reduced_by_absence_parity` | week 22 `50.0`, week 23 `20.0` (asserted individually), sum `70.0` at both sites | commitment ignored (`56.0`); plain subtraction without `target_reduction_scale` (`76.0`); field neutral (`0.0`) |
  | `planning_target_fixed_contract_differs_by_commitment` | PSOLL-04 in its new form: difference exactly `10.0` (undisturbed) resp. `4.0` (three absence days), and byte-identity at commitment 0 | commitment leaking into `expected_hours`; byte-identity at commitment 0 lost |
  | `planning_target_zero_expected_hours_with_commitment` | `expected_hours == 0` with commitment 10.0 ⇒ `10.0` (A) and `20.0` (B) | the 64-01 gate left in place (`0.0`) |
  | `planning_target_not_clamped_when_reductions_exceed_base` | reductions > base ⇒ **`−10.0`**; at site B decomposed over two windows (until week 22 `10.0`, until week 23 `0.0`, weekly contribution `−10.0`) | a `.max(0.0)` clamp (`0.0`) |

  Plus four pure-fn tests in `reporting.rs` (module
  `test_committed_voluntary_weight_for_week`, from `:3439`) — the only
  place where the distribution decision D-64-06 is directly measurable
  (the integration tests only measure it over the full week, where all
  three candidates yield the same number):

  | Test | measured | mis-implementation it catches |
  | --- | --- | --- |
  | `committed_voluntary_weight_full_week_is_unscaled` | `10.0` | an unconditional division (by 7 or by `workdays_per_week`) |
  | `committed_voluntary_weight_pins_relation_scaling_d_64_06` | `4.2` | `3.0` = day-based 7-day distribution; `7.0` = unscaled SUM (both as explicit counter-assertions) |
  | `committed_voluntary_weight_outside_contract_is_zero` | `0.0` | passing `committed_voluntary` through unchecked |
  | `committed_voluntary_weight_zero_commitment_is_zero` | `0.0` | scaling `expected_hours` instead of `committed_voluntary` |

  A source-side companion lives in the frontend:
  `working_hours_mini_overview_reads_planning_target_only` (§6.5) pins
  PSOLL-02 and was verified against two deliberately re-introduced
  violations (fallback to a legacy field, arithmetic on the field),
  both of which made it fail.
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
- **Phase 64 (PSOLL-01…04, v2.7):** `planning_target_hours` — a second,
  *non*-balance-relevant target figure on `ShortEmployeeReport` /
  `ShortEmployeeReportTO`, so a dynamic contract finally has a
  schedulable target instead of the `0` it inherited from the balance
  semantics (§2.7, §6.5). Every balance figure is unchanged. The phase
  found two traps worth conserving:

  1. **The reduction gate was coupled to the forced zero.** The obvious
     implementation reuses the balance branch's condition, which keys
     off `planned_hours` — a value that `weight_for_week` *forces* to
     `0.0` for a dynamic contract. The reduction would then have been
     skipped for exactly the contracts the feature is for. The paid
     portion of the base is therefore measured against `dynamic_hours`,
     the unforced element of the same tuple, and `has_contract_row`
     remains the second gate half.
  2. **The second construction site had no holiday term at all.** In the
     dynamic branch of `get_reports_for_all_employees` the holiday and
     absence building blocks did not exist before this phase; they had
     to be lifted in front of the three-way branch. A per-site test
     would have gone green on either site alone — only the cross-site
     parity test (§8.1) holds the two against each other.

  3. **The reduction gate was mechanically right and factually wrong.**
     It was correct for a *contract* figure and wrong for a *commitment*
     figure: the committed volunteer hours (`committed_voluntary`)
     appeared in neither formula, and a person with `expected_hours == 0`
     and a real commitment was shown `0.0`. The error was not in the
     mechanics but in the **domain definition** — and it was found in the
     UAT, not by nine green tests. Nine tests can pin a wrong definition
     arbitrarily precisely. Closed by **plan 64-05** (gap closure,
     D-64-05/06/07): the planning target is a commitment figure, the
     commitment flows in, and absences reduce it proportionally.

  `CURRENT_SNAPSHOT_SCHEMA_VERSION` **stays at 13** — see below for why
  that holds structurally. Plan 64-05 changed the computation on
  `ShortEmployeeReport` and still did **not** bump; that proof is below as
  well.

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
Snapshot with a `snapshot_schema_version` (currently **13** since
Phase 57 / D-57-10, see
`service_impl/src/billing_period_report.rs:140`). The snapshot writer
consumes `EmployeeReport` and writes a series of `value_type` rows.
Details see [F08 Billing Period](./F08-billing-period.md).

**Rule for reporting changes:** Any change to the computation of a
snapshot-persisted `value_type` (formula, inputs, filter) MUST bump the
`CURRENT_SNAPSHOT_SCHEMA_VERSION` — otherwise the validator drifts
(see the CLAUDE.md section "Billing Period Snapshot Schema
Versioning").

**Why Phase 64 did not bump it.** `planning_target_hours` (§2.7) lives
on `ShortEmployeeReport`. The snapshot writer
(`build_and_persist_billing_period_report`) consumes **`EmployeeReport`**
— a different type, produced by a third code path
(`get_report_for_employee_range` → `hours_per_week`) that does not have
this field at all. The new value therefore cannot reach a persisted
`value_type` even by accident, and no existing `value_type`'s
computation or input set was touched. This is a *structural* argument,
not an assurance: grep for `planning_target_hours` in
`billing_period_report.rs` returns nothing. The version stays at **13**.

**Why plan 64-05 did not bump either, although it changed a
computation.** What changed are the two `ShortEmployeeReport` construction
sites (`assemble_weeks` and `get_reports_for_all_employees`) — and there
exclusively the planning-target block, which enters no balance figure. The
third `weight_for_week` caller, `hours_per_week`, feeds `EmployeeReport`
and thereby the snapshot; it is untouched by 64-05, as are
`booking_information.rs` and `billing_period_report.rs`. Evidenced by
counter gates rather than assurance: `available_hours: report.expected_hours`
still occurs exactly **twice**, `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13`
exactly **once**, and none of those four files appears in any diff of the
plan. The full test suite (1182 tests, 9 of them new) is green without a
single adjusted expectation — the billing and carryover sections in
particular.

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
