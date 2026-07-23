//! Phase 55 (HR-ALERT-01/03, 55-03): Tests fuer die `has_pending_rebooking`-
//! Berechnung im short-report (`get_reports_for_all_employees`).
//!
//! F5-Modell-B-Banner-Regel (D-55-B + D-55-07):
//! `has_pending_rebooking = alert_predicate(balance, voluntary_ist, cap_active)
//!  UND kein entschiedener (approved|rejected) Batch fuer (sp, aktuelle iso_woche)`.
//! `pending_rebooking_id = Some(id)` falls ein pending-Batch offen ist, sonst None.
//!
//! Vier Behaviors:
//! - predicate_true_no_batch → TRUE / None (Banner zeigt sich, HR loest Suggest aus)
//! - predicate_true_pending_batch → TRUE / Some(id) (offener Vorschlag, Banner bleibt)
//! - decided_batch_suppresses → FALSE / None (approved ODER rejected loescht Alert)
//! - predicate_false → FALSE / None (kein Alert, kein Lookup noetig)

use std::collections::BTreeMap;
use std::sync::Arc;

use dao::rebooking_batch::{RebookingBatchEntity, RebookingBatchKind, RebookingBatchState};
use service::absence::MockAbsenceService;
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::{EmployeeWorkDetails, MockEmployeeWorkDetailsService};
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::permission::Authentication;
use service::rebooking_batch::MockRebookingBatchService;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::special_days::MockSpecialDayService;
use service::toggle::MockToggleService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use uuid::Uuid;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person, fixture_sales_person_id, fixture_work_details_8h_mon_fri,
};

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
    type SpecialDayService = MockSpecialDayService;
    type ToggleService = MockToggleService;
    type RebookingBatchService = MockRebookingBatchService;
}

/// Cap-aktiver 40h-Vertrag KW22-25/2024 (deckt KW23). Ohne Shiftplan ergibt
/// das balance = -40 (weit unter -0.5) → Praedikat-Teil erfuellt.
fn capped_work_details() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        cap_planned_hours_to_expected: true,
        ..fixture_work_details_8h_mon_fri()
    }
}

/// VolunteerWork +5h am Do 2024-06-06 (KW23) → voluntary_ist > 0.
fn volunteer_entry() -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x5555),
        sales_person_id: fixture_sales_person_id(),
        amount: 5.0,
        category: ExtraHoursCategory::VolunteerWork,
        description: Arc::from("fixture volunteer"),
        date_time: time::macros::datetime!(2024 - 06 - 06 09:00:00),
        created: Some(time::macros::datetime!(2024 - 06 - 06 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        source: service::extra_hours::ExtraHoursSource::Manual,
    }
}

/// Ein pending HrSuggestion-Batch im Slot (sp, 2024, KW23).
fn pending_batch(id: Uuid) -> RebookingBatchEntity {
    RebookingBatchEntity {
        id,
        sales_person_id: fixture_sales_person_id(),
        iso_year: 2024,
        iso_week: 23,
        kind: RebookingBatchKind::HrSuggestion,
        state: RebookingBatchState::Pending,
        created: time::macros::datetime!(2024 - 06 - 06 09:00:00),
        approved: None,
        approved_by: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Ein entschiedener (approved|rejected) Batch im Slot.
fn decided_batch(id: Uuid, state: RebookingBatchState) -> RebookingBatchEntity {
    RebookingBatchEntity {
        state,
        ..pending_batch(id)
    }
}

/// Baut den ReportingService fuer den Summary-Pfad. `extra_hours` steuert
/// voluntary_ist; `cap` steuert cap_active; `configure_rebooking` verdrahtet
/// die RebookingBatchService-Mock-Erwartungen (nur relevant wenn das Praedikat
/// true ist — sonst wird der Lookup nie aufgerufen).
fn build_summary_service<F>(
    cap_active: bool,
    extra_hours: Vec<ExtraHours>,
    configure_rebooking: F,
) -> ReportingServiceImpl<TestDeps>
where
    F: FnOnce(&mut MockRebookingBatchService),
{
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_get_all()
        .returning(|_, _| Ok(Arc::from(vec![fixture_sales_person()])));

    let work_details = if cap_active {
        vec![capped_work_details()]
    } else {
        vec![fixture_work_details_8h_mon_fri()]
    };
    let mut employee_work_details_service = MockEmployeeWorkDetailsService::new();
    employee_work_details_service
        .expect_all()
        .returning(move |_, _| Ok(Arc::from(work_details.clone())));

    let mut shiftplan_report_service = MockShiftplanReportService::new();
    shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(move |_, _, _, _, _| Ok(Arc::from(Vec::new())));

    let mut extra_hours_service = MockExtraHoursService::new();
    extra_hours_service
        .expect_find_by_sales_person_id_and_year()
        .returning(move |_, _, _, _, _| Ok(Arc::from(extra_hours.clone())));

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

    let mut rebooking_batch_service = MockRebookingBatchService::new();
    configure_rebooking(&mut rebooking_batch_service);

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
        rebooking_batch_service: Arc::new(rebooking_batch_service),
    }
}

/// Praedikat true (cap aktiv, balance<=-0.5, voluntary_ist>0) UND KEIN
/// entschiedener Batch UND kein pending → has_pending_rebooking=TRUE, id=None.
#[tokio::test]
async fn short_report_predicate_true_no_batch() {
    let service = build_summary_service(true, vec![volunteer_entry()], |m| {
        // decided-Lookup: kein Batch entschieden.
        m.expect_find_by_sales_person_year_week()
            .returning(|_, _, _, _, _| Ok(None));
        // pending-Lookup: kein offener pending-Batch.
        m.expect_find_pending_for_sales_person()
            .returning(|_, _, _, _, _| Ok(None));
    });

    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("summary report must succeed");

    let r = &reports[0];
    assert!(
        r.has_pending_rebooking,
        "predicate true + no decided batch → banner shows (TRUE), got false; balance={}, volunteer={}",
        r.balance_hours, r.volunteer_hours
    );
    assert_eq!(r.pending_rebooking_id, None, "no open pending batch → id None");
}

/// Praedikat true UND ein pending HrSuggestion-Batch existiert →
/// has_pending_rebooking=TRUE, pending_rebooking_id=Some(id).
#[tokio::test]
async fn short_report_predicate_true_pending_batch() {
    let batch_id = Uuid::from_u128(0xBEEF);
    let service = build_summary_service(true, vec![volunteer_entry()], move |m| {
        m.expect_find_by_sales_person_year_week()
            .returning(|_, _, _, _, _| Ok(None));
        m.expect_find_pending_for_sales_person()
            .returning(move |_, _, _, _, _| Ok(Some(pending_batch(batch_id))));
    });

    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("summary report must succeed");

    let r = &reports[0];
    assert!(r.has_pending_rebooking, "pending batch present → banner stays TRUE");
    assert_eq!(
        r.pending_rebooking_id,
        Some(batch_id),
        "open pending batch → id Some(batch.id)"
    );
}

/// Ein approved-ODER-rejected-Batch existiert im Slot → Alert erlischt
/// (D-55-07): has_pending_rebooking=FALSE, id=None.
#[tokio::test]
async fn short_report_decided_batch_suppresses() {
    for state in [RebookingBatchState::Approved, RebookingBatchState::Rejected] {
        let batch_id = Uuid::from_u128(0xCAFE);
        let service = build_summary_service(true, vec![volunteer_entry()], move |m| {
            m.expect_find_by_sales_person_year_week()
                .returning(move |_, _, _, _, _| Ok(Some(decided_batch(batch_id, state))));
            // find_pending darf hier NICHT aufgerufen werden (decided suppress
            // vor dem pending-Lookup); keine Erwartung registriert.
        });

        let reports = service
            .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
            .await
            .expect("summary report must succeed");

        let r = &reports[0];
        assert!(
            !r.has_pending_rebooking,
            "decided batch ({:?}) suppresses the alert (FALSE)",
            state
        );
        assert_eq!(r.pending_rebooking_id, None, "decided → id None");
    }
}

/// Praedikat false (kein cap → cap_active=false) → has_pending_rebooking=false,
/// id=None; KEIN Rebooking-Lookup (keine Mock-Erwartung registriert).
#[tokio::test]
async fn short_report_predicate_false() {
    // cap_active=false ⇒ alert_predicate false, egal wie negativ die Balance ist.
    let service = build_summary_service(false, vec![volunteer_entry()], |_m| {
        // Keine Rebooking-Erwartung: bei false-Praedikat darf kein Lookup laufen.
    });

    let reports = service
        .get_reports_for_all_employees(2024, 23, Authentication::Full, None)
        .await
        .expect("summary report must succeed");

    let r = &reports[0];
    assert!(
        !r.has_pending_rebooking,
        "predicate false (cap inactive) → no alert"
    );
    assert_eq!(r.pending_rebooking_id, None);
}
