//! Server-side gate tests for the attendance statistic (Phase 47 / RPT-01).
//!
//! The endpoint URL and gate semantics are unchanged from v2.1 (D-AVG-05):
//!
//! T-41-03 (D-AVG-05): `get_employee_attendance_statistics` runs the HR_PRIVILEGE
//!   check as its FIRST await — no work_details / report is fetched before auth.
//!   `attendance_statistics_requires_hr` proves this via `.times(0)` on all data mocks.
//! T-41-04 (D-AVG-05): non-flexible employees (no `is_dynamic`) are filtered
//!   server-side → `Ok(None)`; `attendance_statistics_returns_none_for_static`
//!   proves no report is fetched for them.
//! T-47-01 (RPT-01): flexible employee + HR caller produces a per-weekday
//!   distribution in the new v2.2 payload shape.
//!
//! Structural template: service_impl/src/test/reporting_holiday_auto_credit.rs.

use std::collections::BTreeMap;
use std::sync::Arc;

use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, MockExtraHoursService};
use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::{MockShiftplanReportService, ShiftplanReportDay};
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::permission::Authentication;
use service::MockPermissionService;
use service::ServiceError;
use shifty_utils::DayOfWeek;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
    fixture_work_details_dynamic_mon_fri,
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

/// RPT-02 (v2.2 post-ship): der is_dynamic-Filter wurde entfernt — die
/// Wochentag-Verteilung wird für ALLE Mitarbeiter berechnet. Ein static
/// (non-flexible) Employee bekommt jetzt `Ok(Some(stats))` statt `Ok(None)`;
/// der Report WIRD gefetched. Für einen leeren Report ist die Verteilung
/// all-zero (7 Einträge, alle count=0).
#[tokio::test]
async fn attendance_statistics_returns_some_for_static_after_rpt02() {
    let mut mocks = ReportingMocks::new();
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    // Non-flexible Fixture wird jetzt NICHT mehr abgefragt (kein is_dynamic-Filter),
    // aber Mock-Setup bleibt für Kompatibilität mit dem Report-Fetch-Pfad, der
    // find_by_sales_person_id weiter aufruft (get_report_for_employee_range).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    // Leerer Report — deckt den „nichts gearbeitet"-Fall ab, für den die
    // Verteilung all-zero sein muss.
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    mocks
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let service = mocks.build();
    let result = service
        .get_employee_attendance_statistics(&fixture_sales_person_id(), 2024, 25, Authentication::Full, None)
        .await
        .expect("attendance_statistics must succeed for non-flexible employee after RPT-02");

    let stats = result.expect("RPT-02: Some(...) for non-flexible employee");
    assert_eq!(
        stats.attendance_by_weekday.len(),
        7,
        "weekday distribution must always have 7 entries (Mo-So)"
    );
    assert!(
        stats.attendance_by_weekday.iter().all(|w| w.count == 0),
        "empty report → all counts must be 0, got {:?}",
        stats.attendance_by_weekday
    );
}

/// T-47-01 (D-47-BE / D-AVG-05): flexible employee (is_dynamic=true) + HR caller
/// → `Ok(Some(stats))` with `attendance_by_weekday` populated per weekday.
/// Exercises the full chain: HR gate → is_dynamic==true → get_report_for_employee
/// → weekday_attendance_distribution.
///
/// Fixture: 2 shiftplan days in KW23/2024 (Mon + Wed), each 4h
/// → Mon count=1, Wed count=1, all others 0.
#[tokio::test]
async fn attendance_statistics_returns_some_for_flexible() {
    let mut mocks = ReportingMocks::new();

    // HR gate (called in get_employee_attendance_statistics AND get_report_for_employee_range).
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    // is_dynamic filter (called in get_employee_attendance_statistics AND get_report_for_employee_range).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));

    // get_report_for_employee_range: sales person stubs.
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    // 2 shiftplan days in KW23/2024 → 2 distinct attendance days.
    let sp_id = fixture_sales_person_id();
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| {
            Ok(Arc::from(vec![
                ShiftplanReportDay {
                    sales_person_id: sp_id,
                    hours: 4.0,
                    year: 2024,
                    calendar_week: 23,
                    day_of_week: DayOfWeek::Monday,
                },
                ShiftplanReportDay {
                    sales_person_id: sp_id,
                    hours: 4.0,
                    year: 2024,
                    calendar_week: 23,
                    day_of_week: DayOfWeek::Wednesday,
                },
            ]))
        });

    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));

    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    // toggle → None: holiday auto-credit off (no special_day_service calls needed).
    mocks
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let service = mocks.build();
    let result = service
        .get_employee_attendance_statistics(
            &fixture_sales_person_id(),
            2024,
            25,
            Authentication::Full,
            None,
        )
        .await;

    let stats = result
        .expect("should succeed for flexible employee")
        .expect("should be Some for flexible employee");
    // v2.2 (RPT-01): attendance_by_weekday is always length 7, Mon..Sun.
    assert_eq!(stats.attendance_by_weekday.len(), 7);
    // KW23/2024 is a single counted calendar week → Mon count=1 share=1.0, Wed count=1 share=1.0.
    let mon = &stats.attendance_by_weekday[0];
    let wed = &stats.attendance_by_weekday[2];
    assert_eq!(mon.weekday, DayOfWeek::Monday);
    assert_eq!(mon.count, 1, "Mon count should be 1");
    assert_eq!(wed.weekday, DayOfWeek::Wednesday);
    assert_eq!(wed.count, 1, "Wed count should be 1");
    // Others must be zero.
    for (i, stat) in stats.attendance_by_weekday.iter().enumerate() {
        if i == 0 || i == 2 {
            continue;
        }
        assert_eq!(stat.count, 0, "weekday index {} should have count 0", i);
    }
    assert!(stats.counted_calendar_weeks >= 1, "should count at least KW23");
}
