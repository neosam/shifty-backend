//! Phase 54 Plan 03 + Gap-Closure G1 (Plan 54-07) + Gap-Closure 54-09-Ist-Fix —
//! Pure-fn + Service-Tests fuer VoluntaryStatsService (VOL-STAT-01/02,
//! VOL-ACCT-01/02).
//!
//! Die Tests decken:
//!
//! - D-F1-01: `contract_weeks_count_in_range` zaehlt jede ISO-Woche mit
//!   gueltiger `EmployeeWorkDetails`-Row als Vertragswoche — auch wenn
//!   `expected_hours == 0`.
//! - D-F2-01: `committed_voluntary_target_in_range` verteilt
//!   `committed_voluntary` tages-basiert (jeder Range-Tag mit aktivem
//!   Vertrag = committed_voluntary / 7.0).
//! - Gap-Closure G1: Range-Cutoff — 5h/Woche-seit-Mai + Range bis KW 28
//!   liefert ~54h statt der alten Full-Year-Semantik von ~177h.
//! - Regression: Full-Year-Range (1.1.–31.12.) reproduziert die alte
//!   Full-Year-Semantik byte-genau (52.0 fuer 52-Wochen-Jahr, 53.0 fuer
//!   53-Wochen-Jahr).
//! - Edge-Weeks: mid-week-Start / mid-week-Ende zaehlen tages-genau.
//! - Service-Tests: HR-Gate mit Non-HR-Redaktion (VOL-STAT-02, VOL-ACCT-02);
//!   HR-Delegation an `ReportingService::get_report_for_employee_range` fuer
//!   das Ist-Aggregat (Gap-Closure 54-09-Ist-Fix).

use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod, DayFraction};
use service::employee_work_details::EmployeeWorkDetails;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{
    committed_voluntary_prorata_for_week, committed_voluntary_target_in_range,
    contract_weeks_count_in_range,
};

/// Phase 54.5: Helper zum Bauen einer aktiven `AbsencePeriod` fuer die
/// whole-week-out-Tests. `deleted = None`, `day_fraction = Full`.
fn make_absence(
    sp_id: Uuid,
    from: time::Date,
    to: time::Date,
    category: AbsenceCategory,
) -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::new_v4(),
        sales_person_id: sp_id,
        category,
        from_date: from,
        to_date: to,
        description: Arc::from("test"),
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    }
}

// ── Fixture helpers ──────────────────────────────────────────────────────────

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

// ─── Pure-fn Tests (F2-Soll + contract_weeks) ────────────────────────────────

/// D-F2-01: `committed_voluntary_target_in_range` gibt 0 zurueck wenn keine
/// `EmployeeWorkDetails` Rows vorhanden.
#[test]
fn f2_soll_zero_when_no_committed_voluntary() {
    let wh: Vec<EmployeeWorkDetails> = Vec::new();
    let (from, to) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    assert!((total - 0.0).abs() < 1e-3, "expected 0.0, got {total}");
}

/// D-F2-01: `committed_voluntary_prorata_for_week` liefert bei Mid-Week-
/// Vertragswechsel != latest-active-Naeherung.
#[test]
fn f2_soll_prorata_midweek_change_d_f2_01() {
    let sp_id = Uuid::new_v4();
    // Vertrag A: KW 10 Mo..=Di (2026-03-02..=2026-03-03), committed=1.0
    let wh_a = EmployeeWorkDetails {
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 10,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Tuesday,
        to_calendar_week: 10,
        to_year: 2026,
        committed_voluntary: 1.0,
        ..make_working_hours(sp_id, (2026, 10), (2026, 10), 40.0, 1.0)
    };
    // Vertrag B: KW 10 Mi..=So, committed=2.0
    let wh_b = EmployeeWorkDetails {
        from_day_of_week: DayOfWeek::Wednesday,
        from_calendar_week: 10,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 10,
        to_year: 2026,
        committed_voluntary: 2.0,
        ..make_working_hours(sp_id, (2026, 10), (2026, 10), 40.0, 2.0)
    };
    let wh = vec![wh_a, wh_b];
    let prorata = committed_voluntary_prorata_for_week(&wh, 2026, 10);
    // Erwartet: 2/7 * 1.0 + 5/7 * 2.0 = 12/7 ≈ 1.714
    let expected = 12.0 / 7.0;
    assert!(
        (prorata - expected).abs() < 1e-3,
        "expected ~{expected:.3}, got {prorata:.3}",
    );
}

/// D-F1-01: `contract_weeks_count_in_range` zaehlt eine EmployeeWorkDetails-Row
/// mit `expected_hours == 0` MIT.
#[test]
fn contract_weeks_zero_expected_counts_d_f1_01() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 10), (2026, 15), 0.0, 5.0)];
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    // Weeks 10..=15 = 6 weeks.
    assert_eq!(count, 6, "expected 6, got {count}");
}

/// D-F1-01: leere `EmployeeWorkDetails` liefert count = 0.
#[test]
fn contract_weeks_empty_working_hours_returns_zero() {
    let wh: Vec<EmployeeWorkDetails> = Vec::new();
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 0, "expected 0, got {count}");
}

/// D-F2-01 Full-Year-Range: `committed_voluntary_target_in_range` summiert
/// tages-basiert (committed_voluntary/7 pro aktivem Tag). Fuer 2026
/// (365 Kalendertage) mit committed_voluntary=1.0 -> 365/7 ≈ 52.143. Fuer
/// 2025 -> ebenfalls 365/7 ≈ 52.143. Der Unterschied 52-vs-53-Wochen-Jahr
/// zeigt sich in `contract_weeks_count_in_range` (Zaehler), nicht im
/// tages-basierten Zaehler.
#[test]
fn f2_soll_iso_week_53_year_boundary_d_f2_01() {
    let sp_id = Uuid::new_v4();
    // Vertrag ganzes 2026: KW 1..=53, committed=1.0 / week.
    let wh = vec![make_working_hours(sp_id, (2026, 1), (2026, 53), 40.0, 1.0)];
    let (from_2026, to_2026) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from_2026, to_2026, &[]);
    let expected_2026 = 365.0 / 7.0;
    assert!(
        (total - expected_2026).abs() < 0.01,
        "expected ~{expected_2026:.3} for full-year 2026 (365 days / 7), got {total}"
    );

    let wh_2025 = vec![make_working_hours(sp_id, (2025, 1), (2025, 52), 40.0, 1.0)];
    let (from_2025, to_2025) = full_year_range(2025);
    let total_2025 = committed_voluntary_target_in_range(&wh_2025, from_2025, to_2025, &[]);
    // Vertrag KW1-Mo 2025 = 2024-12-30, KW52-So 2025 = 2025-12-28. Range
    // = 2025-01-01..=2025-12-31. Overlap = 2025-01-01..=2025-12-28 = 362 Tage.
    let expected_2025 = 362.0 / 7.0;
    assert!(
        (total_2025 - expected_2025).abs() < 0.01,
        "expected ~{expected_2025:.3} for full-year 2025 (362 overlap-days / 7), got {total_2025}"
    );
}

/// Gap-Closure G1 Regression: Full-Year-Range 2025 = 52 Wochen im Nenner
/// (contract_weeks); tages-basierter Zaehler betrachtet Overlap
/// Range ∩ Vertrag = 2025-01-01..=2025-12-28 = 362 Tage.
#[test]
fn range_regression_full_year_2025_matches_old_semantics() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2025, 1), (2025, 52), 40.0, 1.0)];
    let (from, to) = full_year_range(2025);
    let total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    let expected = 362.0 / 7.0;
    assert!(
        (total - expected).abs() < 0.01,
        "expected ~{expected:.3} (362 overlap-days / 7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 52, "expected 52 contract weeks, got {count}");
}

/// Gap-Closure G1 Regression: Full-Year-Range 2026 = 53 Wochen im Nenner
/// (contract_weeks), tages-basierter Zaehler = 365/7 ≈ 52.143.
#[test]
fn range_regression_full_year_2026_matches_old_semantics() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 1), (2026, 53), 40.0, 1.0)];
    let (from, to) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    let expected = 365.0 / 7.0;
    assert!(
        (total - expected).abs() < 0.01,
        "expected ~{expected:.3} (365/7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 53, "expected 53 contract weeks, got {count}");
}

/// Gap-Closure G1 Edge-Week-Start: Range startet Mittwoch KW 21 = 5 Range-Tage
/// dieser Woche → 5/7 der Wochen-Zusage.
#[test]
fn range_edge_week_start_midweek_wednesday_kw21_2026() {
    let sp_id = Uuid::new_v4();
    // Vertrag ganze KW 21+22, committed_voluntary=7.0/Woche.
    let wh = vec![make_working_hours(sp_id, (2026, 21), (2026, 22), 40.0, 7.0)];
    // Range: Mi KW 21 (2026-05-20) bis So KW 22 (2026-05-31).
    let from = ShiftyDate::from_ymd(2026, 5, 20).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 31).unwrap();
    let total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    // KW 21: 5 Tage (Mi..=So) * 7/7 = 5.0
    // KW 22: 7 Tage * 7/7 = 7.0
    // Summe: 12.0
    assert!(
        (total - 12.0).abs() < 1e-3,
        "expected 12.0 (5+7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 2, "expected 2 contract weeks (KW 21+22), got {count}");
}

/// Gap-Closure G1 Edge-Week-End: Range endet Donnerstag KW 21 = 4 Range-Tage.
#[test]
fn range_edge_week_end_midweek_thursday_kw21_2026() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 21), (2026, 21), 40.0, 7.0)];
    // Range: Mo KW 21 (2026-05-18) bis Do KW 21 (2026-05-21).
    let from = ShiftyDate::from_ymd(2026, 5, 18).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 21).unwrap();
    let total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    // 4 Tage * 7/7 = 4.0.
    assert!(
        (total - 4.0).abs() < 1e-3,
        "expected 4.0 (Mo..=Do 4 Tage * 1), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 1, "expected 1 contract week, got {count}");
}

/// Gap-Closure G1 5h-Mai-Szenario: Vertrag ab KW 18 (2026-04-27) mit
/// committed_voluntary=5.0. Range bis KW 28 So (2026-07-12). Erwartet:
/// 5.0 * (Tage von 2026-04-27 bis 2026-07-12) / 7 = 5.0 * 77 / 7 = 55.0.
/// Regression-Lock gegen den 177h-Bug.
#[test]
fn range_five_h_per_week_since_may_scenario_2026_until_kw28() {
    let sp_id = Uuid::new_v4();
    // Vertrag KW 18..=53/2026, committed=5.0/Woche.
    let wh = vec![make_working_hours(sp_id, (2026, 18), (2026, 53), 20.0, 5.0)];
    // Range: 2026-01-01 bis 2026-07-12 (Sonntag KW 28).
    let from = ShiftyDate::from_ymd(2026, 1, 1).unwrap();
    let to = ShiftyDate::from_ymd(2026, 7, 12).unwrap();
    let soll_total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    // Vertrag aktiv von 2026-04-27 bis 2026-07-12 = 77 Tage.
    // 5.0 * 77 / 7 = 55.0.
    let expected = 5.0 * 77.0 / 7.0;
    assert!(
        (soll_total - expected).abs() < 0.5,
        "expected ~{expected:.2}, got {soll_total}"
    );
    // Regression-Gate gegen 177h-Bug (alte Full-Year-Semantik).
    assert!(
        soll_total < 60.0,
        "regression-gate against 177h bug: got {soll_total}"
    );
}

/// Gap-Closure G1: Range vor Vertragsbeginn liefert 0.
#[test]
fn range_before_contract_start_returns_zero() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 18), (2026, 53), 20.0, 5.0)];
    let from = ShiftyDate::from_ymd(2026, 1, 1).unwrap();
    let to = ShiftyDate::from_ymd(2026, 1, 7).unwrap();
    let soll_total = committed_voluntary_target_in_range(&wh, from, to, &[]);
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert!(
        (soll_total - 0.0).abs() < 1e-3,
        "expected 0.0, got {soll_total}"
    );
    assert_eq!(count, 0, "expected 0 contract weeks, got {count}");
}

// ─── Phase 54.5 Absence-Aware Pure-fn Tests (D-54.5-01 / D-54.5-02) ──────────
//
// Ist/Soll-Symmetrie: `committed_voluntary_target_in_range` traegt fuer eine
// ISO-KW mit >= 1 Absence-Tag desselben SalesPerson **0** bei (whole-week-out
// analog VFA-01 / D-26-03). `contract_weeks_count_in_range` klammert dieselbe
// KW aus dem Nenner aus (D-54.5-02). Overlap-Test via `period_overlaps_week`
// (Single Source of Truth in booking_information.rs).
//
// Regression-Sicherheitsnetz: alle bestehenden Tests oben nutzen `&[]` als
// Absence-Argument und liefern byte-genau denselben Wert wie v2.6.0.

/// D-54.5-01 Golden-Regression-Test (Whole-Week-Out fuer Soll):
///
/// Fixture: SalesPerson mit Vertrag KW 1..=53/2026, `committed_voluntary=5.0`.
/// Range = 2026-01-01..=2026-06-30 (H1). Absence-Period KW 20..=22
/// (2026-05-11..=2026-05-31, 3 zusammenhaengende Wochen, Kategorie Vacation —
/// kategorie-agnostisch).
///
/// **v2.6.0 (falsch, im Kommentar dokumentiert):** die 3 Absence-Wochen (21
/// Tage) haetten weiter mit `5.0 * 21 / 7 = 15.0` zum Soll beigetragen.
///
/// **v2.6.1 (korrekt, assertion — D-26-03 / D-54.5-01):** whole-week-out,
/// die 3 Wochen tragen 0 bei. Symmetrie-Nachweis:
/// `soll_absence == soll_no_absence - 15.0`.
#[test]
fn f2_soll_absence_whole_week_out_d_54_5_01() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 1), (2026, 53), 40.0, 5.0)];
    let from = ShiftyDate::from_ymd(2026, 1, 1).unwrap();
    let to = ShiftyDate::from_ymd(2026, 6, 30).unwrap();
    // Absence 2026-05-11 (Mo KW 20) .. 2026-05-31 (So KW 22) — deckt genau
    // KW 20, 21, 22 komplett ab.
    let abs = make_absence(
        sp_id,
        date!(2026 - 05 - 11),
        date!(2026 - 05 - 31),
        AbsenceCategory::Vacation,
    );

    let soll_no_absence = committed_voluntary_target_in_range(&wh, from, to, &[]);
    let soll_absence =
        committed_voluntary_target_in_range(&wh, from, to, std::slice::from_ref(&abs));

    // v2.6.0-Falschwert waere `soll_no_absence` gewesen (Pfad B war absence-
    // blind); v2.6.1 zieht 3 Wochen a 5.0h = 15.0h ab.
    let expected_diff = 15.0_f32;
    assert!(
        (soll_no_absence - soll_absence - expected_diff).abs() < 1e-3,
        "expected whole-week-out to remove exactly 15.0h (3 weeks * 5.0), \
         got no_absence={soll_no_absence:.3}, absence={soll_absence:.3}"
    );

    // Contract-Weeks (D-54.5-02): 3 Absence-Wochen fallen aus dem Nenner.
    let cw_no_absence = contract_weeks_count_in_range(&wh, from, to, &[]);
    let cw_absence = contract_weeks_count_in_range(&wh, from, to, &[abs]);
    assert_eq!(
        cw_absence,
        cw_no_absence - 3,
        "expected 3 fewer contract weeks (KW 20/21/22 aus D-54.5-02), \
         got no_absence={cw_no_absence}, absence={cw_absence}"
    );
}

/// D-54.5-01: Partial-Absence-Woche — nur 1 Absence-Tag mitten in der Woche
/// (Dienstag KW 21, 2026-05-19) reicht fuer whole-week-out. Die ganze KW 21
/// traegt 0 zum Soll bei UND zaehlt nicht als Vertragswoche.
#[test]
fn f2_soll_partial_absence_week_still_whole_week_out() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 21), (2026, 21), 40.0, 7.0)];
    // Range = ganze KW 21 (2026-05-18 Mo .. 2026-05-24 So).
    let from = ShiftyDate::from_ymd(2026, 5, 18).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 24).unwrap();
    // Nur 1 Absence-Tag: Di 2026-05-19.
    let abs = make_absence(
        sp_id,
        date!(2026 - 05 - 19),
        date!(2026 - 05 - 19),
        AbsenceCategory::SickLeave,
    );

    let soll = committed_voluntary_target_in_range(&wh, from, to, std::slice::from_ref(&abs));
    let cw = contract_weeks_count_in_range(&wh, from, to, &[abs]);
    assert!(
        (soll - 0.0).abs() < 1e-3,
        "expected 0.0 (whole KW 21 out, 1 absence day suffices), got {soll}"
    );
    assert_eq!(cw, 0, "expected 0 contract weeks (KW 21 ausgeklammert), got {cw}");
}

/// D-54.5-02 (Nicht-Doppel-Exklusion): Range beruehrt eine Woche komplett vor
/// Vertragsbeginn (kein Contract, keine Absence) UND eine Woche mit Absence.
/// Beide werden nicht gezaehlt, aber aus unabhaengigen Gruenden — die
/// Absence-Overlay verursacht keine doppelte Reduktion.
#[test]
fn contract_weeks_absence_and_no_contract_do_not_double_exclude() {
    let sp_id = Uuid::new_v4();
    // Vertrag KW 21..=22/2026 (2 Wochen). KW 20 = keine Contract-Woche.
    let wh = vec![make_working_hours(sp_id, (2026, 21), (2026, 22), 40.0, 5.0)];
    // Range = KW 20 Mo..=KW 22 So (3 Kalender-Wochen).
    let from = ShiftyDate::from_ymd(2026, 5, 11).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 31).unwrap();

    // Ohne Absence: KW 20 zaehlt nicht (kein Vertrag), KW 21+22 zaehlen = 2.
    let cw_no_abs = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(cw_no_abs, 2, "expected 2 contract weeks (KW 21+22), got {cw_no_abs}");

    // Mit Absence KW 22 (2026-05-25..=05-31): KW 22 faellt raus (D-54.5-02),
    // KW 20 sowieso (kein Contract), KW 21 bleibt = 1. Keine Doppel-Zaehlung.
    let abs = make_absence(
        sp_id,
        date!(2026 - 05 - 25),
        date!(2026 - 05 - 31),
        AbsenceCategory::UnpaidLeave,
    );
    let cw_abs = contract_weeks_count_in_range(&wh, from, to, &[abs]);
    assert_eq!(
        cw_abs, 1,
        "expected 1 contract week (KW 21) — KW 20 no-contract + KW 22 absence, got {cw_abs}"
    );
}

/// Regressions-Anker (D-54.5-04): 10 Vertragswochen ohne Absence liefern
/// `count == 10` mit `&[]` — die neue Absence-Aware-Signatur veraendert die
/// v2.6.0-Semantik nur, wenn tatsaechlich Absences vorhanden sind.
#[test]
fn contract_weeks_without_absence_matches_v260_semantics() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 10), (2026, 19), 40.0, 5.0)];
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to, &[]);
    assert_eq!(count, 10, "expected 10 contract weeks (KW 10..=19), got {count}");
}

/// Zusatz-Guard: geloeschte Absence (deleted != None) wird IGNORIERT. Der
/// Fix soll aktive Absences beruecksichtigen, tombstones aus dem physischen
/// Update-Modell muessen unsichtbar bleiben.
#[test]
fn f2_soll_deleted_absence_is_ignored() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 21), (2026, 21), 40.0, 7.0)];
    let from = ShiftyDate::from_ymd(2026, 5, 18).unwrap();
    let to = ShiftyDate::from_ymd(2026, 5, 24).unwrap();
    let mut abs = make_absence(
        sp_id,
        date!(2026 - 05 - 19),
        date!(2026 - 05 - 19),
        AbsenceCategory::Vacation,
    );
    // Tombstone: gelaeschte Row darf NICHT die Woche nullen.
    abs.deleted = Some(datetime!(2026 - 06 - 01 10:00:00));

    let soll = committed_voluntary_target_in_range(&wh, from, to, std::slice::from_ref(&abs));
    let cw = contract_weeks_count_in_range(&wh, from, to, &[abs]);
    // Erwartung: identisch zum &[]-Fall — 7 Tage * 7.0/7 = 7.0, 1 Contract-Woche.
    assert!(
        (soll - 7.0).abs() < 1e-3,
        "deleted absence must not zero the week, got {soll}"
    );
    assert_eq!(cw, 1, "deleted absence must not remove contract week, got {cw}");
}

// ─── Service-Tests (mockall) ──────────────────────────────────────────────────
//
// Gap-Closure 54-09-Ist-Fix: Ist-Aggregat kommt aus
// `ReportingService::get_report_for_employee_range` — konsistent zum OVERALL-
// "Ehrenamt"-Wert der UI. Der Service-Test verifiziert die Delegation an den
// ReportingService-Mock (kein Aufruf im Non-HR-Path).
//
// Phase 54.5: neue Service-Tests zeigen dass der VoluntaryStatsService die
// AbsenceService-Liste laedt und an beide pure fns weiterreicht (D-54.5-03).

mod service_tests {
    use super::*;
    use service::absence::MockAbsenceService;
    use service::employee_work_details::MockEmployeeWorkDetailsService;
    use service::permission::Authentication;
    use service::reporting::{EmployeeReport, MockReportingService};
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
        type ReportingService = MockReportingService;
        type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
        type SalesPersonService = MockSalesPersonService;
        // Phase 54.5 (D-54.5-03): AbsenceService-Dep in TestDeps.
        type AbsenceService = MockAbsenceService;
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

    fn make_report(sp: SalesPerson, volunteer_hours: f32) -> EmployeeReport {
        EmployeeReport {
            sales_person: Arc::new(sp),
            balance_hours: 0.0,
            overall_hours: 0.0,
            expected_hours: 0.0,
            dynamic_hours: 0.0,
            shiftplan_hours: 0.0,
            extra_work_hours: 0.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unpaid_leave_hours: 0.0,
            volunteer_hours,
            vacation_carryover: 0,
            vacation_days: 0.0,
            vacation_entitlement: 0.0,
            sick_leave_days: 0.0,
            holiday_days: 0.0,
            absence_days: 0.0,
            carryover_hours: 0.0,
            custom_extra_hours: Arc::from(Vec::new()),
            by_week: Arc::from(Vec::new()),
            by_month: Arc::from(Vec::new()),
        }
    }

    /// VOL-STAT-02 / VOL-ACCT-02: Non-HR liefert VoluntaryStats mit lauter
    /// None-Feldern. Zusaetzlich MUSS kein Datenabruf erfolgen (kein
    /// ReportingService-Call, kein DAO-Call).
    #[tokio::test]
    async fn service_non_hr_returns_all_none_vol_stat_02() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Err(ServiceError::Forbidden));

        // Diese Mocks setzen KEINE Expects — jeder Aufruf wuerde als Panik enden.
        // Phase 54.5 (D-54.5-03): AbsenceService MUSS im Non-HR-Path unangetastet
        // bleiben — kein `expect_find_by_sales_person` -> panicked-on-call.
        let reporting_service = MockReportingService::new();
        let employee_work_details_service = MockEmployeeWorkDetailsService::new();
        let sales_person_service = MockSalesPersonService::new();
        let absence_service = MockAbsenceService::new();
        let transaction_dao = dao::MockTransactionDao::new();

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
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
        assert!(result.ist_per_soll_pct.is_none());
    }

    /// VOL-STAT-01 + VOL-ACCT-01 (Gap-Closure 54-09-Ist-Fix): HR-Aufrufer
    /// bekommt konkrete Werte. `ist_total` wird 1:1 aus
    /// `report.volunteer_hours` uebernommen (deckt alle drei
    /// Ehrenamt-Quellen des OVERALL-Reports ab). Range = KW 10..=13
    /// (2026-03-02..=2026-03-29, 28 Tage, 4 ISO-Wochen).
    #[tokio::test]
    async fn service_hr_returns_some_and_delegates_ist_to_report() {
        let sp_id = Uuid::new_v4();

        // Permission ok.
        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        // Sales Person existiert.
        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        let sp_clone = sp.clone();
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp_clone.clone()));

        // ReportingService liefert Report mit volunteer_hours = 10.0.
        // Simuliert die 3 Quellen kombiniert (manual + auto + no_contract).
        let mut reporting_service = MockReportingService::new();
        let sp_for_report = sp.clone();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(move |_, _, _, _, _, _| Ok(make_report(sp_for_report.clone(), 10.0)));

        // Working hours: KW 10..=13 (= 2026-03-02..=2026-03-29 = 28 Tage),
        // committed_voluntary=1.0. Erwartung:
        //   soll_total = 28 * 1.0/7.0 = 4.0
        //   contract_weeks = 4 (KW 10..=13)
        //   ist_total = 10.0 (aus report.volunteer_hours)
        //   ist_per_contract_week = 10/4 = 2.5
        //   delta = 10 - 4 = 6.0
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

        // Phase 54.5 (D-54.5-03): AbsenceService liefert leere Liste ->
        // Regression-Sicherheitsnetz: gleicher Output wie v2.6.0.
        let mut absence_service = MockAbsenceService::new();
        absence_service
            .expect_find_by_sales_person()
            .returning(|_, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
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
        // Erfuellungsgrad: 10 / 4 * 100 = 250 % (Ist uebersteigt Soll,
        // typisch bei Freiwilligen, die mehr leisten als zugesagt).
        assert!(
            (result.ist_per_soll_pct.unwrap() - 250.0).abs() < 1e-3,
            "expected ist_per_soll_pct=250.0 (10/4*100), got {}",
            result.ist_per_soll_pct.unwrap()
        );
    }

    /// Divisions-Guard: contract_weeks == 0 => ist_per_contract_week = 0
    /// statt f32::NAN oder inf. Report liefert volunteer_hours = 0.0.
    #[tokio::test]
    async fn service_zero_contract_weeks_yields_zero_per_week() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        let sp_clone = sp.clone();
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp_clone.clone()));

        let mut reporting_service = MockReportingService::new();
        let sp_for_report = sp.clone();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(move |_, _, _, _, _, _| Ok(make_report(sp_for_report.clone(), 0.0)));

        // Keine working hours => contract_weeks=0.
        let empty_wh: Arc<[EmployeeWorkDetails]> = Arc::from(Vec::new());
        let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
        employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(empty_wh.clone()));

        let mut absence_service = MockAbsenceService::new();
        absence_service
            .expect_find_by_sales_person()
            .returning(|_, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
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
        // Erfuellungsgrad: soll=0 => None (Division-by-zero-Guard, FE
        // blendet die Zeile aus).
        assert!(
            result.ist_per_soll_pct.is_none(),
            "expected ist_per_soll_pct=None when soll_total=0, got {:?}",
            result.ist_per_soll_pct
        );
    }

    /// Erfuellungsgrad (Quick-Task 260710): Standard-Fall Ist < Soll.
    /// Fixture: committed_voluntary=2.0/Woche ueber KW 10..=13 (4 Wochen)
    /// => soll_total = 8.0. Report liefert ist_total = 6.4.
    /// Erwartung: ist_per_soll_pct = 6.4/8.0*100 = 80.0 %.
    #[tokio::test]
    async fn service_hr_pct_matches_ist_over_soll_at_80_percent() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        let sp_clone = sp.clone();
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp_clone.clone()));

        let mut reporting_service = MockReportingService::new();
        let sp_for_report = sp.clone();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(move |_, _, _, _, _, _| Ok(make_report(sp_for_report.clone(), 6.4)));

        // KW 10..=13/2026 = 28 Tage, committed_voluntary=2.0/Woche.
        // soll_total = 28 * 2.0/7 = 8.0.
        let wh: Arc<[EmployeeWorkDetails]> = Arc::from(vec![make_working_hours(
            sp_id,
            (2026, 10),
            (2026, 13),
            40.0,
            2.0,
        )]);
        let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
        employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(wh.clone()));

        let mut absence_service = MockAbsenceService::new();
        absence_service
            .expect_find_by_sales_person()
            .returning(|_, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::from_ymd(2026, 3, 2).unwrap();
        let to = ShiftyDate::from_ymd(2026, 3, 29).unwrap();
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        assert!((result.soll_total.unwrap() - 8.0).abs() < 1e-3);
        assert!((result.ist_total.unwrap() - 6.4).abs() < 1e-3);
        assert!(
            (result.ist_per_soll_pct.unwrap() - 80.0).abs() < 1e-3,
            "expected ist_per_soll_pct=80.0 (6.4/8.0*100), got {}",
            result.ist_per_soll_pct.unwrap()
        );
    }

    // ─── Phase 54.5 Service-Tests (D-54.5-03) ────────────────────────────────

    /// D-54.5-03 (Golden Service-Test): der Service laedt Absences via
    /// AbsenceService und reicht sie an beide pure fns weiter. Fixture:
    /// Vertrag KW 14..=26/2026, `committed_voluntary = 5.0`, Range =
    /// 2026-04-01 (Mi KW 14) .. 2026-06-30 (Di KW 27). Absence 2026-05-11
    /// (Mo KW 20) .. 2026-05-31 (So KW 22) — 3 zusammenhaengende Wochen.
    ///
    /// **v2.6.0 (falsch, im Kommentar dokumentiert):** `soll_total` haette die
    /// 3 Absence-Wochen mit 5.0h/Woche = 15.0h mitgezaehlt und die
    /// Contract-Wochen wuerden alle 12 ISO-Wochen im Range zaehlen (Range
    /// deckt KW 14..=27 an mind. 1 Tag).
    ///
    /// **v2.6.1 (korrekt, D-26-03 / D-54.5-01/02):** whole-week-out. KW
    /// 20/21/22 tragen 0h zum Soll bei; contract_weeks fallen um 3
    /// (siehe MEMORY `feedback_report_ist_matches_overall_aggregate` —
    /// der Delta-Sprung ist als Story im Test-Kommentar dokumentiert).
    ///
    /// Der Test verifiziert dass service.soll_total == pure_fn(&wh, from,
    /// to, &absences), also dass die Weiterreichung korrekt ist.
    #[tokio::test]
    async fn service_hr_soll_absence_aware_matches_pure_fn() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        let sp_clone = sp.clone();
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp_clone.clone()));

        let mut reporting_service = MockReportingService::new();
        let sp_for_report = sp.clone();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(move |_, _, _, _, _, _| Ok(make_report(sp_for_report.clone(), 0.0)));

        // Vertrag KW 14..=26/2026, committed_voluntary = 5.0/Woche.
        let wh_row = make_working_hours(sp_id, (2026, 14), (2026, 26), 40.0, 5.0);
        let wh_arc: Arc<[EmployeeWorkDetails]> = Arc::from(vec![wh_row.clone()]);
        let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
        employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(wh_arc.clone()));

        // Absence KW 20..=22 (2026-05-11..=2026-05-31).
        let abs = make_absence(
            sp_id,
            date!(2026 - 05 - 11),
            date!(2026 - 05 - 31),
            AbsenceCategory::Vacation,
        );
        let abs_arc: Arc<[AbsencePeriod]> = Arc::from(vec![abs.clone()]);
        let mut absence_service = MockAbsenceService::new();
        absence_service
            .expect_find_by_sales_person()
            .times(1)
            .returning(move |_, _, _| Ok(abs_arc.clone()));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::from_ymd(2026, 4, 1).unwrap();
        let to = ShiftyDate::from_ymd(2026, 6, 30).unwrap();
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        // Berechnung mit derselben pure fn + gleicher Absence-Liste MUSS
        // exakt matchen — beweist Weiterreichung ohne Semantik-Drift.
        let wh_vec = vec![wh_row];
        let expected_soll =
            committed_voluntary_target_in_range(&wh_vec, from, to, std::slice::from_ref(&abs));
        let expected_cw =
            contract_weeks_count_in_range(&wh_vec, from, to, std::slice::from_ref(&abs));
        assert!(
            (result.soll_total.unwrap() - expected_soll).abs() < 1e-3,
            "service soll_total ({}) must match pure fn ({})",
            result.soll_total.unwrap(),
            expected_soll
        );
        assert_eq!(
            result.contract_weeks,
            Some(expected_cw),
            "service contract_weeks ({:?}) must match pure fn ({})",
            result.contract_weeks,
            expected_cw
        );

        // v2.6.0-Falschwert-Nachvollzug: ohne Absence waeren die 3 Wochen
        // mit 5.0h/Woche = 15.0h zusaetzlich ins Soll geflossen.
        let soll_no_absence = committed_voluntary_target_in_range(&wh_vec, from, to, &[]);
        assert!(
            (soll_no_absence - expected_soll - 15.0).abs() < 1e-3,
            "v2.6.0 -> v2.6.1 Delta muss genau 15.0h (3 * 5.0h) sein, \
             no_absence={soll_no_absence}, with_absence={expected_soll}"
        );
    }

    /// D-54.5-03 Regression-Sicherheitsnetz: HR-Path mit leerer Absence-Liste
    /// liefert **byte-genau** dieselben Werte wie v2.6.0. Dieser Test setzt
    /// zusaetzlich `times(1)` auf `find_by_sales_person`, damit auch der
    /// HR-Path immer genau einmal Absences laedt.
    #[tokio::test]
    async fn service_hr_no_absence_matches_v260_output() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));

        let mut sales_person_service = MockSalesPersonService::new();
        let sp = make_sales_person(sp_id);
        let sp_clone = sp.clone();
        sales_person_service
            .expect_get()
            .returning(move |_, _, _| Ok(sp_clone.clone()));

        let mut reporting_service = MockReportingService::new();
        let sp_for_report = sp.clone();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(move |_, _, _, _, _, _| Ok(make_report(sp_for_report.clone(), 10.0)));

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

        let mut absence_service = MockAbsenceService::new();
        absence_service
            .expect_find_by_sales_person()
            .times(1)
            .returning(|_, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::from_ymd(2026, 3, 2).unwrap();
        let to = ShiftyDate::from_ymd(2026, 3, 29).unwrap();
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        // Byte-genaue v2.6.0-Werte (28 Tage * 1.0/7 = 4.0, 4 KW,
        // ist=10, delta=6, per-week=2.5).
        assert_eq!(result.contract_weeks, Some(4));
        assert!((result.ist_total.unwrap() - 10.0).abs() < 1e-3);
        assert!((result.soll_total.unwrap() - 4.0).abs() < 1e-3);
        assert!((result.delta.unwrap() - 6.0).abs() < 1e-3);
        assert!((result.ist_per_contract_week.unwrap() - 2.5).abs() < 1e-3);
        // Erfuellungsgrad: 10/4*100 = 250 %.
        assert!((result.ist_per_soll_pct.unwrap() - 250.0).abs() < 1e-3);
    }

    /// D-54.5-03 Non-HR-Path laedt AbsenceService NICHT. Beweis: MockAbsenceService
    /// ohne `expect_*` -> jeder Aufruf panikt. Zusaetzlich Guard via
    /// `service_non_hr_returns_all_none_vol_stat_02` bereits oben.
    #[tokio::test]
    async fn service_non_hr_does_not_load_absences() {
        let sp_id = Uuid::new_v4();

        let mut permission_service = MockPermissionService::new();
        permission_service
            .expect_check_permission()
            .returning(|_, _| Err(ServiceError::Forbidden));

        // Alle Domain-Mocks OHNE expect_* -> panicked-on-any-call.
        let reporting_service = MockReportingService::new();
        let employee_work_details_service = MockEmployeeWorkDetailsService::new();
        let sales_person_service = MockSalesPersonService::new();
        let absence_service = MockAbsenceService::new();
        let transaction_dao = dao::MockTransactionDao::new();

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
            employee_work_details_service: Arc::new(employee_work_details_service),
            sales_person_service: Arc::new(sales_person_service),
            absence_service: Arc::new(absence_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        };

        let from = ShiftyDate::first_day_in_year(2026);
        let to = ShiftyDate::last_day_in_year(2026);
        let result = svc
            .get_voluntary_stats(sp_id, from, to, Authentication::Context(()), None)
            .await
            .expect("Non-HR must not error");

        // Non-HR: all-None. Wenn AbsenceService::find_by_sales_person haette
        // aufgerufen werden muessen, waere der Test bereits gepanikt.
        assert!(result.soll_total.is_none());
        assert!(result.contract_weeks.is_none());
    }
}
