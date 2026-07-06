//! Phase 52 Wave 1 — Golden-Snapshot-Fixture-Test für `get_weekly_summary`.
//!
//! Dieser Test ist das **harte Regressions-Gate** für den Weekly-Overview
//! Performance-Refactor (WOP-01..05). Er läuft gegen die AKTUELLE
//! unveränderte `get_weekly_summary`-Impl in `booking_information.rs`
//! und pinnt das erwartete `Arc<[WeeklySummary]>` als hart-kodierte
//! Rust-Literale (D-52-13, D-52-14).
//!
//! Byte-Identität via `f32::to_bits()` pro Feld (D-52-12). NaN wird
//! explizit vor `to_bits()` ausgeschlossen.
//!
//! 8 Fixture-Achsen aus WOP-03 (D-52-11):
//! 1. Baseline — leere Setup, keine besonderen Umstände.
//! 2. Holiday — `SpecialDayType::Holiday` in einer Woche.
//! 3. ShortDay — `SpecialDayType::ShortDay` mit `until`, Gate aktiv.
//! 4. Volunteer-Vacation — Freiwilliger mit Vacation-Absence-Period, VFA-01.
//! 5. CVC-06 Cap — `cap_planned_hours_to_expected=true` + Overshoot.
//! 6. Gate off — `active_from = None`, Legacy-Filter greift.
//! 7. Gate on — `active_from = Some(vor Woche N)`, Slot geclippt.
//! 8. Combined — Interaktion aller Achsen inkl. Spillover in year+1.
//!
//! Zweck: Alle folgenden Waves (2-4) müssen diesen Test byte-identisch grün
//! lassen — jeder Semantik-Drift führt zu einem sichtbaren Diff.

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod, DayFraction, MockAbsenceService};
use service::booking::MockBookingService;
use service::booking_information::{
    BookingInformationService, WeeklySummary, WorkingHoursPerSalesPerson,
};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
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
    // Phase 52 (WOP-01, D-52-01): ShiftplanService (catalog) für den
    // `is_planning`-Filter im Slot-Bulk-Load.
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

// ─── Byte-identity comparison helper (D-52-12) ────────────────────────────────

/// Vergleicht zwei f32-Werte bit-exakt via `to_bits()`. NaN ist explizit
/// verboten — der Test panickt vorher (D-52-12).
#[track_caller]
fn assert_f32_bit_eq(actual: f32, expected: f32, field: &str, idx: usize) {
    assert!(
        !actual.is_nan(),
        "field `{field}` in summary[{idx}] is NaN — golden-snapshot cannot compare NaN bits deterministically"
    );
    assert!(
        !expected.is_nan(),
        "expected `{field}` in summary[{idx}] is NaN — fixture-authoring error"
    );
    assert_eq!(
        actual.to_bits(),
        expected.to_bits(),
        "field `{field}` in summary[{idx}] diverges: actual={actual} (0x{:08x}), expected={expected} (0x{:08x})",
        actual.to_bits(),
        expected.to_bits(),
    );
}

/// Vergleicht `WorkingHoursPerSalesPerson` bit-exakt Feld für Feld.
#[track_caller]
fn assert_whps_bit_eq(
    actual: &WorkingHoursPerSalesPerson,
    expected: &WorkingHoursPerSalesPerson,
    idx: usize,
    sub_idx: usize,
) {
    assert_eq!(
        actual.sales_person_id, expected.sales_person_id,
        "sales_person_id in summary[{idx}].working_hours_per_sales_person[{sub_idx}]"
    );
    assert_eq!(
        actual.sales_person_name.as_ref(),
        expected.sales_person_name.as_ref(),
        "sales_person_name in summary[{idx}].working_hours_per_sales_person[{sub_idx}]"
    );
    let prefix = format!("whps[{sub_idx}].");
    assert_f32_bit_eq(
        actual.available_hours,
        expected.available_hours,
        &format!("{prefix}available_hours"),
        idx,
    );
    assert_f32_bit_eq(
        actual.absence_hours,
        expected.absence_hours,
        &format!("{prefix}absence_hours"),
        idx,
    );
    assert_f32_bit_eq(
        actual.vacation_hours,
        expected.vacation_hours,
        &format!("{prefix}vacation_hours"),
        idx,
    );
    assert_f32_bit_eq(
        actual.sick_leave_hours,
        expected.sick_leave_hours,
        &format!("{prefix}sick_leave_hours"),
        idx,
    );
    assert_f32_bit_eq(
        actual.holiday_hours,
        expected.holiday_hours,
        &format!("{prefix}holiday_hours"),
        idx,
    );
    assert_f32_bit_eq(
        actual.unavailable_hours,
        expected.unavailable_hours,
        &format!("{prefix}unavailable_hours"),
        idx,
    );
    assert_eq!(
        actual.custom_absence_hours.len(),
        expected.custom_absence_hours.len(),
        "custom_absence_hours length in summary[{idx}].whps[{sub_idx}]"
    );
    for (k, (a, e)) in actual
        .custom_absence_hours
        .iter()
        .zip(expected.custom_absence_hours.iter())
        .enumerate()
    {
        assert_eq!(
            a.id, e.id,
            "custom_absence_hours[{k}].id in summary[{idx}].whps[{sub_idx}]"
        );
        assert_eq!(
            a.name.as_ref(),
            e.name.as_ref(),
            "custom_absence_hours[{k}].name in summary[{idx}].whps[{sub_idx}]"
        );
        assert_f32_bit_eq(
            a.hours,
            e.hours,
            &format!("{prefix}custom_absence_hours[{k}].hours"),
            idx,
        );
    }
}

/// Vergleicht `Arc<[WeeklySummary]>` bit-exakt (D-52-12). Führt den Diff pro
/// Feld — bricht beim ersten Mismatch mit sprechender Message ab.
#[track_caller]
fn assert_weekly_summary_bit_exact(actual: &[WeeklySummary], expected: &[WeeklySummary]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "length mismatch: actual={}, expected={}",
        actual.len(),
        expected.len()
    );
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(a.year, e.year, "year at summary[{i}]");
        assert_eq!(a.week, e.week, "week at summary[{i}]");
        assert_f32_bit_eq(a.overall_available_hours, e.overall_available_hours, "overall_available_hours", i);
        assert_f32_bit_eq(a.required_hours, e.required_hours, "required_hours", i);
        assert_f32_bit_eq(a.paid_hours, e.paid_hours, "paid_hours", i);
        assert_f32_bit_eq(a.volunteer_hours, e.volunteer_hours, "volunteer_hours", i);
        assert_f32_bit_eq(a.committed_voluntary_hours, e.committed_voluntary_hours, "committed_voluntary_hours", i);
        assert_f32_bit_eq(a.monday_available_hours, e.monday_available_hours, "monday_available_hours", i);
        assert_f32_bit_eq(a.tuesday_available_hours, e.tuesday_available_hours, "tuesday_available_hours", i);
        assert_f32_bit_eq(a.wednesday_available_hours, e.wednesday_available_hours, "wednesday_available_hours", i);
        assert_f32_bit_eq(a.thursday_available_hours, e.thursday_available_hours, "thursday_available_hours", i);
        assert_f32_bit_eq(a.friday_available_hours, e.friday_available_hours, "friday_available_hours", i);
        assert_f32_bit_eq(a.saturday_available_hours, e.saturday_available_hours, "saturday_available_hours", i);
        assert_f32_bit_eq(a.sunday_available_hours, e.sunday_available_hours, "sunday_available_hours", i);
        assert_eq!(
            a.working_hours_per_sales_person.len(),
            e.working_hours_per_sales_person.len(),
            "working_hours_per_sales_person length at summary[{i}]"
        );
        for (j, (aw, ew)) in a
            .working_hours_per_sales_person
            .iter()
            .zip(e.working_hours_per_sales_person.iter())
            .enumerate()
        {
            assert_whps_bit_eq(aw, ew, i, j);
        }
    }
}

// ─── Fixture-Konstanten ───────────────────────────────────────────────────────

const YEAR: u32 = 2026;
/// 2026-W31 (Mon = 2026-07-27, ISO).
const WEEK: u8 = 31;

// ─── Empty-summary helper ─────────────────────────────────────────────────────

fn empty_summary(year: u32, week: u8) -> WeeklySummary {
    // IEEE-754 sign-of-zero note (D-52-12): the current impl produces `-0.0`
    // for `required_hours`, `volunteer_hours`, and `committed_voluntary_hours`
    // even when the underlying iterators are empty. This is due to Rust's
    // `Iterator::sum::<f32>()` starting at `-0.0` for empty iterators when
    // the map body would produce negative-zero-flavoured terms (specifically,
    // `volunteer_surplus_band2` accumulates into a `HashMap` and folds with
    // `+`, which preserves the sign of the initial neutral element).
    // `paid_hours` uses `+=` on a `mut f32 = 0.0` scalar accumulator (line
    // 447-448 in booking_information.rs), which starts at positive zero.
    // `overall_available_hours = committed + volunteer + paid = (-0) + (-0) + 0 = 0`
    // because IEEE-754 says `-0 + 0 = 0`.
    //
    // We pin the current bit-pattern as the golden snapshot (WOP-03).
    WeeklySummary {
        year,
        week,
        overall_available_hours: 0.0,
        required_hours: -0.0,
        paid_hours: 0.0,
        volunteer_hours: -0.0,
        committed_voluntary_hours: -0.0,
        monday_available_hours: 0.0,
        tuesday_available_hours: 0.0,
        wednesday_available_hours: 0.0,
        thursday_available_hours: 0.0,
        friday_available_hours: 0.0,
        saturday_available_hours: 0.0,
        sunday_available_hours: 0.0,
        working_hours_per_sales_person: Arc::from(Vec::<WorkingHoursPerSalesPerson>::new()),
        sales_person_absences: Arc::from(
            Vec::<service::booking_information::SalesPersonAbsence>::new(),
        ),
    }
}

/// Baut die 56-Wochen-Baseline-Erwartung für `YEAR=2026`: `weeks_in_year(2026)=53`,
/// Loop `1..=(53+3)` → 56 Wochen, ab Woche 54 fallthrough auf year=2027 (W1..W3).
/// Alle Felder 0.0, keine WHPS-Einträge.
fn empty_baseline_summaries(year: u32) -> Vec<WeeklySummary> {
    let weeks_in_year = time::util::weeks_in_year(year as i32);
    let mut out = Vec::with_capacity((weeks_in_year + 3) as usize);
    for week in 1..=(weeks_in_year + 3) {
        let (y, w) = if week > weeks_in_year {
            (year + 1, week - weeks_in_year)
        } else {
            (year, week)
        };
        out.push(empty_summary(y, w));
    }
    out
}

// ─── Service-Builder (parametrische Mock-Konfiguration) ──────────────────────

struct FixtureConfig {
    /// Freiwillige Sales-Personen (is_paid=false).
    volunteers: Vec<SalesPerson>,
    /// Bezahlte Sales-Personen (is_paid=true).
    paid: Vec<SalesPerson>,
    /// Alle Contracts.
    work_details: Vec<EmployeeWorkDetails>,
    /// Alle Absencen.
    absences: Vec<AbsencePeriod>,
    /// SpecialDays pro (year, week).
    special_days_by_week: std::collections::HashMap<(u32, u8), Vec<SpecialDay>>,
    /// Slots pro (year, week).
    slots_by_week: std::collections::HashMap<(u32, u8), Vec<Slot>>,
    /// ShiftplanReport-Rows pro (year, week).
    shiftplan_reports_by_week: std::collections::HashMap<(u32, u8), Vec<ShiftplanReportDay>>,
    /// ShortEmployeeReport-Rows pro (year, week).
    week_reports: std::collections::HashMap<(u32, u8), Vec<ShortEmployeeReport>>,
    /// Toggle-Wert für `shortday_slot_clipping_active_from`.
    toggle_active_from: Option<&'static str>,
    /// Ob Shiftplanner-Permission-Check erfolgreich sein soll (Basis: ja).
    is_shiftplanner: bool,
}

impl FixtureConfig {
    fn empty() -> Self {
        Self {
            volunteers: Vec::new(),
            paid: Vec::new(),
            work_details: Vec::new(),
            absences: Vec::new(),
            special_days_by_week: Default::default(),
            slots_by_week: Default::default(),
            shiftplan_reports_by_week: Default::default(),
            week_reports: Default::default(),
            toggle_active_from: None,
            is_shiftplanner: true,
        }
    }
}

fn build_service_with(config: FixtureConfig) -> BookingInformationServiceImpl<TestDeps> {
    let is_shiftplanner = config.is_shiftplanner;
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |priv_name, _| {
            // SHIFTPLANNER_PRIVILEGE = "shiftplanner", SALES_PRIVILEGE = "sales".
            if priv_name == "shiftplanner" && !is_shiftplanner {
                Err(service::ServiceError::Forbidden)
            } else {
                Ok(())
            }
        });

    let all_sp: Vec<SalesPerson> = config
        .volunteers
        .iter()
        .cloned()
        .chain(config.paid.iter().cloned())
        .collect();
    let all_sp_arc: Arc<[SalesPerson]> = Arc::from(all_sp);
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(all_sp_arc.clone()));

    let wd_arc: Arc<[EmployeeWorkDetails]> = Arc::from(config.work_details);
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(wd_arc.clone()));

    let abs_arc: Arc<[AbsencePeriod]> = Arc::from(config.absences);
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(move |_, _| Ok(abs_arc.clone()));

    // Phase 52 (WOP-01, D-52-01): SpecialDays — sowohl `get_by_week` (Legacy
    // für sekundäre Konsumenten wie `get_summery_for_week`) als auch
    // `get_by_year` (neuer Bulk-Load) werden gemockt. Der Bulk-Load flached
    // die (year, week)-Map zu einem Vec pro Jahr; der In-Memory-Filter im
    // Consumer selektiert die Zielwoche.
    let sd_map = config.special_days_by_week;
    let sd_by_year: std::collections::HashMap<u32, Vec<SpecialDay>> = {
        let mut acc: std::collections::HashMap<u32, Vec<SpecialDay>> = Default::default();
        for ((y, _), v) in sd_map.iter() {
            acc.entry(*y).or_default().extend(v.iter().cloned());
        }
        acc
    };
    let sd_map_for_week = sd_map.clone();
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |year, week, _| {
            Ok(Arc::from(
                sd_map_for_week
                    .get(&(year, week))
                    .cloned()
                    .unwrap_or_default(),
            ))
        });
    special_day_service
        .expect_get_by_iso_year()
        .returning(move |year, _| {
            Ok(Arc::from(sd_by_year.get(&year).cloned().unwrap_or_default()))
        });

    // Phase 52 (WOP-02): ReportingService — `get_week` (Legacy) + `get_year`
    // (Bulk). Der Bulk baut einen Vec pro Woche 1..=weeks_in_year(year), leere
    // Wochen erscheinen mit leerem `Arc<[]>` (D-52-03).
    let wr_map = config.week_reports;
    let wr_map_for_week = wr_map.clone();
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(move |year, week, _, _| {
            Ok(Arc::from(
                wr_map_for_week
                    .get(&(year, week))
                    .cloned()
                    .unwrap_or_default(),
            ))
        });
    reporting_service
        .expect_get_year()
        .returning(move |year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| {
                    let rows = wr_map.get(&(year, w)).cloned().unwrap_or_default();
                    (w, Arc::from(rows))
                })
                .collect();
            Ok(Arc::from(out))
        });

    // Phase 52 (WOP-01): ShiftplanReport — `_for_week` + `_for_year` (Bulk).
    let sr_map = config.shiftplan_reports_by_week;
    let sr_by_year: std::collections::HashMap<u32, Vec<ShiftplanReportDay>> = {
        let mut acc: std::collections::HashMap<u32, Vec<ShiftplanReportDay>> = Default::default();
        for ((y, _), v) in sr_map.iter() {
            acc.entry(*y).or_default().extend(v.iter().cloned());
        }
        acc
    };
    let sr_map_for_week = sr_map.clone();
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(move |year, week, _, _| {
            Ok(Arc::from(
                sr_map_for_week
                    .get(&(year, week))
                    .cloned()
                    .unwrap_or_default(),
            ))
        });
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(move |year, _, _| {
            Ok(Arc::from(sr_by_year.get(&year).cloned().unwrap_or_default()))
        });

    // Phase 52 (WOP-01, D-52-01, R1): Slots — `get_slots_for_week_all_plans`
    // (Legacy) + `get_slots` (Bulk). Der Bulk konfiguriert für JEDEN Slot
    // `valid_from = monday_of_target_week` und `valid_to = Some(sunday_of_target_week)`,
    // damit der In-Memory-DAO-Semantik-Filter im Consumer den Slot nur in
    // seiner (year, week) selektiert — reproduziert die Per-Woche-Auswahl aus
    // `get_slots_for_week_all_plans` byte-genau.
    let slots_map = config.slots_by_week;
    let bulk_slots: Vec<Slot> = slots_map
        .iter()
        .flat_map(|((y, w), slot_vec)| {
            let monday =
                time::Date::from_iso_week_date(*y as i32, *w, time::Weekday::Monday).ok();
            let sunday =
                time::Date::from_iso_week_date(*y as i32, *w, time::Weekday::Sunday).ok();
            slot_vec
                .iter()
                .map(move |s| Slot {
                    valid_from: monday.unwrap_or(s.valid_from),
                    valid_to: sunday,
                    ..s.clone()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let slots_map_for_week = slots_map.clone();
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(move |year, week, _, _| {
            Ok(Arc::from(
                slots_map_for_week
                    .get(&(year, week))
                    .cloned()
                    .unwrap_or_default(),
            ))
        });
    slot_service
        .expect_get_slots()
        .returning(move |_, _| Ok(Arc::from(bulk_slots.clone())));

    // Phase 52 (WOP-01, D-52-01): ShiftplanService.get_all — leer. Alle
    // Fixture-Slots haben `shiftplan_id = None`, damit passiert der
    // `is_planning`-Filter automatisch (siehe R1: `shiftplan.is_planning IS
    // NULL` bei LEFT-JOIN ohne Zeile).
    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    let toggle_val: Option<String> = config.toggle_active_from.map(String::from);
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(move |_, _, _| Ok(toggle_val.clone().map(Arc::from)));

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

// ─── Domain-Builder-Helper (deterministisch, hard-coded UUIDs) ───────────────

const SP_VOL_A: Uuid = Uuid::from_bytes([0x22; 16]);

fn sales_person(id: Uuid, name: &'static str, is_paid: bool) -> SalesPerson {
    SalesPerson {
        id,
        name: Arc::from(name),
        background_color: Arc::from("#ffffff"),
        is_paid: Some(is_paid),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

fn work_details(
    sales_person_id: Uuid,
    expected_hours: f32,
    committed_voluntary: f32,
    cap_planned_hours_to_expected: bool,
) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id,
        expected_hours,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2020,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 53,
        to_year: 2030,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected,
        committed_voluntary,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 24,
        created: Some(datetime!(2020 - 01 - 01 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

fn slot(dow: DayOfWeek, from: time::Time, to: time::Time) -> Slot {
    Slot {
        id: Uuid::new_v4(),
        day_of_week: dow,
        from,
        to,
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2020, time::Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: Uuid::new_v4(),
        shiftplan_id: None,
    }
}

fn holiday(year: u32, week: u8, dow: DayOfWeek) -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year,
        calendar_week: week,
        day_of_week: dow,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: Some(datetime!(2020 - 01 - 01 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

fn shortday(year: u32, week: u8, dow: DayOfWeek, cutoff_at: time::Time) -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year,
        calendar_week: week,
        day_of_week: dow,
        day_type: SpecialDayType::ShortDay,
        time_of_day: Some(cutoff_at),
        created: Some(datetime!(2020 - 01 - 01 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

fn absence_period(
    sales_person_id: Uuid,
    from: time::Date,
    to: time::Date,
    category: AbsenceCategory,
) -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::nil(),
        sales_person_id,
        category,
        from_date: from,
        to_date: to,
        description: Arc::from(""),
        created: Some(datetime!(2020 - 01 - 01 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    }
}

// ─── FIXTURE 1: BASELINE ─────────────────────────────────────────────────────

/// Baseline — komplett leeres Setup gegen `YEAR=2026` (weeks_in_year=53 → 56
/// iterationen mit Spillover ins Jahr 2027 W1..W3). Alle Felder 0.0, keine
/// working_hours_per_sales_person-Einträge. Erwartung ist die 56-Wochen-
/// Null-Vec.
#[tokio::test]
async fn fixture_1_baseline() {
    let service = build_service_with(FixtureConfig::empty());
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("baseline must succeed");
    let expected = empty_baseline_summaries(YEAR);
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 2: HOLIDAY-WOCHE ────────────────────────────────────────────────

/// Holiday am Montag in Woche 31 — der Slot Mo 09:00-17:00 wird komplett
/// aus dem `required_hours`-Aggregat gefiltert. Baseline mit einem einzigen
/// Slot + Holiday-Filter. Erwartung: alle Wochen wie Baseline, EXCEPT
/// die Ziel-Woche hat `required_hours = 0.0` (weil der Slot Holiday-gefiltert
/// wird — kein Slot außerhalb der Ziel-Woche).
#[tokio::test]
async fn fixture_2_holiday_week_n() {
    let mut cfg = FixtureConfig::empty();
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(9, 0, 0).unwrap(),
        time::Time::from_hms(17, 0, 0).unwrap(),
    );
    cfg.slots_by_week.insert((YEAR, WEEK), vec![s]);
    cfg.special_days_by_week
        .insert((YEAR, WEEK), vec![holiday(YEAR, WEEK, DayOfWeek::Monday)]);
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("holiday fixture must succeed");

    // Erwartung: exakt Baseline. Holiday-Filter → Slot entfernt → required_hours=0
    // → alle Felder 0.0.
    let expected = empty_baseline_summaries(YEAR);
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 3: SHORTDAY-WOCHE (Gate aktiv, clip) ────────────────────────────

/// ShortDay am Montag um 14:30 in Woche 31, Slot Mo 14:00-15:00, Gate aktiv
/// (active_from = 2020-01-01, W31-Mo=2026-07-27 ≥ active_from). Slot wird
/// auf 14:00-14:30 geclippt → required_hours=0.5 in Woche 31.
#[tokio::test]
async fn fixture_3_shortday_week_n() {
    let mut cfg = FixtureConfig::empty();
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    cfg.slots_by_week.insert((YEAR, WEEK), vec![s]);
    cfg.special_days_by_week.insert(
        (YEAR, WEEK),
        vec![shortday(
            YEAR,
            WEEK,
            DayOfWeek::Monday,
            time::Time::from_hms(14, 30, 0).unwrap(),
        )],
    );
    cfg.toggle_active_from = Some("2020-01-01");
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("shortday fixture must succeed");

    // Erwartung: Baseline mit `required_hours=0.5` in Woche 31.
    let mut expected = empty_baseline_summaries(YEAR);
    // Woche 31 = index (WEEK - 1) = 30 im Baseline-Vec, weil `1..=56` → idx 0..55
    let idx = (WEEK - 1) as usize;
    assert_eq!(expected[idx].year, YEAR);
    assert_eq!(expected[idx].week, WEEK);
    expected[idx].required_hours = 0.5;
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 4: VOLUNTEER-VACATION-PERIOD (VFA-01) ───────────────────────────

/// Freiwilliger (nicht bezahlt) mit Vacation-Absence-Period, die Woche 31
/// (Mo 2026-07-27 - So 2026-08-02) komplett überlappt.
/// Kein Contract → committed_voluntary_hours = 0.
/// Kein ShiftplanReport → volunteer_hours = 0.
/// Baseline: alle Felder 0. VFA-01 whole-week-out greift, aber ohne
/// commit/shiftplan-Werte ist der Effekt nicht messbar. Trotzdem: der Test
/// pinnt, dass die Absence-Period keine falschen Werte einführt.
#[tokio::test]
async fn fixture_4_volunteer_vacation_period() {
    let mut cfg = FixtureConfig::empty();
    let vol = sales_person(SP_VOL_A, "Volunteer A", false);
    cfg.volunteers.push(vol);
    cfg.absences.push(absence_period(
        SP_VOL_A,
        time::Date::from_calendar_date(2026, time::Month::July, 27).unwrap(),
        time::Date::from_calendar_date(2026, time::Month::August, 2).unwrap(),
        AbsenceCategory::Vacation,
    ));
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("volunteer-vacation fixture must succeed");

    let expected = empty_baseline_summaries(YEAR);
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 5: CVC-06 CAP AKTIV ─────────────────────────────────────────────

/// Freiwilliger mit `cap_planned_hours_to_expected=true`, `expected_hours=10`,
/// `committed_voluntary=5`. ShiftplanReport-Row Mo=8h in Woche 31.
/// Erwartung: Band 1 (committed_voluntary_hours) = 5 in Woche 31.
/// Band 2 (volunteer_hours) = max(8 - 5, 0) = 3 in Woche 31.
/// paid_hours = 0 (kein bezahlter MA, kein week_report).
/// overall_available_hours = 0 + 3 + 5 = 8.
///
/// Contract läuft Woche 1..=53 in 2020..=2030 → gilt für alle Wochen.
/// Aber die 5h committed_voluntary_hours gelten in JEDER Woche (contract-active).
/// Ergo Baseline: jede Woche committed=5. In Woche 31 kommt der Shiftplan-Report
/// dazu → volunteer_hours=3, overall=8.
#[tokio::test]
async fn fixture_5_cvc06_cap_active() {
    let mut cfg = FixtureConfig::empty();
    let vol = sales_person(SP_VOL_A, "Volunteer A", false);
    cfg.volunteers.push(vol);
    cfg.work_details.push(work_details(SP_VOL_A, 10.0, 5.0, true));
    cfg.shiftplan_reports_by_week.insert(
        (YEAR, WEEK),
        vec![ShiftplanReportDay {
            sales_person_id: SP_VOL_A,
            hours: 8.0,
            year: YEAR,
            calendar_week: WEEK,
            day_of_week: DayOfWeek::Monday,
        }],
    );
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("cvc-06 cap fixture must succeed");

    // Baseline: alle Wochen committed_voluntary_hours=5, overall_available_hours=5
    let mut expected = empty_baseline_summaries(YEAR);
    for s in expected.iter_mut() {
        s.committed_voluntary_hours = 5.0;
        s.overall_available_hours = 5.0;
    }
    // Woche 31 (idx 30): volunteer_hours=3, overall_available_hours=8
    let idx = (WEEK - 1) as usize;
    expected[idx].volunteer_hours = 3.0;
    expected[idx].overall_available_hours = 8.0;
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 6: GATE OFF (Legacy) ────────────────────────────────────────────

/// `active_from = None` → Gate inaktiv → Chain-C-Legacy-Filter: ShortDay-Slot
/// mit `slot.to > cutoff` fällt komplett weg. Slot Mo 14:00-15:00 + Cutoff
/// 14:30 → 15:00 > 14:30 → Slot gedroppt → required_hours = 0.
#[tokio::test]
async fn fixture_6_gate_off_legacy() {
    let mut cfg = FixtureConfig::empty();
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    cfg.slots_by_week.insert((YEAR, WEEK), vec![s]);
    cfg.special_days_by_week.insert(
        (YEAR, WEEK),
        vec![shortday(
            YEAR,
            WEEK,
            DayOfWeek::Monday,
            time::Time::from_hms(14, 30, 0).unwrap(),
        )],
    );
    cfg.toggle_active_from = None;
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("gate-off fixture must succeed");

    // Erwartung: Baseline. Slot gedroppt → required_hours=0 → alles 0.
    let expected = empty_baseline_summaries(YEAR);
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 7: GATE ON, active_from vor Woche N ─────────────────────────────

/// `active_from = 2020-01-01` (weit vor W31-Mo=2026-07-27) → Gate aktiv →
/// ShortDay-Slot wird geclippt (nicht verworfen). Slot Mo 14:00-15:00 +
/// Cutoff 14:30 → 14:00-14:30 → required_hours = 0.5.
/// Identisch zu Fixture 3 aber mit einem alten active_from-Datum.
#[tokio::test]
async fn fixture_7_gate_on_active_from_before_week() {
    let mut cfg = FixtureConfig::empty();
    let s = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    cfg.slots_by_week.insert((YEAR, WEEK), vec![s]);
    cfg.special_days_by_week.insert(
        (YEAR, WEEK),
        vec![shortday(
            YEAR,
            WEEK,
            DayOfWeek::Monday,
            time::Time::from_hms(14, 30, 0).unwrap(),
        )],
    );
    cfg.toggle_active_from = Some("2020-01-01");
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("gate-on fixture must succeed");

    let mut expected = empty_baseline_summaries(YEAR);
    let idx = (WEEK - 1) as usize;
    expected[idx].required_hours = 0.5;
    assert_weekly_summary_bit_exact(&actual, &expected);
}

// ─── FIXTURE 8: COMBINED + SPILLOVER ─────────────────────────────────────────

/// Alle Achsen kombiniert, plus Spillover in year+1:
/// - year=2020 (weeks_in_year=53) → Loop 1..=56, ab Woche 54 fallthrough auf
///   2021 W1..W3.
/// - Woche 53 (Ziel-Woche vor Spillover-Grenze) hat:
///   - Slot Mo 14:00-15:00 mit ShortDay-Cutoff Mo 14:30 → Gate aktiv → 0.5h
///   - Holiday am Di → kein Slot am Di
///   - Freiwilliger mit Vacation-Period, die Woche 53 überlappt
///   - CVC-06 Cap-Contract mit committed=5
/// - Woche 55 (=Spillover-year+1 W2): eigener Slot + Report
///
/// Erwartung ist deterministisch berechnet aus der Impl-Semantik.
#[tokio::test]
async fn fixture_8_combined_holiday_shortday_volunteer_cap_gate() {
    let spillover_year: u32 = 2020;
    // W53 in 2020 (Mo=2020-12-28).
    let target_week: u8 = 53;
    // W55 iteration -> falls in spillover_year+1 = 2021 W2 (1-basiert: 55-53=2).
    let spillover_year_next: u32 = 2021;
    let spillover_target_week: u8 = 2;

    let mut cfg = FixtureConfig::empty();

    // Volunteer with cap-gated contract.
    cfg.volunteers.push(sales_person(SP_VOL_A, "Volunteer A", false));
    cfg.work_details.push(work_details(SP_VOL_A, 10.0, 5.0, true));

    // Vacation absence covering W53 in 2020 (Mo 2020-12-28 - So 2021-01-03).
    cfg.absences.push(absence_period(
        SP_VOL_A,
        time::Date::from_calendar_date(2020, time::Month::December, 28).unwrap(),
        time::Date::from_calendar_date(2021, time::Month::January, 3).unwrap(),
        AbsenceCategory::Vacation,
    ));

    // Slot + ShortDay + Holiday in Woche 53.
    let s_mon = slot(
        DayOfWeek::Monday,
        time::Time::from_hms(14, 0, 0).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    );
    let s_tue = slot(
        DayOfWeek::Tuesday,
        time::Time::from_hms(9, 0, 0).unwrap(),
        time::Time::from_hms(17, 0, 0).unwrap(),
    );
    cfg.slots_by_week
        .insert((spillover_year, target_week), vec![s_mon, s_tue]);
    cfg.special_days_by_week.insert(
        (spillover_year, target_week),
        vec![
            shortday(
                spillover_year,
                target_week,
                DayOfWeek::Monday,
                time::Time::from_hms(14, 30, 0).unwrap(),
            ),
            holiday(spillover_year, target_week, DayOfWeek::Tuesday),
        ],
    );

    // ShiftplanReport in Woche 53: Volunteer Mo=8h. Aber Absence → volunteer_hours
    // fällt für ihn weg (VFA-01), da absent → committed→0 UND surplus-Person absent.
    // Beachte: VFA-01 setzt beide Bänder (committed AND volunteer_hours-Kontribution)
    // für absent-volunteer auf 0. `per_day_actuals`-Filter geht aber trotzdem durch
    // die Sum-Berechnung; die Absence wirkt via `absent_volunteer_ids`-Set in
    // `committed_for_person`-Closure → 0.
    cfg.shiftplan_reports_by_week.insert(
        (spillover_year, target_week),
        vec![ShiftplanReportDay {
            sales_person_id: SP_VOL_A,
            hours: 8.0,
            year: spillover_year,
            calendar_week: target_week,
            day_of_week: DayOfWeek::Monday,
        }],
    );

    // Spillover-Woche (year+1 W2): eigener Slot + Report (checkt R6 Off-by-one).
    let s_spill = slot(
        DayOfWeek::Wednesday,
        time::Time::from_hms(10, 0, 0).unwrap(),
        time::Time::from_hms(12, 0, 0).unwrap(),
    );
    cfg.slots_by_week
        .insert((spillover_year_next, spillover_target_week), vec![s_spill]);

    cfg.toggle_active_from = Some("2020-01-01");
    let service = build_service_with(cfg);
    let actual = service
        .get_weekly_summary(spillover_year, Authentication::Full, None)
        .await
        .expect("combined fixture must succeed");

    // ─── Erwartungs-Berechnung (deterministisch aus Impl-Semantik) ────────
    //
    // Baseline: 56 Wochen (weeks_in_year(2020)=53, Loop 1..=56).
    // In jeder Woche committed_voluntary_hours contract-abhängig:
    //   Für W1..=W52 in 2020: Contract aktiv, Volunteer NICHT absent (Absence
    //     startet 2020-12-28 = W53 Mo). Aber die Volunteer-Vacation-Absence
    //     überlappt NUR die W53. → committed=5 in W1..W52, W54..W56.
    //   W53 (2020): Volunteer absent (VFA-01) → committed=0.
    //   W54..W56 = year+1 (2021) W1..W3: Contract aktiv (2020..=2030) →
    //     committed=5. Aber Absence 2020-12-28..2021-01-03 überlappt W1 2021
    //     (Mo=2021-01-04? Nein, ISO-Woche 2021-W1 startet Mo 2021-01-04). Also
    //     überlappt sie 2020-W53 UND 2021-W53? Nein: 2020-12-28 = 2020-W53-Mo,
    //     2021-01-03 = 2020-W53-So. Die Woche 2021-W1 ist Mo 2021-01-04 - So
    //     2021-01-10, nicht überlappt. → In W54..=W56 (2021 W1..W3) ist volunteer
    //     NICHT absent → committed=5.
    //
    // In W53 (target_week):
    //   - required_hours: Slot Mo geclippt (14:00-14:30=0.5h), Slot Di
    //     Holiday-gefiltert → 0h. Sum: 0.5h.
    //   - Shiftplan-Report Volunteer Mo=8h. Aber VFA-01: absent_volunteer_ids
    //     enthält SP_VOL_A → committed_for_person(SP_VOL_A)=0, aber
    //     per_day_actuals wird trotzdem via Filter (volunteer_ids.contains).
    //     Der Surplus max(8-0, 0) = 8h wäre "raw", ABER: schauen wir in den
    //     Code — der volunteer_hours-Fold summiert per-Person: absent volunteer
    //     drops committed→0, aber per_day_actuals wird NICHT auf absent gefiltert.
    //     → volunteer_hours (Band 2) = max(8-0,0) = 8h.
    //
    // Hmm, das ist subtil. Der `per_day_actuals`-Filter ist `volunteer_ids.contains`,
    // NICHT `!absent_volunteer_ids.contains`. Also fließt der 8h-Beitrag rein und
    // committed_for_person liefert 0 (weil absent) → surplus=8h. Das ist die
    // aktuelle Impl-Semantik (Zeile 373-389 in booking_information.rs).
    //
    //   - committed_voluntary_hours = 0 (VFA-01: absent-Filter greift).
    //   - paid_hours = 0.
    //   - overall_available_hours = 0 + 8 + 0 = 8.
    //
    // Alle nicht-betroffenen Wochen (Baseline): committed=5, overall=5.
    // W53: required=0.5, volunteer=8, committed=0, overall=8.
    // W55 = 2021-W2: required = 2h (Mi 10-12h Slot), committed=5, overall=5.

    let mut expected = empty_baseline_summaries(spillover_year);
    for s in expected.iter_mut() {
        s.committed_voluntary_hours = 5.0;
        s.overall_available_hours = 5.0;
    }
    // W53 (idx 52)
    let idx_w53 = (target_week - 1) as usize;
    expected[idx_w53].required_hours = 0.5;
    expected[idx_w53].volunteer_hours = 8.0;
    // VFA-01: absent volunteer → filter out → sum of empty iter yields -0.0
    // (IEEE-754 sign-of-zero preservation; see empty_summary() note).
    expected[idx_w53].committed_voluntary_hours = -0.0;
    // overall = committed(-0) + volunteer(8) + paid(0) = 8.0 (+0 dominates in `-0 + 8`).
    expected[idx_w53].overall_available_hours = 8.0;

    // W55 (idx 54 = W2 spillover)
    // Iterations-Index in Loop: week=55 → (year=2021, week=55-53=2).
    // Vec-Index = 54.
    let idx_w55 = 54usize;
    assert_eq!(expected[idx_w55].year, spillover_year_next);
    assert_eq!(expected[idx_w55].week, spillover_target_week);
    expected[idx_w55].required_hours = 2.0;

    assert_weekly_summary_bit_exact(&actual, &expected);
}
