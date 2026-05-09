//! Phase 4 — service-level cutover tests.
//!
//! Wave 1 implements:
//!   - 1 cluster-merge happy-path test
//!   - 5 quarantine reason tests (amount-below, amount-above, weekend-on-mo-fr,
//!     contract-not-active, iso-53-gap)
//!   - 1 idempotence test (re-run with empty legacy → 0 clusters, 0 quarantine)
//!   - 2 forbidden tests (HR for commit, unprivileged for dry_run)
//!
//! Wave 2 plans (04-05) will activate the gate-tolerance tests still marked
//! `#[ignore = "wave-2-..."]`.

use std::collections::BTreeMap;
use std::sync::Arc;

use mockall::predicate::{always, eq};
use time::macros::{date, datetime};
use uuid::Uuid;

use dao::absence::MockAbsenceDao;
use dao::cutover::{LegacyExtraHoursRow, MockCutoverDao};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::{MockTransaction, MockTransactionDao};
use service::absence::{AbsenceCategory, MockAbsenceService, ResolvedAbsence};
use service::carryover_rebuild::MockCarryoverRebuildService;
use service::cutover::{CutoverService, CUTOVER_ADMIN_PRIVILEGE};
use service::employee_work_details::{
    EmployeeWorkDetails, MockEmployeeWorkDetailsService,
};
use service::extra_hours::MockExtraHoursService;
use service::feature_flag::MockFeatureFlagService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::{MockPermissionService, ServiceError};
use shifty_utils::DayOfWeek;

use crate::cutover::{CutoverServiceDeps, CutoverServiceImpl};

// ----------------------------------------------------------------------------
// Test harness — multi-mock dependency injection
// ----------------------------------------------------------------------------

pub(crate) struct CutoverDependencies {
    pub cutover_dao: MockCutoverDao,
    pub absence_dao: MockAbsenceDao,
    pub absence_service: MockAbsenceService,
    pub extra_hours_service: MockExtraHoursService,
    pub carryover_rebuild_service: MockCarryoverRebuildService,
    pub feature_flag_service: MockFeatureFlagService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub sales_person_service: MockSalesPersonService,
    pub permission_service: MockPermissionService,
}

impl CutoverServiceDeps for CutoverDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type CutoverDao = MockCutoverDao;
    type AbsenceDao = MockAbsenceDao;
    type AbsenceService = MockAbsenceService;
    type ExtraHoursService = MockExtraHoursService;
    type CarryoverRebuildService = MockCarryoverRebuildService;
    type FeatureFlagService = MockFeatureFlagService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl CutoverDependencies {
    pub(crate) fn build_service(
        self,
        transaction_dao: MockTransactionDao,
    ) -> CutoverServiceImpl<CutoverDependencies> {
        CutoverServiceImpl {
            cutover_dao: self.cutover_dao.into(),
            absence_dao: self.absence_dao.into(),
            absence_service: self.absence_service.into(),
            extra_hours_service: self.extra_hours_service.into(),
            carryover_rebuild_service: self.carryover_rebuild_service.into(),
            feature_flag_service: self.feature_flag_service.into(),
            employee_work_details_service: self.employee_work_details_service.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}

fn build_dependencies() -> CutoverDependencies {
    CutoverDependencies {
        cutover_dao: MockCutoverDao::new(),
        absence_dao: MockAbsenceDao::new(),
        absence_service: MockAbsenceService::new(),
        extra_hours_service: MockExtraHoursService::new(),
        carryover_rebuild_service: MockCarryoverRebuildService::new(),
        feature_flag_service: MockFeatureFlagService::new(),
        employee_work_details_service: MockEmployeeWorkDetailsService::new(),
        sales_person_service: MockSalesPersonService::new(),
        permission_service: MockPermissionService::new(),
    }
}

/// Standard MockTransactionDao that returns MockTransaction for use_transaction
/// and accepts both rollback and commit. Wave-1 (dry_run=true) always rolls
/// back; Wave-2 commit-path tests need both calls available.
fn build_default_transaction_dao() -> MockTransactionDao {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao
        .expect_rollback()
        .returning(|_| Ok(()));
    transaction_dao
        .expect_commit()
        .returning(|_| Ok(()));
    transaction_dao
}

/// Set the gate to "no scope" — `find_legacy_scope_set` returns an empty Arc,
/// which short-circuits the gate's per-(sp, year) loop. Used by all Wave-1
/// tests that were originally written before the gate existed; their
/// migration-phase semantics are unchanged, but `run()` now also calls
/// `compute_gate` which writes the diff-report file. Empty scope → no drift
/// rows → gate.passed = true.
fn install_empty_gate_scope(deps: &mut CutoverDependencies) {
    deps.cutover_dao
        .expect_find_legacy_scope_set()
        .returning(|_| Ok(Arc::from([])));
}

/// Permission service that ALWAYS allows. Used for happy-path heuristic tests
/// where we want to focus on cluster behavior, not auth.
fn permission_service_allow_all() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    permission_service
}

trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }
}

// ----------------------------------------------------------------------------
// Fixture helpers
// ----------------------------------------------------------------------------

/// Stable test sales_person_id used across all fixture-based tests.
fn fixture_sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001)
}

/// 8h/Tag, Mo-Fr contract spanning 2020-01-01 .. 2026-12-31 (covers every test
/// scenario including the ISO-53 cross-year case).
fn fixture_8h_mon_fri_contract() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0010),
        sales_person_id: fixture_sp_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2020,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2026,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 30,
        created: Some(datetime!(2020 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// 3-Tage-Vertrag (Mo/Di/Mi), 20h/Woche → ≈ 6.667h pro Tag. Spans
/// 2020-01-01..=2026-12-31. Used for the Plan 08-09 weekly-lump-sum tests.
fn fixture_3day_mo_tu_we_contract() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0020),
        sales_person_id: fixture_sp_id(),
        expected_hours: 20.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2020,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2026,
        workdays_per_week: 3,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: false,
        friday: false,
        saturday: false,
        sunday: false,
        vacation_days: 18,
        created: Some(datetime!(2020 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// 8h/Tag, Mo-Fr contract that starts 2024-01-01. Used for the
/// `contract_not_active` quarantine test where the row predates the contract.
fn fixture_8h_mon_fri_contract_starting_2024() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0011),
        sales_person_id: fixture_sp_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2024,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2026,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 30,
        created: Some(datetime!(2024 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Build a LegacyExtraHoursRow at midnight on the given date, Vacation, given amount.
fn legacy_row(day: time::Date, amount: f32) -> LegacyExtraHoursRow {
    LegacyExtraHoursRow {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sp_id(),
        category: ExtraHoursCategoryEntity::Vacation,
        date_time: time::PrimitiveDateTime::new(day, time::Time::MIDNIGHT),
        amount,
    }
}

fn legacy_row_with_id(id: Uuid, day: time::Date, amount: f32) -> LegacyExtraHoursRow {
    let mut r = legacy_row(day, amount);
    r.id = id;
    r
}

// ----------------------------------------------------------------------------
// Test 1: Cluster-merge happy path (Mon-Fri exact match)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn cluster_merges_consecutive_workdays_with_exact_match() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // 5 consecutive Mon-Fri rows in week 23/2024 with exact 8h match.
    let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
        legacy_row(date!(2024 - 06 - 03), 8.0),
        legacy_row(date!(2024 - 06 - 04), 8.0),
        legacy_row(date!(2024 - 06 - 05), 8.0),
        legacy_row(date!(2024 - 06 - 06), 8.0),
        legacy_row(date!(2024 - 06 - 07), 8.0),
    ]);
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

    // Expect exactly 1 absence_period insert + 5 mapping rows + 0 quarantine.
    deps.absence_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(5)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());

    // Drive `run()` with dry_run=true (HR allowed in mock).
    let result = service
        .run(true, ().auth(), None)
        .await
        .expect("run succeeded");
    assert_eq!(result.total_clusters, 1);
    // dry_run path rolls back → migrated_clusters reports 0.
    assert_eq!(result.migrated_clusters, 0);
    assert_eq!(result.quarantined_rows, 0);
    assert!(
        result.gate_passed,
        "empty gate scope → gate trivially passes"
    );
    assert!(result.dry_run);
    assert!(result.diff_report_path.is_some());

    // Tuple-shape verification: re-drive the helper directly to assert the
    // returned `(MigrationStats, Arc<[Uuid]>)` contract that Plan 04-05
    // depends on. Build a fresh service for the second pass since deps were
    // moved into the first one.
    let mut deps2 = build_dependencies();
    deps2.permission_service = permission_service_allow_all();
    let rows2 = rows.clone();
    deps2
        .cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows2.clone()));
    deps2
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));
    deps2
        .absence_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps2
        .cutover_dao
        .expect_upsert_migration_source()
        .times(5)
        .returning(|_, _| Ok(()));
    let service2 = deps2.build_service(build_default_transaction_dao());

    let run_id = Uuid::new_v4();
    let migrated_at =
        time::PrimitiveDateTime::new(date!(2026 - 05 - 03), time::Time::MIDNIGHT);
    let (stats, migrated_ids, _quarantine_buckets) = service2
        .migrate_legacy_extra_hours_to_clusters(run_id, migrated_at, MockTransaction)
        .await
        .expect("helper succeeded");
    assert_eq!(stats.clusters, 1);
    assert_eq!(stats.quarantined, 0);
    assert_eq!(
        migrated_ids.len(),
        5,
        "tuple-shape lock: all 5 source ids land in migrated_ids"
    );
}

// ----------------------------------------------------------------------------
// Test 2: Quarantine — amount below contract hours
// ----------------------------------------------------------------------------

#[tokio::test]
async fn quarantine_amount_below_contract() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // 1 row Monday with amount = 4h (8h contract).
    let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([legacy_row(date!(2024 - 06 - 03), 4.0)]);
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "amount_below_contract_hours")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.total_clusters, 0);
    assert_eq!(result.quarantined_rows, 1);

    // Tuple-shape lock: empty `migrated_ids` on quarantine-only run.
    let mut deps2 = build_dependencies();
    deps2.permission_service = permission_service_allow_all();
    let rows2 = rows.clone();
    deps2
        .cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows2.clone()));
    deps2
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));
    deps2
        .cutover_dao
        .expect_upsert_quarantine()
        .returning(|_, _| Ok(()));
    let service2 = deps2.build_service(build_default_transaction_dao());
    let (stats, migrated_ids, _quarantine_buckets) = service2
        .migrate_legacy_extra_hours_to_clusters(
            Uuid::new_v4(),
            time::PrimitiveDateTime::new(date!(2026 - 05 - 03), time::Time::MIDNIGHT),
            MockTransaction,
        )
        .await
        .unwrap();
    assert_eq!(stats.clusters, 0);
    assert_eq!(stats.quarantined, 1);
    assert_eq!(migrated_ids.len(), 0);
}

// ----------------------------------------------------------------------------
// Test 3: Quarantine — amount above contract hours
// ----------------------------------------------------------------------------

#[tokio::test]
async fn quarantine_amount_above_contract() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 03), 10.0)]);
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "amount_above_contract_hours")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.total_clusters, 0);
    assert_eq!(result.quarantined_rows, 1);
}

// ----------------------------------------------------------------------------
// Test 4: Quarantine — weekend entry on Mo-Fr-only contract
// ----------------------------------------------------------------------------

#[tokio::test]
async fn quarantine_weekend_entry_workday_contract() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // Saturday 2024-06-08 — 8h Vacation, but Mo-Fr-only contract.
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 08), 8.0)]);
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "contract_hours_zero_for_day")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.quarantined_rows, 1);
}

// ----------------------------------------------------------------------------
// Test 5: Quarantine — contract not active at date
// ----------------------------------------------------------------------------

#[tokio::test]
async fn quarantine_contract_not_active() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // Contract starts 2024-01-01; row is dated 2023-06-03.
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2023 - 06 - 05), 8.0)]);
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| {
            Ok(Arc::from([fixture_8h_mon_fri_contract_starting_2024()]))
        });

    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "contract_not_active_at_date")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.quarantined_rows, 1);
}

// ----------------------------------------------------------------------------
// Test 6: Quarantine — ISO-53 / year-boundary cluster break
//
// Locked decision: Plan 04-02 implements the simpler year-boundary break (no
// explicit `iso_53_week_gap` reason). The cluster splits into two
// AbsencePeriods, NOT a quarantine row.
// ----------------------------------------------------------------------------

#[tokio::test]
async fn quarantine_iso_53_gap() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // 2 rows that would form a continuous Mon-Fri-only cluster across the year
    // boundary if year-equality were not enforced:
    //   - Thu 2020-12-31 (8h Vacation)
    //   - Fri 2021-01-01 (8h Vacation)
    // Both are workdays on the Mo-Fr contract; with year-equality break the
    // cluster splits → 2 absence_period rows, 2 mapping rows, 0 quarantine.
    let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
        legacy_row(date!(2020 - 12 - 31), 8.0),
        legacy_row(date!(2021 - 01 - 01), 8.0),
    ]);
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

    deps.absence_dao
        .expect_create()
        .times(2)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(2)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(
        result.total_clusters, 2,
        "year-boundary break splits cluster into 2 absence periods"
    );
    assert_eq!(result.quarantined_rows, 0);
}

// ----------------------------------------------------------------------------
// Test 7: Idempotent re-run — already-mapped rows are filtered out by SQL
// ----------------------------------------------------------------------------

#[tokio::test]
async fn idempotent_rerun_skips_mapped() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // First run already migrated everything; second run sees no legacy rows.
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(|_| Ok(Arc::from([])));
    // employee_work_details should NOT be called (no distinct_sps to iterate).
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .times(0);
    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.total_clusters, 0);
    assert_eq!(result.quarantined_rows, 0);

    // Tuple-shape preserved on no-op runs.
    let mut deps2 = build_dependencies();
    deps2.permission_service = permission_service_allow_all();
    deps2
        .cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(|_| Ok(Arc::from([])));
    let service2 = deps2.build_service(build_default_transaction_dao());
    let (stats, migrated_ids, _quarantine_buckets) = service2
        .migrate_legacy_extra_hours_to_clusters(
            Uuid::new_v4(),
            time::PrimitiveDateTime::new(date!(2026 - 05 - 03), time::Time::MIDNIGHT),
            MockTransaction,
        )
        .await
        .unwrap();
    assert_eq!(stats.clusters, 0);
    assert_eq!(stats.quarantined, 0);
    assert_eq!(migrated_ids.len(), 0);
}

// ----------------------------------------------------------------------------
// Test 8 + 9: Forbidden tests (HR/cutover_admin permission gate)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn run_forbidden_for_unprivileged_user() {
    // dry_run=true requires HR; mock returns Forbidden for HR.
    let mut deps = build_dependencies();
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.permission_service = permission_service;

    // CRUCIAL: NO DAO/service call must happen if permission check fails.
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .times(0);
    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao.expect_upsert_quarantine().times(0);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .times(0);

    // No tx interactions: permission must short-circuit BEFORE use_transaction.
    let mut tx_dao = MockTransactionDao::new();
    tx_dao.expect_use_transaction().times(0);
    tx_dao.expect_rollback().times(0);
    tx_dao.expect_commit().times(0);

    let service = deps.build_service(tx_dao);
    let result = service.run(true, ().auth(), None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}

#[tokio::test]
async fn run_forbidden_for_hr_only_when_committing() {
    // dry_run=false requires cutover_admin; mock returns Forbidden for it.
    let mut deps = build_dependencies();
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.permission_service = permission_service;

    // No DAO call may happen; no Tx may open.
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .times(0);
    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao.expect_upsert_quarantine().times(0);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .times(0);

    let mut tx_dao = MockTransactionDao::new();
    tx_dao.expect_use_transaction().times(0);
    tx_dao.expect_rollback().times(0);
    tx_dao.expect_commit().times(0);

    let service = deps.build_service(tx_dao);
    let result = service.run(false, ().auth(), None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}

// ----------------------------------------------------------------------------
// Wave-2 gate-tolerance boundary tests (Plan 04-05).
//
// Both drive `run(dry_run=true, ...)` end-to-end so the full
// migration → gate → branch → rollback path is exercised, including the
// diff-report file write (D-Phase4-06). The migration phase contributes 0
// rows (empty find_legacy_extra_hours_not_yet_migrated) so the assertions
// focus exclusively on gate behavior.
// ----------------------------------------------------------------------------

/// Build a SalesPerson fixture with the given id + name. Local helper since the
/// shared `fixture_sales_person` lives in `reporting_phase2_fixtures` and we
/// want stable values for these tests.
fn cutover_test_sales_person() -> SalesPerson {
    SalesPerson {
        id: fixture_sp_id(),
        name: Arc::from("Test Cutover Person"),
        background_color: Arc::from("#000000"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Common gate-test setup: empty migration phase + 1-tuple scope set
/// (`(fixture_sp_id, 2024)`) + sales-person lookup. Caller provides the
/// derived-hours map and the legacy-Vacation sum; SickLeave + UnpaidLeave
/// always 0 to isolate the Vacation drift.
fn arrange_gate_test(
    deps: &mut CutoverDependencies,
    legacy_vacation_sum: f32,
    derived_hours: BTreeMap<time::Date, ResolvedAbsence>,
) {
    deps.permission_service = permission_service_allow_all();

    // Migration phase: no legacy rows to migrate.
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(|_| Ok(Arc::from([])));

    // Gate phase scope: exactly one (sp, year) tuple.
    deps.cutover_dao
        .expect_find_legacy_scope_set()
        .returning(|_| Ok(Arc::from([(fixture_sp_id(), 2024u32)])));

    // sales_person_service.get for DriftRow.sales_person_name.
    deps.sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(cutover_test_sales_person()));

    // derive_hours_for_range stub (1 call per (sp, year) tuple).
    let derived_clone = derived_hours.clone();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(move |_, _, _, _, _| Ok(derived_clone.clone()));

    // sum_legacy_extra_hours stub: Vacation = caller's value; others = 0.0.
    deps.cutover_dao
        .expect_sum_legacy_extra_hours()
        .withf(|_, cat, _, _| matches!(cat, ExtraHoursCategoryEntity::Vacation))
        .returning(move |_, _, _, _| Ok(legacy_vacation_sum));
    deps.cutover_dao
        .expect_sum_legacy_extra_hours()
        .withf(|_, cat, _, _| {
            matches!(
                cat,
                ExtraHoursCategoryEntity::SickLeave | ExtraHoursCategoryEntity::UnpaidLeave
            )
        })
        .returning(|_, _, _, _| Ok(0.0));
}

#[tokio::test]
async fn gate_tolerance_pass_below_threshold() {
    let mut deps = build_dependencies();

    // legacy_sum = 100.000, derived_sum = 100.005 → drift = 0.005 < 0.01.
    let mut derived: BTreeMap<time::Date, ResolvedAbsence> = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 100.005,
        },
    );
    arrange_gate_test(&mut deps, 100.000, derived);

    // Below-threshold drift → count_quarantine_for_drift_row MUST NOT be
    // called for any category.
    deps.cutover_dao
        .expect_count_quarantine_for_drift_row()
        .times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service
        .run(true, ().auth(), None)
        .await
        .expect("run succeeded");

    assert!(
        result.gate_passed,
        "drift 0.005h is strictly below the 0.01h threshold"
    );
    assert_eq!(result.gate_drift_rows, 0);
    assert!(result.dry_run);
    let report_path = result
        .diff_report_path
        .expect("diff report path is always returned");

    let p = std::path::Path::new(report_path.as_ref());
    assert!(p.exists(), "diff report file should exist at {:?}", p);
    let body = std::fs::read_to_string(p).expect("diff report readable");
    assert!(
        body.contains("\"passed\": true"),
        "diff report JSON should record passed=true: {}",
        body
    );
    assert!(
        body.contains("\"total_drift_rows\": 0"),
        "diff report JSON should record 0 drift rows: {}",
        body
    );

    let _ = std::fs::remove_file(p);
}

#[tokio::test]
async fn gate_tolerance_fail_above_threshold() {
    let mut deps = build_dependencies();

    // legacy_sum = 100.000, derived_sum = 100.020 → drift = 0.020 > 0.01.
    let mut derived: BTreeMap<time::Date, ResolvedAbsence> = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 100.020,
        },
    );
    arrange_gate_test(&mut deps, 100.000, derived);

    // Above-threshold drift → count_quarantine_for_drift_row MUST be called
    // exactly once for the Vacation row.
    deps.cutover_dao
        .expect_count_quarantine_for_drift_row()
        .withf(|_, cat, year, _, _| {
            matches!(cat, ExtraHoursCategoryEntity::Vacation) && *year == 2024
        })
        .times(1)
        .returning(|_, _, _, _, _| {
            Ok((
                2,
                Arc::from([
                    Arc::<str>::from("amount_below_contract_hours"),
                    Arc::<str>::from("contract_hours_zero_for_day"),
                ]),
            ))
        });

    let service = deps.build_service(build_default_transaction_dao());
    let result = service
        .run(true, ().auth(), None)
        .await
        .expect("run succeeded");

    assert!(
        !result.gate_passed,
        "drift 0.02h is strictly above the 0.01h threshold"
    );
    assert_eq!(result.gate_drift_rows, 1);
    assert!(result.dry_run);
    let report_path = result
        .diff_report_path
        .expect("diff report path is always returned");

    let p = std::path::Path::new(report_path.as_ref());
    assert!(p.exists(), "diff report file should exist at {:?}", p);
    let body = std::fs::read_to_string(p).expect("diff report readable");
    assert!(
        body.contains("\"passed\": false"),
        "diff report JSON should record passed=false: {}",
        body
    );
    assert!(
        body.contains("\"total_drift_rows\": 1"),
        "diff report JSON should record 1 drift row: {}",
        body
    );
    assert!(
        body.contains("\"sales_person_name\": \"Test Cutover Person\""),
        "diff report JSON should embed the sales-person name: {}",
        body
    );
    assert!(
        body.contains("\"category\": \"Vacation\""),
        "diff report JSON should record the category: {}",
        body
    );

    let _ = std::fs::remove_file(p);
}

// ----------------------------------------------------------------------------
// Plan 08-08 — defense-in-depth check that every QuarantineReason variant
// returns a non-empty human_text() AND non-empty suggested_action(). Catches
// the "added a new variant but forgot to extend the match in either method"
// regression.
// ----------------------------------------------------------------------------

#[test]
fn quarantine_reason_text_and_action_non_empty_per_variant() {
    use service::cutover::QuarantineReason;

    let all = [
        QuarantineReason::AmountBelowContractHours,
        QuarantineReason::AmountAboveContractHours,
        QuarantineReason::ContractHoursZeroForDay,
        QuarantineReason::ContractNotActiveAtDate,
        QuarantineReason::Iso53WeekGap,
    ];

    for variant in all {
        let text = variant.human_text();
        let action = variant.suggested_action();
        assert!(
            !text.trim().is_empty(),
            "human_text() must be non-empty for variant {:?}",
            variant
        );
        assert!(
            !action.trim().is_empty(),
            "suggested_action() must be non-empty for variant {:?}",
            variant
        );
        // Sanity: persisted code stays non-empty too.
        assert!(
            !variant.as_persisted_str().trim().is_empty(),
            "as_persisted_str() must be non-empty for variant {:?}",
            variant
        );
    }
}

// ----------------------------------------------------------------------------
// Plan 08-09 — Weekly-Lump-Sum Heuristic Tests
//
// Live scenario: 3-day contract (Mon/Tue/Wed, 20h/week). User books 20h
// Vacation as a single extra_hours row on any weekday of the target week.
// Heuristic must map to absence_period {Monday, Sunday} of that ISO-week
// without quarantining the row, so the gate passes (drift = 0).
// ----------------------------------------------------------------------------

/// Helper: stub the migration-phase plumbing for a 3-day-contract test.
/// Caller provides the legacy rows; the helper wires up:
/// - cutover_dao.find_legacy_extra_hours_not_yet_migrated → rows
/// - employee_work_details_service.find_by_sales_person_id → 3-day contract
/// - install_empty_gate_scope (gate trivially passes — we focus on migration)
fn arrange_lump_sum_migration(
    deps: &mut CutoverDependencies,
    rows: Arc<[LegacyExtraHoursRow]>,
) {
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(deps);

    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));
}

#[tokio::test]
async fn test_weekly_lump_sum_at_workday_succeeds() {
    // 3-day contract, 20h Vacation booked at Monday (= a contract workday).
    // ISO-week 23/2024 → 2024-06-03 (Mon) .. 2024-06-09 (Sun).
    let mut deps = build_dependencies();
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 03), 20.0)]);
    arrange_lump_sum_migration(&mut deps, rows);

    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2024 - 06 - 03)
                && entity.to_date == date!(2024 - 06 - 09)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service
        .run(true, ().auth(), None)
        .await
        .expect("run succeeded");
    assert_eq!(result.total_clusters, 1, "1 lump-sum cluster");
    assert_eq!(result.quarantined_rows, 0, "no quarantine for lump-sum");
    assert!(result.gate_passed, "empty gate scope → gate passes");
}

#[tokio::test]
async fn test_weekly_lump_sum_at_non_workday_succeeds() {
    // **LIVE-REPRODUCE — Max Schmidt scenario from the User-UAT.**
    // 3-day contract (Mon/Tue/Wed, 20h/week). 20h Vacation booked at
    // 2026-05-08 (Friday — a NON-workday). ISO-week 19/2026: Mon=2026-05-04,
    // Sun=2026-05-10. Heuristic must accept this even though Friday is not a
    // workday — the convention is "user picked any day of the week".
    let mut deps = build_dependencies();
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2026 - 05 - 08), 20.0)]);
    arrange_lump_sum_migration(&mut deps, rows);

    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2026 - 05 - 04)
                && entity.to_date == date!(2026 - 05 - 10)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service
        .run(true, ().auth(), None)
        .await
        .expect("run succeeded");
    assert_eq!(result.total_clusters, 1);
    assert_eq!(
        result.quarantined_rows, 0,
        "non-workday lump-sum must NOT quarantine (Plan 08-09 fix)"
    );
}

#[tokio::test]
async fn test_weekly_lump_sum_at_weekend_succeeds() {
    // 3-day contract, 20h Vacation booked at Sunday 2024-06-09 (KW 23/2024,
    // Sun=2024-06-09, Mon=2024-06-03). Sunday is the last day of the
    // ISO-week — must still resolve to {Mo, So} of the same week.
    let mut deps = build_dependencies();
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 09), 20.0)]);
    arrange_lump_sum_migration(&mut deps, rows);

    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2024 - 06 - 03)
                && entity.to_date == date!(2024 - 06 - 09)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.total_clusters, 1);
    assert_eq!(result.quarantined_rows, 0);
}

#[tokio::test]
async fn test_strict_match_per_day_still_works_after_pivot() {
    // Backwards-compat sanity: a single hours_per_day-match Vacation row at
    // a contract weekday must still go through the strict-match path (not the
    // lump-sum path). hours_per_day for the 3-day-20h contract = 20/3 ≈ 6.667.
    // 20h would trigger lump-sum, but 6.667h is the strict-match amount and
    // does NOT match the weekly target (= 20h), so the heuristic returns None
    // and the row goes via strict-match → 1-day cluster (Mo-Mo).
    let mut deps = build_dependencies();
    // A hours_per_day-exact Vacation row at Monday — strict-match success.
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 03), 20.0 / 3.0)]);
    arrange_lump_sum_migration(&mut deps, rows);

    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            // 1-day cluster: from = to = Monday (NOT Sunday — strict-match
            // path does NOT extend to the full week).
            entity.from_date == date!(2024 - 06 - 03)
                && entity.to_date == date!(2024 - 06 - 03)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(
        result.total_clusters, 1,
        "strict-match still produces a 1-day cluster for the per-day amount"
    );
    assert_eq!(result.quarantined_rows, 0);
}

#[tokio::test]
async fn test_two_rows_same_week_blocks_lump_sum() {
    // Single-row-per-week violation: two Vacation rows of the same (sp, cat)
    // in the same ISO-week → heuristic must NOT fire for either; both go via
    // strict-match. Row 1 (Mon, 20h) fails strict-match (AmountAbove);
    // Row 2 (Tue, 6.667h) passes strict-match → 1-day cluster.
    let mut deps = build_dependencies();
    let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
        legacy_row(date!(2024 - 06 - 03), 20.0),     // Mon
        legacy_row(date!(2024 - 06 - 04), 20.0 / 3.0), // Tue
    ]);
    arrange_lump_sum_migration(&mut deps, rows);

    // Mon-Row hits AmountAbove (20 > 6.667), Tue-Row builds a 1-day cluster.
    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            // Tue-only cluster.
            entity.from_date == date!(2024 - 06 - 04)
                && entity.to_date == date!(2024 - 06 - 04)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "amount_above_contract_hours")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(
        result.total_clusters, 1,
        "Tue-row builds 1-day cluster; Mon-row quarantines"
    );
    assert_eq!(result.quarantined_rows, 1, "Mon-row quarantines");
}

#[tokio::test]
async fn test_partial_week_amount_falls_to_strict_match() {
    // 13.33h ≈ 2 × hours_per_day = 2 contract-days worth — NOT the weekly
    // total (20h). Heuristic returns None → strict-match → AmountAbove
    // quarantine (13.33 > 6.667).
    let mut deps = build_dependencies();
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 03), 40.0 / 3.0)]); // ≈ 13.333
    arrange_lump_sum_migration(&mut deps, rows);

    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "amount_above_contract_hours")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(result.total_clusters, 0);
    assert_eq!(result.quarantined_rows, 1);
}

#[tokio::test]
async fn test_weekly_lump_sum_with_dynamic_contract_change_mid_week() {
    // Contract change mid-week: 3-day contract (Mon/Tue/Wed, 20h/week) until
    // the end of week 23/2024, then a 4-day contract (Mon/Tue/Wed/Thu,
    // 32h/week → 8h/day) starting week 24/2024.
    //
    // For week 24/2024 (Mon=2024-06-10..Sun=2024-06-16), all 7 days are
    // covered by the 4-day contract. Target sum = 4 × 8 = 32h. A 32h Vacation
    // row anywhere in that week matches lump-sum → maps to {Mo, So}.
    //
    // This covers the "contract_at(weekday)" semantic: the heuristic must
    // call the per-weekday lookup, not assume one contract for the whole
    // week. (Even though the 4-day contract covers all 7 days here, the test
    // exercises the lookup helper.)
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2024 - 06 - 14), 32.0)]); // Friday in week 24
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));

    // Two contracts: 3-day until 2024-06-09, then 4-day from 2024-06-10.
    let three_day = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0030),
        sales_person_id: fixture_sp_id(),
        expected_hours: 20.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2020,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 23, // ends Sun 2024-06-09
        to_year: 2024,
        workdays_per_week: 3,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: false,
        friday: false,
        saturday: false,
        sunday: false,
        vacation_days: 18,
        created: Some(datetime!(2020 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };
    let four_day = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0031),
        sales_person_id: fixture_sp_id(),
        expected_hours: 32.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 24, // starts Mon 2024-06-10
        from_year: 2024,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2026,
        workdays_per_week: 4,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: false,
        saturday: false,
        sunday: false,
        vacation_days: 24,
        created: Some(datetime!(2024 - 06 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([three_day.clone(), four_day.clone()])));

    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2024 - 06 - 10)
                && entity.to_date == date!(2024 - 06 - 16)
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    deps.cutover_dao
        .expect_upsert_migration_source()
        .times(1)
        .returning(|_, _| Ok(()));
    deps.cutover_dao.expect_upsert_quarantine().times(0);

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();
    assert_eq!(
        result.total_clusters, 1,
        "lump-sum maps to {{Mo, So}} of week 24/2024 under the 4-day contract"
    );
    assert_eq!(result.quarantined_rows, 0);
}

// ----------------------------------------------------------------------------
// Phase 8.1 Plan 02 — convert_quarantine_entry tests.
//
// Four mockall unit tests covering the Single-Convert backend method:
//   1. happy path: 3-day Mo/Tu/We contract + 20h Vacation row → absence_period
//      {Mo, So} created, soft_delete called, upsert_migration_source called,
//      refreshed_drift_report Some(_).
//   2. heuristic mismatch (~13.33h amount) → ValidationError, NO DAO writes.
//   3. unprivileged caller → Forbidden, NO Tx + NO DAO writes.
//   4. extra_hours_id absent (already migrated / unknown) → EntityNotFoundGeneric,
//      NO downstream calls.
// ----------------------------------------------------------------------------

mod convert_quarantine_entry_tests {
    use super::*;
    use mockall::Sequence;

    /// Stable id used for the row under test (so tests can match on it).
    fn target_extra_hours_id() -> Uuid {
        Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BEEF_0001)
    }

    fn target_row(day: time::Date, amount: f32) -> LegacyExtraHoursRow {
        legacy_row_with_id(target_extra_hours_id(), day, amount)
    }

    /// Build a permission_service that allows ONLY `cutover_admin` (mirrors
    /// the production privilege gate for the Single-Convert endpoint).
    fn permission_service_allow_cutover_admin() -> MockPermissionService {
        let mut p = MockPermissionService::new();
        p.expect_check_permission()
            .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
            .returning(|_, _| Ok(()));
        p
    }

    #[tokio::test]
    async fn test_convert_single_quarantine_entry_succeeds() {
        // Live-reproduce: 3-day contract (Mon/Tue/Wed, 20h/week). 20h Vacation
        // booked on Friday 2024-06-07 (a NON-workday). Heuristic must accept
        // and the convert flow must write the absence_period {Mo=2024-06-03,
        // So=2024-06-09} of week 23/2024.
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = target_row(date!(2024 - 06 - 07), 20.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row.clone()]);
        let row_arc_clone1 = row_arc.clone();
        // The convert flow calls `find_legacy_extra_hours_not_yet_migrated`
        // twice in this test path: once for the actual convert (returns the
        // row), once during the inline replay
        // (`migrate_legacy_extra_hours_to_clusters`) where in production the
        // row would be soft-deleted — we model that by returning an empty
        // slice on the second call.
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(row_arc_clone1.clone()));
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));

        // The convert path inserts exactly one `absence_period` covering the
        // full ISO-week (Mo, So). The inline replay sees an empty legacy set
        // and so makes no further `absence_dao.create` calls.
        deps.absence_dao
            .expect_create()
            .withf(|entity, process, _| {
                entity.from_date == date!(2024 - 06 - 03)
                    && entity.to_date == date!(2024 - 06 - 09)
                    && entity.sales_person_id == fixture_sp_id()
                    && process == "phase-4-cutover-migration"
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        deps.cutover_dao
            .expect_upsert_migration_source()
            .withf(|src, _| src.extra_hours_id == target_extra_hours_id())
            .times(1)
            .returning(|_, _| Ok(()));

        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .withf(|ids, process, _, _| {
                ids.len() == 1
                    && ids[0] == target_extra_hours_id()
                    && process == "phase-4-cutover-migration"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // The replay's bucket-map is empty (no quarantined rows on the second
        // pass), so no upsert_quarantine call is expected.
        deps.cutover_dao.expect_upsert_quarantine().times(0);

        // Inline gate-diagnostic: `find_legacy_scope_set` returns empty →
        // no per-(sp, year) iteration → drift list is empty, gate trivially
        // passes. `compute_gate_diagnostic` does NOT persist the audit JSON.
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let service = deps.build_service(build_default_transaction_dao());
        let outcome = service
            .convert_quarantine_entry(target_extra_hours_id(), ().auth(), None)
            .await
            .expect("single-convert succeeds");

        assert_eq!(outcome.deleted_extra_hours_id, target_extra_hours_id());
        assert_eq!(outcome.absence_period.from_date, date!(2024 - 06 - 03));
        assert_eq!(outcome.absence_period.to_date, date!(2024 - 06 - 09));
        assert!(
            outcome.refreshed_drift_report.is_some(),
            "inline refreshed_drift_report must be Some(_) (D-08, RESEARCH P-03 option a)"
        );
        let report = outcome.refreshed_drift_report.unwrap();
        assert!(report.passed, "empty scope set → gate trivially passes");
        assert_eq!(report.total_drift_rows, 0);
    }

    #[tokio::test]
    async fn test_convert_single_no_lump_sum_match_returns_validation_error() {
        // 13.33h on a Monday for the 3-day-20h contract → strictly NOT a
        // weekly lump-sum (target sum is 20h). Heuristic returns None →
        // ValidationError; NO DAO writes; Tx implicitly rolls back via Drop.
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = target_row(date!(2024 - 06 - 03), 40.0 / 3.0); // ≈ 13.333
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(row_arc.clone()));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));

        // Hard guard: NO write/upsert/delete may happen on the heuristic-
        // mismatch path. These `.times(0)` expectations make the test fail
        // immediately if the implementation ever falls through.
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);
        // `find_legacy_scope_set` belongs to the inline gate — also unreachable.
        deps.cutover_dao.expect_find_legacy_scope_set().times(0);

        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        // The Tx is opened but never committed on this path — implicit Drop
        // rollback; we accept either rollback or no commit explicitly.
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        let service = deps.build_service(tx_dao);
        let result = service
            .convert_quarantine_entry(target_extra_hours_id(), ().auth(), None)
            .await;
        assert!(
            matches!(result, Err(ServiceError::ValidationError(_))),
            "heuristic mismatch must return ValidationError; got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_convert_quarantine_entry_requires_cutover_admin() {
        // Caller does NOT hold `cutover_admin` → permission_service returns
        // Forbidden. The implementation must short-circuit before opening a
        // Tx; NO DAO/service call may happen.
        let mut deps = build_dependencies();
        let mut p = MockPermissionService::new();
        p.expect_check_permission()
            .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
            .returning(|_, _| Err(ServiceError::Forbidden));
        deps.permission_service = p;

        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(0);
        deps.cutover_dao.expect_find_legacy_scope_set().times(0);
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .times(0);

        // No Tx may open if permission fails first.
        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().times(0);
        tx_dao.expect_rollback().times(0);
        tx_dao.expect_commit().times(0);

        let service = deps.build_service(tx_dao);
        let result = service
            .convert_quarantine_entry(target_extra_hours_id(), ().auth(), None)
            .await;
        assert!(
            matches!(result, Err(ServiceError::Forbidden)),
            "non-cutover_admin caller must be rejected with Forbidden; got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_convert_quarantine_entry_not_found_returns_not_found() {
        // The row is absent from `find_legacy_extra_hours_not_yet_migrated`
        // (e.g. already soft-deleted by an earlier convert / cutover). The
        // implementation must return EntityNotFoundGeneric without making
        // any further DAO calls — this is the idempotent-replay path
        // (RESEARCH P-02).
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(|_| Ok(Arc::from([])));

        // No work-details lookup should happen if the row isn't found.
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .times(0);

        // No mutating DAO call may happen.
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);
        deps.cutover_dao.expect_find_legacy_scope_set().times(0);

        let service = deps.build_service(build_default_transaction_dao());
        let result = service
            .convert_quarantine_entry(target_extra_hours_id(), ().auth(), None)
            .await;
        assert!(
            matches!(result, Err(ServiceError::EntityNotFoundGeneric(_))),
            "missing extra_hours row must return EntityNotFoundGeneric; got {:?}",
            result
        );
    }
}

// ----------------------------------------------------------------------------
// Plan 8.1 Plan 03 — bulk_convert_quarantine_rows tests
// ----------------------------------------------------------------------------

mod bulk_convert_quarantine_rows_tests {
    use super::*;
    use mockall::Sequence;

    fn permission_service_allow_cutover_admin() -> MockPermissionService {
        let mut p = MockPermissionService::new();
        p.expect_check_permission()
            .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
            .returning(|_, _| Ok(()));
        p
    }

    /// Stable ids per row, so withf-predicates can match deterministically.
    fn row_id_for(idx: u32) -> Uuid {
        Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_DEAD_0000 | (idx as u128))
    }

    /// Build a 20h-Vacation row in week 23/2024 on `day`.
    fn lump_row(idx: u32, day: time::Date, amount: f32) -> LegacyExtraHoursRow {
        legacy_row_with_id(row_id_for(idx), day, amount)
    }

    #[tokio::test]
    async fn test_bulk_convert_succeeds_atomic_for_three_matching_rows() {
        // 3 quarantined Vacation rows for `fixture_sp_id()` in 3 different
        // ISO-weeks of 2024 (3-day Mo/Tu/We contract, 20h/week). Each is a
        // valid weekly-lump-sum on its own week. All three must be converted
        // in a single Tx and share one `cutover_run_id` (RESEARCH Q3).
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        // 3 separate ISO-weeks → no inter-row interference for the heuristic.
        let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
            lump_row(1, date!(2024 - 06 - 07), 20.0), // week 23 (Mo 03 .. So 09)
            lump_row(2, date!(2024 - 06 - 14), 20.0), // week 24
            lump_row(3, date!(2024 - 06 - 21), 20.0), // week 25
        ]);
        let rows_for_first = rows.clone();
        // First call (filter / heuristic) — second call (replay inside the
        // inline drift-report compute) returns empty (rows soft-deleted).
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(rows_for_first.clone()));
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));

        // 3 absence_period inserts.
        deps.absence_dao
            .expect_create()
            .withf(|entity, process, _| {
                entity.sales_person_id == fixture_sp_id()
                    && process == "phase-4-cutover-migration"
                    && entity.from_date.weekday() == time::Weekday::Monday
                    && entity.to_date.weekday() == time::Weekday::Sunday
            })
            .times(3)
            .returning(|_, _, _| Ok(()));

        // 3 migration-source upserts, all sharing the SAME cutover_run_id
        // (RESEARCH Q3). Capture the first one's run_id and assert the rest
        // match it via a Mutex-shared Option.
        let captured_run_id = std::sync::Arc::new(std::sync::Mutex::new(None::<Uuid>));
        let captured_for_pred = captured_run_id.clone();
        deps.cutover_dao
            .expect_upsert_migration_source()
            .withf(move |row, _| {
                let mut g = captured_for_pred.lock().unwrap();
                match *g {
                    None => {
                        *g = Some(row.cutover_run_id);
                        true
                    }
                    Some(prev) => prev == row.cutover_run_id,
                }
            })
            .times(3)
            .returning(|_, _| Ok(()));

        // One bulk soft-delete with all 3 ids.
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .withf(|ids, process, _, _| {
                ids.len() == 3
                    && process == "phase-4-cutover-migration"
                    && ids.iter().any(|id| *id == row_id_for(1))
                    && ids.iter().any(|id| *id == row_id_for(2))
                    && ids.iter().any(|id| *id == row_id_for(3))
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // Inline drift-report replay sees empty rows → no quarantine, no
        // migration-source upserts during the replay. Empty scope-set → gate
        // trivially passes.
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let service = deps.build_service(build_default_transaction_dao());
        let outcome = service
            .bulk_convert_quarantine_rows(
                fixture_sp_id(),
                AbsenceCategory::Vacation,
                2024,
                None,
                ().auth(),
                None,
            )
            .await
            .expect("bulk-convert succeeds atomically");

        assert_eq!(outcome.converted_absence_periods.len(), 3);
        assert_eq!(outcome.deleted_extra_hours_ids.len(), 3);
        assert!(outcome.errors.is_empty(), "strict-atomic: errors must be empty on 200");
        assert!(
            outcome.refreshed_drift_report.is_some(),
            "inline refreshed_drift_report must be Some(_) (D-08)"
        );
        // The captured run_id must have been set (= all three calls saw it).
        let captured = captured_run_id.lock().unwrap();
        assert!(
            captured.is_some(),
            "all three migration-source upserts must share one synthetic_run_id (RESEARCH Q3)"
        );
    }

    #[tokio::test]
    async fn test_bulk_convert_strict_atomic_returns_validation_error_on_heuristic_mismatch() {
        // 3 rows in 3 different weeks; row #2 is fractional 13.33h (≠ 20h
        // weekly target) and so the heuristic returns None. Strict-atomic
        // (RESEARCH P-10) means: NO DAO writes happen, the entire batch
        // returns ValidationError, and the implicit Tx-Drop rollback leaves
        // the DB untouched.
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
            lump_row(1, date!(2024 - 06 - 07), 20.0),    // week 23 — valid lump
            lump_row(2, date!(2024 - 06 - 14), 13.33),  // week 24 — fractional, mismatch
            lump_row(3, date!(2024 - 06 - 21), 20.0),    // week 25 — valid lump
        ]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(rows.clone()));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));

        // Hard guards — strict-atomic must NOT touch any mutating DAO call.
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);
        deps.cutover_dao.expect_find_legacy_scope_set().times(0);

        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        // No commit — we error out before the commit point.
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        let service = deps.build_service(tx_dao);
        let result = service
            .bulk_convert_quarantine_rows(
                fixture_sp_id(),
                AbsenceCategory::Vacation,
                2024,
                None,
                ().auth(),
                None,
            )
            .await;

        assert!(
            matches!(result, Err(ServiceError::ValidationError(_))),
            "strict-atomic mismatch must return ValidationError; got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_bulk_convert_with_explicit_ids_narrows_target_set() {
        // 5 rows match the (sp, Vacation, 2024) triple but only #1 + #3 are
        // listed in `explicit_ids`. The implementation must convert exactly
        // those 2 rows; rows #2/#4/#5 stay untouched.
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
            lump_row(1, date!(2024 - 06 - 07), 20.0), // week 23 — selected
            lump_row(2, date!(2024 - 06 - 14), 20.0), // week 24 — NOT selected
            lump_row(3, date!(2024 - 06 - 21), 20.0), // week 25 — selected
            lump_row(4, date!(2024 - 06 - 28), 20.0), // week 26 — NOT selected
            lump_row(5, date!(2024 - 07 - 05), 20.0), // week 27 — NOT selected
        ]);
        let rows_for_first = rows.clone();
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(rows_for_first.clone()));
        // Replay (inline drift-report) — model the "all soft-deleted" state
        // with an empty slice so we can isolate-test the explicit-subset
        // narrowing of the Bulk-Convert phase itself. Mirrors the convention
        // established in Plan 02's `test_convert_single_quarantine_entry_succeeds`.
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_3day_mo_tu_we_contract()])));

        // Exactly 2 absence_period inserts for the explicit subset (#1, #3).
        deps.absence_dao
            .expect_create()
            .times(2)
            .returning(|_, _, _| Ok(()));
        // Exactly 2 migration-source upserts for explicit ids #1 and #3.
        deps.cutover_dao
            .expect_upsert_migration_source()
            .withf(|src, _| {
                src.extra_hours_id == row_id_for(1) || src.extra_hours_id == row_id_for(3)
            })
            .times(2)
            .returning(|_, _| Ok(()));

        // Bulk soft-delete must contain exactly #1 + #3 (and only those).
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .withf(|ids, _, _, _| {
                ids.len() == 2
                    && ids.iter().any(|id| *id == row_id_for(1))
                    && ids.iter().any(|id| *id == row_id_for(3))
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // Replay sees empty rows (model: post-bulk-convert state with all
        // remaining rows considered out-of-scope for this isolation test).
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let service = deps.build_service(build_default_transaction_dao());
        let explicit: Arc<[Uuid]> = Arc::from([row_id_for(1), row_id_for(3)]);
        let outcome = service
            .bulk_convert_quarantine_rows(
                fixture_sp_id(),
                AbsenceCategory::Vacation,
                2024,
                Some(explicit),
                ().auth(),
                None,
            )
            .await
            .expect("subset bulk-convert succeeds");

        assert_eq!(outcome.converted_absence_periods.len(), 2);
        assert_eq!(outcome.deleted_extra_hours_ids.len(), 2);
        assert!(outcome.deleted_extra_hours_ids.contains(&row_id_for(1)));
        assert!(outcome.deleted_extra_hours_ids.contains(&row_id_for(3)));
    }

    #[tokio::test]
    async fn test_bulk_convert_empty_match_set_returns_not_found() {
        // No row matches the requested triple → EntityNotFoundGeneric; no DAO
        // writes; Tx implicitly rolled back via Drop.
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        // The dao returns one row for a different sales_person — no match.
        let other_sp = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_00FF);
        let mut other_row = legacy_row(date!(2024 - 06 - 07), 20.0);
        other_row.sales_person_id = other_sp;
        let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([other_row]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(rows.clone()));

        // No work-details lookup (we error out before that).
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .times(0);

        // No mutating DAO calls allowed.
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);
        deps.cutover_dao.expect_find_legacy_scope_set().times(0);

        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        let service = deps.build_service(tx_dao);
        let result = service
            .bulk_convert_quarantine_rows(
                fixture_sp_id(),
                AbsenceCategory::Vacation,
                2024,
                None,
                ().auth(),
                None,
            )
            .await;

        assert!(
            matches!(result, Err(ServiceError::EntityNotFoundGeneric(_))),
            "empty match-set must return EntityNotFoundGeneric; got {:?}",
            result
        );
    }
}

// Suppress unused-import warning for `legacy_row_with_id` if no test uses it
// directly in this module (kept for future Wave-1 idempotence-with-mapped
// scenarios that Plan 04-05 may extend).
#[allow(dead_code)]
fn _suppress_unused() -> LegacyExtraHoursRow {
    legacy_row_with_id(Uuid::nil(), date!(2024 - 06 - 03), 8.0)
}
