//! Phase 56 Plan 02 Task 2 — Tests fuer `RebookingReconciliationService::backfill_range`
//! (REB-AUTO-04, D-05).
//!
//! Test-Achsen (die vier `<behavior>`-Faelle des Plans):
//! 1. `dry_run=true`: BackfillPreviewItem pro (gedeckelter SalesPerson, ISO-Woche),
//!    aber KEIN Write (RebookingBatch::create nie aufgerufen).
//! 2. `dry_run=false`: pro buchbarem Kandidaten ein AutoCronBackfill-Batch +
//!    Pair; skipped=false fuer gebuchte.
//! 3. Bereits verarbeitete Wochen (UNIQUE-Konflikt) → skipped=true,
//!    skip_reason=AlreadyProcessed (auch im apply-Modus).
//! 4. active_from-Gating: Wochen vor Stichtag → skipped=true, skip_reason=ToggleOff.
//!
//! Zusaetzlich: HR-Admin-Gate (T-56-04) und der ISO-Jahresgrenzfall (from=2026-W52
//! bis upper=2027-W01) walk-Korrektheit.

use std::sync::Arc;

use uuid::Uuid;

use dao::rebooking_batch::{RebookingBatchKind, RebookingBatchState};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::MockExtraHoursService;
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::{EmployeeReport, MockReportingService};
use service::toggle::MockToggleService;
use service::voluntary_stats::{MockVoluntaryStatsService, VoluntaryStats};
use service::week_status::{MockWeekStatusService, WeekStatus};
use service::MockPermissionService;
use service::ServiceError;
use shifty_utils::{DayOfWeek, ShiftyWeek};

use service::rebooking_reconciliation::{RebookingReconciliationService, SkipReason};

use crate::rebooking_reconciliation::{
    RebookingReconciliationServiceDeps, RebookingReconciliationServiceImpl,
};

struct TestDeps;
impl RebookingReconciliationServiceDeps for TestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type ExtraHoursService = MockExtraHoursService;
    type RebookingBatchService = MockRebookingBatchService;
    type ReportingService = MockReportingService;
    type VoluntaryStatsService = MockVoluntaryStatsService;
    type WeekStatusService = MockWeekStatusService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type ToggleService = MockToggleService;
    type ClockService = MockClockService;
    type PermissionService = MockPermissionService;
    type TransactionDao = dao::MockTransactionDao;
}

fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_3001)
}

/// Gedeckelter Vertrag (cap aktiv) fuer sp_id().
fn capped_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_3010),
        sales_person_id: sp_id(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
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
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Ungedeckelter Vertrag (cap inaktiv) — muss vom Backfill uebersprungen werden.
fn uncapped_work_details() -> EmployeeWorkDetails {
    let mut wd = capped_work_details();
    wd.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_3011);
    wd.sales_person_id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_3099);
    wd.cap_planned_hours_to_expected = false;
    wd
}

fn make_voluntary_stats(ist: f32, soll: f32) -> VoluntaryStats {
    VoluntaryStats {
        ist_per_contract_week: Some(ist),
        ist_total: Some(ist),
        soll_total: Some(soll),
        delta: Some(ist - soll),
        contract_weeks: Some(1),
        ist_per_soll_pct: Some(100.0),
    }
}

fn make_report(balance: f32) -> EmployeeReport {
    EmployeeReport {
        sales_person: Arc::new(service::sales_person::SalesPerson {
            id: sp_id(),
            name: Arc::from("Backfill 56"),
            background_color: Arc::from("#123456"),
            is_paid: Some(true),
            inactive: false,
            deleted: None,
            version: Uuid::nil(),
        }),
        balance_hours: balance,
        overall_hours: 0.0,
        expected_hours: 0.0,
        dynamic_hours: 0.0,
        shiftplan_hours: 0.0,
        extra_work_hours: 0.0,
        vacation_hours: 0.0,
        sick_leave_hours: 0.0,
        holiday_hours: 0.0,
        unpaid_leave_hours: 0.0,
        volunteer_hours: 0.0,
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

fn perm_ok() -> MockPermissionService {
    let mut p = MockPermissionService::new();
    p.expect_check_permission().returning(|_, _| Ok(()));
    p
}

fn perm_forbidden() -> MockPermissionService {
    let mut p = MockPermissionService::new();
    p.expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));
    p
}

fn tx_ok() -> dao::MockTransactionDao {
    let mut t = dao::MockTransactionDao::new();
    t.expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    t.expect_commit().returning(|_| Ok(()));
    t
}

/// Clock fixiert auf 2026-03-16 (Mo KW 12/2026) → upper bound = Vorwoche KW 11/2026.
fn clock_2026_w12() -> MockClockService {
    let mut c = MockClockService::new();
    c.expect_date_now()
        .returning(|| time::macros::date!(2026 - 03 - 16));
    c
}

/// Clock fixiert auf 2027-01-11 (Mo KW 02/2027) → upper bound = KW 01/2027.
/// Damit deckt ein from=2026-W52 den Jahresgrenz-Walk 2026-W52 → 2027-W01 ab.
fn clock_2027_w02() -> MockClockService {
    let mut c = MockClockService::new();
    c.expect_date_now()
        .returning(|| time::macros::date!(2027 - 01 - 11));
    c
}

fn toggle_from(raw: &'static str) -> MockToggleService {
    let mut t = MockToggleService::new();
    t.expect_get_toggle_value()
        .returning(move |_, _, _| Ok(Some(Arc::from(raw))));
    t
}

fn contracts(list: Vec<EmployeeWorkDetails>) -> MockEmployeeWorkDetailsService {
    let arc: Arc<[EmployeeWorkDetails]> = Arc::from(list);
    let mut e = MockEmployeeWorkDetailsService::new();
    e.expect_all_for_week()
        .returning(move |_, _, _, _| Ok(arc.clone()));
    e
}

fn stats_service(ist: f32, soll: f32) -> MockVoluntaryStatsService {
    let mut v = MockVoluntaryStatsService::new();
    v.expect_get_voluntary_stats()
        .returning(move |_, _, _, _, _| Ok(make_voluntary_stats(ist, soll)));
    v
}

fn reporting_service() -> MockReportingService {
    let mut r = MockReportingService::new();
    r.expect_get_report_for_employee_range()
        .returning(|_, _, _, _, _, _| Ok(make_report(-8.0)));
    r
}

fn week_unset() -> MockWeekStatusService {
    let mut w = MockWeekStatusService::new();
    w.expect_get_week_status()
        .returning(|_, _, _, _| Ok(WeekStatus::Unset));
    w
}

#[allow(clippy::too_many_arguments)]
fn build_service(
    extra_hours_service: MockExtraHoursService,
    rebooking_batch_service: MockRebookingBatchService,
    reporting_service: MockReportingService,
    voluntary_stats_service: MockVoluntaryStatsService,
    week_status_service: MockWeekStatusService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    toggle_service: MockToggleService,
    clock_service: MockClockService,
    permission_service: MockPermissionService,
    transaction_dao: dao::MockTransactionDao,
) -> RebookingReconciliationServiceImpl<TestDeps> {
    RebookingReconciliationServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        rebooking_batch_service: Arc::new(rebooking_batch_service),
        reporting_service: Arc::new(reporting_service),
        voluntary_stats_service: Arc::new(voluntary_stats_service),
        week_status_service: Arc::new(week_status_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        toggle_service: Arc::new(toggle_service),
        clock_service: Arc::new(clock_service),
        permission_service: Arc::new(permission_service),
        transaction_dao: Arc::new(transaction_dao),
    }
}

// ─── Case: HR-Admin-Gate (T-56-04) ────────────────────────────────────────────

/// T-56-04: Non-HR → Forbidden, kein Datenabruf.
#[tokio::test]
async fn backfill_range_is_hr_gated() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        MockReportingService::new(),
        MockVoluntaryStatsService::new(),
        MockWeekStatusService::new(),
        MockEmployeeWorkDetailsService::new(),
        MockToggleService::new(),
        MockClockService::new(),
        perm_forbidden(),
        dao::MockTransactionDao::new(),
    );

    let result = service
        .backfill_range(
            ShiftyWeek::new(2026, 10),
            true,
            Authentication::Context(()),
            None,
        )
        .await;
    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "non-HR backfill must be Forbidden, got {result:?}"
    );
}

// ─── Case 1: dry_run writes nothing ───────────────────────────────────────────

/// REB-AUTO-04 (D-05): dry_run=true erzeugt Preview-Items, aber RebookingBatch::create
/// wird NIE aufgerufen (kein Write). from=KW 10, upper=KW 11 (clock=KW12) → 2 Wochen.
#[tokio::test]
async fn backfill_dry_run_writes_nothing() {
    // RebookingBatchService::create OHNE Expect → jeder Aufruf paniked (Beweis:
    // im dry_run wird nicht geschrieben).
    let rebooking_batch_service = MockRebookingBatchService::new();
    // ExtraHours::create ebenso nicht.
    let extra_hours_service = MockExtraHoursService::new();

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(),
        stats_service(8.0, 5.0), // Ist=8 > committed=5 → excess=3, would_book
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w12(),
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 10),
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("dry-run backfill must succeed");

    // from=10..=upper(11) → genau 2 Kandidaten-Items (1 capped person je Woche).
    assert_eq!(items.len(), 2, "two weeks (KW10, KW11) x one capped person");
    for item in &items {
        assert!((item.excess - 3.0).abs() < 1e-6, "excess = 8-5 = 3");
        assert!(item.would_book, "would_book in dry-run preview");
        assert_eq!(item.sales_person_id, sp_id());
    }
}

/// Ungedeckelte Personen werden ignoriert (nur cap_planned_hours_to_expected=true).
#[tokio::test]
async fn backfill_ignores_uncapped_persons() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(),
        stats_service(8.0, 5.0),
        week_unset(),
        // capped + uncapped im selben Vertrags-Set; nur capped zaehlt.
        contracts(vec![capped_work_details(), uncapped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w12(),
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 11),
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("backfill must succeed");
    // Nur KW 11 (upper), nur die capped person.
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].sales_person_id, sp_id());
}

// ─── Case 2: apply persists AutoCronBackfill ──────────────────────────────────

/// REB-AUTO-04: dry_run=false persistiert pro buchbarem Kandidaten einen
/// AutoCronBackfill-Batch + Pair; skipped=false.
#[tokio::test]
async fn backfill_apply_persists_autocron_backfill() {
    let batch_kinds: Arc<std::sync::Mutex<Vec<RebookingBatchKind>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let kinds_cap = batch_kinds.clone();
    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .returning(move |batch, _, _, _| {
            assert_eq!(batch.state, RebookingBatchState::Approved);
            kinds_cap.lock().unwrap().push(batch.kind);
            Ok(batch.clone())
        });

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_create()
        .returning(|eh, _, _| Ok(eh.clone()));

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(),
        stats_service(8.0, 5.0),
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w12(), // upper = KW 11
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 11),
            false, // apply
            Authentication::Context(()),
            None,
        )
        .await
        .expect("apply backfill must succeed");

    assert_eq!(items.len(), 1, "one week (KW11)");
    assert!(!items[0].skipped, "booked candidate must be skipped=false");
    let kinds = batch_kinds.lock().unwrap();
    assert_eq!(kinds.len(), 1, "exactly one batch persisted");
    assert_eq!(
        kinds[0],
        RebookingBatchKind::AutoCronBackfill,
        "apply must persist AutoCronBackfill batches (single write-path via run_for_week)"
    );
}

// ─── Case 3: already-processed surfaces as AlreadyProcessed ────────────────────

/// REB-AUTO-04: eine bereits verarbeitete Woche (UNIQUE-Konflikt beim Batch-create
/// in run_for_week) → skipped=true, skip_reason=AlreadyProcessed.
#[tokio::test]
async fn backfill_apply_already_processed_skips() {
    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .returning(|batch, _, _, _| Err(ServiceError::EntityAlreadyExists(batch.id)));

    // ExtraHours::create darf NICHT laufen (Batch-Claim kollidiert zuerst).
    let extra_hours_service = MockExtraHoursService::new();

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(),
        stats_service(8.0, 5.0),
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w12(),
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 11),
            false,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("backfill must succeed even when weeks already processed");

    assert_eq!(items.len(), 1);
    assert!(items[0].skipped, "already-processed week is skipped");
    assert_eq!(items[0].skip_reason, Some(SkipReason::AlreadyProcessed));
}

// ─── Case 4: active_from gating ───────────────────────────────────────────────

/// REB-AUTO-04: Wochen vor active_from → skipped=true, skip_reason=ToggleOff,
/// kein Write (auch im apply-Modus).
#[tokio::test]
async fn backfill_before_active_from_is_toggle_off() {
    // create darf NICHT laufen (alle Wochen vor Stichtag).
    let rebooking_batch_service = MockRebookingBatchService::new();
    let extra_hours_service = MockExtraHoursService::new();

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(),
        stats_service(8.0, 5.0),
        week_unset(),
        contracts(vec![capped_work_details()]),
        // active_from weit in der Zukunft → alle Backfill-Wochen liegen davor.
        toggle_from("2026-12-01"),
        clock_2026_w12(), // upper = KW 11 (Maerz 2026), vor 2026-12-01
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 10),
            false, // apply, aber gate blockt jeden Write
            Authentication::Context(()),
            None,
        )
        .await
        .expect("backfill must succeed");

    assert_eq!(items.len(), 2, "KW10, KW11");
    for item in &items {
        assert!(item.skipped, "pre-cutoff week is skipped");
        assert_eq!(item.skip_reason, Some(SkipReason::ToggleOff));
        assert!(!item.would_book, "would_book false before cutoff");
    }
}

// ─── Year-boundary walk ────────────────────────────────────────────────────────

/// REB-AUTO-04: from=2026-W52, clock=2027-W02 → upper=2027-W01. Der Walk kreuzt
/// die Jahresgrenze korrekt: 2026-W52, 2026-W53, 2027-W01 (2026 hat KW 53).
#[tokio::test]
async fn backfill_year_boundary_walk() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(),
        stats_service(5.0, 5.0), // kein Excess → nur Preview-Items, kein Write
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2027_w02(), // upper = KW 01/2027
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2026, 52),
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("year-boundary backfill must succeed");

    // 2026-W52, 2026-W53, 2027-W01 → 3 Wochen (2026 hat ISO-KW 53).
    let weeks: Vec<(u32, u8)> = items.iter().map(|i| (i.iso_year, i.iso_week)).collect();
    assert_eq!(
        weeks,
        vec![(2026, 52), (2026, 53), (2027, 1)],
        "walk must cross the year boundary via ShiftyWeek arithmetic (2026 has W53)"
    );
    for item in &items {
        // Ist=5, committed=5 → excess=0 → NoExcess skip.
        assert!(item.skipped);
        assert_eq!(item.skip_reason, Some(SkipReason::NoExcess));
    }
}
