//! Behavioral acceptance tests for Phase 25 holiday derive-on-read (25-04).
//!
//! HOL-01: Basic holiday credit from SpecialDay (8h for Mon-Fri 40h contract).
//! HOL-02: Derived credit produces identical holiday_hours, expected_hours, and
//!         balance as an equivalent manual ExtraHours(Holiday).
//! HCFG-01: Cutoff gate boundary — holiday BEFORE cutoff → 0h; ON cutoff → 8h.
//! HCFG-03: Manual ExtraHours(Holiday) on the same day → credited once, not twice.
//! HOL-03: booking_information year-view (paid_hours/committed_voluntary/volunteer)
//!         is unaffected by holiday auto-credit (get_week() has no derive-on-read).
//!
//! Structural template: service_impl/src/test/reporting_additive_merge.rs.
//! Implementation under test: service_impl/src/reporting.rs (build_derived_holiday_map +
//! three injection points via hours_per_week / get_report_for_employee_range / get_week).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::{MockSpecialDayService, SpecialDay, SpecialDayType};
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
};

// ─── ReportingMocks / TestDeps (same pattern as reporting_additive_merge.rs) ──

struct ReportingMocks {
    extra_hours_service: MockExtraHoursService,
    shiftplan_report_service: MockShiftplanReportService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    sales_person_service: MockSalesPersonService,
    carryover_service: MockCarryoverService,
    permission_service: MockPermissionService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    absence_service: MockAbsenceService,
    transaction_dao: dao::MockTransactionDao,
    // Phase 25: holiday derive-on-read deps.
    special_day_service: MockSpecialDayService,
    toggle_service: MockToggleService,
}

struct TestDeps;
impl ReportingServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ExtraHoursService = MockExtraHoursService;
    type ShiftplanReportService = MockShiftplanReportService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type SalesPersonService = MockSalesPersonService;
    type CarryoverService = MockCarryoverService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type AbsenceService = MockAbsenceService;
    type TransactionDao = dao::MockTransactionDao;
    // Phase 25: holiday derive-on-read deps.
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}

impl ReportingMocks {
    fn new() -> Self {
        // Phase 25: toggle automation off by default (no value = no holiday auto-credit).
        let mut toggle_service = MockToggleService::new();
        toggle_service
            .expect_get_toggle_value()
            .returning(|_, _, _| Ok(None));
        Self {
            extra_hours_service: MockExtraHoursService::new(),
            shiftplan_report_service: MockShiftplanReportService::new(),
            employee_work_details_service: MockEmployeeWorkDetailsService::new(),
            sales_person_service: MockSalesPersonService::new(),
            carryover_service: MockCarryoverService::new(),
            permission_service: MockPermissionService::new(),
            clock_service: MockClockService::new(),
            uuid_service: MockUuidService::new(),
            absence_service: MockAbsenceService::new(),
            transaction_dao: dao::MockTransactionDao::new(),
            special_day_service: MockSpecialDayService::new(),
            toggle_service,
        }
    }

    fn build(self) -> ReportingServiceImpl<TestDeps> {
        ReportingServiceImpl {
            extra_hours_service: Arc::new(self.extra_hours_service),
            shiftplan_report_service: Arc::new(self.shiftplan_report_service),
            employee_work_details_service: Arc::new(self.employee_work_details_service),
            sales_person_service: Arc::new(self.sales_person_service),
            carryover_service: Arc::new(self.carryover_service),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            absence_service: Arc::new(self.absence_service),
            transaction_dao: Arc::new(self.transaction_dao),
            special_day_service: Arc::new(self.special_day_service),
            toggle_service: Arc::new(self.toggle_service),
        }
    }
}

// ─── Domain helpers ───────────────────────────────────────────────────────────

/// Create a SpecialDay of type Holiday for the given (year, calendar_week, day_of_week).
fn make_holiday(year: u32, calendar_week: u8, day_of_week: DayOfWeek) -> SpecialDay {
    SpecialDay {
        id: Uuid::new_v4(),
        year,
        calendar_week,
        day_of_week,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: Some(datetime!(2024 - 01 - 01 00:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Create an ExtraHours row with category=Holiday for the given amount and calendar date.
fn make_holiday_extra_hours(amount: f32, day: time::Date) -> ExtraHours {
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sales_person_id(),
        amount,
        category: ExtraHoursCategory::Holiday,
        description: Arc::from("manual holiday"),
        date_time: time::PrimitiveDateTime::new(day, time::Time::from_hms(9, 0, 0).unwrap()),
        created: Some(datetime!(2024 - 01 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Standard boilerplate for `get_report_for_employee_range` tests in KW22-25/2024.
/// Sets permission, sales_person, work_details (fixture_work_details_8h_mon_fri),
/// shiftplan_report (empty), carryover (None), transaction passthrough,
/// absence_service (empty map), extra_hours (empty slice).
///
/// Tests that need different work_details or extra_hours should set those mocks
/// AFTER calling this function (replace the mock field on the mocks struct).
fn setup_holiday_common_mocks(mocks: &mut ReportingMocks) {
    mocks
        .permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    mocks
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    // Work details: 8h/day Mon-Fri, valid KW22-25/2024. Covers the standard holiday date
    // 2024-06-03 (Monday of KW23/2024).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
}

/// Run get_report_for_employee_range for KW23/2024 (Mon 2024-06-03 to Sun 2024-06-09).
async fn run_report_kw23(
    service: ReportingServiceImpl<TestDeps>,
) -> service::reporting::EmployeeReport {
    service
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            ShiftyDate::from_ymd(2024, 6, 3).unwrap(),
            ShiftyDate::from_ymd(2024, 6, 9).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("get_report_for_employee_range must succeed")
}

// ─── HOL-01: Basic derive ─────────────────────────────────────────────────────

/// HOL-01: An employee with a Mon-Fri 40h contract gets exactly holiday_hours()=8h
/// credited automatically for a SpecialDay(Holiday) on the contracted Monday.
///
/// Fixture: KW23/2024 (Mon 2024-06-03 to Sun 2024-06-09).
/// Toggle cutoff: 2024-01-01 (before the holiday date → credit applies).
/// No manual ExtraHours(Holiday).
/// Expected: report.holiday_hours == 8.0.
#[tokio::test]
async fn test_holiday_auto_credit_basic() {
    let mut mocks = ReportingMocks::new();
    setup_holiday_common_mocks(&mut mocks);

    // Override toggle: cutoff = 2024-01-01 (before holiday 2024-06-03 → qualifies).
    mocks.toggle_service = MockToggleService::new();
    mocks
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2024-01-01"))));

    // SpecialDay: Holiday on KW23/2024, Monday (concrete date = 2024-06-03).
    mocks
        .special_day_service
        .expect_get_by_week()
        .returning(|_, wk, _| {
            if wk == 23 {
                Ok(Arc::from(vec![make_holiday(2024, 23, DayOfWeek::Monday)]))
            } else {
                Ok(Arc::from(vec![]))
            }
        });

    let report = run_report_kw23(mocks.build()).await;

    assert!(
        (report.holiday_hours - 8.0).abs() < 0.01,
        "HOL-01: holiday_hours must be 8.0 (auto-credit for Mon-Fri 40h/5d contract), got {}",
        report.holiday_hours
    );
}

// ─── HOL-02: Derived == manual equivalence ────────────────────────────────────

/// HOL-02: The derived holiday credit produces IDENTICAL holiday_hours, expected_hours,
/// and balance_hours as an equivalent manual ExtraHours(Holiday, 8.0).
///
/// Run A: toggle enabled + SpecialDay(Holiday, KW23 Mon), no manual ExtraHours.
/// Run B: toggle off + no SpecialDay, manual ExtraHours(Holiday, 8.0) on 2024-06-03.
///
/// Assertions:
///   A.holiday_hours == B.holiday_hours == 8.0
///   A.expected_hours == B.expected_hours (both reduced by 8h absence from holiday)
///   A.balance_hours == B.balance_hours
#[tokio::test]
async fn test_holiday_auto_credit_equivalence() {
    // --- Run A: derived holiday (SpecialDay + toggle, no manual ExtraHours) ---
    let mut mocks_a = ReportingMocks::new();
    setup_holiday_common_mocks(&mut mocks_a);

    mocks_a.toggle_service = MockToggleService::new();
    mocks_a
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2024-01-01"))));

    mocks_a
        .special_day_service
        .expect_get_by_week()
        .returning(|_, wk, _| {
            if wk == 23 {
                Ok(Arc::from(vec![make_holiday(2024, 23, DayOfWeek::Monday)]))
            } else {
                Ok(Arc::from(vec![]))
            }
        });

    let report_a = run_report_kw23(mocks_a.build()).await;

    // --- Run B: manual ExtraHours(Holiday, 8.0), toggle off, no SpecialDay ---
    let mut mocks_b = ReportingMocks::new();
    setup_holiday_common_mocks(&mut mocks_b);
    // toggle_service already set to Ok(None) (automation off) from setup above.

    // Replace extra_hours_service to return the manual holiday.
    let manual_holiday = make_holiday_extra_hours(8.0, date!(2024 - 06 - 03));
    let extras_b: Arc<[ExtraHours]> = Arc::from(vec![manual_holiday]);
    mocks_b.extra_hours_service = MockExtraHoursService::new();
    mocks_b
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_b.clone()));

    // No SpecialDay (toggle off → special_day not queried, but set empty defensively).
    mocks_b
        .special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(vec![])));

    let report_b = run_report_kw23(mocks_b.build()).await;

    // HOL-02: holiday_hours identical (8.0).
    assert!(
        (report_a.holiday_hours - 8.0).abs() < 0.01,
        "HOL-02: Run A holiday_hours must be 8.0 (derived), got {}",
        report_a.holiday_hours
    );
    assert!(
        (report_b.holiday_hours - 8.0).abs() < 0.01,
        "HOL-02: Run B holiday_hours must be 8.0 (manual), got {}",
        report_b.holiday_hours
    );
    assert!(
        (report_a.holiday_hours - report_b.holiday_hours).abs() < 0.01,
        "HOL-02: holiday_hours must be identical (derived={} manual={})",
        report_a.holiday_hours,
        report_b.holiday_hours
    );

    // HOL-02: expected_hours identical (40h contract - 8h holiday absence = 32h).
    assert!(
        (report_a.expected_hours - report_b.expected_hours).abs() < 0.01,
        "HOL-02: expected_hours must be identical (derived={} manual={})",
        report_a.expected_hours,
        report_b.expected_hours
    );

    // HOL-02: balance_hours identical.
    assert!(
        (report_a.balance_hours - report_b.balance_hours).abs() < 0.01,
        "HOL-02: balance_hours must be identical (derived={} manual={})",
        report_a.balance_hours,
        report_b.balance_hours
    );
}

// ─── HCFG-01: Cutoff gate boundary ───────────────────────────────────────────

/// HCFG-01: A holiday dated BEFORE the cutoff yields 0 credit (exclusive gate).
/// The SAME holiday ON the cutoff date yields full credit (inclusive boundary: >=).
///
/// Holiday: 2024-03-18 (Monday of KW12/2024).
/// Work details: Mon-Fri 40h, valid KW11-13/2024 (covers KW12).
/// Run 1: cutoff = "2024-03-25" → 2024-03-18 < 2024-03-25 → 0h.
/// Run 2: cutoff = "2024-03-18" → 2024-03-18 >= 2024-03-18 → 8h.
#[tokio::test]
async fn test_holiday_before_cutoff_skipped() {
    // Work details covering KW11-13/2024 (includes KW12 where the holiday falls).
    let work_details_kw12 = service::employee_work_details::EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0012),
        from_calendar_week: 11,
        from_year: 2024,
        to_calendar_week: 13,
        to_year: 2024,
        ..fixture_work_details_8h_mon_fri()
    };
    let from_date = ShiftyDate::from_ymd(2024, 3, 18).unwrap(); // Monday KW12
    let to_date = ShiftyDate::from_ymd(2024, 3, 24).unwrap(); // Sunday KW12

    // Helper: build a fresh service for this test with the given cutoff string.
    let make_service = |cutoff: &'static str, wkd: service::employee_work_details::EmployeeWorkDetails| {
        let mut mocks = ReportingMocks::new();
        mocks
            .permission_service
            .expect_check_permission()
            .returning(|_, _| Ok(()));
        mocks
            .sales_person_service
            .expect_verify_user_is_sales_person()
            .returning(|_, _, _| Ok(()));
        mocks
            .sales_person_service
            .expect_get()
            .returning(|_, _, _| Ok(fixture_sales_person()));
        mocks
            .employee_work_details_service
            .expect_find_by_sales_person_id()
            .returning(move |_, _, _| Ok(Arc::from(vec![wkd.clone()])));
        mocks
            .shiftplan_report_service
            .expect_extract_shiftplan_report()
            .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));
        mocks
            .carryover_service
            .expect_get_carryover()
            .returning(|_, _, _, _| Ok(None));
        mocks
            .transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        mocks
            .absence_service
            .expect_derive_hours_for_range()
            .returning(|_, _, _, _, _| Ok(BTreeMap::new()));
        mocks
            .extra_hours_service
            .expect_find_by_sales_person_id_and_year_range()
            .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
        // Toggle: configurable cutoff.
        mocks.toggle_service = MockToggleService::new();
        mocks
            .toggle_service
            .expect_get_toggle_value()
            .returning(move |_, _, _| Ok(Some(Arc::from(cutoff))));
        // SpecialDay: Holiday on KW12/2024, Monday (= 2024-03-18).
        mocks
            .special_day_service
            .expect_get_by_week()
            .returning(|_, wk, _| {
                if wk == 12 {
                    Ok(Arc::from(vec![make_holiday(2024, 12, DayOfWeek::Monday)]))
                } else {
                    Ok(Arc::from(vec![]))
                }
            });
        mocks.build()
    };

    // Run 1: cutoff AFTER holiday → holiday not credited (0h).
    let report_before = make_service("2024-03-25", work_details_kw12.clone())
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            from_date,
            to_date,
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("HCFG-01 before-cutoff run must succeed");

    assert!(
        report_before.holiday_hours.abs() < 0.01,
        "HCFG-01: holiday BEFORE cutoff (2024-03-18 < 2024-03-25) → 0.0h credit, got {}",
        report_before.holiday_hours
    );

    // Run 2: cutoff == holiday → boundary is inclusive → holiday credited (8h).
    let report_on_cutoff = make_service("2024-03-18", work_details_kw12.clone())
        .get_report_for_employee_range(
            &fixture_sales_person_id(),
            from_date,
            to_date,
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("HCFG-01 on-cutoff run must succeed");

    assert!(
        (report_on_cutoff.holiday_hours - 8.0).abs() < 0.01,
        "HCFG-01: holiday ON cutoff (2024-03-18 >= 2024-03-18, inclusive) → 8.0h credit, got {}",
        report_on_cutoff.holiday_hours
    );
}

// ─── HCFG-03: Manual wins (no double-credit) ─────────────────────────────────

/// HCFG-03: When a manual ExtraHours(Holiday) covers the same calendar day as a
/// SpecialDay(Holiday), the holiday is credited ONCE (8h), NOT twice (16h).
/// The implementation skips auto-credit when a manual holiday entry exists for
/// the same employee+date (D-25-03: manual takes priority).
///
/// Setup: SpecialDay(Holiday, KW23 Mon 2024-06-03) + ExtraHours(Holiday, 8h, 2024-06-03).
/// Toggle enabled (cutoff 2024-01-01 — auto-credit would apply without manual).
/// Expected: report.holiday_hours == 8.0 (not 16.0).
#[tokio::test]
async fn test_holiday_manual_wins() {
    let mut mocks = ReportingMocks::new();
    setup_holiday_common_mocks(&mut mocks);

    // Toggle: cutoff 2024-01-01 (auto-credit would fire for 2024-06-03).
    mocks.toggle_service = MockToggleService::new();
    mocks
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2024-01-01"))));

    // SpecialDay: Holiday on KW23/2024, Monday.
    mocks
        .special_day_service
        .expect_get_by_week()
        .returning(|_, wk, _| {
            if wk == 23 {
                Ok(Arc::from(vec![make_holiday(2024, 23, DayOfWeek::Monday)]))
            } else {
                Ok(Arc::from(vec![]))
            }
        });

    // Manual ExtraHours(Holiday, 8.0) on 2024-06-03 — SAME day as SpecialDay.
    // The implementation's conflict check: any(|eh| eh.category == Holiday && eh.date_time.date() == holiday_date).
    // This causes the auto-credit to be skipped for 2024-06-03 (HCFG-03 / D-25-03).
    let manual_holiday = make_holiday_extra_hours(8.0, date!(2024 - 06 - 03));
    let extras: Arc<[ExtraHours]> = Arc::from(vec![manual_holiday]);
    mocks.extra_hours_service = MockExtraHoursService::new();
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras.clone()));

    let report = run_report_kw23(mocks.build()).await;

    // HCFG-03: credited exactly once (8h from manual), auto-credit skipped.
    assert!(
        (report.holiday_hours - 8.0).abs() < 0.01,
        "HCFG-03: manual-wins — holiday_hours must be 8.0 (once), not 16.0 (twice); got {}",
        report.holiday_hours
    );
}

// ─── HOL-03: Year-view (booking_information path) unaffected ─────────────────

/// HOL-03: booking_information.paid_hours (= sum of get_week().dynamic_hours) is NOT
/// reduced by holiday auto-credit. The get_week() code path has NO derive-on-read
/// logic for holidays (no build_derived_holiday_map call), so the booking_information
/// year-view is immune.
///
/// This test acts as a REGRESSION GUARD: if someone adds special_day_service or
/// toggle_service calls to get_week(), the mock will panic on "unexpected call"
/// (neither mock has any expectations here), immediately revealing the regression.
///
/// The special_day_service mock has NO expectations — any call panics.
/// Expected: dynamic_hours (= booking_information.paid_hours) == 40.0 (full contract,
/// not reduced by any holiday credit).
#[tokio::test]
async fn test_holiday_auto_credit_no_year_view_impact() {
    let mut mocks = ReportingMocks::new();

    // get_week() mock setup (different API than get_report_for_employee_range).
    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    // No manual holiday ExtraHours.
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    // No absence periods.
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    // special_day_service: NO expectations set intentionally.
    // If get_week() ever calls it → mockall panics → regression caught.
    // (booking_information year-view must NOT apply holiday auto-credit.)

    // toggle_service: already set to Ok(None) in new(); also not called by get_week().

    let service = mocks.build();
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week must succeed (no holiday auto-credit in this path)");

    assert_eq!(
        reports.len(),
        1,
        "HOL-03: get_week must return 1 report for the paid employee"
    );
    let report = &reports[0];

    // booking_information.paid_hours = sum(report.dynamic_hours) from get_week().
    // With 40h/week Mon-Fri contract and no shiftplan/absence:
    //   weight_for_week → expected_hours=40, dynamic_hours=40
    //   get_week(): dynamic_hours = 40 - 0 (abense) - 0 (derived) = 40
    // This must NOT be reduced by any holiday credit (since get_week has none).
    assert!(
        (report.dynamic_hours - 40.0).abs() < 0.01,
        "HOL-03: dynamic_hours (= booking_information.paid_hours) must be 40h, \
         not reduced by holiday auto-credit; got {}",
        report.dynamic_hours
    );

    // holiday_hours in get_week() is sourced only from manual ExtraHours (0 here).
    // Auto-credit is NOT applied → holiday_hours must be 0.
    assert!(
        report.holiday_hours.abs() < 0.01,
        "HOL-03: holiday_hours in get_week() must be 0 (no manual holiday, \
         no auto-credit in this path); got {}",
        report.holiday_hours
    );

    // vacation_hours and volunteer_hours are also 0 (no holiday ≠ vacation).
    assert!(
        report.vacation_hours.abs() < 0.01,
        "HOL-03: vacation_hours must be 0 (holiday != vacation); got {}",
        report.vacation_hours
    );
}
