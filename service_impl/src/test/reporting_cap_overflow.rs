//! Regression-Test fuer den Detail-Report-Cap-Leak (Debug-Session
//! `report-ehrenamt-gesamtstunden`).
//!
//! Bug: In `get_report_for_employee_range` (reporting.rs) wurde `overall_hours`
//! (und `balance_hours`) aus dem ROHEN, UNGEDECKELTEN `shiftplan_hours` (Z.577)
//! berechnet, obwohl der per-Woche GEDECKELTE Wert (`shiftplan_hours_by_week`,
//! via `apply_weekly_cap`) bereits existierte. Folge: bei
//! `cap_planned_hours_to_expected = true` floss der Cap-Ueberlauf
//! (= auto_volunteer / Ehrenamt-Anteil) faelschlich in die Gesamtstunden ein.
//!
//! Die Summary (`get_reports_for_all_employees`) deckelt korrekt — der
//! Detail-Report muss damit konsistent sein.
//!
//! Phase-15 D-01: committed_voluntary / freiwillige Kapazitaet ist eine reine
//! Achse-B-Groesse und darf NICHT in Achse A (reporting.rs Balance/Overall)
//! erscheinen. Cap-Ueberlauf gehoert in `volunteer_hours`, nicht in `overall`.

use std::collections::BTreeMap;
use std::sync::Arc;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::MockExtraHoursService;
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::{MockShiftplanReportService, ShiftplanReportDay};
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
};

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
    // Phase 25: holiday derive-on-read deps.
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}

/// 8h/Tag Mo-Fr (expected 40h/Woche), KW22-25/2024, cap_planned_hours_to_expected=true.
fn capped_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        cap_planned_hours_to_expected: true,
        ..fixture_work_details_8h_mon_fri()
    }
}

/// Erzeugt einen Shiftplan-Tag fuer KW23/2024.
fn shiftplan_day(day_of_week: DayOfWeek, hours: f32) -> ShiftplanReportDay {
    ShiftplanReportDay {
        sales_person_id: fixture_sales_person_id(),
        hours,
        year: 2024,
        calendar_week: 23,
        day_of_week,
    }
}

/// Baut die ReportingService-Impl mit den noetigen Mocks.
/// shiftplan: 5 Tage x 10h = 50h in KW23 (expected = 40h, cap aktiv).
fn build_capped_service() -> ReportingServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![capped_work_details()])));

    // 5 Werktage Mo-Fr x 10h = 50h shiftplan in KW23/2024.
    let shiftplan: Arc<[ShiftplanReportDay]> = Arc::from(vec![
        shiftplan_day(DayOfWeek::Monday, 10.0),
        shiftplan_day(DayOfWeek::Tuesday, 10.0),
        shiftplan_day(DayOfWeek::Wednesday, 10.0),
        shiftplan_day(DayOfWeek::Thursday, 10.0),
        shiftplan_day(DayOfWeek::Friday, 10.0),
    ]);
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| Ok(shiftplan.clone()));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let mut carryover_service = MockCarryoverService::new();
    carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // Phase 25: toggle automation off by default (no value = no holiday auto-credit).
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    ReportingServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        carryover_service: Arc::new(carryover_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        absence_service: Arc::new(absence_service),
        transaction_dao: Arc::new(transaction_dao),
        special_day_service: Arc::new(MockSpecialDayService::new()),
        toggle_service: Arc::new(toggle_service),
    }
}

/// Detail-Report: cap aktiv, shiftplan (50h) > expected (40h).
/// Der Cap-Ueberlauf (10h) muss in volunteer_hours landen und darf NICHT in
/// overall_hours / balance_hours / shiftplan_hours erscheinen.
#[tokio::test]
async fn capped_overflow_does_not_leak_into_overall_hours() {
    let service = build_capped_service();
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
        .expect("report must succeed");

    // overall_hours muss auf expected (40h) gedeckelt sein, NICHT 50h.
    assert!(
        (report.overall_hours - 40.0).abs() < 0.01,
        "overall_hours must be capped at expected (40h), was {}",
        report.overall_hours
    );
    // shiftplan_hours (Display) muss ebenfalls gedeckelt sein.
    assert!(
        (report.shiftplan_hours - 40.0).abs() < 0.01,
        "shiftplan_hours must be capped at expected (40h), was {}",
        report.shiftplan_hours
    );
    // balance = overall(40) - expected(40) = 0, nicht +10.
    assert!(
        report.balance_hours.abs() < 0.01,
        "balance_hours must be 0 (overall == expected after cap), was {}",
        report.balance_hours
    );
    // Der Ueberlauf (10h) gehoert in volunteer_hours.
    assert!(
        (report.volunteer_hours - 10.0).abs() < 0.01,
        "cap overflow (10h) must land in volunteer_hours, was {}",
        report.volunteer_hours
    );
}

/// Negativ-Kontrolle: ohne Cap (cap=false) bleiben die rohen Stunden in
/// overall_hours — Backward-Compat-Garantie.
#[tokio::test]
async fn uncapped_overflow_stays_in_overall_hours() {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    // cap=false (fixture default).
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));

    let shiftplan: Arc<[ShiftplanReportDay]> = Arc::from(vec![
        shiftplan_day(DayOfWeek::Monday, 10.0),
        shiftplan_day(DayOfWeek::Tuesday, 10.0),
        shiftplan_day(DayOfWeek::Wednesday, 10.0),
        shiftplan_day(DayOfWeek::Thursday, 10.0),
        shiftplan_day(DayOfWeek::Friday, 10.0),
    ]);
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| Ok(shiftplan.clone()));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let mut carryover_service = MockCarryoverService::new();
    carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // Phase 25: toggle automation off by default.
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    let service = ReportingServiceImpl::<TestDeps> {
        extra_hours_service: Arc::new(extra_hours_service),
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        carryover_service: Arc::new(carryover_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        absence_service: Arc::new(absence_service),
        transaction_dao: Arc::new(transaction_dao),
        special_day_service: Arc::new(MockSpecialDayService::new()),
        toggle_service: Arc::new(toggle_service),
    };

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
        .expect("report must succeed");

    assert!(
        (report.overall_hours - 50.0).abs() < 0.01,
        "without cap, overall_hours stays raw (50h), was {}",
        report.overall_hours
    );
    assert!(
        report.volunteer_hours.abs() < 0.01,
        "without cap, no auto volunteer hours, was {}",
        report.volunteer_hours
    );
}
