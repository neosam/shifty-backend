---
plan: 04-05-cutover-gate-and-diff-report
phase: 4
wave: 2
depends_on: [04-02-cutover-service-heuristic, 04-03-carryover-rebuild-service, 04-04-extra-hours-flag-gate-and-soft-delete]
requirements: [MIG-02, MIG-04]
files_modified:
  - service_impl/src/cutover.rs
  - service_impl/src/test/cutover.rs
autonomous: true
must_haves:
  truths:
    - "`CutoverServiceImpl::run` extends to the gate phase: per (sp, kategorie, jahr) compute `legacy_sum` (CutoverDao::sum_legacy_extra_hours) vs `derived_sum` (AbsenceService::derive_hours_for_range filtered by category)."
    - "`run()` consumes the locked Plan-04-02 contract `let (migration_stats, migrated_ids) = self.migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone()).await?;` — no method-rename, no `_with_ids`-suffix sister method."
    - "Gate tolerance < 0.01h absolute (D-Phase4-05). 2 unit tests prove the boundary (0.005h pass, 0.02h fail)."
    - "Diff-report JSON-file persisted at `.planning/migration-backup/cutover-gate-{ISO_TIMESTAMP_FILESAFE}.json` with the schema from CONTEXT.md `<specifics>` (verbatim)."
    - "Filename uses Unix-timestamp OR ISO without colons (Assumption A7 in RESEARCH.md — Linux+Windows safe)."
    - "`tracing::error!` per drift row (D-Phase4-06)."
    - "If `dry_run` OR `!gate_passed`: rollback Tx + return CutoverRunResult with diff_report_path Some + gate_passed flag set correctly."
    - "If `!dry_run` AND `gate_passed`: run COMMIT-PHASE — backup carryover, rebuild carryover per scope tuple, soft-delete migrated extra_hours (using the `migrated_ids` from migrate_legacy_extra_hours_to_clusters), set feature flag true, COMMIT Tx."
    - "Atomic-Tx invariant preserved (Pattern 1 RESEARCH.md): all sub-service calls receive `Some(tx.clone())`, none commit."
  artifacts:
    - path: "service_impl/src/cutover.rs"
      provides: "Patched: + compute_gate(), + commit_phase(), + run() branch logic that threads `migrated_ids` from Plan-04-02 contract verbatim, + diff-report JSON file IO"
    - path: "service_impl/src/test/cutover.rs"
      provides: "Patched: 2 gate-tolerance tests activated (no longer #[ignore])"
  key_links:
    - from: "compute_gate"
      to: "AbsenceService::derive_hours_for_range"
      via: "Reuse Phase-2 single source of truth (no re-implementation)"
    - from: "commit_phase"
      to: "CutoverDao::backup_carryover_for_scope + CarryoverRebuildService::rebuild_for_year + ExtraHoursService::soft_delete_bulk + FeatureFlagService::set"
      via: "All called inside the same Tx with Authentication::Full; soft_delete_bulk receives the `Arc<[Uuid]>` from Plan-04-02 unchanged"
    - from: "Diff-report file path"
      to: "CutoverRunResult.diff_report_path"
      via: "Returned to REST handler (Plan 04-06) for inclusion in response body"
---

<objective>
Wave 2 (first half) — Extend `CutoverServiceImpl::run` to add the gate phase + the commit phase. After this plan, `run` is feature-complete; Plan 04-06 wires REST + DI; Plan 04-07 adds E2E tests.

This plan owns ALL of `service_impl/src/cutover.rs` extensions for Wave 2 — no other plan in Wave 2 touches it. That guarantees atomic-Tx wiring is in one writeable scope.

Purpose: Land the validation-gate logic (the **single line of defense** between heuristic-output and feature-flag-flip per Threat T-04-00 row 4) and the commit-phase orchestration. After this plan, the entire MIG-02 + MIG-04 atomic-Tx surface is implemented and unit-testable.

Output: 1 file extended (~250-300 added LoC), 2 tests un-`#[ignore]`d, 1 helper file in `.planning/migration-backup/` produced by tests at runtime.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/04-migration-cutover/04-CONTEXT.md
@.planning/phases/04-migration-cutover/04-RESEARCH.md
@.planning/phases/04-migration-cutover/04-VALIDATION.md
@.planning/phases/04-migration-cutover/04-02-SUMMARY.md
@.planning/phases/04-migration-cutover/04-03-SUMMARY.md
@.planning/phases/04-migration-cutover/04-04-SUMMARY.md

@service_impl/src/cutover.rs
@service/src/cutover.rs
@service/src/absence.rs
@dao/src/cutover.rs

<interfaces>
<!-- Verbatim contracts. -->

From `service/src/cutover.rs` (Plan 04-01 DTOs — DO NOT change shape):
```rust
pub struct DriftRow {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub category: AbsenceCategory,
    pub year: u32,
    pub legacy_sum: f32,
    pub derived_sum: f32,
    pub drift: f32,
    pub quarantined_extra_hours_count: u32,
    pub quarantine_reasons: Arc<[Arc<str>]>,
}

pub struct GateResult {
    pub passed: bool,
    pub drift_rows: Arc<[DriftRow]>,
    pub diff_report_path: Arc<str>,
    pub scope_set: Arc<[(Uuid, u32)]>,
}
```

From `service_impl/src/cutover.rs` (Plan 04-02 LOCKED contract — consume verbatim):
```rust
// Returns BOTH the stats AND the migrated extra_hours.id list. Locked at Plan 04-02 Task 2.
pub(crate) async fn migrate_legacy_extra_hours_to_clusters(
    &self,
    cutover_run_id: Uuid,
    migrated_at: time::PrimitiveDateTime,
    tx: <<Deps as CutoverServiceDeps>::TransactionDao as dao::TransactionDao>::Transaction,
) -> Result<(MigrationStats, Arc<[Uuid]>), ServiceError>;
```

From `service/src/absence.rs:208` (single source of truth — call verbatim):
```rust
async fn derive_hours_for_range(
    &self,
    from: Date,
    to: Date,
    sales_person_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[DerivedDayHours]>, ServiceError>;
```

From `dao/src/cutover.rs` (Plan 04-01 DAO):
```rust
async fn find_legacy_scope_set(...) -> Result<Arc<[(Uuid, u32)]>, DaoError>;  // distinct (sp, year)
async fn sum_legacy_extra_hours(sp_id, &category, year, tx) -> Result<f32, DaoError>;
async fn count_quarantine_for_drift_row(sp_id, &category, year, run_id, tx) -> Result<(u32, Arc<[Arc<str>]>), DaoError>;
async fn backup_carryover_for_scope(run_id, backed_up_at, scope, tx) -> Result<(), DaoError>;
```

From `service/src/feature_flag.rs:46-52`:
```rust
async fn set(&self, key: &str, value: bool, ctx, tx) -> Result<(), ServiceError>;
```

From `service/src/sales_person.rs` (verify the read method): `get_sales_person(sp_id, ctx, tx) -> Result<SalesPerson, ServiceError>` or similar — used to look up `sales_person_name` for DriftRow.

From CONTEXT.md `<specifics>` (verbatim diff-report JSON schema):
```json
{
  "gate_run_id": "uuid",
  "run_at": "2026-05-03T14:23:00Z",
  "dry_run": true,
  "drift_threshold": 0.01,
  "total_drift_rows": 3,
  "drift": [
    { "sales_person_id": "uuid", "sales_person_name": "...", "category": "Vacation",
      "year": 2024, "legacy_sum": 120.0, "derived_sum": 112.0, "drift": 8.0,
      "quarantined_extra_hours_count": 2,
      "quarantine_reasons": ["amount_below_contract_hours", "weekend_entry_with_workday_only_contract"] }
  ],
  "passed": false
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Extend CutoverServiceImpl with compute_gate(), commit_phase(), and updated run() branch logic</name>
  <read_first>
    - service_impl/src/cutover.rs (Plan 04-02 output — keep the migration phase intact, append new methods)
    - service/src/sales_person.rs (find the method to look up sales_person by id — for DriftRow.sales_person_name)
    - service/src/absence.rs (Z. 113-130 DerivedDayHours; map AbsenceCategory ↔ ExtraHoursCategoryEntity)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Operation 2: Gate-Berechnung mit derive_hours_for_range-Reuse" (verbatim)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "System Architecture Diagram" (commit-phase steps a, b, c, d)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "D-Phase4-05 / D-Phase4-06 / D-Phase4-12 / D-Phase4-13 / D-Phase4-14"
  </read_first>
  <action>
**Patch `service_impl/src/cutover.rs`:**

Add these new private methods to `impl<Deps: CutoverServiceDeps> CutoverServiceImpl<Deps> { ... }`:

```rust
async fn compute_gate(
    &self,
    cutover_run_id: Uuid,
    ran_at: time::PrimitiveDateTime,
    dry_run: bool,
    tx: <<Deps as CutoverServiceDeps>::TransactionDao as dao::TransactionDao>::Transaction,
) -> Result<GateResult, ServiceError> {
    let scope = self.cutover_dao.find_legacy_scope_set(tx.clone()).await?;
    let mut drift_rows: Vec<DriftRow> = Vec::new();

    for &(sp_id, year) in scope.iter() {
        // Pre-fetch sales_person_name once per sp (avoid N×3 lookups across categories).
        let sp = self.sales_person_service
            .get(sp_id, Authentication::Full, Some(tx.clone()))   // verify exact method
            .await?;
        let sp_name: Arc<str> = sp.name.clone();   // verify field name

        let year_start = time::Date::from_calendar_date(year as i32, time::Month::January, 1)?;
        let year_end = time::Date::from_calendar_date(year as i32, time::Month::December, 31)?;

        // Single derive_hours_for_range call per (sp, year), then partition by category.
        let derived = self.absence_service
            .derive_hours_for_range(year_start, year_end, sp_id, Authentication::Full, Some(tx.clone()))
            .await?;

        for category_dao in &[
            dao::extra_hours::ExtraHoursCategoryEntity::Vacation,
            dao::extra_hours::ExtraHoursCategoryEntity::SickLeave,
            dao::extra_hours::ExtraHoursCategoryEntity::UnpaidLeave,
        ] {
            let legacy_sum = self.cutover_dao
                .sum_legacy_extra_hours(sp_id, category_dao, year, tx.clone())
                .await?;

            // Map dao category -> service AbsenceCategory for the filter:
            let svc_cat = match category_dao {
                dao::extra_hours::ExtraHoursCategoryEntity::Vacation => service::absence::AbsenceCategory::Vacation,
                dao::extra_hours::ExtraHoursCategoryEntity::SickLeave => service::absence::AbsenceCategory::SickLeave,
                dao::extra_hours::ExtraHoursCategoryEntity::UnpaidLeave => service::absence::AbsenceCategory::UnpaidLeave,
                _ => unreachable!(),
            };

            let derived_sum: f32 = derived.iter()
                .filter(|r| r.category == svc_cat)
                .map(|r| r.hours)
                .sum();

            let drift = (legacy_sum - derived_sum).abs();
            if drift > 0.01 {
                let (quarantined_count, reasons) = self.cutover_dao
                    .count_quarantine_for_drift_row(sp_id, category_dao, year, cutover_run_id, tx.clone())
                    .await?;
                tracing::error!(
                    "[cutover-gate] drift sp={} cat={:?} year={}: legacy={} derived={} drift={}",
                    sp_id, svc_cat, year, legacy_sum, derived_sum, drift
                );
                drift_rows.push(DriftRow {
                    sales_person_id: sp_id,
                    sales_person_name: sp_name.clone(),
                    category: svc_cat,
                    year,
                    legacy_sum,
                    derived_sum,
                    drift,
                    quarantined_extra_hours_count: quarantined_count,
                    quarantine_reasons: reasons,
                });
            }
        }
    }

    // Diff-report file (Assumption A7 mitigation — use Unix timestamp for filesystem-safe filename):
    std::fs::create_dir_all(".planning/migration-backup")
        .map_err(|_| ServiceError::InternalError)?;
    let ts = ran_at.assume_utc().unix_timestamp();
    let report_path = format!(".planning/migration-backup/cutover-gate-{}.json", ts);

    let report_json = serde_json::json!({
        "gate_run_id": cutover_run_id.to_string(),
        "run_at": ran_at.assume_utc().format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_default(),
        "dry_run": dry_run,
        "drift_threshold": 0.01_f32,
        "total_drift_rows": drift_rows.len(),
        "drift": drift_rows.iter().map(|r| serde_json::json!({
            "sales_person_id": r.sales_person_id.to_string(),
            "sales_person_name": r.sales_person_name,
            "category": format!("{:?}", r.category),
            "year": r.year,
            "legacy_sum": r.legacy_sum,
            "derived_sum": r.derived_sum,
            "drift": r.drift,
            "quarantined_extra_hours_count": r.quarantined_extra_hours_count,
            "quarantine_reasons": &*r.quarantine_reasons,
        })).collect::<Vec<_>>(),
        "passed": drift_rows.is_empty(),
    });

    std::fs::write(&report_path, serde_json::to_string_pretty(&report_json).unwrap())
        .map_err(|_| ServiceError::InternalError)?;

    Ok(GateResult {
        passed: drift_rows.is_empty(),
        drift_rows: Arc::from(drift_rows.as_slice()),
        diff_report_path: report_path.into(),
        scope_set: scope,
    })
}

async fn commit_phase(
    &self,
    cutover_run_id: Uuid,
    ran_at: time::PrimitiveDateTime,
    gate: &GateResult,
    migrated_ids: Arc<[Uuid]>,
    tx: <<Deps as CutoverServiceDeps>::TransactionDao as dao::TransactionDao>::Transaction,
) -> Result<(), ServiceError> {
    // Step a (D-Phase4-13): backup carryover for the gate scope set BEFORE update.
    self.cutover_dao
        .backup_carryover_for_scope(cutover_run_id, ran_at, &gate.scope_set, tx.clone())
        .await?;

    // Step b (D-Phase4-12): rebuild carryover for each (sp, year) in scope.
    for &(sp_id, year) in gate.scope_set.iter() {
        self.carryover_rebuild_service
            .rebuild_for_year(sp_id, year, Authentication::Full, Some(tx.clone()))
            .await?;
    }

    // Step c (D-Phase4-10): soft-delete the legacy extra_hours rows that were migrated.
    // `migrated_ids` is the `Arc<[Uuid]>` returned from migrate_legacy_extra_hours_to_clusters
    // in the same `run()` invocation — locked contract from Plan 04-02 Task 2.
    self.extra_hours_service
        .soft_delete_bulk(migrated_ids, "phase-4-cutover-migration", Authentication::Full, Some(tx.clone()))
        .await?;

    // Step d (D-Phase4-09 atomic transition): flip the feature flag.
    self.feature_flag_service
        .set("absence_range_source_active", true, Authentication::Full, Some(tx.clone()))
        .await?;

    Ok(())
}
```

**Update `run()` branch logic** to replace the Wave-1 always-rollback with:

```rust
async fn run(&self, dry_run, ctx, tx) -> Result<CutoverRunResult, ServiceError> {
    self.permission_service.check_permission(if dry_run { HR_PRIVILEGE } else { CUTOVER_ADMIN_PRIVILEGE }, ctx.clone()).await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;
    let run_id = uuid::Uuid::new_v4();
    let ran_at = /* now_utc as PrimitiveDateTime */;

    // 1. Migration phase (Plan 04-02 LOCKED contract — consume verbatim).
    // The helper returns BOTH the stats AND the merged extra_hours.id list;
    // we thread the id list straight into commit_phase below. There is NO
    // separate `_with_ids` sister method — Plan 04-02 already locked this signature.
    let (migration_stats, migrated_ids) = self
        .migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone())
        .await?;

    // 2. Gate phase
    let gate = self.compute_gate(run_id, ran_at, dry_run, tx.clone()).await?;

    // 3. Branch
    if dry_run || !gate.passed {
        self.transaction_dao.rollback(tx).await?;
        return Ok(CutoverRunResult {
            run_id, ran_at, dry_run,
            gate_passed: gate.passed,
            total_clusters: migration_stats.clusters as u32,
            migrated_clusters: 0,                   // not committed
            quarantined_rows: migration_stats.quarantined as u32,
            gate_drift_rows: gate.drift_rows.len() as u32,
            diff_report_path: Some(gate.diff_report_path.clone()),
        });
    }

    // 4. Commit phase — pass the Plan-04-02 `migrated_ids` straight in.
    self.commit_phase(run_id, ran_at, &gate, migrated_ids, tx.clone()).await?;

    self.transaction_dao.commit(tx).await?;

    Ok(CutoverRunResult {
        run_id, ran_at, dry_run,
        gate_passed: true,
        total_clusters: migration_stats.clusters as u32,
        migrated_clusters: migration_stats.clusters as u32,
        quarantined_rows: migration_stats.quarantined as u32,
        gate_drift_rows: 0,
        diff_report_path: Some(gate.diff_report_path),
    })
}
```

**Add `SalesPersonService` to the `gen_service_impl!` deps block** if not already added by Plan 04-02 (verify; it should be — see Plan 04-02 Task 2 DI block).
  </action>
  <acceptance_criteria>
    - `grep -q 'fn compute_gate' service_impl/src/cutover.rs` exits 0
    - `grep -q 'fn commit_phase' service_impl/src/cutover.rs` exits 0
    - **Locked-contract consumer check:** `grep -q 'let (migration_stats, migrated_ids) = self$' service_impl/src/cutover.rs || grep -E -q 'let \(migration_stats, migrated_ids\) = self\.migrate_legacy_extra_hours_to_clusters' service_impl/src/cutover.rs` exits 0 (run() destructures the tuple from the locked Plan-04-02 method — NO `_with_ids`-suffix variant)
    - **No phantom sister-method:** `grep -q 'migrate_legacy_extra_hours_to_clusters_with_ids' service_impl/src/cutover.rs` exits 1 (must NOT exist — Plan 04-02 locked the single canonical method name)
    - `grep -q 'feature_flag_service.set' service_impl/src/cutover.rs` exits 0
    - `grep -q 'soft_delete_bulk' service_impl/src/cutover.rs` exits 0
    - `grep -q 'backup_carryover_for_scope' service_impl/src/cutover.rs` exits 0
    - `grep -q 'rebuild_for_year' service_impl/src/cutover.rs` exits 0
    - `grep -q 'transaction_dao.commit' service_impl/src/cutover.rs` exits 0 (was 0 in Wave 1; now 1)
    - `grep -q 'transaction_dao.rollback' service_impl/src/cutover.rs` exits 0 (still present for dry-run/gate-fail branch)
    - `grep -q 'tracing::error' service_impl/src/cutover.rs` exits 0
    - `cargo build -p service_impl` exits 0
    - All previously-passing Plan-04-02 tests still pass: `cargo test -p service_impl test::cutover` exits 0 with all 8 prior tests still green
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::cutover</automated>
  </verify>
  <done>
    `CutoverServiceImpl::run` is feature-complete; gate + commit-phase wired into the atomic Tx; tracing emits per drift row; the Plan-04-02 `(MigrationStats, Arc<[Uuid]>)` contract is consumed verbatim with no surface re-negotiation.
  </done>
</task>

<task type="auto">
  <name>Task 2: Activate gate_tolerance_pass_below_threshold + gate_tolerance_fail_above_threshold tests</name>
  <read_first>
    - service_impl/src/test/cutover.rs (Wave-0 + Wave-1 stubs from Plan 04-01 + Plan 04-02)
    - service_impl/src/cutover.rs (Task 1 — find compute_gate signature)
    - service/src/absence.rs (DerivedDayHours)
  </read_first>
  <action>
Replace the two `#[ignore]+unimplemented!()` stubs with full implementations:

```rust
#[tokio::test]
async fn gate_tolerance_pass_below_threshold() {
    // Arrange:
    //   - 1 sp, year 2024, category Vacation
    //   - MockCutoverDao::find_legacy_scope_set returns [(sp_id, 2024)]
    //   - MockCutoverDao::sum_legacy_extra_hours returns 100.000 for Vacation, 0.0 for SickLeave/UnpaidLeave
    //   - MockAbsenceService::derive_hours_for_range returns DerivedDayHours summing to 100.005 (drift = 0.005, < 0.01)
    //   - MockSalesPersonService::get returns a SalesPerson with name "Test"
    //   - MockCutoverDao::count_quarantine_for_drift_row should NOT be called (drift below threshold)
    //   - MockCutoverDao should NOT be called for any commit-phase methods (we only need gate, not full run)
    //
    // Act: call svc.compute_gate(run_id, ran_at, true, tx).await
    //
    // Assert:
    //   - GateResult.passed == true
    //   - GateResult.drift_rows.is_empty()
    //   - File exists at GateResult.diff_report_path
    //   - File JSON has "passed": true
    //
    // Cleanup: std::fs::remove_file(report_path) at end of test
    let svc = build_test_service();   // helper following plan 04-02 test pattern
    // ... mock setups ...
    let run_id = uuid::Uuid::new_v4();
    let ran_at = /* fixed PrimitiveDateTime */;
    let tx = MockTransaction::default();
    let gate = svc.compute_gate(run_id, ran_at, true, tx).await.unwrap();
    assert!(gate.passed);
    assert!(gate.drift_rows.is_empty());
    let p = std::path::Path::new(&*gate.diff_report_path);
    assert!(p.exists());
    let body = std::fs::read_to_string(p).unwrap();
    assert!(body.contains("\"passed\": true") || body.contains("\"passed\":true"));
    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn gate_tolerance_fail_above_threshold() {
    // Same setup as above, but legacy_sum = 100.0, derived_sum = 100.02 (drift = 0.02 > 0.01)
    // Assert: gate.passed == false, drift_rows.len() == 1, file contains "passed": false
    // ... ...
}
```

**Note:** if `compute_gate` is private (recommended for encapsulation), make it `pub(crate)` for testing. Alternatively, test via the full `run()` method with all sub-services mocked — slightly more boilerplate but tests the full path. Prefer the latter (test via `run()`) so the gate-branch logic is also exercised; the assertion for the file-creation side-effect remains the same.
  </action>
  <acceptance_criteria>
    - `grep -c '#\[ignore' service_impl/src/test/cutover.rs` returns 0 (or only counts truly out-of-scope future tests)
    - `cargo test -p service_impl test::cutover::gate_tolerance_pass_below_threshold` exits 0
    - `cargo test -p service_impl test::cutover::gate_tolerance_fail_above_threshold` exits 0
    - `cargo test -p service_impl test::cutover` reports >=10 passed (8 from Plan 04-02 + 2 new)
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::cutover</automated>
  </verify>
  <done>
    Tolerance boundary tests green; diff-report file path verified writable.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Filesystem write to `.planning/migration-backup/` | Diff-report JSON is the only file write inside the cutover Tx; failure rolls back the entire Tx. |
| Tx atomicity boundary | `transaction_dao.commit(tx)` is the SINGLE atomic-flip point. Failure of any sub-service call before this point causes the Drop-rollback. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-05-01 | Tampering | Drift report could be tampered with after write | accept | Operations responsibility (filesystem perms). Audit-relevant data also persisted in `cutover_run_id` references in DB tables (mapping, quarantine, backup). |
| T-04-05-02 | Information Disclosure | Diff report contains `sales_person_name` (PII) | mitigate | `.gitignore` rule from Plan 04-00 prevents accidental git commit. Operations enforces filesystem-level access control. |
| T-04-05-03 | Tampering / Time-of-check-Time-of-use | Concurrent extra_hours INSERT during cutover Tx could pollute the gate read | mitigate | SQLite DELETE-mode + DEFERRED Tx: writers serialize on the WriteLock; concurrent INSERTs block until cutover commits/rolls back. Pitfall 4 documented. |
| T-04-05-04 | Repudiation | If commit phase fails after gate passes, no audit of attempted commit | accept | `tracing::error!` per drift row + Tx rollback ensures no inconsistent state. Optional follow-up: persist `cutover_run_id`+timestamp+outcome in a new audit table — deferred per CONTEXT.md `<deferred>`. |
| T-04-05-05 | Denial of Service | Filesystem disk-full during diff-report write rolls back the entire Tx | accept | Operations concern (disk-space monitoring before cutover). Rollback is the safe default. Documented in Open Question 3 of RESEARCH.md. |
| T-04-05-06 | Elevation of Privilege | `commit_phase` calls `FeatureFlagService::set` and `ExtraHoursService::soft_delete_bulk` with `Authentication::Full` | mitigate | Outer permission gate already enforced (`run()` checks CUTOVER_ADMIN_PRIVILEGE for commit). Inner Auth::Full is service-internal trust per established pattern (feature_flag.rs:31-41). |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test -p service_impl test::cutover` reports 10+ passed
- All Plan 04-02 tests still pass (no regression)
- `compute_gate` and `commit_phase` exist as private (or pub(crate)) helpers
- `transaction_dao.commit` and `transaction_dao.rollback` both reachable via `run()` branches
- `tracing::error!` is called at least once per drift row
- Diff-report file is created under `.planning/migration-backup/`
- The Plan-04-02 `(MigrationStats, Arc<[Uuid]>)` return-shape is consumed via `let (migration_stats, migrated_ids) = self.migrate_legacy_extra_hours_to_clusters(...)` — no `_with_ids` rename, no surface re-negotiation
- No file in `rest/`, `shifty_bin/main.rs` modified (Plan 04-06 owns those)
</verification>

<success_criteria>
1. `CutoverServiceImpl::run` is feature-complete (migration + gate + branch + commit-phase + commit/rollback).
2. Gate uses `derive_hours_for_range` directly (no re-implementation).
3. Diff-report JSON is written with the verbatim CONTEXT.md schema.
4. 2 gate-tolerance tests green at the service layer.
5. Atomic-Tx invariant holds: all sub-service calls receive `Some(tx.clone())`; no inner commit; Outer `run()` controls commit vs rollback.
6. Plan-04-02 method-signature contract `(MigrationStats, Arc<[Uuid]>)` consumed verbatim — Wave-2 surfaces zero surface negotiation.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-05-SUMMARY.md` listing:
- New private methods added (compute_gate, commit_phase, refactored run)
- File-IO path format used (Unix timestamp vs ISO-without-colons)
- Test outcomes: 2 new tests passed, 8 prior tests still green
- Verbatim destructure of Plan-04-02 contract: `let (migration_stats, migrated_ids) = self.migrate_legacy_extra_hours_to_clusters(run_id, ran_at, tx.clone()).await?;`
- Hand-off note for Plan 04-06: "CutoverServiceImpl exposes only the public CutoverService::run + ::profile API; REST handlers in Plan 04-06 call only those methods. CutoverServiceDependencies block: 10 deps total (CutoverDao, AbsenceDao, AbsenceService, ExtraHoursService, CarryoverRebuildService, FeatureFlagService, EmployeeWorkDetailsService, SalesPersonService, PermissionService, TransactionDao)."
</output>
