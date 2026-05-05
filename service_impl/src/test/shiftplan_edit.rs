//! Mock-basierte Service-Tests für `ShiftplanEditServiceImpl::book_slot_with_conflict_check`
//! und `::copy_week_with_conflict_check` (Plan 03-04, Wave 3).
//!
//! Pflicht-Coverage (aus 03-VALIDATION.md):
//!  - `test_book_slot_warning_on_absence_day`        (BOOK-02 / D-Phase3-14 BookingOnAbsenceDay)
//!  - `test_book_slot_warning_on_manual_unavailable` (BOOK-02 / D-Phase3-14 BookingOnUnavailableDay)
//!  - `test_book_slot_no_warning_when_softdeleted_absence` (SC4 / Pitfall-1)
//!  - `test_copy_week_aggregates_warnings`           (BOOK-02 / D-Phase3-02, D-Phase3-15: KEINE De-Dup)
//!  - `test_book_slot_with_conflict_check_forbidden` (D-Phase3-12 HR ∨ self)
//!  - `test_copy_week_with_conflict_check_forbidden` (D-Phase3-12 — bulk-Op fordert shiftplan.edit)
//!
//! Mock-DI-Setup analog `service_impl/src/test/booking.rs:113-192` und
//! `service_impl/src/test/absence.rs:147-260`.

use std::sync::Arc;

use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::{
    absence::{AbsenceCategory, AbsencePeriod, MockAbsenceService},
    booking::{Booking, MockBookingService},
    carryover::MockCarryoverService,
    employee_work_details::MockEmployeeWorkDetailsService,
    extra_hours::MockExtraHoursService,
    reporting::MockReportingService,
    sales_person::{MockSalesPersonService, SalesPerson},
    sales_person_unavailable::{MockSalesPersonUnavailableService, SalesPersonUnavailable},
    shiftplan_edit::ShiftplanEditService,
    slot::{MockSlotService, Slot},
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
        }
    }
}

/// `permission_grants_hr` ⇒ HR-Probe liefert Ok; sonst Forbidden.
/// `verify_grants_self` ⇒ verify_user_is_sales_person liefert Ok; sonst Forbidden.
pub(crate) fn build_dependencies(
    permission_grants_hr: bool,
    verify_grants_self: bool,
) -> ShiftplanEditDependencies {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(move |_, _| {
            if permission_grants_hr {
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
    // Beide Permission-Pfade liefern Forbidden → hr.or(sp) propagiert.
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

