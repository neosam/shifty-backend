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
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::MockSalesPersonService;
use service::special_days::MockSpecialDayService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError, ValidationFailureItem};
use time::macros::{date, datetime};
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

pub(crate) struct AbsenceDependencies {
    pub absence_dao: MockAbsenceDao,
    pub permission_service: MockPermissionService,
    pub sales_person_service: MockSalesPersonService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub special_day_service: MockSpecialDayService,
    pub employee_work_details_service: MockEmployeeWorkDetailsService,
    pub transaction_dao: MockTransactionDao,
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

    AbsenceDependencies {
        absence_dao,
        permission_service,
        sales_person_service,
        clock_service,
        uuid_service,
        special_day_service,
        employee_work_details_service,
        transaction_dao,
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
    let created = result.expect("create should succeed");
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
    let updated = result.expect("update should succeed");
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
