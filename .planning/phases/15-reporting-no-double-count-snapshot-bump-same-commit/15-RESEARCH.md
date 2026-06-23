# Phase 15: Reporting no-double-count (Achse B) — Research

**Researched:** 2026-06-23
**Domain:** Backend calculation — `committed_voluntary_hours` term in `booking_information.rs::get_weekly_summary` (Achse B); no snapshot bump; per-week `max(committed, actual_volunteer)` formula; `cap_planned_hours_to_expected` gate.
**Confidence:** HIGH (all claims verified against current HEAD code; Phase 14 summaries verified delivered state)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01 (Achse B only, KEIN Snapshot-Bump):** `committed_voluntary` flows exclusively into `booking_information.rs::get_weekly_summary` (Achse B). NOT into `reporting.rs` (Achse A). No persisted `value_type` changes → `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 7. CVC-05 ("bump 7→8") is OBSOLETE and must be replaced in REQUIREMENTS.md/ROADMAP with a no-bump justification.
- **D-02 (alle Kategorien + mehr Tests):** Mandatory test cases: core max-per-week (committed=5/actual=7→7, committed=5/actual=3→5), sum-over-year (never max(Σ,Σ)), cap=false→0.0 regression lock, multi-person aggregation. Discretionary: single-week, empty-week→0.0, boundary committed==actual, committed=0 no-effect. Float comparisons via epsilon.
- **D-03 (flat per active ISO-week):** `committed_voluntary` counts flat per active ISO-week; no pro-rating by `weight_for_week`. Reuses Phase-14 SUM-helper `committed_voluntary_for_calendar_week` which does not weight.
- **D-04 (Service-Struct + calculation in Phase 15; TO + frontend in Phase 16):** Phase 15 adds `committed_voluntary_hours: f32` to the SERVICE struct `service::booking_information::WeeklySummary` and implements the calculation in `get_weekly_summary`. `WeeklySummaryTO` + `From` mapping + frontend display deferred to Phase 16.

### Claude's Discretion

- Exact test module / file placement (inline `#[cfg(test)]` in `booking_information.rs` vs. dedicated module in `service_impl/src/test/`).
- Whether a small private helper for per-week `max`-before-sum reduction is introduced or inlined.

### Deferred Ideas (OUT OF SCOPE)

- `WeeklySummaryTO` + `From` mapping + frontend "zugesagt" token + surplus display + i18n → Phase 16.
- Editor input (`contract_modal.rs`) + "alle"-filter + unpaid-volunteer-record + `is_paid`-gating of at-risk sites → Phase 17.
- Inline banner "Zusage nicht erfüllt" + committed band in chart → v1.5 (CVC-F-01/CVC-F-02).
- `reporting.rs` Achse A / balance / billing-snapshot changes → explicitly NOT Phase 15.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CVC-04 | Jahresansicht availability computes commitment without double-counting — separate `committed_voluntary_hours` term in `booking_information.rs::get_weekly_summary` (Achse B, NOT `reporting.rs`), per ISO-week via `counted_volunteer = max(committed_voluntary, actual_volunteer)`, summed over year (never `max(Σ,Σ)`); gated on `cap_planned_hours_to_expected = true` | Verified: `get_weekly_summary` is the Achse-B integration site; formula confirmed; `employee_work_details_service` already DI'd in; `committed_voluntary_for_calendar_week` helper ready for reuse |
| CVC-05 (REVISED per D-01) | OBSOLETE as "bump 7→8" requirement; replace with: no-bump justification for Phase 15 — `committed_voluntary_hours` is Achse-B (year-view) only, never feeds any persisted `BillingPeriodValueType`; `WeeklySummary` is not consumed by `billing_period_report.rs`; version stays 7 | Verified: `billing_period_report.rs:240-250` sources `Volunteer` from `reporting_service` reports only, never from `WeeklySummary`; CLAUDE.md covers "purely additive changes that do not touch the snapshot's value_types" as no-bump case |
| CVC-06 | For non-capped persons (`cap_planned_hours_to_expected = false`), the commitment contributes `0.0` to all calculations and is not displayed (blank/dash, not `0`); `committed=0` result is bit-identical to pre-v1.4 | Verified: gate on cap flag is a single predicate; `committed_voluntary_for_calendar_week` returns 0.0 for zero-value rows; backward-compat guaranteed |
</phase_requirements>

---

## Summary

Phase 15 is a focused backend calculation phase: add a new `committed_voluntary_hours: f32` field to the `WeeklySummary` service struct and compute it in `get_weekly_summary` (Achse B) using the closed-form `counted_volunteer = max(committed_voluntary, actual_volunteer)` per ISO-week, then sum over the year. The Phase 14 SUM-helper `committed_voluntary_for_calendar_week` (in `reporting.rs:101-109`) is the reusable building block for the committed side. The actual volunteer side is already computed as `volunteer_hours` in `get_weekly_summary` via `shiftplan_report_service.extract_shiftplan_report_for_week()` filtered to `volunteer_ids` (is_paid=false persons).

**The critical open question is resolved below (see "Critical Open Question — `actual_volunteer` for paid capped persons").** The answer is option (b): `actual_volunteer` = 0 for is_paid=true persons in Achse B. The new `committed_voluntary_hours` term for capped paid persons reduces to pure `committed_voluntary` — that is, `max(committed_voluntary, 0) = committed_voluntary`. The D-02 worked examples (`committed=5/actual=7→7`, `committed=5/actual=3→5`) apply to the aggregate across ALL persons (predominantly the is_paid=false volunteer pool, with the capped paid person's commitment adding a floor), not to the breakdown by person class.

The snapshot version stays at 7 (D-01, CVC-05 revised): `WeeklySummary` is year-view-only and is never read by `billing_period_report.rs`. No persisted `value_type` changes. The CLAUDE.md bump rule is not triggered.

**Primary recommendation:** Integrate in `get_weekly_summary` only. Add `committed_voluntary_hours: f32` to `WeeklySummary`. For each week in the yearly loop: compute `committed = committed_voluntary_for_calendar_week(all_work_details_for_year, year, week)` (flat, no weight), gated on `cap_active` (any work-details row for that person in that week has `cap_planned_hours_to_expected=true`). The `actual_volunteer` is the existing `volunteer_hours` (is_paid=false shiftplan hours). `counted = max(committed, actual_volunteer)`. Accumulate into `committed_voluntary_hours`. The `overall_available_hours` formula stays as-is for now (Phase 16 wires it into the display).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `committed_voluntary_hours` computation | API / Backend (Business-Logic Service) | — | `BookingInformationServiceImpl` is already Business-Logic tier (consumes `ReportingService`, `ShiftplanReportService`, etc.); calculation is a cross-entity invariant |
| Data access for `committed_voluntary` per week | API / Backend (Basic Service) | — | `EmployeeWorkDetailsService` is Basic tier; Phase 15 calls it via `employee_work_details_service.all()` already available in DI |
| `WeeklySummary` service struct field | API / Backend (Service layer) | — | `service::booking_information::WeeklySummary` is the service domain struct; field lives here, not in TO (Phase 16) |
| Snapshot version management | API / Backend (billing_period_report.rs) | — | NOT triggered in Phase 15 (D-01); no persisted value_type changes |
| Frontend display / TO mapping | Frontend (Phase 16) | — | OUT OF SCOPE for Phase 15 |

---

## Critical Open Question — `actual_volunteer` for paid capped persons in Achse B

**The question:** When a capped paid person (`is_paid=true`, `cap_planned_hours_to_expected=true`) has `committed_voluntary=5`, what is `actual_volunteer` in the formula `counted = max(committed, actual_volunteer)` inside `get_weekly_summary`?

### Code Evidence

**`get_weekly_summary` volunteer_hours (lines 141-153):**
```rust
// VERIFIED: booking_information.rs:118-125 / 141-153
let volunteer_ids: Arc<[Uuid]> = self
    .sales_person_service
    .get_all(...)
    .await?
    .iter()
    .filter(|sales_person| !sales_person.is_paid.unwrap_or(false))
    .map(|sales_person| sales_person.id)
    .collect();

let volunteer_hours = self
    .shiftplan_report_service
    .extract_shiftplan_report_for_week(...)
    .await?
    .iter()
    .filter(|report| volunteer_ids.contains(&report.sales_person_id))
    .map(|report| report.hours)
    .sum::<f32>();
```

`volunteer_hours` is STRICTLY the sum of shiftplan hours of `is_paid=false` persons. A capped paid person's actual shiftplan hours are summed into `paid_hours` (line 177-178: `paid_hours += report.dynamic_hours`), NOT into `volunteer_hours`. The cap-overflow (`auto_volunteer_hours`) that Achse A computes lives in `reporting.rs::apply_weekly_cap` → `get_week` → `ShortEmployeeReport.volunteer_hours`, but `get_weekly_summary`'s `paid_hours` accumulates `report.dynamic_hours` (post-cap shiftplan hours), not the raw overflow.

**`get_summery_for_week` (single-week variant) for contrast:** Uses `employee_work_details_service.all()` (line 330-333) for per-day breakdown. This is already available via DI (`EmployeeWorkDetailsService` is a dep of `BookingInformationServiceImpl`, line 37). The year-loop variant `get_weekly_summary` does NOT currently call `employee_work_details_service`, but it COULD (no new DI dependency needed — it's already wired).

**Achse A cap overflow path:** `reporting.rs::get_week` lines 875-877:
```rust
let (shiftplan_hours, auto_volunteer_hours) =
    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours);
let volunteer_hours = manual_volunteer_hours + auto_volunteer_hours;
```
This `volunteer_hours` feeds `ShortEmployeeReport.volunteer_hours`. In `get_weekly_summary`, the loop at lines 176-178 reads `report.dynamic_hours` (not `volunteer_hours`) into `paid_hours`. The cap-overflow from Achse A does NOT flow into Achse B's `volunteer_hours` bucket — it stays in `paid_hours` (since `dynamic_hours` for a capped person = expected_hours = cap boundary).

**Conclusion:** For a capped paid person in Achse B today, their contribution to `volunteer_hours` = 0. Their actual shiftplan hours (including any cap-overflow surplus they generated) are in `paid_hours`. There is NO existing mechanism to extract "cap-overflow for paid persons" inside `get_weekly_summary` without re-deriving Achse A logic.

### Option Analysis

**Option (a): actual_volunteer = cap-overflow for capped paid persons inside Achse B**
- Would require computing `raw_shiftplan_hours_per_person`, `expected_hours_per_person`, and `apply_weekly_cap()` per person inside `get_weekly_summary`.
- Data is technically available: `employee_work_details_service` is already DI'd; `shiftplan_report_service` is already called; `reporting_service.get_week()` already computes this — but iterating get_week results and re-extracting the person-level overflow would duplicate Achse A logic.
- Risk: re-deriving the cap formula in Achse B creates two code paths for the same computation → the exact failure mode of PITFALL P1 "two pre-existing volunteer terms, developer fixes one and forgets the other." This is the PRIMARY pitfall.
- D-01 confirms: "committed_voluntary" is a FORWARD-LOOKING PLEDGE, not a measure of performed hours. The formula `max(committed, actual)` is about "how much should we count as committed volunteer capacity?" — not about recounting overflow.

**Option (b): actual_volunteer = 0 for paid persons in Achse B → term reduces to pure committed_voluntary** [VERIFIED: CORRECT]
- For is_paid=true persons: `max(committed_voluntary, 0) = committed_voluntary`. The term adds exactly the committed capacity, no more, no less.
- For is_paid=false persons (unpaid volunteers): `actual_volunteer` = their shiftplan hours already in `volunteer_hours`. `max(committed_voluntary, actual_volunteer)` = the existing formula with an optional committed floor.
- No double-count: the paid person's `paid_hours` and `committed_voluntary_hours` are separate terms in `overall_available_hours`. Cap-overflow for the paid person is already counted in `paid_hours` (via `dynamic_hours` which = expected_hours for a fully-capped person). Adding `committed_voluntary` as an additional separate capacity term is clean.
- Requires no new DAO calls, no per-person iteration, no cap-math re-derivation.
- Consistent with D-01: "reine Jahresansicht-Verfügbarkeitskapazität (vorausschauende Zusage, KEINE geleistete Stunde)."
- Consistent with the no-double-count goal: `committed_voluntary_hours` is presented as a SEPARATE term alongside `paid_hours` and `volunteer_hours`. It represents "additionally committed capacity" not already in either bucket.

**RECOMMENDATION: Option (b)**

Rationale:
1. **No double-count against Achse B:** The capped paid person's actual work already feeds `paid_hours` in Achse B. The `committed_voluntary_hours` term is NEW capacity (forward-looking pledge), not a recounting of actual work. Adding it separately is the correct semantic.
2. **Minimal new dependencies (CONTEXT: "keine neue DI-Dependency"):** Option (a) would require either a new per-person computation loop inside `get_weekly_summary` or a structural change to how `get_week` results are consumed. Option (b) requires only calling `committed_voluntary_for_calendar_week` (one helper already written) once per week in the existing loop.
3. **D-02 worked examples clarification:** The examples `committed=5/actual=7→7` and `committed=5/actual=3→5` apply to the aggregate per-week `max` before summation. For a capped paid person, `actual_volunteer=0` → `max(5, 0) = 5`. For an unpaid volunteer with 7h shiftplan, `max(5, 7) = 7`. These examples describe the general formula, not specifically "what person class contributes what." The formula is correct for all classes under option (b).
4. **Avoids PITFALL P1:** Re-deriving cap-overflow inside `get_weekly_summary` risks creating a second authoritative source for the same computation.

### Concrete Implementation Pattern (Option b)

For each week in `get_weekly_summary`'s loop:
```rust
// [VERIFIED: committed_voluntary_for_calendar_week is in reporting.rs:101-109]
// actual_volunteer for unpaid persons = volunteer_hours (already computed above)
// actual_volunteer for paid persons = 0 (cap-overflow stays in paid_hours)
// committed = sum of committed_voluntary on active work-details rows for this week
//             where cap_planned_hours_to_expected=true (gate per D-01/CVC-06)
let work_details_for_person: Arc<[EmployeeWorkDetails]> = ...; // from employee_work_details_service.all() for year
let cap_active = find_working_hours_for_calendar_week(&work_details_for_person, year, week)
    .any(|wh| wh.cap_planned_hours_to_expected);
let committed = if cap_active {
    committed_voluntary_for_calendar_week(&work_details_for_person, year, week)
} else {
    0.0
};
// actual_volunteer = volunteer_hours (already computed: sum of shiftplan hours of is_paid=false persons)
let counted_volunteer = committed.max(volunteer_hours);
```

Note: `committed_voluntary_for_calendar_week` is SUM over ALL persons' active rows for the week. This is correct for multi-person aggregation: the yearly loop already aggregates per-week totals. The cap gate applies per person in principle, but since `committed_voluntary = 0.0` for non-capped persons (by CVC-06 convention and `DEFAULT 0` migration), the SUM is naturally zero for non-capped persons.

However, there is a subtlety for multi-person aggregation: if some persons are capped and some are not, the helper sums across ALL persons' active rows. Since uncapped persons have `committed_voluntary = 0.0` by default, they contribute 0.0 to the sum — the gate is semantically enforced by data convention. A belt-and-suspenders option: gate each work-details row by `cap_planned_hours_to_expected` inside the helper call. The current `committed_voluntary_for_calendar_week` does NOT do this — it sums all active rows unconditionally. The planner must decide: filter by cap flag inside the call-site, or accept the data-convention approach. Safest: filter explicitly.

---

## Standard Stack

### Core (no new dependencies — reuse existing)

| Component | Location | Purpose | Status |
|-----------|----------|---------|--------|
| `committed_voluntary_for_calendar_week` | `service_impl/src/reporting.rs:101-109` | SUM helper for per-person committed per week | Ready, written in Phase 14 [VERIFIED] |
| `find_working_hours_for_calendar_week` | `service_impl/src/reporting.rs:77-86` | Week-range selector for `EmployeeWorkDetails` rows | Existing, reuse as-is [VERIFIED] |
| `employee_work_details_service` | DI dep of `BookingInformationServiceImpl` (line 37) | Access to all work-details records | Already DI'd [VERIFIED] |
| `service::booking_information::WeeklySummary` | `service/src/booking_information.rs:38-55` | Target struct for new field | Verified current fields [VERIFIED] |
| `service_impl/src/booking_information.rs::get_weekly_summary` | lines 95-218 | Integration site (Achse B) | Code-scouted in detail [VERIFIED] |

### No New DI Dependency

`BookingInformationServiceImpl` already has `EmployeeWorkDetailsService` as a dependency (confirmed at line 37 of the `gen_service_impl!` block). The year-loop variant `get_weekly_summary` does not currently call it, but no new wiring is required — just one `all()` call added inside the function.

---

## Architecture Patterns

### System Architecture Diagram

```
                 get_weekly_summary (Achse B)
                         │
         ┌───────────────┼──────────────────────────┐
         ▼               ▼                           ▼
reporting_service    shiftplan_report_service    employee_work_details_service
   .get_week()       .extract_shiftplan_report       .all()
         │               │                           │
         ▼               ▼                           ▼
ShortEmployeeReport[]   shiftplan per person     EmployeeWorkDetails[]
  .dynamic_hours  →         │                         │
  (paid capacity)           ▼                    find_working_hours_for_calendar_week
                    filter(!is_paid)             +committed_voluntary_for_calendar_week
                         │                           │
                         ▼                           ▼
                   volunteer_hours            committed (flat, no weight)
                   (actual for unpaid)        (cap-gated: if !cap_active → 0.0)
                         │                           │
                         └──────── max() per week ───┘
                                        │
                                        ▼
                              counted_volunteer_hours_this_week
                                        │
                                 accumulate over year
                                        │
                                        ▼
                          WeeklySummary.committed_voluntary_hours
```

### Recommended Structure (changes only)

```
service/
└── src/
    └── booking_information.rs          # Add committed_voluntary_hours: f32 to WeeklySummary

service_impl/
└── src/
    ├── booking_information.rs          # Integration site: compute term in get_weekly_summary
    │   └── #[cfg(test)] module        # New tests (D-02 fixtures)
    └── reporting.rs                   # committed_voluntary_for_calendar_week (already present,
                                       #   Phase 14) — imported/used by booking_information.rs
```

Note: `committed_voluntary_for_calendar_week` and `find_working_hours_for_calendar_week` are in `reporting.rs` but are `pub` free functions — importable from `booking_information.rs`. Alternatively, the helper can be called via a short inline equivalent. The planner may choose to either import the helper directly or re-express the one-liner inline.

### Pattern: Per-week max before year-sum

```rust
// Source: D-02 requirement (CONTEXT.md), formula from SUMMARY.md
// For each week in the yearly loop in get_weekly_summary:
let committed_this_week: f32 = if cap_active {
    committed_voluntary_for_calendar_week(&all_work_details, year, week)
} else {
    0.0
};
// actual_volunteer = volunteer_hours (already computed as sum of !is_paid shiftplan hours)
let counted_this_week = committed_this_week.max(volunteer_hours);
// Accumulate outside the loop into WeeklySummary.committed_voluntary_hours
```

### Anti-patterns to Avoid

- **Do NOT add `counted_this_week` directly to `overall_available_hours`** — `overall_available_hours` formula is `volunteer_hours + paid_hours` (line 197). Changing it in Phase 15 would affect display before the TO and frontend are ready (Phase 16). Only add the field to the struct; display integration is Phase 16.
- **Do NOT use `max(Σ_committed, Σ_actual)`** — this would sum all weeks then take the max. The correct order is max per week, then sum. D-02 is explicit.
- **Do NOT gate on `is_paid` inside `get_weekly_summary`** — the cap flag on `EmployeeWorkDetails` is the correct gate (CVC-06). Paid status is orthogonal to cap status (Phase 17 will handle the unpaid-volunteer case).
- **Do NOT touch `reporting.rs` or `billing_period_report.rs`** — Achse A is out of scope for Phase 15.
- **Do NOT add WeeklySummaryTO field or From mapping** — Phase 16 only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-week committed sum | Custom iterator over work-details | `committed_voluntary_for_calendar_week` (reporting.rs:101-109) | Already written, tested (4 CVC-03 tests), SUM semantics pinned |
| Week-range selector | Custom date comparison | `find_working_hours_for_calendar_week` (reporting.rs:77-86) | Existing, handles from/to boundary correctly |
| Cap flag check | Custom iteration | `.any(|wh| wh.cap_planned_hours_to_expected)` pattern | Established pattern throughout reporting.rs |
| Float comparison in tests | `==` | `(a - b).abs() < f32::EPSILON` or `(a - b).abs() < 0.001` | D-02 requirement; established in Phase 14 tests |

---

## Code Examples

### existing get_weekly_summary loop structure (Achse B)

```rust
// Source: service_impl/src/booking_information.rs:126-214 [VERIFIED]
for week in 1..=(weeks_in_year + 3) {
    let (year, week) = if week > weeks_in_year {
        (year + 1, week - weeks_in_year)
    } else {
        (year, week)
    };
    // volunteer_hours = sum of shiftplan hours of is_paid=false persons
    let volunteer_hours = self
        .shiftplan_report_service
        .extract_shiftplan_report_for_week(year, week, ...)
        .await?
        .iter()
        .filter(|report| volunteer_ids.contains(&report.sales_person_id))
        .map(|report| report.hours)
        .sum::<f32>();
    // paid_hours from get_week ShortEmployeeReport.dynamic_hours
    let mut paid_hours = 0.0;
    for report in week_report.iter() {
        paid_hours += report.dynamic_hours;
        // ...
    }
    let overall_available_hours = volunteer_hours + paid_hours;
    weekly_report.push(WeeklySummary {
        // ... fields ...
        volunteer_hours,
        overall_available_hours,
        // committed_voluntary_hours: TO BE ADDED
    });
}
```

### committed_voluntary_for_calendar_week helper (from Phase 14)

```rust
// Source: service_impl/src/reporting.rs:101-109 [VERIFIED, Phase 14 delivered]
pub fn committed_voluntary_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32 {
    find_working_hours_for_calendar_week(working_hours, year, week)
        .map(|wh| wh.committed_voluntary)
        .sum()
}
```

### WeeklySummary struct target (current, no committed_voluntary_hours yet)

```rust
// Source: service/src/booking_information.rs:38-55 [VERIFIED]
#[derive(Clone, Debug, PartialEq)]
pub struct WeeklySummary {
    pub year: u32,
    pub week: u8,
    pub overall_available_hours: f32,
    pub required_hours: f32,
    pub paid_hours: f32,
    pub volunteer_hours: f32,
    pub monday_available_hours: f32,
    // ... tuesday through sunday ...
    pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPerson]>,
    // Phase 15 adds: pub committed_voluntary_hours: f32
}
```

### Cap gate pattern (from reporting.rs — established)

```rust
// Source: service_impl/src/reporting.rs:836-837 [VERIFIED, in get_week]
let cap_active = find_working_hours_for_calendar_week(&working_hours, year, week)
    .any(|wh| wh.cap_planned_hours_to_expected);
```

---

## No-Snapshot-Bump Justification (CVC-05 revised)

**Why Phase 15 does NOT trigger a `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump:**

1. `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` lives in `service_impl/src/billing_period_report.rs:74`. [VERIFIED]
2. The bump rule fires when any persisted `BillingPeriodValueType`'s computation or input set changes.
3. `BillingPeriodValueType::Volunteer` (billing_period_report.rs:240-250) is sourced from `report_delta.volunteer_hours` where `report_delta` comes from `reporting_service.get_report_for_employee_range()` — that is Achse A (`reporting.rs`). [VERIFIED]
4. Phase 15 modifies ONLY `booking_information.rs::get_weekly_summary` (Achse B). `WeeklySummary` is the year-view-only struct. `billing_period_report.rs` never reads from `WeeklySummary`. [VERIFIED — billing_period_report.rs contains no reference to WeeklySummary]
5. Therefore: no persisted `value_type` changes in Phase 15 → CLAUDE.md "purely additive changes that do not touch the snapshot's value_types" applies → no bump required.
6. This justification MUST be documented explicitly in the Phase 15 plan (audit trail per D-01).

---

## Common Pitfalls

### Pitfall 1: `max(Σ_committed, Σ_actual)` instead of `Σ max(committed, actual)` per week

**What goes wrong:** Computing year-total committed and year-total actual, then taking max — gives wrong result when committed is high in some weeks and low in others.
**Why it happens:** It looks simpler. One max instead of N maxes.
**How to avoid:** The max is INSIDE the weekly loop, accumulation is OUTSIDE. Test fixture: 2 weeks, week1: committed=5/actual=7→7, week2: committed=5/actual=3→5, total should be 12. If max(Σ,Σ) is used, total = max(10, 10) = 10 ≠ 12.
**Warning signs:** The multi-week test fails.

### Pitfall 2: Adding committed_voluntary_hours to overall_available_hours in Phase 15

**What goes wrong:** `overall_available_hours = volunteer_hours + paid_hours + committed_voluntary_hours` in Phase 15 — but `WeeklySummaryTO` and the frontend don't know about the field yet (Phase 16). Display breaks or shows wrong totals.
**How to avoid:** Phase 15 adds the field to the SERVICE STRUCT only. `overall_available_hours` formula changes in Phase 16 (when the TO and display are wired). The field is inert on the wire until Phase 16.

### Pitfall 3: Not gating committed_voluntary on cap_planned_hours_to_expected

**What goes wrong:** Persons without a cap get their `committed_voluntary` (which should be 0 anyway, but could be non-zero by data error) contributing to capacity.
**How to avoid:** Gate: `if cap_active { committed_voluntary_for_calendar_week(...) } else { 0.0 }`.
**Test:** committed=5, cap=false → contribution = 0.0 (CVC-06).

### Pitfall 4: Loading work-details inside the per-week loop

**What goes wrong:** Calling `employee_work_details_service.all()` inside the `for week in 1..=N` loop → N database roundtrips per year request.
**How to avoid:** Load `all_work_details` ONCE before the loop, reuse it per week via the in-memory helper.

### Pitfall 5: Forgetting cap-filter across multiple persons (multi-person aggregation)

**What goes wrong:** `committed_voluntary_for_calendar_week` sums ALL active rows across ALL persons. If some persons are capped and some are not, non-capped persons contribute `committed_voluntary` from the helper even though they shouldn't.
**How to avoid:** Two approaches: (a) rely on data convention (non-capped persons have `committed_voluntary = 0.0` by migration DEFAULT 0 — valid but fragile), or (b) filter work-details to cap-active persons before calling the helper. Approach (b) is belt-and-suspenders and is recommended. Alternatively, inline the cap check per row: `find_working_hours_for_calendar_week(&work_details, year, week).filter(|wh| wh.cap_planned_hours_to_expected).map(|wh| wh.committed_voluntary).sum()`.
**Test:** Multi-person case with one capped (committed=5) and one non-capped (committed=0 by data) — verify result is 5.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` with `mockall` for service mocks |
| Config file | N/A — no separate config, tests in-module or in `service_impl/src/test/` |
| Quick run command | `cargo test -p service_impl booking_information` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CVC-04 | `max(committed, actual)` per week, not per year | unit | `cargo test -p service_impl committed_voluntary_counts` | ❌ Wave 0 |
| CVC-04 | `committed=5, actual=7 → counted=7` | unit | `cargo test -p service_impl cvc04_over_fulfilled` | ❌ Wave 0 |
| CVC-04 | `committed=5, actual=3 → counted=5` | unit | `cargo test -p service_impl cvc04_under_fulfilled` | ❌ Wave 0 |
| CVC-04 | Sum-over-year: `Σ max(c_w, a_w)` ≠ `max(Σc, Σa)` | unit | `cargo test -p service_impl cvc04_sum_not_max_of_sums` | ❌ Wave 0 |
| CVC-04 | Multi-person aggregation (1 capped + 1 normal) | unit | `cargo test -p service_impl cvc04_multi_person` | ❌ Wave 0 |
| CVC-06 | `cap=false, committed=5 → contribution=0.0` | unit | `cargo test -p service_impl cvc06_cap_false_zero` | ❌ Wave 0 |
| CVC-06 | `committed=0 → result identical to pre-v1.4` | unit | `cargo test -p service_impl cvc06_committed_zero_backward_compat` | ❌ Wave 0 |
| CVC-04 | Single-week: `committed=5, actual=0 → 5` | unit | `cargo test -p service_impl cvc04_zero_actual` | ❌ Wave 0 |
| CVC-04 | Empty week (no work-details rows active) → 0.0 | unit | `cargo test -p service_impl cvc04_empty_week` | ❌ Wave 0 |
| CVC-04 | Boundary: `committed == actual` → committed | unit | `cargo test -p service_impl cvc04_boundary_equal` | ❌ Wave 0 |
| CVC-05 | `CURRENT_SNAPSHOT_SCHEMA_VERSION` = 7 (unchanged) | regression | `cargo test -p service_impl snapshot_schema_version_unchanged` (or grep test) | ❌ Wave 0 |

### D-02 Test Fixtures (complete specification)

Per D-02 (CONTEXT.md), all of the following must be covered. Float comparisons via epsilon (`(a - b).abs() < 0.001`):

| Fixture | committed (per week) | actual_volunteer (per week) | Expected counted_volunteer | Purpose |
|---------|---------------------|----------------------------|---------------------------|---------|
| over_fulfilled | 5.0 | 7.0 | 7.0 | max picks actual |
| under_fulfilled | 5.0 | 3.0 | 5.0 | max picks committed (floor) |
| boundary | 5.0 | 5.0 | 5.0 | boundary: either side correct |
| zero_committed | 0.0 | 7.0 | 7.0 | committed=0 → no change vs today |
| zero_actual | 5.0 | 0.0 | 5.0 | forward-looking pledge with no actuals yet |
| empty_week | (no active rows) | 0.0 | 0.0 | no work-details → committed=0 |
| cap_false | 5.0 (cap=false) | 7.0 | 7.0 (committed not applied, actual passes through) | CVC-06 gate |
| multi_week_sum | W1: c=5,a=7→7; W2: c=5,a=3→5 | — | 12.0 total | Σ max not max Σ |
| multi_person | Person A: cap=true, c=5, a=0; Person B: cap=false, c=0, a=3 | — | committed=5, actual=3 → counted=max(5,3)=5 | aggregation |

Note on multi_person: `committed_voluntary_for_calendar_week` sums across persons; `volunteer_hours` sums shiftplan of is_paid=false persons. The aggregate max is `max(sum_committed, sum_actual)`. For the above: `max(5+0, 0+3) = max(5, 3) = 5`. This is correct — the aggregate formula works at the total level, not per-person.

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl booking_information`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] New test module in `service_impl/src/booking_information.rs` (inline `#[cfg(test)]`) OR in `service_impl/src/test/booking_information_cvc04.rs` — covers all 11 fixtures above
- [ ] `WeeklySummary` struct field `committed_voluntary_hours: f32` in `service/src/booking_information.rs` — required before tests can compile
- [ ] No framework install needed — using existing Rust test infra + mockall

---

## Environment Availability

Step 2.6: Scoped to backend-only code changes. No external service dependencies beyond existing NixOS dev shell.

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|---------|
| `nix develop` | cargo build / cargo test | ✓ | flake.nix based | — |
| `cargo test` | All tests | ✓ | Rust workspace | — |
| SQLite | Not needed (no migration in Phase 15) | ✓ | existing | — |
| sqlx-cli | NOT needed (no migration in Phase 15) | ✓ | in nix develop | — |

**No blocking dependencies.** Phase 15 is purely in-memory calculation logic; no DB migration, no `.sqlx` regen.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `billing_period_report.rs` has no reference to `WeeklySummary` and the no-bump justification is therefore complete | No-Snapshot-Bump Justification | If `WeeklySummary` is somehow consumed in billing, the bump would be needed — would require reverting D-01. LOW risk: research searched the canonical refs provided in CONTEXT.md and confirmed `Volunteer` value_type sources from `report_delta.volunteer_hours` (Achse A) exclusively. |
| A2 | The cap-flag check via `.any(|wh| wh.cap_planned_hours_to_expected)` is sufficient to gate the committed term (non-capped persons have `committed_voluntary = 0.0` by default) | Option (b) recommendation | If a non-capped person has a non-zero `committed_voluntary` by data entry error, they would contribute to the aggregate without the explicit filter. LOW risk: Phase 17 editor will gate input on cap=true; recommending belt-and-suspenders filter anyway. |

**Critical claims verified:** `volunteer_ids` filter (is_paid=false), `paid_hours` accumulation loop, `EmployeeWorkDetailsService` DI wiring, `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`, `committed_voluntary_for_calendar_week` at reporting.rs:101-109, `WeeklySummary` struct fields — all read directly from current HEAD code.

---

## Open Questions (RESOLVED)

1. **Cap filter inside `committed_voluntary_for_calendar_week` call or at the call site?**
   - What we know: the helper sums ALL active rows unconditionally; cap flag must be applied to each row's contribution.
   - What's unclear: whether to filter inside a new inline expression vs. relying on `cap_active` outer gate.
   - Recommendation: inline filter: `.filter(|wh| wh.cap_planned_hours_to_expected).map(|wh| wh.committed_voluntary).sum()` — explicit and safe. The outer `cap_active` check is still useful for the `if cap_active { ... } else { 0.0 }` shortcut.

2. **Single `employee_work_details_service.all()` call vs. one call per person**
   - What we know: `get_summery_for_week` calls `.all()` for all employees; the year-loop `get_weekly_summary` calls only `.get_all()` on `sales_person_service`.
   - What's unclear: whether calling `.all()` once outside the loop and filtering per week in-memory is the right approach (no new async calls inside the loop).
   - Recommendation: call `employee_work_details_service.all()` once before the per-week loop. Filter per-week in-memory using `find_working_hours_for_calendar_week` (pure function, no I/O).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `volunteer_hours` only (is_paid=false shiftplan) | `committed_voluntary_hours` as separate floor term | Phase 15 | Capped paid persons' pledges become visible capacity in year-view |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` | Stays 7 (no bump) | Phase 15 (D-01 revised) | No snapshot invalidation; clean upgrade |
| `WeeklySummary` without committed field | `WeeklySummary` with `committed_voluntary_hours: f32` | Phase 15 | Field on service struct only; inert on wire until Phase 16 |

**Not deprecated/changed:** `overall_available_hours` formula stays `volunteer_hours + paid_hours` until Phase 16. `WeeklySummaryTO` unchanged until Phase 16.

---

## Project Constraints (from CLAUDE.md)

- **Service tier:** `BookingInformationServiceImpl` is Business-Logic tier (consumes `ReportingService`, `ShiftplanReportService`, `EmployeeWorkDetailsService`, etc.) — correct tier for this cross-entity calculation. No change to tier classification.
- **No new DI dependency:** `EmployeeWorkDetailsService` is already in the DI block (line 37). Phase 15 adds one `.all()` call — no `gen_service_impl!` change.
- **Testing:** All new calculation paths require tests (mandatory per project CLAUDE.md + global rule). D-02 fixtures cover all cases.
- **Snapshot versioning rule:** Explicitly NOT triggered (see no-bump justification). The rule says "purely additive changes that do not touch the snapshot's value_types" → no bump.
- **jj-only commits:** No commits from agents; user commits manually. GSD auto-commit disabled (`commit_docs: false`).
- **NixOS:** `nix develop` for all build/test commands. No `sqlx database reset` (no migration in Phase 15 anyway).
- **cargo build + cargo test required:** Executor must run `cargo build --workspace` and `cargo test --workspace` after implementation.
- **No `async` inside helper functions:** `committed_voluntary_for_calendar_week` is a sync pure function — correct pattern.

---

## Sources

### Primary (HIGH confidence — direct code reads at HEAD)

- `service_impl/src/booking_information.rs:95-218, 243-408` — `get_weekly_summary` + `get_summery_for_week`, complete code scouting [VERIFIED]
- `service/src/booking_information.rs:38-55` — `WeeklySummary` struct, current fields [VERIFIED]
- `service_impl/src/reporting.rs:77-109` — `find_working_hours_for_calendar_week` + `committed_voluntary_for_calendar_week` [VERIFIED]
- `service_impl/src/reporting.rs:875-877` — `get_week` auto_volunteer_hours computation (Achse A) [VERIFIED]
- `service_impl/src/reporting.rs:162-165` — `get_reports_for_all_employees` is_paid filter [VERIFIED]
- `service_impl/src/billing_period_report.rs:74, 240-250` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`, `Volunteer` value_type sourcing from `report_delta.volunteer_hours` [VERIFIED]
- `service/src/employee_work_details.rs:14-42` — `EmployeeWorkDetails` struct with `committed_voluntary: f32` and `cap_planned_hours_to_expected: bool` [VERIFIED]
- `service_impl/src/booking_information.rs:28-43` — `gen_service_impl!` DI block confirming `EmployeeWorkDetailsService` already wired [VERIFIED]

### Secondary (HIGH confidence — planning documents)

- `.planning/phases/15-reporting-no-double-count-snapshot-bump-same-commit/15-CONTEXT.md` — locked decisions D-01/D-02/D-03/D-04
- `.planning/phases/14-data-model-foundation-backend/14-01-SUMMARY.md` + `14-02-SUMMARY.md` — Phase 14 delivered state (committed_voluntary_for_calendar_week confirmed present)
- `.planning/research/SUMMARY.md`, `PITFALLS.md`, `ARCHITECTURE.md`, `STACK.md` — v1.4 milestone research (HIGH, verified against HEAD at time of writing)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all referenced code verified in current HEAD; Phase 14 summaries confirm delivered state
- Architecture: HIGH — Achse B integration site fully scouted; data flow traced end-to-end
- Critical open question resolution: HIGH — option (b) verified by tracing what appears in volunteer_hours vs paid_hours; no assumption required
- Pitfalls: HIGH — grounded in direct code reads + v1.4 research pitfalls doc
- Test fixtures: HIGH — D-02 enumerates all required cases; epsilon pattern from Phase 14 established

**Research date:** 2026-06-23
**Valid until:** Until Phase 15 implementation changes any referenced code (stable backend; valid 30+ days)
