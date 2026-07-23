//! Phase 52 — Jahresübergang-Regression-Test (KW 53 / KW 1).
//!
//! Zweck: Reproduziert den vom User gemeldeten Bug — an Jahresgrenzen (KW 53 des
//! aktuellen Jahres und KW 1 des Folgejahres) unterscheiden sich `paid_hours`
//! und/oder `required_hours` zwischen der ALTEN Legacy-`get_summery_for_week`-
//! Semantik (Per-Week-DAO-Call) und der NEUEN Bulk-Load-Semantik in
//! `get_weekly_summary` (Phase 52).
//!
//! Strategie:
//! - Realistischer Slot: `valid_from = 2019-01-01`, `valid_to = None`
//!   → in ALLEN Wochen aktiv (Produktions-Muster).
//! - Legacy-Mock `get_slots_for_week_all_plans` liefert denselben Slot für jede
//!   angefragte (year, week).
//! - Bulk-Mock `get_slots` liefert genau eine Slot-Kopie (nicht künstlich pro
//!   Woche gestempelt wie in den existierenden Fixtures).
//! - Wir rufen `get_weekly_summary(YEAR)` auf UND parallel
//!   `get_summery_for_week(YEAR, w)` bzw. `.., YEAR+1, 1..3)` für jede Woche und
//!   vergleichen `required_hours` und `paid_hours`.
//!
//! Getestete Jahre:
//! - 2020 (KW-53-Jahr) — spannend, weil w53 existiert.
//! - 2021 (52-Wochen-Jahr, folgt auf 2020) — spannend für den KW-1-Fall.

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::booking::MockBookingService;
use service::booking_information::BookingInformationService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::Authentication;
use service::reporting::{MockReportingService, ShortEmployeeReport};
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::shiftplan_report::{MockShiftplanReportService, ShiftplanReportDay};
use service::slot::{MockSlotService, Slot};
use service::special_days::{MockSpecialDayService, SpecialDay};
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::DayOfWeek;

use crate::booking_information::{BookingInformationServiceDeps, BookingInformationServiceImpl};

// ─── TestDeps (spiegelt Struktur aus year_batch-Test) ────────────────────────

struct TestDeps;

impl BookingInformationServiceDeps for TestDeps {
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

// ─── Fixture-Builder ──────────────────────────────────────────────────────────

const SP_PAID: Uuid = Uuid::from_bytes([0x11; 16]);

fn paid_person() -> SalesPerson {
    SalesPerson {
        id: SP_PAID,
        name: Arc::from("Paid Person"),
        background_color: Arc::from("#ffffff"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Der "immer aktive" Produktions-Slot: valid_from weit in der Vergangenheit,
/// valid_to = None (unbegrenzt). Montag 10:00-18:00, min_resources=1
/// → 8h required_hours pro Woche.
fn always_active_slot() -> Slot {
    Slot {
        id: Uuid::from_bytes([0xAA; 16]),
        day_of_week: DayOfWeek::Monday,
        from: time::Time::from_hms(10, 0, 0).unwrap(),
        to: time::Time::from_hms(18, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2019, time::Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::nil(),
        shiftplan_id: None,
    }
}

/// Baut die Service-Instanz mit Legacy- UND Bulk-Mocks die konsistent denselben
/// "immer aktiven" Slot ausliefern. Die Legacy-Mock `get_slots_for_week_all_plans`
/// gibt den Slot pro (year, week) zurück — spiegelt was der echte DAO tun würde,
/// da `valid_from <= sunday_of_week AND (valid_to IS NULL OR valid_to >= monday_of_week)`
/// für JEDE Woche true ist.
fn build_service(paid_report_by_week: Vec<((u32, u8), f32)>) -> BookingInformationServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let sp_arc: Arc<[SalesPerson]> = Arc::from(vec![paid_person()]);
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(sp_arc.clone()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

    // SpecialDays: leer für beide Jahre.
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<SpecialDay>::new())));
    special_day_service
        .expect_get_by_iso_year()
        .returning(|_, _| Ok(Arc::from(Vec::<SpecialDay>::new())));

    // week_reports pro (year, week) — `paid_hours` in dieser Woche = report.dynamic_hours.
    let paid_report_map: std::collections::HashMap<(u32, u8), f32> =
        paid_report_by_week.into_iter().collect();

    // Reporting-Service: sowohl get_week (Legacy) als auch get_year (Bulk) müssen
    // aus DERSELBEN Datenquelle liefern, damit der Vergleich fair ist.
    let map_for_week = paid_report_map.clone();
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(move |year, week, _, _| {
            let hours = *map_for_week.get(&(year, week)).unwrap_or(&0.0);
            if hours == 0.0 {
                return Ok(Arc::from(Vec::<ShortEmployeeReport>::new()));
            }
            Ok(Arc::from(vec![ShortEmployeeReport {
                sales_person: Arc::new(paid_person()),
                balance_hours: 0.0,
                dynamic_hours: hours,
                expected_hours: 0.0,
                overall_hours: 0.0,
                vacation_hours: 0.0,
                sick_leave_hours: 0.0,
                holiday_hours: 0.0,
                unavailable_hours: 0.0,
                unpaid_leave_hours: 0.0,
                volunteer_hours: 0.0,
                custom_absence_hours: Arc::from(Vec::new()),
                has_pending_rebooking: false,
                pending_rebooking_id: None,
            }]))
        });
    let map_for_year = paid_report_map.clone();
    reporting_service
        .expect_get_year()
        .returning(move |year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| {
                    let hours = *map_for_year.get(&(year, w)).unwrap_or(&0.0);
                    let rows: Vec<ShortEmployeeReport> = if hours == 0.0 {
                        Vec::new()
                    } else {
                        vec![ShortEmployeeReport {
                            sales_person: Arc::new(paid_person()),
                            balance_hours: 0.0,
                            dynamic_hours: hours,
                            expected_hours: 0.0,
                            overall_hours: 0.0,
                            vacation_hours: 0.0,
                            sick_leave_hours: 0.0,
                            holiday_hours: 0.0,
                            unavailable_hours: 0.0,
                            unpaid_leave_hours: 0.0,
                            volunteer_hours: 0.0,
                            custom_absence_hours: Arc::from(Vec::new()),
                            has_pending_rebooking: false,
                            pending_rebooking_id: None,
                        }]
                    };
                    (w, Arc::from(rows))
                })
                .collect();
            Ok(Arc::from(out))
        });

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ShiftplanReportDay>::new())));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::<ShiftplanReportDay>::new())));

    // Slot-Service:
    //   - Legacy `get_slots_for_week_all_plans(year, week)`: der Slot ist in
    //     ALLEN Wochen aktiv (valid_from 2019, valid_to None) → gib ihn zurück.
    //   - Bulk `get_slots()`: gibt denselben Slot ein einziges Mal zurück.
    let slot_bulk = always_active_slot();
    let slot_legacy = slot_bulk.clone();
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |_, _, _, _| Ok(Arc::from(vec![slot_legacy.clone()])));
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(vec![slot_bulk.clone()])));

    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
    sales_person_unavailable_service
        .expect_get_by_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<
                service::sales_person_unavailable::SalesPersonUnavailable,
            >::new()))
        });

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    // clock/uuid nicht relevant für diesen Test
    let _ = datetime!(2020 - 01 - 01 08:00:00);

    BookingInformationServiceImpl::<TestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
        shiftplan_service: Arc::new(shiftplan_service_mock),
        booking_service: Arc::new(MockBookingService::new()),
        sales_person_service: Arc::new(sales_person_service),
        sales_person_unavailable_service: Arc::new(sales_person_unavailable_service),
        reporting_service: Arc::new(reporting_service),
        special_day_service: Arc::new(special_day_service),
        toggle_service: Arc::new(toggle_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        absence_service: Arc::new(absence_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        transaction_dao: Arc::new(transaction_dao),
    }
}

// ─── Test 1: KW 53 / KW 1 required_hours mit "immer aktivem" Slot ─────────────

/// Kernbaustein-Reproduktion: Ein Slot valid_from=2019 valid_to=None ist in
/// ALLEN Wochen aktiv. `required_hours` MUSS in jeder Woche 8.0 sein (Mo 10-18).
///
/// Erwartung: Der neue Bulk-Load im get_weekly_summary sollte den Slot ebenso
/// in ALLEN 56 Wochen sehen. Wenn er das nicht tut, ist die Regression bewiesen.
#[tokio::test]
async fn required_hours_always_8_across_year_boundary_2020_2021() {
    let service = build_service(vec![]);
    let year: u32 = 2020;
    let summary = service
        .get_weekly_summary(year, Authentication::Full, None)
        .await
        .expect("get_weekly_summary should succeed");

    let weeks_in_year = time::util::weeks_in_year(year as i32);
    assert_eq!(
        summary.len() as u8,
        weeks_in_year + 3,
        "expected {} weekly summaries (weeks_in_year + 3 spillover)",
        weeks_in_year + 3
    );

    for (i, s) in summary.iter().enumerate() {
        // Bug-Reproduktion: In dieser Konfiguration MUSS jede Woche required=8.0 haben.
        assert!(
            (s.required_hours - 8.0).abs() < 1e-6,
            "week idx={} year={} week={} required_hours={} (expected 8.0)",
            i,
            s.year,
            s.week,
            s.required_hours
        );
    }
}

/// Nummer 2: gleicher Test für 2026 (auch KW-53-Jahr). Deckt einen kürzeren Zeitraum
/// hinsichtlich User-Report (test-env 2026?) ab.
#[tokio::test]
async fn required_hours_always_8_across_year_boundary_2026_2027() {
    let service = build_service(vec![]);
    let year: u32 = 2026;
    let summary = service
        .get_weekly_summary(year, Authentication::Full, None)
        .await
        .expect("get_weekly_summary should succeed");

    let weeks_in_year = time::util::weeks_in_year(year as i32);
    for (i, s) in summary.iter().enumerate() {
        assert!(
            (s.required_hours - 8.0).abs() < 1e-6,
            "week idx={} year={} week={} required_hours={} (expected 8.0). weeks_in_year={}",
            i,
            s.year,
            s.week,
            s.required_hours,
            weeks_in_year,
        );
    }
}

// ─── Test 3: Bounded valid_to slot (Kalender-Jahr-Ende) ──────────────────────

/// Slot mit `valid_to = 2020-12-31` (Kalender-Jahr-Ende): der Slot ist in
/// ISO-KW 53 / 2020 aktiv (Mo=2020-12-28, So=2021-01-03) weil valid_to
/// (2020-12-31) >= monday (2020-12-28). Sowohl Legacy als auch Bulk sollten
/// den Slot in W53 zählen, aber in W54(=2021-W1) NICHT mehr.
#[tokio::test]
async fn slot_with_calendar_year_end_valid_to_bulk_matches_legacy() {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let sp_arc: Arc<[SalesPerson]> = Arc::from(vec![paid_person()]);
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(sp_arc.clone()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::new())));

    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<SpecialDay>::new())));
    special_day_service
        .expect_get_by_iso_year()
        .returning(|_, _| Ok(Arc::from(Vec::<SpecialDay>::new())));

    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ShortEmployeeReport>::new())));
    reporting_service
        .expect_get_year()
        .returning(|year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| (w, Arc::from(Vec::new())))
                .collect();
            Ok(Arc::from(out))
        });

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ShiftplanReportDay>::new())));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| Ok(Arc::from(Vec::<ShiftplanReportDay>::new())));

    let bounded_slot = Slot {
        id: Uuid::from_bytes([0xBB; 16]),
        day_of_week: DayOfWeek::Monday,
        from: time::Time::from_hms(10, 0, 0).unwrap(),
        to: time::Time::from_hms(18, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2020, time::Month::January, 1).unwrap(),
        valid_to: Some(time::Date::from_calendar_date(2020, time::Month::December, 31).unwrap()),
        deleted: None,
        version: Uuid::nil(),
        shiftplan_id: None,
    };
    let slot_bulk = bounded_slot.clone();
    let slot_legacy = bounded_slot.clone();

    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |year, week, _, _| {
            let monday =
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday).unwrap();
            let sunday =
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday).unwrap();
            let s = &slot_legacy;
            let is_active =
                s.valid_from <= sunday && s.valid_to.map(|vt| vt >= monday).unwrap_or(true);
            if is_active {
                Ok(Arc::from(vec![slot_legacy.clone()]))
            } else {
                Ok(Arc::from(Vec::<Slot>::new()))
            }
        });
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(vec![slot_bulk.clone()])));

    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
    sales_person_unavailable_service
        .expect_get_by_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<
                service::sales_person_unavailable::SalesPersonUnavailable,
            >::new()))
        });

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let service = BookingInformationServiceImpl::<TestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
        shiftplan_service: Arc::new(shiftplan_service_mock),
        booking_service: Arc::new(MockBookingService::new()),
        sales_person_service: Arc::new(sales_person_service),
        sales_person_unavailable_service: Arc::new(sales_person_unavailable_service),
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

    let year: u32 = 2020;
    let bulk = service
        .get_weekly_summary(year, Authentication::Full, None)
        .await
        .expect("bulk");

    let mut mismatches: Vec<String> = Vec::new();
    for s in bulk.iter() {
        let legacy = service
            .get_summery_for_week(s.year, s.week, Authentication::Full, None)
            .await
            .expect("legacy");
        if (s.required_hours - legacy.required_hours).abs() > 1e-6 {
            mismatches.push(format!(
                "required_hours diverges at year={} week={}: bulk={} legacy={}",
                s.year, s.week, s.required_hours, legacy.required_hours
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "bulk vs legacy diverges for slot with valid_to=2020-12-31:\n  - {}",
        mismatches.join("\n  - ")
    );
}

// ─── Test 2: Legacy-vs-Bulk-Cross-Check via get_summery_for_week ──────────────

/// Vergleicht direkt `get_weekly_summary(Y)` (Bulk) gegen `get_summery_for_week(Y, w)`
/// (Legacy Per-Week-DAO) für jede Woche im spillover 2020→2021 und meldet Diskrepanzen
/// pro (year, week).
#[tokio::test]
async fn compare_bulk_vs_legacy_per_week_2020_spillover() {
    let paid_reports = vec![
        ((2020, 52), 3.0),
        ((2020, 53), 5.0),
        ((2021, 1), 7.0),
        ((2021, 2), 9.0),
    ];
    let service = build_service(paid_reports.clone());
    let year: u32 = 2020;
    let bulk = service
        .get_weekly_summary(year, Authentication::Full, None)
        .await
        .expect("bulk get_weekly_summary");

    let weeks_in_year = time::util::weeks_in_year(year as i32);
    // Für jede Woche im Bulk-Ergebnis: hole Legacy-Wert per get_summery_for_week
    // und vergleiche required_hours + paid_hours.
    let mut mismatches: Vec<String> = Vec::new();
    for s in bulk.iter() {
        let legacy = service
            .get_summery_for_week(s.year, s.week, Authentication::Full, None)
            .await
            .expect("legacy get_summery_for_week");
        if (s.required_hours - legacy.required_hours).abs() > 1e-6 {
            mismatches.push(format!(
                "required_hours diverges at year={} week={}: bulk={} legacy={}",
                s.year, s.week, s.required_hours, legacy.required_hours
            ));
        }
        if (s.paid_hours - legacy.paid_hours).abs() > 1e-6 {
            mismatches.push(format!(
                "paid_hours diverges at year={} week={}: bulk={} legacy={}",
                s.year, s.week, s.paid_hours, legacy.paid_hours
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "bulk vs legacy diverges (weeks_in_year({})={}):\n  - {}",
        year,
        weeks_in_year,
        mismatches.join("\n  - ")
    );
}
