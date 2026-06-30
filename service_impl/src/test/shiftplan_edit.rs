//! Mock-basierte Service-Tests für `ShiftplanEditServiceImpl::book_slot_with_conflict_check`
//! und `::copy_week_with_conflict_check` (Plan 03-04, Wave 3).
//!
//! Pflicht-Coverage (aus 03-VALIDATION.md):
//!  - `test_book_slot_warning_on_absence_day`        (BOOK-02 / D-Phase3-14 BookingOnAbsenceDay)
//!  - `test_book_slot_warning_on_manual_unavailable` (BOOK-02 / D-Phase3-14 BookingOnUnavailableDay)
//!  - `test_book_slot_no_warning_when_softdeleted_absence` (SC4 / Pitfall-1)
//!  - `test_copy_week_aggregates_warnings`           (BOOK-02 / D-Phase3-02, D-Phase3-15: KEINE De-Dup)
//!  - `test_book_slot_with_conflict_check_forbidden` (D-24-04 Shiftplanner ∨ self)
//!  - `test_copy_week_with_conflict_check_forbidden` (D-Phase3-12 — bulk-Op fordert shiftplan.edit)
//!
//! Phase 24 (D-24-02, D-24-04, D-24-08) neue Tests:
//!  - `test_hard_block_non_shiftplanner_over_limit`   (toggle ON + non-SP over limit → PaidLimitExceeded)
//!  - `test_hard_block_shiftplanner_bypasses`         (toggle ON + SP → Ok, no block)
//!  - `test_soft_mode_over_limit_warns_not_blocks`    (toggle OFF → soft warning, no error)
//!  - `test_hard_block_unpaid_never_blocked`          (toggle ON + unpaid person → Ok)
//!
//! Mock-DI-Setup analog `service_impl/src/test/booking.rs:113-192` und
//! `service_impl/src/test/absence.rs:147-260`.

use std::sync::{Arc, Mutex};

use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::{
    absence::{AbsenceCategory, AbsencePeriod, DayFraction, MockAbsenceService},
    booking::{Booking, MockBookingService},
    carryover::MockCarryoverService,
    employee_work_details::MockEmployeeWorkDetailsService,
    extra_hours::MockExtraHoursService,
    reporting::MockReportingService,
    sales_person::{MockSalesPersonService, SalesPerson},
    sales_person_unavailable::{MockSalesPersonUnavailableService, SalesPersonUnavailable},
    shiftplan_edit::ShiftplanEditService,
    slot::{MockSlotService, Slot},
    toggle::MockToggleService,
    uuid_service::MockUuidService,
    warning::Warning,
    MockPermissionService, ServiceError,
};
use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use time::{Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

use crate::shiftplan_edit::{ShiftplanEditServiceDeps, ShiftplanEditServiceImpl};
use crate::test::error_test::{test_forbidden, NoneTypeExt};

// ---------- Fixtures ----------

fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

fn default_slot_id() -> Uuid {
    uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380")
}

fn default_booking_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD0")
}

fn default_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}

fn default_absence_id() -> Uuid {
    uuid!("AB000000-0000-0000-0000-000000000001")
}

fn default_unavailable_id() -> Uuid {
    uuid!("CC000000-0000-0000-0000-000000000010")
}

fn monday_slot() -> Slot {
    Slot {
        id: default_slot_id(),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(9, 0, 0).unwrap(),
        to: Time::from_hms(17, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: default_version(),
        shiftplan_id: None,
    }
}

/// 2026-W17 Monday — `time::Date::from_iso_week_date(2026, 17, Monday)` =
/// 2026-04-20.
fn default_booking() -> Booking {
    Booking {
        id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        slot_id: default_slot_id(),
        calendar_week: 17,
        year: 2026,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    }
}

fn persisted_booking() -> Booking {
    Booking {
        id: default_booking_id(),
        version: default_version(),
        created: Some(PrimitiveDateTime::new(
            time::Date::from_calendar_date(2026, Month::April, 20).unwrap(),
            Time::from_hms(0, 0, 0).unwrap(),
        )),
        ..default_booking()
    }
}

fn default_absence_period() -> AbsencePeriod {
    AbsencePeriod {
        id: default_absence_id(),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2026 - 04 - 20),
        to_date: date!(2026 - 04 - 24),
        description: "Urlaub".into(),
        created: Some(datetime!(2026 - 04 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-000000000099"),
        day_fraction: DayFraction::Full,
    }
}

fn default_manual_unavailable() -> SalesPersonUnavailable {
    SalesPersonUnavailable {
        id: default_unavailable_id(),
        sales_person_id: default_sales_person_id(),
        year: 2026,
        calendar_week: 17,
        day_of_week: DayOfWeek::Monday,
        created: Some(datetime!(2026 - 04 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-000000000100"),
    }
}

// ---------- DI-Setup ----------

pub(crate) struct ShiftplanEditDependencies {
    pub permission_service: MockPermissionService,
    pub slot_service: MockSlotService,
    pub booking_service: MockBookingService,
    pub carryover_service: MockCarryoverService,
    pub reporting_service: MockReportingService,
    pub sales_person_service: MockSalesPersonService,
    pub sales_person_unavailable_service: MockSalesPersonUnavailableService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub extra_hours_service: MockExtraHoursService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
    pub absence_service: MockAbsenceService,
    pub toggle_service: MockToggleService,
}

impl ShiftplanEditServiceDeps for ShiftplanEditDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type PermissionService = MockPermissionService;
    type SlotService = MockSlotService;
    type BookingService = MockBookingService;
    type CarryoverService = MockCarryoverService;
    type ReportingService = MockReportingService;
    type SalesPersonService = MockSalesPersonService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type ExtraHoursService = MockExtraHoursService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
    type AbsenceService = MockAbsenceService;
    type ToggleService = MockToggleService;
}

impl ShiftplanEditDependencies {
    pub(crate) fn build_service(self) -> ShiftplanEditServiceImpl<ShiftplanEditDependencies> {
        ShiftplanEditServiceImpl {
            permission_service: self.permission_service.into(),
            slot_service: self.slot_service.into(),
            booking_service: self.booking_service.into(),
            carryover_service: self.carryover_service.into(),
            reporting_service: self.reporting_service.into(),
            sales_person_service: self.sales_person_service.into(),
            sales_person_unavailable_service: self.sales_person_unavailable_service.into(),
            employee_work_details_service: self.employee_work_details_service.into(),
            extra_hours_service: self.extra_hours_service.into(),
            uuid_service: self.uuid_service.into(),
            transaction_dao: self.transaction_dao.into(),
            absence_service: self.absence_service.into(),
            toggle_service: self.toggle_service.into(),
        }
    }
}

/// `permission_grants_shiftplanner` ⇒ Shiftplanner-Probe liefert Ok; sonst Forbidden.
/// (D-24-04: gate wurde von HR auf Shiftplanner korrigiert.)
/// `verify_grants_self` ⇒ verify_user_is_sales_person liefert Ok; sonst Forbidden.
pub(crate) fn build_dependencies(
    permission_grants_shiftplanner: bool,
    verify_grants_self: bool,
) -> ShiftplanEditDependencies {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |_, _| {
            if permission_grants_shiftplanner {
                Ok(())
            } else {
                Err(ServiceError::Forbidden)
            }
        });
    // Default: book_slot_with_conflict_check resolves the originator for
    // created_by attribution. Tests that exercise system pathways
    // (Authentication::Full) can override this.
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("test-user".into())));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(move |_, _, _| {
            if verify_grants_self {
                Ok(())
            } else {
                Err(ServiceError::Forbidden)
            }
        });

    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(monday_slot()));

    // Defaults: leere Konflikt-Quellen — Tests, die Warnings asserten,
    // überschreiben diese mit `.checkpoint()` + neuer expect.
    let mut absence_service = MockAbsenceService::new();
    absence_service
        .expect_find_overlapping_for_booking()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<AbsencePeriod>::new())));

    let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
    sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<SalesPersonUnavailable>::new())));

    let mut booking_service = MockBookingService::new();
    booking_service
        .expect_create()
        .returning(|_, _, _| Ok(persisted_booking()));
    booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<Booking>::new())));

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    // Default toggle: soft mode (false). Tests that exercise the hard-block must
    // override via `.checkpoint()` + new `expect_is_enabled()`.
    // Note: toggle is only called when slot.max_paid_employees is Some(_); the
    // default `monday_slot()` has None, so tests without a slot limit do not trigger
    // the toggle mock at all.
    let mut toggle_service = MockToggleService::new();
    toggle_service
        .expect_is_enabled()
        .returning(|_, _, _| Ok(false)); // soft mode by default → existing tests unaffected

    ShiftplanEditDependencies {
        permission_service,
        slot_service,
        booking_service,
        carryover_service: MockCarryoverService::new(),
        reporting_service: MockReportingService::new(),
        sales_person_service,
        sales_person_unavailable_service,
        employee_work_details_service: MockEmployeeWorkDetailsService::new(),
        extra_hours_service: MockExtraHoursService::new(),
        uuid_service: MockUuidService::new(),
        transaction_dao,
        absence_service,
        toggle_service,
    }
}

// ---------- Tests ----------

#[tokio::test]
async fn test_book_slot_warning_on_absence_day() {
    let mut deps = build_dependencies(true, false);
    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_overlapping_for_booking()
        .returning(|_, _, _, _| Ok(Arc::from(vec![default_absence_period()])));
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert_eq!(result.booking, persisted_booking());
    assert_eq!(result.warnings.len(), 1);
    match &result.warnings[0] {
        Warning::BookingOnAbsenceDay {
            booking_id,
            date,
            absence_id,
            category,
        } => {
            assert_eq!(*booking_id, default_booking_id());
            assert_eq!(*date, date!(2026 - 04 - 20));
            assert_eq!(*absence_id, default_absence_id());
            assert_eq!(*category, AbsenceCategory::Vacation);
        }
        other => panic!("expected BookingOnAbsenceDay, got {other:?}"),
    }
}

#[tokio::test]
async fn test_book_slot_warning_on_manual_unavailable() {
    let mut deps = build_dependencies(true, false);
    deps.sales_person_unavailable_service.checkpoint();
    deps.sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| Ok(Arc::from(vec![default_manual_unavailable()])));
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert_eq!(result.warnings.len(), 1);
    match &result.warnings[0] {
        Warning::BookingOnUnavailableDay {
            booking_id,
            year,
            week,
            day_of_week,
        } => {
            assert_eq!(*booking_id, default_booking_id());
            assert_eq!(*year, 2026);
            assert_eq!(*week, 17);
            assert_eq!(*day_of_week, DayOfWeek::Monday);
        }
        other => panic!("expected BookingOnUnavailableDay, got {other:?}"),
    }
}

/// Pitfall 1 / SC4: Soft-deleted AbsencePeriod darf KEINE Warning erzeugen.
/// DAO-Layer filtert `deleted IS NULL` — `find_overlapping_for_booking`
/// liefert hier eine leere Liste; doppelt-defensiv via Mock-Default.
/// Gleichzeitig wird ein soft-deleted ManualUnavailable in den Lookup
/// eingespeist und MUSS vom Service-Layer ignoriert werden (client-side
/// `deleted.is_none()`-Check im Reverse-Warning-Loop).
#[tokio::test]
async fn test_book_slot_no_warning_when_softdeleted_absence() {
    let mut deps = build_dependencies(true, false);
    // DAO (via AbsenceService) liefert nichts — soft-deleted Eintrag wurde
    // im SQL-Layer schon gefiltert.
    // SalesPersonUnavailable: Wir injizieren explizit einen soft-deleted
    // Datensatz, der vom Service-Layer client-side ignoriert werden MUSS.
    deps.sales_person_unavailable_service.checkpoint();
    deps.sales_person_unavailable_service
        .expect_get_by_week_for_sales_person()
        .returning(|_, _, _, _, _| {
            Ok(Arc::from(vec![SalesPersonUnavailable {
                deleted: Some(datetime!(2026 - 04 - 02 09:00:00)),
                ..default_manual_unavailable()
            }]))
        });
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert!(
        result.warnings.is_empty(),
        "soft-deleted sources MUST NOT trigger warnings, got {:?}",
        result.warnings
    );
}

/// D-Phase3-02 + D-Phase3-15: copy_week aggregiert Warnings über alle
/// inneren `book_slot_with_conflict_check`-Calls; KEINE De-Dup.
///
/// Setup: 3 Source-Bookings; 2 davon liegen auf einer AbsencePeriod (die
/// DAO liefert pro Booking-Aufruf je eine AbsencePeriod-Match), 1 davon ist
/// frei. Erwartung: 3 copied, 2 Warnings.
#[tokio::test]
async fn test_copy_week_aggregates_warnings() {
    use std::sync::Mutex;

    let mut deps = build_dependencies(true, false);

    // 3 Source-Bookings — alle Slot=Monday, alle in W16/2026 (Source).
    let source_bookings: Arc<[Booking]> = Arc::from(vec![
        Booking {
            id: uuid!("BB000000-0000-0000-0000-000000000001"),
            ..default_booking()
        },
        Booking {
            id: uuid!("BB000000-0000-0000-0000-000000000002"),
            ..default_booking()
        },
        Booking {
            id: uuid!("BB000000-0000-0000-0000-000000000003"),
            ..default_booking()
        },
    ]);

    deps.booking_service.checkpoint();
    let src = source_bookings.clone();
    deps.booking_service
        .expect_get_for_week()
        .with(eq(16u8), eq(2026u32), always(), always())
        .returning(move |_, _, _, _| Ok(src.clone()));

    // BookingService::create wird 3x aufgerufen — wir vergeben eindeutige IDs
    // pro Aufruf via Mutex-Counter, damit Warnings auseinanderzuhalten sind.
    let create_counter = Arc::new(Mutex::new(0u32));
    let counter_for_mock = create_counter.clone();
    deps.booking_service
        .expect_create()
        .returning(move |b, _, _| {
            let mut guard = counter_for_mock.lock().unwrap();
            *guard += 1;
            let n = *guard;
            Ok(Booking {
                id: Uuid::from_bytes([
                    0xCC, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, n as u8,
                ]),
                version: default_version(),
                created: Some(PrimitiveDateTime::new(
                    time::Date::from_calendar_date(2026, Month::April, 20).unwrap(),
                    Time::from_hms(0, 0, 0).unwrap(),
                )),
                ..b.clone()
            })
        });

    // AbsenceService liefert für die ersten 2 Aufrufe einen Match, für den
    // 3. nichts.
    deps.absence_service.checkpoint();
    let absence_counter = Arc::new(Mutex::new(0u32));
    let ac = absence_counter.clone();
    deps.absence_service
        .expect_find_overlapping_for_booking()
        .returning(move |_, _, _, _| {
            let mut guard = ac.lock().unwrap();
            *guard += 1;
            if *guard <= 2 {
                Ok(Arc::from(vec![default_absence_period()]))
            } else {
                Ok(Arc::from(Vec::<AbsencePeriod>::new()))
            }
        });

    let service = deps.build_service();
    let result = service
        .copy_week_with_conflict_check(16, 2026, 17, 2026, ().auth(), None)
        .await
        .expect("copy_week_with_conflict_check should succeed");

    assert_eq!(result.copied_bookings.len(), 3, "expected 3 copied bookings");
    assert_eq!(
        result.warnings.len(),
        2,
        "expected 2 aggregated warnings (no de-dup), got {:?}",
        result.warnings
    );
    for warning in result.warnings.iter() {
        assert!(matches!(
            warning,
            Warning::BookingOnAbsenceDay { .. }
        ));
    }
}

#[tokio::test]
async fn test_book_slot_with_conflict_check_forbidden() {
    // D-24-04: gate ist nun Shiftplanner ∨ self. Beide Pfade liefern Forbidden.
    // Ein non-shiftplanner non-self Akteur darf nicht buchen → Err(Forbidden).
    let deps = build_dependencies(false, false);
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_copy_week_with_conflict_check_forbidden() {
    // shiftplan.edit-Probe liefert Forbidden — bulk-Op-Permission ist
    // einkanalig, KEIN HR ∨ self pro Source.
    let deps = build_dependencies(false, false);
    let service = deps.build_service();

    let result = service
        .copy_week_with_conflict_check(16, 2026, 17, 2026, ().auth(), None)
        .await;
    test_forbidden(&result);
}

/// Regression: book_slot_with_conflict_check must attribute created_by to the
/// originating user (resolved via permission_service.current_user_id) before
/// delegating to BookingService::create with Authentication::Full. Otherwise
/// the bookings_view ends up with NULL created_by entries that crash
/// BookingLogService reads.
#[tokio::test]
async fn test_book_slot_attributes_creator_when_input_lacks_created_by() {
    let mut deps = build_dependencies(true, false);
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .withf(|booking, _, _| booking.created_by.as_deref() == Some("test-user"))
        .returning(|_, _, _| Ok(persisted_booking()));
    let service = deps.build_service();

    // Input booking deliberately has created_by: None (the realistic frontend
    // payload — the auth context is the only source of truth).
    let input = Booking {
        created_by: None,
        ..default_booking()
    };

    let result = service
        .book_slot_with_conflict_check(&input, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");
    assert_eq!(result.booking.id, default_booking_id());
}

// ---------- Phase 5 (Plan 05-06): Paid-Employee-Limit-Warning Tests ----------
//
// Pflicht-Coverage (CONTEXT.md):
//  - test_book_paid_into_full_slot_emits_warning      (D-04, D-06, D-08, D-13)
//  - test_book_paid_at_limit_no_warning               (D-06 strikt: equal → kein Trigger)
//  - test_book_unpaid_into_full_slot_no_warning       (D-04: nur paid zaehlt)
//  - test_book_with_no_limit_no_warning               (D-15: NULL = no check)
//  - test_book_paid_in_absence_still_counts           (D-05: Absence orthogonal)
//  - test_book_persists_even_when_warning_fires       (D-07: kein Rollback)
//
// D-16 (Endpoint-Scope) wird NICHT durch einen Service-Tier-Test abgedeckt —
// er ist eine architektonische Aussage, die durch grep auf
// `service_impl/src/booking.rs` und `rest/src/booking.rs` belegt wird:
// dort darf `PaidEmployeeLimitExceeded` NICHT erscheinen. Die Acceptance-
// Criteria in 05-06-PLAN.md decken das ab.

fn paid_sp_a_id() -> Uuid {
    uuid!("11111111-1111-1111-1111-111111111111")
}
fn paid_sp_b_id() -> Uuid {
    uuid!("22222222-2222-2222-2222-222222222222")
}
fn paid_sp_c_id() -> Uuid {
    uuid!("33333333-3333-3333-3333-333333333333")
}
fn unpaid_sp_id() -> Uuid {
    uuid!("44444444-4444-4444-4444-444444444444")
}

fn paid_sales_person(id: Uuid) -> SalesPerson {
    SalesPerson {
        id,
        name: "Paid SP".into(),
        background_color: "#fff".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: uuid!("AAAA0000-0000-0000-0000-000000000001"),
    }
}

fn slot_with_paid_limit(max: u8) -> Slot {
    Slot {
        max_paid_employees: Some(max),
        ..monday_slot()
    }
}

/// Existing paid booking in the same (slot_id, year=2026, week=17) tuple,
/// pre-populated by a different sales-person-id. Used to seed
/// `BookingService::get_for_week` so the post-persist count includes them.
fn existing_paid_booking(sales_person_id: Uuid, booking_id: Uuid) -> Booking {
    Booking {
        id: booking_id,
        sales_person_id,
        slot_id: default_slot_id(),
        calendar_week: 17,
        year: 2026,
        created: Some(datetime!(2026 - 04 - 19 12:00:00)),
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: uuid!("BBBB0000-0000-0000-0000-000000000001"),
    }
}

/// D-04, D-06, D-08, D-13: Slot mit `max=2`, 2 bestehende paid-Bookings + 1
/// neu zu persistierendes (paid_sp_c) → post-persist-count = 3 > max=2 →
/// genau eine `PaidEmployeeLimitExceeded`-Warning mit den korrekten Feldern.
/// Buchung wird trotzdem persistiert (D-07-Vorboten — der dedizierte
/// Persistence-Test ist `test_book_persists_even_when_warning_fires`).
#[tokio::test]
async fn test_book_paid_into_full_slot_emits_warning() {
    let mut deps = build_dependencies(true, false);

    // Slot trägt das Limit max=2.
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    // Booking zu persistieren = paid_sp_c. Override `expect_create` so the
    // returned Booking trägt den Test-Sales-Person-Id (Default-Mock liefert
    // einen anderen sp). Die persistierte ID bleibt `default_booking_id()`.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });

    // get_for_week liefert (D-04: post-persist-Sicht): 2 bestehende
    // paid-Bookings + 1 frisch persistiertes = 3 paid in slot.
    deps.booking_service
        .expect_get_for_week()
        .with(eq(17u8), eq(2026u32), always(), always())
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_c_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    // get_all_paid liefert die 3 paid Sales-Personen.
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
                paid_sales_person(paid_sp_c_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_c_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    let paid_warnings: Vec<&Warning> = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. }))
        .collect();
    assert_eq!(
        paid_warnings.len(),
        1,
        "expected exactly one PaidEmployeeLimitExceeded warning, got {:?}",
        result.warnings
    );
    match paid_warnings[0] {
        Warning::PaidEmployeeLimitExceeded {
            slot_id,
            booking_id,
            year,
            week,
            current_paid_count,
            max_paid_employees,
        } => {
            assert_eq!(*slot_id, default_slot_id());
            assert_eq!(*booking_id, default_booking_id());
            assert_eq!(*year, 2026);
            assert_eq!(*week, 17);
            assert_eq!(*current_paid_count, 3);
            assert_eq!(*max_paid_employees, 2);
        }
        other => panic!("expected PaidEmployeeLimitExceeded, got {other:?}"),
    }
}

/// D-06 strikt: bei `current_paid_count == max_paid_employees` darf KEINE
/// Warning fliegen. Setup: max=2, post-persist 2 paid bookings.
#[tokio::test]
async fn test_book_paid_at_limit_no_warning() {
    let mut deps = build_dependencies(true, false);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_b_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_b_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert!(
        !result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "D-06 strikt: equal-count must NOT trigger PaidEmployeeLimitExceeded; got {:?}",
        result.warnings
    );
}

/// D-04: unpaid Sales-Person zaehlt nicht zum Limit. Setup: max=2, 2
/// bestehende paid-Bookings, neu zu persistierendes Booking gehoert zu einer
/// **unpaid** Sales-Person. Post-persist paid-count = 2 (das neue zaehlt
/// NICHT) → kein `current > max` → keine Warning.
#[tokio::test]
async fn test_book_unpaid_into_full_slot_no_warning() {
    let mut deps = build_dependencies(true, false);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
                // Frisch persistiertes Booking — Sales-Person ist UNPAID.
                Booking {
                    id: default_booking_id(),
                    sales_person_id: unpaid_sp_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    // get_all_paid liefert NUR die 2 paid SPs — der unpaid SP ist nicht in
    // der Liste, daher zaehlt er via `paid_ids.contains(...)` nicht.
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: unpaid_sp_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert!(
        !result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "D-04: unpaid sales-person must NOT count toward paid-limit; got {:?}",
        result.warnings
    );
}

/// D-15: `slot.max_paid_employees = None` ⇒ kein Check, keine Warning. Der
/// Helper darf hier nicht aufgerufen werden — wir registrieren bewusst KEIN
/// `expect_get_all_paid`. Dass `get_all_paid` nicht aufgerufen wird, wird
/// implizit dadurch verifiziert, dass `MockSalesPersonService` ohne
/// matching expectation panic'en wuerde, falls der Helper triggert.
#[tokio::test]
async fn test_book_with_no_limit_no_warning() {
    // Default-Slot hat max_paid_employees: None — kein Override noetig.
    let deps = build_dependencies(true, false);
    let service = deps.build_service();

    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    assert!(
        !result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "D-15: NULL limit must NOT emit any PaidEmployeeLimitExceeded; got {:?}",
        result.warnings
    );
}

/// D-05: Absence-Status der gebuchten Person ist orthogonal. Setup: max=1,
/// 1 bestehendes paid-Booking + 1 frisches paid-Booking ergeben 2 Paid-
/// Eintraege > max=1 → Warning fliegt — UND zusaetzlich greift die
/// existierende `BookingOnAbsenceDay`-Warning (Plan 03-04). Beide Pfade
/// fliegen unabhaengig.
#[tokio::test]
async fn test_book_paid_in_absence_still_counts() {
    let mut deps = build_dependencies(true, false);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(1)));

    // AbsenceService liefert eine ueberlappende AbsencePeriod fuer die
    // gebuchte Person — sollte D-05 NICHT vom Count ausschliessen.
    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_overlapping_for_booking()
        .returning(|_, _, _, _| Ok(Arc::from(vec![default_absence_period()])));

    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_b_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_b_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    // Paid-Limit-Warning fliegt — Absence verhindert das Counting NICHT.
    let paid_count = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. }))
        .count();
    assert_eq!(
        paid_count, 1,
        "D-05: absence orthogonal — paid-limit warning must still fire; got {:?}",
        result.warnings
    );

    // Absence-Pfad fliegt unabhaengig (Plan 03-04 BookingOnAbsenceDay).
    let absence_count = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::BookingOnAbsenceDay { .. }))
        .count();
    assert_eq!(
        absence_count, 1,
        "BookingOnAbsenceDay must fire independently; got {:?}",
        result.warnings
    );
}

/// D-07: trotz Warning bleibt das Booking persistiert (kein Tx-Rollback).
/// `result.booking.id == persisted_booking.id` belegt, dass das vom
/// `BookingService::create`-Mock zurueckgegebene Booking unverändert in
/// `BookingCreateResult.booking` landet — der commit am Ende von
/// `book_slot_with_conflict_check` lief durch.
#[tokio::test]
async fn test_book_persists_even_when_warning_fires() {
    let mut deps = build_dependencies(true, false);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(1)));

    deps.booking_service.checkpoint();
    // Persistiertes Booking trägt eine eindeutige UUID, die wir im Result
    // wiederfinden muessen.
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| {
            // 2 paid Bookings → > max=1 → Warning fliegt.
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_b_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_b_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    // D-07: Booking ist im Result-Wrapper mit der vom Mock vergebenen ID
    // — der commit ist durchgelaufen, kein Rollback.
    assert_eq!(
        result.booking.id,
        default_booking_id(),
        "D-07: booking must be persisted even when warning fires"
    );
    // Warning ist trotzdem da.
    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "warning must fire alongside persistence; got {:?}",
        result.warnings
    );
}

// ---------- Phase 8.3 (Plan 04) — Booking-Conflict-Suppression bei Half ----------
//
// ROADMAP SC #6 / D-08.3-05: Halbtag-Absence + Booking am selben Tag ist ein
// legitimer Workflow (Mitarbeiter arbeitet die andere Tageshaelfte). Der
// `Warning::BookingOnAbsenceDay`-Emitter in `shiftplan_edit.rs` filtert
// AbsencePeriods mit `day_fraction = Half` schweigend aus.

/// "Karin-aehnliches Heiligabend-Pattern": ein Booking am 2026-12-24 und
/// gleichzeitig eine Halbtag-Absence fuer denselben Tag DARF KEINE
/// `BookingOnAbsenceDay`-Warning erzeugen. Erfuellt ROADMAP Phase 8.3 SC #6.
#[tokio::test]
async fn booking_conflict_half_day_does_not_warn() {
    let mut deps = build_dependencies(true, false);

    // AbsenceService liefert genau eine Halbtag-Absence ueber den Booking-Tag.
    // Alle anderen Felder analog `default_absence_period()` — wir setzen nur
    // `day_fraction = Half`.
    deps.absence_service.checkpoint();
    deps.absence_service
        .expect_find_overlapping_for_booking()
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![AbsencePeriod {
                day_fraction: DayFraction::Half,
                ..default_absence_period()
            }]))
        });

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&default_booking(), ().auth(), None)
        .await
        .expect("book_slot_with_conflict_check should succeed");

    // Persistenz lief durch — Booking ist da.
    assert_eq!(result.booking, persisted_booking());

    // SC #6: KEIN BookingOnAbsenceDay-Warning, weil die Absence Half ist.
    let absence_warnings: Vec<&Warning> = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::BookingOnAbsenceDay { .. }))
        .collect();
    assert!(
        absence_warnings.is_empty(),
        "Phase 8.3 SC #6: Half-day absence + booking on same day MUST NOT \
         emit BookingOnAbsenceDay warning; got: {:?}",
        result.warnings
    );
}

/// Regression (Phase 23 Browser-UAT): `modify_slot` MUSS `max_paid_employees`
/// vom eingehenden Slot in den neu erzeugten (versionierten) Slot übernehmen.
/// Vorher kopierte der Versionierungs-Pfad nur `min_resources`/`from`/`to`, so
/// dass das im Slot-Editor gesetzte Paid-Limit beim Speichern bestehender Slots
/// stillschweigend verloren ging (der neue Slot erbte `None` vom Vorgänger).
#[tokio::test]
async fn test_modify_slot_carries_max_paid_employees() {
    let mut deps = build_dependencies(true, true);

    // stored_slot (get_slot) = monday_slot(): version == default_version(),
    // valid_from = 2024-01-01, max_paid_employees = None. Default-Mock liefert ihn.

    // Der alte Slot wird auf valid_to = 2026-06-21 verkürzt → update_slot
    // (nicht delete, da 2026-06-21 >= 2024-01-01).
    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));

    // Keine Buchungen im Änderungszeitraum.
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<Booking>::new())));

    // Kern der Assertion: der an create_slot übergebene NEUE Slot trägt das im
    // Editor gesetzte Limit (Some(7)) sowie das geänderte min_resources (4).
    deps.slot_service
        .expect_create_slot()
        .returning(|slot, _, _| {
            assert_eq!(
                slot.max_paid_employees,
                Some(7),
                "modify_slot muss max_paid_employees in den neuen Slot übernehmen"
            );
            assert_eq!(slot.min_resources, 4);
            Ok(slot.clone())
        });

    let service = deps.build_service();

    // Eingehender Slot: gleiche version wie stored (sonst EntityConflicts),
    // mit gesetztem Paid-Limit und geändertem min_resources.
    let input = Slot {
        min_resources: 4,
        max_paid_employees: Some(7),
        ..monday_slot()
    };

    let result = service
        .modify_slot(&input, 2026, 26, ().auth(), None)
        .await
        .expect("modify_slot should succeed");

    assert_eq!(
        result.max_paid_employees,
        Some(7),
        "der zurückgegebene neue Slot muss das gesetzte Paid-Limit tragen"
    );
}

// ---------- Phase 24 (Plan 02): Hard-Enforcement Tests (D-24-02, D-24-04, D-24-08) ----------

/// D-24-02 / D-24-08: toggle ON + non-shiftplanner + paid person over limit →
/// Err(ServiceError::PaidLimitExceeded { current, max }). Keine Buchung persistiert.
///
/// Setup: max=2, 2 bestehende paid-Bookings → existing_paid=2; neue Buchung ist paid.
/// prospective = 2 + 1 = 3 > max=2 → Block. toggle ON, non-shiftplanner (check_permission → Err).
#[tokio::test]
async fn test_hard_block_non_shiftplanner_over_limit() {
    // non-shiftplanner (permission Err), non-self (verify Err) → gate fails unless we allow verify.
    // Actually: we want the booking gate (Shiftplanner ∨ self) to PASS so we reach the guard.
    // Use verify_grants_self=true so the self-path allows the booking, but check_permission=false
    // so the shiftplanner bypass in the enforcement guard is NOT granted.
    let mut deps = build_dependencies(false, true);

    // Slot with limit max=2.
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    // Toggle: hard mode ON.
    deps.toggle_service.checkpoint();
    deps.toggle_service
        .expect_is_enabled()
        .returning(|_, _, _| Ok(true));

    // 2 existing paid bookings pre-populate the slot/week.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_get_for_week()
        .with(eq(17u8), eq(2026u32), always(), always())
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
            ]))
        });
    // BookingService::create MUST NOT be called (block happens pre-persist).
    // No expect_create() set → if called, mock will panic.

    // get_all_paid: paid_sp_c is paid → booked_is_paid = true.
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
                paid_sales_person(paid_sp_c_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_c_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await;

    match result {
        Err(ServiceError::PaidLimitExceeded { current, max }) => {
            assert_eq!(current, 3, "prospective count should be existing(2) + 1 = 3");
            assert_eq!(max, 2);
        }
        other => panic!(
            "expected PaidLimitExceeded, got {other:?}"
        ),
    }
}

/// D-24-02: toggle ON + actor IS shiftplanner → bypass; booking persists (Ok).
/// Even when paid count would exceed limit, a shiftplanner is never blocked.
///
/// Setup: max=2, 2 existing paid bookings + paid_sp_c new. toggle ON.
/// check_permission grants shiftplanner → is_shiftplanner=true → bypass → Ok + soft warning.
#[tokio::test]
async fn test_hard_block_shiftplanner_bypasses() {
    // check_permission=true (shiftplanner granted), verify_grants_self=false.
    let mut deps = build_dependencies(true, false);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    // Toggle: hard mode ON.
    deps.toggle_service.checkpoint();
    deps.toggle_service
        .expect_is_enabled()
        .returning(|_, _, _| Ok(true));

    // BookingService::create IS called (shiftplanner bypasses block).
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    // Post-persist soft-warning count: 3 paid → warning fires (soft path still active).
    deps.booking_service
        .expect_get_for_week()
        .with(eq(17u8), eq(2026u32), always(), always())
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_c_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
                paid_sales_person(paid_sp_c_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_c_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("shiftplanner must bypass hard-block; booking should succeed");

    assert_eq!(
        result.booking.id,
        default_booking_id(),
        "shiftplanner bypass: booking must be persisted"
    );
    // Soft warning still fires for the shiftplanner overage case.
    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "soft warning should still fire for shiftplanner over-limit; got {:?}",
        result.warnings
    );
}

/// D-24-01: toggle OFF (soft mode) → paid booking over limit persists + emits soft warning.
/// This is the unchanged existing soft path; verified here explicitly for D-24-01 regression.
///
/// Setup: max=2, 2 existing paid + 1 new paid, toggle OFF. Should succeed with warning.
#[tokio::test]
async fn test_soft_mode_over_limit_warns_not_blocks() {
    // non-shiftplanner, but self-booking allowed.
    let mut deps = build_dependencies(false, true);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    // Toggle: soft mode (false). Default in build_dependencies is already false,
    // but checkpoint + re-register explicitly for clarity.
    deps.toggle_service.checkpoint();
    deps.toggle_service
        .expect_is_enabled()
        .returning(|_, _, _| Ok(false));

    // Booking persists in soft mode.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    deps.booking_service
        .expect_get_for_week()
        .with(eq(17u8), eq(2026u32), always(), always())
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
                Booking {
                    id: default_booking_id(),
                    sales_person_id: paid_sp_c_id(),
                    slot_id: default_slot_id(),
                    calendar_week: 17,
                    year: 2026,
                    created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                    deleted: None,
                    created_by: None,
                    deleted_by: None,
                    version: default_version(),
                },
            ]))
        });

    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
                paid_sales_person(paid_sp_c_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: paid_sp_c_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("soft mode must not block; booking should succeed");

    // Soft warning fires (not blocked).
    assert!(
        result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "D-24-01: soft mode must emit PaidEmployeeLimitExceeded warning; got {:?}",
        result.warnings
    );
    assert_eq!(
        result.booking.id,
        default_booking_id(),
        "soft mode: booking must be persisted"
    );
}

/// D-24-Grenzregel: toggle ON + non-shiftplanner + UNPAID person → never blocked.
/// Unpaid persons do not count toward the paid limit.
///
/// Setup: max=2, 2 existing paid, new booking is UNPAID. prospective=2 (no change) → Ok.
#[tokio::test]
async fn test_hard_block_unpaid_never_blocked() {
    // non-shiftplanner, self-booking allowed.
    let mut deps = build_dependencies(false, true);

    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(slot_with_paid_limit(2)));

    // Toggle: hard mode ON.
    deps.toggle_service.checkpoint();
    deps.toggle_service
        .expect_is_enabled()
        .returning(|_, _, _| Ok(true));

    // Booking persists — unpaid person does not count.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_create()
        .returning(|b, _, _| {
            Ok(Booking {
                id: default_booking_id(),
                version: default_version(),
                created: Some(datetime!(2026 - 04 - 20 00:00:00)),
                ..b.clone()
            })
        });
    // Pre-persist count: 2 paid existing; new (unpaid) not in list.
    deps.booking_service
        .expect_get_for_week()
        .with(eq(17u8), eq(2026u32), always(), always())
        .returning(|_, _, _, _| {
            Ok(Arc::from(vec![
                existing_paid_booking(
                    paid_sp_a_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000001"),
                ),
                existing_paid_booking(
                    paid_sp_b_id(),
                    uuid!("CCCCCCCC-0000-0000-0000-000000000002"),
                ),
            ]))
        });

    // get_all_paid does NOT include unpaid_sp_id → booked_is_paid = false → no block.
    // Called once by pre-persist guard; once again by post-persist soft-warning.
    deps.sales_person_service
        .expect_get_all_paid()
        .returning(|_, _| {
            Ok(Arc::from(vec![
                paid_sales_person(paid_sp_a_id()),
                paid_sales_person(paid_sp_b_id()),
            ]))
        });

    let booking_to_create = Booking {
        sales_person_id: unpaid_sp_id(),
        ..default_booking()
    };

    let service = deps.build_service();
    let result = service
        .book_slot_with_conflict_check(&booking_to_create, ().auth(), None)
        .await
        .expect("unpaid person must never be blocked; booking should succeed");

    assert_eq!(
        result.booking.id,
        default_booking_id(),
        "unpaid booking must be persisted even when slot is at paid limit"
    );
    // No PaidEmployeeLimitExceeded warning (unpaid doesn't count; post-persist count = 2 = max, not > max).
    assert!(
        !result
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::PaidEmployeeLimitExceeded { .. })),
        "D-24-Grenzregel: unpaid booking MUST NOT trigger PaidLimitExceeded warning; got {:?}",
        result.warnings
    );
}

// ---------- Phase 35 (Plan 01): modify_slot_single_week — D-35-05 TDD-Tests ----------
//
// Szenario: change_year=2026, change_week=26
//   Montag KW26 = 2026-06-22; Sonntag KW25 (Seg1-valid_to) = 2026-06-21
//   Segment 2: valid_from=2026-06-22, valid_to=Some(2026-06-28) (Sonntag KW26)
//   Segment 3: valid_from=2026-06-29 (Montag KW27), valid_to=None (Original unbegrenzt)

fn msw_seg2_id() -> Uuid {
    uuid!("35350002-0000-0000-0000-000000000002")
}

fn msw_seg3_id() -> Uuid {
    uuid!("35350003-0000-0000-0000-000000000003")
}

/// Setzt create_slot-Mock: Seg2 (valid_from=2026-06-22) → msw_seg2_id(),
/// Seg3 (valid_from=2026-06-29) → msw_seg3_id(). Erwartet genau 2 Aufrufe.
fn msw_setup_create_slot_2x(deps: &mut ShiftplanEditDependencies) {
    deps.slot_service
        .expect_create_slot()
        .times(2)
        .returning(|slot, _, _| {
            let id = if slot.valid_from == date!(2026 - 06 - 22) {
                msw_seg2_id()
            } else {
                msw_seg3_id()
            };
            Ok(Slot { id, ..slot.clone() })
        });
}

/// Setzt get_for_slot_id_since-Mock auf leere Buchungsliste.
fn msw_no_bookings(deps: &mut ShiftplanEditDependencies) {
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<Booking>::new())));
}

// Test 1: 3-Segment-Struktur mit korrekten Datumsgrenzen
#[tokio::test]
#[allow(clippy::type_complexity)]
async fn test_msw_three_segment_structure() {
    let mut deps = build_dependencies(true, true);

    let seg1_valid_to: Arc<Mutex<Option<Option<time::Date>>>> = Arc::new(Mutex::new(None));
    let seg1_vt_c = seg1_valid_to.clone();
    deps.slot_service
        .expect_update_slot()
        .times(1)
        .returning(move |slot, _, _| {
            *seg1_vt_c.lock().unwrap() = Some(slot.valid_to);
            Ok(())
        });
    msw_no_bookings(&mut deps);

    let created: Arc<Mutex<Vec<(time::Date, Option<time::Date>)>>> =
        Arc::new(Mutex::new(Vec::new()));
    let created_c = created.clone();
    deps.slot_service
        .expect_create_slot()
        .times(2)
        .returning(move |slot, _, _| {
            created_c
                .lock()
                .unwrap()
                .push((slot.valid_from, slot.valid_to));
            let id = if slot.valid_from == date!(2026 - 06 - 22) {
                msw_seg2_id()
            } else {
                msw_seg3_id()
            };
            Ok(Slot { id, ..slot.clone() })
        });

    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await
        .expect("3-Segment-Split muss gelingen");

    // Rückgabe = Segment 2 (Ausnahme-Slot)
    assert_eq!(
        result.id,
        msw_seg2_id(),
        "modify_slot_single_week muss Segment 2 zurückgeben"
    );

    // Segment 1: valid_to = Sonntag KW25
    let vt = seg1_valid_to
        .lock()
        .unwrap()
        .expect("update_slot muss aufgerufen worden sein");
    assert_eq!(
        vt,
        Some(date!(2026 - 06 - 21)),
        "Seg1 valid_to = Sonntag KW25 (2026-06-21)"
    );

    // Segmente 2 + 3
    let segs = created.lock().unwrap();
    assert_eq!(segs.len(), 2, "genau 2 neue Slots erstellt");
    let seg2 = segs
        .iter()
        .find(|(vf, _)| *vf == date!(2026 - 06 - 22))
        .expect("Segment 2 mit valid_from=2026-06-22 muss existieren");
    assert_eq!(
        seg2.1,
        Some(date!(2026 - 06 - 28)),
        "Seg2 valid_to = Sonntag KW26 (2026-06-28)"
    );
    let seg3 = segs
        .iter()
        .find(|(vf, _)| *vf == date!(2026 - 06 - 29))
        .expect("Segment 3 mit valid_from=2026-06-29 muss existieren");
    assert_eq!(seg3.1, None, "Seg3 valid_to = None (original unbegrenzt)");
}

// Test 2+3+4: Booking-Partition (KW26→Seg2, KW27→Seg3, je-genau-einmal, kein Doppel-/Waisen-Row)
#[tokio::test]
async fn test_msw_booking_partition_and_each_exactly_once() {
    let mut deps = build_dependencies(true, true);

    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    msw_setup_create_slot_2x(&mut deps);

    let kw26 = Booking {
        id: uuid!("B0260000-0000-0000-0000-000000000001"),
        calendar_week: 26,
        year: 2026,
        version: uuid!("B0260000-0000-0000-0000-000000000010"),
        ..default_booking()
    };
    let kw27 = Booking {
        id: uuid!("B0270000-0000-0000-0000-000000000001"),
        calendar_week: 27,
        year: 2026,
        version: uuid!("B0270000-0000-0000-0000-000000000010"),
        ..default_booking()
    };

    // Checkpoint: Default-Booking-Expectations entfernen, eigene setzen
    deps.booking_service.checkpoint();
    let kw26_c = kw26.clone();
    let kw27_c = kw27.clone();
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(move |_, _, _, _, _| Ok(Arc::from(vec![kw26_c.clone(), kw27_c.clone()])));

    deps.booking_service
        .expect_delete()
        .times(2)
        .returning(|_, _, _| Ok(()));

    // Sammle (calendar_week, new_slot_id) pro create-Aufruf
    let repoints: Arc<Mutex<Vec<(i32, Uuid)>>> = Arc::new(Mutex::new(Vec::new()));
    let repoints_c = repoints.clone();
    deps.booking_service
        .expect_create()
        .times(2)
        .returning(move |booking, _, _| {
            repoints_c
                .lock()
                .unwrap()
                .push((booking.calendar_week, booking.slot_id));
            Ok(persisted_booking())
        });

    let service = deps.build_service();
    service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await
        .expect("Booking-Partition muss gelingen");

    let pairs = repoints.lock().unwrap();
    assert_eq!(pairs.len(), 2, "genau 2 Buchungen re-gepointed (je-genau-einmal)");

    let kw26_target = pairs
        .iter()
        .find(|(cw, _)| *cw == 26)
        .map(|(_, id)| *id)
        .expect("KW26-Buchung muss re-gepointed worden sein");
    let kw27_target = pairs
        .iter()
        .find(|(cw, _)| *cw == 27)
        .map(|(_, id)| *id)
        .expect("KW27-Buchung muss re-gepointed worden sein");

    assert_eq!(kw26_target, msw_seg2_id(), "KW26-Buchung muss auf Segment 2 landen");
    assert_eq!(kw27_target, msw_seg3_id(), "KW27-Buchung muss auf Segment 3 landen");
}

// Test 5: Rollback — kein commit bei Fehler mitten im Vorgang
#[tokio::test]
async fn test_msw_rollback_no_commit_on_error() {
    let mut deps = build_dependencies(true, true);

    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    msw_no_bookings(&mut deps);

    // Zweites create_slot (Segment 3) schlägt fehl
    let call_n = Arc::new(Mutex::new(0u32));
    let call_n_c = call_n.clone();
    deps.slot_service
        .expect_create_slot()
        .returning(move |slot, _, _| {
            let mut n = call_n_c.lock().unwrap();
            *n += 1;
            if *n == 1 {
                Ok(Slot {
                    id: msw_seg2_id(),
                    ..slot.clone()
                })
            } else {
                Err(ServiceError::Forbidden) // simulierter Fehler → Rollback
            }
        });

    // Commit darf NICHT aufgerufen werden: checkpoint → kein expect_commit gesetzt
    // → unerwarteter commit-Aufruf würde Panic auslösen
    deps.transaction_dao.checkpoint();
    deps.transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));

    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;

    assert!(
        result.is_err(),
        "Fehler im Seg3-Create muss propagiert werden (kein commit)"
    );
}

// Test 6: Erste-KW-Edge — delete_slot statt update_slot für Segment 1
#[tokio::test]
async fn test_msw_first_kw_edge_delete_slot() {
    let mut deps = build_dependencies(true, true);

    // Slot mit valid_from = Montag KW26 → Seg1 hätte valid_to < valid_from → delete_slot
    deps.slot_service.checkpoint();
    deps.slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(Slot { valid_from: date!(2026 - 06 - 22), ..monday_slot() }));
    deps.slot_service
        .expect_delete_slot()
        .times(1)
        .returning(|_, _, _| Ok(()));
    // KEIN expect_update_slot → unerwarteter Aufruf würde Panic auslösen
    msw_setup_create_slot_2x(&mut deps);
    msw_no_bookings(&mut deps);

    let service = deps.build_service();
    let input = Slot { valid_from: date!(2026 - 06 - 22), ..monday_slot() };
    let result = service
        .modify_slot_single_week(&input, 2026, 26, ().auth(), None)
        .await;

    assert!(
        result.is_ok(),
        "Erste-KW-Edge muss gelingen (delete_slot statt update_slot): {result:?}"
    );
}

// Test 7: Unbegrenztes valid_to → Segment 3 erbt None (bleibt unbegrenzt)
#[tokio::test]
async fn test_msw_unbounded_valid_to_seg3() {
    let mut deps = build_dependencies(true, true);

    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    msw_no_bookings(&mut deps);

    // monday_slot() hat valid_to = None → original_valid_to = None → Seg3 valid_to = None
    let seg3_valid_to: Arc<Mutex<Option<Option<time::Date>>>> = Arc::new(Mutex::new(None));
    let seg3_vt_c = seg3_valid_to.clone();
    deps.slot_service
        .expect_create_slot()
        .times(2)
        .returning(move |slot, _, _| {
            let id = if slot.valid_from == date!(2026 - 06 - 22) {
                msw_seg2_id()
            } else {
                *seg3_vt_c.lock().unwrap() = Some(slot.valid_to);
                msw_seg3_id()
            };
            Ok(Slot { id, ..slot.clone() })
        });

    let service = deps.build_service();
    service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await
        .expect("unbegrenztes valid_to muss gelingen");

    assert_eq!(
        *seg3_valid_to.lock().unwrap(),
        Some(None),
        "Seg3 valid_to muss None sein (original unbegrenzt)"
    );
}

// Test 8: Keine Buchungen → keine booking.delete / booking.create-Aufrufe
#[tokio::test]
async fn test_msw_no_bookings_no_booking_mutations() {
    let mut deps = build_dependencies(true, true);

    deps.slot_service
        .expect_update_slot()
        .returning(|_, _, _| Ok(()));
    msw_setup_create_slot_2x(&mut deps);

    // Checkpoint: nur get_for_slot_id_since erlaubt; delete/create → panic bei unerwartetem Aufruf
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_get_for_slot_id_since()
        .returning(|_, _, _, _, _| Ok(Arc::from(Vec::<Booking>::new())));

    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;

    assert!(result.is_ok(), "ohne Buchungen muss 3-Segment-Split gelingen: {result:?}");
}

// Test 9: Forbidden — check_permission schlägt fehl → kein Slot-/Booking-Mutation
#[tokio::test]
async fn test_msw_forbidden() {
    let deps = build_dependencies(false, false);
    // Keine Slot-/Booking-Expectations → unerw. Aufrufe würden Panic auslösen

    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;

    test_forbidden(&result);
}
