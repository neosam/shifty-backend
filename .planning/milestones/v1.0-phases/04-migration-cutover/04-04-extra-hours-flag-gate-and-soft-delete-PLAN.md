---
plan: 04-04-extra-hours-flag-gate-and-soft-delete
phase: 4
wave: 1
depends_on: [04-01-service-traits-and-stubs]
requirements: [MIG-05]
files_modified:
  - service_impl/src/extra_hours.rs
  - dao/src/extra_hours.rs
  - dao_impl_sqlite/src/extra_hours.rs
  - service_impl/src/test/extra_hours.rs
autonomous: true
must_haves:
  truths:
    - "`ExtraHoursService::soft_delete_bulk(ids, update_process, ctx, tx)` is implemented in service_impl: requires CUTOVER_ADMIN_PRIVILEGE OR Authentication::Full; calls a new bulk DAO method."
    - "`ExtraHoursServiceImpl::create` has a flag-gated pre-check (per RESEARCH.md Operation 5): if `feature_flag_service.is_enabled(\"absence_range_source_active\", ...)` AND category in {Vacation, SickLeave, UnpaidLeave} → return `Err(ServiceError::ExtraHoursCategoryDeprecated(category))`."
    - "Service-Layer flag-gate (NOT REST-layer) chosen per Architectural Responsibility Map row 9 — better test isolation."
    - "FeatureFlagService is added as a new DI dependency on `ExtraHoursServiceImpl` (RESEARCH.md Operation 5 hint at Z. 780)."
    - "Bulk DAO method `extra_hours_dao.soft_delete_bulk(ids, update_process, tx)` exists in dao + dao_impl_sqlite."
    - "Tests: 5 new tests cover (a) flag-off + Vacation -> 200, (b) flag-on + Vacation -> 403, (c) flag-on + ExtraWork -> 200, (d) soft_delete_bulk happy path, (e) `soft_delete_bulk_forbidden_for_unprivileged_user` — proving Permission gate sits BEFORE the DAO call."
  artifacts:
    - path: "service_impl/src/extra_hours.rs"
      provides: "Patched: + soft_delete_bulk impl, + flag-gated create-pre-check, + new FeatureFlagService dep"
    - path: "dao/src/extra_hours.rs"
      provides: "Patched: + soft_delete_bulk(ids, update_process, tx) trait method"
    - path: "dao_impl_sqlite/src/extra_hours.rs"
      provides: "Patched: SQLx impl of soft_delete_bulk (single UPDATE with IN clause via QueryBuilder)"
    - path: "service_impl/src/test/extra_hours.rs"
      provides: "Patched: 3 flag-gate tests + 1 soft_delete_bulk happy-path test + 1 soft_delete_bulk_forbidden test"
  key_links:
    - from: "service_impl::extra_hours::ExtraHoursServiceImpl::create"
      to: "service::feature_flag::FeatureFlagService::is_enabled"
      via: "Pre-create flag check, exact pattern from service_impl/src/reporting.rs:475"
    - from: "ServiceError::ExtraHoursCategoryDeprecated(category)"
      to: "Plan 04-06 error_handler 403 mapping"
      via: "Plan 04-04 sets the error variant; Plan 04-06 maps it to HTTP"
    - from: "ExtraHoursService::soft_delete_bulk"
      to: "Plan 04-05 (cutover commit phase step c)"
      via: "Called inside the cutover Tx with `Authentication::Full` once gate passes"
    - from: "ExtraHoursServiceImpl test-helper(s) constructing the impl directly"
      to: "Updated test-helpers across service_impl/src/test/ + shifty_bin/src/"
      via: "New `feature_flag_service` field MUST be added to every `ExtraHoursServiceImpl{...}` initializer (test-helpers + main.rs DI block)"
---

<objective>
Wave 1 — Patch the existing `ExtraHoursService` to (a) flag-gate the `create` method (D-Phase4-09 — service-layer check per Architectural Responsibility Map), (b) add the new `soft_delete_bulk` method that the cutover commit phase will call. Both changes are surgical patches to the existing extra_hours service + DAO + tests.

This plan is independent from Plan 04-02 (cutover heuristic) and Plan 04-03 (carryover rebuild) — no file overlap. They run in parallel within Wave 1.

Purpose: Cleanly extend the existing extra_hours surface so Wave 2 commit phase has all needed lever points. Flag-gate testing requires a new `FeatureFlagService` DI dep on `ExtraHoursServiceImpl` — a small, contained change that ripples into every test-helper that constructs the impl.

Output: 4 file patches + 5 new tests; test-helper updates for the new DI dep.
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
@.planning/phases/04-migration-cutover/04-01-SUMMARY.md

@service/src/extra_hours.rs
@service_impl/src/extra_hours.rs
@dao/src/extra_hours.rs
@dao_impl_sqlite/src/extra_hours.rs
@service_impl/src/test/extra_hours.rs
@service_impl/src/feature_flag.rs
@service_impl/src/reporting.rs

<interfaces>
<!-- Verbatim contracts to consume. -->

From `service_impl/src/reporting.rs:460-505` (Phase-2 reporting-switch — verbatim flag-check pattern):
```rust
let flag_active = self.feature_flag_service
    .is_enabled("absence_range_source_active", Authentication::Full, Some(tx.clone()))
    .await?;
if flag_active { /* new path */ } else { /* legacy path */ }
```

From `service/src/extra_hours.rs` (Plan 04-01 trait extension):
```rust
async fn soft_delete_bulk(
    &self,
    ids: Arc<[Uuid]>,
    update_process: &str,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError>;
```

From `service/src/cutover.rs` (Plan 04-01 export):
```rust
pub const CUTOVER_ADMIN_PRIVILEGE: &str = "cutover_admin";
```

From `service/src/lib.rs` (Plan 04-01 ServiceError variant):
```rust
#[error("ExtraHours category {0:?} is deprecated; use POST /absence-period for this category")]
ExtraHoursCategoryDeprecated(crate::extra_hours::ExtraHoursCategory),
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Patch ExtraHoursService trait + DAO trait — soft_delete_bulk signature</name>
  <read_first>
    - dao/src/extra_hours.rs (existing trait — find pattern of similar bulk-write methods if any)
    - service/src/extra_hours.rs (Plan 04-01 already added the service trait method — verify present)
  </read_first>
  <action>
**Patch `dao/src/extra_hours.rs`:** add a new method to the `ExtraHoursDao` trait (inside the `#[automock] #[async_trait] pub trait ExtraHoursDao` block, after the existing `delete` or similar):

```rust
    /// Bulk soft-delete by id list (Phase 4 cutover, C-Phase4-04).
    /// Issues `UPDATE extra_hours SET deleted = ?, update_process = ?, version = ? WHERE id IN (...)`.
    /// Silently no-ops on ids that are already soft-deleted (`deleted IS NOT NULL`) to keep
    /// re-runs idempotent. Caller passes the cutover Tx as `tx`.
    async fn soft_delete_bulk(
        &self,
        ids: &[uuid::Uuid],
        deleted_at: time::PrimitiveDateTime,
        update_process: &str,
        new_version: uuid::Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
```

Keep the existing trait shape (Authentication-free at DAO layer) — DAO traits do not take `context`.
  </action>
  <acceptance_criteria>
    - `grep -q 'async fn soft_delete_bulk' dao/src/extra_hours.rs` exits 0
    - `cargo build -p dao` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p dao</automated>
  </verify>
  <done>
    DAO trait extended; mock auto-generated.
  </done>
</task>

<task type="auto">
  <name>Task 2: dao_impl_sqlite — soft_delete_bulk SQLx implementation</name>
  <read_first>
    - dao_impl_sqlite/src/extra_hours.rs (existing impl — find an example UPDATE with WHERE clause and version+update_process+update_timestamp pattern)
    - dao/src/extra_hours.rs (Task 1 above)
  </read_first>
  <action>
Add the impl method to `dao_impl_sqlite/src/extra_hours.rs` inside the `#[async_trait] impl ExtraHoursDao for ExtraHoursDaoImpl` block:

```rust
async fn soft_delete_bulk(
    &self,
    ids: &[uuid::Uuid],
    deleted_at: time::PrimitiveDateTime,
    update_process: &str,
    new_version: uuid::Uuid,
    tx: Self::Transaction,
) -> Result<(), DaoError> {
    if ids.is_empty() {
        return Ok(());
    }

    let mut qb = sqlx::QueryBuilder::new(
        "UPDATE extra_hours SET deleted = ", // placeholder for deleted_at + update_process + version
    );
    qb.push_bind(deleted_at)
      .push(", update_process = ").push_bind(update_process)
      .push(", update_timestamp = ").push_bind(deleted_at)
      .push(", version = ").push_bind(new_version)
      .push(" WHERE deleted IS NULL AND id IN (");

    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(*id);
    }
    qb.push(")");

    let mut tx_guard = tx.lock().await;  // adapt to actual TransactionImpl ergonomics
    qb.build().execute(&mut **tx_guard).await?;

    Ok(())
}
```

**Notes:**
- Verify the actual fields on `extra_hours` schema: `deleted`, `update_process`, `update_timestamp`, `version` exist (per `migrations/sqlite/20240618125847_paid-sales-persons.sql`). Adapt column names if they differ.
- Verify the existing `TransactionImpl` mutex-borrow pattern in `dao_impl_sqlite/src/extra_hours.rs` — adapt `tx_guard` accordingly.
- `WHERE deleted IS NULL` ensures already-soft-deleted rows are skipped (idempotent re-runs).
- If your existing DAO uses `query_with` instead of `QueryBuilder`, follow that pattern.
  </action>
  <acceptance_criteria>
    - `grep -q 'fn soft_delete_bulk' dao_impl_sqlite/src/extra_hours.rs` exits 0
    - `grep -q 'WHERE deleted IS NULL' dao_impl_sqlite/src/extra_hours.rs` exits 0
    - `cargo build -p dao_impl_sqlite` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p dao_impl_sqlite</automated>
  </verify>
  <done>
    DAO impl compiles; idempotent bulk soft-delete is available to the service layer.
  </done>
</task>

<task type="auto">
  <name>Task 3: ExtraHoursServiceImpl — soft_delete_bulk impl + flag-gated create + new FeatureFlagService DI dep</name>
  <read_first>
    - service_impl/src/extra_hours.rs (existing impl — find `create` method, find DI block, find existing pattern for getting `update_process`)
    - service/src/cutover.rs (CUTOVER_ADMIN_PRIVILEGE)
    - service_impl/src/feature_flag.rs (Z. 31-67 Authentication::Full bypass + check_permission pattern)
    - service_impl/src/reporting.rs (Z. 460-505 flag-check verbatim pattern)
    - .planning/phases/04-migration-cutover/04-RESEARCH.md § "Operation 5: Service-Layer Flag-Check"
  </read_first>
  <action>
**Patch `service_impl/src/extra_hours.rs`:**

a) Extend the `gen_service_impl!` (or equivalent DI block) to add `FeatureFlagService` as a new dep:
```rust
gen_service_impl! {
    struct ExtraHoursServiceImpl: service::extra_hours::ExtraHoursService = ExtraHoursServiceDeps {
        ExtraHoursDao: dao::extra_hours::ExtraHoursDao = extra_hours_dao,
        // ... all existing deps ...
        FeatureFlagService: service::feature_flag::FeatureFlagService = feature_flag_service,  // NEW Phase-4
        PermissionService: service::permission::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}
```

b) Patch the existing `create` method — insert a flag-gate check AFTER the existing permission check + use_transaction, BEFORE the existing DAO insert (per Operation 5 in RESEARCH.md, verbatim):

```rust
async fn create(
    &self,
    entity: &ExtraHours,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<ExtraHours, ServiceError> {
    self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // Phase-4 D-Phase4-09 flag-gate: deprecated categories produce 403 once cutover is live.
    if matches!(entity.category,
        ExtraHoursCategory::Vacation | ExtraHoursCategory::SickLeave | ExtraHoursCategory::UnpaidLeave)
    {
        let flag_active = self
            .feature_flag_service
            .is_enabled("absence_range_source_active", Authentication::Full, Some(tx.clone()))
            .await?;
        if flag_active {
            // Tx will rollback via Drop — no state change.
            return Err(ServiceError::ExtraHoursCategoryDeprecated(entity.category.clone()));
        }
    }

    // ... existing create logic continues unchanged ...
}
```

**Important:** the `Tx will rollback via Drop` comment supersedes RESEARCH.md Operation 5's `commit-then-error` sketch — rollback-on-drop is cleaner and matches the Pattern-1 Tx-forwarding contract. If existing tests have specific Tx-commit expectations on `create` failure paths, adapt accordingly (verify by reading `service_impl/src/test/extra_hours.rs` first).

c) Implement the new trait method `soft_delete_bulk`:

```rust
async fn soft_delete_bulk(
    &self,
    ids: Arc<[uuid::Uuid]>,
    update_process: &str,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError> {
    // Permission gate FIRST — strictly BEFORE the DAO call. This ordering is
    // verified by `soft_delete_bulk_forbidden_for_unprivileged_user` in Task 4
    // via `MockExtraHoursDao::expect_soft_delete_bulk().times(0)` once permission denies.
    self.permission_service
        .check_permission(service::cutover::CUTOVER_ADMIN_PRIVILEGE, context.clone())
        .await?;

    let tx = self.transaction_dao.use_transaction(tx).await?;

    let now = time::OffsetDateTime::now_utc();
    let now = time::PrimitiveDateTime::new(now.date(), now.time());
    let new_version = uuid::Uuid::new_v4();

    self.extra_hours_dao
        .soft_delete_bulk(&ids, now, update_process, new_version, tx)
        .await?;

    Ok(())
}
```

The TX is held by the caller (Plan 04-05 cutover commit phase) so this method does NOT commit.
  </action>
  <acceptance_criteria>
    - `grep -q 'feature_flag_service' service_impl/src/extra_hours.rs` exits 0 (new DI dep)
    - `grep -q 'is_enabled("absence_range_source_active"' service_impl/src/extra_hours.rs` exits 0
    - `grep -q 'ExtraHoursCategoryDeprecated' service_impl/src/extra_hours.rs` exits 0
    - `grep -q 'fn soft_delete_bulk' service_impl/src/extra_hours.rs` exits 0
    - `grep -q 'CUTOVER_ADMIN_PRIVILEGE' service_impl/src/extra_hours.rs` exits 0
    - `cargo build -p service_impl` exits 0
  </acceptance_criteria>
  <verify>
    <automated>cargo build -p service_impl</automated>
  </verify>
  <done>
    ExtraHoursServiceImpl has flag-gated create + bulk-soft-delete + new FeatureFlagService dep. Existing tests still compile (mocks updated next task).
  </done>
</task>

<task type="auto">
  <name>Task 4: service_impl/src/test/extra_hours.rs — 5 new tests + update existing test mocks for new FeatureFlagService dep</name>
  <read_first>
    - service_impl/src/test/extra_hours.rs (existing tests — find the test helper that constructs ExtraHoursServiceImpl with mocks; the new FeatureFlagService dep needs a default mock)
    - service_impl/src/test/feature_flag.rs (MockFeatureFlagService usage pattern)
    - service_impl/src/extra_hours.rs (Task 3 above)
    - **Test-helper sweep — REQUIRED before patching:**
      ```bash
      grep -rn 'ExtraHoursServiceImpl {' service_impl/src/test/ shifty_bin/src/
      ```
      Every match is a constructor that needs the new `feature_flag_service` field added. Do NOT skip any — silent compile breakage in another test module is the failure mode.
  </read_first>
  <action>
**Patch existing test helper(s):** wherever `ExtraHoursServiceImpl { ... }` is constructed in tests, add a default `MockFeatureFlagService`. For tests that don't care about the flag, set `expect_is_enabled().returning(|_, _, _| Box::pin(async { Ok(false) }))`. This keeps the legacy create-path active for existing tests.

The pre-implementation `grep` from `<read_first>` enumerates every call-site; tick each one off as you patch it. The list typically includes test-helpers in `service_impl/src/test/extra_hours.rs` and the actual DI initializer in `shifty_bin/src/main.rs` (Plan 04-06 owns the main.rs change but you must not leave the workspace red mid-wave — if the main.rs DI is the only outstanding red, document it in the SUMMARY hand-off and tighten the verify command to only the test target until Plan 04-06 lands).

**Add 5 new tests:**

```rust
// Add at end of service_impl/src/test/extra_hours.rs

#[tokio::test]
async fn create_vacation_succeeds_when_flag_off() {
    // Arrange: MockFeatureFlagService returns false; expect DAO::create called once.
    // ... build service via test helper, override flag-mock to return Ok(false) ...
    let result = svc.create(&vacation_entity, Authentication::Full, None).await;
    assert!(result.is_ok(), "got: {:?}", result);
}

#[tokio::test]
async fn create_vacation_returns_403_error_variant_when_flag_on() {
    // Arrange: MockFeatureFlagService returns true; expect DAO::create called 0 times.
    // ... ...
    let result = svc.create(&vacation_entity, Authentication::Full, None).await;
    assert!(matches!(
        result,
        Err(ServiceError::ExtraHoursCategoryDeprecated(ExtraHoursCategory::Vacation))
    ), "got: {:?}", result);
}

#[tokio::test]
async fn create_extra_work_succeeds_when_flag_on() {
    // Arrange: MockFeatureFlagService returns true; expect DAO::create called once
    // (ExtraWork is NOT in the gated set).
    // ... ...
    let result = svc.create(&extra_work_entity, Authentication::Full, None).await;
    assert!(result.is_ok(), "got: {:?}", result);
}

#[tokio::test]
async fn soft_delete_bulk_calls_dao_with_provided_ids_and_update_process() {
    // Arrange: 3 ids; MockExtraHoursDao::expect_soft_delete_bulk
    //   .withf(|ids, _, up, _, _| ids.len() == 3 && up == "phase-4-cutover-migration")
    //   .returning(|_, _, _, _, _| Ok(()));
    // MockPermissionService grants CUTOVER_ADMIN.
    let ids: Arc<[Uuid]> = Arc::from(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);
    let result = svc.soft_delete_bulk(ids, "phase-4-cutover-migration", Authentication::Full, None).await;
    assert!(result.is_ok(), "got: {:?}", result);
}

#[tokio::test]
async fn soft_delete_bulk_forbidden_for_unprivileged_user() {
    // Per CLAUDE.md (memory) — every new public service method gets a `_forbidden`
    // counterpart. This test ALSO proves the permission check sits BEFORE the DAO call:
    // the MockExtraHoursDao::expect_soft_delete_bulk().times(0) will fail the test
    // if the implementation calls the DAO before the permission gate denies.
    use mockall::predicate::eq;

    let mut deps = NoneTypeDeps::default();   // adapt to existing test-helper builder
    deps.permission_service
        .expect_check_permission()
        .with(eq(service::cutover::CUTOVER_ADMIN_PRIVILEGE), mockall::predicate::always())
        .returning(|_, _| Box::pin(async { Err(ServiceError::Forbidden) }));
    deps.extra_hours_dao
        .expect_soft_delete_bulk()
        .times(0);   // CRITICAL: proves permission check sits BEFORE DAO call

    let svc = ExtraHoursServiceImpl::build(deps);   // adapt to actual builder pattern
    let ids: Arc<[Uuid]> = Arc::from(vec![Uuid::new_v4()].as_slice());
    let result = svc
        .soft_delete_bulk(ids, "test-process", Authentication::Full, None)
        .await;

    assert!(matches!(result, Err(ServiceError::Forbidden)), "got: {:?}", result);
}
```

For each test, follow the existing test-construction pattern in `service_impl/src/test/extra_hours.rs` for mock setup boilerplate. The exact builder name (`NoneTypeDeps::default()` vs `default_test_deps()` vs other) must be matched verbatim from existing tests in the same file.
  </action>
  <acceptance_criteria>
    - All five new tests are present (`grep -c '#\[tokio::test\]' service_impl/src/test/extra_hours.rs` increased by 5 vs. baseline)
    - `cargo test -p service_impl test::extra_hours::create_vacation_succeeds_when_flag_off` exits 0
    - `cargo test -p service_impl test::extra_hours::create_vacation_returns_403_error_variant_when_flag_on` exits 0
    - `cargo test -p service_impl test::extra_hours::create_extra_work_succeeds_when_flag_on` exits 0
    - `cargo test -p service_impl test::extra_hours::soft_delete_bulk_calls_dao_with_provided_ids_and_update_process` exits 0
    - `cargo test -p service_impl test::extra_hours::soft_delete_bulk_forbidden_for_unprivileged_user` exits 0
    - All pre-existing `test::extra_hours` tests still exit 0 (no regression)
    - Workspace-wide build/test sweep is GREEN: `cargo test --workspace` exits 0 (catches any test-helper that uses `ExtraHoursServiceImpl { ... }` outside the immediate `test::extra_hours` module). If main.rs DI in Plan 04-06 is the only outstanding red, the executor must document this in the SUMMARY and split the verify command (target test::extra_hours first, then full workspace once Plan 04-06 lands).
  </acceptance_criteria>
  <verify>
    <automated>cargo test -p service_impl test::extra_hours && cargo test --workspace</automated>
  </verify>
  <done>
    Flag-gate behavior proven at the service layer; bulk-soft-delete proven (happy path + forbidden); no regression on existing tests; workspace-wide compile sweep clean.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| HTTP → service `create` | flag-gate check happens INSIDE service layer (post-permission); REST layer does no extra check. |
| Service → DAO `soft_delete_bulk` | bypass per-row permissions — caller gated by CUTOVER_ADMIN at the service layer. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-04-01 | Elevation of Privilege | `soft_delete_bulk` could erase arbitrary `extra_hours` rows | mitigate | `check_permission(CUTOVER_ADMIN_PRIVILEGE, ctx)` first action — verified by `soft_delete_bulk_forbidden_for_unprivileged_user` (Task 4): the test sets `expect_soft_delete_bulk().times(0)` AND a denying MockPermissionService, so the test fails if the impl calls the DAO before the permission check. The cutover-only caller (Plan 04-05) constructs the id list from its own internal mapping table. |
| T-04-04-02 | Spoofing | Flag-gate could be bypassed via REST handler bypassing service | mitigate | Service-Layer check (per ARM row 9) — REST handler unchanged; any future REST surface that posts `extra_hours` flows through `ExtraHoursServiceImpl::create` which enforces the gate. |
| T-04-04-03 | Time-of-check-Time-of-use | Flag-check at create-time vs. cutover-Tx commit could race | mitigate | Both reads happen within their own Tx; SQLite SERIALIZABLE ensures consistent view. The cutover Tx holds the WriteLock during commit, so concurrent `create` calls block until Tx completes (DELETE-mode + DEFERRED isolation). Documented in Pitfall 4. |
| T-04-04-04 | Information Disclosure | Error message via `ExtraHoursCategoryDeprecated` includes the category name | accept | Public REST surface (Plan 04-06) returns `category` field intentionally — clients need to know which category is deprecated. No PII. |
</threat_model>

<verification>
- `cargo build --workspace` GREEN
- `cargo test -p service_impl test::extra_hours` reports +5 passed; no regressions
- `cargo test --workspace` GREEN (workspace-sweep catches any test-helper that constructs `ExtraHoursServiceImpl{...}` and was missed in the patch)
- `cargo test -p dao_impl_sqlite` GREEN (compile-time queries against `extra_hours` schema verified)
- No file in `rest/` modified (Plan 04-06 owns REST changes)
- New FeatureFlagService dep on ExtraHoursServiceImpl is documented for Plan 04-06 DI re-order
- Forbidden-test ordering verified: `soft_delete_bulk_forbidden_for_unprivileged_user` proves permission check sits BEFORE DAO call (mock `times(0)` assertion)
</verification>

<success_criteria>
1. Service-layer flag-gate is in place; tests prove the 3 categories x 2 flag-states matrix.
2. Bulk soft-delete is implementable surface (callable by cutover Tx in Plan 04-05).
3. `_forbidden`-counterpart for soft_delete_bulk exists (CLAUDE.md memory rule satisfied).
4. `FeatureFlagService` is a new dep on `ExtraHoursServiceImpl` — Plan 04-06 must re-order DI in `shifty_bin/src/main.rs` to construct FeatureFlagService BEFORE ExtraHoursService (RESEARCH.md Operation 5 footnote).
5. No regression in existing `test::extra_hours` suite; workspace-wide compile sweep clean.
</success_criteria>

<output>
After completion, create `.planning/phases/04-migration-cutover/04-04-SUMMARY.md` listing:
- DI dep added: FeatureFlagService on ExtraHoursServiceImpl
- Tests added: 5 (3 flag-gate + 1 soft_delete_bulk happy + 1 soft_delete_bulk_forbidden)
- Test-helper sites updated (output of `grep -rn 'ExtraHoursServiceImpl {' service_impl/src/test/ shifty_bin/src/` enumerated and individually patched)
- Hand-off note for Plan 04-06: "ExtraHoursServiceImpl now requires FeatureFlagService — DI re-order in shifty_bin/src/main.rs MANDATORY: FeatureFlagService construction MUST happen BEFORE ExtraHoursService construction (currently FeatureFlagService is at Z. 795, ExtraHoursService at Z. 770 — must swap)."
- Hand-off note for Plan 04-05: "ExtraHoursService::soft_delete_bulk(ids, update_process, ctx, tx) is callable; cutover commit phase calls it with `Authentication::Full` + cutover Tx."
</output>
