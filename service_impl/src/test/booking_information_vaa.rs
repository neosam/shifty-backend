//! VAA-03 backend test: `sales_person_absences` in `get_weekly_summary`.
//!
//! Three assertions verify D-53-01/02/03/05 (Phase 53):
//!  1. `vaa03_volunteer_with_period_appears_with_correct_hours` — a volunteer with an active
//!     `AbsencePeriod` overlapping `WEEK_UNDER_TEST` appears in `sales_person_absences` with
//!     `hours == committed_voluntary` (cap-gated, D-53-02).
//!  2. `vaa03_volunteer_without_period_absent_not_in_list` — a volunteer without an active
//!     period is NOT in `sales_person_absences` for that week (visibility = exact
//!     `absent_volunteer_ids`, D-53-03).
//!  3. `vaa03_paid_employee_unchanged_regression_lock` — a paid employee stays in
//!     `working_hours_per_sales_person` AND is NOT leaked into `sales_person_absences`
//!     (Regression-Lock VAA-03 #3, Pitfall 6).
//!
//! Test template is a 1:1 copy of `booking_information_vfa.rs` (Mock setup, TestDeps,
//! service construction). Only Sales-Persons + AbsencePeriods + reporting differ.

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod, DayFraction, MockAbsenceService};
use service::booking::MockBookingService;
use service::booking_information::BookingInformationService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::permission::Authentication;
use service::reporting::{MockReportingService, ShortEmployeeReport};
use service::sales_person::{MockSalesPersonService, SalesPerson};
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::shiftplan_report::MockShiftplanReportService;
use service::slot::MockSlotService;
use service::special_days::MockSpecialDayService;
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

// ─── Fixture constants ────────────────────────────────────────────────────────

const YEAR: u32 = 2026;
/// 2026-W20 = Mon May 11 .. Sun May 17.
const WEEK_UNDER_TEST: u8 = 20;

/// Volunteer WITH an `AbsencePeriod` overlapping `WEEK_UNDER_TEST`.
/// UUID last block encodes "0053_0001" (phase 53, seq 1) for grep-ability.
fn volunteer_id_absent() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0053_0001)
}

/// Volunteer WITHOUT an `AbsencePeriod` in `WEEK_UNDER_TEST`.
fn volunteer_id_present() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0053_0002)
}

/// Paid employee (regression-lock VAA-03 #3).
fn paid_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0053_0003)
}

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn sales_person(id: Uuid, name: &'static str, is_paid: bool) -> SalesPerson {
    SalesPerson {
        id,
        name: Arc::from(name),
        background_color: Arc::from("#123456"),
        is_paid: Some(is_paid),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

fn work_details(sales_person_id: Uuid) -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id,
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

fn absence_period(sales_person_id: Uuid, seq: u128) -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_AB53_0000u128 + seq),
        sales_person_id,
        category: AbsenceCategory::Vacation,
        from_date: time::Date::from_iso_week_date(2026, WEEK_UNDER_TEST, time::Weekday::Monday)
            .expect("2026-W20-Mon is a valid ISO week date"),
        to_date: time::Date::from_iso_week_date(2026, WEEK_UNDER_TEST, time::Weekday::Sunday)
            .expect("2026-W20-Sun is a valid ISO week date"),
        description: Arc::from("VAA-03 test period"),
        created: Some(datetime!(2026 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    }
}

// ─── Shared test harness ──────────────────────────────────────────────────────

/// Build the full `BookingInformationServiceImpl` with the shared 3-person population
/// (`volunteer_absent`, `volunteer_present`, `paid`), all work_details, and the given
/// `absence_periods`. `paid_report_for_week` toggles whether the paid employee gets a
/// `ShortEmployeeReport` row for `WEEK_UNDER_TEST` (needed for the regression-lock test).
fn build_service(
    all_persons: Vec<SalesPerson>,
    absence_periods: Vec<AbsencePeriod>,
    paid_report_for_week: Option<Arc<SalesPerson>>,
) -> BookingInformationServiceImpl<TestDeps> {
    // ── permission_service ──
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    // ── sales_person_service ──
    let persons_arc: Arc<[SalesPerson]> = Arc::from(all_persons.clone());
    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(persons_arc.clone()));

    // ── employee_work_details_service ──
    let work_details_vec: Vec<EmployeeWorkDetails> =
        all_persons.iter().map(|sp| work_details(sp.id)).collect();
    let work_details_arc: Arc<[EmployeeWorkDetails]> = Arc::from(work_details_vec);
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(work_details_arc.clone()));

    // ── absence_service ──
    let absence_arc: Arc<[AbsencePeriod]> = Arc::from(absence_periods);
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_all()
        .returning(move |_, _| Ok(absence_arc.clone()));

    // ── toggle_service ──
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    // ── special_day_service ──
    let mut special_day_service = MockSpecialDayService::new();
    special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(vec![])));
    special_day_service
        .expect_get_by_iso_year()
        .returning(|_, _| Ok(Arc::from(vec![])));

    // ── reporting_service: empty per-week reports; optionally inject a paid-report row for W20 ──
    let paid_arc_opt = paid_report_for_week.clone();
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    reporting_service
        .expect_get_year()
        .returning(move |year, _, _| {
            let weeks_in_year = time::util::weeks_in_year(year as i32);
            let out: Vec<(u8, Arc<[ShortEmployeeReport]>)> = (1..=weeks_in_year)
                .map(|w| {
                    if year == YEAR && w == WEEK_UNDER_TEST {
                        if let Some(paid_arc) = paid_arc_opt.clone() {
                            let report = ShortEmployeeReport {
                                sales_person: paid_arc,
                                balance_hours: 0.0,
                                dynamic_hours: 0.0,
                                expected_hours: 8.0,
                                overall_hours: 0.0,
                                vacation_hours: 0.0,
                                sick_leave_hours: 0.0,
                                holiday_hours: 0.0,
                                unavailable_hours: 0.0,
                                unpaid_leave_hours: 0.0,
                                volunteer_hours: 0.0,
                                custom_absence_hours: Arc::from(Vec::new()),
                                has_pending_rebooking: false,
                                pending_rebooking_id: None,
                            };
                            (w, Arc::from(vec![report]))
                        } else {
                            (w, Arc::from(Vec::new()))
                        }
                    } else {
                        (w, Arc::from(Vec::new()))
                    }
                })
                .collect();
            Ok(Arc::from(out))
        });

    // ── shiftplan_report_service ──
    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    shiftplan_report_service
        .expect_extract_shiftplan_report_for_iso_year()
        .returning(|_, _, _| Ok(Arc::from(vec![])));

    // ── slot_service ──
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slots_for_week_all_plans()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    slot_service
        .expect_get_slots()
        .returning(|_, _| Ok(Arc::from(vec![])));

    // ── shiftplan_service ──
    let mut shiftplan_service_mock = service::shiftplan_catalog::MockShiftplanService::new();
    shiftplan_service_mock
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(Vec::<service::shiftplan_catalog::Shiftplan>::new())));

    // ── transaction_dao ──
    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    transaction_dao
        .expect_commit()
        .returning(|_| Ok(()));

    BookingInformationServiceImpl::<TestDeps> {
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        slot_service: Arc::new(slot_service),
        shiftplan_service: Arc::new(shiftplan_service_mock),
        booking_service: Arc::new(MockBookingService::new()),
        sales_person_service: Arc::new(sales_person_service),
        sales_person_unavailable_service: Arc::new(MockSalesPersonUnavailableService::new()),
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

// ─── Tests ────────────────────────────────────────────────────────────────────

/// VAA-01 + VAA-02 (D-53-01/02): the absent volunteer is in the DTO field
/// and `hours` equals cap-gated `committed_voluntary` (== 5.0).
#[tokio::test]
async fn vaa03_volunteer_with_period_appears_with_correct_hours() {
    let vol_absent = sales_person(volunteer_id_absent(), "Volunteer Absent", false);
    let vol_present = sales_person(volunteer_id_present(), "Volunteer Present", false);
    let paid = sales_person(paid_id(), "Paid Employee", true);
    let persons = vec![vol_absent.clone(), vol_present.clone(), paid.clone()];
    let periods = vec![absence_period(volunteer_id_absent(), 1)];

    let service = build_service(persons, periods, None);
    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");
    let week = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK_UNDER_TEST)
        .expect("WEEK_UNDER_TEST must be present in year-view summary");

    assert!(
        week.sales_person_absences
            .iter()
            .any(|a| a.sales_person_id == volunteer_id_absent()),
        "VAA-01: absent volunteer must appear in sales_person_absences for the covered week"
    );
    let entry = week
        .sales_person_absences
        .iter()
        .find(|a| a.sales_person_id == volunteer_id_absent())
        .expect("absent volunteer entry must be findable");
    assert!(
        (entry.hours - 5.0).abs() < 0.001,
        "VAA-02: hours must equal cap-gated committed_voluntary (5.0), got {}",
        entry.hours
    );
}

/// VAA-03 #2 (D-53-03): a volunteer without an active period must NOT appear
/// in `sales_person_absences` for the week under test.
#[tokio::test]
async fn vaa03_volunteer_without_period_absent_not_in_list() {
    let vol_absent = sales_person(volunteer_id_absent(), "Volunteer Absent", false);
    let vol_present = sales_person(volunteer_id_present(), "Volunteer Present", false);
    let paid = sales_person(paid_id(), "Paid Employee", true);
    let persons = vec![vol_absent.clone(), vol_present.clone(), paid.clone()];
    // Only the absent volunteer has a period; the present volunteer has none.
    let periods = vec![absence_period(volunteer_id_absent(), 1)];

    let service = build_service(persons, periods, None);
    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");
    let week = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK_UNDER_TEST)
        .expect("WEEK_UNDER_TEST must be present in year-view summary");

    assert!(
        !week
            .sales_person_absences
            .iter()
            .any(|a| a.sales_person_id == volunteer_id_present()),
        "VAA-03 #2: volunteer without an active period must NOT appear in sales_person_absences"
    );
}

/// VAA-03 #3 (Pitfall 6, Regression-Lock): a paid employee with an active
/// `AbsencePeriod` must NOT leak into `sales_person_absences`, but MUST remain
/// present in `working_hours_per_sales_person` (bezahlten-only Vertrag).
#[tokio::test]
async fn vaa03_paid_employee_unchanged_regression_lock() {
    let vol_absent = sales_person(volunteer_id_absent(), "Volunteer Absent", false);
    let vol_present = sales_person(volunteer_id_present(), "Volunteer Present", false);
    let paid = sales_person(paid_id(), "Paid Employee", true);
    let paid_arc = Arc::new(paid.clone());
    let persons = vec![vol_absent.clone(), vol_present.clone(), paid.clone()];
    // Both a volunteer AND the paid employee have absence periods — the paid one
    // must be filtered out of sales_person_absences by the volunteer_ids gate.
    let periods = vec![
        absence_period(volunteer_id_absent(), 1),
        absence_period(paid_id(), 2),
    ];

    let service = build_service(persons, periods, Some(paid_arc));
    let summaries = service
        .get_weekly_summary(YEAR, Authentication::Full, None)
        .await
        .expect("get_weekly_summary must succeed");
    let week = summaries
        .iter()
        .find(|s| s.year == YEAR && s.week == WEEK_UNDER_TEST)
        .expect("WEEK_UNDER_TEST must be present in year-view summary");

    assert!(
        !week
            .sales_person_absences
            .iter()
            .any(|a| a.sales_person_id == paid_id()),
        "VAA-03 #3 / Pitfall 6: paid employee must NOT leak into sales_person_absences"
    );
    assert!(
        week.working_hours_per_sales_person
            .iter()
            .any(|wh| wh.sales_person_id == paid_id()),
        "VAA-03 #3 Regression-Lock: paid employee must remain in working_hours_per_sales_person"
    );
}
