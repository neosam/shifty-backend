//! Phase 51 Chain C — `booking_information` ShortDay-Clip Regression Suite.
//!
//! Diese Datei pinnt das Fix aus D-51-06 Chain C für zwei Sites in
//! `booking_information.rs`:
//!
//! - `get_weekly_summary` (Year-View, Achse-B) — Slots werden pro Wochentag
//!   an einem ShortDay-Cutoff geclippt statt komplett verworfen (D-04 Zeile 4).
//! - `get_summery_for_week` (Single-Week) + implizit `required_hours_by_day`
//!   — beide fußen auf derselben `slots`-Variable und fallen mit dem Clip
//!   automatisch korrekt.
//!
//! Ausserdem D-51-07 Stichtag-Gate: `active_from = None` → Legacy-Verhalten
//! (unclipped 1h statt 0,5h). Und die Boundary `booking_date == active_from`
//! (SHC-06 Grenzfall).
//!
//! Für D-51-03 (Booking-Create bleibt unangefasst — kein 409, kein neuer
//! Error-Variant) siehe `test/booking.rs::phase51_create_post_cutoff_slot_not_rejected`.

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::{AbsencePeriod, MockAbsenceService};
use service::booking::MockBookingService;
use service::booking_information::BookingInformationService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::Authentication;
use service::reporting::MockReportingService;
use service::sales_person::MockSalesPersonService;
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::shiftplan_report::MockShiftplanReportService;
use service::slot::{MockSlotService, Slot};
use service::special_days::{MockSpecialDayService, SpecialDay, SpecialDayType};
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::DayOfWeek;

use crate::booking_information::{BookingInformationServiceDeps, BookingInformationServiceImpl};

// ─── TestDeps ─────────────────────────────────────────────────────────────────

struct TestDeps;

impl BookingInformationServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ShiftplanReportService = MockShiftplanReportService;
    type SlotService = MockSlotService;
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

// ─── Fixture helpers ──────────────────────────────────────────────────────────

const YEAR: u32 = 2026;
/// 2026-W31 (Mon = 2026-07-27, ISO). Gate-aktiv, wenn active_from ≤ 2026-07-27.
const WEEK: u8 = 31;

/// Slot am Wochentag `dow` von `from` bis `to` mit `min_resources=1`.
fn slot(dow: DayOfWeek, from: time::Time, to: time::Time) -> Slot {
    Slot {
        id: Uuid::new_v4(),
        day_of_week: dow,
        from,
        to,
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::new_v4(),
        shiftplan_id: None,
    }
}

/// ShortDay-Cutoff am Wochentag `dow` um `cutoff_at` (nur für passende Woche geliefert).
fn shortday(dow: DayOfWeek, cutoff_at: time::Time) -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year: YEAR,
        calendar_week: WEEK,
        day_of_week: dow,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(cutoff_at),
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Baut den Service mit einer parametrischen Slot-Liste, SpecialDay-Liste und Toggle-Wert.
fn build_service(
    slots: Vec<Slot>,
    special_days_for_week: Vec<SpecialDay>,
    toggle_active_from: Option<&'static str>,
) -> BookingInformationServiceImpl<TestDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    // Kein Volunteer, kein Paid → volunteer_ids leer, paid_employees leer.
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::sales_person::SalesPerson>::new())));

    // Keine Contracts.
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| Ok(Arc::from(Vec::<
            service::employee_work_details::EmployeeWorkDetails,
        >::new())));

    // Keine Absences.
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

    // SpecialDays: für passende (year, week) → gefixte Liste; für andere Wochen (year-view
    // durchläuft weeks_in_year + 3) → leer.
    let sd_clone = special_days_for_week.clone();
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == WEEK {
                Ok(Arc::from(sd_clone.clone()))
            } else {
                Ok(Arc::from(Vec::<SpecialDay>::new()))
            }
        });

    // Reporting-Service: leerer Report für alle Wochen (isoliert slot_hours).
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<
                service::reporting::ShortEmployeeReport,
            >::new()))
        });

    // ShiftplanReport: leer.
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<
                service::shiftplan_report::ShiftplanReportDay,
            >::new()))
        });

    // Slots: nur für die Ziel-Woche liefern; andere Wochen leer.
    let slots_clone = slots.clone();
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |year, week, _, _| {
            if year == YEAR && week == WEEK {
                Ok(Arc::from(slots_clone.clone()))
            } else {
                Ok(Arc::from(Vec::<Slot>::new()))
            }
        });

    // Toggle: die parametrische Antwort.
    let toggle_val: Option<String> = toggle_active_from.map(String::from);
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(move |_, _, _| Ok(toggle_val.clone().map(Arc::from)));

    // sales_person_unavailable: leer (nur für get_summery_for_week gebraucht).
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

    BookingInformationServiceImpl::<TestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
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

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

// ─── Test A: get_weekly_summary clippt slot_hours am ShortDay ───────────────

/// D-51-06 Chain C, SHC-02, D-04 Zeile 4:
/// Slot Mo 14:00–15:00, min_resources=1, ShortDay Mo 14:30, Gate aktiv
/// → `required_hours` in W31-WeeklySummary enthält 0,5h für diesen Slot (statt 1h).
#[tokio::test]
async fn test_get_weekly_summary_clips_slot_hours_at_shortday() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());
    let service = build_service(vec![s], vec![sd], Some("2026-07-01"));

    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");

    let w = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK)
        .expect("W31 must be present in year-view");

    assert!(
        approx(w.required_hours, 0.5),
        "Chain C / D-04 Zeile 4: geclipptes 14:00–14:30 = 0,5h, got {}",
        w.required_hours
    );
}

// ─── Test B: get_weekly_summary ohne Gate → kein Clip ──────────────────────

/// D-51-07: `active_from = None` → Slot bleibt ungeclippt (Legacy 1h statt 0,5h).
#[tokio::test]
async fn test_get_weekly_summary_ungated_no_clip() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    // ShortDay ist zwar konfiguriert, aber ohne Toggle greift das Gate nicht.
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());
    let service = build_service(vec![s], vec![sd], None);

    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");

    let w = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK)
        .expect("W31 must be present in year-view");

    assert!(
        approx(w.required_hours, 1.0),
        "D-51-07 Gate off: Slot 14:00–15:00 muss volle 1,0h liefern, got {}",
        w.required_hours
    );
}

// ─── Test C: Post-Cutoff-Slot komplett weg ─────────────────────────────────

/// D-04 Zeile 3 / SHC-05:
/// Slot Mo 15:00–16:00, ShortDay Cutoff 14:30, Gate aktiv → Slot fällt komplett
/// weg (`from >= cutoff`), `required_hours` = 0.
#[tokio::test]
async fn test_get_weekly_summary_drops_post_cutoff_slot() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(15, 0, 0).unwrap(),
        time::Time::from_hms(16, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());
    let service = build_service(vec![s], vec![sd], Some("2026-07-01"));

    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");

    let w = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK)
        .expect("W31 must be present in year-view");

    assert!(
        approx(w.required_hours, 0.0),
        "D-04 Zeile 3: Post-Cutoff-Slot muss ganz wegfallen, required_hours = 0, got {}",
        w.required_hours
    );
}

// ─── Test D: get_summery_for_week + required_hours_by_day ───────────────────

/// Chain C site 2 + `required_hours_by_day`-Fold:
/// - Mo Slot 14:00–15:00 mit ShortDay 14:30 → Mo required = 0,5h.
/// - Di Slot 10:00–12:00 ohne ShortDay am Di → Di required = 2,0h (ungeclippt).
///
/// Beweist zusätzlich, dass `required_hours_by_day` (fold über die geclippte
/// `slots`-Variable, keine eigene Filter-Logik) automatisch mitclippt (P-INDEX
/// Warning-Reuse-Choice: kein zweiter Fix nötig).
#[tokio::test]
async fn test_get_summery_for_week_required_hours_by_day_respects_clip() {
    let monday_slot = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let tuesday_slot = slot(
        DayOfWeek::Tuesday,
        time::Time::from_hms(10, 0, 0).unwrap(),
        time::Time::from_hms(12, 0, 0).unwrap(),
    );
    // ShortDay nur am Montag.
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());
    let service = build_service(
        vec![monday_slot, tuesday_slot],
        vec![sd],
        Some("2026-07-01"),
    );

    let summary = service
        .get_summery_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("get_summery_for_week must succeed");

    // required_hours_by_day → available = 0 - required = -required
    // (paid_employees leer, volunteer_hours = 0 → Monday_hours = 0.0 -> available = -required).
    assert!(
        approx(summary.monday_available_hours, -0.5),
        "Mo: 0,5h required (geclippt), 0 available (no employees) → -0,5h, got {}",
        summary.monday_available_hours
    );
    assert!(
        approx(summary.tuesday_available_hours, -2.0),
        "Di: 2,0h required (kein ShortDay am Di), 0 available → -2,0h, got {}",
        summary.tuesday_available_hours
    );
    // Gesamt-required = 0.5 + 2.0
    assert!(
        approx(summary.required_hours, 2.5),
        "Gesamt required = 0,5 (Mo) + 2,0 (Di) = 2,5h, got {}",
        summary.required_hours
    );
}

// ─── Test E: Stichtag-Boundary ─────────────────────────────────────────────

/// D-51-07 / SHC-06 Grenzfall: `active_from` inklusiv am Tag.
/// - `booking_date == active_from - 1 day` → ungeclippt (1,0h).
/// - `booking_date == active_from` → geclippt (0,5h).
///
/// Konkret: 2026-W31-Mo = 2026-07-27. Wir schalten das Gate mit `active_from`
/// einmal auf 2026-07-28 (=> W31-Mo liegt einen Tag davor → ungeclippt) und
/// einmal auf 2026-07-27 (=> W31-Mo == active_from → geclippt).
#[tokio::test]
async fn test_get_summery_for_week_stichtag_boundary() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());

    // Fall 1: active_from = 2026-07-28 (Di), W31-Mo (2026-07-27) < active_from → ungeclippt.
    let service_before = build_service(vec![s.clone()], vec![sd.clone()], Some("2026-07-28"));
    let summary_before = service_before
        .get_summery_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("must succeed");
    assert!(
        approx(summary_before.required_hours, 1.0),
        "SHC-06 boundary (booking_date < active_from): ungeclippt 1,0h, got {}",
        summary_before.required_hours
    );

    // Fall 2: active_from = 2026-07-27 (=W31-Mo), inklusiv → geclippt.
    let service_at = build_service(vec![s], vec![sd], Some("2026-07-27"));
    let summary_at = service_at
        .get_summery_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("must succeed");
    assert!(
        approx(summary_at.required_hours, 0.5),
        "SHC-06 boundary (booking_date == active_from, inklusiv): geclippt 0,5h, got {}",
        summary_at.required_hours
    );
}
