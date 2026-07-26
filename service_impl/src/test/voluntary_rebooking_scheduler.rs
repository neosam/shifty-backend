//! Phase 56 Plan 03 Task 3 — Tests fuer
//! `VoluntaryRebookingScheduler::run_once_now` (REB-AUTO-01).
//!
//! Der Scheduler ist duenne Orchestrierung: er berechnet die Vorwoche und
//! delegiert pro gedeckelter `SalesPerson` an den echten
//! `RebookingReconciliationService::run_for_week`. Diese Tests bauen daher den
//! ECHTEN `RebookingReconciliationServiceImpl` (mit gemockten DAO-/Service-
//! Deps) und verdrahten ihn in den Scheduler — so wird die Delegation
//! `run_once_now → run_for_week` real durchlaufen (kein Mock des Motors).
//!
//! Test-Achsen:
//! 1. active_from gesetzt + gedeckelte Person mit Excess in der Vorwoche →
//!    genau EIN AutoCron-Batch + EIN source=Rebooking-Pair fuer die Vorwoche.
//! 2. active_from=None → voller No-op (kein Batch, kein Pair — jede Person
//!    Skipped{ToggleOff}).
//! 3. Deterministische Clock beweist: die verarbeitete Woche == letzte
//!    abgeschlossene ISO-Woche (Vorwoche) fuer das Clock-Datum. Der tokio-cron-
//!    Timer wird NICHT gefeuert (out of scope fuer cargo test) — `run_once_now`
//!    wird direkt aufgerufen.

use std::sync::Arc;

use time::macros::date;
use uuid::Uuid;

use dao::rebooking_batch::{RebookingBatchEntity, RebookingBatchKind, RebookingBatchState};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::MockReportingService;
use service::toggle::MockToggleService;
use service::voluntary_stats::{MockVoluntaryStatsService, VoluntaryStats};
use service::week_status::{MockWeekStatusService, WeekStatus};
use service::MockPermissionService;

use service::voluntary_rebooking_scheduler::VoluntaryRebookingScheduler;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::rebooking_reconciliation::{
    RebookingReconciliationServiceDeps, RebookingReconciliationServiceImpl,
};
use crate::voluntary_rebooking_scheduler::{
    VoluntaryRebookingSchedulerDeps, VoluntaryRebookingSchedulerImpl,
};

// ─── Deps-Marker ─────────────────────────────────────────────────────────────

struct ReconTestDeps;
impl RebookingReconciliationServiceDeps for ReconTestDeps {
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

struct SchedTestDeps;
impl VoluntaryRebookingSchedulerDeps for SchedTestDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type RebookingReconciliationService = RebookingReconciliationServiceImpl<ReconTestDeps>;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type TransactionDao = dao::MockTransactionDao;
}

// ─── Fixtures ────────────────────────────────────────────────────────────────

fn sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_3001)
}

/// A capped SalesPerson contract fixture (only the fields the scheduler cares
/// about — `sales_person_id` + `cap_planned_hours_to_expected` — are load-
/// bearing; the rest are inert defaults).
fn capped_contract() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::nil(),
        sales_person_id: sp_id(),
        expected_hours: 20.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 1,
        from_year: 2020,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 52,
        to_year: 2999,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: true,
        committed_voluntary: 0.0,
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

fn perm_full_ok() -> MockPermissionService {
    let mut p = MockPermissionService::new();
    p.expect_check_only_full_authentication()
        .returning(|_| Ok(()));
    p
}

fn tx_ok() -> dao::MockTransactionDao {
    let mut t = dao::MockTransactionDao::new();
    t.expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    t.expect_commit().returning(|_| Ok(()));
    t
}

/// Reconciliation-perm mock: run_for_week gates via `check_permission`
/// (HR_PRIVILEGE) internally? No — run_for_week does NOT re-run the HR gate
/// (it is an internal engine, Plan 02 decision). It only reads the toggle. So
/// the reconciliation service needs no permission expectation here.
fn recon_perm() -> MockPermissionService {
    MockPermissionService::new()
}

/// Toggle-Mock: active_from = 2020-01-01 (deckt alle Test-Wochen ab → Feature
/// aktiv).
fn toggle_on() -> MockToggleService {
    let mut t = MockToggleService::new();
    t.expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2020-01-01"))));
    t
}

/// Toggle-Mock: None → Feature dormant.
fn toggle_off() -> MockToggleService {
    let mut t = MockToggleService::new();
    t.expect_get_toggle_value().returning(|_, _, _| Ok(None));
    t
}

fn week_unset() -> MockWeekStatusService {
    let mut w = MockWeekStatusService::new();
    w.expect_get_week_status()
        .returning(|_, _, _, _| Ok(WeekStatus::Unset));
    w
}

/// A fixed clock so the Vorwoche is deterministic. 2026-03-16 is Monday of
/// KW 12/2026 (ISO). The last completed ISO week (Vorwoche) is therefore
/// KW 11/2026.
fn clock_2026_03_16() -> MockClockService {
    let mut c = MockClockService::new();
    c.expect_date_now().returning(|| date!(2026 - 03 - 16));
    c
}

/// Build the real reconciliation engine with the given mocks.
#[allow(clippy::too_many_arguments)]
fn build_recon(
    extra_hours_service: MockExtraHoursService,
    rebooking_batch_service: MockRebookingBatchService,
    voluntary_stats_service: MockVoluntaryStatsService,
    week_status_service: MockWeekStatusService,
    toggle_service: MockToggleService,
) -> RebookingReconciliationServiceImpl<ReconTestDeps> {
    RebookingReconciliationServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        rebooking_batch_service: Arc::new(rebooking_batch_service),
        reporting_service: Arc::new(MockReportingService::new()),
        voluntary_stats_service: Arc::new(voluntary_stats_service),
        week_status_service: Arc::new(week_status_service),
        employee_work_details_service: Arc::new(MockEmployeeWorkDetailsService::new()),
        toggle_service: Arc::new(toggle_service),
        clock_service: Arc::new(MockClockService::new()),
        permission_service: Arc::new(recon_perm()),
        transaction_dao: Arc::new(tx_ok()),
    }
}

fn build_scheduler(
    recon: RebookingReconciliationServiceImpl<ReconTestDeps>,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    clock_service: MockClockService,
) -> VoluntaryRebookingSchedulerImpl<SchedTestDeps> {
    VoluntaryRebookingSchedulerImpl::new(
        Arc::new(recon),
        Arc::new(employee_work_details_service),
        Arc::new(perm_full_ok()),
        Arc::new(clock_service),
        Arc::new(dao::MockTransactionDao::new()), // scheduler opens no tx itself
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// Case 1: active toggle + a capped person with excess in the Vorwoche.
/// `run_once_now` books exactly one AutoCron pair for that person+week: one
/// `rebooking_batch(kind=AutoCron, state=Approved)` and a source=Rebooking pair
/// (2 ExtraHours rows). The processed week is the last completed ISO week
/// (KW 11/2026 for a 2026-03-16 clock — Case 3 assertion folded in).
#[tokio::test]
async fn run_once_now_books_excess_for_vorwoche() {
    // Capture the ExtraHours pair the engine writes.
    let created: Arc<std::sync::Mutex<Vec<ExtraHours>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let created_cap = created.clone();
    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_create()
        .times(2)
        .returning(move |eh, _, _| {
            created_cap.lock().unwrap().push(eh.clone());
            Ok(eh.clone())
        });

    // Capture the batch (kind + week coordinates).
    let batches: Arc<std::sync::Mutex<Vec<RebookingBatchEntity>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let batches_cap = batches.clone();
    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(move |batch, entries, _, _| {
            assert_eq!(batch.kind, RebookingBatchKind::AutoCron);
            assert_eq!(batch.state, RebookingBatchState::Approved);
            assert!(entries.is_empty(), "F4 batch claims the slot; no entry");
            batches_cap.lock().unwrap().push(batch.clone());
            Ok(batch.clone())
        });

    let recon = build_recon(
        extra_hours_service,
        rebooking_batch_service,
        {
            let mut v = MockVoluntaryStatsService::new();
            v.expect_get_voluntary_stats()
                .returning(|_, _, _, _, _| Ok(make_voluntary_stats(8.0, 5.0)));
            v
        },
        week_unset(),
        toggle_on(),
    );

    // Vorwoche für die Clock 2026-03-16 (KW12) ist KW11/2026. Assert der
    // Scheduler genau diese Woche an all_for_week übergibt.
    let mut ewd = MockEmployeeWorkDetailsService::new();
    ewd.expect_all_for_week()
        .withf(|cw, year, _, _| *cw == 11 && *year == 2026)
        .times(1)
        .returning(|_, _, _, _| Ok(Arc::from(vec![capped_contract()])));

    let scheduler = build_scheduler(recon, ewd, clock_2026_03_16());

    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run_once_now must succeed");

    // Exactly one source=Rebooking pair for the Vorwoche.
    let rows = created.lock().unwrap();
    assert_eq!(rows.len(), 2, "exactly one pair (out + in)");
    let out = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::VolunteerWork)
        .expect("VolunteerWork out-row");
    let inn = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::ExtraWork)
        .expect("ExtraWork in-row");
    assert!((out.amount + 3.0).abs() < 1e-6, "out must be -3.0 (excess 8-5)");
    assert!((inn.amount - 3.0).abs() < 1e-6, "in must be +3.0");
    // Pair-Datum fällt in KW11/2026 (Montag 2026-03-09).
    let expected_monday = ShiftyDate::new(2026, 11, DayOfWeek::Monday)
        .unwrap()
        .to_date();
    for r in rows.iter() {
        assert_eq!(
            r.date_time.date(),
            expected_monday,
            "pair must fall into the Vorwoche KW11/2026 (Mon 2026-03-09)"
        );
    }

    // Exactly one AutoCron batch, stamped with the Vorwoche.
    let bs = batches.lock().unwrap();
    assert_eq!(bs.len(), 1, "exactly one AutoCron batch");
    assert_eq!(bs[0].iso_year, 2026);
    assert_eq!(bs[0].iso_week, 11, "processed week == last completed ISO week");
}

/// Case 2: active_from=None → full no-op. Every capped person is
/// Skipped{ToggleOff} inside run_for_week (before any tx / write), so NO batch
/// and NO pair are written. The reconciliation mocks assert-by-absence: no
/// `expect_create` on either the batch or extra-hours service means any call
/// would panic.
#[tokio::test]
async fn run_once_now_toggle_off_is_full_noop() {
    // No expect_create on either service → any write panics the test.
    let extra_hours_service = MockExtraHoursService::new();
    let rebooking_batch_service = MockRebookingBatchService::new();

    let recon = RebookingReconciliationServiceImpl::<ReconTestDeps> {
        extra_hours_service: Arc::new(extra_hours_service),
        rebooking_batch_service: Arc::new(rebooking_batch_service),
        reporting_service: Arc::new(MockReportingService::new()),
        voluntary_stats_service: Arc::new(MockVoluntaryStatsService::new()),
        week_status_service: Arc::new(MockWeekStatusService::new()),
        employee_work_details_service: Arc::new(MockEmployeeWorkDetailsService::new()),
        toggle_service: Arc::new(toggle_off()),
        clock_service: Arc::new(MockClockService::new()),
        permission_service: Arc::new(recon_perm()),
        // toggle-off short-circuits BEFORE use_transaction → no tx expected.
        transaction_dao: Arc::new(dao::MockTransactionDao::new()),
    };

    let mut ewd = MockEmployeeWorkDetailsService::new();
    ewd.expect_all_for_week()
        .times(1)
        .returning(|_, _, _, _| Ok(Arc::from(vec![capped_contract()])));

    let scheduler = build_scheduler(recon, ewd, clock_2026_03_16());

    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run_once_now must succeed and be a full no-op when toggle off");
    // Success + no mock-create panic == no batch and no pair written.
}

/// Case 2b: an uncapped person in the Vorwoche is never handed to the engine —
/// the scheduler filters on `cap_planned_hours_to_expected`. With only an
/// uncapped contract, run_for_week is never called (no tx / toggle read),
/// proving the cap-filter guard.
#[tokio::test]
async fn run_once_now_skips_uncapped_person() {
    let recon = RebookingReconciliationServiceImpl::<ReconTestDeps> {
        extra_hours_service: Arc::new(MockExtraHoursService::new()),
        rebooking_batch_service: Arc::new(MockRebookingBatchService::new()),
        reporting_service: Arc::new(MockReportingService::new()),
        voluntary_stats_service: Arc::new(MockVoluntaryStatsService::new()),
        week_status_service: Arc::new(MockWeekStatusService::new()),
        employee_work_details_service: Arc::new(MockEmployeeWorkDetailsService::new()),
        // No toggle read expected — the engine is never entered.
        toggle_service: Arc::new(MockToggleService::new()),
        clock_service: Arc::new(MockClockService::new()),
        permission_service: Arc::new(recon_perm()),
        transaction_dao: Arc::new(dao::MockTransactionDao::new()),
    };

    let mut uncapped = capped_contract();
    uncapped.cap_planned_hours_to_expected = false;
    let mut ewd = MockEmployeeWorkDetailsService::new();
    ewd.expect_all_for_week()
        .times(1)
        .returning(move |_, _, _, _| Ok(Arc::from(vec![uncapped.clone()])));

    let scheduler = build_scheduler(recon, ewd, clock_2026_03_16());

    scheduler
        .run_once_now(Authentication::Full)
        .await
        .expect("run_once_now must succeed and skip the uncapped person");
}
