//! Phase 52 — 2026-Blindsweep für `required_hours` an Jahresgrenzen.
//!
//! Follow-up zu `booking_information_weekly_summary_year_boundary.rs` (2020).
//! Der User hat im Test-Env **2026** geprüft (nicht 2020). Diese Datei
//! reproduziert den 2020-Ansatz für 2026 und legt zusätzliche Fixtures auf,
//! die spezifische ISO-2026-Grenzfälle prüfen:
//!
//! - 2026 hat **53 ISO-Wochen** (`weeks_in_year(2026) == 53`).
//! - ISO-2026-W1 = Mo **2025-12-29** .. So **2026-01-04**.
//! - ISO-2026-W53 = Mo **2026-12-28** .. So **2027-01-03**.
//! - 2026-01-01 ist Donnerstag ⇒ ISO-Wochenjahr 2026, Woche 1.
//! - 2026-12-31 ist Donnerstag ⇒ ISO-Wochenjahr 2026, Woche 53.
//!
//! Zielhypothesen (falsifiable):
//! - H1: `required_hours` in 2026-W1 (Bulk-Impl) weicht von `get_summery_for_week`
//!   ab, weil `all_slots` (bulk-load, keine Jahres-Filter) einen Slot enthält,
//!   der in W1 nicht sichtbar sein sollte.
//! - H2: `required_hours` in 2026-W53 weicht ab, weil `weeks_in_year(2026)==53`
//!   und die Spillover-Remap-Logik (`year=2027, week=w−53`) einen Slot doppelt
//!   zählt oder einen ShortDay-Clip nicht findet.
//! - H3: SpecialDay am 2026-01-01 (ISO-2026-W1-Thu) wird korrekt in
//!   `special_days_this = get_by_year(2026)` gefunden und der Holiday-Filter
//!   greift für W1.
//! - H4: SpecialDay am 2027-01-01 (Freitag = ISO-2026-W53-Fri) wird im
//!   `special_days_next` gefunden **oder** im `special_days_this`, je nach
//!   ISO-Wochenjahr-Speicherung. → Grenzfall für den Filter.

use std::sync::Arc;

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

// ─── ISO-2026-Boundary sanity ────────────────────────────────────────────────

#[test]
fn iso_2026_boundary_facts() {
    use time::util::weeks_in_year;
    use time::{Date, Month, Weekday};

    // 53-Wochen-Jahr sandwiched zwischen 52-Wochen-Jahren.
    assert_eq!(weeks_in_year(2025), 52);
    assert_eq!(weeks_in_year(2026), 53);
    assert_eq!(weeks_in_year(2027), 52);

    // ISO-2026-W1: Mo=2025-12-29 (im Kalenderjahr 2025!), So=2026-01-04.
    let mon_w1 = Date::from_iso_week_date(2026, 1, Weekday::Monday).unwrap();
    let sun_w1 = Date::from_iso_week_date(2026, 1, Weekday::Sunday).unwrap();
    assert_eq!(
        mon_w1,
        Date::from_calendar_date(2025, Month::December, 29).unwrap()
    );
    assert_eq!(
        sun_w1,
        Date::from_calendar_date(2026, Month::January, 4).unwrap()
    );

    // ISO-2026-W53: Mo=2026-12-28, So=2027-01-03 (crosses into 2027).
    let mon_w53 = Date::from_iso_week_date(2026, 53, Weekday::Monday).unwrap();
    let sun_w53 = Date::from_iso_week_date(2026, 53, Weekday::Sunday).unwrap();
    assert_eq!(
        mon_w53,
        Date::from_calendar_date(2026, Month::December, 28).unwrap()
    );
    assert_eq!(
        sun_w53,
        Date::from_calendar_date(2027, Month::January, 3).unwrap()
    );

    // ISO-2027-W1: Mo=2027-01-04 (nach 2027-01-03), So=2027-01-10.
    let mon_w1_27 = Date::from_iso_week_date(2027, 1, Weekday::Monday).unwrap();
    let sun_w1_27 = Date::from_iso_week_date(2027, 1, Weekday::Sunday).unwrap();
    assert_eq!(
        mon_w1_27,
        Date::from_calendar_date(2027, Month::January, 4).unwrap()
    );
    assert_eq!(
        sun_w1_27,
        Date::from_calendar_date(2027, Month::January, 10).unwrap()
    );

    // 2026-01-01 = Do; ISO-2026-W1-Thu.
    let d1 = Date::from_calendar_date(2026, Month::January, 1).unwrap();
    assert_eq!(d1.weekday(), Weekday::Thursday);
    assert_eq!(d1.iso_week(), 1);
    assert_eq!(d1.to_iso_week_date().0, 2026);

    // 2026-12-31 = Do; ISO-2026-W53-Thu.
    let d2 = Date::from_calendar_date(2026, Month::December, 31).unwrap();
    assert_eq!(d2.weekday(), Weekday::Thursday);
    assert_eq!(d2.iso_week(), 53);
    assert_eq!(d2.to_iso_week_date().0, 2026);

    // 2027-01-01 = Fr; ISO-2026-W53-Fri! (Kalender-Jahr 2027, ISO-Wochenjahr 2026)
    let d3 = Date::from_calendar_date(2027, Month::January, 1).unwrap();
    assert_eq!(d3.weekday(), Weekday::Friday);
    assert_eq!(d3.iso_week(), 53);
    assert_eq!(d3.to_iso_week_date().0, 2026);
}

// ─── Shared Service-Builder mit steuerbaren Fixture-Elementen ─────────────────

struct Fixture {
    /// Slots, die der Bulk-Load `slot_service.get_slots()` zurückgibt.
    slots: Vec<Slot>,
    /// SpecialDays PER (year, week) — für Legacy-Path (`get_by_week`).
    special_days_by_week: std::collections::HashMap<(u32, u8), Vec<SpecialDay>>,
    /// SpecialDays PER Kalender-Jahr — für Bulk-Path (`get_by_year`).
    /// Die echte Impl macht Union(year, year-1) und filtert nach Kalender-Datum;
    /// im Test füttern wir direkt die "as if by get_by_year(y)" Liste.
    special_days_by_year: std::collections::HashMap<u32, Vec<SpecialDay>>,
    /// active_from für den Chain-C-Toggle. None = Gate aus (Legacy-Modus).
    active_from: Option<String>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            special_days_by_week: std::collections::HashMap::new(),
            special_days_by_year: std::collections::HashMap::new(),
            active_from: None,
        }
    }
    fn with_slot(mut self, slot: Slot) -> Self {
        self.slots.push(slot);
        self
    }
    fn with_special_day_at(mut self, year: u32, week: u8, day: SpecialDay) -> Self {
        self.special_days_by_week
            .entry((year, week))
            .or_default()
            .push(day.clone());
        // Auch dem Bulk-Bucket für alle Kalender-Jahre zufügen, die diese Woche
        // berührt (grob heuristisch: für year und year-1 und year+1).
        self.special_days_by_year
            .entry(year)
            .or_default()
            .push(day.clone());
        self
    }
    /// Wenn die Bulk-Impl `get_by_year(cal_year)` aufruft, wollen wir ein
    /// bestimmtes Set liefern. Für den Test füllen wir das explizit.
    #[allow(dead_code)]
    fn with_special_days_get_by_year(mut self, year: u32, days: Vec<SpecialDay>) -> Self {
        self.special_days_by_year.insert(year, days);
        self
    }
}

fn build_service(fixture: Fixture) -> BookingInformationServiceImpl<TestDeps> {
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

    // Legacy special_day: get_by_week(y,w).
    let sd_by_week = fixture.special_days_by_week.clone();
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |y, w, _| {
            let days = sd_by_week.get(&(y, w)).cloned().unwrap_or_default();
            Ok(Arc::from(days))
        });
    // Bulk special_day: get_by_year(cal_year) — real Impl unioniert (year, year-1)
    // und filtert per Kalender-Datum. Für den Mock: liefer die vom Test
    // hinterlegte Liste (soll semantisch das gleiche Ergebnis geben).
    let sd_by_year = fixture.special_days_by_year.clone();
    special_day_service
        .expect_get_by_iso_year()
        .returning(move |y, _| {
            let days = sd_by_year.get(&y).cloned().unwrap_or_default();
            Ok(Arc::from(days))
        });

    // Reporting: leer (wir prüfen hier nur required_hours, nicht paid_hours).
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ShortEmployeeReport>::new())));
    reporting_service
        .expect_get_year()
        .returning(|year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| (w, Arc::from(Vec::<ShortEmployeeReport>::new())))
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

    // Slot-Service: Legacy filtert per Woche wie der echte DAO;
    // Bulk liefert die ganze Slot-Liste (jahresagnostisch).
    let all_slots = fixture.slots.clone();
    let legacy_slots = fixture.slots.clone();
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |year, week, _, _| {
            let monday =
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday).unwrap();
            let sunday =
                time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday).unwrap();
            let filtered: Vec<Slot> = legacy_slots
                .iter()
                .filter(|s| {
                    s.deleted.is_none()
                        && s.valid_from <= sunday
                        && s.valid_to.map(|vt| vt >= monday).unwrap_or(true)
                })
                .cloned()
                .collect();
            Ok(Arc::from(filtered))
        });
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(all_slots.clone())));

    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    let active_from: Option<Arc<str>> = fixture.active_from.clone().map(Arc::from);
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(move |_, _, _| Ok(active_from.clone()));

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

fn slot(
    day: DayOfWeek,
    from_h: u8,
    to_h: u8,
    valid_from: (i32, u8, u8),
    valid_to: Option<(i32, u8, u8)>,
) -> Slot {
    Slot {
        id: Uuid::new_v4(),
        day_of_week: day,
        from: time::Time::from_hms(from_h, 0, 0).unwrap(),
        to: time::Time::from_hms(to_h, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(
            valid_from.0,
            time::Month::try_from(valid_from.1).unwrap(),
            valid_from.2,
        )
        .unwrap(),
        valid_to: valid_to.map(|(y, m, d)| {
            time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
        }),
        deleted: None,
        version: Uuid::nil(),
        shiftplan_id: None,
    }
}

fn special_day(year: u32, week: u8, dow: DayOfWeek, ty: SpecialDayType, hour: Option<u8>) -> SpecialDay {
    SpecialDay {
        id: Uuid::new_v4(),
        year,
        calendar_week: week,
        day_of_week: dow,
        day_type: ty,
        time_of_day: hour.map(|h| time::Time::from_hms(h, 0, 0).unwrap()),
        created: Some(time::PrimitiveDateTime::MIN),
        deleted: None,
        version: Uuid::nil(),
    }
}

// ─── H1 / H2: Immer-aktiver Slot in 2026 → jede Woche 8h ─────────────────────

/// Bulk-Impl vs Legacy: Slot valid_from=2019, valid_to=None, Mo 10-18h. In JEDER
/// Woche des 2026-Loops (inkl. Spillover 2027-W1..W3) muss required_hours=8.0.
#[tokio::test]
async fn h1_immer_aktiver_slot_2026_jede_woche_8h() {
    let fixture = Fixture::new().with_slot(slot(
        DayOfWeek::Monday,
        10,
        18,
        (2019, 1, 1),
        None,
    ));
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    assert_eq!(bulk.len(), 56, "2026 hat weeks_in_year=53 → 53+3 Spillover");

    for s in bulk.iter() {
        assert!(
            (s.required_hours - 8.0).abs() < 1e-6,
            "week idx year={} week={}: required={}",
            s.year,
            s.week,
            s.required_hours
        );
    }
}

/// Cross-check: same fixture, but compare bulk[i] with legacy.get_summery_for_week(y,w).
#[tokio::test]
async fn h2_immer_aktiver_slot_bulk_vs_legacy_2026() {
    let fixture = Fixture::new().with_slot(slot(
        DayOfWeek::Monday,
        10,
        18,
        (2019, 1, 1),
        None,
    ));
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");

    let mut mismatches = Vec::new();
    for s in bulk.iter() {
        let legacy = service
            .get_summery_for_week(s.year, s.week, Authentication::Full, None)
            .await
            .expect("legacy");
        if (s.required_hours - legacy.required_hours).abs() > 1e-6 {
            mismatches.push(format!(
                "year={} week={}: bulk={} legacy={}",
                s.year, s.week, s.required_hours, legacy.required_hours
            ));
        }
    }
    assert!(mismatches.is_empty(), "diffs:\n  - {}", mismatches.join("\n  - "));
}

/// Slot mit `valid_to = 2026-12-31` (Kalender-Jahr-Ende, Do). Slot ist in
/// ISO-2026-W53 (Mo=2026-12-28, So=2027-01-03) noch aktiv (valid_to >= monday
/// = 2026-12-28), aber ab W1 des ISO-2027 (Mo=2027-01-04) inaktiv.
/// Bulk vs Legacy müssen bit-identisch sein.
#[tokio::test]
async fn h2b_slot_valid_to_2026_12_31_bulk_vs_legacy() {
    let fixture = Fixture::new().with_slot(slot(
        DayOfWeek::Monday,
        10,
        18,
        (2026, 1, 1),
        Some((2026, 12, 31)),
    ));
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");

    let mut mismatches = Vec::new();
    for s in bulk.iter() {
        let legacy = service
            .get_summery_for_week(s.year, s.week, Authentication::Full, None)
            .await
            .expect("legacy");
        if (s.required_hours - legacy.required_hours).abs() > 1e-6 {
            mismatches.push(format!(
                "year={} week={}: bulk={} legacy={}",
                s.year, s.week, s.required_hours, legacy.required_hours
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "valid_to=2026-12-31 diffs:\n  - {}",
        mismatches.join("\n  - ")
    );
}

/// Slot mit valid_from = 2026-06-01 (Sommer 2026), valid_to = None. In 2026-W1
/// (Mo=2025-12-29 .. So=2026-01-04) darf der Slot NICHT gefunden werden
/// (valid_from > sunday). In 2026-W53 (Mo=2026-12-28) MUSS er gefunden werden.
#[tokio::test]
async fn h2c_slot_valid_from_2026_06_01_boundary_correctness() {
    let fixture = Fixture::new().with_slot(slot(
        DayOfWeek::Monday,
        10,
        18,
        (2026, 6, 1),
        None,
    ));
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");

    // Woche 1 (2026): required = 0 (Slot ist ab 2026-06-01 gültig, W1-Sonntag=2026-01-04).
    let w1 = bulk.iter().find(|s| s.year == 2026 && s.week == 1).unwrap();
    assert!(
        (w1.required_hours - 0.0).abs() < 1e-6,
        "2026-W1 required should be 0.0 (slot starts 2026-06-01), got {}",
        w1.required_hours
    );

    // Woche 53 (2026, Mo=2026-12-28): valid_from=2026-06-01 <= 2027-01-03 (sunday) ✓
    let w53 = bulk.iter().find(|s| s.year == 2026 && s.week == 53).unwrap();
    assert!(
        (w53.required_hours - 8.0).abs() < 1e-6,
        "2026-W53 required should be 8.0, got {}",
        w53.required_hours
    );

    // Cross-check with legacy for both weeks.
    let leg_w1 = service
        .get_summery_for_week(2026, 1, Authentication::Full, None)
        .await
        .unwrap();
    let leg_w53 = service
        .get_summery_for_week(2026, 53, Authentication::Full, None)
        .await
        .unwrap();
    assert_eq!(w1.required_hours.to_bits(), leg_w1.required_hours.to_bits());
    assert_eq!(w53.required_hours.to_bits(), leg_w53.required_hours.to_bits());
}

// ─── H3: Holiday am 2026-01-01 (ISO-2026-W1-Thu) ──────────────────────────────

/// Ein Slot am Donnerstag 10-18h, und ein Holiday am 2026-01-01 (Do = W1-Thu).
/// Der Holiday-Filter soll den Slot in W1 komplett entfernen (nicht clippen).
/// → required_hours(W1) = 0.0.
///
/// **Bulk-Impl:** `special_days_this = get_by_year(2026)` MUSS diesen Holiday
/// enthalten. Der Filter `d.year==2026 && d.calendar_week==1` MUSS matchen.
#[tokio::test]
async fn h3_holiday_2026_01_01_filters_slot_in_w1() {
    let holiday = special_day(2026, 1, DayOfWeek::Thursday, SpecialDayType::Holiday, None);
    let fixture = Fixture::new()
        .with_slot(slot(DayOfWeek::Thursday, 10, 18, (2019, 1, 1), None))
        // Legacy get_by_week(2026, 1) sieht den Holiday.
        .with_special_day_at(2026, 1, holiday.clone())
        // Bulk get_by_year(2026) muss auch sehen. (with_special_day_at fügt es hinzu.)
        ;
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    let w1 = bulk.iter().find(|s| s.year == 2026 && s.week == 1).unwrap();

    let legacy_w1 = service
        .get_summery_for_week(2026, 1, Authentication::Full, None)
        .await
        .expect("legacy");

    assert_eq!(
        w1.required_hours.to_bits(),
        legacy_w1.required_hours.to_bits(),
        "bulk={} legacy={}",
        w1.required_hours,
        legacy_w1.required_hours,
    );
    // Und der Holiday soll den Slot filtern.
    assert!(
        (w1.required_hours - 0.0).abs() < 1e-6,
        "Holiday at 2026-01-01 (W1-Thu) should filter Thursday slot, got {}",
        w1.required_hours
    );
}

// ─── H4: Holiday am 2027-01-01 (ISO-2026-W53-Fri, Kalenderjahr 2027) ──────────

/// Ein Feiertag am 2027-01-01 (Freitag). Er wird in der DB als
/// `(year=2026, calendar_week=53, day=Fri)` gespeichert (ISO-Wochenjahr),
/// weil `2027-01-01.to_iso_week_date() == (2026, 53)`.
///
/// - Bulk-Impl: `special_days_this = get_by_year(2026)` liefert nach der
///   echten Impl **die Union aus (year=2026 UND year=2025)** filtered by
///   Kalender-Datum in 2026. Aber 2027-01-01 ist im Kalenderjahr **2027**,
///   also NICHT in dieser Union. → Slot wird NICHT gefiltert.
/// - Loop-Iteration W53 (outer_year=2026): `special_days_source =
///   special_days_this`. Filter `d.year==2026 && d.calendar_week==53` — die
///   Row `year=2026, week=53, day=Fri` ist zwar semantisch relevant, aber
///   in unserem `special_days_this` GAR NICHT ENTHALTEN (weil Kalender-
///   Datum 2027-01-01 in `get_by_year(2026)` wegge­filtert wird).
/// - Legacy-Impl: `get_by_week(2026, 53)` → DAO-Query `WHERE year=2026 AND
///   calendar_week=53` — findet die Row. → Slot WIRD gefiltert.
///
/// **Erwartung (Bug-Hypothese H4):** Bulk `required_hours(2026-W53)` ≠
/// Legacy `required_hours(2026-W53)`, weil der Bulk-Path den Holiday am
/// 2027-01-01 nicht sieht.
///
/// **Alternativ:** Wenn `get_by_year` semantisch als "alle Rows die ihr
/// Kalender-Datum in `year` haben" implementiert ist und die Bulk-Impl in
/// `booking_information.rs` das trotzdem korrekt findet, ist H4 falsifiziert.
///
/// Wir müssen den Fixture `special_days_by_year` sorgfältig aufsetzen, um
/// die echte Impl-Semantik zu spiegeln:
/// - `get_by_year(2026)` in der Realität würde die 2027-01-01-Row NICHT
///   enthalten (Kalender-Datum in 2027).
/// - `get_by_year(2027)` würde sie enthalten (Kalender-Datum in 2027, DB-
///   `year=2026`, wird durch Union(2027, 2026)-Filter gefunden).
/// **REGRESSION-GATE (post Follow-up #3).**
///
/// Feiertag am 2027-01-01 (Fr, ISO-2026-W53-Fri). DB-Row:
/// `(year=2026, calendar_week=53, day=Fri)`.
///
/// **Bug-Zustand vor Follow-up #3:** Bulk-Impl rief
/// `special_day_service.get_by_year(2026)`, das per Kalender-Datum filterte.
/// 2027-01-01 fiel raus → Feiertag im Bulk-Pfad unsichtbar → Slot blieb →
/// `required_hours(W53) = 8.0`, Legacy 0.0, Drift +8h.
///
/// **Nach Fix:** Bulk-Impl ruft `get_by_iso_year(2026)`, das direkt an
/// `SpecialDayDao::find_by_iso_year` delegiert (matched das ISO-Wochenjahr-
/// Feld `year` in der DB). Die Row `(year=2026, week=53, day=Fri)` landet im
/// `iso_year=2026`-Bucket → Loop-Filter matched → Slot wird gedroppt.
/// Bulk = Legacy = 0.0.
#[tokio::test]
async fn h4_holiday_2027_01_01_iso_2026_w53_bulk_vs_legacy() {
    // DB-Row: year=2026, calendar_week=53, day=Fri, Holiday.
    let holiday = special_day(2026, 53, DayOfWeek::Friday, SpecialDayType::Holiday, None);

    let mut sd_by_week = std::collections::HashMap::new();
    sd_by_week.insert((2026u32, 53u8), vec![holiday.clone()]);

    // Follow-up #3 Mock-Semantik: `get_by_iso_year(2026)` liefert die Row
    // direkt aus der DB (Spalte `year` IS ISO-Wochenjahr, kein Kalender-
    // Datum-Post-Filter).
    let mut sd_by_year = std::collections::HashMap::new();
    sd_by_year.insert(2026u32, vec![holiday.clone()]);
    // ISO-Jahr 2027 enthält die Row NICHT (Row lebt in ISO-2026-W53).
    sd_by_year.insert(2027u32, Vec::new());

    let fixture = Fixture {
        slots: vec![slot(DayOfWeek::Friday, 10, 18, (2019, 1, 1), None)],
        special_days_by_week: sd_by_week,
        special_days_by_year: sd_by_year,
        active_from: None,
    };
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    let w53_bulk = bulk.iter().find(|s| s.year == 2026 && s.week == 53).unwrap();

    let legacy_w53 = service
        .get_summery_for_week(2026, 53, Authentication::Full, None)
        .await
        .expect("legacy");

    // REGRESSION-GATE: bulk und legacy müssen bit-identisch 0.0h sein.
    assert_eq!(
        w53_bulk.required_hours.to_bits(),
        legacy_w53.required_hours.to_bits(),
        "H4 Regression: bulk W53={}h, legacy W53={}h — Follow-up #3 Fix gebrochen",
        w53_bulk.required_hours,
        legacy_w53.required_hours,
    );
    assert!(
        w53_bulk.required_hours.abs() < 1e-6,
        "H4 bulk W53 sollte 0.0h sein (Holiday gedroppt), got {}",
        w53_bulk.required_hours,
    );
    assert!(
        legacy_w53.required_hours.abs() < 1e-6,
        "H4 legacy W53 sollte 0.0h sein (Holiday gedroppt), got {}",
        legacy_w53.required_hours,
    );
}

// ─── H5: ShortDay am 2026-12-31 (ISO-2026-W53-Thu) ────────────────────────────

/// ShortDay am 2026-12-31 (Do = W53-Thu). DB-Row: `year=2026, week=53,
/// day=Thu, ShortDay, time_of_day=14:00`. Ein Slot Do 10-18h wird im
/// Legacy-Mode (active_from=None) gedroppt, weil slot.to (18) > cutoff (14).
///
/// - Legacy-Impl: `get_by_week(2026, 53)` → sieht die Row → Slot dropped.
/// - Bulk-Impl: `get_by_year(2026)` → Row hat Kalender-Datum 2026-12-31,
///   das ist noch in 2026 → drin. Filter `d.year==2026 && d.calendar_week==53`
///   matched. → Slot dropped.
///
/// Beide sollten identisch sein. Wenn nicht → Bug in Bulk-Path.
#[tokio::test]
async fn h5_shortday_2026_12_31_iso_2026_w53_bulk_vs_legacy() {
    let sd = special_day(2026, 53, DayOfWeek::Thursday, SpecialDayType::ShortDay, Some(14));

    let mut sd_by_week = std::collections::HashMap::new();
    sd_by_week.insert((2026u32, 53u8), vec![sd.clone()]);

    let mut sd_by_year = std::collections::HashMap::new();
    // 2026-12-31 ist Kalender-Datum in 2026 → in get_by_year(2026).
    sd_by_year.insert(2026u32, vec![sd.clone()]);
    sd_by_year.insert(2027u32, Vec::new());

    let fixture = Fixture {
        slots: vec![slot(DayOfWeek::Thursday, 10, 18, (2019, 1, 1), None)],
        special_days_by_week: sd_by_week,
        special_days_by_year: sd_by_year,
        active_from: None, // Gate off → Legacy-Mode: slot.to > cutoff ⇒ Drop.
    };
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    let w53_bulk = bulk.iter().find(|s| s.year == 2026 && s.week == 53).unwrap();

    let legacy_w53 = service
        .get_summery_for_week(2026, 53, Authentication::Full, None)
        .await
        .expect("legacy");

    println!(
        "H5: bulk[W53].required = {}, legacy[W53].required = {}",
        w53_bulk.required_hours, legacy_w53.required_hours
    );
    assert_eq!(
        w53_bulk.required_hours.to_bits(),
        legacy_w53.required_hours.to_bits(),
    );
    // In Legacy-Mode: ShortDay droppt Slot komplett (slot.to > cutoff).
    // Erwartung: beide = 0.0.
    assert!(
        (w53_bulk.required_hours - 0.0).abs() < 1e-6,
        "Legacy-Mode ShortDay at W53-Thu with slot.to(18) > cutoff(14) should Drop → 0.0",
    );
}

// ─── H4b: Symmetrischer Fall — Holiday am 2025-12-29 (ISO-2026-W1-Mo) ────────

/// **REGRESSION-GATE (post Follow-up #3, symmetrisch zu H4).**
///
/// Feiertag am 2025-12-29 (Montag). DB-Row: `(year=2026, week=1, day=Mon)`
/// (weil `2025-12-29.to_iso_week_date() == (2026, 1, Mon)`).
///
/// **Bug-Zustand vor Follow-up #3:** Bulk-Impl rief `get_by_year(2026)`
/// (Kalender-Datum-Filter). 2025-12-29 fiel raus (Kalender-Jahr 2025) →
/// Feiertag im Bulk-Pfad unsichtbar → Slot blieb → bulk W1 = 8.0, legacy 0.0,
/// Drift +8h.
///
/// **Nach Fix:** Bulk-Impl ruft `get_by_iso_year(2026)`, das die Row direkt
/// aus der DB liest (Spalte `year` IS ISO-Wochenjahr). Row landet im
/// `iso_year=2026`-Bucket → Loop-Filter matched → Slot gedroppt.
/// Bulk = Legacy = 0.0.
#[tokio::test]
async fn h4b_holiday_2025_12_29_iso_2026_w1_bulk_vs_legacy() {
    let holiday = special_day(2026, 1, DayOfWeek::Monday, SpecialDayType::Holiday, None);

    let mut sd_by_week = std::collections::HashMap::new();
    sd_by_week.insert((2026u32, 1u8), vec![holiday.clone()]);

    // Follow-up #3 Mock-Semantik: `get_by_iso_year(2026)` liefert die Row
    // direkt (Spalte `year=2026` IS ISO-Wochenjahr).
    let mut sd_by_year = std::collections::HashMap::new();
    sd_by_year.insert(2026u32, vec![holiday.clone()]);
    // ISO-Jahr 2025 / 2027 enthalten die Row NICHT.
    sd_by_year.insert(2025u32, Vec::new());
    sd_by_year.insert(2027u32, Vec::new());

    let fixture = Fixture {
        slots: vec![slot(DayOfWeek::Monday, 10, 18, (2019, 1, 1), None)],
        special_days_by_week: sd_by_week,
        special_days_by_year: sd_by_year,
        active_from: None,
    };
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    let w1_bulk = bulk.iter().find(|s| s.year == 2026 && s.week == 1).unwrap();
    let legacy_w1 = service
        .get_summery_for_week(2026, 1, Authentication::Full, None)
        .await
        .expect("legacy");

    // REGRESSION-GATE: bulk und legacy müssen bit-identisch 0.0h sein.
    assert_eq!(
        w1_bulk.required_hours.to_bits(),
        legacy_w1.required_hours.to_bits(),
        "H4b Regression: bulk W1={}h, legacy W1={}h — Follow-up #3 Fix gebrochen",
        w1_bulk.required_hours,
        legacy_w1.required_hours,
    );
    assert!(
        w1_bulk.required_hours.abs() < 1e-6,
        "H4b bulk W1 sollte 0.0h sein (Holiday gedroppt), got {}",
        w1_bulk.required_hours,
    );
    assert!(
        legacy_w1.required_hours.abs() < 1e-6,
        "H4b legacy W1 sollte 0.0h sein (Holiday gedroppt), got {}",
        legacy_w1.required_hours,
    );
}

// ─── H6: Holiday am 2026-01-01 in `special_days_this` und Loop-Filter ─────────

/// Sanity: Holiday am 2026-01-01. In der ECHTEN `get_by_year(2026)` (per
/// Impl in special_days.rs:82) wird die Union(2026, 2025) gebildet und per
/// `to_date().year() == 2026` gefiltert.
///
/// DB-Row: `(year=2026, week=1, day=Thu)` mit `to_date() = 2026-01-01`
/// → in `get_by_year(2026)` DRIN.
///
/// Loop-Iteration W1 mit `year=2026, week=1`. Filter `d.year==2026 &&
/// d.calendar_week==1` matched. → Bulk sieht Holiday. Legacy sieht ebenfalls.
/// Beide filtern den Thursday-Slot. → required=0.0.
///
/// Aber: der Loop hat auch eine iteration am ANFANG (week=1 vom Ur-Loop, nicht
/// vom Spillover). Prüfen wir das genauer.
#[tokio::test]
async fn h6_holiday_2026_01_01_variant_iso_year_match() {
    let holiday = special_day(2026, 1, DayOfWeek::Thursday, SpecialDayType::Holiday, None);
    let mut sd_by_week = std::collections::HashMap::new();
    sd_by_week.insert((2026u32, 1u8), vec![holiday.clone()]);
    let mut sd_by_year = std::collections::HashMap::new();
    sd_by_year.insert(2026u32, vec![holiday.clone()]);
    sd_by_year.insert(2027u32, Vec::new());

    let fixture = Fixture {
        slots: vec![slot(DayOfWeek::Thursday, 10, 18, (2019, 1, 1), None)],
        special_days_by_week: sd_by_week,
        special_days_by_year: sd_by_year,
        active_from: None,
    };
    let service = build_service(fixture);

    let bulk = service
        .get_weekly_summary(2026, Authentication::Full, None)
        .await
        .expect("bulk");
    let w1_bulk = bulk.iter().find(|s| s.year == 2026 && s.week == 1).unwrap();
    let legacy_w1 = service
        .get_summery_for_week(2026, 1, Authentication::Full, None)
        .await
        .expect("legacy");

    println!(
        "H6: bulk[W1].required = {}, legacy[W1].required = {}",
        w1_bulk.required_hours, legacy_w1.required_hours
    );
    assert_eq!(
        w1_bulk.required_hours.to_bits(),
        legacy_w1.required_hours.to_bits(),
    );
    assert!((w1_bulk.required_hours - 0.0).abs() < 1e-6);
}
