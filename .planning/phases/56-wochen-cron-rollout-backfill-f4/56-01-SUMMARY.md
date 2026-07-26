---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 01
subsystem: testing
tags: [rebooking, iso-week, cutoff-gate, golden-snapshot, property-test, billing-period, voluntary-stats]

# Dependency graph
requires:
  - phase: 55-f3-manuelle-umbuchung-hr-alert
    provides: "source==Manual filter at every read-aggregation entry point (reporting.rs), rebook_manual pair-write-path (source=Rebooking)"
  - phase: 54-datenmodell-f1-f2
    provides: "ExtraHoursSource enum, voluntary_rebooking_auto_active_from toggle seed migration, VoluntaryStatsService (ist/soll)"
provides:
  - "voluntary_rebooking_gate module: TOGGLE_NAME, parse_active_from, should_run_for_week, read_active_from (ISO-week cutoff predicate)"
  - "D-06 rebooking-neutrality property tests (balance / F1-Ist / F2-Soll bit-identical after a rebooking pair)"
  - "D-01 straddling golden-snapshot fixture — proves snapshot schema version stays at 12"
affects: [56-02, 56-03, 56-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ISO-week cutoff gate mirrors shortday_gate.rs (Monday resolved via Date::from_iso_week_date, inclusive lower boundary)"
    - "Rebooking-neutrality proven at the real ReportingServiceImpl driven by MockExtraHoursService (not a full SQLite stack) — deterministic, to_bits() bit-identity"
    - "Straddling golden-snapshot drives the real BillingPeriodReportServiceImpl over the real ReportingServiceImpl; toggle value varied per scenario"

key-files:
  created:
    - service_impl/src/voluntary_rebooking_gate.rs
    - service_impl/src/test/rebooking_neutrality.rs
    - service_impl/src/test/billing_period_straddling.rs
  modified:
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs

key-decisions:
  - "D-01 outcome: STAYS AT snapshot version 12 — the straddling fixture is byte-identical across pre/post/straddling/None toggle scenarios; no chain leaks rebooking rows into the snapshot."
  - "read_active_from carries #[allow(dead_code)] with a reason (consumed by Plan 02/03 scheduler write-guard); the shortday_gate analog had a same-phase consumer so did not need this."
  - "Neutrality/straddling fixtures drive real service impls with MockExtraHoursService rather than in-memory SQLite — deterministic and matches the existing rebooking_reconciliation.rs neutrality harness pattern."

patterns-established:
  - "voluntary_rebooking_gate::should_run_for_week is the single ISO-week cutoff predicate for the scheduler write-guard (Plan 02/03)."
  - "Byte-identity via f32::to_bits() with an explicit non-NaN guard is the neutrality/snapshot assertion contract (mirrors D-52-12)."

requirements-completed: [REB-AUTO-02, REB-AUTO-05]

coverage:
  - id: D1
    description: "voluntary_rebooking_gate module (TOGGLE_NAME/parse_active_from/should_run_for_week/read_active_from) with ISO-week cutoff semantics"
    requirement: "REB-AUTO-05"
    verification:
      - kind: unit
        ref: "service_impl/src/voluntary_rebooking_gate.rs (8 unit tests: parse None/empty/malformed/valid, should_run None/pre/at-or-after, week-53 boundary)"
        status: pass
    human_judgment: false
  - id: D2
    description: "D-06 rebooking-neutrality property tests: balance_hours / F1-Ist (volunteer_hours) / F2-Soll (soll_total) bit-identical after a single source=Rebooking pair"
    requirement: "REB-AUTO-02"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_neutrality.rs#test_neutrality_balance,test_neutrality_f1_ist,test_neutrality_f2_soll"
        status: pass
    human_judgment: false
  - id: D3
    description: "D-01 straddling golden-snapshot fixture: billing period spanning a rebooking week produces byte-identical snapshot value rows across pre/post/straddling/None cutoff scenarios"
    requirement: "REB-AUTO-05"
    verification:
      - kind: unit
        ref: "service_impl/src/test/billing_period_straddling.rs#test_straddling_snapshot_pre,test_straddling_snapshot_post,test_straddling_snapshot_straddling"
        status: pass
    human_judgment: false

# Metrics
duration: 9min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 01: Wave-0 Validation Backbone Summary

**ISO-week cutoff gate (`voluntary_rebooking_gate`) plus the D-06 rebooking-neutrality property tests and the D-01 straddling golden-snapshot fixture — the straddling fixture is byte-identical, so `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays at 12.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-07-26T05:45:39Z
- **Completed:** 2026-07-26T05:55:00Z
- **Tasks:** 3
- **Files modified:** 5 (3 created, 2 modified)

## Accomplishments

- **`voluntary_rebooking_gate` module** — the single ISO-week cutoff predicate the Plan 02/03 scheduler write-guard will reuse. `should_run_for_week(iso_year, iso_week, active_from)` resolves the ISO-week Monday via `Date::from_iso_week_date` (year-boundary safe, defensive skip on invalid week) and returns `monday >= cutoff` (inclusive lower boundary), `false` when the feature is dormant (`active_from == None`). 8 unit tests green.
- **D-06 rebooking-neutrality property tests** — a single `source=Rebooking` pair (VolunteerToExtra N=2) leaves `balance_hours`, F1-Ist (`volunteer_hours` == `VoluntaryStats::ist_total`), and F2-Soll (`VoluntaryStats::soll_total`) **bit-identical** (`f32::to_bits()`). Proven with `active_from == None`, so the baseline neutrality holds without the toggle — this is the CI-guard.
- **D-01 straddling golden-snapshot fixture** — a billing period spanning a rebooking week (KW 16/2026) produces byte-identical snapshot value rows whether the cutoff toggle is set after (2026-06-01), before (2026-01-01), inside (2026-04-06), or unset the period. **D-01 outcome: stays at snapshot version 12.**

## D-01 Outcome (the gate)

**STAYS AT 12.** All four straddling scenarios (pre / post / straddling / None) produce byte-identical `BillingPeriodSalesPerson.values` rows. No chain leaks a rebooking row into the snapshot — confirming the Phase-55 `source==Manual` read-aggregation filter is unconditional and toggle-independent. Plan 06 therefore does **NOT** bump `CURRENT_SNAPSHOT_SCHEMA_VERSION`; the conditional 12→13 bump branch is not taken.

## Task Commits

1. **Task 1: voluntary_rebooking_gate module** — `69c0171d` (feat)
2. **Task 2: three rebooking-neutrality property tests** — `14f6ae0e` (test)
3. **Task 3: straddling golden-snapshot fixture** — `77221742` (test)

## Files Created/Modified

- `service_impl/src/voluntary_rebooking_gate.rs` (created) — ISO-week cutoff gate mirroring `shortday_gate.rs`.
- `service_impl/src/test/rebooking_neutrality.rs` (created) — 3 D-06 neutrality property tests.
- `service_impl/src/test/billing_period_straddling.rs` (created) — 3 D-01 straddling golden-snapshot scenarios.
- `service_impl/src/lib.rs` (modified) — register `voluntary_rebooking_gate` module.
- `service_impl/src/test/mod.rs` (modified) — register the two new test modules.

## Decisions Made

- **D-01 stays at 12** (see above) — the straddling fixture is byte-identical, so the "Nein"-branch of D-01 (no snapshot bump) is proven and locked.
- Neutrality/straddling fixtures drive the **real** `ReportingServiceImpl` / `BillingPeriodReportServiceImpl` with a `MockExtraHoursService` (not in-memory SQLite). This matches the existing `rebooking_reconciliation.rs` neutrality harness, is fully deterministic, and exercises the actual `source==Manual` filter site.
- The straddling snapshot is computed via `build_billing_period_report_for_sales_person` (the value-row assembly), not `build_new_billing_period`, so the fixture needs no `BillingPeriodService`/`get_all` orchestration and pins exactly the value rows that matter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `#[allow(dead_code)]` on `read_active_from`**
- **Found during:** Task 1 (voluntary_rebooking_gate module)
- **Issue:** `read_active_from` is only consumed by the Plan 02/03 scheduler write-guard; in Plan 01 (Wave-0 backbone landing first) it is unused, tripping `-D dead-code` under the clippy hard gate. The `shortday_gate.rs` analog did not need this because its consumers landed in the same phase.
- **Fix:** Added `#[allow(dead_code)]` with a reason comment pointing at the downstream Plan 02/03 consumer.
- **Files modified:** service_impl/src/voluntary_rebooking_gate.rs
- **Verification:** `cargo test -p service_impl voluntary_rebooking_gate` green; `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Committed in:** `69c0171d` (Task 1 commit)

**2. [Rule 3 - Blocking] `MockSpecialDayService::get_by_week` expectation in the straddling fixture**
- **Found during:** Task 3 (straddling golden-snapshot fixture)
- **Issue:** The billing-period report calls the reporting fn with `carryover=true` over full-year ranges, which iterates every ISO week and calls `special_day_service.get_by_week(year, week, ctx)`. The bare `MockSpecialDayService::new()` had no expectation → panic. (The narrower neutrality fixture used `carryover=false` within-year and never hit this path.)
- **Fix:** Added `expect_get_by_week().returning(|_,_,_| Ok(empty))` to the straddling fixture's reporting stack (no SpecialDays in this fixture).
- **Files modified:** service_impl/src/test/billing_period_straddling.rs
- **Verification:** All three straddling scenarios pass byte-identity; clippy clean.
- **Committed in:** `77221742` (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 3 blocking test/build issues).
**Impact on plan:** Both were mechanical unblockers for the clippy hard gate and the mock harness; no scope change and no semantic effect on the tested properties.

## Issues Encountered

None beyond the two Rule-3 blockers documented above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `voluntary_rebooking_gate::should_run_for_week` / `read_active_from` are ready for the Plan 02/03 scheduler write-guard.
- D-01 is locked at snapshot version 12 — Plan 06 skips the bump branch (no migration, no F08 version-table change for a bump).
- The neutrality property tests are the standing CI-guard: any future chain that leaks a rebooking row will break them immediately.

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- All 3 created files present on disk.
- All 3 task commits present in git history (`69c0171d`, `14f6ae0e`, `77221742`).
- Full workspace `cargo test` green (service_impl lib 805 passed, +14 from this plan); `cargo clippy --workspace --all-targets -- -D warnings` clean.
