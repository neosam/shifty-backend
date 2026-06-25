//! Additiver-Merge-Test (Phase 8.4 / SC-1). Verifiziert dass
//! `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` aus BEIDEN Quellen
//! additiv summiert werden: lebende `extra_hours` (deleted IS NULL) PLUS
//! `AbsenceService::derive_hours_for_range`. Es gibt keinen globalen
//! Quellen-Schalter mehr (M-03) — der Flag `absence_range_source_active` wird
//! im Reporting-Pfad nicht mehr gelesen.
//!
//! WICHTIG (Test-first): Diese Tests sind RED bis Plan 02 den Produktionscode
//! in `reporting.rs` auf den additiven Merge umstellt (Flag-Read entfernt,
//! `derive_hours_for_range` unbedingt aufgerufen, beide Quellen addiert).
//!
//! Kritische Mock-Erwartungen pro Szenario:
//! - Kein `feature_flag_service` mehr — die FeatureFlagService-Dep wurde aus
//!   `ReportingServiceImpl` entfernt (Phase 8.4, M-03).
//! - `absence_service.expect_derive_hours_for_range()` IMMER (auch bei leeren
//!   extra_hours) — kein `times(0)`-Trap.
//! - `extra_hours_service.expect_find_by_sales_person_id_and_year_range()`
//!   immer erwartet (kann leeren Slice liefern).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::{AbsenceCategory, MockAbsenceService, ResolvedAbsence};
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::ShiftyDate;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
    fixture_work_details_dynamic_mon_fri,
};

// ─── Hilfsfunktionen fuer CVC-10-is_paid-Gate-Tests ──────────────────────────

/// Deterministische Id fuer eine unbezahlte Freiwilligen-Person.
fn unpaid_person_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_BEEF)
}

/// Unbezahlte Freiwillige: is_paid=false, expected_hours=0, committed_voluntary=5.
/// Repraesentiert den neuen Typ "rein freiwilliger Helfer" (D-04 / Phase 17).
fn unpaid_volunteer_sales_person() -> service::sales_person::SalesPerson {
    service::sales_person::SalesPerson {
        id: unpaid_person_id(),
        name: Arc::from("Unpaid Volunteer"),
        background_color: Arc::from("#FF0000"),
        is_paid: Some(false),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Work-Details fuer die unbezahlte Person: expected_hours=0, committed_voluntary=5,
/// gueltig in KW 23/2024 (Mo-Fr).
fn unpaid_volunteer_work_details() -> service::employee_work_details::EmployeeWorkDetails {
    service::employee_work_details::EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BEEF_0010),
        sales_person_id: unpaid_person_id(),
        expected_hours: 0.0,
        from_day_of_week: shifty_utils::DayOfWeek::Monday,
        from_calendar_week: 22,
        from_year: 2024,
        to_day_of_week: shifty_utils::DayOfWeek::Sunday,
        to_calendar_week: 25,
        to_year: 2024,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        committed_voluntary: 5.0,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 0,
        created: Some(time::macros::datetime!(2024 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

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

/// Setzt die Standard-Boilerplate-Mocks, die in JEDEM additiven Szenario
/// identisch sind (permission, sales_person, work_details, shiftplan_report,
/// carryover, transaction) und macht explizit, dass der Feature-Flag nicht
/// mehr gelesen wird (`is_enabled().times(0)`).
fn setup_common_mocks(mocks: &mut ReportingMocks) {
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    // employee_work_details + shiftplan_report: leer (isoliert Stunden-Aggregation).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    // carryover: None.
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    // transaction: passthrough.
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // Phase 8.4: KEIN Flag-Read mehr im Reporting-Pfad (M-03) — die
    // FeatureFlagService-Dep wurde aus ReportingServiceImpl entfernt.
}

async fn run_report(service: ReportingServiceImpl<TestDeps>) -> service::reporting::EmployeeReport {
    service
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            ShiftyDate::from_ymd(2024, 6, 3).unwrap(),
            ShiftyDate::from_ymd(2024, 6, 9).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("additiver Reporting-Pfad muss erfolgreich durchlaufen")
}

/// Szenario 1: Nur `extra_hours` vorhanden (keine absence_period).
/// derive_hours_for_range -> leere BTreeMap.
/// extra_hours -> [Vacation 8h, SickLeave 4h, UnpaidLeave 2h].
/// Erwartung: vacation=8, sick=4, unpaid=2 (= ExtraHours-Summe + 0).
#[tokio::test]
async fn test_additive_only_extra_hours() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    // absence_period leer — derive_hours_for_range wird trotzdem aufgerufen.
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

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

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "nur extra_hours: vacation = 8h ExtraHours + 0h absence_period"
    );
    assert_eq!(
        report.sick_leave_hours, 4.0,
        "nur extra_hours: sick_leave = 4h ExtraHours + 0h absence_period"
    );
    assert_eq!(
        report.unpaid_leave_hours, 2.0,
        "nur extra_hours: unpaid_leave = 2h ExtraHours + 0h absence_period"
    );
}

/// Szenario 2: Nur absence_period vorhanden (keine extra_hours).
/// derive_hours_for_range -> {2024-06-03: Vacation 8h, 2024-06-04: SickLeave 8h}.
/// extra_hours -> leerer Slice.
/// Erwartung: vacation=8, sick=8, unpaid=0.
#[tokio::test]
async fn test_additive_only_absence_period() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    derived.insert(
        date!(2024 - 06 - 04),
        ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    // extra_hours leer.
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "nur absence_period: vacation = 0h ExtraHours + 8h absence_period"
    );
    assert_eq!(
        report.sick_leave_hours, 8.0,
        "nur absence_period: sick_leave = 0h ExtraHours + 8h absence_period"
    );
    assert_eq!(
        report.unpaid_leave_hours, 0.0,
        "nur absence_period: unpaid_leave = 0 (kein UnpaidLeave aktiv)"
    );
}

/// Szenario 3 (Kern-SC-1): Beide Quellen, distinkte Tage -> additive Summe.
/// derive_hours_for_range -> {2024-06-03: Vacation 8h}.
/// extra_hours -> [Vacation 4h am 2024-06-05 (anderer Tag, lebt noch)].
/// Erwartung: vacation = 12 (4h ExtraHours + 8h absence_period).
#[tokio::test]
async fn test_additive_both_sources_distinct_days() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    let extras = vec![make_extra_hours(
        ExtraHoursCategory::Vacation,
        4.0,
        date!(2024 - 06 - 05),
    )];
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extras);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 12.0,
        "beide Quellen distinkte Tage: vacation = 4h ExtraHours + 8h absence_period = 12h"
    );
}

/// Szenario 4: Konvertierte (soft-deleted) Rows zählen nicht doppelt.
/// Die per-row-Invariante (D-02): konvertierte extra_hours sind via
/// `deleted IS NULL` bereits aus dem DAO-Load ausgeschlossen — der Mock liefert
/// daher schlicht [] für den konvertierten Tag. Kein neuer DAO-Code nötig.
/// extra_hours -> leerer Slice.
/// derive_hours_for_range -> {2024-06-03: Vacation 8h}.
/// Erwartung: vacation = 8 (nur einmal gezählt).
#[tokio::test]
async fn test_additive_converted_rows_not_double_counted() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    // Konvertierte Row ist soft-deleted -> faellt per `deleted IS NULL` im DAO
    // heraus -> Mock liefert leeren Slice.
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "konvertierte Rows zählen nicht doppelt: vacation = 8h (nur aus absence_period)"
    );
}

// ─── Tests fuer Schwester-Methoden (Plan 08.4-03) ────────────────────────────

/// Test A — get_reports_for_all_employees: beide Quellen, additive Summe.
/// extra_hours -> [Vacation 4h am 2024-06-05].
/// derive_hours_for_range -> {2024-06-03: Vacation 8h}.
/// Erwartung: vacation = 12.0 (4h extra_hours + 8h absence_period).
#[tokio::test]
async fn test_all_employees_additive_merge() {
    let mut mocks = ReportingMocks::new();

    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    // fixture_work_details_8h_mon_fri gibt expected_hours > 0 fuer KW23/2024 zurueck,
    // damit der else-Zweig in get_reports_for_all_employees greift und vacation_hours
    // aus extra_hours_array gelesen wird.
    mocks
        .employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| {
            Ok(Arc::from(vec![make_extra_hours(
                ExtraHoursCategory::Vacation,
                4.0,
                date!(2024 - 06 - 05),
            )]))
        });
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    let service = mocks.build();
    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein");

    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].vacation_hours, 12.0,
        "get_reports_for_all_employees: 4h extra_hours + 8h absence_period = 12h additiv"
    );
}

/// Test B — get_week: beide Quellen, additive Summe.
/// extra_hours -> [SickLeave 3h am 2024-06-04].
/// derive_hours_for_range -> {2024-06-05: SickLeave 8h}.
/// Erwartung: sick_leave = 11.0 (3h extra_hours + 8h absence_period).
#[tokio::test]
async fn test_get_week_additive_merge() {
    let mut mocks = ReportingMocks::new();

    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));

    let sick_entry = make_extra_hours(
        ExtraHoursCategory::SickLeave,
        3.0,
        date!(2024 - 06 - 04),
    );
    let extras: Arc<[ExtraHours]> = Arc::from(vec![sick_entry]);
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(move |_, _, _, _| Ok(extras.clone()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 05),
        ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    let service = mocks.build();
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein");

    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].sick_leave_hours, 11.0,
        "get_week: 3h extra_hours + 8h absence_period = 11h SickLeave additiv"
    );
}

/// Test C — WR-02 Gleichtags-Overlap: lebende extra_hours + absence_period am selben Tag
/// summieren additiv zu 16h (kein Dedup, M-02).
/// extra_hours -> [Vacation 8h am 2024-06-03 (lebt — nicht soft-deleted)].
/// derive_hours_for_range -> {2024-06-03: Vacation 8h (gleicher Tag)}.
/// Erwartung: vacation = 16.0.
#[tokio::test]
async fn test_same_day_overlap_additive_no_dedup() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    // Gleicher Tag 2024-06-03 wie die derived Vacation — lebende (nicht soft-deleted) Zeile.
    let extras: Arc<[ExtraHours]> = Arc::from(vec![make_extra_hours(
        ExtraHoursCategory::Vacation,
        8.0,
        date!(2024 - 06 - 03),
    )]);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras.clone()));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 16.0,
        "Gleichtags-Koexistenz: 8h extra_hours + 8h absence_period = 16h additiv (kein Dedup, M-02)"
    );
}

/// Test D — IN-03 Year-Bounds: get_reports_for_all_employees uebergibt exakt
/// [first_day_in_year(2024)..until_week-Sonntag] an derive_hours_for_range.
/// Woche-0 (Vorjahr) und Jahresend-Overflow leaken keine Absence-Stunden.
#[tokio::test]
async fn test_all_employees_year_bounds_no_leak() {
    let mut mocks = ReportingMocks::new();

    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    mocks
        .employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(vec![])));
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // Assertiert die uebergebenen Bounds direkt im Mock (IN-03).
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(|from, to, _, _, _| {
            assert_eq!(
                from,
                time::macros::date!(2024 - 01 - 01),
                "from MUSS first_day_in_year(2024) sein, nicht Vorjahres-Woche-0"
            );
            assert!(
                to <= time::macros::date!(2024 - 12 - 31),
                "to darf nicht ins Folgejahr leaken, war: {:?}",
                to
            );
            Ok(BTreeMap::new())
        });

    let service = mocks.build();
    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein");

    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].vacation_hours, 0.0,
        "Year-Bounds-Test: vacation_hours = 0.0 (beide Quellen leer)"
    );
}

// ─── Balance-Parity-Test (Plan 08.4-04) ─────────────────────────────────────

/// Baut ein Mock-Setup fuer get_report_for_employee_range mit echter Work-Details-
/// Fixture (8h Mo-Fr KW22-25/2024) und ohne Shiftplan-Stunden.
/// extra_hours_list: was find_by_sales_person_id_and_year_range liefert.
/// derived_map: was derive_hours_for_range liefert.
fn build_parity_service(
    extra_hours_list: Vec<ExtraHours>,
    derived_map: BTreeMap<time::Date, ResolvedAbsence>,
) -> ReportingServiceImpl<TestDeps> {
    let mut mocks = ReportingMocks::new();

    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    // Work-Details mit echten Vertragsstunden (8h Mo-Fr KW22-25/2024).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));

    // Kein Shiftplan -> overall_hours aus extra_work = 0.
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    // Kein Carryover.
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    // Transaction passthrough.
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // extra_hours-Mock.
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extra_hours_list);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    // derive_hours_for_range-Mock.
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived_map.clone()));

    mocks.build()
}

/// Balance-Parity-Test: eine reine absence_period-Vacation-Buchung (8h an 2024-06-03)
/// muss balance_hours + expected_hours IDENTISCH bewegen wie das aequivalente
/// extra_hours-Vacation-Eintraeg (Gap 2 / WR-01).
///
/// Sanity-Check: expected_hours MUSS unter die vollen 40h gedrueckt werden (8h reduziert).
#[tokio::test]
async fn test_balance_parity_absence_period_vs_extra_hours() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // Lauf 1: extra_hours-Vacation 8h, kein derived.
    let extra_vacation = make_extra_hours(ExtraHoursCategory::Vacation, 8.0, vacation_day);
    let service_extra = build_parity_service(vec![extra_vacation], BTreeMap::new());
    let report_extra = run_report(service_extra).await;

    // Lauf 2: kein extra_hours, derived Vacation 8h.
    let mut derived_map = BTreeMap::new();
    derived_map.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    let service_absence = build_parity_service(vec![], derived_map);
    let report_absence = run_report(service_absence).await;

    // Balance-Parity: expected_hours muss quellen-unabhaengig identisch sein.
    assert!(
        (report_extra.expected_hours - report_absence.expected_hours).abs() < 0.01,
        "expected_hours muss quellen-unabhaengig identisch sein: extra={} absence={}",
        report_extra.expected_hours,
        report_absence.expected_hours
    );
    assert!(
        (report_extra.balance_hours - report_absence.balance_hours).abs() < 0.01,
        "balance_hours muss quellen-unabhaengig identisch sein: extra={} absence={}",
        report_extra.balance_hours,
        report_absence.balance_hours
    );
    // Sanity: absence_period-Vacation MUSS expected_hours unter die vollen 40h druecken.
    assert!(
        report_absence.expected_hours < 40.0,
        "absence_period-Vacation MUSS expected_hours unter die vollen 40h druecken (8h reduziert), got {}",
        report_absence.expected_hours
    );
}

// ─── Dynamische Balance-Parity-Tests (Plan 08.4-05) ─────────────────────────

/// Baut ein Mock-Setup fuer get_report_for_employee_range mit dynamischer Work-Details-
/// Fixture (is_dynamic=true, 8h Mo-Fr KW22-25/2024) und ohne Shiftplan-Stunden.
fn build_parity_service_dynamic(
    extra_hours_list: Vec<ExtraHours>,
    derived_map: BTreeMap<time::Date, ResolvedAbsence>,
) -> ReportingServiceImpl<TestDeps> {
    let mut mocks = ReportingMocks::new();

    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));

    // Dynamische Work-Details (is_dynamic=true).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));

    // Kein Shiftplan -> overall_hours = 0.
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    // Kein Carryover.
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    // Transaction passthrough.
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // extra_hours-Mock.
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extra_hours_list);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    // derive_hours_for_range-Mock.
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived_map.clone()));

    mocks.build()
}

/// Balance-Parity-Test fuer DYNAMISCHE Vertraege via get_report_for_employee_range.
/// Eine reine absence_period-Vacation-Buchung (8h an 2024-06-03) muss balance_hours ~0 liefern
/// (quellen-unabhaengig wie extra_hours-Vacation) — get_report_for_employee_range war bereits
/// korrekt (Referenz via hours_per_week Guard). Bleibt gruen pre+post-Fix.
#[tokio::test]
async fn test_balance_parity_dynamic_employee_range() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // Lauf 1: extra_hours-Vacation 8h, kein derived.
    let extra_vacation = make_extra_hours(ExtraHoursCategory::Vacation, 8.0, vacation_day);
    let service_extra = build_parity_service_dynamic(vec![extra_vacation], BTreeMap::new());
    let report_extra = run_report(service_extra).await;

    // Lauf 2: kein extra_hours, derived Vacation 8h.
    let mut derived_map = BTreeMap::new();
    derived_map.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    let service_absence = build_parity_service_dynamic(vec![], derived_map);
    let report_absence = run_report(service_absence).await;

    // Parity: expected_hours und balance_hours muessen quellen-unabhaengig identisch sein.
    assert!(
        (report_extra.expected_hours - report_absence.expected_hours).abs() < 0.01,
        "dynamic range: expected_hours quellen-unabhaengig: extra={} absence={}",
        report_extra.expected_hours,
        report_absence.expected_hours
    );
    assert!(
        (report_extra.balance_hours - report_absence.balance_hours).abs() < 0.01,
        "dynamic range: balance_hours quellen-unabhaengig: extra={} absence={}",
        report_extra.balance_hours,
        report_absence.balance_hours
    );
    // Zusaetzlich: bei dynamischem Vertrag soll Vacation balance ~0 lassen.
    assert!(
        report_extra.balance_hours.abs() < 0.01,
        "dynamic range: extra_hours-Vacation darf balance nicht bewegen, got {}",
        report_extra.balance_hours
    );
    assert!(
        report_absence.balance_hours.abs() < 0.01,
        "dynamic range: absence_period-Vacation darf balance nicht bewegen, got {}",
        report_absence.balance_hours
    );
}

/// Balance-Parity-Test fuer DYNAMISCHE Vertraege via get_reports_for_all_employees.
/// Eine reine absence_period-Vacation-Buchung (8h an 2024-06-03) muss balance_hours ~0 liefern
/// (quellen-unabhaengig wie extra_hours-Vacation) — war pre-Fix DEFEKT (balance=8, nicht 0).
/// RED vor Task 1, GREEN nach Task 1.
#[tokio::test]
async fn test_balance_parity_dynamic_all_employees() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // Lauf A: extra_hours-Vacation 8h, kein derived.
    let mut mocks_a = ReportingMocks::new();
    mocks_a
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks_a
        .sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    mocks_a
        .employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));
    let extra_vacation_a = make_extra_hours(ExtraHoursCategory::Vacation, 8.0, vacation_day);
    let extras_a: Arc<[ExtraHours]> = Arc::from(vec![extra_vacation_a]);
    mocks_a
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(move |_, _, _, _, _| Ok(extras_a.clone()));
    mocks_a
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks_a
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    mocks_a
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks_a
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let service_a = mocks_a.build();
    let reports_a = service_a
        .get_reports_for_all_employees(2024, 23, service::permission::Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees Lauf A");
    let report_a = &reports_a[0];

    // Lauf B: kein extra_hours, derived Vacation 8h.
    let mut mocks_b = ReportingMocks::new();
    mocks_b
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks_b
        .sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    mocks_b
        .employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));
    mocks_b
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks_b
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks_b
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    mocks_b
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    let mut derived_b = BTreeMap::new();
    derived_b.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks_b
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived_b.clone()));

    let service_b = mocks_b.build();
    let reports_b = service_b
        .get_reports_for_all_employees(2024, 23, service::permission::Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees Lauf B");
    let report_b = &reports_b[0];

    // Parity: quellen-unabhaengig.
    assert!(
        (report_a.expected_hours - report_b.expected_hours).abs() < 0.01,
        "dynamic all_employees: expected quellen-unabhaengig: extra={} absence={}",
        report_a.expected_hours,
        report_b.expected_hours
    );
    assert!(
        (report_a.balance_hours - report_b.balance_hours).abs() < 0.01,
        "dynamic all_employees: balance quellen-unabhaengig: extra={} absence={}",
        report_a.balance_hours,
        report_b.balance_hours
    );
    assert!(
        report_a.balance_hours.abs() < 0.01,
        "dynamic all_employees extra: balance ~0, got {}",
        report_a.balance_hours
    );
    assert!(
        report_b.balance_hours.abs() < 0.01,
        "dynamic all_employees absence: balance ~0, got {}",
        report_b.balance_hours
    );
}

/// Balance-Parity-Test fuer DYNAMISCHE Vertraege via get_week.
/// Eine reine absence_period-Vacation-Buchung (8h an 2024-06-03) muss balance_hours ~0 liefern
/// (quellen-unabhaengig wie extra_hours-Vacation) — war pre-Fix DEFEKT (balance aufgeblasen).
/// RED vor Task 2, GREEN nach Task 2.
#[tokio::test]
async fn test_balance_parity_dynamic_get_week() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // Lauf A: extra_hours-Vacation 8h, kein derived.
    let mut mocks_a = ReportingMocks::new();
    mocks_a
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));
    mocks_a
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    let extra_vacation_a = make_extra_hours(ExtraHoursCategory::Vacation, 8.0, vacation_day);
    let extras_a: Arc<[ExtraHours]> = Arc::from(vec![extra_vacation_a]);
    mocks_a
        .extra_hours_service
        .expect_find_by_week()
        .returning(move |_, _, _, _| Ok(extras_a.clone()));
    mocks_a
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks_a
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks_a
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let service_a = mocks_a.build();
    let reports_a = service_a
        .get_week(2024, 23, service::permission::Authentication::Full, None)
        .await
        .expect("get_week Lauf A");
    let report_a = &reports_a[0];

    // Lauf B: kein extra_hours, derived Vacation 8h.
    let mut mocks_b = ReportingMocks::new();
    mocks_b
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_dynamic_mon_fri()])));
    mocks_b
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    mocks_b
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks_b
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks_b
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    let mut derived_b = BTreeMap::new();
    derived_b.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    mocks_b
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived_b.clone()));

    let service_b = mocks_b.build();
    let reports_b = service_b
        .get_week(2024, 23, service::permission::Authentication::Full, None)
        .await
        .expect("get_week Lauf B");
    let report_b = &reports_b[0];

    // Parity: quellen-unabhaengig.
    assert!(
        (report_a.expected_hours - report_b.expected_hours).abs() < 0.01,
        "dynamic get_week: expected quellen-unabhaengig: extra={} absence={}",
        report_a.expected_hours,
        report_b.expected_hours
    );
    assert!(
        (report_a.balance_hours - report_b.balance_hours).abs() < 0.01,
        "dynamic get_week: balance quellen-unabhaengig: extra={} absence={}",
        report_a.balance_hours,
        report_b.balance_hours
    );
    assert!(
        report_a.balance_hours.abs() < 0.01,
        "dynamic get_week extra: balance ~0, got {}",
        report_a.balance_hours
    );
    assert!(
        report_b.balance_hours.abs() < 0.01,
        "dynamic get_week absence: balance ~0, got {}",
        report_b.balance_hours
    );
}

// ─── CVC-10: is_paid-Gate fuer get_week (D-06 / Phase 17) ───────────────────

/// CVC-10 Test 1 — get_week_skips_unpaid_person:
/// Eine Person mit is_paid=false, expected_hours=0, committed_voluntary=5 und
/// aktivem Vertrag in der Testwoche erscheint NICHT im get_week-Result.
/// Eine paid-Person mit Vertrag erscheint weiterhin (kein over-gate).
///
/// Sichert D-06 ab: Gate auf `sales_person.is_paid`, NICHT auf Record-Praesenz.
#[tokio::test]
async fn get_week_skips_unpaid_person() {
    let mut mocks = ReportingMocks::new();

    // all_for_week liefert ZWEI Work-Details: paid + unpaid.
    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                fixture_work_details_8h_mon_fri(),
                unpaid_volunteer_work_details(),
            ]))
        });
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    // find_by_week liefert leere ExtraHours (isoliert den is_paid-Gate).
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<service::extra_hours::ExtraHours>::new())));
    // get() wird pro sales_person_id aufgerufen — liefert den passenden Typ.
    let paid_id = fixture_sales_person_id();
    let unpaid_id = unpaid_person_id();
    mocks
        .sales_person_service
        .expect_get()
        .returning(move |id, _, _| {
            if id == paid_id {
                Ok(fixture_sales_person())
            } else if id == unpaid_id {
                Ok(unpaid_volunteer_sales_person())
            } else {
                Err(service::ServiceError::EntityNotFound(id))
            }
        });
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    // derive_hours_for_range: leere BTreeMap fuer alle Personen (kein Ablenkungsfeuer).
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(std::collections::BTreeMap::new()));

    let service = mocks.build();
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein");

    // Nur die paid-Person erscheint im Result.
    assert_eq!(
        reports.len(),
        1,
        "get_week darf exakt 1 Report enthalten (nur paid-Person), got {}",
        reports.len()
    );
    assert_eq!(
        reports[0].sales_person.id,
        fixture_sales_person_id(),
        "Der einzige Report muss von der paid-Person stammen (is_paid=true)"
    );
    // Unbezahlte Person ist NICHT im Result.
    let has_unpaid = reports
        .iter()
        .any(|r| r.sales_person.id == unpaid_person_id());
    assert!(
        !has_unpaid,
        "Unbezahlte Person (is_paid=false) darf NICHT im get_week-Result erscheinen (D-06/CVC-10)"
    );
}

/// CVC-10 Test 2 — get_week_unpaid_no_paid_hours_leak:
/// Verifiziert dass die unbezahlte Person (is_paid=false, expected_hours=0,
/// committed_voluntary=5) keinen Beitrag zu `paid_hours` (= Summe dynamic_hours
/// ueber get_week-Reports) leistet und nicht in working_hours_per_sales_person
/// auftaucht (Personen-Set-Konsistenz).
///
/// Da paid_hours = Σ report.dynamic_hours ueber get_week-Ergebnis (booking_information
/// get_weekly_summary Z.250-251) und working_hours_per_sales_person aus demselben
/// Report-Vec aufgebaut wird, ist ein get_week-Ergebnis ohne die unbezahlte Person
/// aequivalent zum No-Leak in beiden nachgelagerten Aggregaten.
#[tokio::test]
async fn get_week_unpaid_no_paid_hours_leak() {
    let mut mocks = ReportingMocks::new();

    // Zwei Personen: paid (8h/Tag Mo-Fr) + unpaid (0h, committed=5).
    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                fixture_work_details_8h_mon_fri(),
                unpaid_volunteer_work_details(),
            ]))
        });
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<service::extra_hours::ExtraHours>::new())));
    let paid_id = fixture_sales_person_id();
    let unpaid_id = unpaid_person_id();
    mocks
        .sales_person_service
        .expect_get()
        .returning(move |id, _, _| {
            if id == paid_id {
                Ok(fixture_sales_person())
            } else if id == unpaid_id {
                Ok(unpaid_volunteer_sales_person())
            } else {
                Err(service::ServiceError::EntityNotFound(id))
            }
        });
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(std::collections::BTreeMap::new()));

    let service = mocks.build();
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein");

    // paid_hours = Σ report.dynamic_hours (wie in booking_information::get_weekly_summary).
    let paid_hours: f32 = reports.iter().map(|r| r.dynamic_hours).sum();

    // Nur paid-Person (40h/Woche) traegt zu paid_hours bei, unpaid-Person (0h) NICHT.
    // Da keine Shiftplan-Stunden und kein Carryover gesetzt sind, ist paid_hours = expected_hours
    // der paid-Person fuer diese Woche (40h geteilt auf 5 Tage × 5 Tage in KW23 = 40h).
    assert!(
        paid_hours > 0.0,
        "paid_hours muss > 0 sein (paid-Person traegt 40h/Woche bei), got {}",
        paid_hours
    );
    // Die unbezahlte Person (expected_hours=0) taucht NICHT in den Reports auf,
    // daher ist ihr Beitrag zu paid_hours exakt 0 (sie ist nicht im Ergebnis).
    let unpaid_in_result = reports
        .iter()
        .any(|r| r.sales_person.id == unpaid_person_id());
    assert!(
        !unpaid_in_result,
        "Unbezahlte Person darf NICHT in get_week-Reports erscheinen — kein paid_hours-Leak (CVC-10)"
    );
    // Personen-Set-Konsistenz: working_hours_per_sales_person (abgeleitet aus get_week-Berichten)
    // wuerde nur paid-Person enthalten, weil get_week die unbezahlte filtert.
    assert_eq!(
        reports.len(),
        1,
        "Genau 1 Report (paid-Person) — keine unbezahlte Person im Personen-Set"
    );
    assert_eq!(
        reports[0].sales_person.id,
        fixture_sales_person_id(),
        "Der Report gehoert der paid-Person, nicht der unbezahlten"
    );
}

// ─── UV-05 Regression Tests (D-18-06) ────────────────────────────────────────

/// UV-05 Test 1 — Conversion parity: vacation_days same & >0 before/after conversion.
/// Lauf 1: Vacation week as extra_hours (8h) -> vacation_days_extra.
/// Lauf 2: SAME week as absence_period (8h derived, extra_hours empty) -> vacation_days_absence.
/// Assert: both > 0.0 AND equal (within tolerance).
///
/// Pre-fix: absence_period path yielded vacation_days = 0 (vacation_hours = 0 in per-week
/// category fields — derived not merged). Post-fix: per-week merges derived, so days == hours / hours_per_day > 0.
#[tokio::test]
async fn test_converted_vacation_preserves_days() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // Lauf 1: extra_hours Vacation 8h (no derived).
    let service_extra = build_parity_service(
        vec![make_extra_hours(ExtraHoursCategory::Vacation, 8.0, vacation_day)],
        BTreeMap::new(),
    );
    let report_extra = run_report(service_extra).await;
    let vacation_days_extra = report_extra.vacation_days;

    // Lauf 2: absence_period only — extra_hours soft-deleted (empty slice), 8h derived.
    let mut derived_map = BTreeMap::new();
    derived_map.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    let service_absence = build_parity_service(vec![], derived_map);
    let report_absence = run_report(service_absence).await;
    let vacation_days_absence = report_absence.vacation_days;

    assert!(
        vacation_days_extra > 0.0,
        "UV-05: vacation_days (extra_hours path) muss > 0 sein, got {}",
        vacation_days_extra
    );
    assert!(
        vacation_days_absence > 0.0,
        "UV-05: vacation_days (absence_period path) muss > 0 sein, got {} (war 0 vor dem Fix)",
        vacation_days_absence
    );
    assert!(
        (vacation_days_extra - vacation_days_absence).abs() < 0.01,
        "UV-05: vacation_days muss quellen-unabhaengig identisch sein: extra={} absence={}",
        vacation_days_extra,
        vacation_days_absence
    );
}

/// UV-05 Test 2 — No double-count: absence_period-only case, vacation_hours == single derived total.
/// Uses 8h fixture (same as test_additive_only_absence_period) to detect doubling (16h would fail).
/// Also asserts vacation_days == vacation_hours / hours_per_day.
#[tokio::test]
async fn test_converted_vacation_no_double_count() {
    let vacation_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    // absence_period only: 8h Vacation derived. extra_hours empty (converted = soft-deleted).
    let mut derived_map = BTreeMap::new();
    derived_map.insert(
        vacation_day,
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    let service = build_parity_service(vec![], derived_map);
    let report = run_report(service).await;

    // vacation_hours must equal the single derived total (8h), NOT doubled (16h).
    assert!(
        (report.vacation_hours - 8.0).abs() < 0.01,
        "UV-05 no-double-count: vacation_hours muss 8.0h sein (nicht 16.0h), got {}",
        report.vacation_hours
    );

    // vacation_days = vacation_hours / hours_per_day (8h / 8h = 1.0).
    // hours_per_day = contract_weekly_hours / workdays_per_week = 40/5 = 8.0.
    let hours_per_day = 8.0_f32; // fixture_work_details_8h_mon_fri: 40h/5days
    assert!(
        (report.vacation_days - report.vacation_hours / hours_per_day).abs() < 0.01,
        "UV-05: vacation_days muss vacation_hours / hours_per_day sein: days={} hours={} hpd={}",
        report.vacation_days,
        report.vacation_hours,
        hours_per_day
    );
}

/// UV-05 Test 3 — sick_leave_days > 0 for absence_period SickLeave week.
/// An absence_period SickLeave entry (8h) must yield sick_leave_days > 0 and
/// absence_days >= sick_leave_days after the UV-05 fix.
#[tokio::test]
async fn test_converted_sick_leave_preserves_days() {
    let sick_day = time::macros::date!(2024 - 06 - 03); // Montag KW23

    let mut derived_map = BTreeMap::new();
    derived_map.insert(
        sick_day,
        ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
            days: 1.0,
        },
    );
    let service = build_parity_service(vec![], derived_map);
    let report = run_report(service).await;

    assert!(
        report.sick_leave_days > 0.0,
        "UV-05: sick_leave_days muss > 0 sein fuer absence_period SickLeave 8h, got {}",
        report.sick_leave_days
    );
    assert!(
        report.absence_days >= report.sick_leave_days,
        "UV-05: absence_days ({}) muss >= sick_leave_days ({}) sein",
        report.absence_days,
        report.sick_leave_days
    );
}
