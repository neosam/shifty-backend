---
plan: 04-07-integration-tests-and-profile
phase: 4
wave: 3
depends_on: [04-06-cutover-rest-and-openapi, 04-05-cutover-gate-and-diff-report, 04-04-extra-hours-flag-gate-and-soft-delete]
requirements: [MIG-01, MIG-02, MIG-03, MIG-04, MIG-05, SC-1]
files_modified:
  - service_impl/src/cutover.rs
  - shifty_bin/src/integration_test.rs
  - shifty_bin/src/integration_test/cutover.rs
autonomous: true
must_haves:
  truths:
    - "`CutoverServiceImpl::profile()` is implemented (no longer a stub) per SC-1 + C-Phase4-05; produces JSON at `.planning/migration-backup/profile-{ts}.json` with bucket histograms (counts, sums, fractional-quote, weekend-on-workday-only-count, ISO-53-indicator)."
    - "`shifty_bin/src/integration_test/cutover.rs` exists with all 17 integration tests from VALIDATION.md Per-Task Verification Map (Wave 3 column)."
    - "Per-(sales_person, kategorie, jahr)-Invariant test (SC-5) passes: pre-migration sum == post-migration derived sum (≤ 0.001h drift)."
    - "Atomic-Tx-rollback-on-subservice-error test passes: when CarryoverRebuildService fails, feature_flag stays false, no soft-delete, no backup."
    - "Idempotence test passes: 2 consecutive cutover runs — second run sees 0 new clusters."
    - "All 5 REST integration tests pass (gate-dry-run + commit; HR-only forbidden; cutover_admin success; gate-fail with quarantine fixture)."
    - "Profile-via-REST integration test (test #15) passes: HR posts to `/admin/cutover/profile`, receives `CutoverProfileTO` body, JSON file generated under `.planning/migration-backup/`."
    - "Flag-gated extra_hours POST tests pass (before cutover 200; after cutover 403; ExtraWork unaffected)."
    - "`cargo test --workspace` green; `timeout 30 cargo run` boots."
  artifacts:
    - path: "service_impl/src/cutover.rs"
      provides: "Patched: profile() method real impl"
    - path: "shifty_bin/src/integration_test/cutover.rs"
      provides: "17 E2E integration tests against the real wired RestStateImpl + in-memory SQLite, plus the SC-5 invariant test (18 total)"
    - path: "shifty_bin/src/integration_test.rs"
      provides: "Patched: + mod cutover;"
  key_links:
    - from: "Integration tests"
      to: "RestStateImpl wired in main.rs (Plan 04-06)"
      via: "Test fixture constructs RestStateImpl + uses TowerService::oneshot or in-process HTTP-Client"
    - from: "SC-5 invariant test"
      to: "AbsenceService::derive_hours_for_range + CutoverDao::sum_legacy_extra_hours"
      via: "Pre-cutover: sum_legacy; post-cutover (within same Tx): derive_hours; assert delta < 0.001h"
    - from: "profile() method"
      to: "CutoverDao::find_all_legacy_extra_hours + EmployeeWorkDetailsService"
      via: "Bucket each row by (sp, category, year), compute fractional-quote + weekend-count"
    - from: "Profile-via-REST test (test #15)"
      to: "POST /admin/cutover/profile (Plan 04-06 Task 2)"
      via: "TowerService::oneshot — exercises HR permission gate, 6-DTO serialization path, and JSON-file side effect"
---

<objective>
Wave 3 — Final E2E surface. Two parallel concerns:

1. Implement `CutoverServiceImpl::profile()` (SC-1 — Production-Data-Profile). It is intentionally separate from `run()` so it can be slow on large data without affecting cutover correctness. The REST endpoint that exposes it (Plan 04-06 Task 2) calls into THIS implementation.
2. Add the full integration-test suite in `shifty_bin/src/integration_test/cutover.rs` covering MIG-01..05 + SC-1 + SC-5 + atomic-rollback + idempotence + REST permission matrix + flag-gated extra_hours behavior.

Output: 1 file extended (profile method), 1 new test file (~18 tests), 1 mod-import patch.
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
@.planning/phases/04-migration-cutover/04-05-SUMMARY.md
@.planning/phases/04-migration-cutover/04-06-SUMMARY.md

@shifty_bin/src/integration_test.rs
@shifty_bin/src/integration_test/absence_period.rs
@shifty_bin/src/integration_test/booking_absence_conflict.rs
@service_impl/src/cutover.rs

<interfaces>
<!-- Patterns to mirror. -->

From `shifty_bin/src/integration_test/absence_period.rs` (Phase-1 pattern — DB setup + RestStateImpl + tower::oneshot):
```rust
async fn setup() -> RestStateImpl {
    let pool = create_in_memory_sqlite_pool().await;
    sqlx::migrate!("migrations/sqlite").run(&pool).await.unwrap();
    build_rest_state(pool).await
}

#[tokio::test]
async fn create_absence_period_round_trip() {
    let state = setup().await;
    let app = build_router(state.clone());
    let req = Request::post("/absence-period").body(/* ... */).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);
}
```

From `shifty_bin/src/integration_test/booking_absence_conflict.rs` (Phase-3 pattern — multi-step fixture setup with soft-delete + cross-source assertions).

From CONTEXT.md `<specifics>` (Production-Data-Profile-Format-Detail per C-Phase4-05):
- Per (sp, category, year) bucket
- Fields: row_count, sum_amount, fractional_quote (% rows where amount != contract_hours), weekend_on_workday_only_contract_count, iso_53_indicator
- File: `.planning/migration-backup/profile-{ISO_TS}.json`
- Permission: HR
- REST surface: `POST /admin/cutover/profile` (Plan 04-06 Task 2)

From `rest-types/src/lib.rs` (Plan 04-06 Task 1):
```rust
pub struct CutoverProfileTO { pub profile_run_id: Uuid, pub run_at: String, pub total_buckets: u32, pub buckets: Vec<CutoverProfileBucketTO>, pub output_path: String }
pub struct CutoverProfileBucketTO { pub sales_person_id: Uuid, pub sales_person_name: String, pub category: AbsenceCategoryTO, pub year: u32, pub row_count: u32, pub sum_hours: f32, pub fractional_count: u32, pub weekend_on_workday_only_count: u32, pub iso_53_indicator: bool }
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement CutoverServiceImpl::profile() per SC-1 + C-Phase4-05</name>
  <read_first>
    - service_impl/src/cutover.rs (Wave-1 stub returns InternalError; Wave-2 implementation of run() / compute_gate())
    - service/src/cutover.rs (CutoverProfile + CutoverProfileBucket DTOs from Plan 04-01)
    - dao/src/cutover.rs (`find_all_legacy_extra_hours` method from Plan 04-01)
    - .planning/phases/04-migration-cutover/04-CONTEXT.md § "C-Phase4-05 Production-Data-Profile-Format-Detail"
  </read_first>
  <action>
**Replace the `profile()` stub** in `service_impl/src/cutover.rs` with a full implementation:

```rust
async fn profile(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<CutoverProfile, ServiceError> {
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone())
        .await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;

    let run_id = uuid::Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();
    let generated_at = time::PrimitiveDateTime::new(now.date(), now.time());

    let all_legacy = self.cutover_dao
        .find_all_legacy_extra_hours(tx.clone()).await?;

    // Pre-fetch contracts per sp (mirror cluster algorithm pre-fetch from Plan 04-02 Task 2).
    let mut work_details_by_sp: HashMap<Uuid, _> = HashMap::new();
    let mut sp_names: HashMap<Uuid, Arc<str>> = HashMap::new();
    let sps: BTreeSet<Uuid> = all_legacy.iter().map(|r| r.sales_person_id).collect();
    for sp_id in sps {
        let wd = self.employee_work_details_service
            .find_by_sales_person_id(sp_id, Authentication::Full, Some(tx.clone()))
            .await?;
        work_details_by_sp.insert(sp_id, wd);
        let sp = self.sales_person_service.get(sp_id, Authentication::Full, Some(tx.clone())).await?;
        sp_names.insert(sp_id, sp.name.clone());
    }

    // Bucket: (sp_id, AbsenceCategory, year) -> aggregate counters
    use std::collections::BTreeMap;
    let mut buckets: BTreeMap<(Uuid, AbsenceCategory, u32), CutoverProfileBucket> = BTreeMap::new();

    for row in all_legacy.iter() {
        let svc_cat = match &row.category {
            dao::extra_hours::ExtraHoursCategoryEntity::Vacation => AbsenceCategory::Vacation,
            dao::extra_hours::ExtraHoursCategoryEntity::SickLeave => AbsenceCategory::SickLeave,
            dao::extra_hours::ExtraHoursCategoryEntity::UnpaidLeave => AbsenceCategory::UnpaidLeave,
            _ => continue,
        };
        let year = row.date_time.date().year() as u32;
        let day = row.date_time.date();
        let key = (row.sales_person_id, svc_cat.clone(), year);

        let wd = work_details_by_sp.get(&row.sales_person_id).expect("pre-fetched");
        let active_contract = wd.iter().find(|w| {
            w.deleted.is_none()
                && w.from_date().map(|d| d.to_date() <= day).unwrap_or(false)
                && w.to_date().map(|d| day <= d.to_date()).unwrap_or(false)
        });
        let contract_hours = active_contract.map(|c| c.hours_per_day()).unwrap_or(0.0);
        let is_workday = active_contract.map(|c| c.has_day_of_week(day.weekday())).unwrap_or(false);
        let is_weekend_on_workday_only = !is_workday && row.amount > 0.0;
        let is_fractional = (row.amount - contract_hours).abs() > 0.001;
        let is_iso_53 = day.iso_week() == 53;

        let entry = buckets.entry(key.clone()).or_insert_with(|| CutoverProfileBucket {
            sales_person_id: row.sales_person_id,
            sales_person_name: sp_names.get(&row.sales_person_id).cloned().unwrap_or_else(|| Arc::from("")),
            category: svc_cat,
            year,
            row_count: 0,
            sum_amount: 0.0,
            fractional_count: 0,
            weekend_on_workday_only_contract_count: 0,
            iso_53_indicator: false,
        });
        entry.row_count += 1;
        entry.sum_amount += row.amount;
        if is_fractional { entry.fractional_count += 1; }
        if is_weekend_on_workday_only { entry.weekend_on_workday_only_contract_count += 1; }
        if is_iso_53 { entry.iso_53_indicator = true; }
    }

    // Persist JSON file (use Unix timestamp for filesystem-safe filename — Assumption A7).
    std::fs::create_dir_all(".planning/migration-backup")
        .map_err(|_| ServiceError::InternalError)?;
    let ts = generated_at.assume_utc().unix_timestamp();
    let profile_path = format!(".planning/migration-backup/profile-{}.json", ts);

    let body = serde_json::json!({
        "run_id": run_id.to_string(),
        "generated_at": generated_at.assume_utc()
            .format(&time::format_description::well_known::Iso8601::DEFAULT).unwrap_or_default(),
        "buckets": buckets.values().map(|b| serde_json::json!({
            "sales_person_id": b.sales_person_id.to_string(),
            "sales_person_name": b.sales_person_name,
            "category": format!("{:?}", b.category),
            "year": b.year,
            "row_count": b.row_count,
            "sum_amount": b.sum_amount,
            "fractional_count": b.fractional_count,
            "fractional_quote": if b.row_count > 0 { b.fractional_count as f32 / b.row_count as f32 } else { 0.0 },
            "weekend_on_workday_only_contract_count": b.weekend_on_workday_only_contract_count,
            "iso_53_indicator": b.iso_53_indicator,
        })).collect::<Vec<_>>(),
    });
    std::fs::write(&profile_path, serde_json::to_string_pretty(&body).unwrap())
        .map_err(|_| ServiceError::InternalError)?;

    // profile() is read-only; rollback the Tx.
    self.transaction_dao.rollback(tx).await?;

    let buckets_arc: Arc<[CutoverProfileBucket]> = Arc::from(buckets.into_values().collect::<Vec<_>>());
    Ok(CutoverProfile {
        run_id,
        generated_at,
        buckets: buckets_arc,
        profile_path: profile_path.into(),
    })
}
```
  </action>
  <acceptance_criteria>
    - `grep -q 'find_all_legacy_extra_hours' service_impl/src/cutover.rs` exits 0
    - `grep -q 'fractional_count' service_impl/src/cutover.rs` exits 0
    - `grep -q 'weekend_on_workday_only_contract_count' service_impl/src/cutover.rs` exits 0
    - `grep -q 'iso_53_indicator' service_impl/src/cutover.rs` exits 0
    - `cargo build -p service_impl` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service_impl</automated>
  </verify>
  <done>
    `profile()` produces a structured JSON profile per the SC-1 spec; the Plan-04-06 REST handler can now call it end-to-end.
  </done>
</task>

<task type="auto">
  <name>Task 2: shifty_bin/src/integration_test/cutover.rs — 18 E2E tests</name>
  <read_first>
    - shifty_bin/src/integration_test.rs
    - shifty_bin/src/integration_test/absence_period.rs (DB+RestState setup pattern)
    - shifty_bin/src/integration_test/booking_absence_conflict.rs (multi-step fixture pattern)
    - shifty_bin/src/integration_test/feature_flag.rs (if exists; permission setup pattern)
    - .planning/phases/04-migration-cutover/04-VALIDATION.md § "Per-Task Verification Map" (Wave 3 rows — every test command listed there must have an implemented test in this file)
  </read_first>
  <action>
**Patch `shifty_bin/src/integration_test.rs`:** add `mod cutover;`.

**Create `shifty_bin/src/integration_test/cutover.rs`** with these 18 tests (test names match VALIDATION.md verbatim — `cargo test` filters depend on this exact spelling):

| # | Test name | What it asserts |
|---|-----------|-----------------|
| 1 | `test_idempotence_rerun_no_op` | First commit migrates N clusters; second commit migrates 0 (mapping table catches re-runs). |
| 2 | `test_atomic_rollback_on_subservice_error` | Inject a CarryoverRebuildService failure (e.g., via a fixture that triggers a numeric overflow); assert feature_flag stays 0, extra_hours unchanged, backup table empty. |
| 3 | `test_carryover_refresh_scope_only_affected_tuples` | Fixture: 2 sps with Vac/Sick legacy, 1 sp without; assert only the 2 sps' carryover rows changed. |
| 4 | `test_pre_cutover_backup_populated_before_update` | Assert backup table has 2 rows (matching scope set) BEFORE carryover_yearly_carryover gets the rebuild. Check via Tx-savepoint trick: insert a deliberate failure AFTER backup but BEFORE rebuild_for_year and verify the partial state. (Or use `tracing` capture if Tx-savepoint is too invasive.) |
| 5 | `test_soft_delete_migrated_rows_only` | Migrate cluster of 5 rows + 1 quarantine row; commit; assert: 5 rows have `deleted IS NOT NULL` AND `update_process = 'phase-4-cutover-migration'`; 1 quarantine row has `deleted IS NULL`. |
| 6 | `test_feature_flag_set_to_true_on_commit` | Pre-commit: `feature_flag.absence_range_source_active = 0`. Post-commit: `= 1`. |
| 7 | `test_extra_hours_post_flag_gated_before_after` | Before commit: POST /extra-hours with Vacation → 200. Run cutover-commit. POST /extra-hours with Vacation → 403. POST /extra-hours with ExtraWork → 200 (still). |
| 8 | `test_403_body_format_for_deprecated_category` | After cutover, POST Vacation; assert response status 403 + body JSON has fields `error="extra_hours_category_deprecated"`, `category="vacation"`, `message="Use POST /absence-period for this category"`. |
| 9 | `test_gate_dry_run_endpoint_success` | HR user POST /admin/cutover/gate-dry-run → 200 + body `{ dry_run: true, ... }`. |
| 10 | `test_gate_dry_run_forbidden_for_unprivileged` | Non-HR user POST /admin/cutover/gate-dry-run → 403. |
| 11 | `test_gate_dry_run_returns_failure_with_quarantine` | Fixture with intentionally-broken row (e.g., 4h Vacation against 8h contract → quarantine; sum mismatch → drift); HR POST gate-dry-run → 200 with gate_passed=false, drift_drift_rows>0. |
| 12 | `test_commit_forbidden_for_hr_only` | HR-only user (no cutover_admin) POST /admin/cutover/commit → 403. |
| 13 | `test_commit_success_for_cutover_admin` | cutover_admin user POST /admin/cutover/commit on clean fixture → 200, gate_passed=true, feature_flag flipped. |
| 14 | `test_diff_report_json_schema` | After gate run, parse the `.planning/migration-backup/cutover-gate-{ts}.json`; assert all 7 top-level fields exist (gate_run_id, run_at, dry_run, drift_threshold, total_drift_rows, drift, passed). |
| **15** | `test_profile_generates_json_with_histograms` | **Plan 04-06 Task 2 added `POST /admin/cutover/profile`. This test calls the REST endpoint, not the service method directly.** Build a fixture (≥ 2 sps × 2 categories × 1 year × ≥ 3 rows, with at least 1 fractional + 1 weekend-on-workday-only entry to populate the histogram counters). Authenticate as HR. Use `tower::ServiceExt::oneshot` to issue `POST /admin/cutover/profile` with empty JSON body (`CutoverProfileRequest {}`). Assert: (a) HTTP status `200 OK`; (b) response body parses as `CutoverProfileTO` (deserialize via `serde_json`); (c) `body.total_buckets == buckets.len()`; (d) `buckets[i]` carries the C-Phase4-05 fields (`sales_person_id`, `sales_person_name`, `category`, `year`, `row_count`, `sum_hours`, `fractional_count`, `weekend_on_workday_only_count`, `iso_53_indicator`); (e) `body.output_path` starts with `.planning/migration-backup/profile-`; (f) the file referenced by `output_path` exists on disk and parses as JSON; (g) at least one bucket has `fractional_count > 0` and at least one has `weekend_on_workday_only_count > 0` (proves the per-day contract lookup is exercised). Cleanup: `std::fs::remove_file(output_path)` at end-of-test. |
| 16 | `test_gate_uses_derive_hours_for_range_path` | Sentinel-test: construct a fixture where `extra_hours.amount` summed = 100 and AbsencePeriod-derived = 100 (perfect match → gate passes). Mutate `EmployeeWorkDetails` to halve `hours_per_day` → re-run gate → derive_hours_for_range output halves → drift = 50, gate fails. Proves gate uses derive path (not a re-implementation). |
| 17 | `test_gate_fail_no_state_change` | Fixture with quarantine + drift → POST commit → 200 with gate_passed=false → ALL of: feature_flag still 0, extra_hours unchanged, no backup row, no carryover write. (SC-3 atomicity.) |

Plus the SC-5 invariant test (already in VALIDATION.md as `per_sales_person_per_year_per_category_invariant`):

| # | Test name | What it asserts |
|---|-----------|-----------------|
| 18 | `per_sales_person_per_year_per_category_invariant` | Fixture: 2 sps × 2 years × 3 categories × ≥3 clusters each. Pre-commit: `cutover_dao.sum_legacy_extra_hours(sp, cat, yr)`. Run commit. Post-commit (within fresh Tx): for each (sp, cat, year), compute `derive_hours_for_range(year_start..year_end, sp).where_category(cat).sum()`. Assert `(pre - post).abs() < 0.001` per tuple. |

**Test infrastructure:** mirror the absence_period.rs setup — in-memory SQLite pool + `sqlx::migrate!()` + `RestStateImpl` builder. For HTTP tests use `tower::ServiceExt::oneshot` against the router. For permission setup, look for an existing helper that creates users with specific privileges (likely in `integration_test/permission_test.rs` or `mock_auth` setup).

If a needed test infrastructure doesn't exist (e.g., creating a `cutover_admin` user fixture), add a small test-helper at the top of `cutover.rs`:

```rust
async fn add_user_with_privilege(state: &RestStateImpl, user: &str, privilege: &str) {
    // ... DAO insert into user_role + role_privilege tables ...
}
```

For tests #2 and #4 (atomic-rollback / before-rebuild assertion): if forcing a sub-service failure mid-Tx is hard, use a mock-friendlier approach — for example a fixture that triggers a real DAO failure (e.g., reference a non-existent FK) at the right step.

**For test #15:** the REST handler signature is `POST /admin/cutover/profile` with empty `CutoverProfileRequest {}` body. The response body deserializes to `rest_types::CutoverProfileTO`. If the test fixture's HR-user setup is missing a privilege (`HR_PRIVILEGE`), assert 403 first to verify the permission gate, then upgrade the user to HR and retry.
  </action>
  <acceptance_criteria>
    - File `shifty_bin/src/integration_test/cutover.rs` exists
    - `grep -q 'mod cutover' shifty_bin/src/integration_test.rs` exits 0
    - `grep -c '#\[tokio::test\]' shifty_bin/src/integration_test/cutover.rs` returns >= 18
    - All 18 test names from VALIDATION.md Per-Task Verification Map exist:
      - `grep -q 'fn test_idempotence_rerun_no_op' shifty_bin/src/integration_test/cutover.rs` exits 0
      - `grep -q 'fn test_atomic_rollback_on_subservice_error' shifty_bin/src/integration_test/cutover.rs` exits 0
      - `grep -q 'fn test_profile_generates_json_with_histograms' shifty_bin/src/integration_test/cutover.rs` exits 0
      - `grep -q 'fn per_sales_person_per_year_per_category_invariant' shifty_bin/src/integration_test/cutover.rs` exits 0
      - (… and the other 14 — verify each)
    - **Test #15 uses REST, not service-method-direct:** `grep -q '"/admin/cutover/profile"' shifty_bin/src/integration_test/cutover.rs` exits 0 AND `grep -q 'CutoverProfileTO' shifty_bin/src/integration_test/cutover.rs` exits 0
    - `cargo test --workspace` exits 0 (or all tests targeted via `cargo test cutover` exit 0)
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p shifty_bin --test integration_test cutover</automated>
  </verify>
  <done>
    All 18 integration tests pass; MIG-01..05 + SC-1 + SC-3 + SC-5 verified end-to-end. Test #15 exercises the full REST path for the profile endpoint (HR auth → handler → service → DAO → JSON file → response body).
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Test ↔ Real DB | In-memory SQLite — no production data exposure. |
| Profile() write to filesystem | Test cleanup MUST remove generated JSON files to avoid polluting the repo. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-07-01 | Tampering | A test could leave the FeatureFlag in a flipped state, contaminating a subsequent test in the same process | mitigate | Each test uses its own in-memory pool (per absence_period.rs setup pattern); state is fresh per test. |
| T-04-07-02 | Repudiation | Test assertions could pass a buggy implementation if the assertions are weak | mitigate | Per-test assertions use exact-equality on numeric fields (drift < 0.001), structural-match on JSON keys, status code assertions. SC-5 is a closed-loop invariant (pre-sum == post-derived). |
| T-04-07-03 | Information Disclosure | Tests print real fixture data (sp_id, sp_name) to stdout on failure | accept | Test fixtures use synthetic data (e.g., "Test Person 1") — no real PII. |
| T-04-07-04 | DoS | Large-fixture tests could exceed CI runtime budget | mitigate | Fixture sizes capped at moderate (≤ 100 rows per test); SC-5 fixture is 2 sps × 2 years × 3 cats × 3 clusters = 36 rows worst case. |
| T-04-07-05 | Tampering | Test #2 (atomic-rollback-on-subservice-error) — forced failure must be inside the Tx, not at REST level | mitigate | Use a DAO-level failure (e.g., FK violation) that surfaces inside the inner Tx scope. Documented in test comment. |
| T-04-07-06 | Spoofing — Profile-via-REST | Test #15 must verify HR-only access; weakening to "anyone authenticated" would mask a permission regression | mitigate | Test #15 first attempts the call as a non-HR user (expect 403), then re-attempts as HR (expect 200). Documented in test comment. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test --workspace` GREEN (all 18 new integration tests + all prior tests)
- `cargo run` boots within 30s (final Bin-Boot-Smoke)
- `.planning/migration-backup/*.json` files generated by tests are cleaned up at end of each test (use `Drop`-guard or explicit `std::fs::remove_file` in test-end)
- `cargo test -p rest --test openapi_snapshot openapi_snapshot_locks_full_api_surface` still GREEN (Plan 04-06 snapshot still valid — Plan 04-07 doesn't change the OpenAPI surface)
- VALIDATION.md `nyquist_compliant: false` can be flipped to `true` after this plan lands (as a Wave-3-cleanup task in `04-07-SUMMARY.md`)
- Test #15 exercises the REST path (not service-method-direct), proving SC-1's REST surface is reachable in production-like conditions.
</verification>

<success_criteria>
1. All 17 VALIDATION.md Wave-3 tests + the SC-5 invariant test = 18 total, all green.
2. profile() implementation generates JSON with the documented schema.
3. Test #15 covers the full REST path for profile (HR auth → handler → service → JSON file).
4. Atomic-Tx invariant verified end-to-end (test #2 + test #17).
5. Idempotence verified end-to-end (test #1).
6. Cross-cutting invariant per (sp, cat, year) verified (test #18).
7. REST permission matrix verified (tests #9-#13 + #7 + #15).
8. Phase 4 is feature-complete and ready for `/gsd:verify-phase 04`.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-07-SUMMARY.md` listing:
- Profile() implementation summary (line count, key data shape)
- Integration test summary: 18 test names + per-test PASS/FAIL
- VALIDATION.md sign-off: flip `nyquist_compliant: true` if all sampling-rate boxes pass
- Final phase status: Phase 4 ready for `/gsd:verify-phase 04`
- Cross-cutting truths verified across all plans:
  - Atomic-Tx via TransactionDao::use_transaction (Pattern 1) — verified by test #2, test #17
  - OpenAPI snapshot pin locks API surface — verified by Plan 04-06 Task 6 + this plan's no-op snapshot run
  - Service-Tier-Konvention preserved — verified by absence of CutoverService consumption from any sub-service grep
  - Authentication::Full bypass for service-internal calls — verified by run() acceptance + carryover_rebuild test
  - SC-1 REST surface end-to-end exercised by test #15 (profile-via-REST)
- Hand-off note: "/gsd:verify-phase 04 is the next command. SUMMARY files for plans 04-00..04-07 are ready; ROADMAP can be marked Phase 4 complete after verify."
</output>
