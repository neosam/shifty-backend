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
//!
//! ## Nachtrag Plan 64-05 (D-64-05 / D-64-06 / D-64-07)
//!
//! Der Modul-Kopf oben beschreibt das Planungs-Soll als „Vertrags-Soll". Das war
//! die Fassung aus 64-01 und ist **ueberholt**: das Planungs-Soll ist eine
//! **Zusage**-Groesse [D-64-05]. Die ehrenamtliche Zusage
//! (`employee_work_details.committed_voluntary`) fliesst ein, denn der Zweck des
//! Feldes ist die Schichtplanung — die planende Person muss sehen, was die
//! jeweilige Person fuer die Woche zusagt, unabhaengig davon, ob die Stunden
//! bezahlt sind. Abwesenheiten und Feiertage reduzieren den Ehrenamts-Anteil
//! proportional mit.
//!
//! Herleitung der vier Kern-Zahlen der Zusage-Tests (Fixtures mit
//! `committed_voluntary = 10.0`, volle Vertragswoche, `relation = 1.0`):
//! - `dynamic_hours = 40.0`, `committed_voluntary_for_target = 10.0`
//!   => `planning_base_for_target = 50.0` (Stelle A, eine Vertragswoche)
//! - Stelle B bis KW 23 summiert die Vertragswochen 22 und 23 => `100.0`
//! - `target_reduction_scale = 50 / 40 = 1.25` [D-64-07] — die Reduktion wird um
//!   genau den Faktor hochskaliert, um den die Basis gewachsen ist; eine reine
//!   Subtraktion kuerzte nur den bezahlten Anteil (50 - 24 = 26 statt 20)
//! - KW 23 mit drei abwesenden Werktagen: `50 - 24 * 1.25 = 20.0`, in der Summe
//!   mit KW 22 also `70.0`

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
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource, MockExtraHoursService};
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
    fixture_work_details_dynamic_mon_fri, fixture_work_details_dynamic_with_commitment,
    fixture_work_details_fixed_with_commitment, fixture_work_details_zero_hours_with_commitment,
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
    // Plan 64-05: die bestehenden Aufrufer bekommen unveraendert eine leere
    // ExtraHours-Liste — das Verhalten dieser Harness-Variante aendert sich nicht.
    build_get_week_service_with_extra_hours(work_details, absence_periods, vec![])
}

/// Plan 64-05: wie [`build_get_week_service`], aber mit manuellen ExtraHours der
/// Woche. Gebraucht fuer die No-Clamp-Sperre, die eine manuelle
/// `Vacation`-Reduktion groesser als die Planungs-Basis setzt.
fn build_get_week_service_with_extra_hours(
    work_details: EmployeeWorkDetails,
    absence_periods: Vec<AbsencePeriod>,
    extra_hours: Vec<ExtraHours>,
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
    let week_extra_hours: Arc<[ExtraHours]> = Arc::from(extra_hours);
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(move |_, _, _, _| Ok(week_extra_hours.clone()));
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
    // Plan 64-05: die bestehenden Aufrufer bekommen unveraendert eine leere
    // ExtraHours-Liste — das Verhalten dieser Harness-Variante aendert sich nicht.
    build_all_employees_service_with_extra_hours(work_details, derived, vec![])
}

/// Plan 64-05: wie [`build_all_employees_service`], aber mit manuellen ExtraHours
/// des Jahres (Gegenstueck zu [`build_get_week_service_with_extra_hours`]).
fn build_all_employees_service_with_extra_hours(
    work_details: EmployeeWorkDetails,
    derived: BTreeMap<time::Date, ResolvedAbsence>,
    extra_hours: Vec<ExtraHours>,
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
    let year_extra_hours: Arc<[ExtraHours]> = Arc::from(extra_hours);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(move |_, _, _, _, _| Ok(year_extra_hours.clone()));
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

// ─── Plan 64-05: die ehrenamtliche Zusage fliesst ein (D-64-05) ──────────────

/// Manuelle `Vacation`-ExtraHours-Zeile fuer die No-Clamp-Sperre.
///
/// Die Kategorie ist bewusst `Vacation`: sie erfuellt BEIDE Filter der beiden
/// Konstruktionsstellen (`availability() == Availability::Unavailable` an Stelle A,
/// `as_report_type() == ReportType::AbsenceHours` an Stelle B). Die in F07 §2.7
/// dokumentierte vorbestehende Filter-Asymmetrie beisst hier deshalb nicht.
fn manual_vacation_20h_week23() -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_2000),
        sales_person_id: fixture_sales_person_id(),
        amount: 20.0,
        category: ExtraHoursCategory::Vacation,
        description: Arc::from("64-05 no-clamp lock"),
        date_time: datetime!(2024 - 06 - 03 09:00:00),
        created: Some(datetime!(2024 - 06 - 03 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source: ExtraHoursSource::Manual,
    }
}

/// PSOLL-01 (neue Fassung, D-64-05): die Zusage ist Teil des Planungs-Solls, an
/// BEIDEN Konstruktionsstellen.
///
/// Dynamischer 40h-Vertrag mit 10h Zusage, keine Abwesenheiten.
/// Stelle A (`get_week` 2024/23, eine Vertragswoche): 40 + 10 = **50.0**.
/// Stelle B (`get_reports_for_all_employees` bis 2024/23, Vertragswochen 22+23):
/// 2 x 50 = **100.0**.
///
/// **Faengt:** (i) die Zusage wird ignoriert — dann 40.0 / 80.0, genau der Zustand
/// vor Plan 64-05; (ii) die Zusage schlaegt in das bilanzrelevante Soll durch —
/// dann ist `expected_hours` nicht mehr 0.0.
#[tokio::test]
async fn planning_target_includes_commitment_dynamic_both_sites() {
    // ── Stelle A ──
    let service_week = build_get_week_service(fixture_work_details_dynamic_with_commitment(), vec![]);
    let week_report = &service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein")[0];
    assert!(
        (week_report.planning_target_hours - 50.0).abs() < 0.01,
        "D-64-05 / Stelle A: Planungs-Soll muss 40 (Vertrag) + 10 (Zusage) = 50.0 sein; \
         40.0 waere „Zusage ignoriert\" (Zustand vor 64-05), got {}",
        week_report.planning_target_hours
    );
    assert!(
        week_report.expected_hours.abs() < 0.01,
        "D-64-05 / Stelle A: die Zusage darf das bilanzrelevante Soll NICHT bewegen \
         (erwartet 0.0), got {}",
        week_report.expected_hours
    );

    // ── Stelle B ──
    let service_all =
        build_all_employees_service(fixture_work_details_dynamic_with_commitment(), BTreeMap::new());
    let all_report = &service_all
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein")[0];
    assert!(
        (all_report.planning_target_hours - 100.0).abs() < 0.01,
        "D-64-05 / Stelle B: Planungs-Soll muss 2 x 50.0 = 100.0 sein (KW22 + KW23); \
         80.0 waere „Zusage ignoriert\", got {}",
        all_report.planning_target_hours
    );
    assert!(
        all_report.expected_hours.abs() < 0.01,
        "D-64-05 / Stelle B: die Zusage darf das bilanzrelevante Soll NICHT bewegen \
         (erwartet 0.0), got {}",
        all_report.expected_hours
    );
}

/// D-64-05 + D-64-07: Abwesenheiten reduzieren den Ehrenamts-Anteil
/// **proportional** mit — und beide Konstruktionsstellen tun das gleich.
///
/// Dynamischer 40h-Vertrag mit 10h Zusage, KW 23 mit drei abwesenden Werktagen
/// (24.0h). Basis 50.0, `target_reduction_scale = 50/40 = 1.25`, also
/// `50 - 24 * 1.25 = 20.0` in KW 23 und 50.0 in KW 22.
///
/// Die Wochen werden EINZELN assertiert — bei einer reinen Summen-Assertion
/// koennte sich ein Fehler zwischen den Wochen ausgleichen.
///
/// **Faengt:** (i) Zusage ignoriert => 40 + (40 - 24) = 56.0; (ii) reine
/// Subtraktion ohne `target_reduction_scale` => 50 + (50 - 24) = 76.0 — genau die
/// arithmetische Mutation, die D-64-07 verhindert; (iii) Feld neutral => 0.0.
#[tokio::test]
async fn planning_target_commitment_reduced_by_absence_parity() {
    let absences = vec![fixture_vacation_period(), fixture_sick_period()];

    let service_week =
        build_get_week_service(fixture_work_details_dynamic_with_commitment(), absences);
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

    assert!(
        (week22 - 50.0).abs() < 0.01,
        "D-64-05 / Stelle A KW22 (ohne Abwesenheit): 40 + 10 = 50.0 erwartet; \
         40.0 waere „Zusage ignoriert\", got {week22}"
    );
    assert!(
        (week23 - 20.0).abs() < 0.01,
        "D-64-07 / Stelle A KW23: 50 - 24 x 1.25 = 20.0 erwartet. 26.0 waere die reine \
         Subtraktion ohne `target_reduction_scale` (kuerzt nur den bezahlten Anteil), \
         16.0 waere „Zusage ignoriert\", got {week23}"
    );

    let service_all = build_all_employees_service(
        fixture_work_details_dynamic_with_commitment(),
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
         get_week(KW22={week22} + KW23={week23}) = {get_week_sum}, \
         all_employees = {all_employees}"
    );
    assert!(
        (all_employees - 70.0).abs() < 0.01,
        "D-64-05 / D-64-07 / Stelle B: 50.0 (KW22) + 20.0 (KW23) = 70.0 erwartet. \
         56.0 waere „Zusage ignoriert\", 76.0 die reine Subtraktion ohne \
         `target_reduction_scale`, 0.0 ein neutrales Feld, got {all_employees}"
    );
    assert!(
        all_employees > 0.0,
        "D-64-05: das Planungs-Soll eines dynamischen Vertrags mit Zusage muss > 0 sein, \
         got {all_employees}"
    );
}

/// PSOLL-04 in seiner neuen Fassung: Byte-Identitaet mit `expected_hours` gilt nur
/// noch **bei Zusage 0**; mit Zusage ist die Differenz genau die um Abwesenheiten
/// und Feiertage reduzierte Zusage.
///
/// Nicht-dynamischer 40h-Vertrag mit 10h Zusage.
/// Block 1 (KW 22, ohne Abwesenheit): 50.0 gegen 40.0, Differenz **10.0**.
/// Block 2 (KW 23, drei abwesende Werktage): 20.0 gegen 16.0, Differenz **4.0**
/// (= 10 x (1 - 24/40) = 10 x 0.4).
/// Block 3 (dieselbe Fixture mit Zusage 0): Byte-Identitaet, `< 0.001`.
///
/// **Faengt:** jede Loesung, die die Zusage in `expected_hours` durchschlagen
/// laesst, und jede, die die Byte-Identitaet bei Zusage 0 verliert.
#[tokio::test]
async fn planning_target_fixed_contract_differs_by_commitment() {
    let absences = vec![fixture_vacation_period(), fixture_sick_period()];
    let service_week =
        build_get_week_service(fixture_work_details_fixed_with_commitment(), absences.clone());

    // ── Block 1: KW 22, ohne Abwesenheit ──
    let kw22 = &service_week
        .get_week(2024, 22, Authentication::Full, None)
        .await
        .expect("get_week KW22 muss erfolgreich sein")[0];
    assert!(
        (kw22.planning_target_hours - 50.0).abs() < 0.01,
        "PSOLL-04 KW22: Planungs-Soll 50.0 erwartet, got {}",
        kw22.planning_target_hours
    );
    assert!(
        (kw22.expected_hours - 40.0).abs() < 0.01,
        "PSOLL-04 KW22: bilanzrelevantes Soll muss unveraendert 40.0 sein, got {}",
        kw22.expected_hours
    );
    assert!(
        (kw22.planning_target_hours - kw22.expected_hours - 10.0).abs() < 0.001,
        "PSOLL-04 KW22: die Differenz muss exakt die unreduzierte Zusage (10.0) sein, \
         got {}",
        kw22.planning_target_hours - kw22.expected_hours
    );

    // ── Block 2: KW 23, drei abwesende Werktage ──
    let kw23 = &service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week KW23 muss erfolgreich sein")[0];
    assert!(
        (kw23.planning_target_hours - 20.0).abs() < 0.01,
        "PSOLL-04 KW23: Planungs-Soll 50 - 24 x 1.25 = 20.0 erwartet, got {}",
        kw23.planning_target_hours
    );
    assert!(
        (kw23.expected_hours - 16.0).abs() < 0.01,
        "PSOLL-04 KW23: bilanzrelevantes Soll muss unveraendert 40 - 24 = 16.0 sein, got {}",
        kw23.expected_hours
    );
    assert!(
        (kw23.planning_target_hours - kw23.expected_hours - 4.0).abs() < 0.001,
        "PSOLL-04 KW23: die Differenz muss exakt die reduzierte Zusage sein \
         (10 x (1 - 24/40) = 4.0), got {}",
        kw23.planning_target_hours - kw23.expected_hours
    );

    // ── Block 3: dieselbe Fixture, aber Zusage 0 => Byte-Identitaet ──
    let no_commitment = EmployeeWorkDetails {
        committed_voluntary: 0.0,
        ..fixture_work_details_fixed_with_commitment()
    };
    let service_zero_commitment = build_get_week_service(no_commitment, absences);
    let zero_kw23 = &service_zero_commitment
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week KW23 (Zusage 0) muss erfolgreich sein")[0];
    assert!(
        (zero_kw23.planning_target_hours - zero_kw23.expected_hours).abs() < 0.001,
        "PSOLL-04: bei Zusage 0 faellt `target_reduction_scale` auf 1.0 und die Formel \
         strukturell auf die 64-01-Formel zurueck — Planungs-Soll {} muss byte-identisch \
         mit expected_hours {} sein",
        zero_kw23.planning_target_hours,
        zero_kw23.expected_hours
    );
}

/// PSOLL-01 / der Kern des in 64-04 gefundenen Gaps: `expected_hours == 0` mit
/// Zusage > 0.
///
/// Das 64-01-Gate (`has_contract_row && dynamic_hours > 0.0`) lieferte fuer diese
/// Klasse **0.0**, obwohl die Person eine echte Zusage hat. Mit dem neuen Gate an
/// `planning_base_for_target` gilt: `dynamic_hours = 0` => `scale = 1.0`,
/// Basis 10.0, keine Reduktion => Stelle A **10.0**, Stelle B **20.0**
/// (Vertragswochen 22 und 23).
///
/// **Faengt:** das alte Gate bleibt stehen (dann 0.0).
#[tokio::test]
async fn planning_target_zero_expected_hours_with_commitment() {
    // ── Stelle A ──
    let service_week =
        build_get_week_service(fixture_work_details_zero_hours_with_commitment(), vec![]);
    let week_report = &service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein")[0];
    assert!(
        (week_report.planning_target_hours - 10.0).abs() < 0.01,
        "D-64-05 / Stelle A: 0h-Vertrag mit 10h Zusage muss ein Planungs-Soll von 10.0 \
         zeigen, got {}",
        week_report.planning_target_hours
    );
    assert!(
        week_report.planning_target_hours > 0.0,
        "D-64-05 / Stelle A: das 64-01-Gate (`dynamic_hours > 0.0`) lieferte fuer diese \
         Klasse 0.0 — genau das ist der in 64-04 gefundene Gap, got {}",
        week_report.planning_target_hours
    );
    assert!(
        week_report.expected_hours.abs() < 0.01,
        "D-64-05 / Stelle A: das bilanzrelevante Soll bleibt 0.0, got {}",
        week_report.expected_hours
    );

    // ── Stelle B ──
    let service_all = build_all_employees_service(
        fixture_work_details_zero_hours_with_commitment(),
        BTreeMap::new(),
    );
    let all_report = &service_all
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees muss erfolgreich sein")[0];
    assert!(
        (all_report.planning_target_hours - 20.0).abs() < 0.01,
        "D-64-05 / Stelle B: 2 x 10.0 = 20.0 erwartet (KW22 + KW23), got {}",
        all_report.planning_target_hours
    );
    assert!(
        all_report.planning_target_hours > 0.0,
        "D-64-05 / Stelle B: das 64-01-Gate lieferte fuer diese Klasse 0.0, got {}",
        all_report.planning_target_hours
    );
    assert!(
        all_report.expected_hours.abs() < 0.01,
        "D-64-05 / Stelle B: das bilanzrelevante Soll bleibt 0.0, got {}",
        all_report.expected_hours
    );
}

/// Die No-Clamp-Sperre: ein negatives Planungs-Soll ist **assertiertes**
/// Verhalten, kein Unfall.
///
/// 0h-Vertrag mit 10h Zusage plus eine manuelle `Vacation`-Zeile mit 20.0h in
/// KW 23. Basis 10.0, `scale = 1.0` (weil `dynamic_hours == 0`), Reduktion 20.0
/// => **-10.0**. Der Bilanzpfad klemmt ebenfalls nicht; ein `.max(0.0)` wuerde
/// hier 0.0 liefern und diesen Test brechen.
///
/// Stelle B kann den Wochenwert nicht einzeln zeigen (der Report traegt nur die
/// Jahressumme): dort wird die Zerlegung ueber zwei Fenster gemessen —
/// bis KW 22 (nur die ungestoerte Woche, 10.0) gegen bis KW 23 (10.0 + (-10.0) =
/// 0.0). Die Differenz der beiden Fenster ist der Wochenbeitrag der KW 23 und
/// muss **-10.0** sein; mit einem Clamp waere sie 0.0.
#[tokio::test]
async fn planning_target_not_clamped_when_reductions_exceed_base() {
    // ── Stelle A ──
    let service_week = build_get_week_service_with_extra_hours(
        fixture_work_details_zero_hours_with_commitment(),
        vec![],
        vec![manual_vacation_20h_week23()],
    );
    let week_report = &service_week
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week muss erfolgreich sein")[0];
    assert!(
        (week_report.planning_target_hours - (-10.0)).abs() < 0.01,
        "Kein `.max(0.0)`: Basis 10.0 minus Reduktion 20.0 muss -10.0 ergeben. \
         0.0 waere ein Clamp — der Bilanzpfad klemmt ebenfalls nicht, got {}",
        week_report.planning_target_hours
    );
    assert!(
        week_report.planning_target_hours < 0.0,
        "Kein `.max(0.0)`: das Planungs-Soll MUSS hier negativ sein, got {}",
        week_report.planning_target_hours
    );

    // ── Stelle B: Zerlegung ueber zwei Fenster ──
    let service_all_22 = build_all_employees_service_with_extra_hours(
        fixture_work_details_zero_hours_with_commitment(),
        BTreeMap::new(),
        vec![manual_vacation_20h_week23()],
    );
    let until_22 = service_all_22
        .get_reports_for_all_employees(2024, 22, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees bis KW22 muss erfolgreich sein")[0]
        .planning_target_hours;
    let service_all_23 = build_all_employees_service_with_extra_hours(
        fixture_work_details_zero_hours_with_commitment(),
        BTreeMap::new(),
        vec![manual_vacation_20h_week23()],
    );
    let until_23 = service_all_23
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("get_reports_for_all_employees bis KW23 muss erfolgreich sein")[0]
        .planning_target_hours;

    assert!(
        (until_22 - 10.0).abs() < 0.01,
        "Stelle B: das Fenster bis KW22 enthaelt nur die ungestoerte Vertragswoche \
         => 10.0 erwartet, got {until_22}"
    );
    let week23_contribution = until_23 - until_22;
    assert!(
        (week23_contribution - (-10.0)).abs() < 0.01,
        "Kein `.max(0.0)` an Stelle B: der Wochenbeitrag der KW 23 muss -10.0 sein \
         (Fenster bis KW23 = {until_23}, bis KW22 = {until_22}). Mit einem Clamp waere \
         die Differenz 0.0"
    );
}
