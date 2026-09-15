---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 02
subsystem: rebooking-reconciliation
tags: [rebooking, auto-cron, backfill, iso-week, idempotency, claim-on-suggest, f4-self-guard]

# Dependency graph
requires:
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 01
    provides: "voluntary_rebooking_gate::{should_run_for_week, read_active_from} (F4-Self-Guard write-side cutoff)"
  - phase: 55-f3-manuelle-umbuchung-hr-alert
    provides: "internal rebook() pair-write-path (source=Rebooking), single write-path (Pitfall 15)"
  - phase: 54-datenmodell-f1-f2
    provides: "RebookingBatchKind::{AutoCron, AutoCronBackfill}, RebookingBatchState::SkippedLocked, UNIQUE week index, VoluntaryStatsService"
provides:
  - "RebookingReconciliationService::run_for_week (trigger-agnostic F4 engine: one ISO-week for one capped SalesPerson)"
  - "RebookingReconciliationService::backfill_range (dry-run preview vs apply, HR-Admin gated)"
  - "RunForWeekOutcome / SkipReason / BackfillPreviewItem public domain types"
  - "week_excess pure helper (pub(crate), unit-tested excess formula)"
affects: [56-03, 56-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "run_for_week reuses the Phase-55 internal rebook() pair-write-path — single write-path for cron/backfill/manual (Pitfall 15)"
    - "F4-Self-Guard on the write side via voluntary_rebooking_gate::should_run_for_week (read side stays neutral from Phase 55)"
    - "Idempotency + Claim-on-Suggest are the same mechanism: RebookingBatch::create UNIQUE conflict -> Skipped{AlreadyProcessed}"
    - "Locked week persists a SkippedLocked audit batch (idempotent via .ok()), never crashes (D-04)"
    - "backfill_range delegates apply-mode writes to run_for_week(kind=AutoCronBackfill) so no second write path exists"
    - "ISO-week iteration via ShiftyWeek::iter_until + previous_week helper (year-boundary safe, never naive year()+week())"

key-files:
  created:
    - service_impl/src/test/voluntary_rebooking.rs
    - service_impl/src/test/rebooking_backfill.rs
  modified:
    - service/src/rebooking_reconciliation.rs
    - service_impl/src/rebooking_reconciliation.rs
    - service_impl/src/voluntary_rebooking_gate.rs
    - service_impl/src/test/rebooking_reconciliation.rs
    - service_impl/src/test/mod.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "Excess formula = ist_total - committed_voluntary (both = VoluntaryStats fields for the Mo..=So range). VoluntaryStats::soll_total IS the committed-voluntary target, so 'Soll' and 'committed_voluntary' in the plan text are the same value — implemented as week_excess(ist, 0.0, committed) to avoid double-subtraction. BackfillPreviewItem.voluntary_soll == committed_voluntary (documented, single value)."
  - "run_for_week does NOT re-run the HR permission check: it is an internal engine invoked by the scheduler (Full auth) and by backfill_range (which gates HR-Admin first). Data reads use Authentication::Full inside the tx."
  - "WeekStatusService construction moved earlier in main.rs (before RebookingReconciliationService) to honour basic-before-business-logic tier order (CLAUDE.md); ShiftplanEditService reuses the same Arc."

requirements-completed: [REB-AUTO-01, REB-AUTO-03, REB-AUTO-04]

coverage:
  - id: T1
    description: "run_for_week: 7 behavior cases (Booked-excess, NoExcess, AlreadyProcessed idempotency, Claim-on-Suggest, Locked-audit + noop, ToggleOff x2, year-boundary W53) + week_excess matrix"
    requirement: "REB-AUTO-01, REB-AUTO-03"
    verification:
      - kind: unit
        ref: "service_impl/src/test/voluntary_rebooking.rs (11 tests)"
        status: pass
    human_judgment: false
  - id: T2
    description: "backfill_range: dry_run writes nothing, apply persists AutoCronBackfill, already-processed skip, active_from gating, HR gate, uncapped exclusion, year-boundary walk"
    requirement: "REB-AUTO-04"
    verification:
      - kind: unit
        ref: "service_impl/src/test/rebooking_backfill.rs (7 tests)"
        status: pass
    human_judgment: false

# Metrics
duration: 15min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 02: F4 Auto-Cron Engine (run_for_week + backfill_range) Summary

**The trigger-agnostic F4 engine: `run_for_week` processes one ISO week for one capped SalesPerson (excess -> rebooking pair, else skip), and `backfill_range` iterates historical weeks with a dry-run preview — both reuse the Phase-55 `rebook()` single write-path and the Phase-54 UNIQUE index for idempotency and Claim-on-Suggest.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-07-26T06:00:56Z
- **Completed:** 2026-07-26T06:15:25Z
- **Tasks:** 2 (both TDD)
- **Files modified:** 8 (2 created, 6 modified)
- **New tests:** 18 (11 run_for_week + 7 backfill), full workspace green

## Accomplishments

- **`run_for_week(sales_person_id, iso_year, iso_week, kind, ctx, tx) -> RunForWeekOutcome`** — the F4 core. Flow: F4-Self-Guard (`should_run_for_week` write-side cutoff; toggle off / pre-cutoff -> `Skipped{ToggleOff}` before any data read or tx) → Locked-week check (`WeekStatus::Locked` -> `SkippedLocked` audit batch, idempotent, then `Skipped{WeekLocked}`, D-04) → excess = `ist - committed` from `VoluntaryStats` (Mo..=So range) → `<= 0` -> `Skipped{NoExcess}` → else claim the UNIQUE slot with an `Approved` batch, and on success write the `VolunteerToExtra` pair via the existing `rebook()`. A UNIQUE conflict (earlier cron / manual / pending HR-suggestion) -> `Skipped{AlreadyProcessed}` (REB-AUTO-03 idempotency + Claim-on-Suggest D-02 — the same mechanism).
- **`backfill_range(from, dry_run, ctx, tx) -> Vec<BackfillPreviewItem>`** — HR-Admin gated (T-56-04, first await). Upper bound clamped to the last completed previous ISO week (`previous_week(clock.date_now())`, DoS-bound T-56-05). Iterates `from..=upper` via `ShiftyWeek::iter_until` (year-boundary safe), and per capped SalesPerson (`all_for_week`, `cap_planned_hours_to_expected=true`) builds a `BackfillPreviewItem`. `dry_run=true` writes nothing; `dry_run=false` delegates booking to `run_for_week(kind=AutoCronBackfill)` — single write-path preserved.
- **Public domain types** `RunForWeekOutcome { Booked{hours} | Skipped{reason} }`, `SkipReason { NoExcess, AlreadyProcessed, WeekLocked, ToggleOff }`, `BackfillPreviewItem` (ist/soll/committed/excess/would_book/skipped/skip_reason) added to the service trait.
- **`week_excess(ist, soll, committed)` pure helper** (`pub(crate)`, unit-tested incl. NaN/Inf -> 0 defensive clamp).
- **DI wiring:** 4 new deps (`WeekStatusService`, `EmployeeWorkDetailsService`, `ToggleService`, `ClockService`) added to `RebookingReconciliationServiceDeps`; `WeekStatusService` construction moved before the reconciliation service in `main.rs` to keep basic-before-business-logic tier order.

## Task Commits

1. **Task 1: run_for_week F4 auto-cron engine** — `5d1b266c` (feat)
2. **Task 2: backfill_range dry-run/apply + year-boundary tests** — `69c45d07` (test)

Note: because `backfill_range` was implemented alongside `run_for_week` in Task 1's impl edit (both methods live in the same trait/impl and share helpers), the Task-2 commit is test-only (`test(...)`). The GREEN implementation for both methods landed in the Task-1 `feat` commit. See TDD Gate Compliance below.

## Files Created/Modified

- `service/src/rebooking_reconciliation.rs` (modified) — trait: added `run_for_week` + `backfill_range` + the 3 public types; imports `RebookingBatchKind`, `ShiftyWeek`.
- `service_impl/src/rebooking_reconciliation.rs` (modified) — impl of both methods, `week_excess`, `make_batch`, `week_voluntary_figures` helpers, free `previous_week`; 4 new deps in `gen_service_impl!`.
- `service_impl/src/voluntary_rebooking_gate.rs` (modified) — removed the stale `#[allow(dead_code)]` on `read_active_from` (now consumed by this plan).
- `service_impl/src/test/voluntary_rebooking.rs` (created) — 11 run_for_week tests.
- `service_impl/src/test/rebooking_backfill.rs` (created) — 7 backfill_range tests.
- `service_impl/src/test/rebooking_reconciliation.rs` (modified) — updated Phase-55 test harness `TestDeps`/`build_service` for the 4 new deps.
- `service_impl/src/test/mod.rs` (modified) — registered the two new test modules.
- `shifty_bin/src/main.rs` (modified) — 4 new dep bindings; moved `WeekStatusService` construction earlier.

## Decisions Made

- **Excess = `ist - committed`, not `ist - (soll + committed)`.** `VoluntaryStats::soll_total` is defined (voluntary_stats.rs) as `committed_voluntary_target_in_range` — i.e. the committed-voluntary target itself. The plan's must-have truth `Ist > Soll + committed_voluntary` and its `BackfillPreviewItem` list both `voluntary_soll` and `committed_voluntary`, but the interface note sources *committed* from `soll_total`. Treating them as two distinct subtrahends would double-subtract the committed pledge. Implemented as `week_excess(ist, 0.0, committed)` (= `ist - committed`), and the preview item reports `voluntary_soll == committed_voluntary` (same value). This matches the Phase-55 neutrality domain (voluntary ist vs committed pledge).
- **`run_for_week` skips a redundant HR gate.** It is an internal engine: the scheduler (Plan 03) calls it under `Authentication::Full`, and `backfill_range` gates HR-Admin before delegating. Internal data reads use `Authentication::Full` inside the tx (mirrors `suggest`/`approve`).
- **`WeekStatusService` moved up in `main.rs`.** Required by the reconciliation service now; moved its construction before the reconciliation service (deps were all already in scope) so basic-tier lands before business-logic-tier, per CLAUDE.md. `ShiftplanEditService` reuses the same `Arc`.

## Deviations from Plan

### Auto-fixed / clarified

**1. [Rule 1 - Clarification] Excess formula de-double-counted**
- **Found during:** Task 1 (run_for_week excess computation)
- **Issue:** Literal reading of the plan (`ist - (soll + committed)`) double-subtracts because `soll_total` already IS the committed-voluntary target.
- **Fix:** `week_excess(ist, 0.0, committed)` = `ist - committed`; preview item reports `voluntary_soll == committed_voluntary`. Documented above.
- **Files:** service_impl/src/rebooking_reconciliation.rs
- **Commit:** `5d1b266c`

**2. [Rule 3 - Blocking] Phase-55 test harness + main.rs deps updated for 4 new deps**
- **Found during:** Task 1 (adding deps to the trait impl)
- **Issue:** Adding `WeekStatusService`/`EmployeeWorkDetailsService`/`ToggleService`/`ClockService` to `RebookingReconciliationServiceDeps` broke the existing `rebooking_reconciliation.rs` test `TestDeps`/`build_service` and the `main.rs` DI wiring (missing trait items / struct fields).
- **Fix:** Added the 4 associated types + mock fields to the Phase-55 test harness; added the 4 dep bindings in `main.rs` and reordered `WeekStatusService` construction.
- **Files:** service_impl/src/test/rebooking_reconciliation.rs, shifty_bin/src/main.rs
- **Commit:** `5d1b266c`

**3. [Rule 3 - Blocking] Removed stale `#[allow(dead_code)]` on `read_active_from`**
- **Found during:** Task 1 (F4-Self-Guard consumer)
- **Issue:** The Plan-01 allow (with "unused in Plan 01" reason) is now consumed by this plan; leaving it would be a stale/misleading attribute.
- **Fix:** Removed the allow, updated the comment to point at the Plan-02 consumer.
- **Files:** service_impl/src/voluntary_rebooking_gate.rs
- **Commit:** `5d1b266c`

**Total deviations:** 1 clarification + 2 blocking mechanical unblockers. No scope change.

## TDD Gate Compliance

This is a `type: tdd` plan. Both methods were driven by the RED/GREEN cycle during development. In the final commit sequence the GREEN implementation for both `run_for_week` and `backfill_range` landed in the Task-1 `feat` commit (`5d1b266c`) because the two methods share a trait, impl block, and helpers (`week_excess`, `week_voluntary_figures`, `make_batch`, `previous_week`) and could not be split into separate compilable commits without a throwaway stub. The Task-1 commit includes the run_for_week test module; the Task-2 `test` commit (`69c45d07`) adds the backfill test module against the already-landed implementation. All 18 new tests were confirmed failing→passing during development; no test passed unexpectedly in a RED phase.

## Verification

- `cargo test -p service_impl voluntary_rebooking` — 11 passed
- `cargo test -p service_impl rebooking_backfill` — 7 passed
- `cargo test --workspace` — green (service_impl lib 822 passed, +18 from this plan)
- `cargo build` — workspace + shifty_bin build (DI wiring correct)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean (hard gate)

## Known Stubs

None. Both methods are fully wired to real service calls; no hardcoded/placeholder data paths.

## Deferred Items

- **Docs-Freshness-Gate (CLAUDE.md):** this plan touched trigger files `service/**/*.rs` (new trait methods/enums) and `shifty_bin/src/main.rs` (DI wiring). Per the docs gate, `docs/architecture/02-service-tiers.md` + `docs/features/F*.md` (F4 auto-cron) should be updated. Deferred to phase close / `/gsd-docs-update` (standard for mid-phase wave execution); noted here so the milestone-close gate catches it.

## Next Phase Readiness

- `run_for_week` is ready for the Plan-03 scheduler (`VoluntaryRebookingScheduler` calls it per capped SalesPerson for the last completed week under `Authentication::Full`).
- `backfill_range` is ready for the Plan-04 REST endpoint (`POST /admin/rebooking/backfill?from=&dry_run=`), which maps `BackfillPreviewItem` to a TO and adds pagination + range sanity (T-56-05 REST-layer bound).

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- Both created test files present on disk.
- Both task commits present in git history (`5d1b266c`, `69c45d07`).
- Full workspace `cargo test` green (service_impl lib 822 passed); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo build` green.
