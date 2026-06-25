---
phase: 15-reporting-no-double-count-snapshot-bump-same-commit
reviewed: 2026-06-24T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - service/src/booking_information.rs
  - service_impl/src/booking_information.rs
  - service_impl/src/billing_period_report.rs
  - service_impl/src/test/booking_information.rs
  - service_impl/src/test/mod.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# Phase 15: Code Review Report

**Reviewed:** 2026-06-24
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Phase 15 decomposes voluntary capacity into two bands: `committed_voluntary_hours`
(Band 1, the cap-gated pledge) and `volunteer_hours` (Band 2, the per-person surplus
above the pledge), wired into `get_weekly_summary`. The helper
`volunteer_surplus_above_committed`, the snapshot-version-unchanged regression test,
and the no-bump decision are all correct.

However, the core no-double-count / per-person invariant — the explicit focus of this
phase — is **violated in production** by a granularity bug: Band 2 applies the
per-person max to each per-DAY shiftplan report row, not to the per-person weekly
total. The D-05 fixtures never exercise a person who works multiple days in a week,
so they pass while the production path silently under-counts surplus. This is a
BLOCKER. The supporting Band-1/Band-2 asymmetry and the misleading "per-person"
comments compound the risk.

## Critical Issues

### CR-01: Band-2 per-person surplus is applied per-DAY row, breaking the no-double-count invariant

**File:** `service_impl/src/booking_information.rs:160-186`

**Issue:** `extract_shiftplan_report_for_week` returns one `ShiftplanReportDay` **per
person per day** — the DAO query in `dao_impl_sqlite/src/shiftplan_report.rs:156` uses
`GROUP BY sales_person_id, year, day_of_week`. The Band-2 computation maps
`volunteer_surplus_above_committed(report.hours, committed_p)` over each of those
**daily** rows, subtracting the person's full weekly `committed_p` from *each day*
independently, then sums.

Because `max(actual − committed, 0)` is nonlinear (the exact property the phase says
must be respected), applying it per-day instead of per-person-week produces the wrong
result whenever a person works more than one day:

- Volunteer with `committed_p = 5`, working Mon 3h + Tue 4h (weekly actual = 7).
- Correct (per-person): `max(7 − 5, 0) = 2`.
- Actual code (per-day): `max(3 − 5, 0) + max(4 − 5, 0) = 0 + 0 = 0`.

The surplus is systematically under-reported, and the "committed + surplus =
max(committed, actual)" invariant from the fixture suite is broken. The D-05 tests in
`service_impl/src/test/booking_information.rs` all use a single scalar `actual` per
person and never construct a multi-day-per-person scenario, so they cannot catch this.

**Fix:** Aggregate `report.hours` to a per-person weekly total *before* applying the
surplus helper. For example:

```rust
use std::collections::HashMap;

// 1. Sum actual volunteer hours per person for the week.
let mut actual_by_person: HashMap<Uuid, f32> = HashMap::new();
for report in shiftplan_reports.iter() {
    if volunteer_ids.contains(&report.sales_person_id) {
        *actual_by_person.entry(report.sales_person_id).or_default() += report.hours;
    }
}

// 2. Apply max(actual_p − committed_p, 0) once per person, then sum.
let volunteer_hours: f32 = actual_by_person
    .iter()
    .map(|(&sp_id, &actual_p)| {
        let committed_p: f32 = find_working_hours_for_calendar_week(&all_work_details, year, week)
            .filter(|wh| wh.sales_person_id == sp_id && wh.cap_planned_hours_to_expected)
            .map(|wh| wh.committed_voluntary)
            .sum();
        volunteer_surplus_above_committed(actual_p, committed_p)
    })
    .sum();
```

Add a fixture/integration test where one volunteer has two daily report rows whose sum
exceeds `committed_p` but neither single day does, asserting Band 2 equals the
per-person (not per-day) surplus.

## Warnings

### WR-01: A volunteer with a committed pledge but zero actual report rows is dropped from Band 2

**File:** `service_impl/src/booking_information.rs:169-186`

**Issue:** Band 2 is computed by iterating the shiftplan report rows. A capped
volunteer who has `committed_voluntary > 0` but no bookings that week produces **no**
`ShiftplanReportDay` row, so the `.map` never runs for them and they contribute
nothing to Band 2. That is arguably correct for Band 2 itself (`max(0 − c, 0) = 0`),
but it means the iteration structure depends on report rows existing — once CR-01 is
fixed by aggregating per person, make sure the per-person set is derived from the
*union* of report-row persons and committed-row persons if any consumer ever expects a
zero-surplus person to be represented. Today this only hides a 0 contribution, but it
is a latent correctness assumption worth pinning with a test (`fn cvc04_zero_actual`
exists at the helper level but no production-path test covers "pledge, no bookings").

**Fix:** After fixing CR-01, drive the per-person loop from a set that includes
volunteers with committed pledges, not only those with report rows, and add a test for
a capped volunteer with a pledge and no bookings.

### WR-02: Band 1 sums committed over all persons; Band 2 only over volunteers — undocumented asymmetry

**File:** `service_impl/src/booking_information.rs:160-196`

**Issue:** Band 1 (`committed_voluntary_hours`, lines 189-196) filters only on
`cap_planned_hours_to_expected` and sums `committed_voluntary` across **all** active
work-details rows, including `is_paid == true` persons. Band 2 (lines 169-170) filters
on `volunteer_ids` (i.e. `!is_paid`). So a paid+capped person with
`committed_voluntary > 0` adds to Band 1 but is structurally excluded from Band 2. The
fixture `cvc04_paid_capped_band2_zero` asserts this is intended (paid person's
`actual_vol = 0`), but the production code never proves the paid person's actual
volunteer hours are zero — it simply never looks. If a paid person ever has shiftplan
report hours, those hours are silently absent from Band 2 while their pledge inflates
Band 1, so the system-level `Band1 + Band2 = Σ_person max(committed_p, actual_p)`
invariant no longer holds for that person.

**Fix:** Either (a) gate Band 1 with the same `volunteer_ids` membership as Band 2 if
pledges are only meaningful for volunteers, or (b) explicitly document and test that
paid persons have `actual_vol == 0` in this path so the asymmetry is provably safe.
Add an integration test with a paid+capped person carrying both a pledge and booking
rows to lock the intended behavior.

### WR-03: `committed_p` recomputed inside the row map causes repeated full scans and invites drift

**File:** `service_impl/src/booking_information.rs:171-183`

**Issue:** Inside the `.map` over report rows, `find_working_hours_for_calendar_week`
re-scans `all_work_details` and re-sums `committed_voluntary` for every row. Beyond the
redundant work, the per-row recomputation is exactly what made CR-01 easy to introduce:
the committed value is conceptually per-person-week but is being threaded through a
per-row pipeline. After CR-01 is fixed by hoisting actuals into a per-person map,
compute `committed_p` once per person in the same loop so the two bands share one
person-keyed pass and cannot diverge.

**Fix:** Build a single per-person structure `{ actual_p, committed_p }` for the week
and derive both bands from it, eliminating the per-row recompute.

## Info

### IN-01: Code comments claim "per person" but the implementation iterates per-day rows

**File:** `service_impl/src/booking_information.rs:157-159, 172, 184`

**Issue:** The comments repeatedly assert "per-person surplus", "Per-person committed
for this week", and "max(actual_p − committed_p, 0)" with subscript `p` for person —
but the surrounding `.iter().filter(...).map(...)` runs over per-day
`ShiftplanReportDay` rows, so `report.hours` is a single day's hours, not `actual_p`.
The comments actively mislead a future reader into believing the invariant holds. Once
CR-01 is fixed the comments become accurate; until then they should not assert a
property the code does not have.

**Fix:** Align comment subscripts with the actual iteration granularity, or fix CR-01
so the per-person framing is true.

### IN-02: `committed_voluntary_hours` placeholder hard-coded to 0.0 in `get_summery_for_week`

**File:** `service_impl/src/booking_information.rs:514-517`

**Issue:** The single-week variant sets `committed_voluntary_hours: 0.0` while
`volunteer_hours` there is left as the full actual (no surplus reduction). This is
documented as intentional (Band 1 is year-view-only, Phase 16 wires display), so it is
not a defect today. Flagging only so the divergence between the two `WeeklySummary`
producers (one does two-band decomposition, one does not) is tracked: a consumer that
reads both fields from `get_summery_for_week` would double-count, since here
`volunteer_hours` still includes hours that would land in Band 1 in the year view.

**Fix:** No change required for Phase 15. When Phase 16 wires display, ensure no
consumer sums `volunteer_hours + committed_voluntary_hours` from the single-week
variant, or populate Band 1 consistently here.

---

_Reviewed: 2026-06-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
