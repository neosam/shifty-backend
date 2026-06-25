//! Mock-basierte Service-Tests für `VacationBalanceServiceImpl` (Plan 08-02).
//!
//! Pflicht-Coverage (≥6 Tests, 08-02-PLAN.md):
//! 1. `get_returns_entitlement_minus_used_minus_planned`: Self-Path Happy.
//! 2. `get_with_hr_succeeds`: HR-Path Happy.
//! 3. `get_other_sales_person_without_hr_is_forbidden`: T-8-AUTH-01 + T-8-IDOR-01.
//! 4. `get_team_without_hr_is_forbidden`: T-8-AUTH-02.
//! 5. `get_team_aggregates_per_paid_sales_person`: HR-Path Aggregat.
//! 6. `get_with_no_active_contract_returns_zero_entitlement`: Edge-Case.
//! 7. `get_year_without_carryover_returns_zero_carryover`: Carryover-None-Path.

use std::collections::BTreeMap;
use std::sync::Arc;

use dao::MockTransaction;
use dao::MockTransactionDao;
use service::absence::{AbsenceCategory, MockAbsenceService, ResolvedAbsence};
use service::carryover::{Carryover, MockCarryoverService};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::vacation_balance::VacationBalanceService;
use service::{MockPermissionService, ServiceError};
use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use time::Duration;
use uuid::{uuid, Uuid};

use crate::test::error_test::test_forbidden;
use crate::vacation_balance::{VacationBalanceServiceDeps, VacationBalanceServiceImpl};

const TEST_YEAR: u32 = 2026;

fn default_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000001")
}

fn other_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000002")
}

/// Baut die `derive_hours_for_range`-Mock-Antwort: eine Tag→ResolvedAbsence-Map
/// mit Kategorie `Vacation`, den gegebenen effektiven Stunden und Tagesanteil
/// (`days`) pro Tag — exakt so, wie es der echte `derive_hours_for_range`
/// liefert (Halbtage und Wochen-Deckelung sind in `days` bereits eingerechnet;
/// `hours == days * hours_per_day`). `used_days`/`planned_days` werden im
/// Service nun direkt aus `days` summiert.
fn vacation_hours_map(entries: &[(time::Date, f32, f32)]) -> BTreeMap<time::Date, ResolvedAbsence> {
    entries
        .iter()
        .map(|(d, h, days)| {
            (
                *d,
                ResolvedAbsence {
                    category: AbsenceCategory::Vacation,
                    hours: *h,
                    days: *days,
                },
            )
        })
        .collect()
}

/// `n` aufeinanderfolgende Vacation-Tage ab `start`, je `hours_each` Stunden
/// und `days_each` Tagesanteil.
fn consecutive_vacation(
    start: time::Date,
    n: i64,
    hours_each: f32,
    days_each: f32,
) -> Vec<(time::Date, f32, f32)> {
    (0..n)
        .map(|i| (start + Duration::days(i), hours_each, days_each))
        .collect()
}

fn full_year_contract(sp_id: Uuid, vacation_days: u8) -> EmployeeWorkDetails {
    contract(sp_id, vacation_days, 40.0, 5)
}

/// Voll-Jahres-Vertrag (KW1/2025 .. KW52/2030) mit parametrierbaren Stunden /
/// Workdays — für Teilzeit-Tests. `hours_per_day = expected_hours / workdays`.
fn contract(
    sp_id: Uuid,
    vacation_days: u8,
    expected_hours: f32,
    workdays_per_week: u8,
) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: uuid!("11111111-0000-0000-0000-000000000001"),
        sales_person_id: sp_id,
        expected_hours,
        from_day_of_week: DayOfWeek::Monday,
        // Calendar week 1 of 2026 → covers full year for our tests.
        from_calendar_week: 1,
        from_year: 2025,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2030,
        workdays_per_week,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        committed_voluntary: 0.0,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days,
        created: Some(datetime!(2024 - 12 - 15 09:00:00)),
        deleted: None,
        version: uuid!("DD000000-0000-0000-0000-000000000001"),
    }
}

fn paid_sales_person(id: Uuid, name: &str) -> SalesPerson {
    SalesPerson {
        id,
        name: name.into(),
        background_color: "#888888".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: uuid!("EE000000-0000-0000-0000-000000000001"),
    }
}

pub(crate) struct VacationBalanceDependencies {
    pub absence_service: MockAbsenceService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub carryover_service: MockCarryoverService,
    pub sales_person_service: MockSalesPersonService,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub transaction_dao: MockTransactionDao,
}

impl VacationBalanceServiceDeps for VacationBalanceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type AbsenceService = MockAbsenceService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type CarryoverService = MockCarryoverService;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type TransactionDao = MockTransactionDao;
}

impl VacationBalanceDependencies {
    pub(crate) fn build_service(self) -> VacationBalanceServiceImpl<VacationBalanceDependencies> {
        VacationBalanceServiceImpl {
            absence_service: self.absence_service.into(),
            employee_work_details_service: self.employee_work_details_service.into(),
            carryover_service: self.carryover_service.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

/// Build dependencies with a default `today = 2026-06-15` and a happy-path
/// transaction setup. Permission/sales-person checks default to `Ok(())`
/// so individual tests can override them for forbidden flows.
pub(crate) fn build_dependencies() -> VacationBalanceDependencies {
    let absence_service = MockAbsenceService::new();
    let employee_work_details_service = MockEmployeeWorkDetailsService::new();
    let carryover_service = MockCarryoverService::new();
    let sales_person_service = MockSalesPersonService::new();
    let permission_service = MockPermissionService::new();
    let mut clock_service = MockClockService::new();
    let mut transaction_dao = MockTransactionDao::new();

    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));
    clock_service
        .expect_date_now()
        .returning(|| date!(2026 - 06 - 15));

    VacationBalanceDependencies {
        absence_service,
        employee_work_details_service,
        carryover_service,
        sales_person_service,
        permission_service,
        clock_service,
        transaction_dao,
    }
}

// =========================================================================
// get
// =========================================================================

#[tokio::test]
async fn get_returns_entitlement_minus_used_minus_planned() {
    let mut deps = build_dependencies();

    // Self-Path: HR fails, verify_user_is_sales_person succeeds.
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    // 5 vergangene Vacation-Tage (je 8h, vor today=06-15 → used = 40h/8 = 5 Tage).
    // 10 zukünftige Vacation-Tage (je 8h, nach today → planned = 80h/8 = 10 Tage).
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(move |_, _, _, _, _| {
            let mut entries = consecutive_vacation(date!(2026 - 04 - 01), 5, 8.0, 1.0);
            entries.extend(consecutive_vacation(date!(2026 - 08 - 01), 10, 8.0, 1.0));
            Ok(vacation_hours_map(&entries))
        });

    // Vertrag: 25 Vacation-Tage, deckt das ganze Jahr ab.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 25)])));

    // Carryover: 5 Tage.
    deps.carryover_service
        .expect_get_carryover()
        .returning(move |_, _, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year: TEST_YEAR,
                carryover_hours: 0.0,
                vacation: 5,
                created: datetime!(2025 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000001"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert_eq!(result.sales_person_id, sp_id);
    assert_eq!(result.year, TEST_YEAR);
    // entitled_days approx 25.0 (full year contract).
    assert!(
        (result.entitled_days - 25.0).abs() < 0.01,
        "entitled_days = {}",
        result.entitled_days
    );
    assert_eq!(result.carryover_days, 5);
    assert!(
        (result.used_days - 5.0).abs() < 0.01,
        "used_days = {}",
        result.used_days
    );
    assert!(
        (result.planned_days - 10.0).abs() < 0.01,
        "planned_days = {}",
        result.planned_days
    );
    // remaining = 25 + 5 - (5 + 10) = 15
    assert!(
        (result.remaining_days - 15.0).abs() < 0.01,
        "remaining_days = {}",
        result.remaining_days
    );
}

#[tokio::test]
async fn get_with_hr_succeeds() {
    let mut deps = build_dependencies();

    // HR-Path: HR succeeds (verify_user_is_sales_person may fail — `or()` filters it).
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));

    let sp_id = other_sales_person_id();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 30)])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("HR-path get should succeed");

    assert_eq!(result.sales_person_id, sp_id);
    assert_eq!(result.carryover_days, 0);
    assert!(
        (result.used_days - 0.0).abs() < 0.01,
        "used_days = {}",
        result.used_days
    );
    assert!(
        (result.planned_days - 0.0).abs() < 0.01,
        "planned_days = {}",
        result.planned_days
    );
    // entitled approx 30 (full year contract), remaining = 30 - 0 = 30
    assert!(
        (result.entitled_days - result.remaining_days).abs() < 0.01,
        "entitled and remaining should match without used/planned/carryover: entitled={}, remaining={}",
        result.entitled_days,
        result.remaining_days
    );
}

#[tokio::test]
async fn get_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();

    // Both HR and verify_user_is_sales_person fail → Forbidden surfaces.
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));

    let svc = deps.build_service();
    let result = svc
        .get(
            other_sales_person_id(),
            TEST_YEAR,
            Authentication::Full,
            None,
        )
        .await;

    test_forbidden(&result);
}

// =========================================================================
// get_team
// =========================================================================

#[tokio::test]
async fn get_team_without_hr_is_forbidden() {
    let mut deps = build_dependencies();

    // get_team is HR-only; no verify_user_is_sales_person fallback.
    deps.permission_service
        .expect_check_permission()
        .with(mockall::predicate::eq(HR_PRIVILEGE), mockall::predicate::always())
        .returning(|_, _| Err(ServiceError::Forbidden));

    let svc = deps.build_service();
    let result = svc.get_team(TEST_YEAR, Authentication::Full, None).await;

    test_forbidden(&result);
}

#[tokio::test]
async fn get_team_aggregates_per_paid_sales_person() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let sp_a = default_sales_person_id();
    let sp_b = other_sales_person_id();
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(move |_, _| {
            Ok(Arc::from([
                paid_sales_person(sp_a, "Alice"),
                paid_sales_person(sp_b, "Bob"),
            ]))
        });

    // Pro Person 3 vergangene Vacation-Tage (je 8h → used = 24h/8 = 3 Tage).
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(move |_, _, _, _, _| {
            Ok(vacation_hours_map(&consecutive_vacation(
                date!(2026 - 03 - 01),
                3,
                8.0,
                1.0,
            )))
        });

    // Pro Person ein Voll-Jahres-Vertrag, beide mit 20 Vacation-Tagen.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |sp_id, _, _| Ok(Arc::from([full_year_contract(sp_id, 20)])));

    // Pro Person ein Carryover von 2 Tagen.
    deps.carryover_service
        .expect_get_carryover()
        .returning(move |sp_id, year, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year,
                carryover_hours: 0.0,
                vacation: 2,
                created: datetime!(2025 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000002"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get_team(TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get_team should succeed");

    assert_eq!(result.len(), 2);
    let ids: Vec<Uuid> = result.iter().map(|b| b.sales_person_id).collect();
    assert!(ids.contains(&sp_a));
    assert!(ids.contains(&sp_b));
    for balance in result.iter() {
        assert_eq!(balance.year, TEST_YEAR);
        assert_eq!(balance.carryover_days, 2);
        assert!((balance.used_days - 3.0).abs() < 0.01);
        assert!((balance.planned_days - 0.0).abs() < 0.01);
        assert!((balance.entitled_days - 20.0).abs() < 0.01);
        // remaining = 20 + 2 - 3 - 0 = 19
        assert!(
            (balance.remaining_days - 19.0).abs() < 0.01,
            "remaining_days = {}",
            balance.remaining_days
        );
    }
}

// =========================================================================
// edge cases
// =========================================================================

#[tokio::test]
async fn get_with_no_active_contract_returns_zero_entitlement() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    // No active contracts.
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from([])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(
            default_sales_person_id(),
            TEST_YEAR,
            Authentication::Full,
            None,
        )
        .await
        .expect("get should succeed even without contracts");

    assert!(
        (result.entitled_days - 0.0).abs() < 0.01,
        "entitled_days = {}",
        result.entitled_days
    );
    assert_eq!(result.carryover_days, 0);
    assert!(
        (result.remaining_days - 0.0).abs() < 0.01,
        "remaining_days = {}",
        result.remaining_days
    );
}

/// Unterjähriger Vertragsstart erzeugt einen aliquoten (fraktionalen)
/// Anspruch — der Service MUSS ihn auf eine ganze Zahl runden.
///
/// Vertrag startet KW27/2026 (= 2026-06-29, ordinal 180), 30 Urlaubstage.
/// Roher Anspruch = 30 − 30·(180/365) ≈ 15.21 → gerundet exakt 15.
/// Ohne `.round()` wäre `entitled_days` ≈ 15.21 und der `== 15.0`-Assert
/// schlüge fehl.
#[tokio::test]
async fn get_rounds_aliquot_entitlement_to_whole_number() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();

    // Vertrag mit unterjährigem Start KW27/2026 → anteiliger Anspruch.
    let mut mid_year_contract = full_year_contract(sp_id, 30);
    mid_year_contract.from_year = 2026;
    mid_year_contract.from_calendar_week = 27;
    mid_year_contract.from_day_of_week = DayOfWeek::Monday;

    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([mid_year_contract.clone()])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    // Immer eine gerundete ganze Zahl.
    assert_eq!(
        result.entitled_days,
        result.entitled_days.round(),
        "entitled_days muss ganzzahlig sein, war {}",
        result.entitled_days
    );
    assert_eq!(
        result.entitled_days, 15.0,
        "aliquoter Anspruch (~15.21) muss auf 15 gerundet werden, war {}",
        result.entitled_days
    );
    // remaining = entitled (kein carryover/used/planned) → ebenfalls ganzzahlig.
    assert_eq!(result.remaining_days, 15.0);
}

#[tokio::test]
async fn get_year_without_carryover_returns_zero_carryover() {
    let mut deps = build_dependencies();

    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 24)])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert_eq!(result.carryover_days, 0);
    // entitled approx 24, remaining = 24 - 0 - 0 = 24
    assert!(
        (result.entitled_days - result.remaining_days).abs() < 0.01,
        "entitled and remaining should match without used/planned/carryover"
    );
}

// =========================================================================
// stundenbasierte Tage (Decision 2026-06-12) — Konsistenz mit ReportingService.
//
// Feiertags-/Nicht-Workday-Filterung lebt in `derive_hours_for_range` selbst
// (hier gemockt) und ist in den absence/reporting-Tests abgedeckt. Auf dieser
// Ebene wird die Stunden→Tage-Umrechnung verifiziert: Halbtage, das
// hours_per_day des Vertrags (Teilzeit), der today-Split und der
// Kategorie-Filter (nur Vacation).
// =========================================================================

/// Convenience: happy-path deps mit gegebenem derive-Resultat + Vertrag, ohne
/// Carryover. `today = 2026-06-15`.
fn happy_deps_with(
    sp_id: Uuid,
    resolved: BTreeMap<time::Date, ResolvedAbsence>,
    contract_details: EmployeeWorkDetails,
) -> VacationBalanceDependencies {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    deps.absence_service
        .expect_derive_hours_for_range()
        .return_once(move |_, _, _, _, _| Ok(resolved));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([contract_details.clone()])));
    deps.carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    let _ = sp_id;
    deps
}

/// Ein vergangener Halbtag (4h bei hours_per_day=8) → 0.5 used-Tage.
#[tokio::test]
async fn half_day_vacation_counts_as_half_day() {
    let sp_id = default_sales_person_id();
    // hours_per_day = 40 / 5 = 8; Halbtag = 4h / days=0.5 (so liefert
    // derive_hours_for_range einen Half-Day bereits: hours_per_day * 0.5, days 0.5).
    let resolved = vacation_hours_map(&[(date!(2026 - 04 - 10), 4.0, 0.5)]);
    let deps = happy_deps_with(sp_id, resolved, full_year_contract(sp_id, 25));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert!(
        (result.used_days - 0.5).abs() < 0.001,
        "half day must count as 0.5, got {}",
        result.used_days
    );
    assert!((result.planned_days - 0.0).abs() < 0.001);
}

/// Teilzeit-Vertrag (20h/5 Tage → hours_per_day=4): 3 vergangene volle
/// Vacation-Tage (je 4h / days=1.0) → 3 used-Tage. Die Tage kommen jetzt
/// direkt aus `ResolvedAbsence.days` (gedeckelt von derive_hours_for_range),
/// nicht aus einer Stunden→Tage-Division.
#[tokio::test]
async fn part_time_contract_used_days_come_from_days_field() {
    let sp_id = default_sales_person_id();
    let resolved = vacation_hours_map(&consecutive_vacation(date!(2026 - 02 - 02), 3, 4.0, 1.0));
    let deps = happy_deps_with(sp_id, resolved, contract(sp_id, 20, 20.0, 5));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert!(
        (result.used_days - 3.0).abs() < 0.001,
        "3 volle Tage (days=1.0) müssen 3 used-Tage ergeben, got {}",
        result.used_days
    );
}

/// Aktive Periode um today=06-15: 06-13/14/15 (≤ today) → used, 06-16/17 →
/// planned. today selbst zählt zu used.
#[tokio::test]
async fn active_period_splits_used_and_planned_at_today() {
    let sp_id = default_sales_person_id();
    let resolved = vacation_hours_map(&consecutive_vacation(date!(2026 - 06 - 13), 5, 8.0, 1.0));
    let deps = happy_deps_with(sp_id, resolved, full_year_contract(sp_id, 30));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    // 06-13, 06-14, 06-15 = 3 Tage used (today inklusive); 06-16, 06-17 = 2 planned.
    assert!(
        (result.used_days - 3.0).abs() < 0.001,
        "used_days = {}",
        result.used_days
    );
    assert!(
        (result.planned_days - 2.0).abs() < 0.001,
        "planned_days = {}",
        result.planned_days
    );
}

/// Conflict-resolved Nicht-Vacation-Tage (z.B. SickLeave) zählen NICHT zum
/// Urlaubs-Aggregat.
#[tokio::test]
async fn non_vacation_categories_are_ignored() {
    let sp_id = default_sales_person_id();
    let resolved: BTreeMap<time::Date, ResolvedAbsence> = [
        (
            date!(2026 - 03 - 02),
            ResolvedAbsence {
                category: AbsenceCategory::SickLeave,
                hours: 8.0,
                days: 1.0,
            },
        ),
        (
            date!(2026 - 03 - 03),
            ResolvedAbsence {
                category: AbsenceCategory::UnpaidLeave,
                hours: 8.0,
                days: 1.0,
            },
        ),
    ]
    .into_iter()
    .collect();
    let deps = happy_deps_with(sp_id, resolved, full_year_contract(sp_id, 25));

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert!(
        (result.used_days - 0.0).abs() < 0.001,
        "sick/unpaid must not count as vacation, got {}",
        result.used_days
    );
    assert!((result.planned_days - 0.0).abs() < 0.001);
}

// =========================================================================
// Carryover-Year-Semantik (Alignierung mit ReportingService)
//
// Eine Carryover-Row mit `year = Y` speichert den Ende-von-Jahr-Y-Saldo, der
// in Jahr Y+1 eingebracht wird. VacationBalanceService muss `get_carryover(sp,
// year - 1)` aufrufen — exakt wie `ReportingService::get_employee`. Tests
// prüfen:
//   a) Dass `get_carryover` mit `TEST_YEAR - 1` aufgerufen wird.
//   b) Dass eine Carryover-Row für `TEST_YEAR - 1` korrekt in die Balance
//      eingerechnet wird.
//   c) Dass eine Carryover-Row für `TEST_YEAR` (aktuelles Jahr) NICHT zur
//      Balance beiträgt — sie beschreibt den Übertrag ins NÄCHSTE Jahr.
// =========================================================================

/// VacationBalanceService ruft `get_carryover(sp_id, year - 1)` auf.
/// Test prüft dies über eine `with`-Expectation auf den year-Parameter.
#[tokio::test]
async fn get_carryover_is_called_with_previous_year() {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(Default::default()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 25)])));

    // Expectation: get_carryover MUSS mit TEST_YEAR - 1 aufgerufen werden.
    deps.carryover_service
        .expect_get_carryover()
        .with(
            mockall::predicate::always(),
            mockall::predicate::eq(TEST_YEAR - 1),
            mockall::predicate::always(),
            mockall::predicate::always(),
        )
        .once()
        .returning(|_, _, _, _| Ok(None));

    let svc = deps.build_service();
    svc.get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");
    // If get_carryover was called with a different year, mockall will panic here.
}

/// Eine Carryover-Row für `TEST_YEAR - 1` (Vorjahr) fließt korrekt in die
/// Balance ein: `carryover_days` = vacation-Field der Row.
#[tokio::test]
async fn carryover_from_previous_year_is_included_in_balance() {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(Default::default()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 25)])));

    // Carryover für das Vorjahr (TEST_YEAR - 1): 7 Tage Urlaub.
    deps.carryover_service
        .expect_get_carryover()
        .returning(move |_, _, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year: TEST_YEAR - 1,
                carryover_hours: 0.0,
                vacation: 7,
                created: datetime!(2025 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000003"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    assert_eq!(
        result.carryover_days, 7,
        "Carryover from previous year must appear in carryover_days"
    );
    // remaining = entitled (25) + carryover (7) - used (0) - planned (0) = 32
    assert!(
        (result.remaining_days - 32.0).abs() < 0.01,
        "remaining_days = {}, expected 32",
        result.remaining_days
    );
}

/// Regression-Pin: VacationBalanceService ruft `get_carryover` mit `year - 1`
/// auf — exakt wie `ReportingService::get_employee` in `reporting.rs:662-672`.
///
/// Die `withf`-Expectation bricht den Test, sobald der Produktiv-Code auf
/// `year` (ohne `- 1`) geändert wird. D-18-02: Vacation-Balance carryover ==
/// Report-Service carryover für gleiche Person/Jahr.
#[tokio::test]
async fn carryover_read_uses_prior_year() {
    let mut deps = build_dependencies();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let sp_id = default_sales_person_id();
    deps.absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(Default::default()));
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(Arc::from([full_year_contract(sp_id, 25)])));

    // withf matcher: schlägt fehl, wenn der zweite Argument nicht TEST_YEAR - 1 ist.
    deps.carryover_service
        .expect_get_carryover()
        .withf(|_sp, year, _auth, _tx| *year == TEST_YEAR - 1)
        .times(1)
        .returning(move |_, _, _, _| {
            Ok(Some(Carryover {
                sales_person_id: sp_id,
                year: TEST_YEAR - 1,
                carryover_hours: 0.0,
                vacation: 7,
                created: datetime!(2024 - 12 - 31 23:59:00),
                deleted: None,
                version: uuid!("FF000000-0000-0000-0000-000000000010"),
            }))
        });

    let svc = deps.build_service();
    let result = svc
        .get(sp_id, TEST_YEAR, Authentication::Full, None)
        .await
        .expect("get should succeed");

    // Der Carryover-Wert aus der Vorjahres-Row muss in der Balance erscheinen.
    assert_eq!(
        result.carryover_days, 7,
        "carryover_days must come from the prior-year (TEST_YEAR - 1) record"
    );
}
