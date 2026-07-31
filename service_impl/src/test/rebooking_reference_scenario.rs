//! Phase 57 Plan 04 — das Referenz-Szenario aus **D-57-02** als ausführbarer
//! Ende-zu-Ende-Test. Dies ist die Akzeptanzbedingung von **VOL-ACCT-04** (und
//! die Gegenprobe von **REB-AUTO-06**), in den Worten des Users:
//!
//! > „Vertrag 5 h bezahlt + 5 h freiwillig pro Woche; Woche 1 drei Stunden,
//! > Woche 2 siebzehn Stunden; das sind 20 Stunden gearbeitet bei 20 Stunden
//! > zugesagt, aber Shifty rechnet 8 zu 12 und zeigt Balance −2."
//!
//! **Was diesen Test von `rebooking_neutrality.rs` unterscheidet.** Dort
//! entstehen die Ehrenamt-Stunden aus direkt gesetzten
//! `VolunteerWork`-ExtraHours — das prüft die Aggregations-Deltas, aber nicht
//! den Pfad, über den die Stunden in der Produktion tatsächlich entstehen. Hier
//! kommen sie ausschließlich aus dem **Shiftplan-Cap-Überlauf**
//! (`cap_planned_hours_to_expected = true`, `apply_weekly_cap`): der
//! `MockShiftplanReportService` liefert echte `ShiftplanReportDay`-Einträge, und
//! `hours_per_week` erzeugt daraus `auto_volunteer_hours`. Die einzigen
//! `ExtraHours`-Rows dieser Datei sind die beiden Beine des Rebooking-Paars —
//! es gibt hier bewusst KEINE manuellen Rows.
//!
//! Statt abstrakter Deltas assertiert dieser Test die konkreten Kontostände:
//! 8 h bezahlt / 12 h Ehrenamt / Balance −2 vor dem Übertrag, 10 h / 10 h /
//! Balance 0 danach.

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
use service::reporting::{EmployeeReport, ReportingService};
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::{MockShiftplanReportService, ShiftplanReportDay};
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};

/// f32-Toleranz für alle Vergleiche dieser Datei (Summationsrauschen).
const EPS: f32 = 1e-3;

#[track_caller]
fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < EPS,
        "{what}: expected {expected}, got {actual} (tolerance {EPS})"
    );
}

// ─── Fixture ──────────────────────────────────────────────────────────────────

fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0057_0001)
}

/// Der Vertrag aus D-57-02, unverändert: 5 h bezahlt pro Woche (Mo–Fr),
/// KW 10..=20/2026, Cap aktiv, dazu 5 h freiwillige Zusage pro Woche.
fn work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0057_0010),
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
        name: Arc::from("Reference 57"),
        background_color: Arc::from("#570002"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Range Mo 2026-03-02 (Montag KW 10/2026) .. So 2026-03-15 (Sonntag KW 11/2026)
/// — genau zwei ISO-Wochen, also `expected_hours` von 10.0 über die Range.
fn range() -> (ShiftyDate, ShiftyDate) {
    (
        ShiftyDate::from_ymd(2026, 3, 2).unwrap(),
        ShiftyDate::from_ymd(2026, 3, 15).unwrap(),
    )
}

/// Das Rebooking-Paar: `-h` `VolunteerWork` und `+h` `ExtraWork`, beide
/// `source = Rebooking`, datiert auf Mi 2026-03-11 (KW 11/2026). Das sind die
/// **einzigen** ExtraHours-Rows dieser Datei — die Ehrenamt-Stunden des
/// Szenarios stammen ausschließlich aus dem Cap-Überlauf.
fn rebooking_pair(h: f32) -> Vec<ExtraHours> {
    let row = |amount: f32, category: ExtraHoursCategory, tag: u128| ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0057_1000 + tag),
        sales_person_id: sp_id(),
        amount,
        category,
        description: Arc::from("reference-57-rebooking"),
        date_time: datetime!(2026 - 03 - 11 09:00:00),
        created: Some(datetime!(2026 - 03 - 11 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source: ExtraHoursSource::Rebooking,
    };
    vec![
        row(-h, ExtraHoursCategory::VolunteerWork, 1),
        row(h, ExtraHoursCategory::ExtraWork, 2),
    ]
}

// ─── Shiftplan-getriebener Harness ────────────────────────────────────────────

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

/// Wie `build_reporting_service` in `rebooking_neutrality.rs`, mit dem einen
/// inhaltlichen Unterschied: der `MockShiftplanReportService` liefert echte
/// Shiftplan-Stunden — eine Row für KW 10 und eine für KW 11. Die
/// Tages-Granularität ist irrelevant, weil `hours_per_week` die Stunden pro
/// Woche aufsummiert; deshalb genügt je ein Montags-Eintrag.
fn build_reporting_service(
    extra_hours: Vec<ExtraHours>,
    week_10_hours: f32,
    week_11_hours: f32,
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

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![work_details()])));

    let shiftplan_days: Arc<[ShiftplanReportDay]> = Arc::from(vec![
        ShiftplanReportDay {
            sales_person_id: sp_id(),
            hours: week_10_hours,
            year: 2026,
            calendar_week: 10,
            day_of_week: DayOfWeek::Monday,
        },
        ShiftplanReportDay {
            sales_person_id: sp_id(),
            hours: week_11_hours,
            year: 2026,
            calendar_week: 11,
            day_of_week: DayOfWeek::Monday,
        },
    ]);
    let shiftplan_days_week = shiftplan_days.clone();
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| Ok(shiftplan_days.clone()));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(move |_, _, _, _| Ok(shiftplan_days_week.clone()));

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

/// Der Range-Report über beide Wochen. `include_carryover` ist `false`, weil das
/// Szenario keine Vorjahresgeschichte hat.
async fn report(extra_hours: Vec<ExtraHours>, week_10_hours: f32, week_11_hours: f32) -> EmployeeReport {
    let (from, to) = range();
    build_reporting_service(extra_hours, week_10_hours, week_11_hours)
        .get_report_for_employee_range(&sp_id(), from, to, false, Authentication::Full, None)
        .await
        .expect("report must succeed")
}

// ─── Referenz-Szenario D-57-02 ────────────────────────────────────────────────

/// **VOL-ACCT-04, Basis-Szenario (D-57-02).**
///
/// Herleitung der Ausgangslage — die Ehrenamt-Stunden entstehen komplett aus dem
/// Cap-Überlauf, es gibt keine manuellen ExtraHours:
///
/// | Woche | `raw_shiftplan` | `expected` | Cap? | bezahlt | Ehrenamt |
/// |---|---|---|---|---|---|
/// | KW 10 | 3 | 5 | nein | 3 | 0 |
/// | KW 11 | 17 | 5 | ja | 5 | 12 |
/// | Summe | 20 | 10 | | **8** | **12** |
///
/// Balance = bezahlt 8 − erwartet 10 = **−2**. Der Mitarbeiter hat 20 h
/// gearbeitet bei 20 h zugesagt (10 bezahlt + 10 freiwillig) und steht trotzdem
/// mit −2 im Minus — genau der Befund, den diese Phase behebt.
///
/// Nach dem Übertrag von 2 h (Rebooking-Paar): das `+2`-ExtraWork-Bein zählt in
/// `overall_hours` (Invariante I-1), das `−2`-VolunteerWork-Bein senkt
/// `volunteer_hours` (Invariante I-2) → 10 h bezahlt / 10 h Ehrenamt /
/// Balance 0.
///
/// Die bezahlten Stunden trägt `EmployeeReport::overall_hours` (gedeckelte
/// Shiftplan-Stunden + balance-relevante ExtraHours); äquivalent gilt
/// `balance_hours + expected_hours`.
#[tokio::test]
async fn rebooking_reference_scenario_balances_account() {
    // ── Ausgangslage: ohne Rebooking-Paar ───────────────────────────────────
    let before = report(Vec::new(), 3.0, 17.0).await;
    assert_close(before.expected_hours, 10.0, "expected_hours (2 Wochen à 5 h)");
    assert_close(before.overall_hours, 8.0, "bezahlte Stunden vor dem Übertrag");
    assert_close(before.volunteer_hours, 12.0, "volunteer_hours vor dem Übertrag");
    assert_close(before.balance_hours, -2.0, "balance_hours vor dem Übertrag");
    assert_close(
        before.balance_hours + before.expected_hours,
        8.0,
        "bezahlte Stunden als balance + expected (Gegenprobe zu overall_hours)",
    );

    // ── Nach dem Übertrag von 2 h freiwillig → bezahlt ──────────────────────
    let after = report(rebooking_pair(2.0), 3.0, 17.0).await;
    assert_close(after.expected_hours, 10.0, "expected_hours bleibt unberührt");
    assert_close(after.overall_hours, 10.0, "bezahlte Stunden nach dem Übertrag");
    assert_close(after.volunteer_hours, 10.0, "volunteer_hours nach dem Übertrag");
    assert_close(after.balance_hours, 0.0, "balance_hours nach dem Übertrag");
}
