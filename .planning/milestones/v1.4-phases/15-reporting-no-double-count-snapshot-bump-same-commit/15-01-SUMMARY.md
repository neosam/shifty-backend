---
phase: 15-reporting-no-double-count-snapshot-bump-same-commit
plan: 01
subsystem: api
tags: [booking_information, reporting, committed_voluntary, no-double-count, formula-b, cvc-04, cvc-06]

# Dependency graph
requires:
  - phase: 14-data-model-foundation-backend
    provides: "committed_voluntary field on EmployeeWorkDetails + committed_voluntary_for_calendar_week helper in reporting.rs:101-109"
provides:
  - "WeeklySummary.committed_voluntary_hours: f32 (Band 1 — cap-gated pledge sum per ISO week)"
  - "volunteer_surplus_above_committed(actual, committed) -> f32 pure helper (Band 2 per-person floor)"
  - "get_weekly_summary wired with two-band FORMULA B decomposition (D-05)"
  - "get_summery_for_week sets committed_voluntary_hours: 0.0 (inert placeholder, year-view-only)"
affects: [phase-16-weekly-summary-to-mapping, phase-15-plan-02-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "FORMULA B per-person surplus: (actual_p - committed_p).max(0.0) before summing — never subtract aggregate from aggregate"
    - "Cap gate per row: .filter(|wh| wh.cap_planned_hours_to_expected) applied to each EmployeeWorkDetails row independently"
    - "Work-details loaded once before per-week loop (Pitfall 4 guard)"
    - "Pure helper volunteer_surplus_above_committed is pub(crate) for Plan 15-02 test access"

key-files:
  created: []
  modified:
    - service/src/booking_information.rs
    - service_impl/src/booking_information.rs

key-decisions:
  - "D-05 FORMULA B (two-band decomposition): Band 1 = cap-gated Σ_person committed; Band 2 = Σ_person max(actual_p - committed_p, 0) — per-person subtraction mandatory because max is nonlinear and person-set overlap is real"
  - "get_summery_for_week option (a): committed_voluntary_hours: 0.0 inert placeholder; volunteer_hours left at full actual (no surplus reduction) — single-week variant feeds a per-day consumer, year-view Band-2 reduction is Achse-B-only"
  - "overall_available_hours formula stays volunteer_hours + paid_hours — Phase 16 wires display (Pitfall 2 guard)"
  - "reporting.rs and billing_period_report.rs untouched — D-01: no Achse-A modification, no snapshot version bump (stays 7)"

patterns-established:
  - "Two-band stacked capacity model: committed_voluntary_hours (Band 1, pledge color) + volunteer_hours (Band 2, surplus color) — bands never overlap by CVC-04 invariant"
  - "Band-2 per-person surplus via pure helper volunteer_surplus_above_committed ensures per-person max before aggregation"

requirements-completed: [CVC-04, CVC-06]

# Metrics
duration: 35min
completed: 2026-06-24
---

# Phase 15 Plan 01: Two-Band Committed Voluntary Capacity Decomposition Summary

**WeeklySummary gains committed_voluntary_hours (Band 1, pledge) with per-person surplus reduction for volunteer_hours (Band 2) using FORMULA B — no-double-count invariant: committed + max(actual - committed, 0) = max(committed, actual) per person/week**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-06-24T05:00:00Z
- **Completed:** 2026-06-24T05:35:00Z
- **Tasks:** 2 (Task 1 + Task 2, executed atomically)
- **Files modified:** 2

## Accomplishments

- Added `committed_voluntary_hours: f32` field (Band 1) to `WeeklySummary` service struct in `service/src/booking_information.rs`
- Implemented pure per-person surplus helper `volunteer_surplus_above_committed(actual, committed) -> f32` (`pub(crate)` for Plan 15-02 test access)
- Wired both bands into `get_weekly_summary` (Achse B): Band 1 as cap-gated Σ_person committed per week, Band 2 as Σ_person max(actual_p - committed_p, 0) per week (FORMULA B)
- Loaded work-details once before the per-week loop (Pitfall 4 guard)
- Set `committed_voluntary_hours: 0.0` with explanatory comment in `get_summery_for_week` (option (a), year-view-only)
- 4 inline unit tests covering the surplus helper (t1-t4) — all green
- Full workspace: 429 tests, zero failures

## Task Commits

No commits — jj-managed repository; user commits manually. Code changes only.

## Files Created/Modified

- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service/src/booking_information.rs` — Added `committed_voluntary_hours: f32` field to `WeeklySummary` struct with Band 1/2 comments
- `/home/neosam/programming/rust/projects/shifty/shifty-backend/service_impl/src/booking_information.rs` — Added `use crate::reporting::find_working_hours_for_calendar_week` import; `volunteer_surplus_above_committed` pure helper; Band 1 + Band 2 wiring in `get_weekly_summary`; inert 0.0 placeholder in `get_summery_for_week`; 4 inline `#[cfg(test)]` tests

## Decisions Made

### FORMULA B Two-Band Decomposition (D-05)

The plan adopts two stacked bands per the resolved semantic decision from CONTEXT.md D-05:

- **Band 1 `committed_voluntary_hours`:** `Σ_week Σ_person committed` (flat, cap-gated per row via `cap_planned_hours_to_expected`). The Phase-14 helper `committed_voluntary_for_calendar_week` is NOT used directly — instead `find_working_hours_for_calendar_week(...).filter(|wh| wh.cap_planned_hours_to_expected).map(|wh| wh.committed_voluntary).sum()` is inlined to guarantee the per-row cap filter (Pitfall 5 guard).
- **Band 2 `volunteer_hours` (reduced):** `Σ_week Σ_person max(actual_p - committed_p, 0)`. Per-person subtraction is mandatory: `max` is nonlinear, and the plan confirms person-set overlap is real (an `is_paid=false` volunteer can have `committed_voluntary` set). Delegate to `volunteer_surplus_above_committed` before the outer `.sum()`.

No-double-count invariant per person/week: `committed_p + max(actual_p - committed_p, 0) = max(committed_p, actual_p)`.

Multi-person worked example (D-05): Person A (cap=true, c=5, a=0) + Person B (cap=false→committed gated to 0, a=3) → `committed_voluntary_hours = 5.0`, `volunteer_hours = 3.0`, total = **8.0** (FORMULA B, not Formula A's 5.0).

### get_summery_for_week: Option (a) — inert 0.0

`committed_voluntary_hours: 0.0` with comment in `get_summery_for_week`. This variant feeds a per-day breakdown consumer; reducing its `volunteer_hours` to the Band 2 surplus would be an out-of-scope behavior change affecting a different consumer path. The year-view Band 2 surplus reduction lives exclusively in `get_weekly_summary` (year-view path).

### No Snapshot Version Bump (D-01)

`CURRENT_SNAPSHOT_SCHEMA_VERSION` stays at 7. Phase 15 modifies only Achse B (`booking_information.rs::get_weekly_summary`). `WeeklySummary` is year-view-only and is never read by `billing_period_report.rs` — verified: zero references to `WeeklySummary` in `billing_period_report.rs`. No persisted `BillingPeriodValueType` changes. CLAUDE.md "purely additive changes that do not touch the snapshot's value_types" applies.

## Deviations from Plan

None — plan executed exactly as written.

The plan's Task 1 and Task 2 were implemented in a single pass (both tasks modify the same two files), but all specified behaviors, acceptance criteria, and structural decisions were followed precisely.

## Issues Encountered

None — all builds and tests passed on the first attempt.

## User Setup Required

None — no external service configuration required. Pure in-memory calculation change, no DB migration.

## Next Phase Readiness

- Plan 15-02: `volunteer_surplus_above_committed` is `pub(crate)` — accessible from `service_impl/src/test/booking_information.rs` for the D-02 fixture suite (cvc04_*, cvc06_* tests)
- Phase 16: `WeeklySummaryTO` mapping + `From<&WeeklySummary>` update for `committed_voluntary_hours` wire-tier field; frontend two-color stacked band rendering
- `overall_available_hours` formula is deliberately unchanged — Phase 16 will decide how to compose both bands for the display total

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The computation runs inside the existing `get_weekly_summary` service method under the existing `SHIFTPLANNER_PRIVILEGE`/`SALES_PRIVILEGE` permission check. No new threat surface introduced.

## Known Stubs

- `committed_voluntary_hours: 0.0` in `get_summery_for_week` — intentional per decision option (a); resolved in Phase 16 if the per-day consumer needs Band 1 too (currently it does not)

## Self-Check

- `grep -n "committed_voluntary_hours: f32" service/src/booking_information.rs` → line 45: FOUND
- `grep -n "fn volunteer_surplus_above_committed" service_impl/src/booking_information.rs` → line 35: FOUND
- `cargo test --workspace` → 429 tests, 0 failures: PASSED
- `reporting.rs` and `billing_period_report.rs` unmodified (D-01): VERIFIED

## Self-Check: PASSED

All acceptance criteria verified:
- `committed_voluntary_hours: f32` on WeeklySummary: FOUND
- `volunteer_surplus_above_committed` helper with `(actual - committed).max(0.0)`: FOUND
- `cap_planned_hours_to_expected` filter present for both Band 1 and Band 2: FOUND
- `all_work_details` loaded before the `for week` loop (Pitfall 4): CONFIRMED at line 138-141 vs loop at line 142
- `committed_voluntary_hours` set in both WeeklySummary constructions: FOUND
- `overall_available_hours = volunteer_hours + paid_hours` unchanged (Pitfall 2): CONFIRMED
- `WeeklySummary` not referenced in `billing_period_report.rs` (D-01): CONFIRMED (zero matches)
- Full workspace: 429 tests, 0 failures

## CR-01 fix

**Befund (15-REVIEW.md CR-01):** Die DAO-Query `extract_shiftplan_report_for_week` liefert
einen Row pro `(sales_person_id, year, day_of_week)`, weil der SQL-Ausdruck
`GROUP BY sales_person_id, year, day_of_week` gruppiert. Die ursprüngliche Band-2-Implementierung
mappte `volunteer_surplus_above_committed(report.hours, committed_p)` über jeden dieser
Tages-Rows — d.h. das wöchentliche `committed_p` wurde von jedem einzelnen Tagestunden-Wert
subtrahiert, bevor summiert wurde. Da `max(x − c, 0)` nichtlinear ist, führt das bei Personen
mit mehr als einem Arbeitstag pro Woche zur systematischen Unterschätzung des Überschusses:

> Person mit committed=5, Mo 3h + Di 4h (weekly 7h):
> - Korrekt (per-Woche): max(7 − 5, 0) = **2.0**
> - Buggy (per-Tag):     max(3 − 5, 0) + max(4 − 5, 0) = **0.0** ← CR-01

**Fix:** Neuer purer Helper `volunteer_surplus_band2` in `service_impl/src/booking_information.rs`:

```rust
pub(crate) fn volunteer_surplus_band2(
    per_day_actuals: impl IntoIterator<Item = (uuid::Uuid, f32)>,
    committed_for_person: impl Fn(uuid::Uuid) -> f32,
) -> f32
```

Der Helper aggregiert zunächst alle Tages-Rows in eine `HashMap<Uuid, f32>` (wöchentliche
Ist-Stunden pro Person) und wendet dann `volunteer_surplus_above_committed(weekly_actual_p, committed_p)`
einmal pro Person an — erst danach wird über Personen summiert. Die bestehende skalare Hilfsfunktion
`volunteer_surplus_above_committed` bleibt unverändert; `volunteer_surplus_band2` ist ihre
korrekte Kompositionsschicht für den Mehrfach-Tag-Fall.

`get_weekly_summary` ruft nun `volunteer_surplus_band2(per_day_actuals, |sp_id| …)` auf statt
des per-Row-Map. Band 1, `overall_available_hours`, `get_summery_for_week` und alle anderen
Dateien bleiben unverändert (D-01 eingehalten).

**Regressionstests** in `service_impl/src/test/booking_information.rs`:

- `cvc04_multi_day_single_person`: eine Volunteer-Person, committed=5, Mo 3.0h + Di 4.0h →
  `volunteer_surplus_band2` muss 2.0 liefern. Die buggy per-Tag-Form würde 0.0 liefern.
- `cvc04_multi_day_multi_person`: zwei Personen (A: committed=5, Mo 3h + Di 4h; B: committed=0,
  Mo 1.5h + Mi 1.5h) → erwartetes Gesamt-Band-2 = 5.0. Buggy per-Tag-Form würde 3.0 liefern
  (A's Überschuss geht verloren).

Alle 437 `service_impl`-Tests grün, 0 Failures im gesamten Workspace.

---
*Phase: 15-reporting-no-double-count-snapshot-bump-same-commit*
*Completed: 2026-06-24*
