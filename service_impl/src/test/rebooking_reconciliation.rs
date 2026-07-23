//! Phase 55 Plan 02 — Tests fuer den Business-Logic `RebookingReconciliationService`
//! (REB-MANUAL-01/02, HR-ALERT-02/03/04, VOL-ACCT-03) + die pure fns
//! `alert_predicate` / `proposed_hours`.
//!
//! Test-Achsen:
//!
//! 1. **VOL-ACCT-03 Neutralitaet (Property-Test, HART, TDD-RED-first):** eine
//!    EINZELNE manuelle Umbuchung (das -N/+N-Pair mit `source=Rebooking`) laesst
//!    `balance_hours` UND `volunteer_hours` (= Ist-Aggregat der VoluntaryStats)
//!    in den Read-Aggregaten UNVERAENDERT. Der Nachweis erfolgt am
//!    `ReportingServiceImpl` mit einem `MockExtraHoursService`, der einmal die
//!    Baseline und einmal Baseline+Rebooking-Pair liefert — beide Reports muessen
//!    balance/volunteer byte-genau (Epsilon) gleich sein. Damit ist die
//!    Filter-Site-Autoritaet festgenagelt (D-55-A1), bevor der Filter existiert.
//!    (Soll + Delta sind reine committed_voluntary-pure-fns und rebooking-blind;
//!    ist == volunteer_hours ⇒ ist/soll/delta alle neutral.)
//! 2. **Pure-fn-Matrizen:** `alert_predicate` + `proposed_hours` (HR-ALERT-01/D-55-01/03).
//! 3. **BL-Service (mockall):** rebook_manual erzeugt das Pair + Batch (beide
//!    Richtungen), suggest claimt einen pending-Batch und liefert IST/DANN,
//!    approve/reject transitionieren den state (approve schreibt Pair, reject nicht),
//!    Rollback-Sicherheit, Conflict/Idempotenz (REB-MANUAL, HR-ALERT, Modell B).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::datetime;
use uuid::Uuid;

use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursSource, MockExtraHoursService};
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::{DayOfWeek, ShiftyDate};

use crate::reporting::{
    alert_predicate, proposed_hours, ReportingServiceDeps, ReportingServiceImpl,
};

// ─── Fixtures fuer die Neutralitaets-Property (ReportingService-Ebene) ────────

fn neutrality_sp_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00AC_C303)
}

/// Gedeckelter Vertrag 5h/Woche Mo-Fr, KW 10..=20/2026, cap aktiv,
/// committed_voluntary=5.0 (der Mitarbeiter hat freiwillige Zusagen).
fn neutrality_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00AC_C310),
        sales_person_id: neutrality_sp_id(),
        expected_hours: 5.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 10,
        from_year: 2026,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 20,
        to_year: 2026,
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

/// Baut eine `ExtraHours`-Row in KW 12/2026 (Mi 2026-03-18).
fn eh(amount: f32, category: ExtraHoursCategory, source: ExtraHoursSource, tag: u128) -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00AC_0000 + tag),
        sales_person_id: neutrality_sp_id(),
        amount,
        category,
        description: Arc::from("neutrality-fixture"),
        date_time: datetime!(2026 - 03 - 18 09:00:00),
        created: Some(datetime!(2026 - 03 - 18 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source,
    }
}

/// Baseline: einige manuelle VolunteerWork-Stunden + etwas manuelle ExtraWork.
fn baseline_extra_hours() -> Vec<ExtraHours> {
    vec![
        eh(3.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Manual, 1),
        eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Manual, 2),
    ]
}

/// Baseline + EIN Rebooking-Pair (VolunteerToExtra ueber N=2.0): -2 VolunteerWork
/// + +2 ExtraWork, BEIDE `source=Rebooking`. Muss neutral sein.
fn extra_hours_with_rebooking_pair() -> Vec<ExtraHours> {
    let mut v = baseline_extra_hours();
    v.push(eh(-2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 3));
    v.push(eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 4));
    v
}

struct NeutralityReportingDeps;
impl ReportingServiceDeps for NeutralityReportingDeps {
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
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
}

fn make_neutrality_sales_person() -> service::sales_person::SalesPerson {
    service::sales_person::SalesPerson {
        id: neutrality_sp_id(),
        name: Arc::from("Neutrality Test"),
        background_color: Arc::from("#123456"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Baut einen `ReportingServiceImpl`, dessen `ExtraHoursService` die uebergebene
/// Liste liefert. Alle anderen Deps sind neutral gemockt (kein Shiftplan, keine
/// Absence, kein Carryover, kein Holiday-Toggle).
fn build_reporting_service(
    extra_hours: Vec<ExtraHours>,
) -> ReportingServiceImpl<NeutralityReportingDeps> {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(make_neutrality_sales_person()));

    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![neutrality_work_details()])));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let eh_arc: Arc<[ExtraHours]> = Arc::from(extra_hours);
    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(eh_arc.clone()));
    extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let mut carryover_service = MockCarryoverService::new();
    carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    let mut transaction_dao = dao::MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(None));

    ReportingServiceImpl {
        extra_hours_service: Arc::new(extra_hours_service),
        shiftplan_report_service: Arc::new(shiftplan_report_service),
        employee_work_details_service: Arc::new(employee_work_details_service),
        sales_person_service: Arc::new(sales_person_service),
        carryover_service: Arc::new(carryover_service),
        permission_service: Arc::new(permission_service),
        clock_service: Arc::new(MockClockService::new()),
        uuid_service: Arc::new(MockUuidService::new()),
        absence_service: Arc::new(absence_service),
        transaction_dao: Arc::new(transaction_dao),
        special_day_service: Arc::new(MockSpecialDayService::new()),
        toggle_service: Arc::new(toggle_service),
    }
}

async fn balance_and_volunteer(extra_hours: Vec<ExtraHours>) -> (f32, f32) {
    let service = build_reporting_service(extra_hours);
    let report = service
        .get_report_for_employee_range(
            &neutrality_sp_id(),
            ShiftyDate::from_ymd(2026, 3, 2).unwrap(),
            ShiftyDate::from_ymd(2026, 5, 17).unwrap(),
            false,
            Authentication::Full,
            None,
        )
        .await
        .expect("report must succeed");
    (report.balance_hours, report.volunteer_hours)
}

/// VOL-ACCT-03 (D-55-A1) HART: eine EINZELNE VolunteerToExtra-Umbuchung
/// (-N VolunteerWork + +N ExtraWork, beide source=Rebooking) laesst balance
/// UND volunteer_hours (Ist-Aggregat) unveraendert. Property: baseline == pair.
#[tokio::test]
async fn rebooking_neutrality_single_manual() {
    let (bal_before, vol_before) = balance_and_volunteer(baseline_extra_hours()).await;
    let (bal_after, vol_after) = balance_and_volunteer(extra_hours_with_rebooking_pair()).await;

    assert!(
        (bal_before - bal_after).abs() < 1e-3,
        "balance must be rebooking-neutral: before={bal_before}, after={bal_after}"
    );
    assert!(
        (vol_before - vol_after).abs() < 1e-3,
        "volunteer_hours (Ist) must be rebooking-neutral: before={vol_before}, after={vol_after}"
    );
}

/// VOL-ACCT-03 Roundtrip: hin (VolunteerToExtra) UND zurueck (ExtraToVolunteer)
/// — also zwei Pairs (4 Rebooking-Rows, netto 0) — ebenfalls neutral.
#[tokio::test]
async fn rebooking_neutrality_roundtrip() {
    let mut roundtrip = baseline_extra_hours();
    // VolunteerToExtra N=2: -2 Volunteer, +2 Extra.
    roundtrip.push(eh(-2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 3));
    roundtrip.push(eh(2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 4));
    // ExtraToVolunteer N=2 (zurueck): -2 Extra, +2 Volunteer.
    roundtrip.push(eh(-2.0, ExtraHoursCategory::ExtraWork, ExtraHoursSource::Rebooking, 5));
    roundtrip.push(eh(2.0, ExtraHoursCategory::VolunteerWork, ExtraHoursSource::Rebooking, 6));

    let (bal_before, vol_before) = balance_and_volunteer(baseline_extra_hours()).await;
    let (bal_after, vol_after) = balance_and_volunteer(roundtrip).await;

    assert!(
        (bal_before - bal_after).abs() < 1e-3,
        "roundtrip balance-neutral: before={bal_before}, after={bal_after}"
    );
    assert!(
        (vol_before - vol_after).abs() < 1e-3,
        "roundtrip volunteer-neutral: before={vol_before}, after={vol_after}"
    );
}

// ─── Pure-fn-Matrizen ─────────────────────────────────────────────────────────

/// HR-ALERT-01 / D-55-01: alert_predicate == cap_active && balance <= -0.5 &&
/// voluntary_ist > 0.0. Grenzfaelle -0.49 / -0.5 / -0.51.
#[test]
fn alert_predicate_matrix() {
    // (balance, voluntary_ist, cap_active) -> expected
    assert!(!alert_predicate(-0.49, 5.0, true), "(-0.49,5,true) must be false");
    assert!(alert_predicate(-0.5, 5.0, true), "(-0.5,5,true) must be true");
    assert!(alert_predicate(-0.51, 5.0, true), "(-0.51,5,true) must be true");
    assert!(!alert_predicate(-3.0, 0.0, true), "(-3,0,true) must be false (no voluntary)");
    assert!(!alert_predicate(-3.0, 5.0, false), "(-3,5,false) must be false (cap inactive)");
}

/// D-55-03: proposed_hours == min(|balance|, voluntary_ist).
#[test]
fn proposed_hours_matrix() {
    assert!((proposed_hours(-10.0, 5.0) - 5.0).abs() < 1e-6, "min(10,5)=5");
    assert!((proposed_hours(-3.0, 10.0) - 3.0).abs() < 1e-6, "min(3,10)=3");
    assert!((proposed_hours(0.0, 5.0) - 0.0).abs() < 1e-6, "min(0,5)=0");
    assert!((proposed_hours(-5.0, 5.0) - 5.0).abs() < 1e-6, "min(5,5)=5");
}

// ─── BL-Service-Tests (mockall) ───────────────────────────────────────────────

mod service_tests {
    use super::*;

    use dao::rebooking_batch::{
        RebookingBatchEntity, RebookingBatchKind, RebookingBatchState,
    };
    use service::rebooking_batch::MockRebookingBatchService;
    use service::reporting::{EmployeeReport, MockReportingService};
    use service::voluntary_stats::{MockVoluntaryStatsService, VoluntaryStats};
    use service::ServiceError;

    use service::rebooking_reconciliation::{RebookingDirection, RebookingReconciliationService};
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
        type PermissionService = MockPermissionService;
        type TransactionDao = dao::MockTransactionDao;
    }

    fn sp_id() -> Uuid {
        Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00BB_0001)
    }

    fn make_batch(kind: RebookingBatchKind, state: RebookingBatchState) -> RebookingBatchEntity {
        RebookingBatchEntity {
            id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00BB_0100),
            sales_person_id: sp_id(),
            iso_year: 2026,
            iso_week: 12,
            kind,
            state,
            created: datetime!(2026 - 03 - 18 09:00:00),
            approved: None,
            approved_by: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    fn make_report(balance: f32, volunteer: f32) -> EmployeeReport {
        EmployeeReport {
            sales_person: Arc::new(make_neutrality_sales_person()),
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
            volunteer_hours: volunteer,
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

    fn make_voluntary_stats(ist: f32) -> VoluntaryStats {
        VoluntaryStats {
            ist_per_contract_week: Some(ist),
            ist_total: Some(ist),
            soll_total: Some(10.0),
            delta: Some(ist - 10.0),
            contract_weeks: Some(1),
            ist_per_soll_pct: Some(50.0),
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

    fn build_service(
        extra_hours_service: MockExtraHoursService,
        rebooking_batch_service: MockRebookingBatchService,
        reporting_service: MockReportingService,
        voluntary_stats_service: MockVoluntaryStatsService,
        permission_service: MockPermissionService,
        transaction_dao: dao::MockTransactionDao,
    ) -> RebookingReconciliationServiceImpl<TestDeps> {
        RebookingReconciliationServiceImpl {
            extra_hours_service: Arc::new(extra_hours_service),
            rebooking_batch_service: Arc::new(rebooking_batch_service),
            reporting_service: Arc::new(reporting_service),
            voluntary_stats_service: Arc::new(voluntary_stats_service),
            permission_service: Arc::new(permission_service),
            transaction_dao: Arc::new(transaction_dao),
        }
    }

    /// REB-MANUAL-01: rebook_manual (VolunteerToExtra) erzeugt genau 2
    /// ExtraHours-Rows (source=Rebooking, -N VolunteerWork + +N ExtraWork) +
    /// 1 rebooking_batch (kind=Manual, state=Approved) mit 1 Entry (out/in gesetzt).
    #[tokio::test]
    async fn rebooking_reconciliation_creates_pair() {
        // ExtraHoursService::create wird genau 2x aufgerufen und liefert die
        // gesetzte Row zurueck (mit source=Rebooking). Wir pruefen category +
        // Vorzeichen + source je Call.
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

        // RebookingBatchService::create wird 1x aufgerufen (kind=Manual,
        // state=Approved) mit genau 1 Entry, dessen out/in-ids gesetzt sind.
        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_create()
            .times(1)
            .returning(|batch, entries, _, _| {
                assert_eq!(batch.kind, RebookingBatchKind::Manual);
                assert_eq!(batch.state, RebookingBatchState::Approved);
                assert_eq!(entries.len(), 1, "exactly one entry per pair");
                assert!(
                    entries[0].extra_hours_out_id.is_some()
                        && entries[0].extra_hours_in_id.is_some(),
                    "entry must carry both pair ids"
                );
                Ok(batch.clone())
            });

        let service = build_service(
            extra_hours_service,
            rebooking_batch_service,
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_ok(),
            tx_ok(),
        );

        service
            .rebook_manual(
                sp_id(),
                RebookingDirection::VolunteerToExtra,
                2.0,
                2026,
                12,
                Authentication::Context(()),
                None,
            )
            .await
            .expect("rebook_manual must succeed");

        let rows = created.lock().unwrap();
        assert_eq!(rows.len(), 2);
        // Genau eine -N VolunteerWork + eine +N ExtraWork, beide Rebooking.
        let out = rows
            .iter()
            .find(|r| r.category == ExtraHoursCategory::VolunteerWork)
            .expect("VolunteerWork out-row");
        let inn = rows
            .iter()
            .find(|r| r.category == ExtraHoursCategory::ExtraWork)
            .expect("ExtraWork in-row");
        assert!((out.amount + 2.0).abs() < 1e-6, "out row must be -2.0, got {}", out.amount);
        assert!((inn.amount - 2.0).abs() < 1e-6, "in row must be +2.0, got {}", inn.amount);
        assert_eq!(out.source, ExtraHoursSource::Rebooking);
        assert_eq!(inn.source, ExtraHoursSource::Rebooking);
    }

    /// REB-MANUAL-02: ExtraToVolunteer schreibt -N ExtraWork + +N VolunteerWork.
    #[tokio::test]
    async fn rebooking_direction_both() {
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
            .returning(|batch, _, _, _| Ok(batch.clone()));

        let service = build_service(
            extra_hours_service,
            rebooking_batch_service,
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_ok(),
            tx_ok(),
        );

        service
            .rebook_manual(
                sp_id(),
                RebookingDirection::ExtraToVolunteer,
                3.0,
                2026,
                12,
                Authentication::Context(()),
                None,
            )
            .await
            .expect("rebook_manual ExtraToVolunteer must succeed");

        let rows = created.lock().unwrap();
        let out = rows
            .iter()
            .find(|r| r.category == ExtraHoursCategory::ExtraWork)
            .expect("ExtraWork out-row");
        let inn = rows
            .iter()
            .find(|r| r.category == ExtraHoursCategory::VolunteerWork)
            .expect("VolunteerWork in-row");
        assert!((out.amount + 3.0).abs() < 1e-6, "ExtraWork out must be -3.0");
        assert!((inn.amount - 3.0).abs() < 1e-6, "VolunteerWork in must be +3.0");
        assert_eq!(out.source, ExtraHoursSource::Rebooking);
        assert_eq!(inn.source, ExtraHoursSource::Rebooking);
    }

    /// REB-MANUAL-01 Atomaritaet: der Batch-Write schlaegt fehl
    /// (EntityAlreadyExists, UNIQUE-Slot belegt) → rebook_manual propagiert den
    /// Fehler; die Tx wird NICHT committet (kein Teil-Commit). Wir pruefen den
    /// Fehler-Pfad; die Rollback-Garantie liegt in der einen Tx (Basic-Calls mit
    /// Some(tx.clone()), commit erst am Ende).
    #[tokio::test]
    async fn rebooking_rollback_on_error() {
        let mut extra_hours_service = MockExtraHoursService::new();
        extra_hours_service
            .expect_create()
            .returning(|eh, _, _| Ok(eh.clone()));

        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_create()
            .returning(|batch, _, _, _| Err(ServiceError::EntityAlreadyExists(batch.id)));

        // commit darf im Fehlerfall NICHT aufgerufen werden.
        let mut transaction_dao = dao::MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(dao::MockTransaction));
        // Kein expect_commit -> falls doch committet wird, panik.

        let service = build_service(
            extra_hours_service,
            rebooking_batch_service,
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_ok(),
            transaction_dao,
        );

        let result = service
            .rebook_manual(
                sp_id(),
                RebookingDirection::VolunteerToExtra,
                2.0,
                2026,
                12,
                Authentication::Context(()),
                None,
            )
            .await;
        assert!(
            matches!(result, Err(ServiceError::EntityAlreadyExists(_))),
            "batch-write failure must propagate (no partial commit), got {result:?}"
        );
    }

    /// T-55-05: Non-HR auf rebook_manual → Forbidden (HR-Gate an erster Stelle,
    /// kein Datenabruf → keine Mock-Expects notwendig).
    #[tokio::test]
    async fn rebook_manual_is_hr_gated() {
        let transaction_dao = dao::MockTransactionDao::new(); // kein use_transaction erwartet
        let service = build_service(
            MockExtraHoursService::new(),
            MockRebookingBatchService::new(),
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_forbidden(),
            transaction_dao,
        );

        let result = service
            .rebook_manual(
                sp_id(),
                RebookingDirection::VolunteerToExtra,
                2.0,
                2026,
                12,
                Authentication::Context(()),
                None,
            )
            .await;
        assert!(
            matches!(result, Err(ServiceError::Forbidden)),
            "non-HR rebook_manual must be Forbidden, got {result:?}"
        );
    }

    /// HR-ALERT-02 + HR-ALERT-04 (Modell B Claim): suggest auf leerem Slot →
    /// create_pending_suggestion erzeugt pending-Batch; die zurueckgegebene
    /// RebookingSuggestionTO traegt IST + DANN backend-computed (proposed_hours =
    /// min(|balance|, voluntary_ist)) + die neue pending-id.
    #[tokio::test]
    async fn suggest_claims_pending_and_returns_dann() {
        // Kein offener pending-Batch → find_pending liefert None.
        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_find_pending_for_sales_person()
            .returning(|_, _, _, _, _| Ok(None));
        let pending = make_batch(RebookingBatchKind::HrSuggestion, RebookingBatchState::Pending);
        let pending_id = pending.id;
        rebooking_batch_service
            .expect_create_pending_suggestion()
            .times(1)
            .returning(move |_, _, _, _, _| Ok(pending.clone()));

        // ReportingService liefert balance = -8.0.
        let mut reporting_service = MockReportingService::new();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(|_, _, _, _, _, _| Ok(make_report(-8.0, 5.0)));

        // VoluntaryStats liefert ist_total = 5.0.
        let mut voluntary_stats_service = MockVoluntaryStatsService::new();
        voluntary_stats_service
            .expect_get_voluntary_stats()
            .returning(|_, _, _, _, _| Ok(make_voluntary_stats(5.0)));

        let service = build_service(
            MockExtraHoursService::new(),
            rebooking_batch_service,
            reporting_service,
            voluntary_stats_service,
            perm_ok(),
            tx_ok(),
        );

        let suggestion = service
            .suggest(sp_id(), 2026, 12, Authentication::Context(()), None)
            .await
            .expect("suggest must succeed on empty slot");

        assert_eq!(suggestion.pending_rebooking_id, pending_id);
        // proposed_hours = min(|-8|, 5) = 5.
        assert!(
            (suggestion.proposed_hours - 5.0).abs() < 1e-6,
            "proposed_hours must be min(8,5)=5, got {}",
            suggestion.proposed_hours
        );
        // IST-Felder.
        assert!((suggestion.balance - (-8.0)).abs() < 1e-6);
        assert!((suggestion.voluntary_ist - 5.0).abs() < 1e-6);
        // DANN-Felder backend-computed: balance_after = balance + proposed,
        // voluntary_ist_after = voluntary_ist - proposed.
        assert!(
            (suggestion.balance_after - (-3.0)).abs() < 1e-6,
            "balance_after must be -8+5=-3, got {}",
            suggestion.balance_after
        );
        assert!(
            (suggestion.voluntary_ist_after - 0.0).abs() < 1e-6,
            "voluntary_ist_after must be 5-5=0, got {}",
            suggestion.voluntary_ist_after
        );
    }

    /// Modell B idempotent: existiert bereits ein pending-Batch fuer den Slot,
    /// erzeugt suggest KEINEN zweiten Claim → liefert dieselbe pending-id.
    #[tokio::test]
    async fn suggest_is_idempotent_on_existing_pending() {
        let existing = make_batch(RebookingBatchKind::HrSuggestion, RebookingBatchState::Pending);
        let existing_id = existing.id;
        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_find_pending_for_sales_person()
            .returning(move |_, _, _, _, _| Ok(Some(existing.clone())));
        // create_pending_suggestion darf NICHT aufgerufen werden (kein Expect).

        let mut reporting_service = MockReportingService::new();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(|_, _, _, _, _, _| Ok(make_report(-8.0, 5.0)));

        let mut voluntary_stats_service = MockVoluntaryStatsService::new();
        voluntary_stats_service
            .expect_get_voluntary_stats()
            .returning(|_, _, _, _, _| Ok(make_voluntary_stats(5.0)));

        let service = build_service(
            MockExtraHoursService::new(),
            rebooking_batch_service,
            reporting_service,
            voluntary_stats_service,
            perm_ok(),
            tx_ok(),
        );

        let suggestion = service
            .suggest(sp_id(), 2026, 12, Authentication::Context(()), None)
            .await
            .expect("suggest idempotent must succeed");
        assert_eq!(
            suggestion.pending_rebooking_id, existing_id,
            "idempotent suggest must return existing pending id"
        );
    }

    /// D-55-B: bei bereits entschiedenem (approved|rejected) Batch im Slot →
    /// create_pending_suggestion propagiert EntityAlreadyExists → Conflict (409).
    #[tokio::test]
    async fn suggest_on_decided_slot_is_conflict() {
        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_find_pending_for_sales_person()
            .returning(|_, _, _, _, _| Ok(None));
        rebooking_batch_service
            .expect_create_pending_suggestion()
            .returning(|_, _, _, _, _| Err(ServiceError::EntityAlreadyExists(Uuid::nil())));

        let mut reporting_service = MockReportingService::new();
        reporting_service
            .expect_get_report_for_employee_range()
            .returning(|_, _, _, _, _, _| Ok(make_report(-8.0, 5.0)));

        let mut voluntary_stats_service = MockVoluntaryStatsService::new();
        voluntary_stats_service
            .expect_get_voluntary_stats()
            .returning(|_, _, _, _, _| Ok(make_voluntary_stats(5.0)));

        let service = build_service(
            MockExtraHoursService::new(),
            rebooking_batch_service,
            reporting_service,
            voluntary_stats_service,
            perm_ok(),
            tx_ok(),
        );

        let result = service
            .suggest(sp_id(), 2026, 12, Authentication::Context(()), None)
            .await;
        assert!(
            matches!(result, Err(ServiceError::EntityAlreadyExists(_))),
            "decided slot must be conflict, got {result:?}"
        );
    }

    /// HR-ALERT-03 (D-55-B): approve → update_state(Approved) ZUERST, dann genau
    /// 2 Pair-ExtraHours (source=Rebooking) via rebook.
    #[tokio::test]
    async fn approve_after_suggest_writes_pair() {
        let batch_id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00BB_0100);
        let pending = make_batch(RebookingBatchKind::HrSuggestion, RebookingBatchState::Pending);

        let mut rebooking_batch_service = MockRebookingBatchService::new();
        // find_by_id liefert den pending-Batch (fuer die week/direction/hours-Ableitung).
        rebooking_batch_service
            .expect_find_by_id()
            .returning(move |_, _, _| Ok(Some(pending.clone())));
        // update_state ZUERST (Approved) — genau 1x.
        rebooking_batch_service
            .expect_update_state()
            .times(1)
            .returning(|_, state, _, _, _| {
                assert_eq!(state, RebookingBatchState::Approved);
                Ok(())
            });
        // Danach der Pair-Batch-Create (kind=HrSuggestion via rebook)? Nein —
        // approve transitioniert den bestehenden Batch; der Pair-Write geht nur
        // ueber ExtraHoursService (2x) + Entry-Update. Kein zweiter Batch-create.
        // Wir erlauben create NICHT.

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

        let service = build_service(
            extra_hours_service,
            rebooking_batch_service,
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_ok(),
            tx_ok(),
        );

        service
            .approve(
                batch_id,
                2.0,
                Authentication::Context(()),
                None,
            )
            .await
            .expect("approve must succeed");

        assert_eq!(created.lock().unwrap().len(), 2, "approve must write exactly the pair");
        for row in created.lock().unwrap().iter() {
            assert_eq!(row.source, ExtraHoursSource::Rebooking);
        }
    }

    /// HR-ALERT-03: reject → update_state(Rejected), KEINE Pair-Rows.
    #[tokio::test]
    async fn reject_after_suggest_no_pair() {
        let mut rebooking_batch_service = MockRebookingBatchService::new();
        rebooking_batch_service
            .expect_update_state()
            .times(1)
            .returning(|_, state, _, _, _| {
                assert_eq!(state, RebookingBatchState::Rejected);
                Ok(())
            });

        // ExtraHoursService::create darf NICHT aufgerufen werden (kein Expect).
        let extra_hours_service = MockExtraHoursService::new();

        let service = build_service(
            extra_hours_service,
            rebooking_batch_service,
            MockReportingService::new(),
            MockVoluntaryStatsService::new(),
            perm_ok(),
            tx_ok(),
        );

        service
            .reject(
                Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_00BB_0100),
                Authentication::Context(()),
                None,
            )
            .await
            .expect("reject must succeed");
    }
}
