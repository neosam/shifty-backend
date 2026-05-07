---
phase: 04-migration-cutover
plan: 05-cutover-gate-and-diff-report
subsystem: cutover
tags: [cutover, gate, drift-detection, diff-report, atomic-tx, derive_hours_for_range, rust]

# Dependency graph
requires:
  - phase: 04-migration-cutover/04-02
    provides: "CutoverServiceImpl::run skeleton + migrate_legacy_extra_hours_to_clusters → (MigrationStats, Arc<[Uuid]>)"
  - phase: 04-migration-cutover/04-03
    provides: "CarryoverRebuildService::rebuild_for_year"
  - phase: 04-migration-cutover/04-04
    provides: "ExtraHoursService::soft_delete_bulk"
  - phase: 02
    provides: "AbsenceService::derive_hours_for_range (single source of truth)"
provides:
  - "compute_gate(): per (sp, kategorie, jahr) drift detection with < 0.01h tolerance"
  - "commit_phase(): backup-carryover → rebuild-carryover → soft-delete → flip-flag, all in atomic Tx"
  - "Diff-report JSON file at .planning/migration-backup/cutover-gate-{nanos}.json"
  - "CutoverServiceImpl::run is feature-complete; only REST + DI wiring remains for Plan 04-06"
affects: [04-06-cutover-rest-and-openapi, 04-07-integration-tests-and-profile]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Atomic-Tx invariant — outer run() controls commit/rollback; all inner sub-service calls share Some(tx.clone()) and never commit themselves"
    - "Single-source-of-truth gate — derive_hours_for_range is reused verbatim; no re-implementation of conflict-resolution"
    - "Locked-tuple contract consumed verbatim — let (migration_stats, migrated_ids) = self.migrate_legacy_extra_hours_to_clusters(...) — no _with_ids sister method, no surface re-negotiation"
    - "Filesystem-safe diff-report filename — unix_timestamp_nanos() (i128) is colon-free (Linux + Windows safe per Assumption A7) and collision-safe under parallel test execution"
    - "Test scope-mock helper install_empty_gate_scope — preserves Wave-1 migration-test semantics by short-circuiting the new gate phase with an empty scope_set"

key-files:
  created: []
  modified:
    - "service_impl/src/cutover.rs (~250 LoC added: compute_gate, commit_phase, run() branch logic)"
    - "service_impl/src/test/cutover.rs (2 ignored stubs activated; 7 Wave-1 tests updated for new gate-phase mocks; arrange_gate_test helper)"
    - ".gitignore (broaden migration-backup glob to **/.planning/migration-backup/*.json so test artifacts under sub-crate cwds are also excluded)"

key-decisions:
  - "Filename suffix uses unix_timestamp_nanos (i128 nanoseconds) instead of plain unix_timestamp (i64 seconds). Colon-free — same Linux+Windows safety as Assumption A7 in 04-RESEARCH.md, but also resolves a real collision observed in parallel cargo test runs where two tests hit the same wall-clock second."
  - "Wave-1 test compatibility achieved by installing an empty find_legacy_scope_set mock. The migration phase remains the focus of Wave-1 assertions; the gate runs but immediately returns passed=true for an empty scope. No Wave-1 mock had to be removed; only an additive helper was needed."
  - "dry_run rollback path reports migrated_clusters=0. The migration work happened in-Tx but was rolled back, so reporting it as committed would be misleading; total_clusters is preserved so the REST handler can still surface the cluster count to HR for review."
  - "compute_gate is pub(crate) for test reachability but only called via run() in the test suite — the gate-tolerance tests drive run(true, ...) end-to-end so the gate-branch + diff-report file IO are exercised in the same test path that REST will hit (via Plan 04-06)."
  - "count_quarantine_for_drift_row is called only when drift > threshold — saves DAO load on the green path. Matches the plan's per-drift-row semantics and keeps the gate read-set tight under SQLite DEFERRED tx serialization (Threat T-04-05-03)."

patterns-established:
  - "Gate-phase pattern: scope_set → per-(sp, year) sales_person lookup once → per-(sp, year) derive_hours_for_range once → per-category sum + drift evaluation. N×3 derive_hours calls collapsed to N."
  - "Diff-report write at end of compute_gate, regardless of pass/fail. Always returns Some(path) so REST handlers can include it in the response body (D-Phase4-08 dry-run drift visibility)."
  - "Commit-phase order is fixed (a → b → c → d): backup before rebuild before soft-delete before flag-flip. Order matters for atomicity — if any step fails the Drop-rollback returns the world to the pre-Tx state, including the flag."

requirements-completed: [MIG-02, MIG-04]

# Metrics
duration: ~14 min
completed: 2026-05-03
---

# Phase 4 Plan 05: Cutover-Gate + Diff-Report Summary

**Cutover-Gate phase + commit-phase orchestration wired into CutoverServiceImpl::run; per-(sp,kategorie,jahr) drift detection with < 0.01h tolerance; JSON diff-report written to `.planning/migration-backup/cutover-gate-{nanos}.json`; atomic-Tx invariant preserved with verbatim consumption of the Plan-04-02 locked tuple.**

## Performance

- **Duration:** ~14 min
- **Started:** 2026-05-03T11:51Z
- **Completed:** 2026-05-03T12:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `compute_gate(run_id, ran_at, dry_run, tx) -> Result<GateResult, ServiceError>` — per (sp, kategorie, jahr) compares legacy_sum (`CutoverDao::sum_legacy_extra_hours`) against derived_sum (filtered output of `AbsenceService::derive_hours_for_range`). Tolerance < 0.01h absolute (D-Phase4-05). Emits `tracing::error!` per drift row (D-Phase4-06) and writes the diff-report JSON.
- `commit_phase(run_id, ran_at, &gate, migrated_ids, tx) -> Result<(), ServiceError>` — backup carryover (D-Phase4-13) → rebuild carryover per (sp, year) (D-Phase4-12) → soft-delete migrated extra_hours via the verbatim Plan-04-02 `migrated_ids` (D-Phase4-10) → flip `absence_range_source_active` (D-Phase4-09).
- `run()` branch logic refactored: `dry_run || !gate.passed` → rollback; `!dry_run && gate.passed` → commit_phase + `transaction_dao.commit(tx)`. Single-Tx atomicity (D-Phase4-14) preserved.
- 2 gate-tolerance boundary tests (drift 0.005h pass, drift 0.020h fail) drive the full `run()` path including diff-report file IO. Total cutover-test surface: 11 passing tests (9 Wave-1 + 2 Wave-2).

## Task Commits

Each task was committed atomically via jj:

1. **Task 1: Extend CutoverServiceImpl with compute_gate, commit_phase, and updated run() branch logic** — `a08eee6a` (feat)
2. **Task 2: Activate gate-tolerance pass/fail boundary tests + collision-safe filename** — `1d4ad842` (test)

## Files Created/Modified

- `service_impl/src/cutover.rs` — Added `compute_gate` (~110 LoC) and `commit_phase` (~50 LoC); rewrote `run()` body to thread the locked Plan-04-02 tuple into both phases and to branch on gate outcome. Module-level docstring updated to describe Wave 1 + Wave 2.
- `service_impl/src/test/cutover.rs` — Replaced 2 `#[ignore]+unimplemented!()` stubs with full implementations; added `install_empty_gate_scope` helper + `cutover_test_sales_person` fixture + `arrange_gate_test` helper; added `expect_commit()` to `build_default_transaction_dao` so commit-path tests are buildable.
- `.gitignore` — Generalized `.planning/migration-backup/*.json` to `**/.planning/migration-backup/*.json` so cargo-test cwd-relative artifacts under `service_impl/.planning/` are also excluded.

## Locked-Contract Verbatim Destructure

The Plan-04-02 method-signature contract is consumed verbatim in `run()`:

```rust
let (migration_stats, migrated_ids) = self
    .migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone())
    .await?;
```

`migrated_ids` (an `Arc<[Uuid]>`) is then passed straight into `commit_phase`, which forwards it to `ExtraHoursService::soft_delete_bulk`. No `_with_ids`-suffix sister method exists; `grep -c migrate_legacy_extra_hours_to_clusters_with_ids service_impl/src/cutover.rs` returns 0.

## Diff-Report File Path Format

Filename: `.planning/migration-backup/cutover-gate-{unix_timestamp_nanos}.json`

The plan permitted either Unix-timestamp or ISO-without-colons (Assumption A7 in 04-RESEARCH.md — Linux+Windows filesystem safety). The implementation uses **`unix_timestamp_nanos()` (i128 nanoseconds)** rather than plain `unix_timestamp()` (i64 seconds). Both are colon-free, so A7 is satisfied. The nanosecond precision additionally resolves a real collision observed in parallel `cargo test` runs where two tests hit the same wall-clock second; production cutover runs once and would never collide, but the collision-safety hardening is essentially free.

The JSON body matches CONTEXT.md `<specifics>` verbatim: `gate_run_id`, `run_at` (ISO-8601 UTC), `dry_run`, `drift_threshold` (0.01), `total_drift_rows`, `drift[]` (with `sales_person_id`, `sales_person_name`, `category`, `year`, `legacy_sum`, `derived_sum`, `drift`, `quarantined_extra_hours_count`, `quarantine_reasons`), `passed`.

## Decisions Made

- **Filename uses `unix_timestamp_nanos` (i128) instead of `unix_timestamp` (i64).** Driver: parallel test collisions. Side effect: production filename is `cutover-gate-1777809873071946783.json` rather than `cutover-gate-1777809873.json` — slightly less human-readable but unambiguous. The `run_at` ISO-8601 field inside the JSON body remains human-readable.
- **dry_run path reports `migrated_clusters: 0`.** The migration phase ran in-Tx but the rollback discarded the work; reporting it as "migrated" would be misleading. `total_clusters` preserves the heuristic count so HR can still see how many clusters the migration would have created.
- **Wave-1 test compatibility via additive `install_empty_gate_scope` helper.** No Wave-1 test had to be removed or migration-phase mocks rewritten. The gate runs but immediately returns `passed=true` because the scope set is empty — minimal disruption to the existing test surface.
- **`compute_gate` and `commit_phase` are `pub(crate)`** for test reachability, but the boundary tests drive `run()` end-to-end rather than calling the helpers directly. This exercises the gate-branch + diff-report file IO via the same path REST handlers will hit in Plan 04-06.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wave-1 tests broke after gate-phase added to run()**
- **Found during:** Task 1 (first cargo test run after compute_gate wired into run())
- **Issue:** The 7 existing Wave-1 tests (cluster-merge happy-path + 5 quarantine + idempotent-rerun) call `run(true, ...)` with no `find_legacy_scope_set` mock. After Task 1, `run()` always invokes `compute_gate`, which calls that DAO method — every Wave-1 test panicked with "No matching expectation found".
- **Fix:** Added an additive helper `install_empty_gate_scope(&mut deps)` that installs `expect_find_legacy_scope_set().returning(|_| Ok(Arc::from([])))`. Empty scope short-circuits the gate's per-(sp, year) loop without any other DAO calls. Applied to all 7 Wave-1 tests.
- **Files modified:** `service_impl/src/test/cutover.rs`
- **Verification:** All 9 Wave-1 tests still green plus all 2 Wave-2 tests new green (11 total).
- **Committed in:** `a08eee6a` (Task 1 commit)

**2. [Rule 1 - Bug] Cluster-merge happy-path assertion outdated**
- **Found during:** Task 1
- **Issue:** Wave-1 test asserted `result.migrated_clusters == 1` and `!result.gate_passed`. After Task 1's branch refactor: dry_run path reports `migrated_clusters=0` (rolled-back) and `gate_passed=true` (empty gate scope passes trivially).
- **Fix:** Updated assertions to match new contract: `migrated_clusters == 0`, `gate_passed == true`, plus a new assertion that `diff_report_path.is_some()`. Comment added explaining the rolled-back semantics.
- **Files modified:** `service_impl/src/test/cutover.rs`
- **Verification:** `cargo test cluster_merges_consecutive_workdays_with_exact_match` green.
- **Committed in:** `a08eee6a` (Task 1 commit)

**3. [Rule 3 - Blocking] MockTransactionDao missing `expect_commit()`**
- **Found during:** Task 1
- **Issue:** Wave-1 `build_default_transaction_dao()` only set up `expect_use_transaction` and `expect_rollback`. Adding gate + commit-phase to `run()` means tests that take the commit path would panic on `expect_commit().returning(...)`. Even though the boundary tests use dry_run=true, the helper had to support both paths for future Wave-2 commit-phase tests in Plan 04-07.
- **Fix:** Added `transaction_dao.expect_commit().returning(|_| Ok(()))` to the helper.
- **Files modified:** `service_impl/src/test/cutover.rs`
- **Verification:** All cutover tests green; no impact on rollback-path tests (mockall accepts unused expectations).
- **Committed in:** `a08eee6a` (Task 1 commit)

**4. [Rule 1 - Bug] Test artifacts leaked into jj working copy**
- **Found during:** After running cargo test to verify Task 2
- **Issue:** Tests write diff-report JSON files to `.planning/migration-backup/` relative to the test's cwd — which is `service_impl/` for `cargo test -p service_impl`. The existing `.gitignore` rule `.planning/migration-backup/*.json` only matched the workspace-root path, so jj began tracking ~30 nanosecond-keyed JSON files under `service_impl/.planning/migration-backup/`. The first round of test artifacts even leaked into Task 1's commit before being noticed.
- **Fix:**
  1. Generalized the `.gitignore` glob to `**/.planning/migration-backup/*.json` so any-depth match is excluded.
  2. Used `jj squash --from @ --into rumwruws` to remove the leaked file (`cutover-gate-1777809532.json`) from Task 1's commit.
  3. Cleaned the on-disk artifacts (`rm -rf service_impl/.planning`).
- **Files modified:** `.gitignore`
- **Verification:** Re-running cargo test produces ~25 new files under `service_impl/.planning/migration-backup/` but `jj status` shows none of them — gitignore takes effect.
- **Committed in:** Gitignore change in `1d4ad842` (Task 2 commit); leaked-file removal squashed back into `a08eee6a`.

**5. [Rule 1 - Bug] Diff-report filename collision under parallel test execution**
- **Found during:** Task 2 (first parallel-test run with both boundary tests active)
- **Issue:** Both boundary tests build a fresh `ran_at = now()` via `run()`. When tokio's parallel runner fires both within the same wall-clock second, they computed identical `unix_timestamp()` (i64 seconds) → same filename → second test's file overwrote the first's, then the read-and-assert in test 2 saw test 1's content (`"passed": true` instead of `"passed": false`).
- **Fix:** Switched the filename suffix in `compute_gate` from `ran_at.assume_utc().unix_timestamp()` to `ran_at.assume_utc().unix_timestamp_nanos()`. Still satisfies Assumption A7 (no colons, Linux + Windows filesystem safe). Production cutover runs once and would never collide either way; the change is essentially free hardening.
- **Files modified:** `service_impl/src/cutover.rs`
- **Verification:** `cargo test test::cutover` runs both tests in parallel and both pass.
- **Committed in:** `1d4ad842` (Task 2 commit, paired with the test activation since the tests are what surfaced the issue).

---

**Total deviations:** 5 auto-fixed (3× Rule 1 — bugs in test plumbing / production filename collision; 1× Rule 3 — missing mock expectation; 1× Rule 1 — repo-pollution from test outputs)
**Impact on plan:** All deviations were corrections to test plumbing or hardening of the production filename. No scope creep — the spec for `compute_gate` / `commit_phase` / branch logic is implemented exactly as the plan dictates.

## Issues Encountered

- See deviation #5 above. Beyond that: the plan's reference to `Arc<[DerivedDayHours]>` for `derive_hours_for_range` was outdated — the actual return type is `BTreeMap<Date, ResolvedAbsence>` (per `service/src/absence.rs:215`). Implementation uses the actual signature; the gate's per-category sum filter walks `.values()` and matches `r.category == svc_cat`. No semantic change vs. the plan's intent.

## Hand-off Note for Plan 04-06

> **CutoverServiceImpl exposes only the public `CutoverService::run` + `::profile` API; REST handlers in Plan 04-06 call only those methods.**
>
> **CutoverServiceDependencies block: 10 deps total** — `CutoverDao`, `AbsenceDao`, `AbsenceService`, `ExtraHoursService`, `CarryoverRebuildService`, `FeatureFlagService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`, `TransactionDao`. (Already locked since Plan 04-02; Plan 04-05 added zero deps.)

The trait surface that REST will consume:

```rust
async fn run(
    &self,
    dry_run: bool,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<CutoverRunResult, ServiceError>;

async fn profile(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<CutoverProfile, ServiceError>;
```

`CutoverRunResult.diff_report_path: Option<Arc<str>>` is set on every gate-evaluated run (always present after Plan 04-05), so REST can surface it in the response body for HR review without nullability gymnastics on the success path.

## Next Phase Readiness

- **Plan 04-06 (REST + DI):** Ready. Service layer is feature-complete. `CutoverServiceDeps` block is final at 10 deps — `shifty_bin/src/main.rs` wiring is purely DI.
- **Plan 04-07 (integration tests + profile):** Ready. The `profile()` method is still a `Err(ServiceError::InternalError)` placeholder per its plan-deferral, and integration tests can now drive the full `run()` path against a real SQLite Tx including the gate + commit-phase.

## Self-Check: PASSED

- [x] `service_impl/src/cutover.rs` modified — `grep -q 'fn compute_gate' service_impl/src/cutover.rs && grep -q 'fn commit_phase' service_impl/src/cutover.rs` returns 0
- [x] `service_impl/src/test/cutover.rs` modified — both `gate_tolerance_pass_below_threshold` and `gate_tolerance_fail_above_threshold` are now full implementations; `grep -c '#\[ignore' service_impl/src/test/cutover.rs` returns 0
- [x] Plan-04-02 locked tuple consumed verbatim — `grep -E 'let \(migration_stats, migrated_ids\) = self' service_impl/src/cutover.rs` returns 1 match
- [x] No `_with_ids` sister method — `grep -c migrate_legacy_extra_hours_to_clusters_with_ids service_impl/src/cutover.rs` returns 0
- [x] `cargo build --workspace` exits 0
- [x] `cargo test -p service_impl --lib test::cutover` reports 11 passed, 0 failed, 0 ignored
- [x] Task 1 commit `a08eee6a` exists in jj log; Task 2 commit `1d4ad842` exists in jj log
- [x] `STATE.md` and `ROADMAP.md` not modified (orchestrator owns those writes)

---
*Phase: 04-migration-cutover*
*Plan: 05-cutover-gate-and-diff-report*
*Completed: 2026-05-03*
