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

use std::sync::Arc;

use mockall::predicate::{always, eq};
use time::macros::{date, datetime};
use uuid::Uuid;

use dao::absence::MockAbsenceDao;
use dao::cutover::{LegacyExtraHoursRow, MockCutoverDao};
use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::{MockTransaction, MockTransactionDao};
use service::absence::MockAbsenceService;
use service::carryover_rebuild::MockCarryoverRebuildService;
use service::cutover::{CutoverService, CUTOVER_ADMIN_PRIVILEGE};
use service::employee_work_details::{
    EmployeeWorkDetails, MockEmployeeWorkDetailsService,
};
use service::extra_hours::MockExtraHoursService;
use service::feature_flag::MockFeatureFlagService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::MockSalesPersonService;
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
/// and accepts rollback. Wave-1 always rolls back at the end of run().
fn build_default_transaction_dao() -> MockTransactionDao {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao
        .expect_rollback()
        .returning(|_| Ok(()));
    // commit is never called in Wave-1 run()
    transaction_dao
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
    assert_eq!(result.migrated_clusters, 1);
    assert_eq!(result.quarantined_rows, 0);
    assert!(!result.gate_passed, "Wave-1 leaves gate_passed=false");
    assert!(result.dry_run);

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
    let (stats, migrated_ids) = service2
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
    let (stats, migrated_ids) = service2
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
    let (stats, migrated_ids) = service2
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
// Wave-2 placeholders — implemented in Plan 04-05.
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_pass_below_threshold() {
    unimplemented!("wave-2");
}

#[tokio::test]
#[ignore = "wave-2-implements-gate-tolerance"]
async fn gate_tolerance_fail_above_threshold() {
    unimplemented!("wave-2");
}

// Suppress unused-import warning for `legacy_row_with_id` if no test uses it
// directly in this module (kept for future Wave-1 idempotence-with-mapped
// scenarios that Plan 04-05 may extend).
#[allow(dead_code)]
fn _suppress_unused() -> LegacyExtraHoursRow {
    legacy_row_with_id(Uuid::nil(), date!(2024 - 06 - 03), 8.0)
}
