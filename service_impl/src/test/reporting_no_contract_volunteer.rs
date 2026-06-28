//! Tests fuer die no-contract-Ehrenamt-Klassifikation (quick-260624-ujk).
//!
//! User-Regel: Eine Kalenderwoche OHNE EmployeeWorkDetails-Zeile bedeutet, dass
//! der Mitarbeiter in dieser Woche KEINEN Vertrag hat. Geleistete Shiftplan-Stunden
//! zaehlen dann als Ehrenamt (volunteer_hours), NICHT als Soll=Ist-neutralisiertes
//! "bezahltes Arbeiten". Kein Soll, kein Balance-Einfluss, Saldo bleibt +-0.
//!
//! Vier Testfaelle:
//! - Fall A: KW OHNE Vertragszeile, 30h Shiftplan => volunteer=30, overall=0, balance=0.
//! - Fall B: Dynamische Zeile (is_dynamic=true, expected weighted=0), 30h Shiftplan =>
//!   Soll=Ist unveraendert (overall=30, expected=30, balance=0, volunteer=0).
//! - Fall C: Zeile mit expected=40h, 30h Shiftplan => Normal (expected=40, overall=30, balance=-10).
//! - Fall D: Konsistenz-Check — gleiche no-contract-Daten durch get_reports_for_all_employees
//!   (Summary-Pfad) liefern identische volunteer_hours=30, overall=0, balance=0.
//!   Hinweis zu get_week: `all_for_week` liefert nur Persons MIT Vertragszeile fuer
//!   die KW — Personen OHNE Zeile werden in get_week gar nicht iteriert. Der no-contract-
//!   Fall existiert in get_week daher strukturell nicht; stattdessen verifizieren wir
//!   die Detail-vs-Summary-Konsistenz (hours_per_week vs. get_reports_for_all_employees).

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

/// 30h Shiftplan-Stunden in KW23/2024, auf drei Tage verteilt.
fn shiftplan_30h_kw23() -> Arc<[ShiftplanReportDay]> {
    Arc::from(vec![
        ShiftplanReportDay {
            sales_person_id: fixture_sales_person_id(),
            hours: 10.0,
            year: 2024,
            calendar_week: 23,
            day_of_week: DayOfWeek::Monday,
        },
        ShiftplanReportDay {
            sales_person_id: fixture_sales_person_id(),
            hours: 10.0,
            year: 2024,
            calendar_week: 23,
            day_of_week: DayOfWeek::Tuesday,
        },
        ShiftplanReportDay {
            sales_person_id: fixture_sales_person_id(),
            hours: 10.0,
            year: 2024,
            calendar_week: 23,
            day_of_week: DayOfWeek::Wednesday,
        },
    ])
}

/// Baut eine ReportingServiceImpl fuer get_report_for_employee_range-Tests.
///
/// `work_details`: slice der EmployeeWorkDetails-Records (leer = kein Vertrag).
fn build_detail_service(
    work_details: Vec<EmployeeWorkDetails>,
    shiftplan: Arc<[ShiftplanReportDay]>,
) -> ReportingServiceImpl<TestDeps> {
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
        .returning(move |_, _, _| Ok(Arc::from(work_details.clone())));

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

/// Baut eine ReportingServiceImpl fuer get_reports_for_all_employees-Tests.
///
/// `work_details`: alle EmployeeWorkDetails fuer alle Mitarbeiter.
fn build_summary_service(
    work_details: Vec<EmployeeWorkDetails>,
    shiftplan: Arc<[ShiftplanReportDay]>,
) -> ReportingServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(Arc::from(work_details.clone())));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| Ok(shiftplan.clone()));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year()
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
    let mut toggle_service_b = MockToggleService::new();
    toggle_service_b
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
        toggle_service: Arc::new(toggle_service_b),
    }
}

/// Fall A: KW OHNE Vertragszeile, 30h Shiftplan.
/// Erwartet: volunteer_hours == 30, overall_hours == 0, balance_hours == 0, expected_hours == 0.
/// Die Shiftplan-Stunden werden nicht als bezahlte Leistung erfasst, sondern als Ehrenamt.
#[tokio::test]
async fn fall_a_no_contract_shiftplan_goes_to_volunteer() {
    // Kein EmployeeWorkDetails-Record — kein Vertrag fuer diese KW.
    let service = build_detail_service(vec![], shiftplan_30h_kw23());

    let report = service
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            // KW23/2024: Mo 2024-06-03 .. So 2024-06-09
            ShiftyDate::from_ymd(2024, 6, 3).unwrap(),
            ShiftyDate::from_ymd(2024, 6, 9).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("report must succeed");

    assert!(
        (report.volunteer_hours - 30.0).abs() < 0.01,
        "Fall A: no-contract shiftplan (30h) must land in volunteer_hours, got {}",
        report.volunteer_hours
    );
    assert!(
        report.overall_hours.abs() < 0.01,
        "Fall A: overall_hours must be 0 (no paid contract), got {}",
        report.overall_hours
    );
    assert!(
        report.balance_hours.abs() < 0.01,
        "Fall A: balance_hours must be 0 (expected=0, overall=0), got {}",
        report.balance_hours
    );
    assert!(
        report.expected_hours.abs() < 0.01,
        "Fall A: expected_hours must be 0 (no contract), got {}",
        report.expected_hours
    );
}

/// Fall B: Dynamische Zeile (is_dynamic=true) — Soll=Ist-Neutralisierung bleibt unveraendert.
/// Erwartet: overall_hours == 30, expected_hours == 30, balance_hours == 0, volunteer_hours == 0.
/// Eine vorhandene Zeile mit is_dynamic=true ist kein no-contract-Fall.
#[tokio::test]
async fn fall_b_dynamic_contract_keeps_sol_ist_behavior() {
    let dynamic_work_details = EmployeeWorkDetails {
        is_dynamic: true,
        ..fixture_work_details_8h_mon_fri()
    };
    let service = build_detail_service(vec![dynamic_work_details], shiftplan_30h_kw23());

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

    // Dynamischer Vertrag (Zeile vorhanden, expected weighted=0): Soll=Ist.
    // expected_hours wird auf shiftplan_hours gesetzt, balance = 0.
    assert!(
        (report.overall_hours - 30.0).abs() < 0.01,
        "Fall B: dynamic contract overall_hours must be 30 (Soll=Ist), got {}",
        report.overall_hours
    );
    assert!(
        (report.expected_hours - 30.0).abs() < 0.01,
        "Fall B: dynamic contract expected_hours must equal overall (30h), got {}",
        report.expected_hours
    );
    assert!(
        report.balance_hours.abs() < 0.01,
        "Fall B: dynamic contract balance_hours must be 0 (Soll=Ist), got {}",
        report.balance_hours
    );
    assert!(
        report.volunteer_hours.abs() < 0.01,
        "Fall B: dynamic contract must not produce volunteer_hours, got {}",
        report.volunteer_hours
    );
}

/// Fall C: Zeile mit expected=40h, 30h Shiftplan — normales Verhalten unveraendert.
/// Erwartet: expected_hours == 40, overall_hours == 30, balance_hours == -10, volunteer_hours == 0.
#[tokio::test]
async fn fall_c_normal_contract_regression() {
    // fixture_work_details_8h_mon_fri: 40h/Woche Mo-Fr, KW22-25/2024.
    let service = build_detail_service(
        vec![fixture_work_details_8h_mon_fri()],
        shiftplan_30h_kw23(),
    );

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
        (report.expected_hours - 40.0).abs() < 0.01,
        "Fall C: expected_hours must be 40 (contract), got {}",
        report.expected_hours
    );
    assert!(
        (report.overall_hours - 30.0).abs() < 0.01,
        "Fall C: overall_hours must be 30 (shiftplan), got {}",
        report.overall_hours
    );
    assert!(
        (report.balance_hours - (-10.0)).abs() < 0.01,
        "Fall C: balance_hours must be -10 (30-40), got {}",
        report.balance_hours
    );
    assert!(
        report.volunteer_hours.abs() < 0.01,
        "Fall C: no volunteer_hours with normal contract, got {}",
        report.volunteer_hours
    );
}

/// Fall D: Konsistenz-Check Detail-Report vs. Summary-Report.
///
/// Beide Report-Pfade (get_report_for_employee_range via hours_per_week UND
/// get_reports_for_all_employees) muessen fuer die gleichen no-contract-Daten
/// identische Werte liefern:
/// - volunteer_hours == 30, overall_hours == 0, balance_hours == 0.
///
/// Hinweis: get_week iteriert nur Persons MIT Vertragszeile (all_for_week-Semantik),
/// daher kein separater get_week-Testfall fuer no-contract. Die Detail-vs-Summary-
/// Konsistenz ist die massgebliche Verifikation.
#[tokio::test]
async fn fall_d_detail_vs_summary_consistency_no_contract() {
    // Detail-Report (hours_per_week-Pfad)
    let detail_service = build_detail_service(vec![], shiftplan_30h_kw23());
    let detail_report = detail_service
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            ShiftyDate::from_ymd(2024, 6, 3).unwrap(),
            ShiftyDate::from_ymd(2024, 6, 9).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("detail report must succeed");

    // Summary-Report (get_reports_for_all_employees-Pfad)
    // Shiftplan muss auch Jahr-Range abdecken (Jahresanfang bis KW23-Ende).
    // Da der Summary-Report das gesamte Jahr iteriert, liefern wir dieselben
    // Shiftplan-Stunden in KW23 — restliche Wochen sind leer (kein Shiftplan-Record).
    let summary_service = build_summary_service(vec![], shiftplan_30h_kw23());
    let summary_reports = summary_service
        .get_reports_for_all_employees(
            2024,
            23, // until_week = KW23
            Authentication::Full,
            None,
        )
        .await
        .expect("summary report must succeed");

    assert_eq!(
        summary_reports.len(),
        1,
        "Fall D: summary must contain exactly 1 employee"
    );
    let summary = &summary_reports[0];

    // Beide Pfade muessen konsistente volunteer_hours liefern.
    assert!(
        (detail_report.volunteer_hours - 30.0).abs() < 0.01,
        "Fall D (detail): volunteer_hours must be 30, got {}",
        detail_report.volunteer_hours
    );
    assert!(
        (summary.volunteer_hours - 30.0).abs() < 0.01,
        "Fall D (summary): volunteer_hours must be 30, got {}",
        summary.volunteer_hours
    );

    // Beide Pfade: overall_hours == 0 (keine bezahlte Leistung).
    assert!(
        detail_report.overall_hours.abs() < 0.01,
        "Fall D (detail): overall_hours must be 0, got {}",
        detail_report.overall_hours
    );
    assert!(
        summary.overall_hours.abs() < 0.01,
        "Fall D (summary): overall_hours must be 0, got {}",
        summary.overall_hours
    );

    // Beide Pfade: balance_hours == 0 (kein Soll, kein Ist in paid axes).
    assert!(
        detail_report.balance_hours.abs() < 0.01,
        "Fall D (detail): balance_hours must be 0, got {}",
        detail_report.balance_hours
    );
    assert!(
        summary.balance_hours.abs() < 0.01,
        "Fall D (summary): balance_hours must be 0, got {}",
        summary.balance_hours
    );
}
