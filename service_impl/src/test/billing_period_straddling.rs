//! Phase 56 Plan 01 — D-01 Straddling-Golden-Snapshot-Fixture
//! (REB-AUTO-05, D-01-BEWEISLAST / proof-of-burden).
//!
//! **Was dieser Test beweist:** Eine Billing-Period, deren Zeitraum eine
//! Rebooking-Woche (`source=Rebooking`-Pair) enthält, produziert **byte-
//! identische** Snapshot-Value-Rows, egal ob der `voluntary_rebooking_auto_active_from`-
//! Stichtag NACH, VOR oder INNERHALB des Zeitraums liegt. Das ist die Beweislast
//! für den "Nein"-Zweig von D-01: Solange die drei Szenarien byte-identisch
//! sind, bleibt `CURRENT_SNAPSHOT_SCHEMA_VERSION` bei **12** (keine
//! F4-Cron-Rebooking-Quelle ändert einen persistierten `value_type`).
//!
//! **Warum das grün sein MUSS (56-01 key_finding):** Alle Billing-Period-
//! Value-Rows stammen aus `ReportingService::get_report_for_employee_range`
//! (`billing_period_report.rs:146..187`). Diese fn filtert unbedingt
//! `source == ExtraHoursSource::Manual` (Phase 55, D-55-A1) — der Filter ist
//! toggle-UNABHÄNGIG. Der Stichtag-Toggle steuert erst den Auto-Cron-Write
//! (Plan 02/03), NICHT die Read-Aggregation. Die Rebooking-Pair-Rows sind für
//! den Snapshot also unsichtbar, unabhängig vom Toggle-Wert.
//!
//! **Wenn ein Szenario DIVERGIERT:** Der Fixture hat das Gegenteil bewiesen —
//! eine Kette leakt Rebooking-Rows in den Snapshot. Dann kippt D-01 auf einen
//! Bump: Plan 06 muss `CURRENT_SNAPSHOT_SCHEMA_VERSION` 12→13 heben + Migration
//! + F08-Docs. NICHT die Assertion anpassen — im SUMMARY melden.
//!
//! Muster: reales `BillingPeriodReportServiceImpl` mit realem
//! `ReportingServiceImpl` (Mock-ExtraHours mit Rebooking-Pair) als Reporting-
//! Dep; `to_bits()`-Byte-Identität pro Value-Feld
//! (Muster `booking_information_weekly_summary_year_batch.rs`, D-52-12).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::billing_period::{BillingPeriodSalesPerson, BillingPeriodValueType};
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::billing_period_report::{
    BillingPeriodReportServiceDeps, BillingPeriodReportServiceImpl,
};
use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};

// ─── Byte-identity helper ─────────────────────────────────────────────────────

#[track_caller]
fn assert_f32_bit_eq(actual: f32, expected: f32, value_type: &str, field: &str) {
    assert!(
        !actual.is_nan() && !expected.is_nan(),
        "value_type `{value_type}` field `{field}` is NaN — cannot compare snapshot bit-exact (actual={actual}, expected={expected})"
    );
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "value_type `{value_type}` field `{field}` diverges: actual={actual} (0x{:08x}), expected={expected} (0x{:08x}) — a chain leaked the rebooking pair into the snapshot => D-01 flips to a 12->13 bump",
        actual.to_bits(),
        expected.to_bits(),
    );
}

/// Vergleicht zwei `BillingPeriodSalesPerson`-Snapshots value-row für value-row
/// byte-exakt. Verlangt identischen Value-Type-Schlüsselsatz.
#[track_caller]
fn assert_snapshot_bit_eq(actual: &BillingPeriodSalesPerson, expected: &BillingPeriodSalesPerson) {
    assert_eq!(
        actual.sales_person_id, expected.sales_person_id,
        "sales_person_id must match"
    );
    let actual_keys: Vec<_> = actual.values.keys().cloned().collect();
    let expected_keys: Vec<_> = expected.values.keys().cloned().collect();
    assert_eq!(
        actual_keys, expected_keys,
        "value_type key set must be identical between scenarios"
    );
    for (vt, exp_val) in expected.values.iter() {
        let act_val = actual
            .values
            .get(vt)
            .unwrap_or_else(|| panic!("value_type {vt:?} missing in actual snapshot"));
        let name = format!("{vt:?}");
        assert_f32_bit_eq(act_val.value_delta, exp_val.value_delta, &name, "value_delta");
        assert_f32_bit_eq(
            act_val.value_ytd_from,
            exp_val.value_ytd_from,
            &name,
            "value_ytd_from",
        );
        assert_f32_bit_eq(act_val.value_ytd_to, exp_val.value_ytd_to, &name, "value_ytd_to");
        assert_f32_bit_eq(
            act_val.value_full_year,
            exp_val.value_full_year,
            &name,
            "value_full_year",
        );
    }
}

// ─── Fixture ──────────────────────────────────────────────────────────────────

fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_5701)
}

/// Gedeckelter Vertrag 5h/Woche Mo-Fr über das ganze Jahr 2026, cap aktiv,
/// committed_voluntary=5.0.
fn work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_5710),
        sales_person_id: sp_id(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 53,
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
        name: Arc::from("Straddling 56"),
        background_color: Arc::from("#654321"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

fn eh(amount: f32, category: ExtraHoursCategory, source: ExtraHoursSource, tag: u128) -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_5A00 + tag),
        sales_person_id: sp_id(),
        amount,
        category,
        // Rebooking-Woche liegt in der SPÄTEREN Hälfte des Zeitraums:
        // 2026-04-15 ist ein Mittwoch in KW 16/2026 (Zeitraum Mo 2026-03-02 ..
        // So 2026-05-17, Rebooking-Woche im letzten Drittel).
        description: Arc::from("straddling-56-fixture"),
        date_time: datetime!(2026 - 04 - 15 09:00:00),
        created: Some(datetime!(2026 - 04 - 15 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source,
    }
}

/// Dataset: manuelle Baseline + genau EIN Rebooking-Pair (VolunteerToExtra N=2)
/// in der späteren Hälfte des Zeitraums. Über alle drei Toggle-Szenarien
/// identisch — nur der Stichtag-Toggle-Wert unterscheidet sich.
fn straddling_extra_hours() -> Vec<ExtraHours> {
    vec![
        eh(3.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Manual, 1),
        eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Manual, 2),
        eh(-2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 3),
        eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 4),
    ]
}

/// Billing-Period Mo-2026-03-02 .. So-2026-05-17 (~2.5 Monate, straddelt die
/// Rebooking-Woche 2026-04-15).
fn period() -> (ShiftyDate, ShiftyDate) {
    (
        ShiftyDate::from_ymd(2026, 3, 2).unwrap(),
        ShiftyDate::from_ymd(2026, 5, 17).unwrap(),
    )
}

// ─── Real ReportingServiceImpl mit konfigurierbarem Toggle ────────────────────

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

/// Baut die reale Reporting-Impl. `toggle_value` ist der Stichtag, den der
/// `MockToggleService` für JEDEN Toggle-Lookup zurückgibt (Some(ISO-Datum) /
/// None). In dieser Fixture ist der Wert für das Ergebnis IRRELEVANT (der
/// source==Manual-Filter ist toggle-unabhängig) — genau das beweist der Test.
fn build_reporting_service(
    extra_hours: Vec<ExtraHours>,
    toggle_value: Option<&'static str>,
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
        .returning(move |_, _, _| Ok(toggle_value.map(Arc::from)));

    // Der Billing-Period-Report ruft die Reporting-fn mit carryover=true über
    // volle Jahres-Ranges — das iteriert alle ISO-Wochen und ruft pro Woche
    // `special_day_service.get_by_week`. Keine SpecialDays in dieser Fixture.
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));

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
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        rebooking_batch_service: Arc::new(MockRebookingBatchService::new()),
    }
}

// ─── Real BillingPeriodReportServiceImpl über der realen Reporting-Impl ────────

struct BpDeps;
impl BillingPeriodReportServiceDeps for BpDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type BillingPeriodService = service::billing_period::MockBillingPeriodService;
    type ReportingService = ReportingServiceImpl<ReportingDeps>;
    type SalesPersonService = MockSalesPersonService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type TextTemplateService = service::text_template::MockTextTemplateService;
    type PermissionService = MockPermissionService;
    type UuidService = MockUuidService;
    type ClockService = MockClockService;
    type TransactionDao = dao::MockTransactionDao;
}

fn build_billing_period_service(
    toggle_value: Option<&'static str>,
) -> BillingPeriodReportServiceImpl<BpDeps> {
    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    BillingPeriodReportServiceImpl {
        billing_period_service: Arc::new(service::billing_period::MockBillingPeriodService::new()),
        reporting_service: Arc::new(build_reporting_service(straddling_extra_hours(), toggle_value)),
        sales_person_service: Arc::new(MockSalesPersonService::new()),
        employee_work_details_service: Arc::new(MockEmployeeWorkDetailsService::new()),
        text_template_service: Arc::new(service::text_template::MockTextTemplateService::new()),
        permission_service: Arc::new(MockPermissionService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        clock_service: Arc::new(MockClockService::new()),
        transaction_dao: Arc::new(transaction_dao),
    }
}

/// Berechnet die Snapshot-Value-Rows des SalesPerson für den Zeitraum, mit dem
/// gegebenen Stichtag-Toggle-Wert.
async fn compute_snapshot(toggle_value: Option<&'static str>) -> BillingPeriodSalesPerson {
    let (start, end) = period();
    let service = build_billing_period_service(toggle_value);
    service
        .build_billing_period_report_for_sales_person(
            make_sales_person(),
            start,
            end,
            Authentication::Full,
            None,
        )
        .await
        .expect("billing period report must succeed")
}

// ─── Szenarien (D-01) ─────────────────────────────────────────────────────────
//
// Rebooking-Woche = KW 16/2026 (Mi 2026-04-15). Zeitraum Mo 2026-03-02 ..
// So 2026-05-17.

/// PRE-Referenz: Stichtag NACH dem ganzen Zeitraum (2026-06-01) → die
/// Rebooking-Woche liegt pre-Stichtag. Diese Value-Rows sind die Referenz.
#[tokio::test]
async fn test_straddling_snapshot_pre() {
    let reference = compute_snapshot(Some("2026-06-01")).await;
    // Selbstkonsistenz: idempotent — dieselben Eingaben ergeben denselben Snapshot.
    let again = compute_snapshot(Some("2026-06-01")).await;
    assert_snapshot_bit_eq(&again, &reference);
    // Sanity: mindestens die Kern-Value-Types existieren.
    assert!(reference.values.contains_key(&BillingPeriodValueType::Balance));
    assert!(reference.values.contains_key(&BillingPeriodValueType::ExtraWork));
}

/// POST: Stichtag VOR dem ganzen Zeitraum (2026-01-01) → die Rebooking-Woche
/// liegt post-Stichtag. Byte-identisch zur PRE-Referenz.
#[tokio::test]
async fn test_straddling_snapshot_post() {
    let reference = compute_snapshot(Some("2026-06-01")).await;
    let post = compute_snapshot(Some("2026-01-01")).await;
    assert_snapshot_bit_eq(&post, &reference);
}

/// STRADDLING: Stichtag INNERHALB des Zeitraums (2026-04-06, Montag KW 15),
/// sodass die Rebooking-Woche KW 16 post-cutoff liegt → byte-identisch zur
/// PRE-Referenz. Beweist D-01: Snapshot bleibt bei Version 12.
#[tokio::test]
async fn test_straddling_snapshot_straddling() {
    let reference = compute_snapshot(Some("2026-06-01")).await;
    let straddling = compute_snapshot(Some("2026-04-06")).await;
    assert_snapshot_bit_eq(&straddling, &reference);

    // Zusätzlich: auch der None-Fall (Feature dormant) ist identisch.
    let dormant = compute_snapshot(None).await;
    assert_snapshot_bit_eq(&dormant, &reference);
}
