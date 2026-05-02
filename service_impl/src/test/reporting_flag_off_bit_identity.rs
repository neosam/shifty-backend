//! Flag=off Bit-Identitaets-Test (REP-02 / SC-2). Verifiziert dass
//! `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` IDENTISCH zur
//! pre-Phase-2-ExtraHours-Aggregation sind, wenn das Feature-Flag
//! `absence_range_source_active` deaktiviert ist.
//!
//! Wichtig (Pitfall 2): Bit-Identitaet bezieht sich NUR auf die values-Map
//! des EmployeeReports — `snapshot_schema_version` ist auf der
//! Snapshot-Persistenzschicht und gehoert NICHT in diesen Test.

use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::feature_flag::MockFeatureFlagService;
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::ShiftyDate;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{fixture_sales_person, fixture_sales_person_id};

/// Mock-Bundle fuer ReportingServiceImpl. Default-Setup liefert:
/// - permission: HR ok
/// - sales_person: get + verify
/// - employee_work_details: leerer slice (kein Vertrag)
/// - shiftplan_report: leerer slice
/// - extra_hours: konfigurierbar via Closure
/// - carryover: None
struct ReportingMocks {
    extra_hours_service: MockExtraHoursService,
    shiftplan_report_service: MockShiftplanReportService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    sales_person_service: MockSalesPersonService,
    carryover_service: MockCarryoverService,
    permission_service: MockPermissionService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    feature_flag_service: MockFeatureFlagService,
    absence_service: MockAbsenceService,
    transaction_dao: dao::MockTransactionDao,
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
    type FeatureFlagService = MockFeatureFlagService;
    type AbsenceService = MockAbsenceService;
    type TransactionDao = dao::MockTransactionDao;
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
            feature_flag_service: MockFeatureFlagService::new(),
            absence_service: MockAbsenceService::new(),
            transaction_dao: dao::MockTransactionDao::new(),
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
            feature_flag_service: Arc::new(self.feature_flag_service),
            absence_service: Arc::new(self.absence_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

fn make_extra_hours(category: ExtraHoursCategory, amount: f32, day: time::Date) -> ExtraHours {
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sales_person_id(),
        amount,
        category,
        description: Arc::from(""),
        date_time: time::PrimitiveDateTime::new(day, time::Time::from_hms(9, 0, 0).unwrap()),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

#[tokio::test]
async fn test_flag_off_produces_identical_values_to_pre_phase2() {
    let mut mocks = ReportingMocks::new();

    // Permission: HR ok (verify_user_is_sales_person braucht somit nicht
    // erfolgreich zu sein, der HR-Zweig genuegt via `or` in
    // `get_report_for_employee_range`).
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    // Flag = OFF -> AbsenceService DARF NICHT aufgerufen werden.
    mocks
        .feature_flag_service
        .expect_is_enabled()
        .withf(|key, _, _| key == "absence_range_source_active")
        .returning(|_, _, _| Ok(false));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(0);

    // Sales-Person fetch.
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    // Working-Details: leer -> by_week ergibt leeren Vec, expected_hours = 0.
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![])));

    // Shiftplan-Report: leer.
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    // ExtraHours: Vacation 8h, SickLeave 4h, UnpaidLeave 2h.
    let extras = vec![
        make_extra_hours(ExtraHoursCategory::Vacation, 8.0, date!(2024 - 06 - 03)),
        make_extra_hours(ExtraHoursCategory::SickLeave, 4.0, date!(2024 - 06 - 04)),
        make_extra_hours(ExtraHoursCategory::UnpaidLeave, 2.0, date!(2024 - 06 - 05)),
    ];
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extras);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    // Carryover: None
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    // TransactionDao: passthrough
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let service = mocks.build();

    let report = service
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            ShiftyDate::from_ymd(2024, 6, 3).unwrap(),
            ShiftyDate::from_ymd(2024, 6, 9).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("Flag=off Pfad muss erfolgreich durchlaufen");

    // SC-2 Bit-Identitaets-Check: NUR die drei Aggregat-Felder.
    // snapshot_schema_version ist auf Snapshot-Layer, NICHT auf EmployeeReport
    // (Pitfall 2: Bit-Identitaet gilt nur fuer values-Map).
    assert_eq!(
        report.vacation_hours, 8.0,
        "Flag=off: vacation_hours muss aus ExtraHours-Vacation kommen (pre-Phase-2)"
    );
    assert_eq!(
        report.sick_leave_hours, 4.0,
        "Flag=off: sick_leave_hours muss aus ExtraHours-SickLeave kommen"
    );
    assert_eq!(
        report.unpaid_leave_hours, 2.0,
        "Flag=off: unpaid_leave_hours muss aus ExtraHours-UnpaidLeave kommen"
    );
}
