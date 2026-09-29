//! Planungs-Soll-Tests (Phase 64 — PSOLL-01 / PSOLL-03 / PSOLL-04).
//!
//! Gegenstand ist das additive Feld `ShortEmployeeReport::planning_target_hours`:
//! das feiertags- und abwesenheitsreduzierte Vertrags-Soll fuer die PLANUNG. Es
//! wird — anders als `expected_hours` — auch fuer dynamische Vertraege gefuellt
//! und geht in keine Bilanz ein.
//!
//! WICHTIG (Test-first): Diese Datei ist RED vor Task 3 und GREEN nach Task 3.
//! Vor Task 3 tragen beide Konstruktionsstellen nur das Geruest-Literal `0.0`,
//! die vier Planungs-Soll-Tests scheitern deshalb an einer ZAHL (0.0 statt des
//! erwarteten Werts), nicht an einem Kompilierfehler.
//! `test_available_hours_unchanged_dynamic` ist bereits vor Task 3 gruen — es
//! ist ein Regressionsschloss (T-64-02), kein Treiber.
//!
//! Herleitung der Erwartungswerte (alle aus dem Vertrag der Fixture, nicht aus
//! einem Testlauf abgelesen):
//! - `fixture_work_details_dynamic_mon_fri` / `fixture_work_details_8h_mon_fri`:
//!   40h je volle Vertragswoche, Mo-Fr, 5 Werktage => 8h je Werktag,
//!   gueltig KW 22-25/2024.
//! - `get_week(2024, 23)` sieht genau eine Vertragswoche => 40.0.
//! - `get_reports_for_all_employees(2024, 23)` summiert die Vertragswochen 22
//!   und 23 => 80.0.
//! - `fixture_vacation_period` (Mo-Mi 2024-06-03..05) + `fixture_sick_period`
//!   (Di 2024-06-04, BUrlG §9 => Krankheit verdraengt Urlaub) ergeben drei
//!   abwesende Werktage in KW 23 => 24.0h Reduktion.

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod, MockAbsenceService, ResolvedAbsence};
use service::booking::MockBookingService;
use service::booking_information::BookingInformationService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::{MockReportingService, ReportingService, ShortEmployeeReport};
use service::sales_person::MockSalesPersonService;
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::shiftplan_report::MockShiftplanReportService;
use service::slot::MockSlotService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;

use crate::booking_information::{BookingInformationServiceDeps, BookingInformationServiceImpl};
use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_sick_period,
    fixture_vacation_period, fixture_work_details_8h_mon_fri,
    fixture_work_details_dynamic_mon_fri,
};

// ─── Reporting-Mock-Boilerplate (1:1 aus reporting_additive_merge.rs) ────────

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
    type RebookingBatchService = MockRebookingBatchService;
}

impl ReportingMocks {
    fn new() -> Self {
        // Holiday-Automation aus (kein Toggle-Wert = kein derived holiday).
        let mut toggle_service = MockToggleService::new();
        toggle_service
            .expect_get_toggle_value()
            .returning(|_, _, _| Ok(None));
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
            toggle_service,
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
            rebooking_batch_service: Arc::new(MockRebookingBatchService::new()),
        }
    }
}

/// Reporting-Service fuer den `get_week`/`assemble_weeks`-Pfad.
/// Die Abwesenheiten kommen als `AbsencePeriod`-Liste herein und werden vom
/// produktiven `derive_hours_for_week_pure`-Helfer abgeleitet.
fn build_get_week_service(
    work_details: EmployeeWorkDetails,
    absence_periods: Vec<AbsencePeriod>,
) -> ReportingServiceImpl<TestDeps> {
    let mut mocks = ReportingMocks::new();
    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(move |_, _, _, _| Ok(Arc::from(vec![work_details.clone()])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks
        .sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    let periods: Arc<[AbsencePeriod]> = Arc::from(absence_periods);
    mocks
        .absence_service
        .expect_find_all()
        .returning(move |_, _| Ok(periods.clone()));
    mocks
        .special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<service::special_days::SpecialDay>::new())));
    mocks.build()
}

/// Reporting-Service fuer den `get_reports_for_all_employees`-Pfad.
/// Dieser Pfad laedt die abgeleiteten Abwesenheiten ueber
/// `derive_hours_for_range` — deshalb wird hier die bereits aufgeloeste Map
/// uebergeben (Mo Urlaub 8h, Di Krankheit 8h, Mi Urlaub 8h).
fn build_all_employees_service(
    work_details: EmployeeWorkDetails,
    derived: BTreeMap<time::Date, ResolvedAbsence>,
) -> ReportingServiceImpl<TestDeps> {
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
        .returning(move |_, _| Ok(Arc::from(vec![work_details.clone()])));
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
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
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(move |_, _, _, _, _| Ok(derived.clone()));
    mocks.build()
}

/// Die aufgeloeste Abwesenheits-Map zu `fixture_vacation_period` (Mo-Mi) +
/// `fixture_sick_period` (Di). BUrlG §9: der Krankheitstag verdraengt den
/// Urlaubstag am Dienstag. 8h je Werktag (40h / 5 Werktage).
fn derived_absences_week23() -> BTreeMap<time::Date, ResolvedAbsence> {
    let mut map = BTreeMap::new();
    map.insert(
        time::macros::date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    map.insert(
        time::macros::date!(2024 - 06 - 04),
        ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
            days: 1.0,
        },
    );
    map.insert(
        time::macros::date!(2024 - 06 - 05),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        },
    );
    map
}

// ─── PSOLL-01: das Planungs-Soll ist bei dynamischen Vertraegen > 0 ─────────

/// PSOLL-01, Konstruktionsstelle A (`assemble_weeks` via `get_week`).
/// Dynamischer 40h-Vertrag, KW 23/2024, keine Abwesenheiten:
/// Planungs-Soll ~40.0, waehrend `expected_hours` (bilanzrelevant) bei ~0.0
/// und `dynamic_hours` bei ~40.0 bleiben.
#[tokio::test]
async fn test_planning_target_dynamic_get_week() {
    let service = build_get_week_service(fixture_work_details_dynamic_mon_fri(), vec![]);
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein");
    let report = &reports[0];

    assert!(
        (report.planning_target_hours - 40.0).abs() < 0.01,
        "PSOLL-01 get_week: Planungs-Soll muss 40.0 sein, got {}",
        report.planning_target_hours
    );
    assert!(
        report.expected_hours.abs() < 0.01,
        "PSOLL-01 get_week: bilanzrelevantes Soll muss unveraendert 0.0 bleiben, got {}",
        report.expected_hours
    );
    assert!(
        (report.dynamic_hours - 40.0).abs() < 0.01,
        "PSOLL-01 get_week: dynamic_hours muss unveraendert 40.0 bleiben, got {}",
        report.dynamic_hours
    );
}

/// PSOLL-01, Konstruktionsstelle B (`get_reports_for_all_employees`).
/// Dieselbe dynamische Fixture: das Fenster bis KW 23 enthaelt die
/// Vertragswochen 22 und 23 => 2 x 40.0 = 80.0.
#[tokio::test]
async fn test_planning_target_dynamic_all_employees() {
    let service =
        build_all_employees_service(fixture_work_details_dynamic_mon_fri(), BTreeMap::new());
    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein");
    let report = &reports[0];

    assert!(
        (report.planning_target_hours - 80.0).abs() < 0.01,
        "PSOLL-01 all_employees: Planungs-Soll muss 80.0 sein (KW22 + KW23), got {}",
        report.planning_target_hours
    );
    assert!(
        report.expected_hours.abs() < 0.01,
        "PSOLL-01 all_employees: bilanzrelevantes Soll muss unveraendert 0.0 bleiben, got {}",
        report.expected_hours
    );
}

// ─── PSOLL-03: beide Konstruktionsstellen liefern denselben Wert ────────────

/// PSOLL-03 — der Test, der die beiden unabhaengig geschriebenen
/// Aggregationspfade gegeneinander haelt (Lehre aus dem Phase-57-Fund).
///
/// Dynamische Fixture MIT Urlaub (Mo-Mi) und Krankheit (Di) in KW 23.
/// Verglichen wird `get_reports_for_all_employees(2024, 23)` gegen die Summe
/// aus `get_week(2024, 22)` und `get_week(2024, 23)`.
///
/// Die Ober-/Untergrenzen-Assertion ist der eigentliche Zweck: sie faellt genau
/// dann um, wenn jemand die bestehenden — fuer dynamische Vertraege auf 0
/// gezwungenen — Reduktionsterme wiederverwendet (dann 80.0) oder das Feld
/// neutral laesst (dann 0.0).
#[tokio::test]
async fn test_planning_target_parity_both_sites() {
    let absences = vec![fixture_vacation_period(), fixture_sick_period()];

    let service_week = build_get_week_service(fixture_work_details_dynamic_mon_fri(), absences);
    let week22 = service_week
        .get_week(2024, 22, Authentication::Full, None)
        .await
        .expect("get_week KW22 muss erfolgreich sein")[0]
        .planning_target_hours;
    let week23 = service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week KW23 muss erfolgreich sein")[0]
        .planning_target_hours;
    let get_week_sum = week22 + week23;

    let service_all = build_all_employees_service(
        fixture_work_details_dynamic_mon_fri(),
        derived_absences_week23(),
    );
    let all_employees = service_all
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein")[0]
        .planning_target_hours;

    assert!(
        (get_week_sum - all_employees).abs() < 0.01,
        "PSOLL-03: beide Konstruktionsstellen muessen denselben Wert liefern: \
         get_week(KW22={week22} + KW23={week23}) = {get_week_sum}, all_employees = {all_employees}"
    );
    // Untergrenze: ein dynamischer Vertrag mit Vertragszeile hat ein Planungs-Soll > 0
    // (faellt um, wenn das Feld neutral bleibt).
    assert!(
        all_employees > 0.0,
        "PSOLL-03: Planungs-Soll eines dynamischen Vertrags muss > 0 sein, got {all_employees}"
    );
    // Obergrenze: die Reduktion muss GREIFEN (faellt um, wenn die bestehenden,
    // auf 0 gezwungenen Reduktionsterme wiederverwendet werden).
    assert!(
        all_employees < 80.0,
        "ROADMAP-Erfolgskriterium 3: Abwesenheiten muessen das Planungs-Soll reduzieren \
         (unreduziert waeren es 80.0), got {all_employees}"
    );
}

// ─── Byte-Identitaet fuer nicht-dynamische Vertraege ────────────────────────

/// ROADMAP-Erfolgskriterium 4: fuer einen NICHT-dynamischen Vertrag ist das
/// Planungs-Soll zeichengleich mit `expected_hours` — an beiden Entry-Points.
///
/// Zweiter Assertion-Block: dieselbe Fixture, aber mit `expected_hours: 0.0`
/// (weiterhin nicht dynamisch). Auch dort muss das Planungs-Soll zeichengleich
/// mit `expected_hours` sein, also 0.0 und NICHT negativ. Das ist der direkte
/// Regressionstest fuer die zweite Gate-Bedingung (`Rohwert > 0.0`): faellt sie
/// weg, wird von einer 0-Basis abgezogen und das Planungs-Soll wird negativ.
#[tokio::test]
async fn test_planning_target_matches_expected_for_fixed_contract() {
    let absences = vec![fixture_vacation_period(), fixture_sick_period()];

    // ── Block 1: 40h-Vertrag, nicht dynamisch ──
    let service_week = build_get_week_service(fixture_work_details_8h_mon_fri(), absences.clone());
    let week_report = &service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein")[0];
    assert!(
        (week_report.planning_target_hours - week_report.expected_hours).abs() < 0.001,
        "fester Vertrag / get_week: Planungs-Soll {} muss zeichengleich mit expected_hours {} sein",
        week_report.planning_target_hours,
        week_report.expected_hours
    );

    let service_all =
        build_all_employees_service(fixture_work_details_8h_mon_fri(), derived_absences_week23());
    let all_report = &service_all
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein")[0];
    assert!(
        (all_report.planning_target_hours - all_report.expected_hours).abs() < 0.001,
        "fester Vertrag / all_employees: Planungs-Soll {} muss zeichengleich mit \
         expected_hours {} sein",
        all_report.planning_target_hours,
        all_report.expected_hours
    );

    // ── Block 2: nicht dynamisch, aber 0 Vertragsstunden ──
    let zero_contract = EmployeeWorkDetails {
        expected_hours: 0.0,
        ..fixture_work_details_8h_mon_fri()
    };
    let service_week_zero = build_get_week_service(zero_contract.clone(), absences);
    let week_zero = &service_week_zero
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week (0h-Vertrag) muss erfolgreich sein")[0];
    assert!(
        (week_zero.planning_target_hours - week_zero.expected_hours).abs() < 0.001,
        "0h-Vertrag / get_week: Planungs-Soll {} muss zeichengleich mit expected_hours {} sein",
        week_zero.planning_target_hours,
        week_zero.expected_hours
    );
    assert!(
        week_zero.planning_target_hours >= 0.0,
        "0h-Vertrag / get_week: Planungs-Soll darf nicht negativ werden, got {}",
        week_zero.planning_target_hours
    );

    let service_all_zero = build_all_employees_service(zero_contract, derived_absences_week23());
    let all_zero = &service_all_zero
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees (0h-Vertrag) muss erfolgreich sein")[0];
    assert!(
        (all_zero.planning_target_hours - all_zero.expected_hours).abs() < 0.001,
        "0h-Vertrag / all_employees: Planungs-Soll {} muss zeichengleich mit \
         expected_hours {} sein",
        all_zero.planning_target_hours,
        all_zero.expected_hours
    );
    assert!(
        all_zero.planning_target_hours >= 0.0,
        "0h-Vertrag / all_employees: Planungs-Soll darf nicht negativ werden, got {}",
        all_zero.planning_target_hours
    );
}

// ─── PSOLL-04 / T-64-02: available_hours liest weiterhin das Bilanz-Soll ────

const BI_YEAR: u32 = 2026;
/// 2026-W20 = Mo 11.05. .. So 17.05.
const BI_WEEK: u8 = 20;

struct BookingInfoTestDeps;

impl BookingInformationServiceDeps for BookingInfoTestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ShiftplanReportService = MockShiftplanReportService;
    type SlotService = MockSlotService;
    type ShiftplanService = service::shiftplan_catalog::MockShiftplanService;
    type BookingService = MockBookingService;
    type SalesPersonService = MockSalesPersonService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type ReportingService = MockReportingService;
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type AbsenceService = MockAbsenceService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = dao::MockTransactionDao;
}

fn bi_work_details(sales_person_id: Uuid) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id,
        expected_hours: 0.0,
        from_day_of_week: shifty_utils::DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2026,
        to_day_of_week: shifty_utils::DayOfWeek::Sunday,
        to_calendar_week: 3,
        to_year: 2027,
        workdays_per_week: 5,
        is_dynamic: true,
        cap_planned_hours_to_expected: false,
        committed_voluntary: 0.0,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 0,
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// PSOLL-04 / T-64-02 — Regressionsschloss.
///
/// Der `booking_information`-Pfad bekommt einen `ShortEmployeeReport` mit
/// bilanzrelevantem Soll 0.0 und Planungs-Soll 40.0 (der dynamische Fall).
/// `available_hours` in `WorkingHoursPerSalesPerson` MUSS 0.0 bleiben — die
/// Verfuegbarkeitsrechnung liest `expected_hours`, nicht das Planungs-Soll.
/// Dieser Test ist schon vor Task 3 gruen; er faellt, sobald jemand die beiden
/// `available_hours`-Zuweisungen auf das neue Feld umstellt.
#[tokio::test]
async fn test_available_hours_unchanged_dynamic() {
    let person = fixture_sales_person();
    let person_arc = Arc::new(person.clone());

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let persons: Arc<[service::sales_person::SalesPerson]> = Arc::from(vec![person.clone()]);
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(persons.clone()));

    let work_details: Arc<[EmployeeWorkDetails]> =
        Arc::from(vec![bi_work_details(fixture_sales_person_id())]);
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(work_details.clone()));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(vec![])));
    special_day_service
        .expect_get_by_iso_year()
        .returning(|_, _| Ok(Arc::from(vec![])));

    // Der Report, den die Verfuegbarkeitsrechnung liest: Bilanz-Soll 0.0
    // (dynamischer Vertrag), Planungs-Soll 40.0.
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    reporting_service
        .expect_get_year()
        .returning(move |year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| {
                    if year == BI_YEAR && w == BI_WEEK {
                        let report = ShortEmployeeReport {
                            sales_person: person_arc.clone(),
                            balance_hours: 0.0,
                            dynamic_hours: 40.0,
                            expected_hours: 0.0,
                            overall_hours: 0.0,
                            planning_target_hours: 40.0,
                            vacation_hours: 0.0,
                            sick_leave_hours: 0.0,
                            holiday_hours: 0.0,
                            unavailable_hours: 0.0,
                            unpaid_leave_hours: 0.0,
                            volunteer_hours: 0.0,
                            custom_absence_hours: Arc::from(Vec::new()),
                            has_pending_rebooking: false,
                            pending_rebooking_id: None,
                        };
                        (w, Arc::from(vec![report]))
                    } else {
                        (w, Arc::from(Vec::new()))
                    }
                })
                .collect();
            Ok(Arc::from(out))
        });

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| Ok(Arc::from(vec![])));

    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    slot_service
        .expect_get_slots()
        .returning(|_, _| Ok(Arc::from(vec![])));

    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let service = BookingInformationServiceImpl::<BookingInfoTestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
        shiftplan_service: Arc::new(shiftplan_service_mock),
        booking_service: Arc::new(MockBookingService::new()),
        sales_person_service: Arc::new(sales_person_service),
        sales_person_unavailable_service: Arc::new(MockSalesPersonUnavailableService::new()),
        reporting_service: Arc::new(reporting_service),
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        absence_service: Arc::new(absence_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        transaction_dao: Arc::new(transaction_dao),
    };

    let summaries = service
        .get_weekly_summary(BI_YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary muss erfolgreich sein");
    let week = summaries
        .iter()
        .find(|s| s.year == BI_YEAR && s.week == BI_WEEK)
        .expect("Testwoche muss in der Jahresuebersicht vorkommen");
    let entry = week
        .working_hours_per_sales_person
        .iter()
        .find(|wh| wh.sales_person_id == fixture_sales_person_id())
        .expect("bezahlte Person muss in working_hours_per_sales_person stehen");

    assert!(
        entry.available_hours.abs() < 0.001,
        "PSOLL-04 / T-64-02: available_hours muss das bilanzrelevante Soll (0.0) lesen, \
         nicht das Planungs-Soll (40.0), got {}",
        entry.available_hours
    );
}
