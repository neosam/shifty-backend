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
use service::ServiceError;
use shifty_utils::DayOfWeek;

use crate::booking_information::{BookingInformationServiceDeps, BookingInformationServiceImpl};

// ─── TestDeps ─────────────────────────────────────────────────────────────────

struct TestDeps;

impl BookingInformationServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ShiftplanReportService = MockShiftplanReportService;
    type SlotService = MockSlotService;
    // Phase 52 (WOP-01, D-52-01): ShiftplanService (catalog) für `is_planning`
    // -Filter im Slot-Bulk-Load in `get_weekly_summary`.
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
    let sd_year_this: Vec<SpecialDay> = sd_clone
        .iter()
        .filter(|d| d.year == YEAR)
        .cloned()
        .collect();
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
    // Phase 52 (WOP-01): Bulk-Load je Jahr — der In-Memory-Filter im
    // Consumer wählt die (year, week)-Rows.
    special_day_service
        .expect_get_by_iso_year()
        .returning(move |year, _| {
            if year == YEAR {
                Ok(Arc::from(sd_year_this.clone()))
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
    // Phase 52 (WOP-02): Bulk-Variante liefert einen Vec von (week, empty)
    // — Wave-4-Delegation garantiert byte-identisches Verhalten zu 55×get_week.
    reporting_service
        .expect_get_year()
        .returning(|year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[service::reporting::ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| (w, Arc::from(Vec::new())))
                .collect();
            Ok(Arc::from(out))
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
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| {
            Ok(Arc::from(Vec::<
                service::shiftplan_report::ShiftplanReportDay,
            >::new()))
        });

    // Slots: nur für die Ziel-Woche liefern; andere Wochen leer.
    // Phase 52 (WOP-01): Bulk `get_slots()` liefert alle Slots mit
    // korrigiertem `valid_from`/`valid_to` gegen die Zielwoche, damit der
    // In-Memory-Filter im Consumer (year, week) korrekt selektiert.
    let slots_clone = slots.clone();
    let target_monday = time::Date::from_iso_week_date(YEAR as i32, WEEK, time::Weekday::Monday)
        .expect("YEAR/WEEK must map to a valid ISO week Monday");
    let target_sunday = time::Date::from_iso_week_date(YEAR as i32, WEEK, time::Weekday::Sunday)
        .expect("YEAR/WEEK must map to a valid ISO week Sunday");
    let bulk_slots: Vec<Slot> = slots_clone
        .iter()
        .map(|s| Slot {
            valid_from: target_monday,
            valid_to: Some(target_sunday),
            ..s.clone()
        })
        .collect();
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
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(bulk_slots.clone())));

    // Phase 52 (WOP-01, D-52-01): ShiftplanService.get_all — leer (kein
    // planning-Shiftplan → alle Slots ohne shiftplan_id oder mit
    // shiftplan_id passieren den `is_planning`-Filter).
    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

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

// ─── Test B: get_weekly_summary ohne Gate → Legacy-Filter (Gap-Closure) ─────

/// D-51-07 + Gap-Closure (Chain C Legacy-Filter):
/// `active_from = None` + ShortDay + `slot.to > cutoff` → Slot fällt weg
/// (Pre-Phase-51-Verhalten, `booking_information.rs:394-401` vor 62a2f35).
/// Slot 14:00–15:00, Cutoff 14:30 → 15:00 > 14:30 → `required_hours = 0`.
#[tokio::test]
async fn test_get_weekly_summary_ungated_legacy_drops_overlap() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
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
        approx(w.required_hours, 0.0),
        "Gap-Closure Chain C: Legacy-Drop (slot.to > cutoff) → required_hours = 0, got {}",
        w.required_hours
    );
}

/// D-51-07 + Gap-Closure Companion (Chain C Legacy-Keep):
/// `active_from = None` + ShortDay + `slot.to <= cutoff` → Slot bleibt roh.
/// Beweis dass der Legacy-Filter nur droppen kann, wenn Slot echt hinter
/// Cutoff hinausragt.
#[tokio::test]
async fn test_get_weekly_summary_ungated_legacy_keeps_pre_cutoff() {
    // Slot 12:00–14:30 endet exakt am Cutoff (14:30) → nicht `> cutoff` → Keep.
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(12, 0, 0).unwrap(),
        time::Time::from_hms(14, 30, 0).unwrap(),
    );
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
        approx(w.required_hours, 2.5),
        "Gap-Closure Chain C: Legacy-Keep (slot.to == cutoff) → volle 2,5h, got {}",
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

/// D-51-07 / SHC-06 Grenzfall + Gap-Closure (Chain C Legacy-Filter):
/// `active_from` inklusiv am Tag.
/// - `booking_date == active_from - 1 day` → Gate aus + Legacy-Drop (0h),
///   weil `slot.to (15:00) > cutoff (14:30)` (Pre-Phase-51-Verhalten).
/// - `booking_date == active_from` → Gate an → geclippt (0,5h).
///
/// Konkret: 2026-W31-Mo = 2026-07-27. Wir schalten das Gate mit `active_from`
/// einmal auf 2026-07-28 (=> W31-Mo liegt einen Tag davor → Legacy-Drop) und
/// einmal auf 2026-07-27 (=> W31-Mo == active_from → geclippt).
#[tokio::test]
async fn test_get_summery_for_week_stichtag_boundary() {
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());

    // Fall 1: active_from = 2026-07-28 (Di), W31-Mo (2026-07-27) < active_from
    // → Gate aus, aber Chain-C-Legacy-Filter greift (slot.to > cutoff) → Drop.
    let service_before = build_service(vec![s.clone()], vec![sd.clone()], Some("2026-07-28"));
    let summary_before = service_before
        .get_summery_for_week(YEAR, WEEK, Authentication::Full, None)
        .await
        .expect("must succeed");
    assert!(
        approx(summary_before.required_hours, 0.0),
        "SHC-06 + Gap-Closure (booking_date < active_from): Legacy-Drop → 0h, got {}",
        summary_before.required_hours
    );

    // Fall 2: active_from = 2026-07-27 (=W31-Mo), inklusiv → Gate aktiv → geclippt.
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

// ─── Gap-Closure Regression (Chain C) ───────────────────────────────────────

/// Wenn `ToggleService::get_toggle_value` `ServiceError::Unauthorized` liefert,
/// darf `get_weekly_summary` NICHT mit 401 durchschlagen. Statt dessen: Gate
/// inaktiv (Legacy) → Chain-C-Legacy-Filter → Slot mit `slot.to > cutoff`
/// wird gedroppt (Gap-Closure Phase 51).
///
/// Live-Symptom: `GET /booking-information/weekly-resource-report/{year}`
/// gab 401 zurück, weil der Endpoint intern `Authentication::Full` an den
/// `ToggleService` durchreichte und der `current_user_id → None → Unauthorized`
/// zurücklieferte. Der zentrale `shortday_gate::read_active_from`-Helper fängt
/// das jetzt ab.
#[tokio::test]
async fn test_get_weekly_summary_tolerates_toggle_unauthorized() {
    // Slot Mo 14:00–15:00 mit ShortDay-Cutoff 14:30. Gap-Closure: Legacy-Filter
    // → Slot mit slot.to (15:00) > cutoff (14:30) fällt weg → required_hours = 0.
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());

    // Custom-Setup wie in `build_service`, aber Toggle liefert Unauthorized.
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::sales_person::SalesPerson>::new())));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(|_, _| {
            Ok(Arc::from(Vec::<
                service::employee_work_details::EmployeeWorkDetails,
            >::new()))
        });

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(|_, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

    let sd_clone = vec![sd];
    let sd_year: Vec<SpecialDay> = sd_clone.clone();
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
    special_day_service
        .expect_get_by_iso_year()
        .returning(move |year, _| {
            if year == YEAR {
                Ok(Arc::from(sd_year.clone()))
            } else {
                Ok(Arc::from(Vec::<SpecialDay>::new()))
            }
        });

    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<service::reporting::ShortEmployeeReport>::new()))
        });
    reporting_service
        .expect_get_year()
        .returning(|year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[service::reporting::ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| (w, Arc::from(Vec::new())))
                .collect();
            Ok(Arc::from(out))
        });

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(Vec::<service::shiftplan_report::ShiftplanReportDay>::new()))
        });
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| {
            Ok(Arc::from(Vec::<service::shiftplan_report::ShiftplanReportDay>::new()))
        });

    let slots_clone = vec![s];
    let target_monday_2 =
        time::Date::from_iso_week_date(YEAR as i32, WEEK, time::Weekday::Monday).unwrap();
    let target_sunday_2 =
        time::Date::from_iso_week_date(YEAR as i32, WEEK, time::Weekday::Sunday).unwrap();
    let bulk_slots_2: Vec<Slot> = slots_clone
        .iter()
        .map(|s| Slot {
            valid_from: target_monday_2,
            valid_to: Some(target_sunday_2),
            ..s.clone()
        })
        .collect();
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |year, week, _, _| {
            if year == YEAR && week == WEEK {
                Ok(Arc::from(slots_clone.clone()))
            } else {
                Ok(Arc::from(Vec::<service::slot::Slot>::new()))
            }
        });
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(bulk_slots_2.clone())));

    let mut shiftplan_service_mock_2 = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock_2
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    // Kernstück des Regression-Guards: Toggle → Unauthorized.
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Err(ServiceError::Unauthorized));

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
        shiftplan_service: Arc::new(shiftplan_service_mock_2),
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

    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect(
            "Unauthorized-Toleranz: get_weekly_summary muss Ok liefern (Legacy off, kein 401)",
        );

    let w = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK)
        .expect("W31 must be present in year-view");

    assert!(
        approx(w.required_hours, 0.0),
        "Gap-Closure: Unauthorized → Legacy-Filter → Overlap-Slot gedroppt \
         → required_hours = 0, got {}",
        w.required_hours
    );
}

// ─── Gap-Closure Phase 51 (Live-Symptom): Full-Auth honoriert Toggle ────────
//
// Live-Symptom (Milestone v2.4, 2026-07-04):
// - User setzt Toggle `shortday_slot_clipping_active_from = 2026-06-28`.
// - `GET /booking-information/weekly-resource-report/{year}` läuft intern mit
//   `Authentication::Full` in `get_summery_for_week` und `get_weekly_summary`
//   und ruft `shortday_gate::read_active_from(toggle_service, Full)` auf.
// - Vor Fix: `ToggleService::get_toggle_value(name, Full, tx)` warf
//   `Unauthorized`, weil `PermissionService::current_user_id(Full) → None`
//   den `user_id.is_none()` Guard triggerte.
// - `read_active_from` fing das defensiv als `Ok(None)` ab → Legacy-Modus mit
//   `active_from = None`. In Chain C bedeutet Legacy: Slot mit `slot.to > cutoff`
//   fällt aus dem `required_hours`-Aggregat komplett heraus (`0h` statt geklippten
//   `0.5h`), was am Backend zu **anderen** falschen Zahlen führte — aber
//   ebenfalls nicht dem konfigurierten Verhalten entsprach.
//
// Fix (`toggle.rs`): `Authentication::Full` überspringt den `current_user_id`-
// Guard. Dieser Test pinnt: mit Toggle-Wert + Full → geklippte 0,5h, nicht 0h
// (Legacy) und nicht 1h (Modern ungeklippt).
#[tokio::test]
async fn test_get_weekly_summary_honors_toggle_under_full_auth() {
    // Slot Mo 14:00–15:00 + ShortDay-Cutoff 14:30 + active_from = 2026-07-01
    // (< 2026-07-27 = W31-Mo) → Gate aktiv → 0,5h.
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let sd = shortday(DayOfWeek::Monday, time::Time::from_hms(14, 30, 0).unwrap());
    // Toggle liefert den echten Wert (kein Unauthorized). Simuliert das
    // Verhalten NACH dem toggle.rs-Fix.
    let service = build_service(vec![s], vec![sd], Some("2026-07-01"));

    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed under Authentication::Full");

    let w = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK)
        .expect("W31 must be present in year-view");

    assert!(
        approx(w.required_hours, 0.5),
        "Full-Auth honoriert Toggle (Phase 51 Gap-Closure Live-Symptom): \
         Gate aktiv → 0,5h Clip, got {}",
        w.required_hours
    );
}
