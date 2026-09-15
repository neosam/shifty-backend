---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 06
subsystem: docs
tags: [docs-freshness-gate, rebooking, f4-cron, backfill, snapshot-versioning, service-tiers, i18n-sync]

# Dependency graph
requires:
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 01
    provides: "D-01 outcome — straddling golden-snapshot byte-identical → snapshot stays at 12 (drives the F14/F08 wording)"
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 03
    provides: "VoluntaryRebookingScheduler trait/impl + main.rs DI wiring (drives the service-tiers + runtime-graph docs)"
provides:
  - "F14 (EN+DE) documents F4 auto-cron scheduler + rollout backfill endpoint as shipped, with the D-01 no-bump snapshot decision"
  - "F07 (EN+DE) records rebooking-neutral read chains covering the F4 writers + the three D-06 neutrality property tests"
  - "F08 (EN+DE) records the Phase 54–56 non-bump confirmation with the straddling-fixture proof"
  - "02-service-tiers.md + service-graph-runtime.mmd carry the new VoluntaryRebookingScheduler BL-service + edges"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Docs-Freshness-Gate: every EN doc edited in lockstep with its _de.md twin, verified against shipped code (grep of real signatures) before writing"

key-files:
  created: []
  modified:
    - docs/features/F14-rebooking.md
    - docs/features/F14-rebooking_de.md
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
    - docs/features/F08-billing-period.md
    - docs/features/F08-billing-period_de.md
    - docs/architecture/02-service-tiers.md
    - docs/architecture/diagrams/service-graph-runtime.mmd

key-decisions:
  - "Snapshot schema version STAYS AT 12 — F4 does NOT bump to 13 (D-01, mirrored exactly from the Plan-01 SUMMARY straddling-fixture outcome). The stale F14 'bumps snapshot 12 → 13' line was corrected in both EN and DE; the F08 'deferred to Phase 56' bump wording was replaced with the resolved no-bump confirmation."
  - "Documented the shipped MINIMAL scheduler dep set (RebookingReconciliation + EmployeeWorkDetails/WorkingHours), not the plan's original interface_context list (SalesPerson/Toggle/WeekStatus) — matched Plan-03's actual gen_service_impl! + main.rs wiring, verified by grep."
  - "Backfill endpoint documented as HR-gated (HR_PRIVILEGE, verified in rebooking_reconciliation.rs) rather than a distinct 'HR-Admin' privilege — the route nests under /admin/rebooking but the service gate is HR_PRIVILEGE."

requirements-completed: [REB-AUTO-05]

# Metrics
duration: 7min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 06: Docs-Freshness-Gate Sync (F4 Cron + Backfill) Summary

**Brought the `docs/` reference in sync with the Phase-56 code diff in both EN and DE — F14 now documents the F4 auto-cron scheduler + rollout backfill endpoint as shipped with the D-01 "snapshot stays at 12" decision, F07/F08 record the rebooking-neutral read chains and the straddling-fixture proof, and the service-tiers doc + runtime graph carry the new `VoluntaryRebookingScheduler` BL-service.**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-07-26T07:02:01Z
- **Completed:** 2026-07-26T07:08:54Z
- **Tasks:** 3
- **Files modified:** 8 (0 created, 8 modified)

## Accomplishments

- **Task 1 — F14 (EN+DE):** flipped the F4 slice from `planned` to `shipped (Phase 56)`; rewrote its description to cover the `VoluntaryRebookingScheduler` (Monday 03:15 cron), the trigger-agnostic `run_for_week` engine, the `AutoCron` / `AutoCronBackfill` batch kinds, the `voluntary_rebooking_auto_active_from` stichtag toggle (write-side self-guard), Claim-on-Suggest interaction with F5, and the `SkippedLocked` audit row for `Locked` weeks. Added a full **§9 F4 Auto-Cron + Rollout Backfill** section (engine, scheduler, `POST /admin/rebooking/backfill` endpoint with dry-run/apply + HR gate, and the §9.4 snapshot decision). **Corrected the stale `bumps snapshot schema 12 → 13` line** to the D-01 outcome (stays at 12, straddling fixture as proof) in the slice table, §7 Related Features, and the conclusion. DE twin updated identically.
- **Task 2 — F07 + F08 (EN+DE):** F07 §2.6 now notes the neutrality filter covers **every** `Rebooking`-source writer (F3/F5 **and** the F4 cron + backfill), with the three Phase-56 D-06 property tests (`test_neutrality_balance` / `_f1_ist` / `_f2_soll`) cited as the standing CI-guard. F08 §7.4 rewrote the v2.6 note into a Phase 54–56 non-bump confirmation, replacing the "12 → 13 deferred to Phase 56" wording with the resolved **no-bump** decision backed by the D-01 straddling golden-snapshot fixture.
- **Task 3 — architecture:** `02-service-tiers.md` lists the new BL-tier `VoluntaryRebookingScheduler` with its minimal dep set and boot order (after `pdf_export_scheduler` in `main()`); `service-graph-runtime.mmd` adds the scheduler node + edges to `RebookingReconciliation` and `WorkingHours` (EmployeeWorkDetails), mirroring the `PdfExportScheduler` node style.

## Snapshot Version — Confirmed STAYS AT 12

**`CURRENT_SNAPSHOT_SCHEMA_VERSION` = 12, no bump.** Mirrored exactly from the Plan-01 SUMMARY (D-01): the straddling golden-snapshot fixture is byte-identical across pre/post/straddling/None toggle scenarios, so F4 rebooking never reaches any persisted `value_type` (every billing-period input chain filters `source == Manual`). The docs now record this as the resolved REB-AUTO-05 "no bump" branch — no migration, no `billing_period_snapshot_locking.rs` assert change.

## Code-Verification Before Writing

Every documented code claim was grep-verified against the shipped source (not invented):

- Cron `0 15 3 * * 1` / `VOLUNTARY_REBOOKING_CRON` — `service_impl/src/voluntary_rebooking_scheduler.rs:49`.
- Scheduler dep set (RebookingReconciliation + EmployeeWorkDetails + Permission + Clock + Transaction) — `gen_service_impl!` in the same file.
- `AutoCron` / `AutoCronBackfill` batch kinds — `dao/src/rebooking_batch.rs:19-27` (`RebookingBatchKind`).
- `SkippedLocked` state + `RunForWeekOutcome::{Booked,Skipped}` + `SkipReason::{AlreadyProcessed,WeekLocked,ToggleOff}` — `service/src/rebooking_reconciliation.rs`.
- Backfill route `POST /admin/rebooking/backfill`, `from`/`dry_run`/`page`/`per_page` params, 400-before-service parse, `HR_PRIVILEGE` gate — `rest/src/rebooking.rs` + `rest/src/lib.rs:753` (`/admin/rebooking` nest) + `service_impl/src/rebooking_reconciliation.rs:676`.

## Task Commits

1. **Task 1: F14 EN+DE — F4 cron + backfill + D-01 snapshot decision** — `be59eb47` (docs)
2. **Task 2: F07 + F08 EN+DE — neutral read chains + snapshot pin** — `d4b37cf5` (docs)
3. **Task 3: 02-service-tiers.md + service-graph-runtime.mmd** — `9bef8d8f` (docs)

## Files Modified

- `docs/features/F14-rebooking.md` / `_de.md` — F4 slice shipped, §9 added, stale 12→13 corrected.
- `docs/features/F07-reporting-balance.md` / `_de.md` — §2.6 neutrality note extended to F4 writers + D-06 tests.
- `docs/features/F08-billing-period.md` / `_de.md` — §7.4 Phase 54–56 non-bump confirmation with straddling-fixture proof.
- `docs/architecture/02-service-tiers.md` — new BL-service listing.
- `docs/architecture/diagrams/service-graph-runtime.mmd` — new node + edges.

## Deviations from Plan

### Clarifications (documented shipped reality, not the plan's literal wording)

**1. [Rule 1 - Clarification] Documented the minimal shipped scheduler dep set**
- **Found during:** Task 3
- **Issue:** The plan's Task-3 action said the scheduler "consumes RebookingReconciliationService + SalesPersonService + ToggleService + WeekStatusService" and to draw edges to Toggle/WeekStatus/SalesPerson. The shipped code (Plan 03) uses the MINIMAL dep set — `RebookingReconciliationService` + `EmployeeWorkDetailsService` (+ Permission/Clock/Transaction) — because the toggle-gate and week-lock live inside `run_for_week`, and the capped-person source is `EmployeeWorkDetailsService::all_for_week`, not `SalesPersonService`.
- **Fix:** Documented the real deps + drew only the two real edges (`--> RebookingReconciliation`, `--> WorkingHours`), with a diagram comment explaining why there are no Toggle/WeekStatus edges. Documenting the plan's list would have described wiring that was never built (`<project_gates>`: "Do not document behavior that was not built").
- **Files:** docs/architecture/02-service-tiers.md, docs/architecture/diagrams/service-graph-runtime.mmd
- **Commit:** `9bef8d8f`

**2. [Rule 1 - Clarification] Backfill gate documented as HR_PRIVILEGE (not a distinct HR-Admin privilege)**
- **Found during:** Task 1
- **Issue:** `<project_gates>` described the backfill endpoint as "HR-Admin gate". The shipped `backfill_range` checks `HR_PRIVILEGE` (`service_impl/src/rebooking_reconciliation.rs:676`); the "admin" qualifier refers to the `/admin/rebooking` route nest, not a separate privilege constant.
- **Fix:** Documented it as HR-gated (non-HR → HTTP 403) with the route living under the `/admin/rebooking` nest — accurate to the code.
- **Files:** docs/features/F14-rebooking.md, docs/features/F14-rebooking_de.md
- **Commit:** `be59eb47`

**Total deviations:** 2 clarifications. No scope change; both align the docs to shipped code rather than the plan's pre-implementation phrasing.

## Verification

- **F14 EN+DE:** `grep -l "AutoCronBackfill"` matches both files; `grep -c "12 → 13\|12 -> 13"` = 0 in both (stale bump wording gone).
- **F07/F08 EN+DE:** `grep -c "neutral\|Rebooking\|source"` > 0 in all four; no lingering "deferred to Phase 56" / "auf Phase 56 verschoben" bump wording in F08.
- **02-service-tiers.md + mmd:** `grep -c "VoluntaryRebookingScheduler\|Voluntary Rebooking Scheduler"` = 1 (doc) / 4 (mmd); node declared inside the Business subgraph, 2 edges, balanced `subgraph`/`end` (3/3), valid `-->` syntax. (`mmdc` not installed — structural validity confirmed by grep; no CLI render available.)
- **Docs-only change:** no Rust source touched → `cargo build` / clippy unaffected (not re-run for a pure-docs plan, per plan `<verification>`).

## Known Stubs

None — pure documentation update.

## Threat Flags

None. Documentation only; no code, no network surface, no package installs. The docs now record the T-56-01 snapshot-drift mitigation (straddling fixture) so future maintainers understand why the version stays at 12.

## Next Phase Readiness

- Docs-Freshness-Gate satisfied for every Phase-56 trigger file: `main.rs` DI (→ 02-service-tiers + runtime graph), new `service/**` traits (→ F14), reporting/billing semantics (→ F07 + F08). EN + DE synchronized.
- REB-AUTO-05 closed — the snapshot decision is recorded with the empirical straddling-fixture proof.
- Phase 56 is code-complete (Plans 01–05) and docs-complete (Plan 06); ready for phase verification / milestone-close docs gate.

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- All 8 modified doc files present on disk.
- SUMMARY present on disk.
- All 3 task commits present in git history (`be59eb47`, `d4b37cf5`, `9bef8d8f`).
- Docs-Freshness-Gate automated checks green (AutoCronBackfill in both F14 files; zero stale `12 → 13` bump wording; VoluntaryRebookingScheduler in service-tiers + runtime graph; EN/DE twins in sync).
