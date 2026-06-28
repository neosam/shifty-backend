//! VFA-02 full-service regression test: `get_weekly_summary` holiday-vs-absence asymmetry.
//!
//! Proves D-26-04: a `SpecialDayType::Holiday` in a volunteer's week does NOT reduce
//! `committed_voluntary_hours`, while an `AbsencePeriod` (any category) DOES reduce it to 0
//! for the whole week (D-26-03 whole-week-out). Both are exercised for the same volunteer in
//! one test context to make the asymmetry explicit and regression-proof.
//!
//! Also asserts D-26-02: `CURRENT_SNAPSHOT_SCHEMA_VERSION` remains 11 — VFA-01/VFA-02 change
//! only the live, non-persisted year-view; no persisted `BillingPeriodValueType` is added or
//! changed, so no schema bump is required or expected.
//!
//! Test structure:
//! - `TestDeps` implements `BookingInformationServiceDeps` over Mock* types.
//! - `vfa02_holiday_vs_absence_asymmetry` builds the full service + calls `get_weekly_summary`
//!   for year 2026, then asserts on HOLIDAY_WEEK (W15) and ABSENCE_WEEK (W20).
//! - `phase26_vfa_no_snapshot_bump` is a zero-cost const assertion of the schema version.

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod, DayFraction, MockAbsenceService};
use service::booking::MockBookingService;
use service::booking_information::BookingInformationService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::permission::Authentication;
use service::reporting::MockReportingService;
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::shiftplan_report::MockShiftplanReportService;
use service::slot::MockSlotService;
use service::special_days::{MockSpecialDayService, SpecialDay, SpecialDayType};
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
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type AbsenceService = MockAbsenceService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = dao::MockTransactionDao;
}

// ─── Fixture constants ────────────────────────────────────────────────────────

/// Year under test.
const YEAR: u32 = 2026;

/// 2026-W15 (Mon Apr 6 – Sun Apr 12): contains a `SpecialDayType::Holiday`.
/// Per D-26-04 / VFA-02, a holiday must NOT reduce `committed_voluntary_hours`.
const HOLIDAY_WEEK: u8 = 15;

/// 2026-W20 (Mon May 11 – Sun May 17): covered by the volunteer's `AbsencePeriod`.
/// Per D-26-03 / VFA-01, an absence must reduce `committed_voluntary_hours` to 0.
const ABSENCE_WEEK: u8 = 20;

// ─── Fixture helpers ──────────────────────────────────────────────────────────

/// Deterministic volunteer UUID (is_paid=false, committed_voluntary > 0).
fn volunteer_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_CAFE_0042)
}

/// Volunteer `SalesPerson` fixture: `is_paid = Some(false)`.
fn volunteer_sales_person() -> SalesPerson {
    SalesPerson {
        id: volunteer_id(),
        name: Arc::from("VFA-02 Test Volunteer"),
        background_color: Arc::from("#00FF00"),
        is_paid: Some(false),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// `EmployeeWorkDetails` for the volunteer:
/// - `cap_planned_hours_to_expected = true` — satisfies the D-05 Band-1 gate
/// - `committed_voluntary = 5.0` — the pledge under test
/// - valid from 2026-W01 to 2027-W03, covering all 52 weeks of 2026 plus the 3 overflow
///   weeks that `get_weekly_summary` iterates into the following year.
fn volunteer_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id: volunteer_id(),
        expected_hours: 8.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 3,
        to_year: 2027,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: true,
        committed_voluntary: 5.0,
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

/// `AbsencePeriod` (Vacation) spanning exactly `ABSENCE_WEEK` (2026-W20: Mon May 11 – Sun May 17).
/// Category is `Vacation` — D-26-01 states category is NOT consulted by `period_overlaps_week`;
/// any of the three categories (Vacation / SickLeave / UnpaidLeave) would behave identically.
fn volunteer_absence_period() -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_AB5E_0001),
        sales_person_id: volunteer_id(),
        category: AbsenceCategory::Vacation,
        from_date: time::Date::from_iso_week_date(2026, 20, time::Weekday::Monday)
            .expect("2026-W20-Mon is a valid ISO week date"),
        to_date: time::Date::from_iso_week_date(2026, 20, time::Weekday::Sunday)
            .expect("2026-W20-Sun is a valid ISO week date"),
        description: Arc::from("VFA-02 test vacation"),
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    }
}

/// `SpecialDay` (Holiday) placed in `HOLIDAY_WEEK` (2026-W15).
/// The specific day_of_week is irrelevant for `committed_voluntary_hours` — the holiday only
/// affects slot filtering, and slots are empty in this test (isolating the committed band).
fn holiday_special_day() -> SpecialDay {
    SpecialDay {
        id: Uuid::nil(),
        year: 2026,
        calendar_week: 15,
        day_of_week: DayOfWeek::Monday,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

// ─── Test 1: VFA-02 holiday-vs-absence asymmetry ──────────────────────────────

/// VFA-02 / D-26-04 regression test: the same volunteer appears in two weeks.
///
/// HOLIDAY_WEEK (W15): `special_day_service` returns a `Holiday`; `absence_service.find_all`
/// returns only the W20 absence (no overlap) → `absent_volunteer_ids` is empty for W15 →
/// `committed_voluntary_hours` = 5.0 (unchanged).
///
/// ABSENCE_WEEK (W20): `special_day_service` returns nothing; `absence_service.find_all`
/// returns the W20 absence (overlaps) → volunteer is in `absent_volunteer_ids` →
/// `committed_voluntary_hours` = 0.0 (whole-week-out per D-26-03).
///
/// A third assertion names the asymmetry explicitly so any future coupling of holiday
/// handling to the committed band immediately fails this test.
#[tokio::test]
async fn vfa02_holiday_vs_absence_asymmetry() {
    let volunteer = volunteer_sales_person();
    let work_details = volunteer_work_details();
    let absence = volunteer_absence_period();
    let holiday = holiday_special_day();

    // ── permission_service: Ok for SHIFTPLANNER + SALES + is_shiftplanner check ──
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    // ── sales_person_service: one volunteer (is_paid=false) ──
    let volunteer_clone = volunteer.clone();
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(Arc::from(vec![volunteer_clone.clone()])));

    // ── employee_work_details_service: volunteer row valid all 2026 + overflow ──
    let work_details_clone = work_details.clone();
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(Arc::from(vec![work_details_clone.clone()])));

    // ── absence_service: one AbsencePeriod (W20 only, not overlapping W15) ──
    let absence_clone = absence.clone();
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(move |_, _| Ok(Arc::from(vec![absence_clone.clone()])));

    // ── special_day_service: Holiday returned ONLY for HOLIDAY_WEEK; empty otherwise ──
    // D-26-04: the holiday code path does NOT touch committed_voluntary_hours (only slots).
    let holiday_clone = holiday.clone();
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(move |year, week, _| {
            if year == YEAR && week == HOLIDAY_WEEK {
                Ok(Arc::from(vec![holiday_clone.clone()]))
            } else {
                Ok(Arc::from(vec![]))
            }
        });

    // ── reporting_service: empty week reports (isolates committed_voluntary band) ──
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));

    // ── shiftplan_report_service: no actuals (isolates Band-1 committed pledge) ──
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));

    // ── slot_service: no slots (removes slot_hours from the equation) ──
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));

    // ── transaction_dao: passthrough ──
    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao
        .expect_commit()
        .returning(|_| Ok(()));

    // ── Build BookingInformationServiceImpl ──
    // BookingService and SalesPersonUnavailableService are not called by get_weekly_summary;
    // no expectations are set on them (mockall panics only if an expectation is unmet, not
    // if the mock is never called without expectations).
    let service = BookingInformationServiceImpl::<TestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
        booking_service: Arc::new(MockBookingService::new()),
        sales_person_service: Arc::new(sales_person_service),
        sales_person_unavailable_service: Arc::new(MockSalesPersonUnavailableService::new()),
        reporting_service: Arc::new(reporting_service),
        special_day_service: Arc::new(special_day_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        absence_service: Arc::new(absence_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        transaction_dao: Arc::new(transaction_dao),
    };

    // ── Call get_weekly_summary for YEAR ──
    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed for VFA-02 asymmetry test");

    // ── Locate HOLIDAY_WEEK and ABSENCE_WEEK entries ──
    let holiday_week = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == HOLIDAY_WEEK)
        .expect("HOLIDAY_WEEK (2026-W15) must be present in the year-view summary");
    let absence_week = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == ABSENCE_WEEK)
        .expect("ABSENCE_WEEK (2026-W20) must be present in the year-view summary");

    // ── D-26-04 / VFA-02: Holiday MUST NOT reduce committed_voluntary_hours ──
    assert!(
        (holiday_week.committed_voluntary_hours - 5.0).abs() < 0.001,
        "D-26-04 VFA-02: HOLIDAY_WEEK (W{}) must NOT reduce committed_voluntary_hours; \
         expected 5.0 (full pledge unchanged), got {}. \
         A holiday special_day affects only slot filtering, not the committed band.",
        HOLIDAY_WEEK,
        holiday_week.committed_voluntary_hours
    );

    // ── D-26-03 / VFA-01: Absence MUST reduce committed_voluntary_hours to 0 ──
    assert!(
        absence_week.committed_voluntary_hours.abs() < 0.001,
        "D-26-03 VFA-01: ABSENCE_WEEK (W{}) MUST reduce committed_voluntary_hours to 0.0 \
         (whole-week-out — any overlap of Mon–Sun → full exclusion, not pro-rated); \
         got {}",
        ABSENCE_WEEK,
        absence_week.committed_voluntary_hours
    );

    // ── VFA-02 asymmetry: explicit contrast between the two behaviours ──
    assert!(
        holiday_week.committed_voluntary_hours > absence_week.committed_voluntary_hours + 0.001,
        "VFA-02 asymmetry (D-26-04): holiday-week committed ({}) must exceed absence-week \
         committed ({}) — a Holiday special_day does NOT reduce the committed pledge \
         (holiday-only path touches slot filtering); an AbsencePeriod DOES reduce it to 0 \
         (absent_volunteer_ids exclusion). If this fails, the two code paths have been \
         accidentally coupled.",
        holiday_week.committed_voluntary_hours,
        absence_week.committed_voluntary_hours
    );
}

// ─── Test 2: no snapshot-schema bump (D-26-02) ───────────────────────────────

/// Phase-26 snapshot-schema regression guard (D-26-02).
///
/// `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUST remain 11 after VFA-01/VFA-02.
///
/// Rationale (CLAUDE.md bump rule): a bump is required when a new persisted
/// `BillingPeriodValueType` is added, removed, or renamed, OR when the computation that
/// produces an existing value_type changes. VFA-01/VFA-02 affect only `get_weekly_summary`,
/// which is a live, non-persisted year-view (Achse-B). No `BillingPeriodValueType` row is
/// added or changed → no bump.
///
/// This assertion is independent of `snapshot_schema_version_pinned_at_10` in
/// `test/booking_information.rs` — that test covers the Phase-15..25 history; this one is a
/// dedicated Phase-26 CI guard that will catch any accidental bump introduced by VFA work.
#[test]
fn phase26_vfa_no_snapshot_bump() {
    assert_eq!(
        crate::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION,
        11,
        "D-26-02: VFA (Phase 26) must NOT bump CURRENT_SNAPSHOT_SCHEMA_VERSION. \
         get_weekly_summary changes are live-view-only (Achse-B, not persisted). \
         If this fails, a Phase-26 change accidentally added/changed a persisted \
         BillingPeriodValueType and the version must be justified and bumped intentionally."
    );
}
