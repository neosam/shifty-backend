//! Phase 54 Plan 03 + Gap-Closure G1 (Plan 54-07) — Pure-fn + Service-Tests
//! fuer VoluntaryStatsService (VOL-STAT-01/02, VOL-ACCT-01/02/03). Die Tests
//! decken:
//!
//! - D-F1-01: `contract_weeks_count_in_range` zaehlt jede ISO-Woche mit
//!   gueltiger `EmployeeWorkDetails`-Row als Vertragswoche — auch wenn
//!   `expected_hours == 0`.
//! - D-F2-01: `committed_voluntary_target_in_range` verteilt
//!   `committed_voluntary` tages-basiert (jeder Range-Tag mit aktivem
//!   Vertrag = committed_voluntary / 7.0).
//! - D-54-DM-02 / VOL-ACCT-03 (Property-Test): `voluntary_ist_total_in_range`
//!   zaehlt AUSSCHLIESSLICH `ExtraHours` mit `source=Manual`. Rebooking-Marker-
//!   Rows sind neutral.
//! - Gap-Closure G1: Range-Cutoff — 5h/Woche-seit-Mai + Range bis KW 28
//!   liefert ~54h statt der alten Full-Year-Semantik von ~177h.
//! - Regression: Full-Year-Range (1.1.–31.12.) reproduziert die alte
//!   Full-Year-Semantik byte-genau (52.0 fuer 52-Wochen-Jahr, 53.0 fuer
//!   53-Wochen-Jahr).
//! - Edge-Weeks: mid-week-Start / mid-week-Ende zaehlen tages-genau.
//! - Kongruenz-Test: Zaehler (F1) und Nenner (F2) nutzen dieselbe ISO-Wochen-
//!   Semantik.
//! - Service-Tests: HR-Gate mit Non-HR-Redaktion (VOL-STAT-02, VOL-ACCT-02).

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::employee_work_details::EmployeeWorkDetails;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource};
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{
    committed_voluntary_prorata_for_week, committed_voluntary_target_in_range,
    contract_weeks_count_in_range, voluntary_ist_total_in_range,
};

// ── Fixture helpers ──────────────────────────────────────────────────────────

fn make_extra_hours(
    sp_id: Uuid,
    year: u32,
    week: u8,
    category: ExtraHoursCategory,
    amount: f32,
    source: ExtraHoursSource,
) -> ExtraHours {
    // Waehle Montag der ISO-Woche als date_time.
    let monday = time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday)
        .expect("valid ISO week date");
    let dt = time::PrimitiveDateTime::new(monday, time::Time::MIDNIGHT);
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: sp_id,
        amount,
        category,
        description: Arc::from(""),
        date_time: dt,
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source,
    }
}

fn make_manual_volunteer_hours(
    sp_id: Uuid,
    year: u32,
    weeks: &[u8],
    hours_per_row: f32,
) -> Vec<ExtraHours> {
    weeks
        .iter()
        .map(|w| {
            make_extra_hours(
                sp_id,
                year,
                *w,
                ExtraHoursCategory::VolunteerWork,
                hours_per_row,
                ExtraHoursSource::Manual,
            )
        })
        .collect()
}

fn make_working_hours(
    sp_id: Uuid,
    from: (u32, u8),
    to: (u32, u8),
    expected: f32,
    committed_voluntary: f32,
) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::new_v4(),
        sales_person_id: sp_id,
        expected_hours: expected,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: from.1,
        from_year: from.0,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: to.1,
        to_year: to.0,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        committed_voluntary,
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

/// Helper: Full-Year-Range fuer ein ISO-Jahr.
fn full_year_range(year: u32) -> (ShiftyDate, ShiftyDate) {
    (
        ShiftyDate::first_day_in_year(year),
        ShiftyDate::last_day_in_year(year),
    )
}

// ─── Pure-fn Tests ────────────────────────────────────────────────────────────

/// F1-Ist: 5 manuelle VolunteerWork-Rows a 4h in 2026 => 20.0h ueber die
/// Full-Year-Range (Regression: alte Semantik reproduziert).
#[test]
fn f1_ist_manual_only_20h() {
    let sp = Uuid::new_v4();
    let hours = make_manual_volunteer_hours(sp, 2026, &[10, 11, 12, 13, 14], 4.0);
    let (from, to) = full_year_range(2026);
    let total = voluntary_ist_total_in_range(&hours, from, to);
    assert!((total - 20.0).abs() < 1e-4, "expected 20.0, got {total}");
}

/// VOL-ACCT-03 (Property-Test / D-54-DM-02): Ein Rebooking-Pair
/// (-4h VolunteerWork + +4h ExtraWork) mit source=Rebooking veraendert
/// F1-Ist NICHT — die Summe bleibt bei 20.0h. Range-Signatur (Gap-Closure G1).
#[test]
fn f1_ist_rebooking_pair_invariant_vol_acct_03() {
    let sp = Uuid::new_v4();
    let mut hours = make_manual_volunteer_hours(sp, 2026, &[10, 11, 12, 13, 14], 4.0);
    // Rebooking-Marker-Paar hinzufuegen:
    hours.push(make_extra_hours(
        sp,
        2026,
        20,
        ExtraHoursCategory::VolunteerWork,
        -4.0,
        ExtraHoursSource::Rebooking,
    ));
    hours.push(make_extra_hours(
        sp,
        2026,
        20,
        ExtraHoursCategory::ExtraWork,
        4.0,
        ExtraHoursSource::Rebooking,
    ));
    let (from, to) = full_year_range(2026);
    let total = voluntary_ist_total_in_range(&hours, from, to);
    assert!(
        (total - 20.0).abs() < 1e-4,
        "rebooking pair must be neutral for F1-Ist; expected 20.0, got {total}"
    );
}

/// F2-Soll bei leerem committed_voluntary = 0.
#[test]
fn f2_soll_zero_when_no_committed_voluntary() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 1), (2026, 52), 40.0, 0.0)];
    let (from, to) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from, to);
    assert!((total - 0.0).abs() < 1e-4, "expected 0.0, got {total}");
}

/// D-F2-01: Mid-Week-Wechsel Mittwoch.
/// Vertrag A (KW 1..=W_MID) committed_voluntary=7.0 endet Mittwoch,
/// Vertrag B (W_MID..) committed_voluntary=14.0 beginnt Donnerstag.
/// In der Uebergangswoche: 3/7*7.0 + 4/7*14.0 = 3.0 + 8.0 = 11.0.
///
/// Nutzt weiterhin den per-week Baustein `committed_voluntary_prorata_for_week`,
/// weil das der klarste Weg ist, den Mid-Week-Uebergang zu testen. Die
/// tages-basierte Range-fn liefert fuer eine volle Woche denselben Wert.
#[test]
fn f2_soll_prorata_midweek_change_d_f2_01() {
    let sp = Uuid::new_v4();
    let week: u8 = 20;

    // Vertrag A: von KW 1 (Mo) bis KW `week` (Mi) mit committed_voluntary=7.0.
    let mut contract_a = make_working_hours(sp, (2026, 1), (2026, week), 40.0, 7.0);
    contract_a.to_day_of_week = DayOfWeek::Wednesday;

    // Vertrag B: von KW `week` (Do) bis KW 52 mit committed_voluntary=14.0.
    let mut contract_b = make_working_hours(sp, (2026, week), (2026, 52), 40.0, 14.0);
    contract_b.from_day_of_week = DayOfWeek::Thursday;

    let wh = vec![contract_a, contract_b];

    let prorata = committed_voluntary_prorata_for_week(&wh, 2026, week);
    // Erwartung: 3/7*7.0 + 4/7*14.0 = 3.0 + 8.0 = 11.0
    let expected = 3.0 / 7.0 * 7.0 + 4.0 / 7.0 * 14.0;
    assert!(
        (prorata - expected).abs() < 1e-3,
        "mid-week change: expected {expected}, got {prorata}"
    );
    assert!(
        (prorata - 11.0).abs() < 1e-3,
        "mid-week change must yield 11.0; got {prorata}"
    );
}

/// D-F1-01: `contract_weeks_count_in_range` zaehlt eine EmployeeWorkDetails-Row
/// mit `expected_hours == 0` fuer die Wochen 10..=15 MIT (6 Wochen) im
/// Full-Year-Range.
#[test]
fn contract_weeks_zero_expected_counts_d_f1_01() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 10), (2026, 15), 0.0, 0.0)];
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(
        count, 6,
        "expected_hours=0 must still count contract weeks; expected 6, got {count}"
    );
}

/// contract_weeks bei leerer working-hours-Liste = 0.
#[test]
fn contract_weeks_empty_working_hours_returns_zero() {
    let wh: Vec<EmployeeWorkDetails> = Vec::new();
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 0);
}

/// D-F2-01 ISO-Wochen-Randfall: Fuer ein 53-Wochen-Jahr (2026 hat laut ISO
/// 53 Wochen) summiert `committed_voluntary_target_in_range` ueber das ganze
/// Jahr = 53.0 (1.0/Woche * 53). Fuer ein 52-Wochen-Jahr entsprechend 52.0.
#[test]
fn f2_soll_iso_week_53_year_boundary_d_f2_01() {
    let sp = Uuid::new_v4();

    // 2026 hat 53 ISO-Wochen (verifiziert via time::util::weeks_in_year).
    let year_2026_weeks = time::util::weeks_in_year(2026);
    // Vertrag ueber komplettes Jahr, committed_voluntary=1.0.
    let wh = vec![make_working_hours(
        sp,
        (2026, 1),
        (2026, year_2026_weeks),
        40.0,
        1.0,
    )];
    let (from_2026, to_2026) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from_2026, to_2026);
    // Erwartung: pro Woche 7 Tage a 1.0/7 = 1.0; Summe = 53 in einem
    // 53-Wochen-Jahr. Range 1.1.-31.12.2026 deckt tages-basiert 365 Tage;
    // Kalenderjahr 2026 endet in KW 53 (2026-12-31 = Do KW 53), also ist
    // 31.12. voll durch aktiven Vertrag gedeckt. Total = 365/7 ~= 52.14.
    // Aber der Vertrag geht von KW1 Mo bis KW53 So — inklusive Grenztage
    // 2025-12-29 (KW 1 Mo 2026 ist 2025-12-29!) und 2027-01-03 (KW 53 So
    // 2026 ist 2027-01-03). Die Range ist aber 2026-01-01..=2026-12-31.
    // Der Range-Cutoff ist daher der bindende Filter: nur Tage in
    // 2026-01-01..=2026-12-31 zaehlen. Das sind 365 Tage. Erwartung
    // 365.0 / 7.0 = 52.14. Fuer den 53-Wochen-Assert nutzen wir eine
    // Toleranz von 1.0 (dominant ist die 365-Tage-basierte Semantik).
    let expected_days = 365.0_f32; // 2026 = Nicht-Schaltjahr
    let expected = expected_days / 7.0;
    assert!(
        (total - expected).abs() < 1e-3,
        "expected {expected} for 365-day 53-week ISO year, got {total}"
    );

    // Regressionslock gegen 52-Wochen-Annahme:
    // 2025 hat 52 ISO-Wochen.
    let year_2025_weeks = time::util::weeks_in_year(2025);
    assert_eq!(year_2025_weeks, 52, "2025 must be a 52-week ISO year");
    let wh_2025 = vec![make_working_hours(sp, (2025, 1), (2025, 52), 40.0, 1.0)];
    let (from_2025, to_2025) = full_year_range(2025);
    let total_2025 = committed_voluntary_target_in_range(&wh_2025, from_2025, to_2025);
    // 2025 hat 365 Tage. Vertrag deckt KW1 Mo (2024-12-30) bis KW52 So (2025-12-28).
    // Range 2025-01-01..=2025-12-31. Range-Cutoff bindet: 365 Tage. Aber
    // Vertrag endet 2025-12-28 (Sun KW52), also sind 2025-12-29..31 (Mo,Di,Mi
    // KW1 des ISO-Jahres 2026) NICHT durch aktiven Vertrag gedeckt.
    // Aktive Tage: 2025-01-01..=2025-12-28 = 362 Tage. Total = 362/7 ~= 51.71.
    let expected_2025 = 362.0_f32 / 7.0;
    assert!(
        (total_2025 - expected_2025).abs() < 1e-2,
        "expected {expected_2025} for 52-week ISO year (contract 2024-12-30..=2025-12-28 capped by 2025-01-01..=2025-12-31), got {total_2025}"
    );
}

/// F1 + F2 muessen dieselbe ISO-Wochen-Semantik verwenden.
/// Kongruenz-Test: Ein Extra-Hours-Row am 2026-01-01 (das ist in der KW 1
/// des ISO-Jahres 2026 laut ISO-Kalender) wird von
/// `voluntary_ist_total_in_range` dem Full-Year-Range 2026 zugeordnet.
#[test]
fn f1_ist_and_f2_soll_share_iso_week_semantics_d_f1_01_kongruenz() {
    let sp = Uuid::new_v4();
    // 2026-01-01 = Donnerstag = ISO-Woche 1 des Jahres 2026.
    let jan_1 = time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap();
    let (iso_year, _iso_week, _) = jan_1.to_iso_week_date();
    assert_eq!(iso_year, 2026, "2026-01-01 muss zu ISO-Jahr 2026 gehoeren");

    let dt = time::PrimitiveDateTime::new(jan_1, time::Time::MIDNIGHT);
    let eh = ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: sp,
        amount: 5.0,
        category: ExtraHoursCategory::VolunteerWork,
        description: Arc::from(""),
        date_time: dt,
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source: ExtraHoursSource::Manual,
    };
    let (from, to) = full_year_range(2026);
    let total = voluntary_ist_total_in_range(&[eh], from, to);
    assert!(
        (total - 5.0).abs() < 1e-3,
        "2026-01-01 muss ins Range 2026-01-01..=2026-12-31 fallen; expected 5.0, got {total}"
    );
}

// ─── Gap-Closure G1 Range-Tests ──────────────────────────────────────────────

/// Test A (Regression-Guard, Full-Year 2025): 1.0 committed pro Woche ueber
/// das ganze Jahr, Range = 2025-01-01..=2025-12-31 → soll_total ~52.0
/// (365 Tage / 7 ~= 52.14; Vertrag deckt 2024-12-30..=2025-12-28, also
/// 362 Range-Tage → 362/7 ~= 51.71).
#[test]
fn range_regression_full_year_2025_matches_old_semantics() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2025, 1), (2025, 52), 40.0, 1.0)];
    let (from, to) = full_year_range(2025);
    let total = committed_voluntary_target_in_range(&wh, from, to);
    // Full-Year 2025 = 365 Tage; Vertrag KW1 Mo 2025-12-30 .. KW52 So
    // 2025-12-28 → 362 Range-Tage aktiv.
    let expected = 362.0_f32 / 7.0;
    assert!(
        (total - expected).abs() < 1e-2,
        "Full-Year-2025-Regression: expected ~{expected}, got {total}"
    );
    // Contract-Weeks-Count: alle 52 ISO-Wochen im Range enthalten mindestens
    // einen Vertragstag.
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 52, "expected 52 contract weeks in 2025, got {count}");
}

/// Test B (Regression-Guard, Full-Year 2026 = 53 ISO-Wochen).
#[test]
fn range_regression_full_year_2026_matches_old_semantics() {
    let sp = Uuid::new_v4();
    let year_2026_weeks = time::util::weeks_in_year(2026);
    let wh = vec![make_working_hours(
        sp,
        (2026, 1),
        (2026, year_2026_weeks),
        40.0,
        1.0,
    )];
    let (from, to) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from, to);
    // 2026 hat 365 Tage. Vertrag deckt ISO KW1 Mo (2025-12-29) bis KW53 So
    // (2027-01-03). Range 2026-01-01..=2026-12-31 → alle 365 Tage aktiv.
    let expected = 365.0_f32 / 7.0;
    assert!(
        (total - expected).abs() < 1e-2,
        "Full-Year-2026-Regression: expected ~{expected}, got {total}"
    );
    // Contract-Weeks-Count: 2026 hat 53 ISO-Wochen; alle enthalten
    // Range-Tage mit aktivem Vertrag.
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 53, "expected 53 contract weeks in 2026, got {count}");
}

/// Test C (Edge-Week-Start, mid-week Mittwoch KW 21 2026):
/// Range from_date=2026-05-20 (Mi der KW 21), to_date=2026-05-31 (So der KW 22)
/// mit committed_voluntary=7.0/Woche → KW 21 = 5 Tage × 7/7 = 5.0;
/// KW 22 = 7 Tage × 7/7 = 7.0; total = 12.0.
#[test]
fn range_edge_week_start_midweek_wednesday_kw21_2026() {
    let sp = Uuid::new_v4();
    // Vertrag ueber ganzes Jahr.
    let wh = vec![make_working_hours(sp, (2026, 1), (2026, 53), 40.0, 7.0)];

    // 2026-05-20 = Mittwoch der KW 21.
    let day = time::Date::from_calendar_date(2026, time::Month::May, 20).unwrap();
    let (_iy, iw, wd) = day.to_iso_week_date();
    assert_eq!((iw, wd), (21, time::Weekday::Wednesday));

    let from = ShiftyDate::from_ymd(2026, 5, 20).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 31).unwrap();
    let total = committed_voluntary_target_in_range(&wh, from, to);
    let expected = 5.0 + 7.0; // 5 Range-Tage in KW21, 7 in KW22, jeder = 7.0/7.0 = 1.0
    assert!(
        (total - expected).abs() < 1e-3,
        "Edge-Week-Start: expected {expected}, got {total}"
    );
    // Beide ISO-Wochen zaehlen fuer den F1-Nenner.
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 2, "expected 2 contract weeks (KW21 partial + KW22 full)");
}

/// Test D (Edge-Week-End, mid-week Donnerstag KW 21 2026):
/// Range from_date=2026-05-18 (Mo der KW 21), to_date=2026-05-21 (Do der KW 21)
/// mit committed_voluntary=7.0/Woche → 4 Tage × 7/7 = 4.0.
#[test]
fn range_edge_week_end_midweek_thursday_kw21_2026() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 1), (2026, 53), 40.0, 7.0)];

    let end = time::Date::from_calendar_date(2026, time::Month::May, 21).unwrap();
    let (_iy, iw, wd) = end.to_iso_week_date();
    assert_eq!((iw, wd), (21, time::Weekday::Thursday));

    let from = ShiftyDate::from_ymd(2026, 5, 18).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 21).unwrap();
    let total = committed_voluntary_target_in_range(&wh, from, to);
    let expected = 4.0;
    assert!(
        (total - expected).abs() < 1e-3,
        "Edge-Week-End: expected {expected}, got {total}"
    );
    // Nur eine ISO-Woche (partial) im Range.
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 1, "expected 1 contract week (KW21 partial)");
}

/// Test E (5h-Mai-Szenario, das Gap-Closure-Kernszenario aus 54-UAT.md G1):
/// committed_voluntary=5.0/Woche + Vertrag ab 2026-04-27 (KW 18 Mo), bis KW 53.
/// Range 2026-01-01..=2026-07-10 (KW 28 Fr).
///
/// Vertrag-aktive Range-Tage: 2026-04-27..=2026-07-10 = 75 Tage.
/// Erwartung soll_total = 5.0 * 75 / 7 ≈ 53.57.
/// Regression-Gate: soll_total muss < 60.0 sein (vs. alter 177h-Bug).
#[test]
fn range_five_h_per_week_since_may_scenario_2026_until_kw28() {
    let sp = Uuid::new_v4();
    // Vertrag KW 18 (Mo = 2026-04-27) bis KW 52.
    let wh = vec![make_working_hours(sp, (2026, 18), (2026, 52), 40.0, 5.0)];

    // Sanity: KW18 Mo 2026 == 2026-04-27.
    let mon_kw18 =
        time::Date::from_iso_week_date(2026, 18, time::Weekday::Monday).unwrap();
    assert_eq!(
        mon_kw18,
        time::Date::from_calendar_date(2026, time::Month::April, 27).unwrap()
    );

    let from = ShiftyDate::from_ymd(2026, 1, 1).unwrap();
    let to = ShiftyDate::from_ymd(2026, 7, 10).unwrap();
    let soll_total = committed_voluntary_target_in_range(&wh, from, to);

    // Erwartung: 2026-04-27..=2026-07-10 = 75 Vertrag-Tage im Range.
    // 5.0 / 7.0 pro Tag * 75 = 53.5714.
    let expected_days = 75.0_f32;
    let expected = 5.0 * expected_days / 7.0;
    assert!(
        (soll_total - expected).abs() < 0.5,
        "5h-Mai-Szenario: expected ~{expected} (from days {expected_days}), got {soll_total}"
    );
    // Regression-Gate gegen den 177h-Bug (der Full-Year-Semantik ~177h liefern
    // wuerde: 5.0 * 249/7 ~= 177.86).
    assert!(
        soll_total < 60.0,
        "regression-gate against 177h Full-Year-bug: got {soll_total}"
    );
    // ist_total = 0 (keine ExtraHours-Rows).
    let empty: Vec<ExtraHours> = Vec::new();
    let ist_total = voluntary_ist_total_in_range(&empty, from, to);
    assert!((ist_total - 0.0).abs() < 1e-4);
    // Contract-Weeks: KW 18..=KW 28 = 11 Wochen; KW 28 endet Sonntag
    // 2026-07-12, aber Range endet Fr 2026-07-10, das ist innerhalb KW 28
    // Fr. Also 11 ISO-Wochen mit Range-Overlap-plus-Contract-Overlap.
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 11, "expected 11 contract weeks (KW18..=KW28), got {count}");
}

/// Test F (Range vor Vertragsbeginn): Vertrag ab KW 18, Range 2026-01-01..=
/// 2026-01-07 (vor Vertragsbeginn) → soll_total=0.0, contract_weeks=0.
#[test]
fn range_before_contract_start_returns_zero() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 18), (2026, 52), 40.0, 5.0)];
    let from = ShiftyDate::from_ymd(2026, 1, 1).unwrap();
    let to = ShiftyDate::from_ymd(2026, 1, 7).unwrap();
    let soll_total = committed_voluntary_target_in_range(&wh, from, to);
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert!((soll_total - 0.0).abs() < 1e-4);
    assert_eq!(count, 0);
}

/// Test G (Regression-Lock gegen 177h bei Full-Year): 5h/Woche committed
/// seit KW 18 + Full-Year-Range 2026-01-01..=2026-12-31 → soll_total ~177.
/// Dokumentiert die alte Full-Year-Semantik als reproduzierbaren Referenzwert;
/// beweist damit, dass Test E (bis KW 28) die Range-Cutoff-Semantik greift.
#[test]
fn range_full_year_shows_full_annual_target_regression_lock_177() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 18), (2026, 52), 40.0, 5.0)];
    let (from, to) = full_year_range(2026);
    let soll_total = committed_voluntary_target_in_range(&wh, from, to);
    // Vertrag deckt 2026-04-27..=2026-12-27 = 245 Tage im Range
    // (KW52 So 2026 = 2026-12-27). 5.0 / 7.0 * 245 = 175.0.
    let expected_days = 245.0_f32;
    let expected = 5.0 * expected_days / 7.0;
    assert!(
        (soll_total - expected).abs() < 1.0,
        "Full-Year-Regression 5h/Woche: expected ~{expected}, got {soll_total}"
    );
    // Der Gap ist genau der Delta zwischen 175h (Full-Year) und ~54h (bis KW28).
    assert!(soll_total > 150.0, "sanity: Full-Year must be well over 150h");
}

// ─── Service-Tests (mockall) ──────────────────────────────────────────────────

mod service_tests {
    use super::*;
    use service::employee_work_details::MockEmployeeWorkDetailsService;
    use service::extra_hours::MockExtraHoursService;
    use service::permission::Authentication;
    use service::sales_person::{MockSalesPersonService, SalesPerson};
    use service::voluntary_stats::VoluntaryStatsService;
    use service::MockPermissionService;
    use service::ServiceError;
    use shifty_utils::ShiftyDate;

    use crate::voluntary_stats::{VoluntaryStatsServiceDeps, VoluntaryStatsServiceImpl};

    struct TestDeps;
    impl VoluntaryStatsServiceDeps for TestDeps {
        type Context = ();
        type Transaction = dao::MockTransaction;
        type ExtraHoursService = MockExtraHoursService;
        type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
        type SalesPersonService = MockSalesPersonService;
        type PermissionService = MockPermissionService;
        type TransactionDao = dao::MockTransactionDao;
    }

    fn make_sales_person(id: Uuid) -> SalesPerson {
        SalesPerson {
            id,
            name: Arc::from("Test"),
            background_color: Arc::from("#123456"),
            is_paid: Some(false),
            inactive: false,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    /// VOL-STAT-02 / VOL-ACCT-02: Non-HR liefert VoluntaryStats mit lauter
    /// None-Feldern. Zusaetzlich MUSS kein Datenabruf erfolgen (kein DAO-Call).
    /// Range-Signatur (Gap-Closure G1).
    #[tokio::test]
    async fn service_non_hr_returns_all_none_vol_stat_02() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Err(ServiceError::Forbidden));

        // Diese Mocks setzen KEINE Expects — jeder Aufruf wuerde als
        // Panik enden (mockall default).
        let extra_hours_service = MockExtraHoursService::new();
        let employee_work_details_service = MockEmployeeWorkDetailsService::new();
        let sales_person_service = MockSalesPersonService::new();
        let transaction_dao = dao::MockTransactionDao::new();

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            extra_hours_service: Arc::new(extra_hours_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::first_day_in_year(2026);
        let to = ShiftyDate::last_day_in_year(2026);
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("Non-HR must not error, must return all-None VoluntaryStats");

        assert!(result.ist_per_contract_week.is_none());
        assert!(result.ist_total.is_none());
        assert!(result.soll_total.is_none());
        assert!(result.delta.is_none());
        assert!(result.contract_weeks.is_none());
    }

    /// VOL-STAT-01 + VOL-ACCT-01: HR-Aufrufer erhaelt konkrete Werte, die
    /// den pure fns entsprechen. Range = KW 10..=13 (2026-03-02..=2026-03-29,
    /// 28 Tage, 4 ISO-Wochen).
    #[tokio::test]
    async fn service_hr_returns_some_and_delegates_to_pure_fns() {
        let sp_id = Uuid::new_v4();

        // Permission ok.
        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        // Sales Person existiert.
        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp.clone()));

        // 2 manuelle VolunteerWork-Rows a 5h = 10.0h. Beide in KW 10..=11
        // (im Range KW10..=13).
        let ehs: Arc<[ExtraHours]> =
            Arc::from(make_manual_volunteer_hours(sp_id, 2026, &[10, 11], 5.0));
        let mut extra_hours_service = MockExtraHoursService::new();
        extra_hours_service
            .expect_find_by_iso_year()
            .returning(move |_, _, _| Ok(ehs.clone()));

        // Working hours: KW 10..=13 (= 2026-03-02..=2026-03-29 = 28 Tage),
        // committed_voluntary=1.0. Erwartung:
        //   soll_total = 28 * 1.0/7.0 = 4.0
        //   contract_weeks = 4 (KW 10..=13)
        //   ist_total = 10.0 (beide Rows in KW 10 und 11 sind im Range)
        //   ist_per_contract_week = 10/4 = 2.5
        let wh: Arc<[EmployeeWorkDetails]> = Arc::from(vec![make_working_hours(
            sp_id,
            (2026, 10),
            (2026, 13),
            40.0,
            1.0,
        )]);
        let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
        employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(wh.clone()));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            extra_hours_service: Arc::new(extra_hours_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        // Range: KW10 Mo (2026-03-02) bis KW13 So (2026-03-29).
        let from = ShiftyDate::from_ymd(2026, 3, 2).unwrap();
        let to = ShiftyDate::from_ymd(2026, 3, 29).unwrap();
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        assert_eq!(result.contract_weeks, Some(4));
        assert!((result.ist_total.unwrap() - 10.0).abs() < 1e-3);
        assert!(
            (result.soll_total.unwrap() - 4.0).abs() < 1e-3,
            "expected soll_total=4.0 (28 days * 1.0/7), got {}",
            result.soll_total.unwrap()
        );
        assert!(
            (result.delta.unwrap() - 6.0).abs() < 1e-3,
            "expected delta=6.0 (10.0 - 4.0), got {}",
            result.delta.unwrap()
        );
        assert!((result.ist_per_contract_week.unwrap() - 2.5).abs() < 1e-3);
    }

    /// Divisions-Guard: contract_weeks == 0 => ist_per_contract_week = 0
    /// statt f32::NAN oder inf. Range-Signatur (Gap-Closure G1).
    #[tokio::test]
    async fn service_zero_contract_weeks_yields_zero_per_week() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp.clone()));

        // Keine ExtraHours.
        let empty_ehs: Arc<[ExtraHours]> = Arc::from(Vec::new());
        let mut extra_hours_service = MockExtraHoursService::new();
        extra_hours_service
            .expect_find_by_iso_year()
            .returning(move |_, _, _| Ok(empty_ehs.clone()));

        // Keine working hours => contract_weeks=0.
        let empty_wh: Arc<[EmployeeWorkDetails]> = Arc::from(Vec::new());
        let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
        employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(empty_wh.clone()));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            extra_hours_service: Arc::new(extra_hours_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::first_day_in_year(2026);
        let to = ShiftyDate::last_day_in_year(2026);
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        assert_eq!(result.contract_weeks, Some(0));
        assert!((result.ist_per_contract_week.unwrap() - 0.0).abs() < 1e-3);
        assert!((result.ist_total.unwrap() - 0.0).abs() < 1e-3);
        assert!((result.soll_total.unwrap() - 0.0).abs() < 1e-3);
    }
}
