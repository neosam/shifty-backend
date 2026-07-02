//! Server-side gate tests for the AVG-01 attendance-day metric (Phase 41-02).
//!
//! T-41-03 (D-AVG-05): `get_employee_attendance_statistics` runs the HR_PRIVILEGE
//!   check as its FIRST await — no work_details / report is fetched before auth.
//!   `attendance_statistics_requires_hr` proves this via `.times(0)` on all data mocks.
//! T-41-04 (D-AVG-05): non-flexible employees (no `is_dynamic`) are filtered
//!   server-side → `Ok(None)`; `attendance_statistics_returns_none_for_static`
//!   proves no report is fetched for them.
//!
//! Structural template: service_impl/src/test/reporting_holiday_auto_credit.rs.

use std::sync::Arc;

use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::MockExtraHoursService;
use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::permission::Authentication;
use service::MockPermissionService;
use service::ServiceError;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person_id, fixture_work_details_8h_mon_fri,
};

// ─── ReportingMocks / TestDeps (same pattern as reporting_holiday_auto_credit.rs) ──

struct ReportingMocks {
    extra_hours_service: MockExtraHoursService,
    shiftplan_report_service: MockShiftplanReportService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    sales_person_service: MockSalesPersonService,
    carryover_service: MockCarryoverService,
    permission_service: MockPermissionService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    absence_service: MockAbsenceService,
    transaction_dao: dao::MockTransactionDao,
    special_day_service: MockSpecialDayService,
    toggle_service: MockToggleService,
}

struct TestDeps;
impl ReportingServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ExtraHoursService = MockExtraHoursService;
    type ShiftplanReportService = MockShiftplanReportService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type SalesPersonService = MockSalesPersonService;
    type CarryoverService = MockCarryoverService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type AbsenceService = MockAbsenceService;
    type TransactionDao = dao::MockTransactionDao;
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}

impl ReportingMocks {
    fn new() -> Self {
        Self {
            extra_hours_service: MockExtraHoursService::new(),
            shiftplan_report_service: MockShiftplanReportService::new(),
            employee_work_details_service: MockEmployeeWorkDetailsService::new(),
            sales_person_service: MockSalesPersonService::new(),
            carryover_service: MockCarryoverService::new(),
            permission_service: MockPermissionService::new(),
            clock_service: MockClockService::new(),
            uuid_service: MockUuidService::new(),
            absence_service: MockAbsenceService::new(),
            transaction_dao: dao::MockTransactionDao::new(),
            special_day_service: MockSpecialDayService::new(),
            toggle_service: MockToggleService::new(),
        }
    }

    fn build(self) -> ReportingServiceImpl<TestDeps> {
        ReportingServiceImpl {
            extra_hours_service: Arc::new(self.extra_hours_service),
            shiftplan_report_service: Arc::new(self.shiftplan_report_service),
            employee_work_details_service: Arc::new(self.employee_work_details_service),
            sales_person_service: Arc::new(self.sales_person_service),
            carryover_service: Arc::new(self.carryover_service),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            absence_service: Arc::new(self.absence_service),
            transaction_dao: Arc::new(self.transaction_dao),
            special_day_service: Arc::new(self.special_day_service),
            toggle_service: Arc::new(self.toggle_service),
        }
    }
}

/// T-41-03 / D-AVG-05: HR gate is the FIRST operation. A non-HR context yields
/// `Forbidden` and NO data is fetched (work_details / shiftplan report never
/// touched — proven by `.times(0)`).
#[tokio::test]
async fn attendance_statistics_requires_hr() {
    let mut mocks = ReportingMocks::new();
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    // Proof: no data is fetched before auth.
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .times(0);
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .times(0);

    let service = mocks.build();
    let result = service
        .get_employee_attendance_statistics(&fixture_sales_person_id(), 2024, 25, Authentication::Full, None)
        .await;

    assert!(matches!(result.unwrap_err(), ServiceError::Forbidden));
}

/// T-41-04 / D-AVG-05: a static (non-flexible) employee is filtered server-side
/// → `Ok(None)`, and the report is never fetched (`.times(0)`).
#[tokio::test]
async fn attendance_statistics_returns_none_for_static() {
    let mut mocks = ReportingMocks::new();
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    // Static contract: is_dynamic == false.
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    // Proof: no report is aggregated for a non-flexible employee.
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .times(0);

    let service = mocks.build();
    let result = service
        .get_employee_attendance_statistics(&fixture_sales_person_id(), 2024, 25, Authentication::Full, None)
        .await;

    assert_eq!(result.unwrap(), None);
}
