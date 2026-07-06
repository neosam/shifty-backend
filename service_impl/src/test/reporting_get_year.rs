//! Phase 52 Plan 04 (WOP-02) — Sanity-Tests fuer `ReportingService::get_year`.
//!
//! Diese Tests sind ein reiner Shape-Guard fuer die Vec-Ordering-Kontrakte aus
//! D-52-03 und die Off-by-one-Index-Semantik (WOP-02). Die volle Byte-Identitaet
//! zu 55x`get_week` wird in Wave 4/5 durch die Wave-1-Fixtures gegen
//! `get_weekly_summary` verifiziert — hier reicht:
//!
//! 1. `test_get_year_returns_all_weeks_in_year_ascending`: Vec-Laenge =
//!    `weeks_in_year(year)`, strikt aufsteigend, erste Woche = 1, letzte Woche =
//!    `weeks_in_year(year)`, Wochen ohne Bookings/Working-Hours haben ein leeres
//!    `Arc<[]>` (nicht ausgelassen).
//!
//! 2. `test_get_year_matches_get_week_for_arbitrary_week`: Fuer eine belegte
//!    Woche liefern `get_year(y)[week-1].1` und `get_week(y, week)` bit-exakt
//!    dieselben `ShortEmployeeReport`-Felder (via `to_bits()`-Vergleich, deckt
//!    IEEE-754 `-0.0` vs `+0.0`).
//!
//! Deckt Threat T-52-09 (Off-by-one) und die Vec-Kontrakt-Zusicherung aus D-52-03.

use std::collections::BTreeMap;
use std::sync::Arc;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::MockExtraHoursService;
use service::permission::Authentication;
use service::reporting::{ReportingService, ShortEmployeeReport};
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_work_details_8h_mon_fri,
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
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}

/// Baut ein minimales Mock-Setup fuer `get_year` / `get_week`.
///
/// - `work_details`: `fixture_work_details_8h_mon_fri` (Contract KW22-25/2024).
/// - `shiftplan_report_for_year` / `_for_week`: leerer Slice (keine Buchungen).
/// - `extra_hours find_by_year` / `find_by_week`: leerer Slice.
/// - `absence_service.derive_hours_for_range`: leere Map (kein Absence).
/// - `sales_person_service.get`: liefert `fixture_sales_person` (is_paid=true).
/// - `toggle_service.get_toggle_value`: `None` (kein holiday_auto_credit).
/// - `transaction_dao.use_transaction`: passthrough.
///
/// Alle Mocks nutzen `returning` (unlimitierte Aufrufe) — `get_year` ruft
/// `sales_person_service.get` und `absence_service.derive_hours_for_range`
/// pro Woche pro Person auf (52-53 Wochen x 1 Person).
fn build_service() -> ReportingServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    // Phase 52 Follow-Up (WOP-04): get_week / get_year now load sales persons
    // via `get_all` once and resolve the index in-memory.
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::new())));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));
    extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    // Phase 52 Follow-Up #2 (WOP-04): get_week / get_year bulk-load absences
    // once via `find_all` and compute per-(person, week) results in-memory.
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::absence::AbsencePeriod>::new())));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    ReportingServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        carryover_service: Arc::new(MockCarryoverService::new()),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        absence_service: Arc::new(absence_service),
        transaction_dao: Arc::new(transaction_dao),
        special_day_service: Arc::new(MockSpecialDayService::new()),
        toggle_service: Arc::new(toggle_service),
    }
}

/// D-52-03: Vec strikt aufsteigend nach `calendar_week` (1..=weeks_in_year).
/// Alle Wochen des Jahres erscheinen — auch die ohne Bookings/WH — mit
/// entsprechendem `Arc<[ShortEmployeeReport]>` (leer oder gefuellt).
#[tokio::test]
async fn test_get_year_returns_all_weeks_in_year_ascending() {
    let service = build_service();

    let result = service
        .get_year(2024, Authentication::Full, None)
        .await
        .expect("get_year must succeed for a clean year");

    let expected_weeks = time::util::weeks_in_year(2024_i32) as usize;
    assert_eq!(
        result.len(),
        expected_weeks,
        "get_year(2024) must return exactly weeks_in_year(2024) = {} entries",
        expected_weeks
    );

    // Erste Woche = KW 1, letzte Woche = weeks_in_year.
    assert_eq!(result[0].0, 1, "first entry must be week 1");
    assert_eq!(
        result.last().expect("non-empty").0,
        expected_weeks as u8,
        "last entry must be weeks_in_year({}) = {}",
        2024,
        expected_weeks
    );

    // Strikt aufsteigend.
    for pair in result.windows(2) {
        assert!(
            pair[0].0 < pair[1].0,
            "weeks must be strictly ascending, saw {} then {}",
            pair[0].0,
            pair[1].0
        );
    }

    // Woche 23/2024 hat einen aktiven Contract -> genau 1 Report (is_paid=true).
    // (Selbst Wochen ausserhalb KW22-25 liefern einen Zero-Report, weil die
    // Per-Person-Schleife auch bei `has_contract_row=false` pushed — dieselbe
    // Semantik wie in `get_week`. Das ist explizit D-52-08/09-konform.)
    let week23 = result
        .iter()
        .find(|(w, _)| *w == 23)
        .expect("week 23 must appear in the vec");
    assert_eq!(
        week23.1.len(),
        1,
        "week 23/2024 has an active contract for exactly 1 sales person"
    );
}

/// Zusatz-Guard: Wenn KEINE `EmployeeWorkDetails` existieren, liefert jede
/// Woche ein leeres `Arc<[]>` (D-52-03: leere Wochen mit leerem Slice).
#[tokio::test]
async fn test_get_year_empty_when_no_work_details() {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    // Phase 52 Follow-Up (WOP-04): get_year now calls get_all once for the
    // in-memory sales_person_index. Empty work-details → empty index is fine.
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::sales_person::SalesPerson>::new())));

    // Phase 52 Follow-Up #2 (WOP-04): absence bulk-load — empty for this test.
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::absence::AbsencePeriod>::new())));

    let service: ReportingServiceImpl<TestDeps> = ReportingServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        carryover_service: Arc::new(MockCarryoverService::new()),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        absence_service: Arc::new(absence_service),
        transaction_dao: Arc::new(transaction_dao),
        special_day_service: Arc::new(MockSpecialDayService::new()),
        toggle_service: Arc::new(toggle_service),
    };

    let result = service
        .get_year(2024, Authentication::Full, None)
        .await
        .expect("get_year must succeed with no employees");

    let expected_weeks = time::util::weeks_in_year(2024_i32) as usize;
    assert_eq!(result.len(), expected_weeks);
    for (w, reports) in result.iter() {
        assert!(
            reports.is_empty(),
            "week {} must have empty Arc<[]> when there are no work_details",
            w
        );
    }
}

/// T-52-09: Off-by-one-Guard — `get_year(y)[week - 1].1` muss bit-exakt
/// gleich `get_week(y, week).clone_slice()` sein. Deckt WOP-02 Byte-Identitaet
/// explizit ab.
#[tokio::test]
async fn test_get_year_matches_get_week_for_arbitrary_week() {
    // Zwei getrennte Service-Instanzen (identisches Mock-Setup) — mockall
    // Expectations sind pro Instanz, nicht global.
    let service_year = build_service();
    let service_week = build_service();

    let year_result = service_year
        .get_year(2024, Authentication::Full, None)
        .await
        .expect("get_year must succeed");

    // Waehle KW 23/2024 — Contract aktiv (KW22-25/2024), damit tatsaechlich
    // mindestens 1 Report entsteht.
    let target_week: u8 = 23;
    let year_slice: Arc<[ShortEmployeeReport]> = year_result
        .iter()
        .find(|(w, _)| *w == target_week)
        .map(|(_, reports)| reports.clone())
        .expect("week 23 must be present in get_year(2024)");

    let week_slice = service_week
        .get_week(2024, target_week, Authentication::Full, None)
        .await
        .expect("get_week must succeed");

    assert_eq!(
        year_slice.len(),
        week_slice.len(),
        "get_year[week-1] and get_week must have identical length"
    );

    for (i, (from_year, from_week)) in year_slice.iter().zip(week_slice.iter()).enumerate() {
        assert_eq!(
            from_year.sales_person.id, from_week.sales_person.id,
            "sales_person.id mismatch at index {}",
            i
        );
        // Bit-exakter Float-Vergleich (IEEE-754 sign-of-zero-safe).
        assert_eq!(
            from_year.balance_hours.to_bits(),
            from_week.balance_hours.to_bits(),
            "balance_hours bit mismatch at index {}: year={} vs week={}",
            i,
            from_year.balance_hours,
            from_week.balance_hours
        );
        assert_eq!(
            from_year.dynamic_hours.to_bits(),
            from_week.dynamic_hours.to_bits(),
            "dynamic_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.expected_hours.to_bits(),
            from_week.expected_hours.to_bits(),
            "expected_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.overall_hours.to_bits(),
            from_week.overall_hours.to_bits(),
            "overall_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.vacation_hours.to_bits(),
            from_week.vacation_hours.to_bits(),
            "vacation_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.sick_leave_hours.to_bits(),
            from_week.sick_leave_hours.to_bits(),
            "sick_leave_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.holiday_hours.to_bits(),
            from_week.holiday_hours.to_bits(),
            "holiday_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.unavailable_hours.to_bits(),
            from_week.unavailable_hours.to_bits(),
            "unavailable_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.unpaid_leave_hours.to_bits(),
            from_week.unpaid_leave_hours.to_bits(),
            "unpaid_leave_hours bit mismatch at index {}",
            i
        );
        assert_eq!(
            from_year.volunteer_hours.to_bits(),
            from_week.volunteer_hours.to_bits(),
            "volunteer_hours bit mismatch at index {}",
            i
        );
    }

    // Zusaetzlich: Index-Semantik — result[week-1].0 == week.
    // (Da alle Wochen aufsteigend 1..=N enthalten sind, gilt result[w-1].0 == w.)
    assert_eq!(
        year_result[(target_week - 1) as usize].0,
        target_week,
        "off-by-one guard: result[week-1].0 must equal week"
    );
}
