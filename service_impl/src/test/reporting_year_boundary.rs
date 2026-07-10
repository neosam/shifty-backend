//! Phase 52 Follow-up #3 — Jahresübergang-**Regression-Gate** für
//! `ReportingService::get_year`.
//!
//! Ursprünglich (vor dem Fix) reproduzierten diese Tests den Bug: der Bulk-
//! Pfad `find_by_year(Y)` filterte kalendarisch, der Legacy-Pfad
//! `find_by_week(Y, w)` per ISO-Woche — und Rows an KW 1 (Y+1) / KW 53 (Y)
//! fielen aus dem Bulk-Bucket.
//!
//! Nach dem Fix (Trait-Method `find_by_iso_year` liefert die ISO-Wochenjahr-
//! Range statt kalendarisch) müssen Bulk und Legacy an denselben KW-Grenzen
//! bit-genau konvergieren. Diese Tests halten die Konvergenz fest — reißt sie
//! später wieder auseinander, gate an.
//!
//! Vergleich: `reporting_service.get_year(Y)[w-1]` (Bulk) vs.
//! `reporting_service.get_week(Y, w)` (Legacy) für kritische Wochen an
//! Grenzen. Absoluter Wert-Vergleich mit ε=1e-6.

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::rebooking_batch::MockRebookingBatchService;
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
    type RebookingBatchService = MockRebookingBatchService;
}

/// Baut ein ReportingService mit fixed contract (fixture_work_details_8h_mon_fri
/// — Contract KW22-25/2024). Wir passen den Contract-Bereich unten zur Laufzeit an,
/// damit wir Jahresübergangs-Fälle testen können.
//
// extras_by_dao_iso_year: was `find_by_iso_year(year)` liefert (ISO-Wochenjahr).
// extras_by_dao_week: was `find_by_week(year, week)` liefert (ISO-Woche).
fn build_service_with_extras(
    extras_by_dao_iso_year: std::collections::HashMap<u32, Vec<ExtraHours>>,
    extras_by_dao_week: std::collections::HashMap<(u32, u8), Vec<ExtraHours>>,
    work_details_from: (u32, u8),
    work_details_to: (u32, u8),
) -> ReportingServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));

    // Contract mit erweiterter Range, damit alle Jahresübergangs-Wochen "have contract row"
    // sind. WICHTIG: um `has_contract_row=true` in W1/W53 zu haben, muss
    // find_working_hours_for_calendar_week eine Row liefern → wir überschreiben
    // work_details' Ranges programmatisch:
    let mut wd = fixture_work_details_8h_mon_fri();
    wd.from_year = work_details_from.0;
    wd.from_calendar_week = work_details_from.1;
    wd.to_year = work_details_to.0;
    wd.to_calendar_week = work_details_to.1;
    let wd_arc: Arc<[service::employee_work_details::EmployeeWorkDetails]> = Arc::from(vec![wd]);

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    let wd_all = wd_arc.clone();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(wd_all.clone()));
    let wd_afw = wd_arc.clone();
    employee_work_details_service
        .expect_all_for_week()
        .returning(move |_, _, _, _| Ok(wd_afw.clone()));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::new())));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::new())));

    let mut extra_hours_service = MockExtraHoursService::new();
    let by_year = extras_by_dao_iso_year.clone();
    extra_hours_service
        .expect_find_by_iso_year()
        .returning(move |year, _, _| {
            Ok(Arc::from(by_year.get(&year).cloned().unwrap_or_default()))
        });
    let by_week = extras_by_dao_week.clone();
    extra_hours_service
        .expect_find_by_week()
        .returning(move |year, week, _, _| {
            Ok(Arc::from(
                by_week.get(&(year, week)).cloned().unwrap_or_default(),
            ))
        });

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
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
        rebooking_batch_service: Arc::new(MockRebookingBatchService::new()),
    }
}

fn extra_hours(
    date_time: time::PrimitiveDateTime,
    amount: f32,
    category: ExtraHoursCategory,
) -> ExtraHours {
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sales_person().id,
        amount,
        category,
        description: Arc::from(""),
        date_time,
        created: Some(datetime!(2020 - 01 - 01 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source: service::extra_hours::ExtraHoursSource::Manual,
    }
}

/// REGRESSION-GATE (post Follow-up #3):
///
/// ExtraHours mit Kalender-Datum 2019-12-30 (Mo) liegt in **ISO 2020-W1**
/// (2020-W1 = Mo 2019-12-30..So 2020-01-05).
///
/// Neuer DAO-Vertrag nach Follow-up #3:
/// - `find_by_week(2020, 1)` (Legacy, unverändert): liefert die Row (ISO-Range).
/// - `find_by_iso_year(2020)` (Bulk, neu): liefert die Row ebenfalls, weil die
///   Range jetzt `[ISO-Mo(2020, 1), ISO-Su(2020, 53) + 1d)` ist — also
///   `[2019-12-30, 2021-01-04)`.
///
/// → `get_week(2020, 1)` und `get_year(2020)[0]` müssen bit-identisch sein.
#[tokio::test]
async fn get_year_matches_get_week_for_extra_hours_at_iso_kw1_boundary() {
    // ISO 2020-W1 startet am Montag 2019-12-30.
    let cross_year_extra = extra_hours(
        datetime!(2019 - 12 - 30 12:00:00),
        4.5,
        ExtraHoursCategory::Vacation,
    );

    // Legacy find_by_week(2020, 1) → Row drin.
    let mut by_week = std::collections::HashMap::new();
    by_week.insert((2020u32, 1u8), vec![cross_year_extra.clone()]);

    // Bulk find_by_iso_year(2020) → Row drin (neuer ISO-Range-Filter, 2019-12-30
    // liegt in ISO-2020-W1). Follow-up #3 fixt genau das.
    let mut by_year = std::collections::HashMap::new();
    by_year.insert(2020u32, vec![cross_year_extra.clone()]);
    // find_by_iso_year(2019) sollte die Row NICHT enthalten (2019 ISO-Range endet
    // vor 2019-12-30 — 2019-W1 startet Mo 2018-12-31, 2019-W52 endet So 2019-12-29,
    // 2019 hat 52 Wochen). Aber wir stellen sicher, dass unser Mock nur den
    // relevanten Bucket testet — get_year(2020) fragt find_by_iso_year(2020).
    by_year.insert(2019u32, Vec::<ExtraHours>::new());

    let service = build_service_with_extras(by_year, by_week, (2019, 1), (2021, 52));

    let week_report = service
        .get_week(2020, 1, Authentication::Full, None)
        .await
        .expect("get_week(2020, 1) must succeed");
    let week_vacation = week_report
        .iter()
        .map(|r| r.vacation_hours)
        .sum::<f32>();

    let year_report = service
        .get_year(2020, Authentication::Full, None)
        .await
        .expect("get_year(2020) must succeed");
    let (_kw, bulk_kw1_reports) = year_report
        .iter()
        .find(|(w, _)| *w == 1)
        .expect("KW1 must be in get_year result");
    let bulk_vacation = bulk_kw1_reports
        .iter()
        .map(|r| r.vacation_hours)
        .sum::<f32>();

    eprintln!(
        "KW1/2020 (regression-gate) — legacy get_week vacation_hours={} vs. bulk get_year[0].1 vacation_hours={}",
        week_vacation, bulk_vacation
    );
    // Beide Pfade müssen die 4.5h Vacation sehen (Follow-up #3 Fix-Assert).
    assert!(
        (week_vacation - 4.5).abs() < 1e-6,
        "Legacy sollte 4.5h vacation liefern, war: {}", week_vacation
    );
    assert!(
        (bulk_vacation - 4.5).abs() < 1e-6,
        "Bulk (nach Fix) sollte 4.5h vacation liefern, war: {}", bulk_vacation
    );
    assert!(
        (week_vacation - bulk_vacation).abs() < 1e-6,
        "Regression: bulk und legacy divergieren an KW1/Jahresgrenze — Follow-up #3 Fix gebrochen. \
         legacy={}, bulk={}",
        week_vacation, bulk_vacation
    );
}

/// REGRESSION-GATE (post Follow-up #3): KW 53 Fall.
///
/// ExtraHours am 2021-01-02 (Sa) liegt in **ISO 2020-W53**
/// (2020-W53 = Mo 2020-12-28..So 2021-01-03).
///
/// Neuer DAO-Vertrag:
/// - `find_by_week(2020, 53)` (Legacy): Row drin (ISO-Range).
/// - `find_by_iso_year(2020)` (Bulk, neu): Row drin, weil ISO-Range
///   `[2019-12-30, 2021-01-04)` das Datum 2021-01-02 enthält.
///
/// → Konvergenz. Bit-genauer Vergleich.
#[tokio::test]
async fn get_year_matches_get_week_for_extra_hours_at_iso_kw53_boundary() {
    let cross_year_extra = extra_hours(
        datetime!(2021 - 01 - 02 12:00:00),
        3.0,
        ExtraHoursCategory::Vacation,
    );

    let mut by_week = std::collections::HashMap::new();
    by_week.insert((2020u32, 53u8), vec![cross_year_extra.clone()]);

    // Bulk find_by_iso_year(2020) → Row drin (neuer ISO-Range).
    let mut by_year = std::collections::HashMap::new();
    by_year.insert(2020u32, vec![cross_year_extra.clone()]);
    // find_by_iso_year(2021) enthält die Row NICHT (ISO-2021-W1 startet Mo 2021-01-04).
    by_year.insert(2021u32, Vec::<ExtraHours>::new());

    let service = build_service_with_extras(by_year, by_week, (2019, 1), (2022, 1));

    let week_report = service
        .get_week(2020, 53, Authentication::Full, None)
        .await
        .expect("get_week(2020, 53) must succeed");
    let week_vac = week_report
        .iter()
        .map(|r| r.vacation_hours)
        .sum::<f32>();

    let year_report = service
        .get_year(2020, Authentication::Full, None)
        .await
        .expect("get_year(2020) must succeed");
    let (_kw, bulk_kw53_reports) = year_report
        .iter()
        .find(|(w, _)| *w == 53)
        .expect("KW53 must be in get_year(2020) result");
    let bulk_vac = bulk_kw53_reports
        .iter()
        .map(|r| r.vacation_hours)
        .sum::<f32>();

    eprintln!(
        "KW53/2020 (regression-gate) — legacy get_week vacation_hours={} vs. bulk get_year[52].1 vacation_hours={}",
        week_vac, bulk_vac
    );
    assert!(
        (week_vac - 3.0).abs() < 1e-6,
        "Legacy sollte 3h vacation liefern, war: {}", week_vac
    );
    assert!(
        (bulk_vac - 3.0).abs() < 1e-6,
        "Bulk (nach Fix) sollte 3h vacation liefern, war: {}", bulk_vac
    );
    assert!(
        (week_vac - bulk_vac).abs() < 1e-6,
        "Regression: bulk und legacy divergieren an KW53/Jahresgrenze — Follow-up #3 Fix gebrochen. \
         legacy={}, bulk={}",
        week_vac, bulk_vac
    );
}
