---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 03
subsystem: voluntary-rebooking-scheduler
tags: [rebooking, auto-cron, scheduler, tokio-cron-scheduler, iso-week, di-wiring, f4]

# Dependency graph
requires:
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 02
    provides: "RebookingReconciliationService::run_for_week (trigger-agnostic F4 engine the scheduler drives)"
  - phase: 48-pdf-export
    provides: "PdfExportSchedulerImpl blueprint (JobScheduler lifecycle, clone_for_job handle, 6-field cron)"
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 01
    provides: "voluntary_rebooking_gate (toggle-gate consumed inside run_for_week)"
provides:
  - "VoluntaryRebookingScheduler trait (start + run_once_now) + Impl (BL-tier, 1:1 pdf_export mirror minus reload_from_db)"
  - "Weekly Monday 03:15 cron (6-field 0 15 3 * * 1) booted at end of main() after pdf_export_scheduler.start()"
  - "run_once_now: year-boundary-safe Vorwoche computation + per-capped-person delegation to run_for_week(AutoCron)"
affects: [56-04, 56-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Scheduler = thin orchestration: computes Vorwoche + iterates capped persons; all F4 logic stays in run_for_week (single write-path preserved)"
    - "No reload_from_db (unlike pdf_export): the stichtag-toggle is read fresh each run inside run_for_week, not compiled into a cron string"
    - "Hardcoded 6-field cron 0 15 3 * * 1 (Monday 03:15) — conflict-free vs pdf_export Monday 06:00 (D-03); 6 fields mandatory (v2.3.1 hotfix)"
    - "clone_for_job copies all Arc handles into a lightweight Handle so self is never moved into the async cron closure"
    - "Vorwoche via ShiftyDate::from_date(clock.date_now()).as_shifty_week() + year-boundary-safe previous_week step-back (never naive year()+week())"
    - "Per-person run_for_week error is logged and the loop continues (T-56-07 — no panic propagation from the cron closure)"

key-files:
  created:
    - service/src/voluntary_rebooking_scheduler.rs
    - service_impl/src/voluntary_rebooking_scheduler.rs
    - service_impl/src/test/voluntary_rebooking_scheduler.rs
  modified:
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - shifty_bin/src/main.rs

key-decisions:
  - "Scheduler dep set is the MINIMAL set it actually consumes: RebookingReconciliationService + EmployeeWorkDetailsService (capped-person source) + PermissionService + ClockService + TransactionDao. The plan's interface_context listed SalesPersonService/ToggleService/WeekStatusService as deps, but the scheduler needs none of them — run_for_week handles the toggle-gate and week-lock internally, and the capped-person source is employee_work_details all_for_week (interface_context line 68), NOT SalesPersonService. Adding unused deps would be dead wiring."
  - "No RestStateDef accessor for the scheduler. pdf_export_scheduler has a trait accessor because of its /pdf-export-config/trigger REST endpoint; the voluntary rebooking scheduler has NO REST surface in this plan (the REST backfill endpoint is Plan 04). It is a plain RestStateImpl field so main() can call .start() on it — adding a trait accessor now would be dead API surface."
  - "Tests are mock-based (mirror Plan 02's voluntary_rebooking.rs harness), not a fresh in-memory SQLite stack. The plan text said 'in-memory stack', but the codebase's F4 test infrastructure is mockall-based. The tests build the REAL RebookingReconciliationServiceImpl (with mocked DAO/service deps) wired into the scheduler, so run_once_now -> run_for_week is exercised for real — the delegation is genuine, only the leaf DAOs are mocked."

requirements-completed: [REB-AUTO-01]

coverage:
  - id: T1
    description: "run_once_now books exactly one AutoCron pair for a capped person with excess in the Vorwoche; deterministic clock (2026-03-16 KW12) proves the processed week is the last completed ISO week KW11/2026"
    requirement: "REB-AUTO-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/voluntary_rebooking_scheduler.rs::run_once_now_books_excess_for_vorwoche"
        status: pass
    human_judgment: false
  - id: T2
    description: "active_from=None -> full no-op (assert-by-absence: no batch, no pair written)"
    requirement: "REB-AUTO-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/voluntary_rebooking_scheduler.rs::run_once_now_toggle_off_is_full_noop"
        status: pass
    human_judgment: false
  - id: T3
    description: "uncapped person in the Vorwoche is filtered out (engine never entered)"
    requirement: "REB-AUTO-01"
    verification:
      - kind: unit
        ref: "service_impl/src/test/voluntary_rebooking_scheduler.rs::run_once_now_skips_uncapped_person"
        status: pass
    human_judgment: false

# Metrics
duration: 12min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 03: F4 Auto-Cron Scheduler (VoluntaryRebookingScheduler) Summary

**The weekly automation surface for REB-AUTO-01: a thin `tokio-cron-scheduler` job (Monday 03:15, 6-field cron) that computes the last completed ISO week and delegates every capped SalesPerson to the Plan-02 `run_for_week` engine — a 1:1 mirror of `PdfExportSchedulerImpl` minus `reload_from_db`, wired into `main.rs` DI and booted right after `pdf_export_scheduler.start()`.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-07-26T06:18:54Z
- **Completed:** 2026-07-26T06:30:11Z
- **Tasks:** 3
- **Files:** 7 (3 created, 4 modified)
- **New tests:** 3 (service_impl lib 822 -> 825), full workspace green

## Accomplishments

- **`VoluntaryRebookingScheduler` trait** (`service/src/voluntary_rebooking_scheduler.rs`) — `start()` + `run_once_now(context)` only. No `reload_from_db`: the stichtag-toggle is read fresh each run inside `run_for_week`, not compiled into a cron expression. `#[automock]` provides `MockVoluntaryRebookingScheduler` for downstream tests.
- **`VoluntaryRebookingSchedulerImpl`** (`service_impl/src/voluntary_rebooking_scheduler.rs`) — BL-tier, `gen_service_impl!` with 5 deps + `scheduler`/`current_job` custom_fields, field-for-field mirroring `PdfExportSchedulerImpl`:
  - `start()`: lazily creates the `JobScheduler`, registers a `Job::new_async` with the hardcoded 6-field cron `0 15 3 * * 1` (constant `VOLUNTARY_REBOOKING_CRON`), stores the job id, logs `"cron job registered ... (Monday 03:15)"`, then `sched.start()`.
  - `run_once_now()`: `check_only_full_authentication` gate -> compute Vorwoche via `ShiftyDate::from_date(clock.date_now()).as_shifty_week()` + year-boundary-safe `previous_week` step-back -> `employee_work_details.all_for_week(cap-filter)` -> per capped person `run_for_week(sp_id, iso_year, iso_week, AutoCron, Full, None)`; a per-person error is logged and the loop continues (T-56-07); logs a `{booked} booked, {skipped} skipped, {failed} failed` summary.
  - `clone_for_job()` -> `VoluntaryRebookingSchedulerHandle` copies all Arc handles so `self` is never moved into the cron closure.
- **main.rs DI wiring** — `VoluntaryRebookingSchedulerDependencies` alias + `Deps` impl (5 associated types); construction after the `pdf_export_scheduler` block (all deps already in scope: `rebooking_reconciliation_service`, `working_hours_service`, `permission_service`, `clock_service`, `transaction_dao`); `RestStateImpl` field + field-init; boot `rest_state.voluntary_rebooking_scheduler.start()` immediately after the pdf_export boot in `main()`.
- **3 integration tests** driving the real reconciliation engine through the scheduler (see coverage).

## Task Commits

1. **Task 1: VoluntaryRebookingScheduler trait + impl** — `ef8419eb` (feat)
2. **Task 2: main.rs DI wiring + boot** — `1a396307` (feat)
3. **Task 3: run_once_now delegation + toggle-off no-op tests** — `644af9ac` (test)

## Files Created/Modified

- `service/src/voluntary_rebooking_scheduler.rs` (created) — trait (start + run_once_now, no reload_from_db) + automock.
- `service/src/lib.rs` (modified) — registered `pub mod voluntary_rebooking_scheduler`.
- `service_impl/src/voluntary_rebooking_scheduler.rs` (created) — BL-tier impl, 6-field cron const, `previous_week` helper, `clone_for_job` handle, Debug impl.
- `service_impl/src/lib.rs` (modified) — registered `pub mod voluntary_rebooking_scheduler`.
- `shifty_bin/src/main.rs` (modified) — imports (`VoluntaryRebookingScheduler as _`, impl types), Deps alias + impl, construction, struct field + field-init, boot.
- `service_impl/src/test/voluntary_rebooking_scheduler.rs` (created) — 3 tests (real engine + mocked leaves).
- `service_impl/src/test/mod.rs` (modified) — registered the test module.

## Decisions Made

- **Minimal dep set (not the plan's literal list).** The scheduler consumes exactly what it uses: `RebookingReconciliationService` (the engine), `EmployeeWorkDetailsService` (capped-person source = `all_for_week` cap-filter, same source as `run_for_week`/`backfill_range`), `PermissionService` (full-auth gate), `ClockService` (Vorwoche), `TransactionDao` (custom-field wiring parity). The plan's interface_context also listed `SalesPersonService`/`ToggleService`/`WeekStatusService`, but the scheduler needs none — the toggle-gate and week-lock live inside `run_for_week`, and the capped-person source is employee_work_details (interface_context line 68), not SalesPersonService. Wiring unused deps would be dead code and fail clippy-adjacent review.
- **No RestStateDef accessor.** The voluntary rebooking scheduler has no REST surface in this plan (the REST backfill endpoint is Plan 04), so it is a plain `RestStateImpl` field rather than a `RestStateDef` trait accessor (which pdf_export has only because of its `/trigger` endpoint). Adding an accessor now would be dead API.
- **Mock-based tests over a fresh in-memory stack.** Mirrors Plan 02's `voluntary_rebooking.rs` harness (the actual F4 test infrastructure). The tests build the REAL `RebookingReconciliationServiceImpl` wired into the scheduler, so `run_once_now -> run_for_week` delegation is genuinely exercised; only leaf DAOs/services are mocked.

## Deviations from Plan

### Clarifications (design-level, no behavior change vs must-haves)

**1. [Rule 1 - Clarification] Dep set trimmed to what the scheduler actually consumes**
- **Found during:** Task 1 (gen_service_impl! deps) + Task 2 (main.rs Deps impl)
- **Issue:** The plan's interface_context listed 7 deps incl. SalesPersonService/ToggleService/WeekStatusService, but the scheduler's own logic needs none of them (they live inside `run_for_week`); the capped-person source is employee_work_details `all_for_week`.
- **Fix:** 5-dep set (RebookingReconciliationService, EmployeeWorkDetailsService, PermissionService, ClockService, TransactionDao). All must-have truths still hold (weekly 6-field cron, run_once_now processes the Vorwoche for every capped person via run_for_week, full no-op at active_from=None).
- **Files:** service_impl/src/voluntary_rebooking_scheduler.rs, shifty_bin/src/main.rs
- **Commit:** `ef8419eb`, `1a396307`

**2. [Rule 1 - Clarification] No RestStateDef accessor (no REST surface this plan)**
- **Found during:** Task 2 (main.rs accessor step)
- **Issue:** Plan said "mirror pdf_export_scheduler at 796/957-958" for the accessor, but that accessor exists for the pdf trigger endpoint; this scheduler has no REST endpoint until Plan 04.
- **Fix:** Plain `RestStateImpl` field + field-init + boot; no `RestStateDef` associated type/accessor.
- **Files:** shifty_bin/src/main.rs
- **Commit:** `1a396307`

**3. [Rule 1 - Clarification] Mock-based tests (not fresh in-memory SQLite)**
- **Found during:** Task 3
- **Issue:** Plan text said "in-memory stack"; the F4 test infrastructure is mockall-based.
- **Fix:** Real reconciliation engine + mocked leaves; delegation exercised for real. Added a 3rd test (uncapped-person filter) for the cap-guard.
- **Files:** service_impl/src/test/voluntary_rebooking_scheduler.rs
- **Commit:** `644af9ac`

**Total deviations:** 3 clarifications. No scope change; all must-have truths satisfied.

## TDD Gate Compliance

`tdd_mode` is active for this phase. This plan's Task 3 is the explicit test task (the plan orders impl -> wiring -> tests). The scheduler impl (Task 1) is thin orchestration whose behavior is only observable through delegation to the real engine, so the meaningful behavior tests land in Task 3 against the completed impl. All 3 new tests were confirmed passing after the impl; the toggle-off and uncapped-person tests are assert-by-absence (a stray write would panic the mock), and the excess test asserts the exact AutoCron batch + source=Rebooking pair for the deterministic Vorwoche. No test passed unexpectedly against a missing implementation.

## Verification

- `cargo build -p service_impl` — green (Task 1)
- `cargo build` (full binary) — green (Task 2, DI wiring compiles)
- `cargo test -p service_impl voluntary_rebooking_scheduler` — 3 passed
- `cargo test --workspace` — green (service_impl lib 825 passed, +3 from this plan; no regressions)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean (hard gate)

## Known Stubs

None. The scheduler is fully wired to the real `run_for_week` engine and the real `all_for_week` capped-person source; no hardcoded/placeholder data paths.

## Threat Flags

None. The scheduler introduces no HTTP surface (REST backfill endpoint is Plan 04) and no package installs. The three threats in the plan's register are mitigated:
- **T-56-07** (one person's error aborting the run): per-person `run_for_week` error is logged, loop continues; the cron closure catches and logs any `run_once_now` error (no panic propagation).
- **T-56-08** (year-boundary wrong week): Vorwoche via `ShiftyDate::from_date().as_shifty_week()` + `previous_week` step-back; the deterministic-clock test asserts KW11/2026 for a 2026-03-16 clock.
- **T-56-09** (5-field cron -> dormant job): hardcoded 6-field `0 15 3 * * 1`; `start()` logs `"cron job registered"`.

## Deferred Items

- **Docs-Freshness-Gate (CLAUDE.md):** this plan touched trigger files `service/**/*.rs` (new trait) and `shifty_bin/src/main.rs` (DI wiring). Per the plan and `<project_gates>`, the docs sync (`docs/architecture/02-service-tiers.md` + `docs/architecture/diagrams/service-graph-runtime.mmd`, and the F4 feature doc) is centralized in **Plan 06** — NOT duplicated here. Noted so the milestone-close gate catches it.

## Next Phase Readiness

- The scheduler boots and drives `run_for_week(AutoCron)` weekly; REB-AUTO-01 automation surface complete.
- Plan 04 (REST backfill endpoint `POST /admin/rebooking/backfill`) is independent of this file; if it needs the scheduler via REST later, it can add the `RestStateDef` accessor at that point.
- Plan 06 owns the docs-sync for the main.rs DI change (service-tier doc + runtime graph) and the F4 feature doc.

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- All 3 created source/test files present on disk.
- SUMMARY present on disk.
- All 3 task commits present in git history (`ef8419eb`, `1a396307`, `644af9ac`).
- Full workspace `cargo test` green (service_impl lib 825 passed, +3); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo build` (full binary) green.
