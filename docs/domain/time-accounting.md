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

### Transferring voluntary hours to the paid account (rebooking)

Voluntary hours can be **transferred** onto the paid hour account. A
transfer over `h` hours raises the worked (paid) hours by `h` and lowers
the voluntary hours by the same amount — the total number of hours the
person put in stays the same, only the account they are booked on
changes.

The voluntary pledge from the contract (`committed_voluntary`) is **not**
affected by a transfer. It is the target figure and is never touched — a
transfer only ever moves the actual (`Ist`) side.

Two lower bounds govern how far a transfer may go, and they belong to two
different actors:

- **The automation** (weekly job) transfers at most as much as brings the
  balance to `0`, and only the surplus above the pledge. It never lifts
  the balance into the positive, and it never lets the person fall below
  their pledge.
- **A proposal confirmed by HR** *may* undercut the pledge — that is
  precisely what the proposal exists for — but it may never push the
  voluntary hours for the year below `0`. You cannot have worked −3 hours
  voluntarily in total.

Example: 5 h paid + 5 h voluntary pledged per week over two weeks; 3 h
worked in week one, 17 h in week two. The account reads 8 h paid / 12 h
voluntary, balance −2. A transfer of 2 h settles it to 10 h / 10 h,
balance 0. Even if 23 h had been worked instead of 20 h, the transfer
would still be 2 h, not 5 — the balance cap binds, and the 3 surplus
hours stay voluntary.

For the mechanics — who writes the pair, when the automation runs, how
the cap is computed — see
[F14 §3 / §9.1](../features/F14-rebooking.md). Note that voluntary hours
count into the balance in the weekly view but not in the annual report;
that pre-existing difference between the two aggregation paths is
documented in [F07 §2.6](../features/F07-reporting-balance.md).

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

### Two kinds of "Soll": balance-relevant expected vs planning target

Since Phase 64 the weekly short report carries **two** target figures.
The balance-relevant expected is a **contract** figure; the planning
target is a **commitment** figure [D-64-05]. They answer different
questions, and they are not fed from the same set of sources.

| | `expected_hours` — *balance-relevant expected* | `planning_target_hours` — *planning target* |
| --- | --- | --- |
| What it is for | the `expected` leg of the base formula above | the figure a planner compares the assigned shifts against |
| Sources | contract, absences, public holidays | the same three **plus** the committed volunteer hours (`committed_voluntary`) |
| Dynamic contract | deliberately **`0`** | deliberately **not `0`** |
| Contract with 0 hours and commitment > 0 | `0` | **not `0`** — the commitment alone carries the figure |
| Includes the committed volunteer hours? | **no**, never | **yes** — reduced by absences and public holidays |
| Enters the balance? | **yes** — it *is* the balance's expectation | **no**, never; also not Carryover, Billing snapshot, or availability |
| Where it becomes visible | the hour-account surfaces (Employee Details, weekly histogram, employee list) | the overview below the shiftplan only |

**The one sentence to remember:** on a dynamic contract the
balance-relevant expected is deliberately zero, because *expected equals
actual*; and the planning target is deliberately non-zero, because you
cannot plan against a target of zero — and it includes the committed
volunteer hours, because the planner needs to see what the person commits
to, not which part of it is paid.

At commitment 0 non-dynamic contracts are unaffected: there both figures
carry the same number, by construction and not by coincidence. With a
commitment they differ by exactly the absence- and holiday-reduced
commitment. The reasoning — why the planning target needs its own base,
its own reduction gate and its own reduction scaling, and why none of them
must be "unified" back into the balance path — is in
[F07 §2.7](../features/F07-reporting-balance.md). Do not re-derive it
here; there is exactly one place where it belongs.

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

Centrally in `service_impl/src/reporting.rs` (3261 lines), aggregated
over `sales_person`, `booking`, `extra_hours`, `absence`, `carryover`,
`special_days`.

Internal reads run with `Authentication::Full` — the REST handler has
already verified the user auth, and the reporting layer needs the raw
data without a per-read permission guard.

The per-week short-report figures are not assembled in one place but in
**two** — `assemble_weeks` (shared by the single-week and the whole-year
read) and `get_reports_for_all_employees`. The **planning target** is
produced at both of them, exactly like every other short-report figure;
its **base**, its **reduction gate** and the **scaling of the reduction**
deliberately differ from the ones the balance formula uses. Reason and
consequences:
[F07 §2.7 and §3](../features/F07-reporting-balance.md).

## Where the calculation becomes visible

- **Weekly Overview** — frontend page with blocks per Sales Person.
- **My Shifts** — Sales Person's own view.
- **Employee Details** — HR view with time range selector.
- **Billing Period Details** — the frozen version.
- **Overview below the shiftplan** — the only surface that shows the
  **planning target** rather than the balance-relevant expected: the
  `actual / target` figure, the utilisation bars and the target sum of
  the footer row. Because the planning target includes the committed
  volunteer hours, that figure can be larger for the same person than any
  number on the hour-account surfaces — and that is intended. Everything
  the hour account shows — Employee Details, weekly histogram, employee
  list, every balance figure — keeps showing the balance-relevant
  expected.

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
