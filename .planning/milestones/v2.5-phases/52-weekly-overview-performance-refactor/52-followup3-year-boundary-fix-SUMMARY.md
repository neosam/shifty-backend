---
phase: 52-weekly-overview-performance-refactor
plan: followup3-year-boundary-fix
subsystem: backend/reporting
tags: [iso-year, year-boundary, extra_hours, special_day, shiftplan_report, reporting, booking_information, sqlite]

requires:
  - phase: 52-weekly-overview-performance-refactor
    provides: [Wave 3 bulk-load year methods (find_by_year / _for_year / get_by_year), Wave 4/5 assemble_weeks + get_weekly_summary consumer bulking, Follow-up #2 chain-A/B/C elimination]

provides:
  - ExtraHoursService::find_by_iso_year (replaces find_by_year)
  - SpecialDayService::get_by_iso_year (new; get_by_year kept for REST endpoint)
  - ShiftplanReportService::extract_shiftplan_report_for_iso_year (replaces _for_year)
  - Matching Trait + Service + DAO + SQLite DAO layers for all three
  - Regression-gate tests at KW 1 / KW 53 (previous bug reproducers flipped)

affects: [phase 53+, any future bulk-load year batches, docs/features/F07 balance semantics]

tech-stack:
  added: []
  patterns:
    - "ISO-week-year naming discipline for bulk methods that feed ISO-bucket consumers"
    - "Semantic-alias DAO methods where DB column semantics already match (special_day, shiftplan_report) — no SQL change but rename clarifies intent"
    - "Range-widening DAO method where DB column semantics differ (extra_hours date_time) — new SQL range [ISO-Mo(y,1), ISO-Su(y,weeks_in_year(y))+1d)"

key-files:
  created:
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup3-latency.txt
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup3-year-boundary-fix-SUMMARY.md
    - service_impl/src/test/booking_information_weekly_summary_year_boundary.rs (moved from untracked)
    - service_impl/src/test/booking_information_weekly_summary_year_boundary_2026.rs (moved from untracked)
    - service_impl/src/test/reporting_year_boundary.rs (moved from untracked)
  modified:
    - dao/src/extra_hours.rs
    - dao/src/shiftplan_report.rs
    - dao/src/special_day.rs
    - dao_impl_sqlite/src/extra_hours.rs
    - dao_impl_sqlite/src/shiftplan_report.rs
    - dao_impl_sqlite/src/special_day.rs
    - service/src/extra_hours.rs
    - service/src/shiftplan_report.rs
    - service/src/special_days.rs
    - service_impl/src/booking_information.rs
    - service_impl/src/extra_hours.rs
    - service_impl/src/reporting.rs
    - service_impl/src/shiftplan_report.rs
    - service_impl/src/special_days.rs
    - service_impl/src/test/mod.rs
    - service_impl/src/test/absence_conversion.rs
    - service_impl/src/test/booking_information_chain_c.rs
    - service_impl/src/test/booking_information_vfa.rs
    - service_impl/src/test/booking_information_weekly_summary_year_batch.rs
    - service_impl/src/test/extra_hours.rs
    - service_impl/src/test/reporting_get_year.rs
    - service_impl/src/test/shiftplan_report.rs

key-decisions:
  - "Use ISO-Wochenjahr naming (_iso_year) on all three new methods, not inline-patch the calendar-year variants (user-chosen fix strategy)."
  - "Delete calendar-year methods extra_hours.find_by_year and shiftplan_report._for_year — grep confirmed zero external consumers."
  - "Keep SpecialDayService::get_by_year (REST endpoint /special-days/year/{y} is an external consumer with calendar-year semantics). Only add get_by_iso_year alongside."
  - "SpecialDayDao::find_by_iso_year and ShiftplanReportDao::_for_iso_year keep the same SQL as their calendar-year predecessors (DB columns are already ISO-year); rename clarifies intent."
  - "ExtraHoursDao::find_by_iso_year uses a new SQL range [ISO-Mo(y,1), ISO-Su(y,weeks_in_year(y))+1d) instead of the calendar Y-01-01/(Y+1)-01-01 range. Max ±3 days wider — negligible perf impact."
  - "No CURRENT_SNAPSHOT_SCHEMA_VERSION bump (Weekly-Overview is not persisted; billing_period_snapshot uses find_by_week / _for_week / get_by_week which are ISO-correct already)."
  - "No new dependencies, no migrations, no frontend changes."

patterns-established:
  - "Byte-identity contract for intra-year rows preserved: 8 Wave-1 fixtures + Chain-C-Toggle-Read invariant all green."
  - "Bug-reproducer tests get flipped to regression-gates after fix (both KW1/KW53 divergence tests, both h4/h4b holiday-boundary tests): assertion inversion, not deletion."

requirements-completed: []

# No coverage: block — this is a bugfix follow-up, tracked via regression tests.

duration: ~1h
completed: 2026-07-06
status: complete
---

# Phase 52 Follow-up #3: Year-Boundary Fix Summary

**Fix three symmetric ISO-week-year vs. calendar-year bulk-load bugs in the Weekly-Overview endpoint by introducing `_iso_year` variants on ExtraHours, SpecialDay, and ShiftplanReport (service + DAO), deleting the calendar-year variants that had no external consumers, and flipping four bug-reproducer tests to regression gates.**

## Performance

- **Started:** 2026-07-06T (Dev-Session)
- **Completed:** 2026-07-06T
- **Tasks:** 3 new methods + 2 consumer switches + 3 test updates + 1 latency + 1 summary
- **Files modified:** 22
- **Files created:** 3 test files registered + 1 latency file + this SUMMARY

## Accomplishments

- **Root-cause fix at the DAO + Service axis, not inline-patched at consumers.** All three offending bulk-load methods now expose ISO-week-year semantics that structurally match the consumer's ISO-week-year bucketing in `assemble_weeks` and `get_weekly_summary`. Rows at KW 1 (Y+1) — which starts in Y in the calendar (e.g. Mo 2019-12-30 for ISO-2020-W1) — and KW 53 (Y) — which ends in Y+1 in the calendar (e.g. Fr 2027-01-01 for ISO-2026-W53) — are now correctly present in the bulk-load result for `iso_year=Y`.
- **Two Wave-3 methods deleted** (`extra_hours.find_by_year`, `shiftplan_report._for_year` at both Trait+Service+DAO+SQLite levels) — grep confirmed only Phase-52 consumers, which were switched in the same wave. No external callers.
- **`SpecialDayService::get_by_year` preserved** — it has an external REST consumer (`GET /special-days/year/{year}` in `rest/src/special_day.rs`) with legitimate calendar-year semantics (frontend shows special days by calendar year). Only added `get_by_iso_year` alongside.
- **Four bug-reproducer tests flipped to regression gates** (previously green because they asserted DIVERGENCE bulk-vs-legacy; now green because they assert CONVERGENCE with the fixed semantics). The rest of the test infrastructure — 8 Wave-1 fixtures, 4 slot cross-checks, 5 additional ISO-2026-boundary tests, Chain-C-toggle-read invariant — all remain green.
- **Latency confirmed unchanged.** Median-of-medians ~0.093s, well under WOP-04 target of <0.5s. Follow-up #2's chain-A/B/C elimination remains the dominant win; the ±3-day widening of the ExtraHours range is invisible in the measurement.

## Task Commits

Each atomic step was committed separately:

1. **feat(52)** — `f86043a` — `find_by_iso_year` on ExtraHours (trait + service + DAO + SQLite; deletes old `find_by_year`).
2. **feat(52)** — `091b203` — `get_by_iso_year` on SpecialDayService (added alongside `get_by_year`, which stays for REST).
3. **feat(52)** — `6ac06b6` — `extract_shiftplan_report_for_iso_year` (deletes old `_for_year`; service switches from `get_by_year`+post-filter to `get_by_iso_year`).
4. **refactor(52)** — `e6e6ba3` — switch `assemble_weeks` (in `reporting::get_year`) and `get_weekly_summary` to the new ISO-year bulk methods.
5. **test(52)** — `d3fc242` — convert bug-reproducer tests to regression gates + register the three test files in `mod.rs`; rename mocks/stubs to new method names.
6. **perf(52)** — `ba2a025` — post-fix latency measurement.
7. **docs(52)** — pending (this SUMMARY commit).

## Files Created/Modified

**DAO trait layer:**
- `dao/src/extra_hours.rs` — trait method renamed and re-documented.
- `dao/src/special_day.rs` — new `find_by_iso_year` method (semantic alias since the DB column is already ISO-year).
- `dao/src/shiftplan_report.rs` — renamed and re-documented.

**SQLite DAO impl layer:**
- `dao_impl_sqlite/src/extra_hours.rs` — new SQL range `[ISO-Mo(y,1), ISO-Su(y,weeks_in_year(y))+1d)` using `time::Date::from_iso_week_date` and `next_day()`. Uses the same offline `.sqlx` cache entry (SQL text unchanged; only Rust binding variables differ).
- `dao_impl_sqlite/src/special_day.rs` — new `find_by_iso_year` with same SQL as `find_by_year` (semantic alias).
- `dao_impl_sqlite/src/shiftplan_report.rs` — renamed method, same SQL body.

**Service trait layer:**
- `service/src/extra_hours.rs` — trait method renamed.
- `service/src/special_days.rs` — added `get_by_iso_year` alongside `get_by_year`.
- `service/src/shiftplan_report.rs` — trait method renamed.

**Service impl layer:**
- `service_impl/src/extra_hours.rs` — impl renamed, delegates to new DAO.
- `service_impl/src/special_days.rs` — added `get_by_iso_year` impl that delegates directly to `DAO::find_by_iso_year` (no Union(y, y-1), no calendar-year post-filter). `get_by_year` unchanged.
- `service_impl/src/shiftplan_report.rs` — impl renamed AND now calls `get_by_iso_year` (was calling `get_by_year` and then post-filtering with `if sd.year == year` — the very calendar-vs-ISO mismatch that dropped SpecialDays at KW 1 / KW 53).

**Consumers:**
- `service_impl/src/booking_information.rs` — `get_weekly_summary` switches both `special_day_service.get_by_year(*)` calls to `get_by_iso_year(*)`, and both `extract_shiftplan_report_for_year(*)` calls to `_for_iso_year(*)`.
- `service_impl/src/reporting.rs` — `get_year` switches `extra_hours_service.find_by_year` and `shiftplan_report_service.extract_shiftplan_report_for_year` to their ISO-year variants.

**Tests:**
- `service_impl/src/test/mod.rs` — registered three previously-untracked test files (`booking_information_weekly_summary_year_boundary`, `booking_information_weekly_summary_year_boundary_2026`, `reporting_year_boundary`).
- `service_impl/src/test/absence_conversion.rs` — `StubExtraHoursService::find_by_year` → `find_by_iso_year`.
- `service_impl/src/test/reporting_year_boundary.rs` — full rewrite: assertions inverted from "bulk and legacy DIVERGE at ISO-KW-1/KW-53" to "bulk and legacy CONVERGE at 4.5h / 3.0h vacation_hours". Mocked `by_year[2020]` now contains the boundary row (simulating the new DAO's ISO-range return), where before it was empty.
- `service_impl/src/test/booking_information_weekly_summary_year_boundary_2026.rs` — h4 and h4b assertions inverted: previously asserted `bulk W53/W1 = 8.0` (bug: Holiday not seen) and `drift = +8h`; now assert `bulk == legacy == 0.0` (Holiday correctly dropped in both paths). Docstrings updated to describe the regression-gate intent.
- All mock renames: `expect_find_by_year` → `expect_find_by_iso_year`, `expect_extract_shiftplan_report_for_year` → `_for_iso_year`, and — in consumer-tests only — `expect_get_by_year` → `expect_get_by_iso_year`. `test/special_days.rs` deliberately keeps `expect_find_by_year` / `expect_get_by_year` because it tests the calendar-year semantics behind the REST endpoint, which are unchanged.

## Decisions Made

- **User-chosen strategy: add new `_iso_year` methods, don't inline-patch the calendar-year ones.** Followed exactly.
- **Delete only what has zero external consumers.** Grep-verified: `extra_hours.find_by_year` and `shiftplan_report._for_year` had only Phase-52 consumers (`reporting::get_year`, `booking_information::get_weekly_summary`); safe to delete. `SpecialDayService::get_by_year` has one REST-endpoint external consumer (`rest/src/special_day.rs::get_special_days_for_year`) with legitimate calendar-year semantics — kept.
- **For DAOs where the DB column already IS ISO-week-year** (`special_day.year`, `booking.year`), the "new" `_iso_year` method is a semantic alias with byte-identical SQL. The rename earns its keep by making the calling-side error-detectable in code review: a future engineer who adds a bulk-loader consuming `_year` gets to see the naming ambiguity in the trait.
- **No `.sqlx` regeneration needed.** All three new SQL queries have byte-identical text to their predecessors (only Rust binding variables changed), so the offline query cache reuses the same hash. Verified via `SQLX_OFFLINE=true cargo test --workspace` (green).

## Deviations from Plan

**None** — plan executed exactly as written.

The `<execution_steps>` in the prompt anticipated I might discover that `SpecialDayDao::find_by_year` and `ShiftplanReportDao::_for_year` are already ISO-year-correct at the SQL level; that was confirmed. The bug in both was purely at the **service layer** (Union+calendar-post-filter for SpecialDay, and `get_by_year`+`if sd.year == year` post-filter for ShiftplanReport). The rename-plus-service-fix approach cleanly addresses both.

## Issues Encountered

- Initial test run after code-only changes showed all 16 boundary tests still passing — including h4/h4b that asserted the buggy +8h drift. This was because the mocks used method-specific per-year buckets (`sd_by_year[2026]=empty`, `sd_by_year[2027]=[row]`) that mocked the OLD `get_by_year` calendar-year behavior. After the fix the consumer calls `get_by_iso_year`, but the mock returned the same "empty" for 2026 — so the drift was preserved by the mock, not by the code. Fixed by updating the mock's per-year data to reflect the NEW DAO semantics (`sd_by_year[2026]=[row]`), then flipping the assertions to "bulk == legacy == 0.0".

## Verification

- `cargo test --workspace` — **873 passed, 0 failed** (including 8 Wave-1 fixtures, 16 year-boundary tests, Chain-C toggle-read invariant, all integration tests).
- `cargo clippy --workspace -- -D warnings` — **clean** (WOP-04 clippy hard gate preserved).
- `SQLX_OFFLINE=true cargo test --workspace` — **all green** (CI-compatible; no `.sqlx` delta needed).
- Latency: `GET /booking-information/weekly-resource-report/2026` — median-of-medians ~0.093s (Wave-0 baseline 2.33s; WOP-04 target <0.5s). See `52-followup3-latency.txt`.

## Next Phase Readiness

- Fix is self-contained. No follow-up work required.
- Weekly-Overview endpoint now consistent across intra-year and year-boundary weeks.
- Recommendation: docs update pass (`docs/features/F07-reporting-balance.md` and `docs/domain/time-accounting.md` could add a note about ISO-week-year vs. calendar-year semantics — but the CLAUDE.md docs-freshness-gate does NOT trigger here because none of the trigger files (auth, migrations, snapshot-schema) were touched).

---
*Phase: 52-weekly-overview-performance-refactor*
*Completed: 2026-07-06*
