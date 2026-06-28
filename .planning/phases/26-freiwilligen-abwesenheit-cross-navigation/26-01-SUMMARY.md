---
phase: 26-freiwilligen-abwesenheit-cross-navigation
plan: "01"
subsystem: api
tags: [rust, service_impl, booking_information, absence, volunteer, weekly_summary, di]

requires:
  - phase: 25-holiday-auto-credit
    provides: CURRENT_SNAPSHOT_SCHEMA_VERSION=11 (no bump required here, D-26-02)
  - phase: 16-ehrenamt-band1-band2
    provides: volunteer_surplus_band2 helper + committed_voluntary_hours two-band decomposition
  - phase: 01-absence-domain
    provides: AbsenceService trait with find_all, AbsencePeriod domain model (from_date/to_date inclusive D-05)

provides:
  - period_overlaps_week pure helper in booking_information.rs (category-agnostic, whole-week-out)
  - AbsenceService dependency wired into BookingInformationServiceImpl via gen_service_impl! + main.rs
  - Absence-driven committed_voluntary exclusion in get_weekly_summary (Band 1 + Band 2)
  - 8 VFA-01 unit tests for period_overlaps_week and whole-week-out in test/booking_information.rs

affects:
  - 26-02 (cross-navigation plan — reads WeeklySummary.committed_voluntary_hours, now absence-aware)
  - any future plan that extends get_weekly_summary volunteering logic

tech-stack:
  added: []
  patterns:
    - "VFA-01 whole-week-out: any absence overlap in [Mon, Sun] → volunteer's committed drops to 0 for both bands (not pro-rated)"
    - "Load-once-before-loop: all_absences loaded once before the week loop, mirrors all_work_details optimisation"
    - "Category-agnostic absence exclusion: period_overlaps_week takes only dates; category not an input"
    - "Band-2 consistency: absent volunteer's committed set to 0.0 in committed_for_person closure so surplus math stays correct"

key-files:
  created: []
  modified:
    - service_impl/src/booking_information.rs
    - service_impl/src/test/booking_information.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "D-26-01 honored: find_all used (category-agnostic) — Vacation + SickLeave + UnpaidLeave all exclude volunteers"
  - "D-26-02 honored: CURRENT_SNAPSHOT_SCHEMA_VERSION stays 11 — get_weekly_summary is live year-view, not persisted"
  - "D-26-03 honored: whole-week-out — any overlap of Mon–Sun calendar week → full exclusion (not per-day pro-ration)"
  - "D-26-04 honored: holiday/special_days code path untouched — reduction keys off AbsencePeriods only"
  - "DI no-cycle preserved: AbsenceServiceImpl does NOT consume BookingInformationService (D-Phase3-18)"
  - "Band-2 consistency decision (26-CONTEXT): absent volunteer returns 0.0 from committed_for_person closure to prevent surplus overstating"
  - "Load-once pattern: all_absences loaded once before week loop (not per-week N×find_overlapping_for_booking)"

patterns-established:
  - "period_overlaps_week: from <= week_sunday && to >= week_monday (standard open-interval overlap, both sides inclusive)"
  - "absent_volunteer_ids HashSet built per week inside the loop from pre-loaded all_absences"

requirements-completed: [VFA-01, VFA-02]

coverage:
  - id: D1
    description: "AbsenceService wired into BookingInformationServiceImpl — DI no cycle, main.rs construction order preserved"
    requirement: VFA-01
    verification:
      - kind: unit
        ref: "cargo build -p service_impl -p shifty_bin (compilation gate)"
        status: pass
    human_judgment: false
  - id: D2
    description: "period_overlaps_week helper — pure date-based overlap test, category-agnostic, pub(crate)"
    requirement: VFA-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_overlap_absence_fully_inside_week"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_overlap_ends_exactly_on_monday_inclusive"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_overlap_starts_exactly_on_sunday_inclusive"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_no_overlap_before_week"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_no_overlap_after_week"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_overlap_multiweek_spanning_whole_week"
        status: pass
    human_judgment: false
  - id: D3
    description: "Whole-week-out (D-26-03): absent volunteer contributes 0 to Band 1, not pro-rated per day"
    requirement: VFA-01
    verification:
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_whole_week_out_d2603_not_prorated"
        status: pass
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#vfa01_non_absent_volunteer_unaffected"
        status: pass
    human_judgment: false
  - id: D4
    description: "CURRENT_SNAPSHOT_SCHEMA_VERSION stays 11 (D-26-02 no-bump)"
    requirement: VFA-02
    verification:
      - kind: unit
        ref: "service_impl/src/test/booking_information.rs#snapshot_schema_version_pinned_at_10"
        status: pass
    human_judgment: false

duration: 30min
completed: 2026-06-28
status: complete
---

# Phase 26 Plan 01: VFA-01 Absence-Driven Committed Reduction Summary

**Volunteer absences (Vacation/SickLeave/UnpaidLeave) now exclude the volunteer's committed_voluntary pledge from both Band-1 and Band-2 in get_weekly_summary for the overlapping calendar week (whole-week-out, not pro-rated), via AbsenceService wired into BookingInformationService.**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-06-28T17:00:00Z
- **Completed:** 2026-06-28T17:29:51Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- `period_overlaps_week(from, to, week_monday, week_sunday) -> bool` added as `pub(crate)` helper in `booking_information.rs` — purely date-based, category-agnostic, whole-week-out semantics (D-26-01/D-26-03)
- `AbsenceService` dependency wired into `BookingInformationServiceImpl` via `gen_service_impl!` and `main.rs` `BookingInformationServiceDependencies`; `absence_service` already constructed before `booking_information_service` in `main.rs` — no DI cycle, no construction-order change
- In `get_weekly_summary`: all absences loaded once before the week loop; per-week `absent_volunteer_ids` HashSet built via `period_overlaps_week`; Band-1 `committed_voluntary_hours` filter excludes absent volunteers; Band-2 `committed_for_person` closure returns `0.0` for absent volunteers (consistency)
- 8 new pure-helper unit tests added: boundary coverage for `period_overlaps_week` (inside, Mon inclusive, Sun inclusive, before, after, multi-week) plus whole-week-out and non-absent-unaffected assertions

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire AbsenceService dependency** - `191948c` (feat)
2. **Task 2: VFA-01 absence-driven committed reduction** - `42ad701` (feat)
3. **Task 3: VFA-01 pure-helper unit tests** - `5f22f69` (test)

## Files Created/Modified

- `service_impl/src/booking_information.rs` — added `period_overlaps_week` helper, `AbsenceService` dep in `gen_service_impl!`, absence load-once before loop, per-week `absent_volunteer_ids` set, Band-1 filter, Band-2 closure guard
- `service_impl/src/test/booking_information.rs` — 8 new VFA-01 tests: `vfa01_overlap_*`, `vfa01_no_overlap_*`, `vfa01_whole_week_out_d2603_not_prorated`, `vfa01_non_absent_volunteer_unaffected`; added `period_overlaps_week`, `HashSet`, `date!`, `Uuid` imports
- `shifty_bin/src/main.rs` — `type AbsenceService = AbsenceService` in `BookingInformationServiceDependencies`; `absence_service: absence_service.clone()` in `booking_information_service` construction

## Decisions Made

- **Band-2 consistency (26-CONTEXT):** absent volunteer returns `0.0` from the `committed_for_person` closure so `volunteer_surplus_band2` sees no committed pledge for that week. This keeps Band-1 and Band-2 consistent — both exclude the absent volunteer's pledge. Without this, an absent volunteer's committed would still be subtracted from their actual hours in Band-2, producing incorrect surplus math.
- **Load-once vs. per-week find_overlapping:** `find_all` once before the loop preferred over N×`find_overlapping_for_booking` calls (matches `all_work_details` pattern, Pitfall 4). Since the full absence list is typically small, the memory overhead is negligible relative to the N extra DB round-trips.
- **Authentication::Full for absence load:** matches every sibling internal load in `get_weekly_summary` (`get_all`, `reporting.get_week`, `all`). System-internal context; no caller-supplied input crosses into the absence query (T-26-02 accepted).

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 26-01 complete: `get_weekly_summary` now reflects volunteer absences in `committed_voluntary_hours`
- Plan 26-02 (cross-navigation UI links) can proceed independently — it reads the WeeklySummary REST response which now has correct absence-aware committed hours

## Self-Check

- [x] `service_impl/src/booking_information.rs` — exists, modified
- [x] `service_impl/src/test/booking_information.rs` — exists, modified (8 new tests)
- [x] `shifty_bin/src/main.rs` — exists, modified
- [x] Commit `191948c` — Task 1 DI wiring
- [x] Commit `42ad701` — Task 2 absence reduction logic
- [x] Commit `5f22f69` — Task 3 pure-helper tests
- [x] `CURRENT_SNAPSHOT_SCHEMA_VERSION` = 11 (unchanged)
- [x] `cargo build --workspace` — pass
- [x] `cargo test --workspace` — 492 service_impl tests + others, all pass
- [x] `cargo clippy --workspace -- -D warnings` — clean

## Self-Check: PASSED

---
*Phase: 26-freiwilligen-abwesenheit-cross-navigation*
*Completed: 2026-06-28*
