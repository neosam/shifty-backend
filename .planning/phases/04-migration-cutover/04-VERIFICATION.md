---
phase: 04-migration-cutover
verified: 2026-05-03T16:09:34Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: n/a
---

# Phase 4: Migration & Cutover — Verification Report

**Phase Goal (ROADMAP.md):**
> Bestehende `ExtraHours`-Einträge der Kategorien Vacation/Sick/UnpaidLeave werden heuristisch zu `AbsencePeriod`-Zeiträumen rekonstruiert. Vor dem Feature-Flag-Flip stellt ein Validierungs-Gate **pro Mitarbeiter und pro Kategorie** sicher, dass die summierten Stunden identisch bleiben. Erst dann wird der Flag in einer atomaren Transaktion geflippt — inklusive Carryover-Refresh. Bestehende ExtraHours-REST-Endpunkte bleiben funktional oder sind klar deprecation-markiert. **Diese Phase ist atomar — MIG-01 bis MIG-04 müssen zusammen committet/deployt werden, weil das Feature dormant bleibt bis das Gate grün ist.**

**Verified:** 2026-05-03T16:09:34Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Success Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Read-only Production-Data-Profile in `.planning/migration-backup/` vor jeglicher Migrations-Logik | VERIFIED | `CutoverServiceImpl::profile()` (service_impl/src/cutover.rs:199-376) implements C-Phase4-05 buckets: row_count, sum_amount, fractional_count, weekend_on_workday_only_count, iso_53_indicator. Persists nano-timestamp JSON to `.planning/migration-backup/profile-{ts}.json`. Reachable via `POST /admin/cutover/profile` (rest/src/cutover.rs:131-150). Integration test #15 `test_profile_generates_json_with_histograms` (shifty_bin/src/integration_test/cutover.rs:996) passes — exercises the full HR-auth → handler → service → JSON-file → response-body path. |
| 2 | Heuristik-Migration: 1 AbsencePeriod pro eindeutigem Cluster; Quarantäne sonst; Re-Run idempotent (keyed on `logical_id`) | VERIFIED | `migrate_legacy_extra_hours_to_clusters` (service_impl/src/cutover.rs:393-577) implements RESEARCH.md Operation 1: per-row contract lookup, strict-match heuristic (epsilon 0.001h), year-boundary cluster break, 5 quarantine reasons (AmountBelowContractHours/AboveContractHours/ContractHoursZeroForDay/ContractNotActiveAtDate/Iso53WeekGap). Idempotency via `absence_period_migration_source` mapping table (DAO query filters `id NOT IN (SELECT extra_hours_id FROM absence_period_migration_source)` — dao_impl_sqlite/src/cutover.rs). Service tests: cluster_merges_consecutive_workdays_with_exact_match + 5 quarantine_* + idempotent_rerun_skips_mapped — all green. Integration test test_idempotence_rerun_no_op (cutover.rs:282) passes. |
| 3 | Cutover-Gate (MIG-02) pro `(sales_person_id, kategorie)` über alle relevanten Zeiträume; Toleranz < 0.01h; eine einzige Abweichung lehnt Flag-Flip ab und produziert strukturierten Diff-Report | VERIFIED | `compute_gate` (service_impl/src/cutover.rs:597-748) walks `find_legacy_scope_set` per `(sp_id, year)`, then per category compares `sum_legacy_extra_hours` (DAO sum of `extra_hours.amount`) with `derive_hours_for_range` filtered by category. DRIFT_THRESHOLD = 0.01 (line 604). On drift: `tracing::error!` per drift row (line 672) + DriftRow appended; on any drift → `passed=false`. Diff-report JSON written to `.planning/migration-backup/cutover-gate-{nanos}.json` with all 7 documented schema fields (gate_run_id, run_at, dry_run, drift_threshold, total_drift_rows, drift, passed). Tests: gate_tolerance_pass_below_threshold + gate_tolerance_fail_above_threshold + integration test_diff_report_json_schema, test_gate_uses_derive_hours_for_range_path — all green. |
| 4 | Bei grünem Gate: `absence.range_source_active = true` in derselben Tx wie MIG-01 (Migration) und MIG-04 (Carryover-Rewrite mit Pre-Migration-Backup); jeder Schritt-Failure rollt gesamte Tx zurück | VERIFIED | `run` (service_impl/src/cutover.rs:99-175) opens Tx via `transaction_dao.use_transaction(tx)`, runs migration → gate → branch. On `dry_run || !gate.passed`: `transaction_dao.rollback(tx)` (line 138). On commit branch: `commit_phase` (lines 772-815) runs (a) `backup_carryover_for_scope` → (b) `carryover_rebuild_service.rebuild_for_year` per scope tuple → (c) `extra_hours_service.soft_delete_bulk(migrated_ids, ...)` → (d) `feature_flag_service.set("absence_range_source_active", true, ...)`, all sharing `tx.clone()`. Outer `transaction_dao.commit(tx)` is the single atomic flip-point (line 162). Tests: test_atomic_rollback_on_subservice_error, test_gate_fail_no_state_change, test_pre_cutover_backup_populated_before_update, test_carryover_refresh_scope_only_affected_tuples, test_feature_flag_set_to_true_on_commit, test_soft_delete_migrated_rows_only — all green. |
| 5 | Per-Mitarbeiter-Per-Jahr-Per-Kategorie-Invariant-Test: Pre-Migration-Stunden-Summe == Post-Migration-derived-Stunden-Summe | VERIFIED | Integration test `per_sales_person_per_year_per_category_invariant` (shifty_bin/src/integration_test/cutover.rs:1242) builds 2 sps × 2 years × 3 categories fixture, runs cutover commit, then asserts `(legacy_sum - derive_hours_for_range_sum_filtered_by_category).abs() < 0.001` per tuple. Test green. |
| 6 | Bestehende `/extra-hours`-REST-Endpunkte bleiben funktional oder sind deprecation-markiert; OpenAPI-Snapshot-Test verhindert stilles Breaking Change | VERIFIED | Service-layer flag-gate at `ExtraHoursServiceImpl::create` (service_impl/src/extra_hours.rs:206-225): if `absence_range_source_active` is enabled AND category in {Vacation, SickLeave, UnpaidLeave} → returns `ServiceError::ExtraHoursCategoryDeprecated(category)`. `error_handler` (rest/src/lib.rs:255-277) maps to HTTP 403 with body `{"error":"extra_hours_category_deprecated","category":<lowercase>,"message":"Use POST /absence-period for this category"}`. ExtraWork and other non-deprecated categories unaffected. OpenAPI snapshot at `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap` is committed — 3 cutover endpoints + 6 cutover schemas verified present (grep counts 3 + 6). 3-run determinism check confirmed: 0 `.snap.new` files generated. Tests: create_vacation_returns_403_error_variant_when_flag_on, test_extra_hours_post_flag_gated_before_after, test_403_body_format_for_deprecated_category — all green. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql` | Quarantine table | VERIFIED | `CREATE TABLE absence_migration_quarantine` present; FKs to sales_person + extra_hours; 2 indexes |
| `migrations/sqlite/20260503000001_create-absence-period-migration-source.sql` | Mapping table (idempotency key) | VERIFIED | `CREATE TABLE absence_period_migration_source` present; FKs to absence_period + extra_hours; PK extra_hours_id |
| `migrations/sqlite/20260503000002_create-employee-yearly-carryover-pre-cutover-backup.sql` | Pre-cutover carryover backup | VERIFIED | `CREATE TABLE employee_yearly_carryover_pre_cutover_backup` present; composite PK (cutover_run_id, sales_person_id, year) |
| `migrations/sqlite/20260503000003_add-cutover-admin-privilege.sql` | cutover_admin privilege seed | VERIFIED | `INSERT INTO privilege ... VALUES ('cutover_admin', 'phase-4-migration')` present |
| `service/src/cutover.rs` | CutoverService trait + DTOs + CUTOVER_ADMIN_PRIVILEGE | VERIFIED | 128 LOC: trait `CutoverService { run(); profile() }`, DTOs (CutoverRunResult, GateResult, DriftRow, QuarantineReason, CutoverProfile, CutoverProfileBucket), const CUTOVER_ADMIN_PRIVILEGE |
| `service/src/carryover_rebuild.rs` | CarryoverRebuildService trait | VERIFIED | 45 LOC; trait with `rebuild_for_year`; locked-decision (variant A) documented at top |
| `dao/src/cutover.rs` | CutoverDao trait (8 methods) | VERIFIED | All 8 async methods present (find_legacy_extra_hours_not_yet_migrated, find_all_legacy_extra_hours, upsert_migration_source, upsert_quarantine, find_legacy_scope_set, sum_legacy_extra_hours, count_quarantine_for_drift_row, backup_carryover_for_scope) |
| `dao_impl_sqlite/src/cutover.rs` | SQLx impl of CutoverDao | VERIFIED | 347 LOC; all 8 trait methods implemented with `sqlx::query!` + `QueryBuilder` |
| `service_impl/src/cutover.rs` | CutoverServiceImpl with run() + profile() + compute_gate() + commit_phase() + heuristic | VERIFIED | 917 LOC; gen_service_impl! DI block (10 deps); run() does permission → use_tx → migrate → gate → branch (rollback or commit_phase + commit). Tracing::error! per drift row. |
| `service_impl/src/carryover_rebuild.rs` | CarryoverRebuildServiceImpl | VERIFIED | 145 LOC; consumes ReportingService + CarryoverService + PermissionService + TransactionDao. NO consumption of CutoverService (verified via grep — only mention is in rejected-variant comment). |
| `service_impl/src/extra_hours.rs` (patches) | flag-gated create + soft_delete_bulk + FeatureFlagService dep | VERIFIED | Lines 197-225: flag-gate. Lines 318-345: soft_delete_bulk with permission gate FIRST. New FeatureFlagService DI dep added (line 30). |
| `dao_impl_sqlite/src/extra_hours.rs` (patches) | soft_delete_bulk SQLx impl | VERIFIED | Line 235: impl uses QueryBuilder with `WHERE deleted IS NULL AND id IN (...)` for idempotency |
| `rest/src/cutover.rs` | 3 axum handlers + utoipa annotations + ApiDoc + generate_route | VERIFIED | 174 LOC; 3 handlers (`/gate-dry-run`, `/commit`, `/profile`); CutoverApiDoc with all 6 schemas + 3 request DTOs |
| `rest/src/lib.rs` (patches) | mod cutover, ApiDoc nest, Router nest, error_handler arm | VERIFIED | `pub mod cutover;` (line 9); `(path = "/admin/cutover", api = cutover::CutoverApiDoc)` (line 505); `.nest("/admin/cutover", cutover::generate_route())` (line 588); error_handler arm at lines 255-277 with 403 body shape |
| `shifty_bin/src/main.rs` (patches) | DI re-order + CutoverServiceDependencies + CarryoverRebuildServiceDependencies | VERIFIED | DI ordering check (awk): FeatureFlagServiceImpl line 824 < ExtraHoursServiceImpl line 829 = correct. CutoverServiceDependencies + CarryoverRebuildServiceDependencies marker structs present. 1 new DAO type alias (CutoverDao at line 43). RestStateImpl carries `cutover_service: Arc<CutoverService>` (line 555). |
| `rest/tests/openapi_snapshot.rs` | OpenAPI snapshot test (no longer ignored) | VERIFIED | `#[ignore]` removed; test passes |
| `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap` | Locked OpenAPI surface | VERIFIED | File present in committed snapshots dir; 3-run determinism check passes (0 `.snap.new` files generated); contains 3 cutover endpoints + all 6 expected schemas |
| `shifty_bin/src/integration_test/cutover.rs` | 18 E2E integration tests | VERIFIED | 1338 LOC; 18 `#[tokio::test]` annotations; all 18 tests run green |
| `.planning/migration-backup/.gitkeep` | PII-safe diff-report directory placeholder | VERIFIED | File exists; .gitignore protects `*.json` with `.gitkeep` exception |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `CutoverServiceImpl::run` | `CutoverDao` (8 methods) | gen_service_impl! DI; `tx.clone()` forwarded | WIRED | All 8 DAO calls in compute_gate + commit_phase + migrate_legacy_extra_hours_to_clusters |
| `CutoverServiceImpl::commit_phase` | `ExtraHoursService::soft_delete_bulk` | `Authentication::Full + tx.clone()` | WIRED | Line 795-802 with `migrated_ids` arg verbatim from migrate-helper |
| `CutoverServiceImpl::commit_phase` | `CarryoverRebuildService::rebuild_for_year` | per (sp,year) loop with `Authentication::Full + tx.clone()` | WIRED | Line 786-790, called for every tuple in `gate.scope_set` |
| `CutoverServiceImpl::commit_phase` | `FeatureFlagService::set("absence_range_source_active", true, ...)` | `Authentication::Full + tx.clone()` | WIRED | Line 805-811 — atomic flag-flip step |
| `CutoverServiceImpl::compute_gate` | `AbsenceService::derive_hours_for_range` | `Authentication::Full + tx.clone()`, year-range | WIRED | Line 637-646; reuses Phase-2 single source of truth |
| `CarryoverRebuildServiceImpl::rebuild_for_year` | `ReportingService::get_report_for_employee` | `Authentication::Full + tx.clone()` | WIRED | service_impl/src/carryover_rebuild.rs:89-98 |
| `CarryoverRebuildServiceImpl::rebuild_for_year` | `CarryoverService::set_carryover` | UPSERT-backed write | WIRED | Lines 137-139 |
| `ExtraHoursServiceImpl::create` | `FeatureFlagService::is_enabled` | `Authentication::Full` flag check | WIRED | Lines 212-224 |
| `ExtraHoursServiceImpl::soft_delete_bulk` | `ExtraHoursDao::soft_delete_bulk` | Permission gate BEFORE DAO | WIRED | Service-layer permission first (line 326-328), then DAO call (line 337-339) — verified by `soft_delete_bulk_forbidden_for_unprivileged_user` test (`expect_soft_delete_bulk().times(0)` passes) |
| `POST /admin/cutover/gate-dry-run` | `CutoverService::run(true, ...)` | axum handler → service-layer permission HR | WIRED | rest/src/cutover.rs:65-84 |
| `POST /admin/cutover/commit` | `CutoverService::run(false, ...)` | axum handler → service-layer permission cutover_admin | WIRED | rest/src/cutover.rs:96-115 |
| `POST /admin/cutover/profile` | `CutoverService::profile(...)` | axum handler → service-layer permission HR | WIRED | rest/src/cutover.rs:131-150 |
| `ServiceError::ExtraHoursCategoryDeprecated(cat)` | HTTP 403 response with documented JSON body | rest::error_handler match arm | WIRED | rest/src/lib.rs:255-277 — body fields error/category/message |
| `shifty_bin/main.rs` DI | `FeatureFlagService` constructed BEFORE `ExtraHoursService` | Arc::new() ordering | WIRED | awk line check: FF=824, EH=829 → 824 < 829 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Workspace builds | `cargo build --workspace` | exit 0 (Finished dev profile) | PASS |
| Cutover unit tests | `cargo test -p service_impl test::cutover` | 11 passed; 0 failed; 0 ignored | PASS |
| CarryoverRebuild unit tests | `cargo test -p service_impl test::carryover_rebuild` | 1 passed; 0 failed; 0 ignored | PASS |
| ExtraHours flag-gate + soft_delete tests | `cargo test -p service_impl test::extra_hours` | 5 new tests + all pre-existing pass; 0 failed | PASS |
| Cutover integration tests (18 E2E) | `cargo test -p shifty_bin integration_test::cutover` | 18 passed; 0 failed; 0 ignored | PASS |
| Full workspace tests | `cargo test --workspace` | All 433+ tests green; 0 failed | PASS |
| OpenAPI snapshot determinism (3 runs) | `for i in 1 2 3; do cargo test -p rest --test openapi_snapshot; done; ls *.snap.new \| wc -l` | 3 ok runs; 0 .snap.new files | PASS |
| Bin boot smoke (`timeout 30 cargo run`) | `timeout 30 cargo run \| grep "Running server"` | logs "Running server at 127.0.0.1:3000" within ~1s | PASS |

### Requirements Coverage

| Requirement | Description (from ROADMAP) | Status | Evidence |
|---|---|---|---|
| MIG-01 | Heuristik-Cluster-Algorithmus + Quarantäne + Idempotenz | SATISFIED | `migrate_legacy_extra_hours_to_clusters` impl + 7 unit tests (1 cluster + 5 quarantine + 1 idempotence) + 1 integration test (`test_idempotence_rerun_no_op`) all green |
| MIG-02 | Cutover-Gate per (sp,kat,jahr) mit derive_hours_for_range; Toleranz <0.01h; Diff-Report-JSON | SATISFIED | `compute_gate` impl + 2 unit tests (gate_tolerance_pass/fail) + 2 integration tests (`test_gate_uses_derive_hours_for_range_path`, `test_diff_report_json_schema`) all green |
| MIG-03 | REST-Surface (3 endpoints) + utoipa + permission matrix | SATISFIED | rest/src/cutover.rs with 3 handlers; tests `test_gate_dry_run_endpoint_success`, `test_gate_dry_run_forbidden_for_unprivileged`, `test_gate_dry_run_returns_failure_with_quarantine`, `test_commit_forbidden_for_hr_only`, `test_commit_success_for_cutover_admin` all green |
| MIG-04 | Atomic-Tx Flag-Flip + Carryover-Refresh + Pre-Backup + Soft-Delete; Rollback-on-Failure | SATISFIED | `commit_phase` (4 steps a-d) + `transaction_dao.commit(tx)` single atomic flip; integration tests `test_atomic_rollback_on_subservice_error`, `test_carryover_refresh_scope_only_affected_tuples`, `test_pre_cutover_backup_populated_before_update`, `test_soft_delete_migrated_rows_only`, `test_feature_flag_set_to_true_on_commit`, `test_gate_fail_no_state_change` all green |
| MIG-05 | REST-Deprecation: /extra-hours flag-gate + 403 body + OpenAPI snapshot lock | SATISFIED | Service-layer flag-gate in ExtraHoursServiceImpl::create + 403-body match arm + OpenAPI snapshot file committed; tests `test_extra_hours_post_flag_gated_before_after`, `test_403_body_format_for_deprecated_category`, `openapi_snapshot_locks_full_api_surface` all green |
| SC-1 | Production-Data-Profile via REST | SATISFIED | `CutoverServiceImpl::profile()` + REST endpoint + integration test `test_profile_generates_json_with_histograms` (full HR auth → handler → service → JSON-file → response-body path) green |

All 6 requirement IDs from PLAN frontmatters (MIG-01..05 + SC-1) accounted for. No orphaned requirements detected.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `service_impl/src/extra_hours.rs` | 258 | `unimplemented!()` in `update` method | Info | Pre-existing — `ExtraHoursService::update` was already a stub before Phase 4. Not introduced by this phase, no Phase-4 code calls it. |
| `rest/src/cutover.rs` | 41, 47, 51 | "Empty placeholder" doc-comments on request DTOs | Info | Intentional forward-compatible API surface — empty `CutoverGateDryRunRequest` / `CutoverCommitRequest` / `CutoverProfileRequest` struct bodies preserve the option to add fields without breaking the OpenAPI snapshot. Documented behavior. |
| `service_impl/src/carryover_rebuild.rs` | 11 | Mention of "CutoverService" | Info | Only reference is a code comment in the rejected-variant note. Verified no actual code dependency — cycle break preserved. |

No blockers, no warnings. All flagged items are intentional or pre-existing.

### Human Verification Required

(empty — none required for goal verification)

The two manual-verification items from VALIDATION.md are operational, not goal-blocking:
1. **Production-Data-Profile-Run + Diff-Report-Review (SC-1, MIG-02) im Live-System** — Requires real production data + HR sign-off. Pre-deployment Operations concern; tests cover schema + logic.
2. **Bin-Boot-Smoke** — Already verified PASS during this verification (`timeout 30 cargo run` boots within ~1s, logs "Running server at 127.0.0.1:3000").

Both items per VALIDATION.md are explicitly framed as operations-time verifications that occur DURING/AFTER deployment, not phase-completion gates. The boot-smoke was successfully executed during this verification.

### Gaps Summary

No gaps detected. All 6 ROADMAP Success Criteria + all 6 PLAN-declared requirements (MIG-01..05 + SC-1) are satisfied with reproducible automated tests. The full workspace builds and tests pass (`cargo build --workspace` + `cargo test --workspace` exit 0). The bin boots cleanly with the new DI tree. OpenAPI snapshot is locked and deterministic across 3 runs. The atomic-Tx invariant is enforced by single `transaction_dao.commit(tx)` after gate-pass + tested by `test_atomic_rollback_on_subservice_error` + `test_gate_fail_no_state_change` + `test_pre_cutover_backup_populated_before_update`. The cycle-break locked decision (variant A — separate CarryoverRebuildService BL service) is documented in source and verified by grep.

### Notes — minor housekeeping (informational only, not gaps)

VALIDATION.md still has `nyquist_compliant: false` and unchecked Wave 0 Requirements / sign-off boxes. This is the planner's final tick-off after `/gsd:verify-phase` per the document's own instructions ("`nyquist_compliant: true` setzen, sobald die obigen Boxen alle ✓ sind"). Updating that document is post-verification housekeeping, not a code-correctness gap.

---

*Verified: 2026-05-03T16:09:34Z*
*Verifier: Claude (gsd-verifier)*
