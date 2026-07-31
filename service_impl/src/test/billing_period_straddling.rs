//! Straddling-Golden-Snapshot-Fixture — zwei getrennte Beweislasten.
//!
//! Ursprung: Phase 56 Plan 01 (REB-AUTO-05, D-01-BEWEISLAST). Seit **Phase 57**
//! (VOL-ACCT-04, D-57-10/D-57-11) beweist die Datei zwei verschiedene Dinge, die
//! man nicht verwechseln darf.
//!
//! ## Beweislast 1 — der Stichtag-Toggle steuert nur die Schreibseite (Invariante I-8)
//!
//! Auf **demselben** Datensatz liefern alle vier Toggle-Szenarien — Stichtag NACH
//! dem Zeitraum (pre), VOR dem Zeitraum (post), INNERHALB des Zeitraums
//! (straddling) und `None` (Feature dormant) — **byte-identische**
//! Snapshot-Value-Rows. Grund: `voluntary_rebooking_auto_active_from` wird
//! ausschließlich vom Auto-Cron-Write ausgewertet; die Read-Aggregation in
//! `ReportingService::get_report_for_employee_range` liest den Stichtag für
//! Rebooking-Rows gar nicht. Das ist die Regressions-Absicherung und wird von
//! `test_straddling_snapshot_pre` / `_post` / `_straddling` unverändert getragen.
//!
//! ## Beweislast 2 — ein Rebooking-Paar IST seit Phase 57 im Snapshot sichtbar
//!
//! Zwei Datensätze, die sich nur um ein Rebooking-Paar über `h` unterscheiden,
//! liefern **verschiedene** Snapshots: das `+h ExtraWork`-Bein hebt Overall,
//! Balance und ExtraWork, das `−h VolunteerWork`-Bein senkt Volunteer.
//! Grund: Plan 57-02 hat den pauschalen `source == ExtraHoursSource::Manual`-
//! Vorfilter (Phase 55, D-55-A1) aus den Read-Aggregaten entfernt (D-57-06).
//!
//! Genau diese Verschiebung ist die Begründung für den Bump
//! `CURRENT_SNAPSHOT_SCHEMA_VERSION` **12 → 13** (D-57-10): eine frische
//! Neuberechnung liefert für dieselbe Periode andere Werte als ein v12-Snapshot.
//! Sie wird von `test_straddling_snapshot_rebooking_shifts_values` getragen.
//!
//! **Alarm-Richtung (seit Phase 57 umgedreht):** Wenn der with-vs-without-Vergleich
//! KEINE Verschiebung mehr zeigt, ist der Filter-Fix aus 57-02 wieder wirkungslos
//! geworden — eine Kette filtert das Rebooking-Paar erneut heraus. Dann NICHT die
//! Assertion abschwächen, sondern melden. Umgekehrt gilt für Beweislast 1
//! unverändert: divergieren zwei Toggle-Szenarien auf demselben Datensatz, leckt
//! der Stichtag in die Leseseite.
//!
//! Muster: reales `BillingPeriodReportServiceImpl` mit realem
//! `ReportingServiceImpl` (Mock-ExtraHours mit/ohne Rebooking-Pair) als Reporting-
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
        "value_type `{value_type}` field `{field}` diverges: actual={actual} (0x{:08x}), expected={expected} (0x{:08x}) — je nach Aufrufer bedeutet das: (a) assert_snapshot_bit_eq => der Stichtag-Toggle leckt in die Read-Aggregation (Invariante I-8 verletzt), oder (b) assert_snapshot_shift => ein value_type verschiebt sich durch das Rebooking-Paar, obwohl er das nicht darf",
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

/// Derselbe Datensatz OHNE das Rebooking-Paar — die Vergleichsseite für
/// Beweislast 2 (D-57-11). Identische manuelle Baseline (3.0 VolunteerWork,
/// 2.0 ExtraWork, beide `source = Manual`, Tags 1 und 2), nur die beiden
/// Rebooking-Beine fehlen.
///
/// HINWEIS (Pitfall 5): Die Baseline von 3.0 VolunteerWork und der
/// Rebooking-Betrag von 2.0 sind aufeinander abgestimmt und dürfen NICHT
/// verändert werden. `BillingPeriodValueType::Volunteer` wird nur geschrieben,
/// wenn `report_delta.volunteer_hours != 0.0`
/// (`billing_period_report.rs:283`). Ein Betrag, der die Baseline exakt
/// neutralisiert, ließe den Volunteer-Key aus der Snapshot-Map fallen — der
/// Shift-Test schlüge dann mit einer irreführenden "key set differs"-Meldung
/// fehl statt mit einer Werte-Abweichung. Mit 3.0 − 2.0 = 1.0 ≠ 0 existiert der
/// Key in BEIDEN Snapshots.
fn straddling_extra_hours_without_rebooking() -> Vec<ExtraHours> {
    vec![
        eh(3.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Manual, 1),
        eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Manual, 2),
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
/// None). Der Wert ist für das Ergebnis IRRELEVANT — nicht mehr, weil ein
/// Marker-Filter ihn ausblendet (den gibt es seit 57-02 nicht mehr), sondern
/// weil der Stichtag ausschließlich den Auto-Cron-Write steuert: die
/// Read-Aggregation liest ihn für Rebooking-Rows gar nicht. Genau das beweisen
/// die drei Toggle-Szenarien (Beweislast 1, Invariante I-8).
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

    let eh_vec: Vec<ExtraHours> = extra_hours;
    let mut extra_hours_service = MockExtraHoursService::new();
    // Der Mock MUSS seine Range-Parameter respektieren: der Billing-Period-Report
    // ruft `get_report_for_employee_range` viermal mit VERSCHIEDENEN Ranges
    // (ytd_from = Jahresstart..Perioden-Start-1, ytd_to, full_year, delta =
    // Perioden-Range). Gäbe der Mock stets alle Rows zurück, wäre die
    // Datums-Position der Rebooking-Woche innerhalb des Zeitraums bedeutungslos
    // und `value_ytd_from` würde sich mitbewegen, obwohl die Produktion (DAO-
    // seitige Datumsfilterung) das nicht tut.
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, from_date, to_date, _, _| {
            let from = from_date.to_date();
            let to = to_date.to_date();
            Ok(eh_vec
                .iter()
                .filter(|eh| {
                    let day = eh.date_time.date();
                    day >= from && day <= to
                })
                .cloned()
                .collect())
        });
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
    extra_hours: Vec<ExtraHours>,
    toggle_value: Option<&'static str>,
) -> BillingPeriodReportServiceImpl<BpDeps> {
    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    BillingPeriodReportServiceImpl {
        billing_period_service: Arc::new(service::billing_period::MockBillingPeriodService::new()),
        reporting_service: Arc::new(build_reporting_service(extra_hours, toggle_value)),
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
/// gegebenen Stichtag-Toggle-Wert und dem gegebenen ExtraHours-Datensatz.
///
/// Der Datensatz ist der einzige Freiheitsgrad für Beweislast 2 (with- vs.
/// without-Rebooking); der Toggle-Wert ist der einzige Freiheitsgrad für
/// Beweislast 1 (Toggle-Unabhängigkeit).
async fn compute_snapshot_for(
    extra_hours: Vec<ExtraHours>,
    toggle_value: Option<&'static str>,
) -> BillingPeriodSalesPerson {
    let (start, end) = period();
    let service = build_billing_period_service(extra_hours, toggle_value);
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

/// Dünner Wrapper über `compute_snapshot_for` mit dem Standard-Datensatz
/// (`straddling_extra_hours()`, also MIT Rebooking-Paar). Die drei
/// Toggle-Szenarien der Beweislast 1 nutzen ihn unverändert.
async fn compute_snapshot(toggle_value: Option<&'static str>) -> BillingPeriodSalesPerson {
    compute_snapshot_for(straddling_extra_hours(), toggle_value).await
}

/// Vergleicht zwei Snapshots, die sich nur um das Rebooking-Paar unterscheiden.
///
/// Für jeden `value_type` in `expected_shifts` muss die Verschiebung in **drei**
/// Feldern messbar sein — `value_delta`, `value_ytd_to` und `value_full_year`
/// (Toleranz 1e-3, WR-10) —, weil die Rebooking-Woche sowohl innerhalb des
/// Zeitraums als auch innerhalb des Jahres liegt. `value_ytd_from` muss
/// byte-identisch bleiben: es steht auf dem Perioden-Start und damit VOR der
/// Rebooking-Woche. Alle übrigen `value_type`s müssen in allen vier Feldern
/// byte-identisch sein.
#[track_caller]
fn assert_snapshot_shift(
    with_pair: &BillingPeriodSalesPerson,
    without_pair: &BillingPeriodSalesPerson,
    expected_shifts: &[(BillingPeriodValueType, f32)],
) {
    assert_eq!(
        with_pair.sales_person_id, without_pair.sales_person_id,
        "sales_person_id must match"
    );
    let with_keys: Vec<_> = with_pair.values.keys().cloned().collect();
    let without_keys: Vec<_> = without_pair.values.keys().cloned().collect();
    assert_eq!(
        with_keys, without_keys,
        "value_type key set must be identical between the with-pair and without-pair dataset — \
         wenn hier der Volunteer-Key fehlt, hat der Rebooking-Betrag die VolunteerWork-Baseline \
         exakt neutralisiert (Pitfall 5, billing_period_report.rs:283)"
    );

    for (vt, exp_val) in without_pair.values.iter() {
        let act_val = with_pair
            .values
            .get(vt)
            .unwrap_or_else(|| panic!("value_type {vt:?} missing in with-pair snapshot"));
        let name = format!("{vt:?}");

        match expected_shifts.iter().find(|(shift_vt, _)| shift_vt == vt) {
            Some((_, expected_shift)) => {
                // WR-10 (Code-Review Phase 57): dieselbe Verschiebung MUSS in
                // allen drei betroffenen Feldern messbar sein, nicht nur in
                // `value_delta`. Die Rebooking-Woche (KW 16/2026) liegt sowohl
                // innerhalb des Zeitraums als auch innerhalb des Jahres, also
                // gilt fuer `value_ytd_to` und `value_full_year` derselbe Shift.
                // Genau diese beiden Felder sind die, in denen sich der
                // Bump-Grund („eine frische Neuberechnung liefert andere Werte")
                // manifestiert — ein Regressions-Fix, der `value_full_year`
                // wieder rebooking-blind macht, bliebe sonst unentdeckt.
                for (field, act, exp) in [
                    ("value_delta", act_val.value_delta, exp_val.value_delta),
                    ("value_ytd_to", act_val.value_ytd_to, exp_val.value_ytd_to),
                    (
                        "value_full_year",
                        act_val.value_full_year,
                        exp_val.value_full_year,
                    ),
                ] {
                    let actual_shift = act - exp;
                    assert!(
                        (actual_shift - expected_shift).abs() < 1e-3,
                        "value_type `{name}` field `{field}`: erwartete Verschiebung durch das \
                         Rebooking-Paar {expected_shift}, gemessen {actual_shift} \
                         (mit Paar={act}, ohne Paar={exp}). Eine Verschiebung von 0 bedeutet, \
                         dass eine Kette das Rebooking-Paar wieder herausfiltert — der \
                         Filter-Fix aus 57-02 erreicht den Billing-Period-Pfad dann nicht mehr."
                    );
                }
                // `value_ytd_from` steht auf dem Perioden-Start und liegt damit
                // VOR der Rebooking-Woche — es darf sich nicht bewegen.
                assert_f32_bit_eq(
                    act_val.value_ytd_from,
                    exp_val.value_ytd_from,
                    &name,
                    "value_ytd_from",
                );
            }
            None => {
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
    }
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

// ─── Beweislast 2: das Rebooking-Paar ist im Snapshot sichtbar (D-57-11) ──────

/// Zwei Datensätze, identisch bis auf das Rebooking-Paar über `h = 2.0`,
/// liefern verschiedene Snapshots — und genau das ist die Begründung für
/// `CURRENT_SNAPSHOT_SCHEMA_VERSION = 13` (D-57-10).
///
/// Herleitung der drei Verschiebungen:
///
/// | value_type | Quelle | Wirkung des Paars |
/// |---|---|---|
/// | `Overall`   | `report_delta.overall_hours`    | `+2.0` — `shiftplan_hours_by_week + overall_extra_work_hours`, das `+h ExtraWork`-Bein hebt den zweiten Summanden |
/// | `Balance`   | `report_delta.balance_hours`    | `+2.0` — `overall_hours − expected_hours`, das Soll ist unberührt |
/// | `ExtraWork` | `report_delta.extra_work_hours` | `+2.0` — dasselbe Bein, direkt |
/// | `Volunteer` | `report_delta.volunteer_hours`  | `−2.0` — das `−h VolunteerWork`-Bein senkt das Ehrenamt-Ist |
///
/// `ExpectedHours` ist ausdrücklich NICHT betroffen: das Soll stammt aus dem
/// Vertrag, nicht aus `extra_hours`. Ebenso unberührt bleiben alle Absence- und
/// Urlaubs-value_types — `rebook()` schreibt ausschließlich die Kategorien
/// `VolunteerWork` und `ExtraWork`.
///
/// Bis Phase 56 war diese Verschiebung 0, weil `get_report_for_employee_range`
/// alle `source = Rebooking`-Rows pauschal verwarf (D-55-A1). Plan 57-02 hat
/// diesen Vorfilter entfernt (D-57-06). Damit gilt die CLAUDE.md-Klausel
/// „Change the input set the computation reads from": eine frische
/// Neuberechnung derselben Periode weicht von einem v12-Snapshot ab, und ohne
/// Bump wäre diese Drift von einem echten Datenfehler nicht unterscheidbar.
///
/// `value_ytd_from` bleibt in allen value_types byte-identisch: die
/// Rebooking-Woche (KW 16/2026, Mi 2026-04-15) liegt strikt innerhalb des
/// Zeitraums (Mo 2026-03-02 .. So 2026-05-17), also nach dem Perioden-Start.
///
/// Beide Snapshots werden mit demselben Toggle-Wert berechnet (`2026-06-01`,
/// die PRE-Referenz der Beweislast-1-Tests) — der Toggle ist hier bewusst
/// konstant, damit die gemessene Verschiebung ausschließlich vom Datensatz
/// stammt.
#[tokio::test]
async fn test_straddling_snapshot_rebooking_shifts_values() {
    let with_pair = compute_snapshot_for(straddling_extra_hours(), Some("2026-06-01")).await;
    let without_pair =
        compute_snapshot_for(straddling_extra_hours_without_rebooking(), Some("2026-06-01")).await;

    // Pitfall-5-Wächter: der Volunteer-Key wird nur bei value_delta != 0.0
    // geschrieben (billing_period_report.rs:283). Mit Paar: 3.0 − 2.0 = 1.0,
    // ohne Paar: 3.0 — beide ungleich 0, der Key existiert also beidseitig.
    // Fehlt er, ist die Fixture-Baseline verstellt worden und die
    // Schlüsselsatz-Assertion in assert_snapshot_shift würde eine irreführende
    // Meldung statt der Werte-Abweichung liefern.
    assert!(
        with_pair.values.contains_key(&BillingPeriodValueType::Volunteer),
        "Volunteer-Key fehlt im with-pair-Snapshot — die Fixture-Beträge (Baseline 3.0 \
         VolunteerWork, Rebooking −2.0) wurden verändert und neutralisieren sich jetzt exakt \
         (Pitfall 5)"
    );
    assert!(
        without_pair.values.contains_key(&BillingPeriodValueType::Volunteer),
        "Volunteer-Key fehlt im without-pair-Snapshot — die manuelle Baseline von 3.0 \
         VolunteerWork wurde entfernt oder auf 0 gesetzt (Pitfall 5)"
    );

    assert_snapshot_shift(
        &with_pair,
        &without_pair,
        &[
            (BillingPeriodValueType::Overall, 2.0),
            (BillingPeriodValueType::Balance, 2.0),
            (BillingPeriodValueType::ExtraWork, 2.0),
            (BillingPeriodValueType::Volunteer, -2.0),
        ],
    );
}
