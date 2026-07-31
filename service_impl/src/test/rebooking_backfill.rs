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
//!
//! Phase 57 Plan 03 (REB-AUTO-06) ergaenzt eine Deckel-Achse:
//! 5. Der Preview-Excess ist GEDECKELT — `BackfillPreviewItem.excess` zeigt exakt
//!    den Betrag, den ein spaeterer Apply-Lauf ueber `run_for_week` buchen wuerde
//!    (Invariante I-7). Deshalb ist `reporting_service` parametrisiert: `-8.0`
//!    (Deckel bindet nicht, Bestandsfaelle) vs. `-2.0` (Deckel bindet).

use std::sync::Arc;

use uuid::Uuid;

use dao::rebooking_batch::{RebookingBatchKind, RebookingBatchState};
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::{EmployeeReport, MockReportingService};
use service::toggle::MockToggleService;
use service::voluntary_stats::{MockVoluntaryStatsService, VoluntaryStats};
use service::week_status::{MockWeekStatusService, WeekStatus};
use service::MockPermissionService;
use service::ServiceError;
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};

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

/// Clock fixiert auf 2026-03-23 (Mo KW 13/2026) → upper bound = KW 12/2026.
/// Damit deckt ein from=KW 10 drei Wochen ab (KW 10, 11, 12).
fn clock_2026_w13() -> MockClockService {
    let mut c = MockClockService::new();
    c.expect_date_now()
        .returning(|| time::macros::date!(2026 - 03 - 23));
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

/// Balance-Mock fuer den Deckel (REB-AUTO-06). Spiegelbildlich zum gleichnamigen
/// Helper in `voluntary_rebooking.rs`: `-8.0` laesst genug Headroom, sodass der
/// Deckel in den Bestandsfaellen nicht bindet.
fn reporting_service(balance: f32) -> MockReportingService {
    let mut r = MockReportingService::new();
    r.expect_get_report_for_employee_range()
        .returning(move |_, _, _, _, _, _| Ok(make_report(balance)));
    r
}

/// Balance-Mock, der die Wirkung bereits geschriebener Paare abbildet: die
/// gelieferte Balance ist `start_balance + booked`, wobei `booked` die Summe der
/// bisher ueber `ExtraHoursService::create` geschriebenen `+h`-ExtraWork-Beine
/// ist. Das ist genau die Sichtbarkeit, die SQLite innerhalb der geteilten
/// `read_tx` liefert — ein konstanter Mock koennte die Divergenz zwischen
/// Preview und Apply gar nicht sichtbar machen (CR-02).
fn reporting_service_tracking(
    start_balance: f32,
    booked: Arc<std::sync::Mutex<f32>>,
) -> MockReportingService {
    let mut r = MockReportingService::new();
    r.expect_get_report_for_employee_range()
        .returning(move |_, _, _, _, _, _| {
            Ok(make_report(start_balance + *booked.lock().unwrap()))
        });
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
        reporting_service(-8.0),
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

/// REB-AUTO-06 / Invariante I-7: der Dry-Run-Preview zeigt den GEDECKELTEN Betrag.
/// Ist=15, committed=5 → unconstrained waere 10; der Headroom aus Balance −2 ist 2
/// → `excess == 2.0`.
///
/// Der Doc-Kommentar auf `BackfillPreviewItem.excess`
/// (`service/src/rebooking_reconciliation.rs:115`) verspricht „der Ueberschuss, der
/// gebucht wuerde" — ein ungedeckelter Preview-Wert waere ein Korrektheitsbruch
/// gegenueber der REST-Antwort und der HR-UI. Zusammen mit
/// `backfill_dry_run_writes_nothing` (excess 3.0 bei Balance −8) faerbt dieser Test
/// jeden konstanten Platzhalter als vierten `week_excess`-Parameter rot (T-57-08).
#[tokio::test]
async fn backfill_dry_run_preview_respects_balance_cap() {
    // Weder Batch- noch ExtraHours-create duerfen laufen (dry_run).
    let rebooking_batch_service = MockRebookingBatchService::new();
    let extra_hours_service = MockExtraHoursService::new();

    let service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service(-2.0),
        stats_service(15.0, 5.0),
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
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("dry-run backfill must succeed");

    assert_eq!(items.len(), 1, "one week (KW11) x one capped person");
    assert!(
        (items[0].excess - 2.0).abs() < 1e-6,
        "capped preview: min(15-5, -(-2)) = 2, got {}",
        items[0].excess
    );
    assert!(items[0].would_book, "would_book in dry-run preview");
}

/// CR-02 (Code-Review Phase 57): Preview-Summe == Apply-Summe ueber MEHRERE
/// Wochen (Invariante I-7).
///
/// `backfill_dry_run_preview_respects_balance_cap` iteriert genau eine Woche und
/// ist deshalb gegen den eigentlichen Fehler blind: der Apply-Pfad verbraucht den
/// Headroom pro geschriebenem Paar (die Writes sind in der geteilten `read_tx`
/// sichtbar), der Dry-Run schreibt nichts und sah damit in JEDER Woche denselben,
/// unverbrauchten Headroom. Der Fehler waechst linear mit der Anzahl der
/// Backfill-Wochen — also genau im Mehrjahres-Szenario, fuer das der Backfill
/// existiert.
///
/// Szenario: Balance −10 zu Beginn, je 4 h freiwilliger Ueberschuss (Ist 9,
/// committed 5) in KW 10/11/12. Erwartet in beiden Modi: 4 + 4 + 2 = 10 (der
/// Deckel bindet in der dritten Woche). Ohne den Fix meldet der Preview
/// 4 + 4 + 4 = 12.
#[tokio::test]
async fn backfill_dry_run_preview_matches_apply_over_multiple_weeks() {
    // ── Lauf 1: Dry-Run ──────────────────────────────────────────────────────
    // `booked` bleibt hier 0.0 (dry_run schreibt nichts) — der Balance-Mock
    // liefert also konstant −10, genau wie SQLite es ohne Writes taete.
    let dry_run_booked = Arc::new(std::sync::Mutex::new(0.0f32));
    let dry_run_service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service_tracking(-10.0, dry_run_booked.clone()),
        stats_service(9.0, 5.0),
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w13(), // upper = KW 12
        perm_ok(),
        tx_ok(),
    );

    let items = dry_run_service
        .backfill_range(
            ShiftyWeek::new(2026, 10),
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("dry-run backfill must succeed");

    assert_eq!(items.len(), 3, "KW 10, 11, 12");
    assert_eq!(
        (*dry_run_booked.lock().unwrap()),
        0.0,
        "dry_run must not write anything"
    );
    let preview: Vec<f32> = items.iter().map(|i| i.excess).collect();
    let preview_sum: f32 = preview.iter().sum();

    // ── Lauf 2: Apply ────────────────────────────────────────────────────────
    let apply_booked = Arc::new(std::sync::Mutex::new(0.0f32));
    let booked_rows: Arc<std::sync::Mutex<Vec<f32>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let booked_for_eh = apply_booked.clone();
    let rows_for_eh = booked_rows.clone();
    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service.expect_create().returning(move |eh, _, _| {
        if eh.category == ExtraHoursCategory::ExtraWork {
            *booked_for_eh.lock().unwrap() += eh.amount;
            rows_for_eh.lock().unwrap().push(eh.amount);
        }
        Ok(eh.clone())
    });

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    rebooking_batch_service
        .expect_create()
        .returning(|batch, _, _, _| Ok(batch.clone()));

    let apply_service = build_service(
        extra_hours_service,
        rebooking_batch_service,
        reporting_service_tracking(-10.0, apply_booked.clone()),
        stats_service(9.0, 5.0),
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2026_w13(),
        perm_ok(),
        tx_ok(),
    );

    apply_service
        .backfill_range(
            ShiftyWeek::new(2026, 10),
            false,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("apply backfill must succeed");

    let applied: Vec<f32> = booked_rows.lock().unwrap().clone();
    let applied_sum: f32 = applied.iter().sum();

    // ── Vergleich ────────────────────────────────────────────────────────────
    assert!(
        (applied_sum - 10.0).abs() < 1e-3,
        "apply consumes the headroom: 4 + 4 + 2 = 10, got {applied_sum} ({applied:?})"
    );
    assert!(
        (preview_sum - applied_sum).abs() < 1e-3,
        "Invariante I-7: der Preview verspricht den Betrag, den ein Apply-Lauf \
         bucht — preview={preview:?} (sum {preview_sum}), apply={applied:?} \
         (sum {applied_sum}). Der Dry-Run fuehrt den bereits vergebenen Headroom \
         nicht mit."
    );
    for (week_index, (p, a)) in preview.iter().zip(applied.iter()).enumerate() {
        assert!(
            (p - a).abs() < 1e-3,
            "week #{week_index}: preview {p} != apply {a}"
        );
    }
}

/// WR-02 (Code-Review Phase 57): die Mindest-Buchungsgroesse gilt AUCH im
/// Dry-Run-Zweig — dort ist sie sogar dringlicher als im Schreibpfad.
///
/// Die `dry_run_consumed`-Buchfuehrung aus CR-02 rechnet den bereits
/// vorgemerkten Uebertrag auf die gelesene Balance an. Ist der Headroom exakt
/// aufgebraucht, entsteht das f32-Residuum ohne jeden Datenbank-Roundtrip —
/// die Vorschau meldete dann `would_book` fuer einen Betrag, den
/// `run_for_week` im Apply-Lauf verweigert (Invariante I-7 gebrochen).
///
/// Hier direkt am Grenzwert geprueft: Headroom 0.005 h (< `MIN_BOOKABLE_HOURS`
/// = 0.01) bei einem unbeschraenkten Ueberschuss von 4 h.
#[tokio::test]
async fn backfill_dry_run_sub_threshold_excess_is_no_excess() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(-0.005),
        stats_service(9.0, 5.0), // unconstrained waere 4.0
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
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("dry-run backfill must succeed");

    assert_eq!(items.len(), 1, "one week (KW11) x one capped person");
    assert!(
        items[0].skipped,
        "a sub-threshold headroom is float noise — the preview must not promise a booking"
    );
    assert_eq!(items[0].skip_reason, Some(SkipReason::NoExcess));
    assert!(
        !items[0].would_book,
        "would_book must mirror the write path's MIN_BOOKABLE_HOURS guard"
    );
}

/// CR-01 (Code-Review Phase 57): der Backfill-Balance-Read hat dieselbe
/// ISO-Verankerung wie `run_for_week`.
///
/// `ShiftyDate::first_day_in_year(2027)` liefert ISO-Woche 53/2026 (1. Jan 2027
/// ist ein Freitag) — als Range-Start waere das ein anderes Carryover-Jahr plus
/// eine zusaetzliche Vorjahres-Woche im „year-to-date"-Aggregat. Da der Preview
/// die HR-facing Entscheidungsgrundlage der `/backfill?dry_run=true`-Antwort ist,
/// muss der Wert exakt der Range aus dem Schreib-Pfad entsprechen (Invariante
/// I-7). Gegenstueck zu
/// `voluntary_rebooking.rs::run_for_week_balance_range_is_iso_anchored_in_2027`.
#[tokio::test]
async fn backfill_balance_range_is_iso_anchored_in_2027() {
    let captured: Arc<std::sync::Mutex<Vec<(ShiftyDate, ShiftyDate)>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_cap = captured.clone();
    let mut reporting_service = MockReportingService::new();
    reporting_service
        .expect_get_report_for_employee_range()
        .returning(move |_, from, to, _, _, _| {
            captured_cap.lock().unwrap().push((from, to));
            Ok(make_report(-8.0))
        });

    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service,
        stats_service(5.0, 5.0), // kein Excess → kein Write, nur der Range-Read zaehlt
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-01-01"),
        clock_2027_w02(), // upper = KW 01/2027
        perm_ok(),
        tx_ok(),
    );

    let items = service
        .backfill_range(
            ShiftyWeek::new(2027, 1),
            true,
            Authentication::Context(()),
            None,
        )
        .await
        .expect("backfill must succeed");
    assert_eq!(items.len(), 1, "one week (KW 01/2027) x one capped person");

    let ranges = captured.lock().unwrap();
    assert_eq!(ranges.len(), 1, "exactly one balance read");
    assert_eq!(
        ranges[0].0,
        ShiftyDate::new(2027, 1, DayOfWeek::Monday).expect("ISO week 1/2027 has a Monday"),
        "range start must be the Monday of ISO week 1/2027 (2027-01-04), not \
         first_day_in_year(2027) which resolves to ISO week 53/2026"
    );
    assert_eq!(ranges[0].0.year(), 2027, "carryover lookup uses from.year() - 1");
    assert_eq!(
        ranges[0].1,
        ShiftyDate::new(2027, 1, DayOfWeek::Sunday).expect("ISO week 1/2027 has a Sunday"),
    );
}

/// Ungedeckelte Personen werden ignoriert (nur cap_planned_hours_to_expected=true).
#[tokio::test]
async fn backfill_ignores_uncapped_persons() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        reporting_service(-8.0),
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
        reporting_service(-8.0),
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
        reporting_service(-8.0),
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
        reporting_service(-8.0),
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

/// WR-06 (Code-Review Phase 57): das Cutoff-Gate steht VOR den teuren Reads.
///
/// In `run_for_week` liegt der Balance-Read ausdruecklich hinter dem Toggle-Gate,
/// „damit ein ausgeschalteter Cron / eine gesperrte Woche ohne Report-Roundtrip
/// zurueckkehrt". Der Backfill las ihn davor. Fuer einen historischen Backfill,
/// dessen `from` vor dem Stichtag liegt — der Normalfall —, zahlte damit jede
/// (Person, Woche) einen vollen year-to-date-Report plus VoluntaryStats, nur um
/// danach als ToggleOff verworfen zu werden. Der YTD-Report ist selbst
/// O(Wochen im Jahr), und der Endpunkt hat keine untere Schranke fuer `from`
/// (T-56-05 DoS-Bound).
///
/// `MockReportingService::new()` und `MockVoluntaryStatsService::new()` sind ohne
/// Expect gebaut: jeder Read paniked. Der Test faerbt also rot, sobald das Gate
/// wieder hinter die Reads rutscht.
///
/// Gegenstueck der Aussage im Item: fuer ToggleOff-Wochen stehen die Kennzahlen
/// auf `0.0` — sie sind nicht berechnet (dokumentiert am `BackfillPreviewItem`).
#[tokio::test]
async fn backfill_toggle_off_week_skips_before_expensive_reads() {
    let service = build_service(
        MockExtraHoursService::new(),
        MockRebookingBatchService::new(),
        MockReportingService::new(),      // kein Report-Roundtrip erwartet
        MockVoluntaryStatsService::new(), // keine VoluntaryStats erwartet
        week_unset(),
        contracts(vec![capped_work_details()]),
        toggle_from("2026-12-01"), // alle Backfill-Wochen liegen davor
        clock_2026_w13(),          // upper = KW 12
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
        .expect("backfill must succeed without touching the report");

    assert_eq!(items.len(), 3, "KW10, KW11, KW12");
    for item in &items {
        assert_eq!(item.skip_reason, Some(SkipReason::ToggleOff));
        assert!(item.skipped);
        assert!(!item.would_book);
        assert_eq!(
            (item.voluntary_ist, item.voluntary_soll, item.committed_voluntary, item.excess),
            (0.0, 0.0, 0.0, 0.0),
            "pre-cutoff figures are not computed — documented on BackfillPreviewItem"
        );
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
        reporting_service(-8.0),
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
