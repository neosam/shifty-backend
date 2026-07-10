# Time Account — How the Balance Is Calculated

This file explains, from the domain perspective, how the time account
(also: Balance) in Shifty is calculated. For the technical deep-dive see
[F07 Reporting & Balance](../features/F07-reporting-balance.md). For edge
cases see [`edge-cases.md`](./edge-cases.md).

## The base formula

```
balance = worked − expected + carryover
```

For a given Sales Person and a time range:

- **`worked`** — actually recorded hours.
- **`expected`** — contractually expected hours.
- **`carryover`** — previous year's balance rolled into this year.

The result is a positive or negative delta:

- **+5 hours:** Five hours of overtime.
- **−3 hours:** Three hours short (not enough worked to meet the
  expectation).

## What counts as `worked`?

Actual hours are composed of:

1. **Bookings** — planned shifts (Sales Person × Slot × date). Each
   Booking contributes the duration of the assigned Slot.
2. **Extra Hours with category `ExtraWork`** — overtime outside of the
   Shiftplan (special duties, unplanned work).
3. **Absences with a "positive" category** — Vacation, SickLeave,
   Holiday, Unavailable, VolunteerWork are treated as "worked" (from the
   perspective of the Balance, so that they satisfy the expectation).
4. **Custom Extra Hours** — depending on the definition of the category.

**Category overview:**

| Category | Contributes to `worked`? | Reduces `expected`? | Note |
| --- | --- | --- | --- |
| Shiftplan (Booking) | Yes | No | Regular case |
| ExtraWork | Yes | No | Overtime |
| Vacation | Yes | No | Vacation counts as "worked" |
| SickLeave | Yes | No | Sick leave counts as "worked" |
| Holiday | Yes | No | Holiday auto-credit |
| Unavailable | Yes | No | Availability block |
| VolunteerWork | Yes | No | Volunteer work |
| **UnpaidLeave** | **No** | **Yes** | Special case: reduces expectation |
| CustomExtraHours | Defined | Defined | Per custom category |

**The special case `UnpaidLeave`:** Instead of contributing to the actual
side, the expectation is reduced by the duration. Effect: A day of unpaid
leave does not push the Balance into the negative but instead lowers the
"how much should have been worked" figure.

### Ist/Soll whole-week-out symmetry for voluntary hours (v2.6.1)

Both the Weekly voluntary display
(`WeeklySummary.committed_voluntary_hours` in
`service_impl/src/booking_information.rs`) and the range-based
voluntary target (`committed_voluntary_target_in_range` in
`service_impl/src/reporting.rs`) treat any calendar week with at least
one Absence day of the same salesperson as **fully out** — the
volunteer commitment for that week drops to `0` on **both** sides
(Ist and Soll). The rule is category-agnostic: Vacation, SickLeave and
UnpaidLeave all trigger it. In the same rollout, the `contract_weeks`
denominator behind `ist_per_contract_week` also excludes Absence weeks
so the average is not diluted by weeks that were unavailable for
volunteer work.

The Weekly display already had this rule since v2.6.0 (D-26-03 /
VFA-01); v2.6.1 aligns the range Soll and the contract-weeks
denominator (D-54.5-01 / D-54.5-02). See
[F14](../features/F14-rebooking.md) for the range Soll and F03
(booking_information) for the weekly display.

## What is `expected`?

Expectation is derived from:

1. **Contract rows** (`employee_work_details`) — weekly hours distributed
   across weekdays.
2. **Special Days** — public holidays or operational special days reduce
   the expectation on that day.
3. **UnpaidLeave** (see above) — reduces the expectation over the booked
   range.

**Per-day base schema:**

```
expected(day) = if working_day(contract, day) then
                    hours_per_day(contract)
                    − special_day_reduction(day)
                    − unpaid_leave_reduction(sales_person, day)
                else
                    0
```

**Aggregation over a time range:**

```
expected(from..to) = Σ expected(day) for day in from..to
```

## What is `carryover`?

**Carryover** is the frozen year-end balance from the previous year.

Example: On 2025-12-31 the Sales Person had `+8` Balance hours. This
value is persisted as `carryover(2025)`. When the Balance for 2026 is
computed, the starting value is not 0 but the 8 hours from the end of
the previous year.

**Why this pattern?** Without Carryover every report would have to
recompute everything since the start of operation. With Carryover the
current year alone is sufficient.

**When Carryover is written:** The scheduler
(`service_impl/src/scheduler.rs:60,68`) calls
`update_carryover_all_employees(year-1, Full)` and
`update_carryover_all_employees(year, Full)` on a schedule. Both years
are updated — the current one is re-adjusted when retroactive changes
arrive.

## Time scales

The Balance can be computed for different time windows:

- **Per day** — base unit.
- **Per calendar week** — default view (Block report).
- **Per month** — HR view.
- **Per year** — full-year balance.
- **Ad-hoc range** — arbitrary `[from, to]`.

Aggregation is additive over the days.

## Weekly Cap

**[Verified via F07 docs]** There is a "weekly cap" mechanism: the
Balance in a week is capped so that extreme values in a single week
don't distort the picture. Details: `apply_weekly_cap` in `reporting.rs`
— see F07.

## Vacation Balance separately

The vacation balance is a parallel calculation, using the same
Carryover idea but on vacation days instead of hours:

```
vacation_balance = entitled + carryover(year−1) − (used + planned) + offset
```

Details: [F06](../features/F06-vacation-management.md).

## Where the calculation happens

Centrally in `service_impl/src/reporting.rs` (2205 lines), aggregated
over `sales_person`, `booking`, `extra_hours`, `absence`, `carryover`,
`special_days`.

Internal reads run with `Authentication::Full` — the REST handler has
already verified the user auth, and the reporting layer needs the raw
data without a per-read permission guard.

## Where the calculation becomes visible

- **Weekly Overview** — frontend page with blocks per Sales Person.
- **My Shifts** — Sales Person's own view.
- **Employee Details** — HR view with time range selector.
- **Billing Period Details** — the frozen version.

## Why this is complicated

Because many edge conditions combine:

- Contract change mid-week.
- Special Days on the weekend (change nothing if not a working day).
- Absences across the year boundary (must cross the Carryover timepoint).
- Toggle rollouts (effective-date based — different calculation paths
  before/after the effective date).
- Snapshot frozen-vs-live diff (Billing Period vs live reporting).

Before any change to the Balance calculation: **read
[`edge-cases.md#1-stundenkonto`](./edge-cases.md#1-stundenkonto)**.
