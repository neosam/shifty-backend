//! Phase 57 Plan 02 (D-57-08) — Rebooking-**Wirksamkeits**-Property-Tests
//! (VOL-ACCT-04) + der eine verbliebene Neutralitäts-Test (VOL-ACCT-03).
//!
//! **Lage nach D-57-08.** Eine EINZELNE Umbuchung (das `-h/+h`-ExtraHours-Pair
//! mit `source=Rebooking`) ist auf der Leseseite ausdrücklich NICHT neutral —
//! sie ist genau dann korrekt, wenn sie wirkt:
//!
//! 1. `test_neutrality_balance` — `EmployeeReport::balance_hours` steigt um `h`
//!    (VOL-ACCT-04, Invariante I-1). Das `+h`-ExtraWork-Bein ist balance-relevant,
//!    das `-h`-VolunteerWork-Bein ist es nicht.
//! 2. `test_neutrality_f1_ist` — `EmployeeReport::volunteer_hours` (= F1-Ist,
//!    identisch zu `VoluntaryStats::ist_total`) sinkt um `h` (VOL-ACCT-04,
//!    Invariante I-2). Die freiwillig geleisteten Stunden sind nach der Umbuchung
//!    bezahlt, zählen also nicht mehr im Ehrenamt-Ist.
//! 3. `test_neutrality_f2_soll` — `VoluntaryStats::soll_total` bleibt
//!    **bit-identisch** (VOL-ACCT-03 Neufassung, Invariante I-6). Das Soll stammt
//!    aus `committed_voluntary` im Arbeitsvertrag und liest überhaupt keine
//!    `extra_hours` — es ist strukturell rebooking-blind. Das ist der einzige
//!    verbliebene Neutralitäts-Test dieser Datei.
//! 4. `rebooking_effective_in_weekly_summary` — dieselbe Wirkung über
//!    `get_week` → `assemble_weeks`, also über den Wochenübersichts-Pfad
//!    (Site 1), den die drei Range-Tests oben nicht erreichen.
//!
//! **Warum die Assertionen 1 und 2 invertiert wurden.** Die Fassung bis Phase 56
//! forderte Bit-Identität auf allen drei Aggregaten und begründete das mit dem
//! Phase-55-Filter (**D-55-A1**): `source == ExtraHoursSource::Manual` an jedem
//! Read-Aggregations-Eingang (`assemble_weeks`, `get_reports_for_all_employees`,
//! `hours_per_week`, `overall_extra_work_hours`). Dieser Filter verwarf aber
//! BEIDE Beine des Pairs — auch das `+h`, das die Balance anheben soll. Der
//! Übertrag freiwillig→bezahlt war damit wirkungslos (empirisch `−53 → −53`), und
//! die Tests hier zementierten genau diesen Fehler als Erfolg. Phase 57 korrigiert
//! zuerst die Spec (VOL-ACCT-03 Neufassung, VOL-ACCT-04) und dann den Filter.
//!
//! **Rolle dieser Datei ab jetzt:** Sie ist der CI-Wächter dagegen, dass der
//! Übertrag erneut wirkungslos wird. Diese Rolle wurde bewusst NICHT dem
//! HR-Alert-Banner zugewiesen (D-57-04) — das Banner-Verhalten aus D-55-07 bleibt
//! in Phase 57 unangetastet. Schlägt eine Assertion hier fehl, ist eine
//! Aggregations-Kette wieder rebooking-blind geworden.
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

// ─── Vergleichs-Helper ────────────────────────────────────────────────────────

/// Prüft, dass sich ein Aggregat um genau `expected_delta` verändert hat
/// (Toleranz 1e-3, f32-Summationsrauschen). NaN ist wie bei [`assert_bit_eq`]
/// explizit verboten. Für die beiden Aggregate, die nach VOL-ACCT-04 wirken
/// MÜSSEN (Balance `+h`, Ehrenamt-Ist `−h`).
#[track_caller]
fn assert_delta(before: f32, after: f32, expected_delta: f32, aggregate: &str) {
    assert!(
        !before.is_nan() && !after.is_nan(),
        "{aggregate} is NaN — cannot verify rebooking effect (before={before}, after={after})"
    );
    let actual_delta = after - before;
    assert!(
        (actual_delta - expected_delta).abs() < 1e-3,
        "{aggregate} must change by {expected_delta} after the rebooking pair (VOL-ACCT-04): \
         before={before}, after={after}, expected delta={expected_delta}, actual delta={actual_delta} \
         — an aggregation chain is rebooking-blind again"
    );
}

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

/// Dynamischer Vertrag: Vertragszeile vorhanden, aber `is_dynamic = true` /
/// `expected_hours = 0.0` → `weight_for_week` liefert
/// `working_hours_for_week == 0.0`, und `hours_per_week` faellt in den
/// Soll=Ist-Zweig (Fall 2 der Drei-Faelle-Unterscheidung `quick-260624-ujk`).
/// Das ist der Zweig, in dem vor dem CR-03-Fix Stunden verschwanden.
fn dynamic_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        expected_hours: 0.0,
        is_dynamic: true,
        ..work_details()
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

/// Baseline + EIN Rebooking-Pair (VolunteerToExtra h=2.0), beide
/// source=Rebooking. Muss die Balance um +2.0 anheben und das Ehrenamt-Ist um
/// −2.0 senken (VOL-ACCT-04); das VOL-ACCT-01-Soll bleibt unberührt.
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
    build_reporting_service_with(extra_hours, work_details())
}

/// Wie [`build_reporting_service`], aber mit frei waehlbarer Vertragszeile —
/// gebraucht fuer den dynamischen Vertrag (CR-03).
fn build_reporting_service_with(
    extra_hours: Vec<ExtraHours>,
    wd: EmployeeWorkDetails,
) -> ReportingServiceImpl<ReportingDeps> {
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

    let wd_arc: Arc<[EmployeeWorkDetails]> = Arc::from(vec![wd]);
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(wd_arc.clone()));

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
    balance_and_ist_with(extra_hours, work_details()).await
}

/// Wie [`balance_and_ist`], aber mit frei waehlbarer Vertragszeile (CR-03).
async fn balance_and_ist_with(extra_hours: Vec<ExtraHours>, wd: EmployeeWorkDetails) -> (f32, f32) {
    let (from, to) = range();
    let service = build_reporting_service_with(extra_hours, wd);
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

// ─── Property-Tests (VOL-ACCT-04 / VOL-ACCT-03) ───────────────────────────────

/// VOL-ACCT-04 (Invariante I-1): `balance_hours` steigt nach einem
/// Rebooking-Pair über `h = 2.0` um genau 2.0. Das `+h`-ExtraWork-Bein zählt in
/// die Balance, das `-h`-VolunteerWork-Bein nicht — genau das ist die
/// Ausgleichswirkung des Übertrags freiwillig→bezahlt. Delta 0.0 hieße: eine
/// Aggregations-Kette verwirft wieder beide Beine (der Phase-55-Zustand).
#[tokio::test]
async fn test_neutrality_balance() {
    let (bal_before, _) = balance_and_ist(baseline_extra_hours()).await;
    let (bal_after, _) = balance_and_ist(with_rebooking_pair()).await;
    assert_delta(bal_before, bal_after, 2.0, "balance_hours");
}

/// VOL-ACCT-04 (Invariante I-2): F1-Ist (`volunteer_hours` ==
/// `VoluntaryStats::ist_total`) sinkt nach demselben Pair um genau 2.0 — die
/// umgebuchten Stunden sind jetzt bezahlt und zählen nicht mehr als Ehrenamt.
#[tokio::test]
async fn test_neutrality_f1_ist() {
    let (_, ist_before) = balance_and_ist(baseline_extra_hours()).await;
    let (_, ist_after) = balance_and_ist(with_rebooking_pair()).await;
    assert_delta(ist_before, ist_after, -2.0, "f1_ist (volunteer_hours)");
}

/// VOL-ACCT-03 (Neufassung, Invariante I-6): F2-Soll
/// (`VoluntaryStats::soll_total`) bleibt bit-identisch — Soll liest überhaupt
/// keine ExtraHours (nur committed_voluntary + Absences), ist also per
/// Konstruktion rebooking-blind. Der Test exerziert den vollen HR-gated
/// VoluntaryStats-Pfad, um diese Unabhängigkeit empirisch zu belegen.
///
/// Nach der Neufassung von VOL-ACCT-03 ist das der **einzige verbliebene
/// Neutralitäts-Test** dieser Datei; Balance und Ehrenamt-Ist sollen sich
/// ausdrücklich verändern (VOL-ACCT-04).
#[tokio::test]
async fn test_neutrality_f2_soll() {
    let soll_before = soll_total(baseline_extra_hours()).await;
    let soll_after = soll_total(with_rebooking_pair()).await;
    assert_bit_eq(soll_before, soll_after, "f2_soll (soll_total)");
}

// ─── Site-1-Wächter: Wochenübersicht über get_week → assemble_weeks ────────────

/// Wie [`build_reporting_service_with`], aber mit den Mock-Erwartungen, die der
/// `get_week`-Pfad zusätzlich braucht (`all_for_week`,
/// `extract_shiftplan_report_for_week`, `find_by_week`, `get_all`, `find_all`).
/// Die Wirkung soll ausschließlich aus den ExtraHours kommen: kein Shiftplan,
/// keine Absence, kein Carryover, kein Holiday-Toggle.
///
/// Die Vertragszeile ist frei wählbar — gebraucht für den dynamischen Vertrag
/// (CR-03).
fn build_reporting_service_for_week_with(
    extra_hours: Vec<ExtraHours>,
    wd: EmployeeWorkDetails,
) -> ReportingServiceImpl<ReportingDeps> {
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
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![make_sales_person()])));

    let wd_arc: Arc<[EmployeeWorkDetails]> = Arc::from(vec![wd]);
    let wd_arc_week = wd_arc.clone();
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(wd_arc.clone()));
    employee_work_details_service
        .expect_all_for_week()
        .returning(move |_, _, _, _| Ok(wd_arc_week.clone()));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::new())));

    let eh_arc: Arc<[ExtraHours]> = Arc::from(extra_hours);
    let eh_arc_week = eh_arc.clone();
    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(eh_arc.clone()));
    extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));
    extra_hours_service
        .expect_find_by_week()
        .returning(move |_, _, _, _| Ok(eh_arc_week.clone()));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

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

/// Liefert `(balance_hours, volunteer_hours)` aus der Wochenübersicht
/// (`get_week` → `assemble_weeks`) für KW 12/2026. Die Fixture `eh()` datiert
/// alle Rows auf Mi 2026-03-18 (KW 12/2026), `work_details()` deckt KW 10..=20
/// ab.
async fn weekly_balance_and_ist(extra_hours: Vec<ExtraHours>) -> (f32, f32) {
    weekly_balance_and_ist_with(extra_hours, work_details()).await
}

/// Wie [`weekly_balance_and_ist`], aber mit frei waehlbarer Vertragszeile (CR-03).
async fn weekly_balance_and_ist_with(
    extra_hours: Vec<ExtraHours>,
    wd: EmployeeWorkDetails,
) -> (f32, f32) {
    let service = build_reporting_service_for_week_with(extra_hours, wd);
    let reports = service
        .get_week(2026, 12, Authentication::Full, None)
        .await
        .expect("weekly summary must succeed");
    let report = reports
        .iter()
        .find(|r| r.sales_person.id == sp_id())
        .expect("weekly summary must contain the fixture sales person");
    (report.balance_hours, report.volunteer_hours)
}

/// VOL-ACCT-04 über die **Wochenübersicht** — der Wächter für Site 1
/// (`assemble_weeks`).
///
/// Warum dieser Test zusätzlich existiert: Die drei Tests oben fahren alle über
/// `get_report_for_employee_range` und erreichen damit nur die Sites 3/4, die
/// über `as_report_type()` filtern. `assemble_weeks` filtert seine
/// `extra_working_hours`-Summe dagegen über `availability()`, und
/// `VolunteerWork.availability()` ist `Available`. Dort genügt das bloße
/// Streichen des `source`-Prädikats also NICHT: das `-h`-VolunteerWork-Bein
/// würde mitgezählt und die Balance-Anhebung des `+h`-Beins netto wieder
/// neutralisieren (Delta 0.0). Dieser Test ist der einzige, der einen
/// übersehenen oder falsch gefixten Site 1 rot färbt.
#[tokio::test]
async fn rebooking_effective_in_weekly_summary() {
    let (bal_before, ist_before) = weekly_balance_and_ist(baseline_extra_hours()).await;
    let (bal_after, ist_after) = weekly_balance_and_ist(with_rebooking_pair()).await;

    assert_delta(bal_before, bal_after, 2.0, "weekly balance_hours");
    assert_delta(ist_before, ist_after, -2.0, "weekly volunteer_hours");
}

// ─── CR-03: dynamischer Vertrag (Soll=Ist-Zweig) ──────────────────────────────

/// CR-03 (Code-Review Phase 57): beim **dynamischen** Vertrag (Vertragszeile
/// vorhanden, `working_hours_for_week == 0.0`) darf das Rebooking-Paar keine
/// Stunden verschwinden lassen.
///
/// `hours_per_week` behandelt diese Woche als Soll=Ist:
/// `expected_hours = shiftplan_hours + extra_work_hours`. Das `+h`-ExtraWork-Bein
/// des Paars hob damit **zugleich** `overall` und `expected` — die Balance blieb
/// unverändert. Das `−h`-VolunteerWork-Bein wird in `volunteer_hours` dagegen
/// sehr wohl gezählt (reiner Kategorie-Filter). Netto verließen `h` Stunden das
/// Ehrenamt-Konto und kamen im bezahlten Konto nie an. Vor Phase 57 war das Paar
/// hier durch den pauschalen Marker-Filter ein harmloses No-op — die
/// Filter-Entfernung hat den Fehler erst freigelegt.
///
/// Erreichbar über den HR-Pfad: weder `rebook_manual` noch `approve` prüfen, ob
/// die Zielperson in der Zielwoche einen nicht-dynamischen Vertrag hat.
///
/// Der Test pinnt drei Dinge:
/// 1. Die Bilanz-Erhaltung `Δbalance + Δvolunteer == 0` — die Formulierung von
///    „keine Stunden verschwinden". Vor dem Fix: `0 + (−2) == −2`.
/// 2. Die Wirkungsrichtung (`+h` Balance, `−h` Ehrenamt), identisch zum
///    nicht-dynamischen Fall.
/// 3. Dass Jahres-Report (`hours_per_week`) und Wochenübersicht
///    (`assemble_weeks`) für dasselbe Paar **dieselbe** Wirkung ausweisen. Die
///    absoluten Balancen dürfen weiterhin auseinanderliegen (RISK-01 +
///    die Soll=Ist-Regel existiert nur in `hours_per_week`) — die Wirkung des
///    Paars darf es nicht.
#[tokio::test]
async fn rebooking_conserves_hours_on_dynamic_contract() {
    let (range_bal_before, range_ist_before) =
        balance_and_ist_with(baseline_extra_hours(), dynamic_work_details()).await;
    let (range_bal_after, range_ist_after) =
        balance_and_ist_with(with_rebooking_pair(), dynamic_work_details()).await;

    let (week_bal_before, week_ist_before) =
        weekly_balance_and_ist_with(baseline_extra_hours(), dynamic_work_details()).await;
    let (week_bal_after, week_ist_after) =
        weekly_balance_and_ist_with(with_rebooking_pair(), dynamic_work_details()).await;

    let range_bal_delta = range_bal_after - range_bal_before;
    let range_ist_delta = range_ist_after - range_ist_before;
    let week_bal_delta = week_bal_after - week_bal_before;
    let week_ist_delta = week_ist_after - week_ist_before;

    // (1) Bilanz-Erhaltung: was das Ehrenamt-Konto verlässt, kommt im bezahlten an.
    assert!(
        (range_bal_delta + range_ist_delta).abs() < 1e-3,
        "dynamischer Vertrag: das Rebooking-Paar darf keine Stunden vernichten — \
         Δbalance={range_bal_delta}, Δvolunteer={range_ist_delta}, Summe={} \
         (erwartet 0). Das +h-ExtraWork-Bein leakt ins Soll der Soll=Ist-Formel.",
        range_bal_delta + range_ist_delta
    );

    // (2) Wirkungsrichtung wie im nicht-dynamischen Fall (I-1 / I-2).
    assert_delta(
        range_bal_before,
        range_bal_after,
        2.0,
        "dynamic range balance_hours",
    );
    assert_delta(
        range_ist_before,
        range_ist_after,
        -2.0,
        "dynamic range volunteer_hours",
    );

    // (3) Wochenübersicht und Jahres-Report weisen dieselbe Wirkung aus.
    assert!(
        (range_bal_delta - week_bal_delta).abs() < 1e-3,
        "hours_per_week und assemble_weeks muessen fuer dasselbe Paar dieselbe \
         Balance-Wirkung ausweisen: Jahres-Report {range_bal_delta}, \
         Wochenuebersicht {week_bal_delta}"
    );
    assert!(
        (range_ist_delta - week_ist_delta).abs() < 1e-3,
        "hours_per_week und assemble_weeks muessen fuer dasselbe Paar dieselbe \
         Ehrenamt-Wirkung ausweisen: Jahres-Report {range_ist_delta}, \
         Wochenuebersicht {week_ist_delta}"
    );
}
