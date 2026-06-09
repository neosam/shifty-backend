//! Additiver-Merge-Test (Phase 8.4 / SC-1). Verifiziert dass
//! `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` aus BEIDEN Quellen
//! additiv summiert werden: lebende `extra_hours` (deleted IS NULL) PLUS
//! `AbsenceService::derive_hours_for_range`. Es gibt keinen globalen
//! Quellen-Schalter mehr (M-03) — der Flag `absence_range_source_active` wird
//! im Reporting-Pfad nicht mehr gelesen.
//!
//! WICHTIG (Test-first): Diese Tests sind RED bis Plan 02 den Produktionscode
//! in `reporting.rs` auf den additiven Merge umstellt (Flag-Read entfernt,
//! `derive_hours_for_range` unbedingt aufgerufen, beide Quellen addiert).
//!
//! Kritische Mock-Erwartungen pro Szenario:
//! - `feature_flag_service.expect_is_enabled().times(0)` — kein Flag-Read mehr.
//! - `absence_service.expect_derive_hours_for_range()` IMMER (auch bei leeren
//!   extra_hours) — kein `times(0)`-Trap.
//! - `extra_hours_service.expect_find_by_sales_person_id_and_year_range()`
//!   immer erwartet (kann leeren Slice liefern).

use std::collections::BTreeMap;
use std::sync::Arc;

use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::{AbsenceCategory, MockAbsenceService, ResolvedAbsence};
use service::carryover::MockCarryoverService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, MockExtraHoursService};
use service::feature_flag::MockFeatureFlagService;
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::MockSalesPersonService;
use service::shiftplan_report::MockShiftplanReportService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use shifty_utils::ShiftyDate;

use crate::reporting::{ReportingServiceDeps, ReportingServiceImpl};
use crate::test::reporting_phase2_fixtures::{fixture_sales_person, fixture_sales_person_id};

struct ReportingMocks {
    extra_hours_service: MockExtraHoursService,
    shiftplan_report_service: MockShiftplanReportService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    sales_person_service: MockSalesPersonService,
    carryover_service: MockCarryoverService,
    permission_service: MockPermissionService,
    clock_service: MockClockService,
    uuid_service: MockUuidService,
    feature_flag_service: MockFeatureFlagService,
    absence_service: MockAbsenceService,
    transaction_dao: dao::MockTransactionDao,
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
    type FeatureFlagService = MockFeatureFlagService;
    type AbsenceService = MockAbsenceService;
    type TransactionDao = dao::MockTransactionDao;
}

impl ReportingMocks {
    fn new() -> Self {
        Self {
            extra_hours_service: MockExtraHoursService::new(),
            shiftplan_report_service: MockShiftplanReportService::new(),
            employee_work_details_service: MockEmployeeWorkDetailsService::new(),
            sales_person_service: MockSalesPersonService::new(),
            carryover_service: MockCarryoverService::new(),
            permission_service: MockPermissionService::new(),
            clock_service: MockClockService::new(),
            uuid_service: MockUuidService::new(),
            feature_flag_service: MockFeatureFlagService::new(),
            absence_service: MockAbsenceService::new(),
            transaction_dao: dao::MockTransactionDao::new(),
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
            feature_flag_service: Arc::new(self.feature_flag_service),
            absence_service: Arc::new(self.absence_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

fn make_extra_hours(category: ExtraHoursCategory, amount: f32, day: time::Date) -> ExtraHours {
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sales_person_id(),
        amount,
        category,
        description: Arc::from(""),
        date_time: time::PrimitiveDateTime::new(day, time::Time::from_hms(9, 0, 0).unwrap()),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Setzt die Standard-Boilerplate-Mocks, die in JEDEM additiven Szenario
/// identisch sind (permission, sales_person, work_details, shiftplan_report,
/// carryover, transaction) und macht explizit, dass der Feature-Flag nicht
/// mehr gelesen wird (`is_enabled().times(0)`).
fn setup_common_mocks(mocks: &mut ReportingMocks) {
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

    // employee_work_details + shiftplan_report: leer (isoliert Stunden-Aggregation).
    mocks
        .employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(|_, _, _| Ok(Arc::from(vec![])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![])));

    // carryover: None.
    mocks
        .carryover_service
        .expect_get_carryover()
        .returning(|_, _, _, _| Ok(None));

    // transaction: passthrough.
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));

    // Phase 8.4: KEIN Flag-Read mehr im Reporting-Pfad (M-03).
    mocks.feature_flag_service.expect_is_enabled().times(0);
}

async fn run_report(service: ReportingServiceImpl<TestDeps>) -> service::reporting::EmployeeReport {
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
        .expect("additiver Reporting-Pfad muss erfolgreich durchlaufen")
}

/// Szenario 1: Nur `extra_hours` vorhanden (keine absence_period).
/// derive_hours_for_range -> leere BTreeMap.
/// extra_hours -> [Vacation 8h, SickLeave 4h, UnpaidLeave 2h].
/// Erwartung: vacation=8, sick=4, unpaid=2 (= ExtraHours-Summe + 0).
#[tokio::test]
async fn test_additive_only_extra_hours() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    // absence_period leer — derive_hours_for_range wird trotzdem aufgerufen.
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    let extras = vec![
        make_extra_hours(ExtraHoursCategory::Vacation, 8.0, date!(2024 - 06 - 03)),
        make_extra_hours(ExtraHoursCategory::SickLeave, 4.0, date!(2024 - 06 - 04)),
        make_extra_hours(ExtraHoursCategory::UnpaidLeave, 2.0, date!(2024 - 06 - 05)),
    ];
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extras);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "nur extra_hours: vacation = 8h ExtraHours + 0h absence_period"
    );
    assert_eq!(
        report.sick_leave_hours, 4.0,
        "nur extra_hours: sick_leave = 4h ExtraHours + 0h absence_period"
    );
    assert_eq!(
        report.unpaid_leave_hours, 2.0,
        "nur extra_hours: unpaid_leave = 2h ExtraHours + 0h absence_period"
    );
}

/// Szenario 2: Nur absence_period vorhanden (keine extra_hours).
/// derive_hours_for_range -> {2024-06-03: Vacation 8h, 2024-06-04: SickLeave 8h}.
/// extra_hours -> leerer Slice.
/// Erwartung: vacation=8, sick=8, unpaid=0.
#[tokio::test]
async fn test_additive_only_absence_period() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        },
    );
    derived.insert(
        date!(2024 - 06 - 04),
        ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    // extra_hours leer.
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "nur absence_period: vacation = 0h ExtraHours + 8h absence_period"
    );
    assert_eq!(
        report.sick_leave_hours, 8.0,
        "nur absence_period: sick_leave = 0h ExtraHours + 8h absence_period"
    );
    assert_eq!(
        report.unpaid_leave_hours, 0.0,
        "nur absence_period: unpaid_leave = 0 (kein UnpaidLeave aktiv)"
    );
}

/// Szenario 3 (Kern-SC-1): Beide Quellen, distinkte Tage -> additive Summe.
/// derive_hours_for_range -> {2024-06-03: Vacation 8h}.
/// extra_hours -> [Vacation 4h am 2024-06-05 (anderer Tag, lebt noch)].
/// Erwartung: vacation = 12 (4h ExtraHours + 8h absence_period).
#[tokio::test]
async fn test_additive_both_sources_distinct_days() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    let extras = vec![make_extra_hours(
        ExtraHoursCategory::Vacation,
        4.0,
        date!(2024 - 06 - 05),
    )];
    let extras_arc: Arc<[ExtraHours]> = Arc::from(extras);
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(move |_, _, _, _, _| Ok(extras_arc.clone()));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 12.0,
        "beide Quellen distinkte Tage: vacation = 4h ExtraHours + 8h absence_period = 12h"
    );
}

/// Szenario 4: Konvertierte (soft-deleted) Rows zählen nicht doppelt.
/// Die per-row-Invariante (D-02): konvertierte extra_hours sind via
/// `deleted IS NULL` bereits aus dem DAO-Load ausgeschlossen — der Mock liefert
/// daher schlicht [] für den konvertierten Tag. Kein neuer DAO-Code nötig.
/// extra_hours -> leerer Slice.
/// derive_hours_for_range -> {2024-06-03: Vacation 8h}.
/// Erwartung: vacation = 8 (nur einmal gezählt).
#[tokio::test]
async fn test_additive_converted_rows_not_double_counted() {
    let mut mocks = ReportingMocks::new();
    setup_common_mocks(&mut mocks);

    let mut derived = BTreeMap::new();
    derived.insert(
        date!(2024 - 06 - 03),
        ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        },
    );
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .times(1)
        .returning(move |_, _, _, _, _| Ok(derived.clone()));

    // Konvertierte Row ist soft-deleted -> faellt per `deleted IS NULL` im DAO
    // heraus -> Mock liefert leeren Slice.
    mocks
        .extra_hours_service
        .expect_find_by_sales_person_id_and_year_range()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));

    let report = run_report(mocks.build()).await;

    assert_eq!(
        report.vacation_hours, 8.0,
        "konvertierte Rows zählen nicht doppelt: vacation = 8h (nur aus absence_period)"
    );
}
