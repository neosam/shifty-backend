//! Mock-basierte Service-Tests für `VacationBalanceServiceImpl` (Plan 08-02).
//!
//! Pflicht-Coverage (≥6 Tests, 08-02-PLAN.md):
//! 1. `get_returns_entitlement_minus_used_minus_planned`: Self-Path Happy.
//! 2. `get_with_hr_succeeds`: HR-Path Happy.
//! 3. `get_other_sales_person_without_hr_is_forbidden`: T-8-AUTH-01 + T-8-IDOR-01.
//! 4. `get_team_without_hr_is_forbidden`: T-8-AUTH-02.
//! 5. `get_team_aggregates_per_paid_sales_person`: HR-Path Aggregat.
//! 6. `get_with_no_active_contract_returns_zero_entitlement`: Edge-Case.
//! 7. `get_year_without_carryover_returns_zero_carryover`: Carryover-None-Path.

use std::sync::Arc;

use dao::MockTransaction;
use dao::MockTransactionDao;
use service::absence::{AbsenceCategory, AbsencePeriod, DayFraction, MockAbsenceService};
use service::carryover::{Carryover, MockCarryoverService};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::vacation_balance::VacationBalanceService;
use service::{MockPermissionService, ServiceError};
use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use uuid::{uuid, Uuid};

use crate::test::error_test::test_forbidden;
use crate::vacation_balance::{VacationBalanceServiceDeps, VacationBalanceServiceImpl};

const TEST_YEAR: u32 = 2026;

fn default_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000001")
}

fn other_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000002")
}

fn vacation_period(from: time::Date, to: time::Date, sp_id: Uuid) -> AbsencePeriod {
    AbsencePeriod {
        id: uuid!("AB000000-0000-0000-0000-0000000000A1"),
        sales_person_id: sp_id,
        category: AbsenceCategory::Vacation,
        from_date: from,
        to_date: to,
        description: "vacation".into(),
        created: Some(datetime!(2026 - 01 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-0000000000A1"),
        day_fraction: DayFraction::Full,
    }
}

fn full_year_contract(sp_id: Uuid, vacation_days: u8) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: uuid!("11111111-0000-0000-0000-000000000001"),
        sales_person_id: sp_id,
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        // Calendar week 1 of 2026 → covers full year for our tests.
        from_calendar_week: 1,
        from_year: 2025,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2030,
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
        vacation_days,
        created: Some(datetime!(2024 - 12 - 15 09:00:00)),
        deleted: None,
        version: uuid!("DD000000-0000-0000-0000-000000000001"),
    }
}

fn paid_sales_person(id: Uuid, name: &str) -> SalesPerson {
    SalesPerson {
        id,
        name: name.into(),
        background_color: "#888888".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: uuid!("EE000000-0000-0000-0000-000000000001"),
    }
}

pub(crate) struct VacationBalanceDependencies {
    pub absence_service: MockAbsenceService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub carryover_service: MockCarryoverService,
    pub sales_person_service: MockSalesPersonService,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub transaction_dao: MockTransactionDao,
}

impl VacationBalanceServiceDeps for VacationBalanceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type AbsenceService = MockAbsenceService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type CarryoverService = MockCarryoverService;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type TransactionDao = MockTransactionDao;
}

impl VacationBalanceDependencies {
    pub(crate) fn build_service(self) -> VacationBalanceServiceImpl<VacationBalanceDependencies> {
        VacationBalanceServiceImpl {
            absence_service: self.absence_service.into(),
            employee_work_details_service: self.employee_work_details_service.into(),
            carryover_service: self.carryover_service.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

/// Build dependencies with a default `today = 2026-06-15` and a happy-path
/// transaction setup. Permission/sales-person checks default to `Ok(())`
/// so individual tests can override them for forbidden flows.
pub(crate) fn build_dependencies() -> VacationBalanceDependencies {
    let absence_service = MockAbsenceService::new();
    let employee_work_details_service = MockEmployeeWorkDetailsService::new();
    let carryover_service = MockCarryoverService::new();
    let sales_person_service = MockSalesPersonService::new();
    let permission_service = MockPermissionService::new();
    let mut clock_service = MockClockService::new();
    let mut transaction_dao = MockTransactionDao::new();

    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));
    clock_service
        .expect_date_now()
        .returning(|| date!(2026 - 06 - 15));

    VacationBalanceDependencies {
        absence_service,
        employee_work_details_service,
        carryover_service,
        sales_person_service,
        permission_service,
        clock_service,
        transaction_dao,
    }
}

// =========================================================================
// get
// =========================================================================

#[tokio::test]
async fn get_returns_entitlement_minus_used_minus_planned() {
    let mut deps = build_dependencies();

    // Self-Path: HR fails, verify_user_is_sales_person succeeds.
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    // 1 vergangene Vacation-Periode (2026-04-01..04-05 → 5 Tage, used).
    // 1 zukünftige Vacation-Periode (2026-08-01..08-10 → 10 Tage, planned).
    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(move |_, _, _| {
            Ok(Arc::from([
                vacation_period(date!(2026 - 04 - 01), date!(2026 - 04 - 05), sp_id),
                vacation_period(date!(2026 - 08 - 01), date!(2026 - 08 - 10), sp_id),
            ]))
        });

    // Vertrag: 25 Vacation-Tage, deckt das ganze Jahr ab.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 25)])));

    // Carryover: 5 Tage.
    deps.carryover_service
        .expect_get_carryover()
        .returning(move |_, _, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year: TEST_YEAR,
                carryover_hours: 0.0,
                vacation: 5,
                created: datetime!(2025 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000001"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert_eq!(result.sales_person_id, sp_id);
    assert_eq!(result.year, TEST_YEAR);
    // entitled_days approx 25.0 (full year contract).
    assert!(
        (result.entitled_days - 25.0).abs() < 0.01,
        "entitled_days = {}",
        result.entitled_days
    );
    assert_eq!(result.carryover_days, 5);
    assert!(
        (result.used_days - 5.0).abs() < 0.01,
        "used_days = {}",
        result.used_days
    );
    assert!(
        (result.planned_days - 10.0).abs() < 0.01,
        "planned_days = {}",
        result.planned_days
    );
    // remaining = 25 + 5 - (5 + 10) = 15
    assert!(
        (result.remaining_days - 15.0).abs() < 0.01,
        "remaining_days = {}",
        result.remaining_days
    );
}

#[tokio::test]
async fn get_with_hr_succeeds() {
    let mut deps = build_dependencies();

    // HR-Path: HR succeeds (verify_user_is_sales_person may fail — `or()` filters it).
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));

    let sp_id = other_sales_person_id();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from([])));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 30)])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("HR-path get should succeed");

    assert_eq!(result.sales_person_id, sp_id);
    assert_eq!(result.carryover_days, 0);
    assert!(
        (result.used_days - 0.0).abs() < 0.01,
        "used_days = {}",
        result.used_days
    );
    assert!(
        (result.planned_days - 0.0).abs() < 0.01,
        "planned_days = {}",
        result.planned_days
    );
    // entitled approx 30 (full year contract), remaining = 30 - 0 = 30
    assert!(
        (result.entitled_days - result.remaining_days).abs() < 0.01,
        "entitled and remaining should match without used/planned/carryover: entitled={}, remaining={}",
        result.entitled_days,
        result.remaining_days
    );
}

#[tokio::test]
async fn get_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();

    // Both HR and verify_user_is_sales_person fail → Forbidden surfaces.
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));

    let svc = deps.build_service();
    let result = svc
        .get(
            other_sales_person_id(),
            TEST_YEAR,
            Authentication::Full,
            None,
        )
        .await;

    test_forbidden(&result);
}

// =========================================================================
// get_team
// =========================================================================

#[tokio::test]
async fn get_team_without_hr_is_forbidden() {
    let mut deps = build_dependencies();

    // get_team is HR-only; no verify_user_is_sales_person fallback.
    deps.permission_service
        .expect_check_permission()
        .with(mockall::predicate::eq(HR_PRIVILEGE), mockall::predicate::always())
        .returning(|_, _| Err(ServiceError::Forbidden));

    let svc = deps.build_service();
    let result = svc.get_team(TEST_YEAR, Authentication::Full, None).await;

    test_forbidden(&result);
}

#[tokio::test]
async fn get_team_aggregates_per_paid_sales_person() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let sp_a = default_sales_person_id();
    let sp_b = other_sales_person_id();
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(move |_, _| {
            Ok(Arc::from([
                paid_sales_person(sp_a, "Alice"),
                paid_sales_person(sp_b, "Bob"),
            ]))
        });

    // Pro Person eine Vacation-Periode in der Vergangenheit (3 Tage used je Person).
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(move |sp_id, _, _| {
            Ok(Arc::from([vacation_period(
                date!(2026 - 03 - 01),
                date!(2026 - 03 - 03),
                sp_id,
            )]))
        });

    // Pro Person ein Voll-Jahres-Vertrag, beide mit 20 Vacation-Tagen.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |sp_id, _, _| Ok(Arc::from([full_year_contract(sp_id, 20)])));

    // Pro Person ein Carryover von 2 Tagen.
    deps.carryover_service
        .expect_get_carryover()
        .returning(move |sp_id, year, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year,
                carryover_hours: 0.0,
                vacation: 2,
                created: datetime!(2025 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000002"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get_team(TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get_team should succeed");

    assert_eq!(result.len(), 2);
    let ids: Vec<Uuid> = result.iter().map(|b| b.sales_person_id).collect();
    assert!(ids.contains(&sp_a));
    assert!(ids.contains(&sp_b));
    for balance in result.iter() {
        assert_eq!(balance.year, TEST_YEAR);
        assert_eq!(balance.carryover_days, 2);
        assert!((balance.used_days - 3.0).abs() < 0.01);
        assert!((balance.planned_days - 0.0).abs() < 0.01);
        assert!((balance.entitled_days - 20.0).abs() < 0.01);
        // remaining = 20 + 2 - 3 - 0 = 19
        assert!(
            (balance.remaining_days - 19.0).abs() < 0.01,
            "remaining_days = {}",
            balance.remaining_days
        );
    }
}

// =========================================================================
// edge cases
// =========================================================================

#[tokio::test]
async fn get_with_no_active_contract_returns_zero_entitlement() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from([])));
    // No active contracts.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(
            default_sales_person_id(),
            TEST_YEAR,
            Authentication::Full,
            None,
        )
        .await
        .expect("get should succeed even without contracts");

    assert!(
        (result.entitled_days - 0.0).abs() < 0.01,
        "entitled_days = {}",
        result.entitled_days
    );
    assert_eq!(result.carryover_days, 0);
    assert!(
        (result.remaining_days - 0.0).abs() < 0.01,
        "remaining_days = {}",
        result.remaining_days
    );
}

#[tokio::test]
async fn get_year_without_carryover_returns_zero_carryover() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from([])));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 24)])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert_eq!(result.carryover_days, 0);
    // entitled approx 24, remaining = 24 - 0 - 0 = 24
    assert!(
        (result.entitled_days - result.remaining_days).abs() < 0.01,
        "entitled and remaining should match without used/planned/carryover"
    );
}
