//! Phase 56 Plan 02 Task 1 — Tests fuer `RebookingReconciliationService::run_for_week`
//! (REB-AUTO-01/03, D-02 Claim-on-Suggest, D-04 Locked-Audit, D-06 F4-Self-Guard).
//!
//! Test-Achsen (alle sieben `<behavior>`-Faelle des Plans):
//! 1. Excess-positiv gedeckelt → `Booked{excess}` + genau 1 Pair + 1 Approved-Batch.
//! 2. Kein Excess → `Skipped{NoExcess}`, kein Write.
//! 3. Zweiter Lauf (UNIQUE-Konflikt) → `Skipped{AlreadyProcessed}`, kein zweites Pair.
//! 4. Pending HR-Suggestion belegt den Slot (Claim-on-Suggest, D-02) → gleicher
//!    UNIQUE-Konflikt → `Skipped{AlreadyProcessed}`.
//! 5. `WeekStatus::Locked` → `Skipped{WeekLocked}` + SkippedLocked-Audit-Batch;
//!    zweiter Locked-Lauf ist no-op (UNIQUE-Konflikt ignoriert) (D-04).
//! 6. `active_from=None` / Woche vor Stichtag → `Skipped{ToggleOff}`, kein Write.
//! 7. ISO-Jahresgrenze (2026, 53) resolvt Montag ueber from_iso_week_date.
//!
//! Zusaetzlich: die pure `week_excess`-Formel-Matrix.
//!
//! Phase 57 Plan 03 (REB-AUTO-06) ergaenzt zwei Deckel-Achsen:
//! 8. Deckel bindet (Ist=15, committed=5, Balance −2) → `Booked{2.0}` statt 10.0,
//!    und die Balance-Range ist per `withf` gepinnt (year-to-date ab 1. Januar bis
//!    Sonntag der Woche, `include_carryover = true`, D-57-07).
//! 9. Balance >= 0 → `Skipped{NoExcess}`, kein Write (Invariante I-4).

use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use dao::rebooking_batch::{RebookingBatchKind, RebookingBatchState};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::{EmployeeReport, MockReportingService};
use service::toggle::MockToggleService;
use service::voluntary_stats::{MockVoluntaryStatsService, VoluntaryStats};
use service::week_status::{MockWeekStatusService, WeekStatus};
use service::MockPermissionService;
use service::ServiceError;
use shifty_utils::{DayOfWeek, ShiftyDate};

use service::rebooking_reconciliation::{
    RebookingReconciliationService, RunForWeekOutcome, SkipReason,
};

use crate::rebooking_reconciliation::{
    week_excess, RebookingReconciliationServiceDeps, RebookingReconciliationServiceImpl,
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
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0056_2001)
}

/// Toggle-Wert-String fuer active_from = 2026-01-01 (deckt alle 2026-Wochen ab,
/// sodass should_run_for_week==true fuer die Test-Wochen).
fn active_from_2026() -> Option<Arc<str>> {
    Some(Arc::from("2026-01-01"))
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
            name: Arc::from("Cron 56"),
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

fn tx_ok() -> dao::MockTransactionDao {
    let mut t = dao::MockTransactionDao::new();
    t.expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    t.expect_commit().returning(|_| Ok(()));
    t
}

/// Toggle-Mock, das active_from = 2026-01-01 liefert (Feature aktiv fuer 2026).
fn toggle_on() -> MockToggleService {
    let mut t = MockToggleService::new();
    t.expect_get_toggle_value()
        .returning(|_, _, _| Ok(active_from_2026()));
    t
}

/// Toggle-Mock, das None liefert (Feature dormant).
fn toggle_off() -> MockToggleService {
    let mut t = MockToggleService::new();
    t.expect_get_toggle_value().returning(|_, _, _| Ok(None));
    t
}

fn week_status(status: WeekStatus) -> MockWeekStatusService {
    let mut w = MockWeekStatusService::new();
    w.expect_get_week_status()
        .returning(move |_, _, _, _| Ok(status.clone()));
    w
}

fn week_unset() -> MockWeekStatusService {
    week_status(WeekStatus::Unset)
}

fn stats_service(ist: f32, soll: f32) -> MockVoluntaryStatsService {
    let mut v = MockVoluntaryStatsService::new();
    v.expect_get_voluntary_stats()
        .returning(move |_, _, _, _, _| Ok(make_voluntary_stats(ist, soll)));
    v
}

fn reporting_service(balance: f32) -> MockReportingService {
    let mut r = MockReportingService::new();
    r.expect_get_report_for_employee_range()
        .returning(move |_, _, _, _, _, _| Ok(make_report(balance)));
    r
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

// ─── Pure-fn Matrix (week_excess) ────────────────────────────────────────────

/// REB-AUTO-01 + REB-AUTO-06: week_excess ==
/// `min( max(ist - (soll + committed), 0), max(-balance, 0) )`.
///
/// Der unbeschraenkte Ueberschuss (REB-AUTO-01) wird seit Phase 57 zusaetzlich
/// vom Headroom des bezahlten Stundenkontos gedeckelt (REB-AUTO-06): die
/// Automatik gleicht das Konto bis 0 aus, hebt es aber nie darueber. Der
/// HR-Vorschlagspfad (`proposed_hours`) bleibt bewusst UNgedeckelt und darf die
/// Freiwilligen-Zusage unterschreiten (Invariante I-5, D-57-01).
///
/// Nicht-endliche Eingaben (NaN/Inf) ergeben in ALLEN vier Parametern 0.0.
#[test]
fn week_excess_matrix() {
    // ist - committed (soll=0 in run_for_week; committed==soll_total). Balance
    // -8 laesst in diesen fuenf Bestandszeilen genug Headroom → Deckel bindet nicht.
    assert!((week_excess(10.0, 0.0, 5.0, -8.0) - 5.0).abs() < 1e-6, "10-5=5");
    assert!((week_excess(5.0, 0.0, 5.0, -8.0) - 0.0).abs() < 1e-6, "5-5=0 clamp");
    assert!((week_excess(3.0, 0.0, 5.0, -8.0) - 0.0).abs() < 1e-6, "negative clamps to 0");
    assert!((week_excess(f32::NAN, 0.0, 5.0, -8.0) - 0.0).abs() < 1e-6, "NaN -> 0");
    assert!((week_excess(f32::INFINITY, 0.0, 5.0, -8.0) - 0.0).abs() < 1e-6, "Inf -> 0");

    // REB-AUTO-06 / Gegenprobe D-57-02: unconstrained waere 15-5=10, der Headroom
    // aus Balance -2 ist 2 → der Deckel bindet (Invariante I-3).
    assert!((week_excess(15.0, 0.0, 5.0, -2.0) - 2.0).abs() < 1e-6, "cap binds: min(10, 2) = 2");
    // Invariante I-4: bei ausgeglichener oder positiver Balance gibt es nichts
    // auszugleichen — kein Uebertrag, auch bei freiwilligem Ueberschuss.
    assert!((week_excess(10.0, 0.0, 5.0, 0.0) - 0.0).abs() < 1e-6, "balance 0 -> no headroom");
    assert!((week_excess(10.0, 0.0, 5.0, 3.0) - 0.0).abs() < 1e-6, "positive balance -> no headroom");
    // Der NaN/Inf-Guard deckt auch den neuen vierten Parameter ab (T-57-07).
    assert!((week_excess(10.0, 0.0, 5.0, f32::NAN) - 0.0).abs() < 1e-6, "NaN balance -> 0");
}

// ─── run_for_week behavior cases ─────────────────────────────────────────────

/// Case 1: Excess-positiv gedeckelter Mitarbeiter (Ist=8 > committed=5) →
/// Booked{3.0} + genau ein Pair (2 ExtraHours source=Rebooking) + 1 Approved-Batch.
#[tokio::test]
async fn run_for_week_books_excess() {
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

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, entries, _, _| {
            assert_eq!(batch.kind, RebookingBatchKind::AutoCron);
            assert_eq!(batch.state, RebookingBatchState::Approved);
            assert!(entries.is_empty(), "F4 batch claims the slot; no entry");
            Ok(batch.clone())
        });

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");

    match outcome {
        RunForWeekOutcome::Booked { hours } => {
            assert!((hours - 3.0).abs() < 1e-6, "excess = 8-5 = 3, got {hours}");
        }
        other => panic!("expected Booked, got {other:?}"),
    }

    let rows = created.lock().unwrap();
    assert_eq!(rows.len(), 2, "exactly one pair");
    let out = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::VolunteerWork)
        .expect("VolunteerWork out-row");
    let inn = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::ExtraWork)
        .expect("ExtraWork in-row");
    assert!((out.amount + 3.0).abs() < 1e-6, "out must be -3.0");
    assert!((inn.amount - 3.0).abs() < 1e-6, "in must be +3.0");
}

/// Case 2: kein Excess (Ist=5 <= committed=5) → Skipped{NoExcess}, kein Write.
#[tokio::test]
async fn run_for_week_no_excess_skips() {
    // ExtraHours/Batch::create duerfen NICHT aufgerufen werden (kein Expect).
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(0.0),
        stats_service(5.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::NoExcess
        }
    );
}

/// Case 8 (REB-AUTO-06, Invariante I-3): der Balance-Deckel bindet. Ist=15,
/// committed=5 → unconstrained waere `15 − 5 = 10`; der Headroom aus Balance −2
/// ist 2 → gebucht werden 2.0, nicht 10.0 (Gegenprobe D-57-02).
///
/// Die Roadmap-Formulierung „Ueberschuss 5" bezieht sich auf das Perioden-Delta
/// `Ist − Soll` ueber BEIDE Wochen des Referenz-Szenarios, nicht auf diesen
/// Wochen-Input — hier zaehlt allein `min(Wochen-Ueberschuss, Headroom)`.
///
/// Das `.withf` pinnt die Balance-Range strukturell (Pitfall 3): `suggest()` und
/// `approve()` lesen die Balance fuer die ISOLIERTE ISO-Woche OHNE Carryover.
/// Wer dieses naheliegende Muster nach `run_for_week` kopiert, faerbt diesen Test
/// rot, statt still den falschen Deckel zu bauen.
#[tokio::test]
async fn run_for_week_balance_cap_binds() {
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

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, entries, _, _| {
            assert_eq!(batch.kind, RebookingBatchKind::AutoCron);
            assert_eq!(batch.state, RebookingBatchState::Approved);
            assert!(entries.is_empty(), "F4 batch claims the slot; no entry");
            Ok(batch.clone())
        });

    // D-57-07: year-to-date ab dem Montag der ISO-Woche 1 bis Sonntag der
    // verarbeiteten ISO-Woche, include_carryover = true. NICHT die isolierte
    // Wochen-Range — und NICHT der Kalender-Anker `first_day_in_year` (CR-01,
    // siehe run_for_week_balance_range_is_iso_anchored_in_2027).
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_report_for_employee_range()
        .withf(|sales_person_id, from, to, include_carryover, _ctx, _tx| {
            *sales_person_id == sp_id()
                && *from
                    == ShiftyDate::new(2026, 1, DayOfWeek::Monday)
                        .expect("ISO week 1/2026 has a Monday")
                && *to
                    == ShiftyDate::new(2026, 12, DayOfWeek::Sunday)
                        .expect("KW 12/2026 has a Sunday")
                && *include_carryover
        })
        .returning(|_, _, _, _, _, _| Ok(make_report(-2.0)));

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service,
        stats_service(15.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");

    match outcome {
        RunForWeekOutcome::Booked { hours } => {
            assert!(
                (hours - 2.0).abs() < 1e-6,
                "capped: min(15-5, -(-2)) = 2, got {hours}"
            );
        }
        other => panic!("expected Booked, got {other:?}"),
    }

    let rows = created.lock().unwrap();
    assert_eq!(rows.len(), 2, "exactly one pair");
    let out = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::VolunteerWork)
        .expect("VolunteerWork out-row");
    let inn = rows
        .iter()
        .find(|r| r.category == ExtraHoursCategory::ExtraWork)
        .expect("ExtraWork in-row");
    assert!((out.amount + 2.0).abs() < 1e-6, "out must be -2.0");
    assert!((inn.amount - 2.0).abs() < 1e-6, "in must be +2.0");
}

/// Case 9 (REB-AUTO-06, Invariante I-4): ausgeglichene Balance → kein Uebertrag.
/// Ist=15 > committed=5 liefert einen freiwilligen Ueberschuss, aber das bezahlte
/// Stundenkonto steht auf 0 — es gibt nichts auszugleichen, und die Automatik
/// darf das Konto nicht ins Plus heben. Kein `ExtraHoursService::create`, kein
/// `RebookingBatchService::create` (beide Mocks ohne Expect → jeder Write paniked).
#[tokio::test]
async fn run_for_week_non_negative_balance_skips() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(0.0),
        stats_service(15.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::NoExcess
        },
        "balance >= 0 leaves no headroom (I-4)"
    );
}

/// CR-01 (Code-Review Phase 57): die Balance-Range MUSS an ISO-Koordinaten
/// verankert sein, nicht am Kalender-Neujahr.
///
/// `ShiftyDate::first_day_in_year(year)` geht ueber `from_ymd(year, 1, 1)` und
/// speichert das Ergebnis als ISO-Tripel. Faellt der 1. Januar auf Fr/Sa/So,
/// gehoert er zum VORHERIGEN ISO-Jahr — fuer 2027 (1. Jan = Freitag) also auf
/// ISO-Woche 53/2026. `get_report_for_employee_range` liest daraus
/// `from_date.year() - 1` fuer den Carryover (→ 2025 statt 2026) und iteriert
/// `hours_per_week` ab KW 53/2026, zieht also eine volle Vorjahres-Woche in die
/// als „year-to-date 2027" gedachte Balance.
///
/// Der Wert steuert `max(-balance, 0)` und damit direkt den geschriebenen
/// `hours`-Betrag — ein Off-by-one ist hier ein Write-Fehler, kein Anzeigefehler.
/// `run_for_week_balance_cap_binds` pinnt nur 2026, und 2026 ist eines der Jahre,
/// in denen Kalender- und ISO-Jahresanfang zusammenfallen (1. Jan = Donnerstag =
/// W01) — dieser Test ist der Waechter fuer die uebrigen Jahre.
///
/// `stats_service(5.0, 5.0)` haelt den Excess bei 0, damit der Test allein die
/// Range prueft und kein Write ausloest (beide Write-Mocks ohne Expect).
#[tokio::test]
async fn run_for_week_balance_range_is_iso_anchored_in_2027() {
    let captured: Arc<std::sync::Mutex<Option<(ShiftyDate, ShiftyDate, bool)>>> =
        Arc::new(std::sync::Mutex::new(None));
    let captured_cap = captured.clone();
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_report_for_employee_range()
        .returning(move |_, from, to, include_carryover, _, _| {
            *captured_cap.lock().unwrap() = Some((from, to, include_carryover));
            Ok(make_report(-8.0))
        });

    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service,
        stats_service(5.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2027,
            3,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::NoExcess
        }
    );

    let (from, to, include_carryover) = captured
        .lock()
        .unwrap()
        .expect("the balance read must happen before week_voluntary_figures");
    assert_eq!(
        from,
        ShiftyDate::new(2027, 1, DayOfWeek::Monday).expect("ISO week 1/2027 has a Monday"),
        "range start must be the Monday of ISO week 1/2027 (2027-01-04), not \
         first_day_in_year(2027) which resolves to ISO week 53/2026"
    );
    assert_eq!(
        from.year(),
        2027,
        "from.year() drives the carryover lookup (from.year() - 1) in \
         get_report_for_employee_range — it MUST be the processed ISO year"
    );
    assert_eq!(
        to,
        ShiftyDate::new(2027, 3, DayOfWeek::Sunday).expect("ISO week 3/2027 has a Sunday"),
    );
    assert!(include_carryover, "D-57-07: include_carryover = true");
}

/// Case 3: zweiter Lauf → UNIQUE-Konflikt beim Batch-create → Skipped{AlreadyProcessed},
/// KEIN zweites Pair (REB-AUTO-03 Idempotenz).
#[tokio::test]
async fn run_for_week_idempotent_on_conflict() {
    // ExtraHours::create darf NICHT aufgerufen werden (Batch-Claim kollidiert
    // ZUERST).
    let extra_hours_service = MockExtraHoursService::new();

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, _, _, _| Err(ServiceError::EntityAlreadyExists(batch.id)));

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed (idempotent skip, not error)");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::AlreadyProcessed
        },
        "second run over processed week must be AlreadyProcessed"
    );
}

/// Case 4: pending HR-Suggestion belegt den Slot (Claim-on-Suggest, D-02). Der
/// UNIQUE-Slot ist belegt → Batch-create liefert EntityAlreadyExists → gleicher
/// AlreadyProcessed-Skip wie Case 3 (der Cron respektiert den HR-Claim).
#[tokio::test]
async fn run_for_week_respects_pending_suggestion_claim() {
    let extra_hours_service = MockExtraHoursService::new();

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    // Der Slot ist durch einen pending HrSuggestion-Batch belegt → create
    // kollidiert am selben UNIQUE-Index.
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, _, _, _| Err(ServiceError::EntityAlreadyExists(batch.id)));

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::AlreadyProcessed
        },
        "cron must yield to an existing HR-suggestion claim (D-02)"
    );
}

/// Case 5: WeekStatus::Locked → Skipped{WeekLocked} + SkippedLocked-Audit-Batch;
/// KEIN Pair. (D-04)
#[tokio::test]
async fn run_for_week_locked_writes_audit_batch() {
    // ExtraHours::create darf NICHT aufgerufen werden (kein Pair auf Locked).
    let extra_hours_service = MockExtraHoursService::new();

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, entries, _, _| {
            assert_eq!(
                batch.state,
                RebookingBatchState::SkippedLocked,
                "locked week persists a SkippedLocked audit batch"
            );
            assert!(entries.is_empty(), "audit batch has no entry");
            Ok(batch.clone())
        });

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_status(WeekStatus::Locked),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed on locked week");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::WeekLocked
        }
    );
}

/// Case 5b: zweiter Locked-Lauf → Audit-Batch-create kollidiert (UNIQUE) → wird
/// ignoriert (`.ok()`), Ergebnis bleibt Skipped{WeekLocked}, kein Crash (D-04).
#[tokio::test]
async fn run_for_week_locked_twice_is_noop() {
    let extra_hours_service = MockExtraHoursService::new();

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, _, _, _| Err(ServiceError::EntityAlreadyExists(batch.id)));

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_status(WeekStatus::Locked),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("second locked run must not crash (UNIQUE ignored)");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::WeekLocked
        }
    );
}

/// Case 6a: active_from=None (Toggle dormant) → Skipped{ToggleOff}, kein Datenabruf,
/// kein Write (F4-Self-Guard, D-06). tx wird nie geoeffnet.
#[tokio::test]
async fn run_for_week_toggle_off_skips() {
    // Nur der Toggle-Read passiert; keine tx, kein Datenabruf, kein Write.
    let transaction_dao = dao::MockTransactionDao::new(); // kein use_transaction erwartet

    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        MockReportingService::new(),
        MockVoluntaryStatsService::new(),
        MockWeekStatusService::new(),
        MockEmployeeWorkDetailsService::new(),
        toggle_off(),
        MockClockService::new(),
        perm_ok(),
        transaction_dao,
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed (toggle off = skip, not error)");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::ToggleOff
        }
    );
}

/// Case 6b: Woche vor Stichtag (active_from = 2026-06-01, Woche KW 12/2026 liegt
/// davor) → Skipped{ToggleOff}, kein Write.
#[tokio::test]
async fn run_for_week_before_active_from_skips() {
    let transaction_dao = dao::MockTransactionDao::new();
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2026-06-01"))));

    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        MockReportingService::new(),
        MockVoluntaryStatsService::new(),
        MockWeekStatusService::new(),
        MockEmployeeWorkDetailsService::new(),
        toggle_service,
        MockClockService::new(),
        perm_ok(),
        transaction_dao,
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            12,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week must succeed");
    assert_eq!(
        outcome,
        RunForWeekOutcome::Skipped {
            reason: SkipReason::ToggleOff
        },
        "KW12/2026 (Mo 2026-03-16) is before 2026-06-01"
    );
}

/// Case 7: ISO-Jahresgrenze (2026, 53) — der Montag wird ueber
/// from_iso_week_date resolvt (2026 hat KW 53, Mo 2026-12-28). Mit
/// active_from=2026-01-01 laeuft die Woche und bucht den Excess. Beweist,
/// dass die Wochen-Range-Aufloesung nicht naiv year()+week() macht.
#[tokio::test]
async fn run_for_week_year_boundary_week_53() {
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

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .times(1)
        .returning(|batch, _, _, _| {
            assert_eq!(batch.iso_week, 53, "week 53 must be threaded verbatim");
            Ok(batch.clone())
        });

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-8.0),
        stats_service(8.0, 5.0),
        week_unset(),
        MockEmployeeWorkDetailsService::new(),
        toggle_on(),
        MockClockService::new(),
        perm_ok(),
        tx_ok(),
    );

    let outcome = service
        .run_for_week(
            sp_id(),
            2026,
            53,
            RebookingBatchKind::AutoCron,
            Authentication::Full,
            None,
        )
        .await
        .expect("run_for_week (2026, 53) must resolve Monday via ISO-week-date");
    match outcome {
        RunForWeekOutcome::Booked { hours } => {
            assert!((hours - 3.0).abs() < 1e-6);
        }
        other => panic!("expected Booked at year boundary, got {other:?}"),
    }
    // Sanity: das Pair-Datum faellt in KW 53 (Dez 2026), nicht in ein naives KW53.
    let rows = created.lock().unwrap();
    for r in rows.iter() {
        assert_eq!(
            r.date_time.date(),
            datetime!(2026 - 12 - 28 00:00:00).date(),
            "week-53 Monday must be 2026-12-28"
        );
    }
}
