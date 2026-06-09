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
            .convert_quarantine_entry(target_extra_hours_id(), None, None, ().auth(), None)
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
            .convert_quarantine_entry(target_extra_hours_id(), None, None, ().auth(), None)
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
            .convert_quarantine_entry(target_extra_hours_id(), None, None, ().auth(), None)
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
            .convert_quarantine_entry(target_extra_hours_id(), None, None, ().auth(), None)
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
                None,
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

// ----------------------------------------------------------------------------
// Phase 8.1 Plan 10 — Diagnose `08-HUMAN-UAT.md` gap-1 (a)
//
// Reproduce the contract-data edge cases (Lila / Anina / Karin) where the
// Plan-08-09 weekly-lump-sum heuristic does NOT match the expected pattern.
//
// Walk three hypotheses with synthetic fixtures driving `service.run(true,
// ...)` end-to-end (the heuristic helpers `detect_weekly_lump_sum`,
// `iso_week_range`, `lookup_active_contract` are file-private — diagnose via
// observed migration outcome: `total_clusters` / `quarantined_rows`).
//
// The assertion in each test reflects the OBSERVED behaviour, not a wished-for
// "ideal" behaviour. Each test's doc-comment documents the verdict (fix vs
// bleibender gap). See `08.1-10-SUMMARY.md` for the per-pattern decision.
// ----------------------------------------------------------------------------

/// Hypothesis 1 — **Lila pattern**: Vertragsbeginn mid-week.
///
/// The legacy `extra_hours` row sits in an ISO week where the contract starts
/// _during_ the week (e.g. Wednesday). For days Mo+Tu the contract-lookup
/// returns `None`; for Wed..Sun it returns Some with the workday-mask. The
/// heuristic walks Mo..So and sums `hours_per_day` only for days where (a)
/// `contract_at(day) = Some` AND (b) the contract has the weekday active.
///
/// **Expected user behaviour ("scheinbar passendes Pattern"):** The user
/// books a single weekly-lump-sum `extra_hours` row whose amount equals the
/// SUM-of-active-workdays-of-that-partial-week (here: Wed+Thu+Fri = 3 ×
/// 8h = 24h for the rest of the contract week).
///
/// **Test verdict:** The heuristic DOES match this pattern (target_sum =
/// 24h, amount = 24h → `Some({Mo, So})`). The resulting absence_period
/// stretches from Monday (= pre-contract-start) to Sunday — semantically
/// over-broad but the live `derive_hours_for_range` skips pre-contract days
/// (returns 0 for those days), so legacy_sum (24h) ≈ derived_sum (24h) and
/// the gate stays clean.
///
/// **=> No bug. Lila pattern is HANDLED by the heuristic.** If int data still
/// shows mismatch, root cause is NOT this hypothesis — the row likely has a
/// different amount (e.g. a half-week lump-sum) covered by gap-1 (b).
#[tokio::test]
async fn diagnose_int_drift_pattern_lila_contract_starts_mid_week() {
    // Build fixture: 8h Mo-Fr contract starting at Wed of ISO-W11/2026.
    // Wed of W11/2026 = 2026-03-11.
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    let lila_contract = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_5111),
        sales_person_id: fixture_sp_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Wednesday,
        from_calendar_week: 11,
        from_year: 2026,
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
        created: Some(datetime!(2026 - 03 - 11 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };

    // 24h Vacation row at Wed 2026-03-11 (= contract start day, also the first
    // post-contract-active workday in this partial week).
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2026 - 03 - 11), 24.0)]);
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([lila_contract.clone()])));

    // Observed behaviour: heuristic maps the row to absence_period
    // {Mo=2026-03-09, So=2026-03-15}. Verify this in the DAO assertion.
    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2026 - 03 - 09)
                && entity.to_date == date!(2026 - 03 - 15)
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

    // Verdict: heuristic accepts the partial-week lump-sum where the partial
    // matches the active-contract-day-sum. The observed match rate aligns
    // with the documented heuristic semantics.
    assert_eq!(
        result.total_clusters, 1,
        "Lila pattern: heuristic SHOULD map partial-week active-contract-sum lump"
    );
    assert_eq!(
        result.quarantined_rows, 0,
        "Lila pattern: no quarantine for partial-week active-contract match"
    );
}

/// Hypothesis 2 — **Anina pattern**: Vertragsende mid-week.
///
/// The legacy row sits in an ISO week where the contract ENDS mid-week
/// (e.g. Thursday). For days Mo..Thu the contract is active; for Fri..Sun
/// `contract_at(day) = None`. Active workdays Mo..Thu (4 days × 8h = 32h)
/// is the partial-week target.
///
/// **Expected user pattern:** A single 32h Vacation row anywhere in this
/// partial week.
///
/// **Test verdict:** The heuristic correctly matches the partial-week sum
/// (target_sum = 32h). It maps the row to absence_period {Mo, So} of the
/// week. The Sunday is post-contract — `derive_hours_for_range` returns 0
/// for those days (no active contract), so legacy_sum (32h) ≈ derived_sum
/// (32h). Gate clean.
///
/// **=> No bug. Anina pattern is HANDLED.** Bleibender concern: the
/// resulting `absence_period.to_date` stretches past the contract end,
/// which is semantically over-broad but harmless for derive computations.
/// Operator can shorten via Edit-modal in the 8.1-UI if desired (D-04
/// allows date edit in extra_hours pre-Convert; absence_period itself is
/// editable via the standard Absences page).
#[tokio::test]
async fn diagnose_int_drift_pattern_anina_contract_ends_mid_week() {
    // 8h Mo-Fr contract ending Thu of ISO-W18/2026 (= 2026-04-30).
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    let anina_contract = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_AAAA),
        sales_person_id: fixture_sp_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2024,
        to_day_of_week: DayOfWeek::Thursday,
        to_calendar_week: 18,
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
    };

    // 32h Vacation row at Wed 2026-04-29 (mid-partial-week).
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2026 - 04 - 29), 32.0)]);
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([anina_contract.clone()])));

    // Observed: heuristic maps to {Mo=2026-04-27, So=2026-05-03}.
    deps.absence_dao
        .expect_create()
        .withf(|entity, _, _| {
            entity.from_date == date!(2026 - 04 - 27)
                && entity.to_date == date!(2026 - 05 - 03)
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
        "Anina pattern: heuristic maps partial-week active-contract-sum lump"
    );
    assert_eq!(
        result.quarantined_rows, 0,
        "Anina pattern: no quarantine for contract-end-mid-week match"
    );
}

/// Hypothesis 3 — **Karin pattern**: Buchung in einer Woche, in der die
/// summierten effektiven Workday-Stunden ungleich dem User-erwarteten
/// Wochenpauschalen-Betrag sind, weil ein Mid-Week-Vertragswechsel die
/// `hours_per_day`-Erwartung in der Woche ändert.
///
/// **Konstruktion:** Vertrag A (Mo-Fr 8h, 40h/Woche) bis ISO-W19/2026-Mi
/// (= 2026-05-06). Vertrag B (Mo-Fr 6h, 30h/Woche) ab Do 2026-05-07.
///
/// In ISO-W19/2026 (Mo=04. .. So=10.):
/// - Mo/Di/Mi → contract_at = Some(A), `hours_per_day(A) = 8` → +8 each = 24
/// - Do/Fr → contract_at = Some(B), `hours_per_day(B) = 6` → +6 each = 12
/// - Sa/So → contract_at = Some(B), but `has_day_of_week(Sat/Sun) = false`
///   → +0
/// - target_sum = 36h
///
/// **User-Erwartung:** Eine ganze Woche Urlaub. User bucht 40h (was unter
/// Vertrag A der Wochenpauschale entsprochen hätte) ODER 30h (Vertrag B).
/// Beide Werte matchen 36h NICHT → Heuristik returns None → Quarantäne via
/// AmountAbove (40h > 8h hpd_at_workday) oder strict-match-fall-through.
///
/// **=> Bleibender gap (gap-1 (a) bestätigt):** Der per-Weekday-Lookup
/// summiert exakt die Stunden des aktiven Vertrags pro Tag. Wenn der User
/// jedoch beim Buchen "ich nehme die ganze Woche frei" mental rechnet,
/// nutzt er für die ganze Woche EINE der Vertragsraten — das passt nicht.
///
/// Operator-Resolution via 8.1-UI: Edit-Modal → Amount auf 36h korrigieren
/// → Convert. Oder Skip-Action und manuelle Anlage einer absence_period im
/// Absences-UI mit korrekter Range.
///
/// Diese Diagnose-Test bleibt mit observation-asserts stehen; die
/// Behavior-Doku ist die Begründung für gap-1 (a).
#[tokio::test]
async fn diagnose_int_drift_pattern_karin_mid_week_contract_change_breaks_lump_sum() {
    let mut deps = build_dependencies();
    deps.permission_service = permission_service_allow_all();
    install_empty_gate_scope(&mut deps);

    // Contract A: 40h/week Mo-Fr ending Wed 2026-05-06 (= ISO-W19 day 3).
    let contract_a = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_CA01),
        sales_person_id: fixture_sp_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2024,
        to_day_of_week: DayOfWeek::Wednesday,
        to_calendar_week: 19,
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
    };
    // Contract B: 30h/week Mo-Fr starting Thu 2026-05-07 (= ISO-W19 day 4).
    let contract_b = EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_CA02),
        sales_person_id: fixture_sp_id(),
        expected_hours: 30.0,
        from_day_of_week: DayOfWeek::Thursday,
        from_calendar_week: 19,
        from_year: 2026,
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
        vacation_days: 25,
        created: Some(datetime!(2026 - 05 - 07 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };

    // User books 40h (= Vertrag-A weekly total) at Wed 2026-05-06.
    // target_sum für ISO-W19 = 3×8 + 2×6 = 36 ≠ 40 → heuristic returns None
    // → strict-match path → AmountAbove quarantine (40 > 8 hpd at Wed).
    let rows: Arc<[LegacyExtraHoursRow]> =
        Arc::from([legacy_row(date!(2026 - 05 - 06), 40.0)]);
    let rows_clone = rows.clone();
    deps.cutover_dao
        .expect_find_legacy_extra_hours_not_yet_migrated()
        .returning(move |_| Ok(rows_clone.clone()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| {
            Ok(Arc::from([contract_a.clone(), contract_b.clone()]))
        });

    // Observed: NO absence_period created, ONE quarantine row with
    // amount_above_contract_hours.
    deps.absence_dao.expect_create().times(0);
    deps.cutover_dao.expect_upsert_migration_source().times(0);
    deps.cutover_dao
        .expect_upsert_quarantine()
        .withf(|row, _| row.reason.as_ref() == "amount_above_contract_hours")
        .times(1)
        .returning(|_, _| Ok(()));

    let service = deps.build_service(build_default_transaction_dao());
    let result = service.run(true, ().auth(), None).await.unwrap();

    // Verdict: heuristic correctly REJECTS the row because target_sum (36h)
    // ≠ amount (40h). Gap-1 (a) confirmed: when a mid-week contract change
    // makes per-day-rates differ across the ISO week, the user's "I took
    // the whole week off" lump-sum convention does not match the heuristic's
    // per-day sum. Operator resolves manually via the 8.1-UI.
    assert_eq!(
        result.total_clusters, 0,
        "Karin pattern: mid-week contract change breaks the weekly-lump-sum match"
    );
    assert_eq!(
        result.quarantined_rows, 1,
        "Karin pattern: row falls to strict-match path → quarantines"
    );
}

// ----------------------------------------------------------------------------
// Phase 8.2 Plan 01 — manual_range branch on convert_quarantine_entry tests.
//
// Four mockall unit tests covering the Manual-Range backend branch (D-29):
//   1. happy path Karin — mid-week contract change, manual_range = ISO-week
//      Mo..So → absence_period created with given range, soft-delete called,
//      upsert_migration_source called. Heuristik wird NICHT aufgerufen
//      (Branch surface-isolated per RESEARCH Open Q 3 / D-35).
//   2. inverted range (start > end) → DateOrderWrong, NO DAO writes.
//   3. cross-year range → ValidationError(InvalidValue("year-mismatch")),
//      NO DAO writes.
//   4. existing absence_period overlaps → ValidationError(OverlappingPeriod),
//      NO writes after find_overlapping.
// ----------------------------------------------------------------------------

mod manual_range_convert_quarantine_tests {
    use super::*;
    use dao::absence::AbsenceCategoryEntity;
    use dao::absence::AbsencePeriodEntity;
    use dao::absence::DayFractionEntity;
    use mockall::Sequence;
    use service::cutover::ManualRange;
    use service::ValidationFailureItem;
    use shifty_utils::DateRange;

    /// Stable id used for the Karin-Pattern row under test.
    fn karin_extra_hours_id() -> Uuid {
        Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_CAFE_0001)
    }

    fn karin_row(day: time::Date, amount: f32) -> LegacyExtraHoursRow {
        legacy_row_with_id(karin_extra_hours_id(), day, amount)
    }

    fn permission_service_allow_cutover_admin() -> MockPermissionService {
        let mut p = MockPermissionService::new();
        p.expect_check_permission()
            .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
            .returning(|_, _| Ok(()));
        p
    }

    /// Karin-Pattern Contract A: 40h/week Mo-Fr ending Wed 2026-05-06.
    fn karin_contract_a() -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_CA01),
            sales_person_id: fixture_sp_id(),
            expected_hours: 40.0,
            from_day_of_week: DayOfWeek::Monday,
            from_calendar_week: 1,
            from_year: 2024,
            to_day_of_week: DayOfWeek::Wednesday,
            to_calendar_week: 19,
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

    /// Karin-Pattern Contract B: 30h/week Mo-Fr starting Thu 2026-05-07.
    fn karin_contract_b() -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_CA02),
            sales_person_id: fixture_sp_id(),
            expected_hours: 30.0,
            from_day_of_week: DayOfWeek::Thursday,
            from_calendar_week: 19,
            from_year: 2026,
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
            vacation_days: 25,
            created: Some(datetime!(2026 - 05 - 07 10:00:00)),
            deleted: None,
            version: Uuid::nil(),
        }
    }

    /// Build an `AbsencePeriodEntity` representing a "conflict" row for the
    /// overlap-test. Only the `logical_id` field is functionally read by the
    /// service path; everything else is informational.
    fn conflicting_absence_period(
        logical_id: Uuid,
        from: time::Date,
        to: time::Date,
    ) -> AbsencePeriodEntity {
        AbsencePeriodEntity {
            id: logical_id,
            logical_id,
            sales_person_id: fixture_sp_id(),
            category: AbsenceCategoryEntity::Vacation,
            from_date: from,
            to_date: to,
            description: Arc::from(""),
            created: time::PrimitiveDateTime::new(
                date!(2026 - 01 - 01),
                time::Time::MIDNIGHT,
            ),
            deleted: None,
            version: Uuid::new_v4(),
            day_fraction: DayFractionEntity::Full,
        }
    }

    // -----------------------------------------------------------------
    // Test 1: Manual-Range happy-path (Karin) — surface-isolated.
    //
    // Karin-Quarantäne: 40h Vacation am Wed 2026-05-06 (mit mid-week
    // Vertragswechsel: Contract A 40h Mo-Fr ending Wed; Contract B 30h Mo-Fr
    // starting Thu). Operator gibt manuell die ISO-Woche {Mo 2026-05-04,
    // So 2026-05-10} an. Backend skipt Heuristik komplett, schreibt
    // absence_period mit dem gegebenen Range, soft-deleted die Row.
    //
    // Per RESEARCH Open Q 3 / D-35 isolieren wir den Test auf die
    // manual_range-Surface — wir asserten NICHT, dass post-Convert-Drift = 0.
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn manual_range_resolves_karin_quarantine() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        // Karin-Row: 40h Vacation am Wed 2026-05-06.
        let row = karin_row(date!(2026 - 05 - 06), 40.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        let row_arc_first = row_arc.clone();
        // Replay-Pass (Step 9) findet keine Legacy-Rows mehr (alle soft-
        // deleted) — empty vector. Mit Sequence garantieren wir die
        // Reihenfolge.
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(row_arc_first.clone()));
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        // EmployeeWorkDetails: Contract A + Contract B (mid-week Wechsel).
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([karin_contract_a(), karin_contract_b()])));

        // find_overlapping: empty (kein Konflikt) — happy-path.
        deps.absence_dao
            .expect_find_overlapping()
            .times(1)
            .returning(|_, _, _, _, _| Ok(Arc::from([])));

        // create: exact 1× mit dem gegebenen Range und Vacation category.
        deps.absence_dao
            .expect_create()
            .withf(|entity, process, _| {
                entity.from_date == date!(2026 - 05 - 04)
                    && entity.to_date == date!(2026 - 05 - 10)
                    && entity.category == AbsenceCategoryEntity::Vacation
                    && entity.sales_person_id == fixture_sp_id()
                    && process == "phase-4-cutover-migration"
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        // upsert_migration_source: 1× mit der Karin-Row-ID.
        deps.cutover_dao
            .expect_upsert_migration_source()
            .withf(|src, _| src.extra_hours_id == karin_extra_hours_id())
            .times(1)
            .returning(|_, _| Ok(()));

        // soft_delete_bulk: 1× mit der Karin-Row-ID.
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .withf(|ids, process, _, _| {
                ids.len() == 1
                    && ids[0] == karin_extra_hours_id()
                    && process == "phase-4-cutover-migration"
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // Replay (Step 9): empty bucket-map (kein Quarantine), find_legacy_scope_set
        // returns empty. Replay ist non-fatal-by-design (8.1-02 Pattern); falls
        // hier was bricht, würde refreshed_drift_report None sein — der Test
        // prüft nur die strict assertions auf absence_period.
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let manual = ManualRange {
            start_date: date!(2026 - 05 - 04),
            end_date: date!(2026 - 05 - 10),
        };

        let service = deps.build_service(build_default_transaction_dao());
        let outcome = service
            .convert_quarantine_entry(karin_extra_hours_id(), Some(manual), None, ().auth(), None)
            .await
            .expect("manual-range convert succeeds for Karin pattern");

        // STRICT assertions — manual_range surface.
        assert_eq!(outcome.deleted_extra_hours_id, karin_extra_hours_id());
        assert_eq!(outcome.absence_period.from_date, date!(2026 - 05 - 04));
        assert_eq!(outcome.absence_period.to_date, date!(2026 - 05 - 10));
        assert_eq!(
            outcome.absence_period.category,
            AbsenceCategoryEntity::Vacation
        );
        assert_eq!(outcome.absence_period.sales_person_id, fixture_sp_id());
        // Best-effort: refreshed_drift_report can be Some or None — replay
        // is non-fatal-by-design (RESEARCH Open Q 3 / D-35). Don't assert.
    }

    // -----------------------------------------------------------------
    // Test 2: Inverted range (start > end) → DateOrderWrong.
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn manual_range_rejects_inverted_range() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = karin_row(date!(2026 - 05 - 06), 40.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(row_arc.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([karin_contract_a(), karin_contract_b()])));

        // Hard guard: NO DAO writes / overlap-check on the inverted-range path.
        deps.absence_dao.expect_find_overlapping().times(0);
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);

        // Tx opened, never committed; rolled back via Drop.
        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        let manual = ManualRange {
            start_date: date!(2026 - 05 - 08),
            end_date: date!(2026 - 05 - 04),
        };

        let service = deps.build_service(tx_dao);
        let result = service
            .convert_quarantine_entry(karin_extra_hours_id(), Some(manual), None, ().auth(), None)
            .await;
        assert!(
            matches!(result, Err(ServiceError::DateOrderWrong(s, e))
                if s == date!(2026 - 05 - 08) && e == date!(2026 - 05 - 04)),
            "inverted manual_range must return DateOrderWrong; got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------
    // Test 3: Year-boundary crossing → ValidationError("year-mismatch").
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn manual_range_rejects_year_boundary_crossing() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        // Karin-Row im Jahr 2026 (Wed 2026-05-06).
        let row = karin_row(date!(2026 - 05 - 06), 40.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(row_arc.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([karin_contract_a(), karin_contract_b()])));

        // No DAO writes / overlap-check.
        deps.absence_dao.expect_find_overlapping().times(0);
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);

        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        // Range straddles 2025/2026 boundary.
        let manual = ManualRange {
            start_date: date!(2025 - 12 - 29),
            end_date: date!(2026 - 01 - 04),
        };

        let service = deps.build_service(tx_dao);
        let result = service
            .convert_quarantine_entry(karin_extra_hours_id(), Some(manual), None, ().auth(), None)
            .await;
        match result {
            Err(ServiceError::ValidationError(items)) => {
                assert_eq!(items.len(), 1, "exactly one validation failure expected");
                match &items[0] {
                    ValidationFailureItem::InvalidValue(msg) => {
                        assert!(
                            msg.contains("calendar year"),
                            "year-mismatch message expected, got {msg:?}"
                        );
                    }
                    other => panic!("expected InvalidValue, got {other:?}"),
                }
            }
            other => panic!("expected ValidationError, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // Test 4: Existing absence_period overlaps → ValidationError
    //         (OverlappingPeriod). NO subsequent writes.
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn manual_range_rejects_when_existing_absence_period_overlaps() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = karin_row(date!(2026 - 05 - 06), 40.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(row_arc.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([karin_contract_a(), karin_contract_b()])));

        // Conflict exists: an absence_period covering parts of Mo..So already
        // exists for the same (sales_person, category).
        let conflict_logical_id =
            Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_C0FE);
        deps.absence_dao
            .expect_find_overlapping()
            .withf(move |sp_id, cat, _range, exclude, _tx| {
                *sp_id == fixture_sp_id()
                    && *cat == AbsenceCategoryEntity::Vacation
                    && exclude.is_none()
            })
            .times(1)
            .returning(move |_, _, _, _, _| {
                Ok(Arc::from([conflicting_absence_period(
                    conflict_logical_id,
                    date!(2026 - 05 - 05),
                    date!(2026 - 05 - 07),
                )]))
            });

        // Hard guard: writes MUST NOT happen on the overlap path.
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        deps.extra_hours_service.expect_soft_delete_bulk().times(0);

        let mut tx_dao = MockTransactionDao::new();
        tx_dao.expect_use_transaction().returning(|_| Ok(MockTransaction));
        tx_dao.expect_commit().times(0);
        tx_dao.expect_rollback().returning(|_| Ok(()));

        let manual = ManualRange {
            start_date: date!(2026 - 05 - 04),
            end_date: date!(2026 - 05 - 10),
        };

        let service = deps.build_service(tx_dao);
        let result = service
            .convert_quarantine_entry(karin_extra_hours_id(), Some(manual), None, ().auth(), None)
            .await;
        match result {
            Err(ServiceError::ValidationError(items)) => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    ValidationFailureItem::OverlappingPeriod(id) => {
                        assert_eq!(
                            *id, conflict_logical_id,
                            "OverlappingPeriod must surface the conflict's logical_id"
                        );
                    }
                    other => panic!("expected OverlappingPeriod, got {other:?}"),
                }
            }
            other => panic!("expected ValidationError, got {other:?}"),
        }
    }

    // Suppress dead-code warnings for the `DateRange` import used only for
    // documentation-style purposes — `DateRange::new` is actually exercised
    // inside the production code under test, but if the compiler decides
    // the import is unused after expansion we keep it explicit.
    #[allow(dead_code)]
    fn _suppress_unused_imports() -> Option<DateRange> {
        DateRange::new(date!(2026 - 01 - 01), date!(2026 - 01 - 02)).ok()
    }
}

// ----------------------------------------------------------------------------
// Phase 8.3 Plan 05 — day_fraction threading on convert + bulk-convert.
//
// Three mockall unit tests covering the new `day_fraction` parameter:
//   1. convert_quarantine_entry_with_half_day_persists_day_fraction —
//      Heiligabend-Pattern (Mo 8h-Vertrag, 4h Vacation row, manual_range =
//      single day, day_fraction=Some(Half)) → AbsencePeriodEntity mit
//      DayFractionEntity::Half wird persistiert.
//   2. convert_quarantine_entry_without_day_fraction_defaults_to_full —
//      Backwards-Compat: day_fraction = None defaultet zu Full.
//   3. bulk_convert_quarantine_rows_with_half_applies_to_all_rows —
//      D-08.3-07: alle Rows derselben Bulk-Operation teilen denselben
//      Half-Wert.
//
// `withf`-Predicate auf `entity.day_fraction == DayFractionEntity::Half`
// statt Full-Payload-Equality, weil die anderen Felder bereits durch die
// existing 8.1/8.2-Tests abgedeckt sind und die new Tests sich strikt auf
// die day_fraction-Threading-Surface fokussieren sollen.
// ----------------------------------------------------------------------------

mod day_fraction_convert_tests {
    use super::*;
    use dao::absence::AbsenceCategoryEntity;
    use dao::absence::DayFractionEntity;
    use mockall::Sequence;
    use service::absence::DayFraction;
    use service::cutover::ManualRange;

    fn permission_service_allow_cutover_admin() -> MockPermissionService {
        let mut p = MockPermissionService::new();
        p.expect_check_permission()
            .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
            .returning(|_, _| Ok(()));
        p
    }

    /// Stable extra_hours_id used für den Heiligabend-Halbtag-Test.
    fn half_day_extra_hours_id() -> Uuid {
        Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BAFF_0001)
    }

    fn half_day_row(day: time::Date, amount: f32) -> LegacyExtraHoursRow {
        legacy_row_with_id(half_day_extra_hours_id(), day, amount)
    }

    // -----------------------------------------------------------------
    // Test 1: convert_quarantine_entry with day_fraction=Some(Half) →
    //         AbsencePeriodEntity.day_fraction == Half persistiert.
    //
    // Heiligabend-Pattern: 4h Vacation am Donnerstag 2026-12-24, 8h-Vertrag
    // (Mo-Fr). manual_range = {2026-12-24, 2026-12-24} (single day, operator
    // gibt das Datum explizit vor, weil die Heuristik bei amount=4 nicht
    // matched). day_fraction=Some(Half) signalisiert: 0.5 Tag Vacation.
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn convert_quarantine_entry_with_half_day_persists_day_fraction() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = half_day_row(date!(2026 - 12 - 24), 4.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        let row_arc_first = row_arc.clone();
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(row_arc_first.clone()));
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

        deps.absence_dao
            .expect_find_overlapping()
            .times(1)
            .returning(|_, _, _, _, _| Ok(Arc::from([])));

        // STRICT assert: entity.day_fraction MUST be Half.
        deps.absence_dao
            .expect_create()
            .withf(|entity, _, _| {
                entity.from_date == date!(2026 - 12 - 24)
                    && entity.to_date == date!(2026 - 12 - 24)
                    && entity.category == AbsenceCategoryEntity::Vacation
                    && entity.day_fraction == DayFractionEntity::Half
            })
            .times(1)
            .returning(|_, _, _| Ok(()));

        deps.cutover_dao
            .expect_upsert_migration_source()
            .times(1)
            .returning(|_, _| Ok(()));
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        // Replay (Step 9): empty bucket-map. Replay non-fatal-by-design.
        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let manual = ManualRange {
            start_date: date!(2026 - 12 - 24),
            end_date: date!(2026 - 12 - 24),
        };

        let service = deps.build_service(build_default_transaction_dao());
        let outcome = service
            .convert_quarantine_entry(
                half_day_extra_hours_id(),
                Some(manual),
                Some(DayFraction::Half),
                ().auth(),
                None,
            )
            .await
            .expect("half-day convert succeeds");

        assert_eq!(outcome.deleted_extra_hours_id, half_day_extra_hours_id());
        // absence_period.day_fraction reflects Operator-supplied Half.
        assert_eq!(
            outcome.absence_period.day_fraction,
            DayFractionEntity::Half,
            "outcome.absence_period.day_fraction must reflect the operator-supplied Half"
        );
    }

    // -----------------------------------------------------------------
    // Test 2: convert_quarantine_entry with day_fraction = None →
    //         AbsencePeriodEntity.day_fraction == Full (Backwards-Compat).
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn convert_quarantine_entry_without_day_fraction_defaults_to_full() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let row = half_day_row(date!(2026 - 12 - 24), 4.0);
        let row_arc: Arc<[LegacyExtraHoursRow]> = Arc::from([row]);
        let row_arc_first = row_arc.clone();
        let mut seq = Sequence::new();
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_| Ok(row_arc_first.clone()));
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(Arc::from([])));

        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_8h_mon_fri_contract()])));

        deps.absence_dao
            .expect_find_overlapping()
            .times(1)
            .returning(|_, _, _, _, _| Ok(Arc::from([])));

        // STRICT assert: entity.day_fraction MUST be Full (Default).
        deps.absence_dao
            .expect_create()
            .withf(|entity, _, _| entity.day_fraction == DayFractionEntity::Full)
            .times(1)
            .returning(|_, _, _| Ok(()));

        deps.cutover_dao
            .expect_upsert_migration_source()
            .times(1)
            .returning(|_, _| Ok(()));
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        deps.cutover_dao.expect_upsert_quarantine().times(0);
        deps.cutover_dao
            .expect_find_legacy_scope_set()
            .returning(|_| Ok(Arc::from([])));

        let manual = ManualRange {
            start_date: date!(2026 - 12 - 24),
            end_date: date!(2026 - 12 - 24),
        };

        let service = deps.build_service(build_default_transaction_dao());
        let outcome = service
            .convert_quarantine_entry(
                half_day_extra_hours_id(),
                Some(manual),
                None, // <-- backwards-compat: kein day_fraction
                ().auth(),
                None,
            )
            .await
            .expect("convert succeeds with None day_fraction");

        assert_eq!(
            outcome.absence_period.day_fraction,
            DayFractionEntity::Full,
            "None day_fraction must default to Full (no-drift / Backwards-Compat)"
        );
    }

    // -----------------------------------------------------------------
    // Test 3: bulk_convert_quarantine_rows with day_fraction = Some(Half) →
    //         ALLE konvertierten Entities haben Half (D-08.3-07).
    //
    // 3 weekly-lump-sum-Rows (Mo-Mi 3-day-Vertrag, 20h/Woche, valid lumps in
    // KW 23/24/25 von 2024) — alle bekommen denselben Half-Wert.
    // -----------------------------------------------------------------
    #[tokio::test]
    async fn bulk_convert_quarantine_rows_with_half_applies_to_all_rows() {
        let mut deps = build_dependencies();
        deps.permission_service = permission_service_allow_cutover_admin();

        let rows: Arc<[LegacyExtraHoursRow]> = Arc::from([
            legacy_row_with_id(
                Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BABE_0001),
                date!(2024 - 06 - 07),
                20.0,
            ),
            legacy_row_with_id(
                Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BABE_0002),
                date!(2024 - 06 - 14),
                20.0,
            ),
            legacy_row_with_id(
                Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BABE_0003),
                date!(2024 - 06 - 21),
                20.0,
            ),
        ]);
        let rows_for_first = rows.clone();
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

        // STRICT assert: ALLE 3 absence_period-Inserts müssen day_fraction = Half haben.
        deps.absence_dao
            .expect_create()
            .withf(|entity, _, _| entity.day_fraction == DayFractionEntity::Half)
            .times(3)
            .returning(|_, _, _| Ok(()));

        deps.cutover_dao
            .expect_upsert_migration_source()
            .times(3)
            .returning(|_, _| Ok(()));
        deps.extra_hours_service
            .expect_soft_delete_bulk()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

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
                Some(DayFraction::Half),
                ().auth(),
                None,
            )
            .await
            .expect("bulk-convert-with-Half succeeds");

        assert_eq!(outcome.converted_absence_periods.len(), 3);
        for ap in &outcome.converted_absence_periods {
            assert_eq!(
                ap.day_fraction,
                DayFractionEntity::Half,
                "all bulk-converted entities must share the Half day_fraction (D-08.3-07)"
            );
        }
    }
}

// ============================================================================
// REPRODUCERS für Drift-Symptome auf INT (Mai 2026)
// ============================================================================
//
// Diese Tests dokumentieren bekannte Lücken in der Cutover-Heuristik /
// `compute_gate`-Asymmetrie, die im aktuellen Test-Setup nicht aufgedeckt
// werden, weil `arrange_lump_sum_migration` `install_empty_gate_scope`
// aufruft (gate scope = leer → keine Drift-Berechnung).
//
// User-Symptom: 20h-Vertrag (Di-Fr) + 20h Vacation-Eintrag → Drift im
// Cutover-Gate-Dry-Run. „Sämtliche Tage matchen nicht" und „Vorjahre
// machen Probleme."
// ============================================================================

mod drift_reproducers_int_may_2026 {
    use super::*;

    /// Fixture: 20h/Woche-Vertrag, Workdays Di-Fr (4 Tage → hours_per_day=5).
    /// Spannt 2020..=2026. Stellt die User-INT-Konstellation dar.
    fn fixture_20h_tue_fri_contract() -> EmployeeWorkDetails {
        let mut wd = fixture_3day_mo_tu_we_contract();
        wd.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0030);
        wd.expected_hours = 20.0;
        wd.workdays_per_week = 4;
        wd.monday = false;
        wd.tuesday = true;
        wd.wednesday = true;
        wd.thursday = true;
        wd.friday = true;
        wd
    }

    // ------------------------------------------------------------------------
    // T2: Cross-Year ISO-Woche — Mo 2024-12-30 gehört nach ISO 8601 zu KW 1 /
    // 2025. Die Heuristik schreibt eine `absence_period(2024-12-30,
    // 2025-01-05)` — also über die Jahresgrenze. `compute_gate` aggregiert
    // legacy vs derived per `(sales_person, year)`. legacy_sum ist auf
    // year=2024 gebucht (date_time year aus extra_hours), derived_sum splittet
    // sich zwischen 2024 und 2025 → drift > 0 in mindestens einem Jahr.
    //
    // Dieser Test pinnt das aktuelle Heuristik-Verhalten: Mo 2024-12-30 →
    // absence_period(2024-12-30, 2025-01-05). Der daraus folgende Gate-Drift
    // ist ein separater Bug, der über `derive_hours_for_range`-Filter und
    // die Per-Year-Aggregation in `compute_gate_inner` entsteht.
    // ------------------------------------------------------------------------
    #[tokio::test]
    async fn t2_cross_year_iso_week_lump_sum_writes_cross_year_absence_period() {
        let mut deps = build_dependencies();
        let rows: Arc<[LegacyExtraHoursRow]> =
            Arc::from([legacy_row(date!(2024 - 12 - 30), 20.0)]);
        let rows_clone = rows.clone();

        deps.permission_service = permission_service_allow_all();
        install_empty_gate_scope(&mut deps);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(rows_clone.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_20h_tue_fri_contract()])));

        // Heuristik muss die Cross-Year-Range Mo 2024-12-30 .. So 2025-01-05
        // produzieren. Das ist die KW 1 / 2025 nach ISO 8601 (Do dieser Woche
        // ist 2025-01-02, gehört zu 2025).
        deps.absence_dao
            .expect_create()
            .withf(|entity, _, _| {
                entity.from_date == date!(2024 - 12 - 30)
                    && entity.to_date == date!(2025 - 01 - 05)
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
        assert_eq!(result.total_clusters, 1, "Cross-Year-Lump-Sum migriert");
        assert_eq!(
            result.quarantined_rows, 0,
            "Heuristik akzeptiert die Row trotz Cross-Year-Range"
        );

        // Drift-Diagnose-Hinweis (test-internal): die migrierte Period
        // erstreckt sich über zwei Jahre. `derive_hours_for_range` wird in
        // `compute_gate_inner` per (sp, year) aufgerufen mit year_start =
        // Jan 1 year, year_end = Dec 31 year — der year=2024-Aufruf sieht
        // nur Mo 12-30 (non-workday → skip) + Di 12-31 (workday → 5h),
        // also derived_sum=5h vs legacy_sum=20h → drift=15h. Symmetrisch
        // entsteht ein Phantom-Drift in 2025 (legacy=0, derived=15h aus
        // Mi/Do/Fr 2025-01-01..03), falls (sp, 2025) im scope ist.
        //
        // Dieser Test pinnt nur die Range; der eigentliche Drift-Bug
        // entsteht downstream in `compute_gate_inner` und braucht einen
        // eigenen End-to-End-Test mit gefülltem scope-set.
    }

    // ------------------------------------------------------------------------
    // T3: Soft-deleted historischer Vertrag. `lookup_active_contract` filtert
    // `deleted.is_some()`. Wenn ein Mitarbeiter inzwischen einen neuen
    // Vertrag bekommen hat und der alte (historische) soft-deleted ist,
    // findet die Heuristik für alle Tage im Vorjahres-Range KEINEN aktiven
    // Vertrag → Quarantine als `ContractNotActiveAtDate`.
    //
    // User-Symptom-Beitrag: ALLE Vorjahres-Vacation-Einträge dieser Person
    // erscheinen als Drift, sobald `compute_gate` läuft (Quarantine-Rows
    // werden als drift_rows aggregiert mit reason `contract_not_active_at_date`).
    // ------------------------------------------------------------------------
    #[tokio::test]
    async fn t3_soft_deleted_historical_contract_quarantines_past_year_row() {
        let mut deps = build_dependencies();

        // Alter 20h/4-Tage-Vertrag, gültig 2023..2024, soft-deleted Ende 2024.
        let mut old_contract = fixture_20h_tue_fri_contract();
        old_contract.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_DEAD_0001);
        old_contract.from_year = 2023;
        old_contract.from_calendar_week = 1;
        old_contract.from_day_of_week = DayOfWeek::Monday;
        old_contract.to_year = 2024;
        old_contract.to_calendar_week = 52;
        old_contract.to_day_of_week = DayOfWeek::Sunday;
        old_contract.deleted = Some(datetime!(2024 - 12 - 31 23:00:00));

        // Neuer Vertrag ab 2025 (40h, Mo-Fr — egal, deckt nur 2025+).
        let mut new_contract = fixture_8h_mon_fri_contract();
        new_contract.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BEEF_0001);
        new_contract.from_year = 2025;
        new_contract.from_calendar_week = 1;
        new_contract.from_day_of_week = DayOfWeek::Monday;
        new_contract.to_year = 2026;
        new_contract.to_calendar_week = 52;
        new_contract.to_day_of_week = DayOfWeek::Sunday;

        // 1× Vacation-Eintrag in 2024 (im Bereich des soft-deleted alten
        // Vertrags). Mo 2024-06-03, 20h.
        let rows: Arc<[LegacyExtraHoursRow]> =
            Arc::from([legacy_row(date!(2024 - 06 - 03), 20.0)]);
        let rows_clone = rows.clone();

        deps.permission_service = permission_service_allow_all();
        install_empty_gate_scope(&mut deps);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(rows_clone.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| {
                Ok(Arc::from([old_contract.clone(), new_contract.clone()]))
            });

        // Erwartung: KEIN absence_period (alle Quellen verworfen).
        deps.absence_dao.expect_create().times(0);
        deps.cutover_dao.expect_upsert_migration_source().times(0);
        // Erwartung: 1× Quarantine mit reason `contract_not_active_at_date`.
        deps.cutover_dao
            .expect_upsert_quarantine()
            .withf(|row, _| row.reason.as_ref() == "contract_not_active_at_date")
            .times(1)
            .returning(|_, _| Ok(()));

        let service = deps.build_service(build_default_transaction_dao());
        let result = service
            .run(true, ().auth(), None)
            .await
            .expect("run succeeded");
        assert_eq!(result.total_clusters, 0);
        assert_eq!(
            result.quarantined_rows, 1,
            "Vorjahres-Row mit nur soft-deleted Vertrag → Quarantine"
        );
    }

    // ------------------------------------------------------------------------
    // T1 (Companion): zeigt das LIVE-Ende der Drift-Asymmetrie an — wenn die
    // Heuristik in cutover.rs eine Mo-So-Lump-Sum-Range schreibt, würde
    // `derive_hours_for_range` mit Holiday auf einem Workday einen
    // verkürzten derived_sum produzieren. Der eigentliche Verhaltens-Pin
    // dafür sitzt in
    // `crate::test::absence_derive_hours_range::test_lump_sum_vacation_period_with_holiday_emits_short_derived_sum`
    // — dieser Test hier pinnt nur, dass die Heuristik die Holiday-Woche
    // OHNE Holiday-Bewusstsein als Lump-Sum akzeptiert.
    // ------------------------------------------------------------------------
    #[tokio::test]
    async fn t1_heuristik_akzeptiert_lump_sum_ohne_holiday_korrektur() {
        let mut deps = build_dependencies();
        // 20h Vacation-Eintrag auf Mo 2024-06-03 — KW 23/2024.
        // Bemerkung: die Heuristik fragt KEINEN SpecialDayService an, also
        // hat das Vorhandensein eines Holiday in der Woche keinen Einfluss
        // auf das Migrations-Ergebnis. Das ist die Asymmetrie zu
        // `derive_hours_for_range`, die im Gate downstream Drift erzeugt.
        let rows: Arc<[LegacyExtraHoursRow]> =
            Arc::from([legacy_row(date!(2024 - 06 - 03), 20.0)]);
        let rows_clone = rows.clone();

        deps.permission_service = permission_service_allow_all();
        install_empty_gate_scope(&mut deps);
        deps.cutover_dao
            .expect_find_legacy_extra_hours_not_yet_migrated()
            .returning(move |_| Ok(rows_clone.clone()));
        deps.employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(|_, _, _| Ok(Arc::from([fixture_20h_tue_fri_contract()])));

        // Heuristik schreibt Mo-So der KW 23.
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
        assert_eq!(result.total_clusters, 1);
        assert_eq!(
            result.quarantined_rows, 0,
            "Heuristik kennt keinen Holiday — schreibt ungestört Mo-So-Range. \
             Die spätere derived_sum-Verkürzung in derive_hours_for_range \
             erzeugt den Gate-Drift (siehe Companion-Test in \
             test::absence_derive_hours_range)."
        );
    }
}
