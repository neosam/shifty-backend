//! Mock-basierte Service-Tests für `AbsenceServiceImpl` (Plan 01-02).
//!
//! Pflicht-Coverage:
//! - `_forbidden`-Test pro public method (D-11, ABS-05).
//! - Self-Overlap-Detection mit `OverlappingPeriod(other_logical_id)` (D-13).
//! - `update`-Self-Overlap exkludiert eigene Row via `Some(logical_id)` (D-15).
//! - logical_id-Update-Pattern: Tombstone (UPDATE deleted) + Insert (CREATE
//!   neue physical id, gleiche logical_id, neue version) (D-07).
//! - Range-Validation auf `from > to` mappt nach `DateOrderWrong` (D-14).
//! - Optimistic-Lock: stale `version` → `EntityConflicts`.
//! - `sales_person_id`-Immutability beim Update → `ModificationNotAllowed`.

use std::sync::Arc;

use dao::absence::{AbsenceCategoryEntity, AbsencePeriodEntity, MockAbsenceDao};
use dao::MockTransaction;
use dao::MockTransactionDao;
use mockall::predicate::{always, eq};
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService};
use service::booking::MockBookingService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::MockSalesPersonService;
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::slot::{MockSlotService, Slot};
use service::special_days::MockSpecialDayService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError, ValidationFailureItem};
use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use time::{Month, Time};
use uuid::{uuid, Uuid};

use crate::absence::{AbsenceServiceDeps, AbsenceServiceImpl};
use crate::test::error_test::{
    test_conflicts, test_date_order_wrong, test_forbidden, test_not_found, test_validation_error,
};

fn default_logical_id() -> Uuid {
    uuid!("AB000000-0000-0000-0000-000000000001")
}
fn default_physical_id() -> Uuid {
    default_logical_id()
}
fn alternate_physical_id() -> Uuid {
    uuid!("AB000000-0000-0000-0000-000000000002")
}
fn other_logical_id() -> Uuid {
    uuid!("AB000000-0000-0000-0000-000000000099")
}
fn unknown_logical_id() -> Uuid {
    uuid!("AB000000-0000-0000-0000-0000000000FF")
}
fn default_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000001")
}
fn other_sales_person_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000002")
}
fn default_version() -> Uuid {
    uuid!("CC000000-0000-0000-0000-000000000001")
}
fn alternate_version() -> Uuid {
    uuid!("CC000000-0000-0000-0000-000000000002")
}

fn default_active_entity() -> AbsencePeriodEntity {
    AbsencePeriodEntity {
        id: default_physical_id(),
        logical_id: default_logical_id(),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategoryEntity::Vacation,
        from_date: date!(2026 - 04 - 12),
        to_date: date!(2026 - 04 - 15),
        description: "initial".into(),
        created: datetime!(2026 - 04 - 01 12:00:00),
        deleted: None,
        version: default_version(),
    }
}

fn other_logical_active_entity() -> AbsencePeriodEntity {
    AbsencePeriodEntity {
        id: other_logical_id(),
        logical_id: other_logical_id(),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategoryEntity::Vacation,
        from_date: date!(2026 - 04 - 13),
        to_date: date!(2026 - 04 - 14),
        description: "blocking".into(),
        created: datetime!(2026 - 03 - 01 12:00:00),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-000000000099"),
    }
}

fn default_create_request() -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::nil(),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2026 - 04 - 12),
        to_date: date!(2026 - 04 - 15),
        description: "initial".into(),
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

fn default_update_request() -> AbsencePeriod {
    AbsencePeriod {
        id: default_logical_id(),
        sales_person_id: default_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2026 - 04 - 12),
        to_date: date!(2026 - 04 - 20), // verlängerter Range — D-15-Self-Overlap-Test
        description: "updated".into(),
        created: Some(datetime!(2026 - 04 - 01 12:00:00)),
        deleted: None,
        version: default_version(),
    }
}

/// Default-Slot-Fixture für AbsenceService-Tests. Tests, die einen
/// spezifischen Slot brauchen (z.B. um day_of_week zu variieren),
/// überschreiben `slot_service.expect_get_slot()` lokal — sonst greift
/// dieser Default, damit Bestand-Tests ohne Override (z.B. reine
/// Self-Overlap-Tests, die niemals Booking-Iteration triggern, oder
/// Forward-Warning-Pfade ohne explizite Slot-Erwartung) nicht panicken.
fn default_slot_monday() -> Slot {
    Slot {
        id: uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380"),
        day_of_week: DayOfWeek::Monday,
        from: Time::from_hms(9, 0, 0).unwrap(),
        to: Time::from_hms(17, 0, 0).unwrap(),
        min_resources: 1,
        max_paid_employees: None,
        valid_from: time::Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50"),
        shiftplan_id: None,
    }
}

pub(crate) struct AbsenceDependencies {
    pub absence_dao: MockAbsenceDao,
    pub permission_service: MockPermissionService,
    pub sales_person_service: MockSalesPersonService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub special_day_service: MockSpecialDayService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub transaction_dao: MockTransactionDao,
    // Phase-3 Forward-Warning-Loop-Deps (D-Phase3-08):
    pub booking_service: MockBookingService,
    pub sales_person_unavailable_service: MockSalesPersonUnavailableService,
    pub slot_service: MockSlotService,
}

impl AbsenceServiceDeps for AbsenceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type AbsenceDao = MockAbsenceDao;
    type PermissionService = MockPermissionService;
    type SalesPersonService = MockSalesPersonService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type SpecialDayService = MockSpecialDayService;
    type EmployeeWorkDetailsService = MockEmployeeWorkDetailsService;
    type TransactionDao = MockTransactionDao;
    type BookingService = MockBookingService;
    type SalesPersonUnavailableService = MockSalesPersonUnavailableService;
    type SlotService = MockSlotService;
}

impl AbsenceDependencies {
    pub(crate) fn build_service(self) -> AbsenceServiceImpl<AbsenceDependencies> {
        AbsenceServiceImpl {
            absence_dao: self.absence_dao.into(),
            permission_service: self.permission_service.into(),
            sales_person_service: self.sales_person_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            special_day_service: self.special_day_service.into(),
            employee_work_details_service: self.employee_work_details_service.into(),
            transaction_dao: self.transaction_dao.into(),
            booking_service: self.booking_service.into(),
            sales_person_unavailable_service: self.sales_person_unavailable_service.into(),
            slot_service: self.slot_service.into(),
        }
    }
}

pub(crate) fn build_dependencies() -> AbsenceDependencies {
    let absence_dao = MockAbsenceDao::new();
    let mut permission_service = MockPermissionService::new();
    let mut sales_person_service = MockSalesPersonService::new();
    let mut clock_service = MockClockService::new();
    let uuid_service = MockUuidService::new();
    let special_day_service = MockSpecialDayService::new();
    let employee_work_details_service = MockEmployeeWorkDetailsService::new();
    let mut transaction_dao = MockTransactionDao::new();

    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    clock_service
        .expect_date_time_now()
        .returning(|| datetime!(2026 - 04 - 28 12:00:00));
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    // Phase-3 Forward-Warning-Loop-Defaults: leere Bookings + leere
    // ManualUnavailables; Slot-Default ist `default_slot_monday()` für
    // den Fall, dass ein Test Bookings injectet, ohne Slot zu überschreiben.
    let mut booking_service = MockBookingService::new();
    booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::from([])));
    let mut sales_person_unavailable_service = MockSalesPersonUnavailableService::new();
    sales_person_unavailable_service
        .expect_get_all_for_sales_person()
        .returning(|_, _, _| Ok(Arc::from([])));
    let mut slot_service = MockSlotService::new();
    slot_service
        .expect_get_slot()
        .returning(|_, _, _| Ok(default_slot_monday()));

    AbsenceDependencies {
        absence_dao,
        permission_service,
        sales_person_service,
        clock_service,
        uuid_service,
        special_day_service,
        employee_work_details_service,
        transaction_dao,
        booking_service,
        sales_person_unavailable_service,
        slot_service,
    }
}

// =========================================================================
// create
// =========================================================================

#[tokio::test]
async fn test_create_success() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_create()
        .withf(|entity: &AbsencePeriodEntity, _process: &str, _tx| {
            entity.id == alternate_physical_id()
                && entity.logical_id == alternate_physical_id()
                && entity.sales_person_id == default_sales_person_id()
                && entity.category == AbsenceCategoryEntity::Vacation
                && entity.deleted.is_none()
                && entity.created == datetime!(2026 - 04 - 28 12:00:00)
        })
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("absence_service::create::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("absence_service::create::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await;
    let created = result.expect("create should succeed").absence;
    assert_eq!(created.id, alternate_physical_id());
    assert_eq!(created.version, alternate_version());
    assert_eq!(created.created, Some(datetime!(2026 - 04 - 28 12:00:00)));
}

#[tokio::test]
async fn test_create_inverted_range_returns_date_order_wrong() {
    let mut deps = build_dependencies();
    // No DAO calls beyond the implicit transaction setup are expected.
    let service = {
        let req = AbsencePeriod {
            from_date: date!(2026 - 04 - 20),
            to_date: date!(2026 - 04 - 12),
            ..default_create_request()
        };
        // We don't expect find_overlapping to be called — Range-Inversion
        // returns before reaching the DAO. But if any expectation is violated,
        // mockall checks at drop. Override deps, build, run.
        deps.absence_dao.checkpoint();
        // Deliberately install no expectations on absence_dao so that any call
        // triggers a panic.
        let svc = deps.build_service();
        let r = svc.create(&req, Authentication::Full, None).await;
        r
    };
    test_date_order_wrong(&service);
}

#[tokio::test]
async fn test_create_self_overlap_same_category_returns_validation() {
    let mut deps = build_dependencies();
    // find_overlapping returns one conflicting row with logical_id =
    // other_logical_id() — service must surface this as
    // OverlappingPeriod(other_logical_id()).
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([other_logical_active_entity()])));
    let service = deps.build_service();

    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await;
    // test_validation_error checks ValidationFailureItem::OverlappingPeriod with the conflict id
    test_validation_error(
        &result,
        &ValidationFailureItem::OverlappingPeriod(other_logical_id()),
        1,
    );
}

#[tokio::test]
async fn test_create_self_overlap_different_category_succeeds() {
    // D-12: SickLeave-conflicts würden nicht durch eine Vacation-Anfrage
    // gefiltert werden — aber der Service ruft `find_overlapping` mit der
    // Anfrage-Kategorie. Wir simulieren, dass für Vacation kein Konflikt
    // existiert (auch wenn eine SickLeave-Periode überlappen würde, ist sie
    // im Filter ausgeschlossen). Strukturtest: Vacation-Create mit leerem
    // Vacation-Conflict-Set succeedet.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_overlapping()
        .with(
            eq(default_sales_person_id()),
            eq(AbsenceCategoryEntity::Vacation),
            always(),
            eq(None::<Uuid>),
            always(),
        )
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());
    let service = deps.build_service();

    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await;
    assert!(result.is_ok(), "Different-category overlap must not block (D-12)");
}

#[tokio::test]
async fn test_create_id_set_returns_error() {
    let mut deps = build_dependencies();
    // No DAO calls expected — id-set guard fires before find_overlapping.
    deps.absence_dao.checkpoint();
    let service = deps.build_service();

    let bad = AbsencePeriod {
        id: alternate_physical_id(),
        ..default_create_request()
    };
    let result = service.create(&bad, Authentication::Full, None).await;
    assert!(matches!(result, Err(ServiceError::IdSetOnCreate)));
}

#[tokio::test]
async fn test_create_version_set_returns_error() {
    let mut deps = build_dependencies();
    deps.absence_dao.checkpoint();
    let service = deps.build_service();

    let bad = AbsencePeriod {
        version: alternate_version(),
        ..default_create_request()
    };
    let result = service.create(&bad, Authentication::Full, None).await;
    assert!(matches!(result, Err(ServiceError::VersionSetOnCreate)));
}

#[tokio::test]
async fn test_create_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

// =========================================================================
// update
// =========================================================================

#[tokio::test]
async fn test_update_success_soft_deletes_old_inserts_new() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .with(eq(default_logical_id()), always())
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_update()
        .withf(|entity: &AbsencePeriodEntity, _process: &str, _tx| {
            entity.id == default_physical_id() && entity.deleted.is_some()
        })
        .returning(|_, _, _| Ok(()));
    deps.absence_dao
        .expect_create()
        .withf(|entity: &AbsencePeriodEntity, _process: &str, _tx| {
            entity.id == alternate_physical_id()
                && entity.logical_id == default_logical_id()
                && entity.sales_person_id == default_sales_person_id()
                && entity.version == alternate_version()
                && entity.deleted.is_none()
                && entity.from_date == date!(2026 - 04 - 12)
                && entity.to_date == date!(2026 - 04 - 20)
                && entity.created == datetime!(2026 - 04 - 28 12:00:00)
        })
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("absence_service::update::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("absence_service::update::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    let updated = result.expect("update should succeed").absence;
    assert_eq!(updated.id, default_logical_id());
    assert_eq!(updated.version, alternate_version());
    assert_eq!(updated.to_date, date!(2026 - 04 - 20));
}

#[tokio::test]
async fn test_update_self_overlap_excludes_self() {
    // D-15 Strukturtest: find_overlapping muss mit
    // exclude_logical_id = Some(default_logical_id()) aufgerufen werden.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.absence_dao
        .expect_find_overlapping()
        .with(
            eq(default_sales_person_id()),
            eq(AbsenceCategoryEntity::Vacation),
            always(),
            eq(Some(default_logical_id())), // <-- D-15
            always(),
        )
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.absence_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    assert!(result.is_ok(), "Update must succeed when excluding self");
}

#[tokio::test]
async fn test_update_self_overlap_same_category_returns_validation() {
    // Update mit echten überlappenden Rows derselben Kategorie (nicht die
    // eigene) muss als ValidationError(OverlappingPeriod) gemeldet werden.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([other_logical_active_entity()])));
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::OverlappingPeriod(other_logical_id()),
        1,
    );
}

#[tokio::test]
async fn test_update_unknown_logical_id_returns_not_found() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let request = AbsencePeriod {
        id: unknown_logical_id(),
        ..default_update_request()
    };
    let result = service.update(&request, Authentication::Full, None).await;
    test_not_found(&result, &unknown_logical_id());
}

#[tokio::test]
async fn test_update_changing_sales_person_id_is_rejected() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let bad = AbsencePeriod {
        sales_person_id: other_sales_person_id(),
        ..default_update_request()
    };
    let result = service.update(&bad, Authentication::Full, None).await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("sales_person_id".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_stale_version_returns_conflict() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let stale = AbsencePeriod {
        version: alternate_version(),
        ..default_update_request()
    };
    let result = service.update(&stale, Authentication::Full, None).await;
    test_conflicts(
        &result,
        &default_logical_id(),
        &alternate_version(),
        &default_version(),
    );
}

#[tokio::test]
async fn test_update_inverted_range_returns_date_order_wrong() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let bad = AbsencePeriod {
        from_date: date!(2026 - 04 - 20),
        to_date: date!(2026 - 04 - 12),
        ..default_update_request()
    };
    let result = service.update(&bad, Authentication::Full, None).await;
    test_date_order_wrong(&result);
}

#[tokio::test]
async fn test_update_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

// =========================================================================
// delete
// =========================================================================

#[tokio::test]
async fn test_delete_success_soft_deletes() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.absence_dao
        .expect_update()
        .withf(|entity: &AbsencePeriodEntity, _process: &str, _tx| {
            entity.id == default_physical_id() && entity.deleted.is_some()
        })
        .returning(|_, _, _| Ok(()));
    let service = deps.build_service();

    let result = service
        .delete(default_logical_id(), Authentication::Full, None)
        .await;
    assert!(result.is_ok(), "delete should succeed");
}

#[tokio::test]
async fn test_delete_unknown_logical_id_returns_not_found() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service
        .delete(unknown_logical_id(), Authentication::Full, None)
        .await;
    test_not_found(&result, &unknown_logical_id());
}

#[tokio::test]
async fn test_delete_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .delete(default_logical_id(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

// =========================================================================
// find_by_id
// =========================================================================

#[tokio::test]
async fn test_find_by_id_returns_active() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .find_by_id(default_logical_id(), Authentication::Full, None)
        .await;
    let entity = result.expect("find_by_id should succeed");
    assert_eq!(entity.id, default_logical_id());
    assert_eq!(entity.sales_person_id, default_sales_person_id());
}

#[tokio::test]
async fn test_find_by_id_unknown_returns_not_found() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service
        .find_by_id(unknown_logical_id(), Authentication::Full, None)
        .await;
    test_not_found(&result, &unknown_logical_id());
}

#[tokio::test]
async fn test_find_by_id_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .find_by_id(default_logical_id(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

// =========================================================================
// find_by_sales_person
// =========================================================================

#[tokio::test]
async fn test_find_by_sales_person_self_succeeds() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_sales_person()
        .with(eq(default_sales_person_id()), always())
        .returning(|_, _| Ok(Arc::from([])));
    let service = deps.build_service();

    let result = service
        .find_by_sales_person(default_sales_person_id(), Authentication::Full, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_by_sales_person_other_without_permission_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service
        .find_by_sales_person(other_sales_person_id(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

// =========================================================================
// find_all
// =========================================================================

#[tokio::test]
async fn test_find_all_hr_succeeds() {
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_all()
        .returning(|_| Ok(Arc::from([])));
    let service = deps.build_service();

    let result = service.find_all(Authentication::Full, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_all_non_hr_is_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service.find_all(Authentication::Full, None).await;
    test_forbidden(&result);
}

// =========================================================================
// Phase 3 — Forward-Warning-Tests (Plan 03-06 / BOOK-01)
//
// Tests den Forward-Warning-Loop in `compute_forward_warnings`, der NACH
// dem DAO-Persist von `create`/`update` läuft. Pro Booking-Tag in der
// neuen Range entsteht eine `Warning::AbsenceOverlapsBooking`; pro
// überlappendem ManualUnavailable eine
// `Warning::AbsenceOverlapsManualUnavailable`. D-Phase3-15: keine De-Dup.
// =========================================================================

use service::booking::Booking;
use service::sales_person_unavailable::SalesPersonUnavailable;
use service::warning::Warning;

/// Booking auf 2026-04-13 (W16 Mon) — liegt INNERHALB von
/// `default_create_request()` und `default_update_request()` (beide Ranges
/// schließen W16 Mon ein).
fn fixture_booking_in_range() -> Booking {
    Booking {
        id: uuid!("BB000000-0000-0000-0000-0000000000A1"),
        sales_person_id: default_sales_person_id(),
        slot_id: uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380"),
        calendar_week: 16,
        year: 2026,
        created: Some(datetime!(2026 - 04 - 01 12:00:00)),
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: uuid!("BB000000-0000-0000-0000-0000000000B1"),
    }
}

/// ManualUnavailable auf 2026-04-13 (W16 Mon) — liegt INNERHALB der
/// Default-Create-Range.
fn fixture_manual_unavailable_in_range() -> SalesPersonUnavailable {
    SalesPersonUnavailable {
        id: uuid!("CC000000-0000-0000-0000-0000000000A1"),
        sales_person_id: default_sales_person_id(),
        year: 2026,
        calendar_week: 16,
        day_of_week: DayOfWeek::Monday,
        created: Some(datetime!(2026 - 04 - 01 12:00:00)),
        deleted: None,
        version: uuid!("CC000000-0000-0000-0000-0000000000B1"),
    }
}

#[tokio::test]
async fn test_create_warning_for_booking_in_range() {
    // SC1 / Forward-Warning auf einem Booking in der neuen Range.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());

    // BookingService liefert pro Wochenaufruf je 1 Booking auf Monday.
    // Forward-Warning-Loop iteriert über die Range; W15 hat den Sonntag,
    // W16 hat Mon-Wed. Slot ist Monday → nur W16-Mon matcht.
    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_get_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_booking_in_range()])));

    let service = deps.build_service();
    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await
        .expect("create should succeed");

    assert!(
        !result.warnings.is_empty(),
        "expected at least one forward warning, got 0"
    );
    let any_booking_warning = result.warnings.iter().any(|w| {
        matches!(
            w,
            Warning::AbsenceOverlapsBooking { booking_id, date, .. }
                if *booking_id == fixture_booking_in_range().id
                    && *date == date!(2026 - 04 - 13)
        )
    });
    assert!(
        any_booking_warning,
        "expected AbsenceOverlapsBooking with booking-id + date 2026-04-13, got {:?}",
        result.warnings
    );
    // absence_id muss auf die NEU erstellte AbsencePeriod zeigen.
    for w in result.warnings.iter() {
        if let Warning::AbsenceOverlapsBooking { absence_id, .. } = w {
            assert_eq!(*absence_id, alternate_physical_id());
        }
    }
}

#[tokio::test]
async fn test_create_warning_for_manual_unavailable_in_range() {
    // Forward-Warning bei ManualUnavailable in der neuen Range.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());

    deps.sales_person_unavailable_service.checkpoint();
    deps.sales_person_unavailable_service
        .expect_get_all_for_sales_person()
        .returning(|_, _, _| Ok(Arc::from(vec![fixture_manual_unavailable_in_range()])));

    let service = deps.build_service();
    let result = service
        .create(&default_create_request(), Authentication::Full, None)
        .await
        .expect("create should succeed");

    let manual_warning = result.warnings.iter().find(|w| {
        matches!(
            w,
            Warning::AbsenceOverlapsManualUnavailable { unavailable_id, .. }
                if *unavailable_id == fixture_manual_unavailable_in_range().id
        )
    });
    assert!(
        manual_warning.is_some(),
        "expected AbsenceOverlapsManualUnavailable, got {:?}",
        result.warnings
    );
    if let Some(Warning::AbsenceOverlapsManualUnavailable { absence_id, .. }) = manual_warning {
        assert_eq!(*absence_id, alternate_physical_id());
    }
}

#[tokio::test]
async fn test_update_returns_warnings_for_full_new_range() {
    // D-Phase3-04: update-Warnings für ALLE Tage der NEUEN Range, kein
    // Diff-Modus. Update erweitert Range bis 2026-04-20 (W17 Mon) — Booking
    // auf W16 Mon UND Booking auf W17 Mon würden beide warnen, falls beide
    // im Mock vorhanden. Hier: 1 Booking auf W16 Mon + 1 weiteres auf W17
    // Mon → 2 Forward-Warnings.
    let mut deps = build_dependencies();
    deps.absence_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.absence_dao
        .expect_find_overlapping()
        .returning(|_, _, _, _, _| Ok(Arc::from([])));
    deps.absence_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.absence_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());

    deps.booking_service.checkpoint();
    deps.booking_service
        .expect_get_for_week()
        .returning(|week, year, _, _| {
            // W16 → Booking-Mon; W17 → weiteres Booking-Mon.
            let booking = Booking {
                id: uuid!("BB000000-0000-0000-0000-0000000000C0"),
                sales_person_id: default_sales_person_id(),
                slot_id: uuid!("7A7FF57A-782B-4C2E-A68B-4E2D81D79380"),
                calendar_week: week as i32,
                year,
                created: Some(datetime!(2026 - 04 - 01 12:00:00)),
                deleted: None,
                created_by: None,
                deleted_by: None,
                version: Uuid::nil(),
            };
            if year == 2026 && (week == 16 || week == 17) {
                Ok(Arc::from(vec![booking]))
            } else {
                Ok(Arc::from(Vec::<Booking>::new()))
            }
        });

    let service = deps.build_service();
    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await
        .expect("update should succeed");

    let booking_warnings: Vec<&Warning> = result
        .warnings
        .iter()
        .filter(|w| matches!(w, Warning::AbsenceOverlapsBooking { .. }))
        .collect();
    assert_eq!(
        booking_warnings.len(),
        2,
        "expected 2 booking-warnings (W16 Mon + W17 Mon — full new range, no diff), got {:?}",
        result.warnings
    );
    // absence_id muss auf den logical_id (= default_logical_id) zeigen
    // (D-07: stable über Updates).
    for w in booking_warnings.iter() {
        if let Warning::AbsenceOverlapsBooking { absence_id, .. } = w {
            assert_eq!(*absence_id, default_logical_id());
        }
    }
}

#[tokio::test]
async fn test_find_overlapping_for_booking_forbidden() {
    // D-Phase3-12: Permission HR ∨ verify_user_is_sales_person — beide
    // Pfade Forbidden → Forbidden propagiert.
    use shifty_utils::DateRange;

    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(ServiceError::Forbidden));

    let service = deps.build_service();
    let range = DateRange::new(date!(2026 - 04 - 12), date!(2026 - 04 - 15)).unwrap();
    let result = service
        .find_overlapping_for_booking(default_sales_person_id(), range, Authentication::Full, None)
        .await;
    test_forbidden(&result);
}
