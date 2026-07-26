//! Phase 56 Plan 01 — D-06 Rebooking-Neutralitäts-Property-Tests (REB-AUTO-02).
//!
//! Diese drei Tests sind das **CI-Gate** für D-06: eine EINZELNE Umbuchung
//! (das -N/+N-ExtraHours-Pair mit `source=Rebooking`) lässt die drei
//! Read-Aggregate **bit-identisch**:
//!
//! 1. `test_neutrality_balance` — `EmployeeReport::balance_hours`
//! 2. `test_neutrality_f1_ist`  — `EmployeeReport::volunteer_hours` (= F1-Ist,
//!    identisch zu `VoluntaryStats::ist_total`)
//! 3. `test_neutrality_f2_soll` — `VoluntaryStats::soll_total`
//!
//! **Warum das auf `to_bits()` grün sein MUSS (56-01 key_finding):** Phase 55
//! (D-55-A1) filtert `source == ExtraHoursSource::Manual` an jedem
//! Read-Aggregations-Eingang (`assemble_weeks`, `get_reports_for_all_employees`,
//! `hours_per_week`, `overall_extra_work_hours`). Rebooking-Rows sind damit für
//! Balance/F1-Ist unsichtbar; F2-Soll liest überhaupt keine ExtraHours (nur
//! `committed_voluntary` + Absences). Diese Tests nageln diese Eigenschaft
//! empirisch fest — mit `active_from = None` (Feature dormant), sodass die
//! Neutralität OHNE den Toggle bewiesen ist.
//!
//! Wenn eine Assertion FEHLSCHLÄGT, ist das eine echte Entdeckung (eine Kette
//! leakt Rebooking-Rows) — NICHT die Assertion anpassen, sondern melden.
//!
//! Harness-Muster gespiegelt aus
//! `service_impl/src/test/rebooking_reconciliation.rs` (Mock-ExtraHours-Service
//! liefert einmal Baseline, einmal Baseline+Rebooking-Pair).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::voluntary_stats::VoluntaryStatsService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::voluntary_stats::{VoluntaryStatsServiceDeps, VoluntaryStatsServiceImpl};

// ─── Byte-identity helper (to_bits, NaN verboten) ─────────────────────────────

/// Vergleicht zwei f32 bit-exakt via `to_bits()`. NaN ist explizit verboten
/// (nicht deterministisch vergleichbar) — panickt vorher.
#[track_caller]
fn assert_bit_eq(before: f32, after: f32, aggregate: &str) {
    assert!(
        !before.is_nan() && !after.is_nan(),
        "{aggregate} is NaN — cannot compare rebooking-neutrality bit-exact (before={before}, after={after})"
    );
    assert_eq!(
        before.to_bits(),
        after.to_bits(),
        "{aggregate} must be rebooking-neutral: before={before} (0x{:08x}), after={after} (0x{:08x}) — a chain leaked the rebooking pair",
        before.to_bits(),
        after.to_bits(),
    );
}

// ─── Fixture ──────────────────────────────────────────────────────────────────

fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_0001)
}

/// Gedeckelter Vertrag 5h/Woche Mo-Fr, KW 10..=20/2026, cap aktiv,
/// committed_voluntary=5.0.
fn work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_0010),
        sales_person_id: sp_id(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 10,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 20,
        to_year: 2026,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: true,
        committed_voluntary: 5.0,
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

fn make_sales_person() -> service::sales_person::SalesPerson {
    service::sales_person::SalesPerson {
        id: sp_id(),
        name: Arc::from("Neutrality 56"),
        background_color: Arc::from("#123456"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// ExtraHours-Row in KW 12/2026 (Mi 2026-03-18).
fn eh(amount: f32, category: ExtraHoursCategory, source: ExtraHoursSource, tag: u128) -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_1000 + tag),
        sales_person_id: sp_id(),
        amount,
        category,
        description: Arc::from("neutrality-56-fixture"),
        date_time: datetime!(2026 - 03 - 18 09:00:00),
        created: Some(datetime!(2026 - 03 - 18 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source,
    }
}

/// Baseline: manuelle VolunteerWork + ExtraWork (source=Manual).
fn baseline_extra_hours() -> Vec<ExtraHours> {
    vec![
        eh(3.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Manual, 1),
        eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Manual, 2),
    ]
}

/// Baseline + EIN Rebooking-Pair (VolunteerToExtra N=2.0), beide
/// source=Rebooking. Muss über alle drei Aggregate hinweg neutral sein.
fn with_rebooking_pair() -> Vec<ExtraHours> {
    let mut v = baseline_extra_hours();
    v.push(eh(-2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 3));
    v.push(eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 4));
    v
}

// ─── Real ReportingServiceImpl mit Mock-ExtraHours ────────────────────────────

struct ReportingDeps;
impl ReportingServiceDeps for ReportingDeps {
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

fn build_reporting_service(extra_hours: Vec<ExtraHours>) -> ReportingServiceImpl<ReportingDeps> {
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
        .returning(|_, _, _| Ok(make_sales_person()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![work_details()])));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let eh_arc: Arc<[ExtraHours]> = Arc::from(extra_hours);
    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(eh_arc.clone()));
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
        rebooking_batch_service: Arc::new(MockRebookingBatchService::new()),
    }
}

/// Range Mo-2026-03-02 .. So-2026-05-17 (deckt KW 10..=20).
fn range() -> (ShiftyDate, ShiftyDate) {
    (
        ShiftyDate::from_ymd(2026, 3, 2).unwrap(),
        ShiftyDate::from_ymd(2026, 5, 17).unwrap(),
    )
}

/// Liefert (balance_hours, volunteer_hours) aus dem Range-Report für die
/// gegebene ExtraHours-Liste.
async fn balance_and_ist(extra_hours: Vec<ExtraHours>) -> (f32, f32) {
    let (from, to) = range();
    let service = build_reporting_service(extra_hours);
    let report = service
        .get_report_for_employee_range(&sp_id(), from, to, false, Authentication::Full, None)
        .await
        .expect("report must succeed");
    (report.balance_hours, report.volunteer_hours)
}

// ─── Real VoluntaryStatsServiceImpl (für F2-Soll) ─────────────────────────────

struct StatsDeps;
impl VoluntaryStatsServiceDeps for StatsDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ReportingService = ReportingServiceImpl<ReportingDeps>;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type SalesPersonService = MockSalesPersonService;
    type AbsenceService = MockAbsenceService;
    type PermissionService = MockPermissionService;
    type TransactionDao = dao::MockTransactionDao;
}

fn build_voluntary_stats_service(
    extra_hours: Vec<ExtraHours>,
) -> VoluntaryStatsServiceImpl<StatsDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(make_sales_person()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![work_details()])));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_by_sales_person()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    VoluntaryStatsServiceImpl {
        reporting_service: Arc::new(build_reporting_service(extra_hours)),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        absence_service: Arc::new(absence_service),
        permission_service: Arc::new(permission_service),
        transaction_dao: Arc::new(transaction_dao),
    }
}

async fn soll_total(extra_hours: Vec<ExtraHours>) -> f32 {
    let (from, to) = range();
    let service = build_voluntary_stats_service(extra_hours);
    let stats = service
        .get_voluntary_stats(sp_id(), from, to, Authentication::Full, None)
        .await
        .expect("voluntary stats must succeed");
    stats.soll_total.expect("HR path yields soll_total")
}

// ─── Property-Tests (D-06) ────────────────────────────────────────────────────

/// D-06 (1/3): `balance_hours` bleibt bit-identisch nach einem Rebooking-Pair.
#[tokio::test]
async fn test_neutrality_balance() {
    let (bal_before, _) = balance_and_ist(baseline_extra_hours()).await;
    let (bal_after, _) = balance_and_ist(with_rebooking_pair()).await;
    assert_bit_eq(bal_before, bal_after, "balance_hours");
}

/// D-06 (2/3): F1-Ist (`volunteer_hours` == `VoluntaryStats::ist_total`) bleibt
/// bit-identisch — die Rebooking-Rows (source=Rebooking) werden aus dem
/// Ist-Aggregat gefiltert.
#[tokio::test]
async fn test_neutrality_f1_ist() {
    let (_, ist_before) = balance_and_ist(baseline_extra_hours()).await;
    let (_, ist_after) = balance_and_ist(with_rebooking_pair()).await;
    assert_bit_eq(ist_before, ist_after, "f1_ist (volunteer_hours)");
}

/// D-06 (3/3): F2-Soll (`VoluntaryStats::soll_total`) bleibt bit-identisch —
/// Soll liest überhaupt keine ExtraHours (nur committed_voluntary + Absences),
/// ist also per Konstruktion rebooking-blind. Der Test exerziert den vollen
/// HR-gated VoluntaryStats-Pfad, um diese Unabhängigkeit empirisch zu belegen.
#[tokio::test]
async fn test_neutrality_f2_soll() {
    let soll_before = soll_total(baseline_extra_hours()).await;
    let soll_after = soll_total(with_rebooking_pair()).await;
    assert_bit_eq(soll_before, soll_after, "f2_soll (soll_total)");
}
