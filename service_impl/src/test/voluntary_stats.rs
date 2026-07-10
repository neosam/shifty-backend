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

use time::macros::datetime;
use uuid::Uuid;

use service::employee_work_details::EmployeeWorkDetails;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{
    committed_voluntary_prorata_for_week, committed_voluntary_target_in_range,
    contract_weeks_count_in_range,
};

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
    let total = committed_voluntary_target_in_range(&wh, from, to);
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
    let count = contract_weeks_count_in_range(&wh, from, to);
    // Weeks 10..=15 = 6 weeks.
    assert_eq!(count, 6, "expected 6, got {count}");
}

/// D-F1-01: leere `EmployeeWorkDetails` liefert count = 0.
#[test]
fn contract_weeks_empty_working_hours_returns_zero() {
    let wh: Vec<EmployeeWorkDetails> = Vec::new();
    let (from, to) = full_year_range(2026);
    let count = contract_weeks_count_in_range(&wh, from, to);
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
    let total = committed_voluntary_target_in_range(&wh, from_2026, to_2026);
    let expected_2026 = 365.0 / 7.0;
    assert!(
        (total - expected_2026).abs() < 0.01,
        "expected ~{expected_2026:.3} for full-year 2026 (365 days / 7), got {total}"
    );

    let wh_2025 = vec![make_working_hours(sp_id, (2025, 1), (2025, 52), 40.0, 1.0)];
    let (from_2025, to_2025) = full_year_range(2025);
    let total_2025 = committed_voluntary_target_in_range(&wh_2025, from_2025, to_2025);
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
    let total = committed_voluntary_target_in_range(&wh, from, to);
    let expected = 362.0 / 7.0;
    assert!(
        (total - expected).abs() < 0.01,
        "expected ~{expected:.3} (362 overlap-days / 7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert_eq!(count, 52, "expected 52 contract weeks, got {count}");
}

/// Gap-Closure G1 Regression: Full-Year-Range 2026 = 53 Wochen im Nenner
/// (contract_weeks), tages-basierter Zaehler = 365/7 ≈ 52.143.
#[test]
fn range_regression_full_year_2026_matches_old_semantics() {
    let sp_id = Uuid::new_v4();
    let wh = vec![make_working_hours(sp_id, (2026, 1), (2026, 53), 40.0, 1.0)];
    let (from, to) = full_year_range(2026);
    let total = committed_voluntary_target_in_range(&wh, from, to);
    let expected = 365.0 / 7.0;
    assert!(
        (total - expected).abs() < 0.01,
        "expected ~{expected:.3} (365/7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to);
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
    let total = committed_voluntary_target_in_range(&wh, from, to);
    // KW 21: 5 Tage (Mi..=So) * 7/7 = 5.0
    // KW 22: 7 Tage * 7/7 = 7.0
    // Summe: 12.0
    assert!(
        (total - 12.0).abs() < 1e-3,
        "expected 12.0 (5+7), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to);
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
    let total = committed_voluntary_target_in_range(&wh, from, to);
    // 4 Tage * 7/7 = 4.0.
    assert!(
        (total - 4.0).abs() < 1e-3,
        "expected 4.0 (Mo..=Do 4 Tage * 1), got {total}"
    );
    let count = contract_weeks_count_in_range(&wh, from, to);
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
    let soll_total = committed_voluntary_target_in_range(&wh, from, to);
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
    let soll_total = committed_voluntary_target_in_range(&wh, from, to);
    let count = contract_weeks_count_in_range(&wh, from, to);
    assert!(
        (soll_total - 0.0).abs() < 1e-3,
        "expected 0.0, got {soll_total}"
    );
    assert_eq!(count, 0, "expected 0 contract weeks, got {count}");
}

// ─── Service-Tests (mockall) ──────────────────────────────────────────────────
//
// Gap-Closure 54-09-Ist-Fix: Ist-Aggregat kommt aus
// `ReportingService::get_report_for_employee_range` — konsistent zum OVERALL-
// "Ehrenamt"-Wert der UI. Der Service-Test verifiziert die Delegation an den
// ReportingService-Mock (kein Aufruf im Non-HR-Path).

mod service_tests {
    use super::*;
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
        let reporting_service = MockReportingService::new();
        let employee_work_details_service = MockEmployeeWorkDetailsService::new();
        let sales_person_service = MockSalesPersonService::new();
        let transaction_dao = dao::MockTransactionDao::new();

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
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

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
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

        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        let svc: VoluntaryStatsServiceImpl<TestDeps> = VoluntaryStatsServiceImpl {
            reporting_service: Arc::new(reporting_service),
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
