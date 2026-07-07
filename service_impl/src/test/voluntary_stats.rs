//! Phase 54 Plan 03 — Pure-fn + Service-Tests fuer VoluntaryStatsService (VOL-STAT-01/02,
//! VOL-ACCT-01/02/03). Die Tests decken:
//!
//! - D-F1-01: `contract_weeks_count` zaehlt jede ISO-Woche mit gueltiger
//!   `EmployeeWorkDetails`-Row als Vertragswoche — auch wenn `expected_hours == 0`.
//! - D-F2-01: `committed_voluntary_prorata_for_week` verteilt `committed_voluntary`
//!   tagesweise (Mo..So, je Tag 1/7); Mid-Week-Wechsel = 3/7*alt + 4/7*neu.
//! - D-54-DM-02 / VOL-ACCT-03 (Property-Test): `voluntary_ist_total_for_year`
//!   zaehlt AUSSCHLIESSLICH `ExtraHours` mit `source=Manual`. Rebooking-Marker-
//!   Rows sind neutral.
//! - Kongruenz-Test: Zaehler (F1) und Nenner (F2) nutzen dieselbe ISO-Wochen-
//!   Semantik (`ShiftyDate::as_shifty_week`).
//! - Service-Tests: HR-Gate mit Non-HR-Redaktion (VOL-STAT-02, VOL-ACCT-02).

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::employee_work_details::EmployeeWorkDetails;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource};
use shifty_utils::DayOfWeek;

use crate::reporting::{
    committed_voluntary_prorata_for_week, committed_voluntary_target_for_year,
    contract_weeks_count, voluntary_ist_total_for_year,
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

// ─── Pure-fn Tests ────────────────────────────────────────────────────────────

/// F1-Ist: 5 manuelle VolunteerWork-Rows a 4h in 2026 => 20.0h.
#[test]
fn f1_ist_manual_only_20h() {
    let sp = Uuid::new_v4();
    let hours = make_manual_volunteer_hours(sp, 2026, &[10, 11, 12, 13, 14], 4.0);
    let total = voluntary_ist_total_for_year(&hours, 2026);
    assert!((total - 20.0).abs() < 1e-4, "expected 20.0, got {total}");
}

/// VOL-ACCT-03 (Property-Test / D-54-DM-02): Ein Rebooking-Pair
/// (-4h VolunteerWork + +4h ExtraWork) mit source=Rebooking veraendert
/// F1-Ist NICHT — die Summe bleibt bei 20.0h.
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
    let total = voluntary_ist_total_for_year(&hours, 2026);
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
    let total = committed_voluntary_target_for_year(&wh, 2026);
    assert!((total - 0.0).abs() < 1e-4, "expected 0.0, got {total}");
}

/// D-F2-01: Mid-Week-Wechsel Mittwoch.
/// Vertrag A (KW 1..=W_MID_YEAR-1) committed_voluntary=7.0 endet Mittwoch,
/// Vertrag B (W_MID_YEAR..) committed_voluntary=14.0 beginnt Donnerstag.
/// In der Uebergangswoche: 3/7*7.0 + 4/7*14.0 = 3.0 + 8.0 = 11.0.
///
/// Da unsere `EmployeeWorkDetails`-Struktur Vertraege auf Wochenebene bewahrt
/// (from/to calendar_week), simulieren wir den Mid-Week-Wechsel dadurch, dass
/// die aktive `EmployeeWorkDetails`-Row per ISO-Woche bestimmt wird — hier
/// pruefen wir das mit einem Setup wo Vertrag A und B in DERSELBEN Woche
/// aktiv sind (via `find_working_hours_for_calendar_week` fuer year=2026,
/// week=W-MID):
///
/// Hinweis: `committed_voluntary_prorata_for_week` iteriert Mo..So und waehlt
/// pro Tag die aktive EmployeeWorkDetails. Der Wechsel Mittwoch=>Donnerstag
/// kann fuer die Woche W_MID modelliert werden durch:
/// - Vertrag A: von KW 1 bis KW W_MID (endet Mi der KW W_MID)
/// - Vertrag B: von KW W_MID (Do) bis KW 52
///
/// Da unsere `EmployeeWorkDetails` mit `from/to_day_of_week` arbeitet, koennen
/// wir das exakt modellieren.
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

/// D-F1-01: `contract_weeks_count` zaehlt eine EmployeeWorkDetails-Row
/// mit `expected_hours == 0` fuer die Wochen 10..=15 MIT (6 Wochen).
#[test]
fn contract_weeks_zero_expected_counts_d_f1_01() {
    let sp = Uuid::new_v4();
    let wh = vec![make_working_hours(sp, (2026, 10), (2026, 15), 0.0, 0.0)];
    let count = contract_weeks_count(&wh, 2026);
    assert_eq!(
        count, 6,
        "expected_hours=0 must still count contract weeks; expected 6, got {count}"
    );
}

/// contract_weeks bei leerer working-hours-Liste = 0.
#[test]
fn contract_weeks_empty_working_hours_returns_zero() {
    let wh: Vec<EmployeeWorkDetails> = Vec::new();
    let count = contract_weeks_count(&wh, 2026);
    assert_eq!(count, 0);
}

/// D-F2-01 ISO-Wochen-Randfall: Fuer ein 53-Wochen-Jahr (2026 hat laut ISO
/// 53 Wochen) summiert `committed_voluntary_target_for_year` ueber ALLE
/// Wochen inkl. 53. Fuer ein 52-Wochen-Jahr entsprechend 52.
#[test]
fn f2_soll_iso_week_53_year_boundary_d_f2_01() {
    let sp = Uuid::new_v4();

    // 2026 hat 53 ISO-Wochen (verifiziert via time::util::weeks_in_year).
    let year_2026_weeks = time::util::weeks_in_year(2026);
    // Vertrag ueber komplettes Jahr, committed_voluntary=1.0.
    let wh = vec![make_working_hours(sp, (2026, 1), (2026, year_2026_weeks), 40.0, 1.0)];
    let total = committed_voluntary_target_for_year(&wh, 2026);
    // Erwartung: pro Woche 7 Tage a 1.0/7 = 1.0; Summe = year_2026_weeks * 1.0.
    let expected = year_2026_weeks as f32;
    assert!(
        (total - expected).abs() < 1e-3,
        "expected {expected} for {year_2026_weeks}-week year, got {total}"
    );

    // Regressionslock gegen 52-Wochen-Annahme:
    // 2025 hat 52 ISO-Wochen.
    let year_2025_weeks = time::util::weeks_in_year(2025);
    assert_eq!(year_2025_weeks, 52, "2025 must be a 52-week ISO year");
    let wh_2025 = vec![make_working_hours(sp, (2025, 1), (2025, 52), 40.0, 1.0)];
    let total_2025 = committed_voluntary_target_for_year(&wh_2025, 2025);
    assert!(
        (total_2025 - 52.0).abs() < 1e-3,
        "expected 52.0 for 52-week ISO year, got {total_2025}"
    );
}

/// F1 + F2 muessen dieselbe ISO-Wochen-Semantik verwenden.
/// Kongruenz-Test: Ein Extra-Hours-Row am 2026-01-01 (das ist in der KW 1
/// des ISO-Jahres 2026 laut ISO-Kalender) wird von voluntary_ist_total_for_year
/// dem year=2026 zugeordnet.
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
    let total = voluntary_ist_total_for_year(&[eh], 2026);
    assert!(
        (total - 5.0).abs() < 1e-3,
        "2026-01-01 muss ISO-Jahr 2026 zugeordnet werden; expected 5.0, got {total}"
    );
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

        let result = svc
            .get_voluntary_stats(sp_id, 2026, Authentication::Context(()), None)
            .await
            .expect("Non-HR must not error, must return all-None VoluntaryStats");

        assert!(result.ist_per_contract_week.is_none());
        assert!(result.ist_total.is_none());
        assert!(result.soll_total.is_none());
        assert!(result.delta.is_none());
        assert!(result.contract_weeks.is_none());
    }

    /// VOL-STAT-01 + VOL-ACCT-01: HR-Aufrufer erhaelt konkrete Werte, die
    /// den pure fns entsprechen.
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

        // 2 manuelle VolunteerWork-Rows a 5h = 10.0h.
        let ehs: Arc<[ExtraHours]> =
            Arc::from(make_manual_volunteer_hours(sp_id, 2026, &[10, 11], 5.0));
        let mut extra_hours_service = MockExtraHoursService::new();
        extra_hours_service
            .expect_find_by_iso_year()
            .returning(move |_, _, _| Ok(ehs.clone()));

        // Working hours: KW 1..=10, committed_voluntary=1.0 => contract_weeks=10.
        // Prorata pro Woche = 7 * 1.0/7 = 1.0. Summe = 10.
        let wh: Arc<[EmployeeWorkDetails]> = Arc::from(vec![make_working_hours(
            sp_id,
            (2026, 1),
            (2026, 10),
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

        let result = svc
            .get_voluntary_stats(sp_id, 2026, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        assert_eq!(result.contract_weeks, Some(10));
        assert!((result.ist_total.unwrap() - 10.0).abs() < 1e-3);
        assert!((result.soll_total.unwrap() - 10.0).abs() < 1e-3);
        assert!((result.delta.unwrap() - 0.0).abs() < 1e-3);
        assert!((result.ist_per_contract_week.unwrap() - 1.0).abs() < 1e-3);
    }

    /// Divisions-Guard: contract_weeks == 0 => ist_per_contract_week = 0
    /// statt f32::NAN oder inf.
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

        let result = svc
            .get_voluntary_stats(sp_id, 2026, Authentication::Context(()), None)
            .await
            .expect("HR must succeed");

        assert_eq!(result.contract_weeks, Some(0));
        assert!((result.ist_per_contract_week.unwrap() - 0.0).abs() < 1e-3);
        assert!((result.ist_total.unwrap() - 0.0).abs() < 1e-3);
        assert!((result.soll_total.unwrap() - 0.0).abs() < 1e-3);
    }
}
