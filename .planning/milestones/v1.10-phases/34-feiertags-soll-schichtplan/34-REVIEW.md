---
phase: 34-feiertags-soll-schichtplan
reviewed: 2026-06-30T15:54:02Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - service_impl/src/reporting.rs
  - service_impl/src/test/reporting_holiday_auto_credit.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# Phase 34: Code Review Report

**Reviewed:** 2026-06-30T15:54:02Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Phase 34 added a 4th holiday injection point in `ReportingService::get_week`
(`reporting.rs:1072-1118`). The core derivation reuses `build_derived_holiday_map`
(injection points 1a/1b), and the direct band-guard for `dynamic_hours` (HSP-03) is
implemented correctly: `holiday_derived_gated` is subtracted from `expected_hours`
(line 1107) but **not** from `dynamic_hours` (line 1116). The Stichtag gate,
manual-wins conflict check, and the no-double-count vs a manual `ExtraHours(Holiday)`
all work as claimed for the empty-shiftplan / cap-inactive cases the tests exercise.

However, the band guard is incomplete. `expected_hours` — now reduced by the derived
holiday — is fed directly into `apply_weekly_cap` (line 1108). When
`cap_planned_hours_to_expected` is active and shiftplan hours exceed the
holiday-reduced expected, the derived holiday inflates `auto_volunteer_hours`, which
leaks into the `volunteer_hours` band (line 1115). HSP-03 / D-25-08 explicitly lists
`volunteer_hours` as a band that must remain untouched. This path is entirely
untested: the fixture has `cap_planned_hours_to_expected: false`
(`reporting_phase2_fixtures.rs:59`) and every `get_week` test uses an empty shiftplan,
so the cap never binds. There are also two cross-path inconsistencies and a stale
module doc comment.

## Critical Issues

### CR-01: Derived holiday leaks into the `volunteer_hours` band via the weekly cap

**File:** `service_impl/src/reporting.rs:1107-1115`
**Issue:**
The HSP-03 / D-25-08 band guard requires the derived holiday to touch ONLY
`expected_hours` / `holiday_hours` / `available_hours` and to leave the capacity
bands (`paid_hours` / `dynamic_hours` / `committed_voluntary_hours` /
`volunteer_hours`) untouched. The direct `dynamic_hours` subtraction is correctly
omitted (line 1116), but the holiday-reduced `expected_hours` is then used as the
cap threshold:

```rust
let expected_hours = planned_hours - abense_hours_for_balance
    - absence_derived_balance_total - holiday_derived_gated;   // line 1107
let (shiftplan_hours, auto_volunteer_hours) =
    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours); // line 1108
...
let volunteer_hours = manual_volunteer_hours + auto_volunteer_hours + no_contract_volunteer; // line 1115
```

`apply_weekly_cap` returns `auto_volunteer = raw_shiftplan_hours - expected_hours`
when `cap_active && raw_shiftplan_hours > expected_hours`. Because the derived
holiday lowers `expected_hours` (e.g. 40 → 32), the cap converts an extra 8h of
worked shiftplan into `auto_volunteer_hours`, inflating `volunteer_hours` by exactly
the holiday amount. Concretely, with `cap_planned_hours_to_expected = true` and
`raw_shiftplan_hours = 40`:
- No holiday: `auto_volunteer = 40 − 40 = 0`.
- Derived 8h holiday: `auto_volunteer = 40 − 32 = 8` → `volunteer_hours` +8.

This is a reachable configuration (the cap is a real per-contract feature flag) and
is **completely untested** — the fixture sets `cap_planned_hours_to_expected: false`
and all three `get_week` tests pass an empty `shiftplan_report`, so the cap branch
never executes. The injection-point comment (lines 1103-1106) only reasons about the
direct `dynamic_hours` subtraction and overlooks the cap path.

Note: the same leak already exists for absence (`abense_hours_for_balance` /
`absence_derived_balance_total` also feed `expected_hours` before the cap), so this
is consistent with established absence handling — but Phase 34's stated band guard
explicitly enumerates `volunteer_hours`, so for the derived holiday it is a spec
violation that must be resolved (either fix, or explicitly document the cap-path
carve-out as accepted).

**Fix:** Cap against the pre-holiday expected so the derived holiday cannot reach the
volunteer band, e.g.:

```rust
// Threshold for the cap must exclude the derived-holiday term (band guard, HSP-03).
let expected_for_cap = planned_hours - abense_hours_for_balance - absence_derived_balance_total;
let (shiftplan_hours, auto_volunteer_hours) =
    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_for_cap);
// Holiday still reduces the *balance* expected only:
let expected_hours = expected_for_cap - holiday_derived_gated;
```

Add a `get_week` regression test with `cap_planned_hours_to_expected: true`,
`raw_shiftplan_hours > expected`, and an active holiday that asserts
`volunteer_hours` is unchanged versus the no-holiday baseline.

## Warnings

### WR-01: `get_week` test gate never exercises the weekly cap

**File:** `service_impl/src/test/reporting_holiday_auto_credit.rs:544-795`
**Issue:** All three Phase-34 `get_week` tests (`test_holiday_auto_credit_no_year_view_impact`,
`test_hsp04_before_cutoff`, `test_hsp04_manual_wins`) build on
`fixture_work_details_8h_mon_fri` (cap=false) and mock
`extract_shiftplan_report_for_week` to return an empty vector. With zero shiftplan
hours the cap can never bind, so the most fragile part of the band guard (CR-01) is
unverified. The tests therefore do not actually pin "bands unchanged" against the
realistic case where the employee worked hours in a holiday week with the cap on.
**Fix:** Add a subtest with a non-empty `shiftplan_report_for_week` (e.g. 40h) and a
work-details row with `cap_planned_hours_to_expected: true`, asserting `volunteer_hours`
and `dynamic_hours` match the no-holiday run while `expected_hours`/`holiday_hours`
reflect the credit.

### WR-02: `dynamic_hours` holiday semantics diverge across the three report entry points

**File:** `service_impl/src/reporting.rs:1116` (vs `:633` and `:1447`)
**Issue:** `get_week` deliberately excludes the derived holiday from `dynamic_hours`
(band guard). But the two year-view paths fold the derived holiday into the absence
total that *does* reduce `dynamic_hours`:
- `get_reports_for_all_employees`: `absense_hours` includes `derived_holiday_for_week`
  (line 543) and `dynamic_hours = weekly_hours.dynamic_hours - weekly_hours.absense_hours - ...`
  (line 633).
- `hours_per_week`: `absence_hours` includes `derived_holiday_for_week` (line 1364)
  and `dynamic_hours: dynamic_working_hours_for_week - absence_hours` (line 1447).

So the same conceptual field (`dynamic_hours` / the `paid_hours` band) reacts to a
derived holiday differently depending on which endpoint produced the report. If the
HSP-03 / D-25-08 band-guard invariant is meant to hold wherever `dynamic_hours` feeds
booking_information, the year-view paths violate it; if only `get_week` feeds the
capacity bands, the divergence is acceptable but currently undocumented and untested.
**Fix:** Decide and document which `dynamic_hours` consumers must honor the band
guard. If the invariant is global, drop `derived_holiday_for_week` from the
`absence_hours`/`absense_hours` sums that feed `dynamic_hours` in the year-view paths
(keeping it in `expected_hours` only), mirroring `get_week`. Add a test pinning the
chosen year-view behavior.

### WR-03: Stale module doc + misleading test name assert the opposite of Phase-34 behavior

**File:** `service_impl/src/test/reporting_holiday_auto_credit.rs:8-9, 544`
**Issue:** The module header still states the pre-Phase-34 invariant:
`"HOL-03: booking_information year-view ... is unaffected by holiday auto-credit
(get_week() has no derive-on-read)."` After Phase 34, `get_week` **does** derive on
read, and the rebuilt `test_holiday_auto_credit_no_year_view_impact` verifies exactly
that (expected 40→32, holiday 8). The doc comment now contradicts the implementation
and the test it documents, and the function name `..._no_year_view_impact` implies no
effect while the body asserts a real effect. Future readers will be misled about the
invariant.
**Fix:** Update the module doc (lines 8-9) to describe the Phase-34 `get_week`
derive-on-read and the band guard (dynamic_hours unchanged), and rename the test to
something like `test_holiday_auto_credit_get_week_band_guard`.

## Info

### IN-01: `build_derived_holiday_map` re-fetched per employee inside the `get_week` loop

**File:** `service_impl/src/reporting.rs:1084-1092`
**Issue:** The helper is invoked once per employee within the `for (sales_person_id,
working_hours)` loop, and each call re-reads the `holiday_auto_credit` toggle and the
week's special days — both of which are identical for every employee in the same
week. Functionally correct (the conflict/contract checks are per-employee), but the
toggle value and `get_by_week` result could be hoisted above the loop and the
per-employee step reduced to the manual-wins + contract-coverage filtering. (Flagged
as maintainability/duplication; runtime cost is out of v1 review scope.)
**Fix:** Hoist the toggle read and `special_day_service.get_by_week` out of the loop;
pass the shared holiday set into a slimmer per-employee resolver.

### IN-02: HSP-04b (`test_hsp04_manual_wins`) only asserts `holiday_hours`

**File:** `service_impl/src/test/reporting_holiday_auto_credit.rs:786-794`
**Issue:** The manual-wins subtest asserts only `holiday_hours == 8.0`. It does not
pin `expected_hours == 32` or `dynamic_hours == 40` for the manual-present case, so a
regression where a manual holiday double-reduces `expected_hours` (once via
`abense_hours`, once via a non-skipped derived term) or perturbs the band would not be
caught. The no-double-count claim is only partially covered.
**Fix:** Add `assert!((report.expected_hours - 32.0).abs() < 0.01)` and
`assert!((report.dynamic_hours - 40.0).abs() < 0.01)` to `test_hsp04_manual_wins`.

---

_Reviewed: 2026-06-30T15:54:02Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
